//! Wi-Fi popup (click Wi-Fi in the bar): switch, network list, password entry. Uses nmcli.

use crate::{conf::Conf, ui};
use gtk::gdk;
use gtk::glib::signal::Inhibit;
use gtk::prelude::*;
use gtk::{glib, pango};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::thread;
use std::time::Duration;

const CSS: &str = r#"
window { background: transparent; }
* { font-family: monospace, "Font Awesome 6 Free", sans-serif; font-size: 14px; color: %TEXT%; }
.card { background: %BG%; border: 5px solid %BG%; border-radius: 6px; padding: 8px; }
.title { color: %GOLD%; font-weight: bold; }
.sep { background: %BORDER%; min-height: 1px; margin: 6px 0; }
button { background: transparent; border: none; box-shadow: none; text-shadow: none; padding: 6px 8px; border-radius: 4px; }
button:hover { background: alpha(%GOLD%, 0.18); }
.row.active { background: %PILL%; }
.bars { color: %GOLD%; }
.dim { color: %DIM%; font-size: 12px; }
.err { color: %ERR%; font-size: 12px; }
entry { background: %PILL%; border: 1px solid %BORDER%; border-radius: 4px; padding: 4px 8px; box-shadow: none; }
entry:focus { border-color: %GOLD%; }
.connect { background: %GOLD%; }
.connect label { color: %BG%; font-weight: bold; }
.connect:hover { background: alpha(%GOLD%, 0.8); }
scrolledwindow undershoot, scrolledwindow overshoot { background: none; }
"#;

#[derive(Clone)]
struct Net {
    active: bool,
    signal: i32,
    secure: bool,
    ssid: String,
    saved: bool,
}

fn nm(args: &[&str]) -> (i32, String) {
    ui::run_text("nmcli", args, Duration::from_secs(40))
}

fn split_terse(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut esc = false;
    for ch in line.chars() {
        if esc {
            cur.push(ch);
            esc = false;
        } else if ch == '\\' {
            esc = true;
        } else if ch == ':' {
            out.push(std::mem::take(&mut cur));
        } else {
            cur.push(ch);
        }
    }
    out.push(cur);
    out
}

fn get_networks() -> (bool, Vec<Net>) {
    // the three queries run in parallel: much faster than one after another
    let (radio, saved_out, out) = thread::scope(|s| {
        let a = s.spawn(|| nm(&["radio", "wifi"]).1);
        let b = s.spawn(|| nm(&["-t", "-f", "NAME,TYPE", "con", "show"]).1);
        let c = s.spawn(|| {
            nm(&["-t", "-f", "IN-USE,SIGNAL,SECURITY,SSID", "dev", "wifi", "list", "--rescan", "no"]).1
        });
        (a.join().unwrap_or_default(), b.join().unwrap_or_default(), c.join().unwrap_or_default())
    });
    let on = radio.trim() == "enabled";
    if !on {
        return (on, Vec::new());
    }
    let saved: Vec<String> = saved_out
        .lines()
        .filter(|l| l.contains("wireless"))
        .map(|l| split_terse(l).remove(0))
        .collect();
    let mut nets: Vec<Net> = Vec::new();
    let mut index: HashMap<String, usize> = HashMap::new();
    for line in out.lines() {
        let f = split_terse(line);
        if f.len() < 4 || f[3].is_empty() {
            continue;
        }
        let net = Net {
            active: f[0].trim() == "*",
            signal: f[1].parse().unwrap_or(0),
            secure: !(f[2].is_empty() || f[2] == "--"),
            ssid: f[3].clone(),
            saved: saved.contains(&f[3]),
        };
        match index.get(&net.ssid) {
            None => {
                index.insert(net.ssid.clone(), nets.len());
                nets.push(net);
            }
            Some(&i) => {
                let old = &nets[i];
                if net.active || (!old.active && net.signal > old.signal) {
                    nets[i] = net;
                }
            }
        }
    }
    nets.sort_by_key(|n| (!n.active, -n.signal));
    (on, nets)
}

#[derive(Clone)]
struct RowUi {
    rev: gtk::Revealer,
    erow: gtk::Box,
    entry: gtk::Entry,
    go: gtk::Button,
    status: gtk::Label,
}

struct Wifi {
    list: gtk::Box,
    switch: gtk::Switch,
    open_rev: RefCell<Option<gtk::Revealer>>,
    setting: Cell<bool>,
}

impl Wifi {
    fn refresh(self: &Rc<Self>, rescan: bool) {
        let me = self.clone();
        ui::spawn_bg(
            move || {
                if rescan {
                    nm(&["dev", "wifi", "rescan"]);
                    thread::sleep(Duration::from_secs(2)); // give the scan a moment to finish
                }
                get_networks()
            },
            move |res| me.populate(res),
        );
    }

    fn on_switch(self: &Rc<Self>, want: bool) {
        let me = self.clone();
        ui::spawn_bg(
            move || {
                if want {
                    let _ = ui::run_bytes("rfkill", &["unblock", "wifi"], Duration::from_secs(10));
                    nm(&["radio", "wifi", "on"]);
                    thread::sleep(Duration::from_millis(1500));
                    nm(&["dev", "wifi", "rescan"]);
                    thread::sleep(Duration::from_millis(1500));
                } else {
                    nm(&["radio", "wifi", "off"]);
                }
                get_networks()
            },
            move |res| me.populate(res),
        );
    }

    fn populate(self: &Rc<Self>, res: (bool, Vec<Net>)) {
        if self.open_rev.borrow().is_some() {
            return; // don't wipe a password box while typing
        }
        let (on, nets) = res;
        self.setting.set(true);
        self.switch.set_active(on);
        self.setting.set(false);
        for c in self.list.children() {
            self.list.remove(&c);
        }
        if !on {
            self.list.pack_start(&ui::note("Wi-Fi is turned off"), false, false, 8);
        } else if nets.is_empty() {
            self.list.pack_start(&ui::note("No networks found"), false, false, 8);
        }
        for n in nets {
            let row = self.make_row(n);
            self.list.pack_start(&row, false, false, 0);
        }
        self.list.show_all();
    }

    fn make_row(self: &Rc<Self>, n: Net) -> gtk::Box {
        let wrap = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let btn = gtk::Button::new();
        ui::class(&btn, "row");
        if n.active {
            ui::class(&btn, "active");
        }
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        let bars_chars = ['\u{2582}', '\u{2584}', '\u{2586}', '\u{2588}'];
        let bars = gtk::Label::new(Some(&bars_chars[((n.signal / 25).clamp(0, 3)) as usize].to_string()));
        ui::class(&bars, "bars");
        row.pack_start(&bars, false, false, 0);
        let name = ui::label(&n.ssid);
        name.set_ellipsize(pango::EllipsizeMode::End);
        row.pack_start(&name, true, true, 0);
        if n.active {
            let tag = gtk::Label::new(Some("connected"));
            ui::class(&tag, "dim");
            row.pack_end(&tag, false, false, 0);
        } else if n.secure {
            let lock = gtk::Label::new(Some("\u{f023}"));
            ui::class(&lock, "dim");
            row.pack_end(&lock, false, false, 0);
        }
        btn.add(&row);
        wrap.pack_start(&btn, false, false, 0);

        // password / status area directly below this row
        let rev = gtk::Revealer::new();
        rev.set_transition_type(gtk::RevealerTransitionType::SlideDown);
        rev.set_transition_duration(120);
        let inner = gtk::Box::new(gtk::Orientation::Vertical, 4);
        inner.set_margin_start(8);
        inner.set_margin_end(8);
        inner.set_margin_top(4);
        inner.set_margin_bottom(6);
        let erow = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let entry = gtk::Entry::new();
        entry.set_visibility(false);
        entry.set_placeholder_text(Some("Password"));
        let go = gtk::Button::with_label("Connect");
        ui::class(&go, "connect");
        erow.pack_start(&entry, true, true, 0);
        erow.pack_start(&go, false, false, 0);
        erow.set_no_show_all(true); // hidden until a password is needed
        let status = gtk::Label::new(Some(""));
        status.set_xalign(0.0);
        ui::class(&status, "dim");
        status.set_line_wrap(true);
        inner.pack_start(&erow, false, false, 0);
        inner.pack_start(&status, false, false, 0);
        rev.add(&inner);
        wrap.pack_start(&rev, false, false, 0);

        let rui = RowUi { rev, erow, entry: entry.clone(), go: go.clone(), status };
        {
            let (me, n, u) = (self.clone(), n.clone(), rui.clone());
            btn.connect_clicked(move |_| me.on_row(&n, &u));
        }
        {
            let (me, n, u) = (self.clone(), n.clone(), rui.clone());
            entry.connect_activate(move |_| me.submit(&n, &u));
        }
        {
            let (me, n, u) = (self.clone(), n, rui);
            go.connect_clicked(move |_| me.submit(&n, &u));
        }
        wrap
    }

    fn show_password(u: &RowUi) {
        // GTK3 ignores show_all() on a no_show_all widget, so clear the flag first
        u.erow.set_no_show_all(false);
        u.erow.show_all();
        u.entry.grab_focus();
    }

    fn on_row(self: &Rc<Self>, n: &Net, u: &RowUi) {
        if n.active {
            return;
        }
        {
            let mut cur = self.open_rev.borrow_mut();
            if let Some(r) = cur.as_ref() {
                if *r != u.rev {
                    r.set_reveal_child(false);
                }
            }
            *cur = Some(u.rev.clone());
        }
        u.status.set_text("");
        u.rev.set_reveal_child(true);
        if n.secure && !n.saved {
            Self::show_password(u);
        } else {
            self.connect_to(n, None, u);
        }
    }

    fn submit(self: &Rc<Self>, n: &Net, u: &RowUi) {
        let pw = u.entry.text().to_string();
        if n.secure && pw.chars().count() < 8 {
            Self::set_status(u, "Password must be at least 8 characters", true);
            return;
        }
        self.connect_to(n, if pw.is_empty() { None } else { Some(pw) }, u);
    }

    fn set_status(u: &RowUi, text: &str, err: bool) {
        let ctx = u.status.style_context();
        ctx.remove_class("err");
        ctx.remove_class("dim");
        ctx.add_class(if err { "err" } else { "dim" });
        u.status.set_text(text);
    }

    fn connect_to(self: &Rc<Self>, n: &Net, pw: Option<String>, u: &RowUi) {
        Self::set_status(u, "Connecting\u{2026}", false);
        u.entry.set_sensitive(false);
        u.go.set_sensitive(false);
        let (n2, u2) = (n.clone(), u.clone());
        ui::spawn_bg(
            move || {
                let ssid = n2.ssid.as_str();
                if pw.is_some() && n2.saved {
                    nm(&["con", "delete", "id", ssid]); // replace the old saved password
                }
                let (rc, out) = if let Some(p) = &pw {
                    nm(&["dev", "wifi", "connect", ssid, "password", p.as_str()])
                } else if n2.saved {
                    nm(&["con", "up", "id", ssid])
                } else {
                    nm(&["dev", "wifi", "connect", ssid])
                };
                if rc != 0 {
                    nm(&["con", "delete", "id", ssid]); // don't leave a broken profile
                }
                (rc, out)
            },
            {
                let n = n.clone();
                move |(rc, _out)| {
                    if rc == 0 {
                        ui::spawn_detached(
                            "notify-send",
                            &["-t", "3000", "Wi-Fi", &format!("Connected to {}", n.ssid)],
                        );
                        gtk::main_quit();
                        return;
                    }
                    u2.entry.set_sensitive(true);
                    u2.go.set_sensitive(true);
                    if n.secure {
                        Self::set_status(&u2, "Could not connect. Check the password.", true);
                        Self::show_password(&u2);
                    } else {
                        Self::set_status(&u2, "Could not connect.", true);
                    }
                }
            },
        );
    }
}

pub fn run(conf: &Conf) {
    let sh = ui::shell(CSS, conf.int("wifi_width", 340), -1);
    sh.card.set_halign(gtk::Align::End);
    sh.card.set_valign(gtk::Align::Start);
    sh.card.set_margin_top(conf.int("top_margin", 52));
    sh.card.set_margin_end(conf.int("right_margin", 150));

    let head = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let title = ui::label("\u{f1eb}  Wi-Fi");
    ui::class(&title, "title");
    head.pack_start(&title, true, true, 4);
    let refresh = gtk::Button::with_label("\u{f021}");
    head.pack_start(&refresh, false, false, 0);
    let switch = gtk::Switch::new();
    switch.set_valign(gtk::Align::Center);
    head.pack_start(&switch, false, false, 4);
    sh.card.pack_start(&head, false, false, 0);
    sh.card.pack_start(&ui::sep(), false, false, 0);

    let scroll = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .build();
    scroll.set_propagate_natural_height(true);
    scroll.set_max_content_height(conf.int("wifi_max_height", 300));
    let list = gtk::Box::new(gtk::Orientation::Vertical, 2);
    list.pack_start(&ui::note("Scanning\u{2026}"), false, false, 8); // shown instantly
    scroll.add(&list);
    sh.card.pack_start(&scroll, false, false, 0);

    sh.card.pack_start(&ui::sep(), false, false, 0);
    let foot = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let close = gtk::Button::with_label("Close");
    close.connect_clicked(|_| gtk::main_quit());
    foot.pack_end(&close, false, false, 0);
    sh.card.pack_start(&foot, false, false, 0);

    let me = Rc::new(Wifi { list, switch: switch.clone(), open_rev: RefCell::new(None), setting: Cell::new(false) });
    {
        let me = me.clone();
        refresh.connect_clicked(move |_| me.refresh(true));
    }
    {
        let me = me.clone();
        switch.connect_active_notify(move |sw| {
            if !me.setting.get() {
                me.on_switch(sw.is_active());
            }
        });
    }
    sh.win.connect_key_press_event(|_, ev| {
        if ev.keyval() == gdk::keys::constants::Escape {
            gtk::main_quit();
            return Inhibit(true);
        }
        Inhibit(false)
    });
    sh.win.show_all();

    me.refresh(false); // cached list right away
    let m2 = me.clone();
    glib::timeout_add_local(Duration::from_millis(100), move || {
        m2.refresh(true); // then a fresh scan
        glib::ControlFlow::Break
    });
}
