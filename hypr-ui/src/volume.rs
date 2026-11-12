//! Volume popup (click the volume in the bar): output, input and per-app sliders. Uses pactl.

use crate::{conf::Conf, ui};
use gtk::gdk;
use gtk::glib::signal::Inhibit;
use gtk::prelude::*;
use gtk::{glib, pango};
use serde_json::Value;
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
.section { color: %GOLD%; font-size: 12px; font-weight: bold; }
.sep { background: %BORDER%; min-height: 1px; margin: 6px 0; }
button { background: transparent; border: none; box-shadow: none; text-shadow: none; padding: 6px 8px; border-radius: 4px; }
button:hover { background: alpha(%GOLD%, 0.18); }
.device.active { background: %PILL%; }
.muted label { color: %ERR%; }
.dim { color: %DIM%; font-size: 12px; }
.pct { color: %DIM%; font-size: 12px; }
scale trough { background: %PILL%; border: 1px solid %BORDER%; border-radius: 4px; min-height: 6px; }
scale highlight { background: %GOLD%; border-radius: 4px; border: none; }
scale slider { background: %TEXT%; border: none; box-shadow: none; border-radius: 8px; min-width: 14px; min-height: 14px; margin: -5px; }
scale slider:hover { background: %GOLD%; }
scale.muted highlight { background: %DIM%; }
scrolledwindow undershoot, scrolledwindow overshoot { background: none; }
"#;

fn pactl(args: &[&str]) -> (i32, String) {
    let (rc, out, err) = ui::run_bytes("pactl", args, Duration::from_secs(5));
    if rc != 0 && out.is_empty() {
        return (rc, String::from_utf8_lossy(&err).trim().to_string());
    }
    (rc, String::from_utf8_lossy(&out).trim().to_string())
}

fn jlist(kind: &str) -> Vec<Value> {
    let (rc, out) = pactl(&["-f", "json", "list", kind]);
    if rc != 0 {
        return Vec::new();
    }
    serde_json::from_str::<Value>(&out)
        .ok()
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default()
}

fn vol_pct(d: &Value) -> i32 {
    if let Some(o) = d.get("volume").and_then(|v| v.as_object()) {
        if let Some(v) = o.values().next() {
            let s = v.get("value_percent").and_then(|x| x.as_str()).unwrap_or("0%");
            return s.trim().trim_end_matches('%').parse().unwrap_or(0);
        }
    }
    0
}

fn text(v: &Value, key: &str) -> String {
    v.get(key).and_then(|x| x.as_str()).unwrap_or("").to_string()
}

#[derive(Clone)]
struct Dev {
    name: String,
    desc: String,
    vol: i32,
    mute: bool,
}

#[derive(Clone)]
struct App {
    id: String,
    label: String,
    vol: i32,
    mute: bool,
}

struct Audio {
    sinks: Vec<Dev>,
    sources: Vec<Dev>,
    apps: Vec<App>,
    dsink: String,
    dsrc: String,
}

fn dev_of(s: &Value) -> Dev {
    let name = text(s, "name");
    let desc = text(s, "description");
    Dev {
        desc: if desc.is_empty() { name.clone() } else { desc },
        name,
        vol: vol_pct(s),
        mute: s.get("mute").and_then(|m| m.as_bool()).unwrap_or(false),
    }
}

fn get_audio() -> Audio {
    // all queries in parallel
    let (sinks, srcs, inputs, dsink, dsrc) = thread::scope(|s| {
        let a = s.spawn(|| jlist("sinks"));
        let b = s.spawn(|| jlist("sources"));
        let c = s.spawn(|| jlist("sink-inputs"));
        let d = s.spawn(|| pactl(&["get-default-sink"]).1);
        let e = s.spawn(|| pactl(&["get-default-source"]).1);
        (
            a.join().unwrap_or_default(),
            b.join().unwrap_or_default(),
            c.join().unwrap_or_default(),
            d.join().unwrap_or_default(),
            e.join().unwrap_or_default(),
        )
    });
    let apps = inputs
        .iter()
        .map(|i| {
            let props = i.get("properties");
            let prop = |k: &str| -> String {
                props.and_then(|p| p.get(k)).and_then(|x| x.as_str()).unwrap_or("").to_string()
            };
            let mut label = prop("application.name");
            if label.is_empty() {
                label = prop("media.name");
            }
            if label.is_empty() {
                label = text(i, "name");
            }
            if label.is_empty() {
                label = "Stream".to_string();
            }
            App {
                id: i.get("index").map(|x| x.to_string()).unwrap_or_default(),
                label,
                vol: vol_pct(i),
                mute: i.get("mute").and_then(|m| m.as_bool()).unwrap_or(false),
            }
        })
        .collect();
    Audio {
        sinks: sinks.iter().map(dev_of).collect(),
        sources: srcs
            .iter()
            .map(dev_of)
            .filter(|d| !d.name.ends_with(".monitor"))
            .collect(),
        apps,
        dsink,
        dsrc,
    }
}

struct Vol {
    list: gtk::Box,
    dragging: Cell<bool>,
    sig: RefCell<Option<String>>,
    rows: RefCell<HashMap<(String, String), Rc<VolRow>>>,
    max: i32,
}

/// mute button + label + slider + percentage
struct VolRow {
    root: gtk::Box,
    mbtn: gtk::Button,
    pct: gtk::Label,
    scale: gtk::Scale,
    kind: String,
    ident: String,
    icon_on: &'static str,
    icon_off: &'static str,
    mute: Cell<bool>,
    updating: Cell<bool>,
    pending: RefCell<Option<glib::SourceId>>,
    popup: Rc<Vol>,
}

impl VolRow {
    fn new(
        popup: &Rc<Vol>,
        kind: &str,
        ident: &str,
        label: &str,
        vol: i32,
        mute: bool,
        icons: (&'static str, &'static str),
    ) -> Rc<VolRow> {
        let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let top = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let mbtn = gtk::Button::with_label(icons.0);
        top.pack_start(&mbtn, false, false, 0);
        let name = ui::label(label);
        name.set_ellipsize(pango::EllipsizeMode::End);
        top.pack_start(&name, true, true, 0);
        let pct = gtk::Label::new(Some("0%"));
        ui::class(&pct, "pct");
        top.pack_end(&pct, false, false, 4);
        root.pack_start(&top, false, false, 0);

        let scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, popup.max as f64, 1.0);
        scale.set_draw_value(false);
        scale.set_margin_start(10);
        scale.set_margin_end(10);
        scale.set_margin_bottom(6);
        scale.add_events(gdk::EventMask::BUTTON_PRESS_MASK | gdk::EventMask::BUTTON_RELEASE_MASK);
        root.pack_start(&scale, false, false, 0);

        let row = Rc::new(VolRow {
            root,
            mbtn,
            pct,
            scale,
            kind: kind.to_string(),
            ident: ident.to_string(),
            icon_on: icons.0,
            icon_off: icons.1,
            mute: Cell::new(mute),
            updating: Cell::new(false),
            pending: RefCell::new(None),
            popup: popup.clone(),
        });
        {
            let r = row.clone();
            row.mbtn.connect_clicked(move |_| r.on_mute());
        }
        {
            let r = row.clone();
            row.scale.connect_button_press_event(move |_, _| {
                r.popup.dragging.set(true);
                Inhibit(false)
            });
        }
        {
            let r = row.clone();
            row.scale.connect_button_release_event(move |_, _| {
                r.popup.dragging.set(false);
                Inhibit(false)
            });
        }
        {
            let r = row.clone();
            row.scale.connect_value_changed(move |s| r.on_changed(s));
        }
        row.set_state(vol, mute);
        row
    }

    fn set_state(&self, vol: i32, mute: bool) {
        self.updating.set(true);
        self.scale.set_value(vol as f64);
        self.updating.set(false);
        self.mute.set(mute);
        self.pct.set_text(&format!("{vol}%"));
        self.mbtn.set_label(if mute { self.icon_off } else { self.icon_on });
        for w in [self.mbtn.upcast_ref::<gtk::Widget>(), self.scale.upcast_ref::<gtk::Widget>()] {
            let ctx = w.style_context();
            if mute {
                ctx.add_class("muted");
            } else {
                ctx.remove_class("muted");
            }
        }
    }

    fn on_changed(self: &Rc<Self>, scale: &gtk::Scale) {
        let v = scale.value() as i32;
        self.pct.set_text(&format!("{v}%"));
        if self.updating.get() {
            return;
        }
        if let Some(id) = self.pending.borrow_mut().take() {
            id.remove();
        }
        // send at most one change every 40 ms while dragging
        let me = self.clone();
        let id = glib::timeout_add_local(Duration::from_millis(40), move || {
            *me.pending.borrow_mut() = None;
            ui::spawn_detached(
                "pactl",
                &[&format!("set-{}-volume", me.kind), me.ident.as_str(), &format!("{v}%")],
            );
            glib::ControlFlow::Break
        });
        *self.pending.borrow_mut() = Some(id);
    }

    fn on_mute(self: &Rc<Self>) {
        ui::spawn_detached(
            "pactl",
            &[&format!("set-{}-mute", self.kind), self.ident.as_str(), "toggle"],
        );
        self.set_state(self.scale.value() as i32, !self.mute.get()); // instant feedback
        self.popup.refresh_soon();
    }
}

fn section(text: &str) -> gtk::Label {
    let l = ui::label(&text.to_uppercase());
    ui::class(&l, "section");
    l.set_margin_start(8);
    l.set_margin_top(8);
    l.set_margin_bottom(2);
    l
}

impl Vol {
    fn refresh(self: &Rc<Self>) {
        let me = self.clone();
        ui::spawn_bg(get_audio, move |d| me.apply(d));
    }

    fn refresh_soon(self: &Rc<Self>) {
        let me = self.clone();
        glib::timeout_add_local(Duration::from_millis(200), move || {
            me.refresh();
            glib::ControlFlow::Break
        });
    }

    fn apply(self: &Rc<Self>, d: Audio) {
        if self.dragging.get() {
            return;
        }
        let sig = format!(
            "{}|{}|{}|{}|{}",
            d.dsink,
            d.dsrc,
            d.sinks.iter().map(|s| s.name.as_str()).collect::<Vec<_>>().join(","),
            d.sources.iter().map(|s| s.name.as_str()).collect::<Vec<_>>().join(","),
            d.apps.iter().map(|a| format!("{}={}", a.id, a.label)).collect::<Vec<_>>().join(","),
        );
        let changed = self.sig.borrow().as_deref() != Some(sig.as_str());
        if changed {
            *self.sig.borrow_mut() = Some(sig);
            self.rebuild(&d);
        } else {
            // same layout: just update values in place (keeps focus and avoids flicker)
            for ((kind, ident), row) in self.rows.borrow().iter() {
                let found = match kind.as_str() {
                    "sink" => d.sinks.iter().find(|s| &s.name == ident).map(|s| (s.vol, s.mute)),
                    "source" => d.sources.iter().find(|s| &s.name == ident).map(|s| (s.vol, s.mute)),
                    _ => d.apps.iter().find(|a| &a.id == ident).map(|a| (a.vol, a.mute)),
                };
                if let Some((v, m)) = found {
                    row.set_state(v, m);
                }
            }
        }
    }

    fn rebuild(self: &Rc<Self>, d: &Audio) {
        for c in self.list.children() {
            self.list.remove(&c);
        }
        let mut rows = HashMap::new();

        let cur = d.sinks.iter().find(|s| s.name == d.dsink).or_else(|| d.sinks.first());
        let cur_src = d.sources.iter().find(|s| s.name == d.dsrc).or_else(|| d.sources.first());

        // output
        self.list.pack_start(&section("Output"), false, false, 0);
        if let Some(c) = cur {
            let r = VolRow::new(self, "sink", &c.name, &c.desc, c.vol, c.mute, ("\u{f028}", "\u{f6a9}"));
            self.list.pack_start(&r.root, false, false, 0);
            rows.insert(("sink".to_string(), c.name.clone()), r);
        } else {
            self.list.pack_start(&ui::note("No output device"), false, false, 6);
        }
        if d.sinks.len() > 1 {
            for s in &d.sinks {
                let active = cur.map(|c| c.name == s.name).unwrap_or(false);
                self.list.pack_start(&self.device_button(s, active), false, false, 0);
            }
        }

        // input
        if let Some(c) = cur_src {
            self.list.pack_start(&section("Input"), false, false, 0);
            let r = VolRow::new(self, "source", &c.name, &c.desc, c.vol, c.mute, ("\u{f130}", "\u{f131}"));
            self.list.pack_start(&r.root, false, false, 0);
            rows.insert(("source".to_string(), c.name.clone()), r);
        }

        // apps
        self.list.pack_start(&section("Applications"), false, false, 0);
        if d.apps.is_empty() {
            self.list.pack_start(&ui::note("Nothing is playing"), false, false, 6);
        }
        for a in &d.apps {
            let r = VolRow::new(self, "sink-input", &a.id, &a.label, a.vol, a.mute, ("\u{f028}", "\u{f6a9}"));
            self.list.pack_start(&r.root, false, false, 0);
            rows.insert(("sink-input".to_string(), a.id.clone()), r);
        }
        *self.rows.borrow_mut() = rows;
        self.list.show_all();
    }

    fn device_button(self: &Rc<Self>, s: &Dev, active: bool) -> gtk::Button {
        let btn = gtk::Button::new();
        ui::class(&btn, "device");
        if active {
            ui::class(&btn, "active");
        }
        let b = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let mark = gtk::Label::new(Some(if active { "\u{f00c}" } else { " " }));
        ui::class(&mark, "dim");
        b.pack_start(&mark, false, false, 0);
        let name = ui::label(&s.desc);
        name.set_ellipsize(pango::EllipsizeMode::End);
        b.pack_start(&name, true, true, 0);
        btn.add(&b);
        let (me, dev) = (self.clone(), s.name.clone());
        btn.connect_clicked(move |_| me.set_default(&dev));
        btn
    }

    fn set_default(self: &Rc<Self>, name: &str) {
        *self.sig.borrow_mut() = None;
        let name = name.to_string();
        let me = self.clone();
        ui::spawn_bg(
            move || {
                pactl(&["set-default-sink", &name]);
                // move playing streams to the new device too
                for i in jlist("sink-inputs") {
                    let idx = i.get("index").map(|x| x.to_string()).unwrap_or_default();
                    pactl(&["move-sink-input", &idx, &name]);
                }
                get_audio()
            },
            move |d| me.apply(d),
        );
    }
}

pub fn run(conf: &Conf) {
    let sh = ui::shell(CSS, conf.int("volume_width", 360), -1);
    sh.card.set_halign(gtk::Align::End);
    sh.card.set_valign(gtk::Align::Start);
    sh.card.set_margin_top(conf.int("top_margin", 52));
    sh.card.set_margin_end(conf.int("right_margin", 150));

    let head = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let title = ui::label("\u{f028}  Volume");
    ui::class(&title, "title");
    head.pack_start(&title, true, true, 4);
    sh.card.pack_start(&head, false, false, 0);
    sh.card.pack_start(&ui::sep(), false, false, 0);

    let scroll = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .build();
    scroll.set_propagate_natural_height(true);
    scroll.set_max_content_height(conf.int("volume_max_height", 420));
    let list = gtk::Box::new(gtk::Orientation::Vertical, 2);
    list.pack_start(&ui::note("Loading\u{2026}"), false, false, 8);
    scroll.add(&list);
    sh.card.pack_start(&scroll, false, false, 0);

    sh.card.pack_start(&ui::sep(), false, false, 0);
    let foot = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let close = gtk::Button::with_label("Close");
    close.connect_clicked(|_| gtk::main_quit());
    foot.pack_end(&close, false, false, 0);
    sh.card.pack_start(&foot, false, false, 0);

    let me = Rc::new(Vol {
        list,
        dragging: Cell::new(false),
        sig: RefCell::new(None),
        rows: RefCell::new(HashMap::new()),
        max: conf.int("volume_max", 100),
    });
    sh.win.connect_key_press_event(|_, ev| {
        if ev.keyval() == gdk::keys::constants::Escape {
            gtk::main_quit();
            return Inhibit(true);
        }
        Inhibit(false)
    });
    sh.win.show_all();

    me.refresh();
    let m2 = me.clone();
    glib::timeout_add_local(Duration::from_millis(1500), move || {
        // keep in sync with changes made elsewhere
        if !m2.dragging.get() {
            m2.refresh();
        }
        glib::ControlFlow::Continue
    });
}
