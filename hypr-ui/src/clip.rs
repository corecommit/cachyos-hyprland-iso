//! Clipboard history popup (Super+C): search, pick, delete, thumbnails. Uses cliphist + wl-copy.

use crate::{conf::Conf, ui};
use gtk::gdk;
use gtk::gdk_pixbuf::{Colorspace, InterpType, Pixbuf};
use gtk::glib::signal::Inhibit;
use gtk::prelude::*;
use gtk::{glib, pango};
use serde_json::Value;
use std::cell::{Cell, RefCell};
use std::process::Command;
use std::rc::Rc;
use std::time::Duration;

const THUMB_W: f64 = 360.0;
const THUMB_H: f64 = 110.0;

const CSS: &str = r#"
window { background: transparent; }
* { font-family: monospace, "Font Awesome 6 Free", sans-serif; font-size: 14px; color: %TEXT%; }
.card { background: %BG%; border: 5px solid %BG%; border-radius: 6px; padding: 8px; }
.title { color: %GOLD%; font-weight: bold; }
.sep { background: %BORDER%; min-height: 1px; margin: 6px 0; }
button { background: transparent; border: none; box-shadow: none; text-shadow: none; padding: 6px 8px; border-radius: 4px; }
button:hover { background: alpha(%GOLD%, 0.18); }
.del label { color: %DIM%; font-size: 12px; }
.del:hover label { color: %ERR%; }
.confirm label { color: %ERR%; }
.dim { color: %DIM%; font-size: 12px; }
entry { background: %PILL%; border: 1px solid %BORDER%; border-radius: 4px; padding: 4px 8px; box-shadow: none; }
entry:focus { border-color: %GOLD%; }
viewport { background: transparent; }
list { background: transparent; }
list row { padding: 6px 8px; border-radius: 4px; }
list row:hover { background: alpha(%GOLD%, 0.18); }
list row:selected { background: %PILL%; }
scrolledwindow undershoot, scrolledwindow overshoot { background: none; }
"#;

const TERMINALS: [&str; 8] =
    ["kitty", "foot", "alacritty", "wezterm", "ghostty", "konsole", "xterm", "gnome-terminal"];

fn hyprctl_json(what: &str) -> Option<Value> {
    let (rc, out, _) = ui::run_bytes("hyprctl", &["-j", what], Duration::from_secs(2));
    if rc != 0 {
        return None;
    }
    serde_json::from_slice(&out).ok()
}

/// Pointer position relative to its monitor: (x, y, monitor_w, monitor_h), or None.
fn pointer_anchor() -> Option<(f64, f64, f64, f64)> {
    let c = hyprctl_json("cursorpos")?;
    let mons = hyprctl_json("monitors")?;
    let (cx, cy) = (c["x"].as_f64()?, c["y"].as_f64()?);
    for m in mons.as_array()? {
        let mut w = m["width"].as_f64().unwrap_or(0.0);
        let mut h = m["height"].as_f64().unwrap_or(0.0);
        if m["transform"].as_i64().unwrap_or(0) % 2 != 0 {
            std::mem::swap(&mut w, &mut h);
        }
        let sc = m["scale"].as_f64().unwrap_or(1.0);
        let (w, h) = (w / sc, h / sc);
        let (mx, my) = (m["x"].as_f64().unwrap_or(0.0), m["y"].as_f64().unwrap_or(0.0));
        if mx <= cx && cx < mx + w && my <= cy && cy < my + h {
            return Some((cx - mx, cy - my, w, h));
        }
    }
    None
}

/// Press Ctrl+V (Ctrl+Shift+V in terminals) in the window you came from.
fn auto_paste() {
    let cls = hyprctl_json("activewindow")
        .and_then(|w| w["class"].as_str().map(|s| s.to_lowercase()))
        .unwrap_or_default();
    if cls.is_empty() {
        return;
    }
    let keys: &[&str] = if TERMINALS.iter().any(|t| cls.contains(t)) {
        &["-M", "ctrl", "-M", "shift", "-k", "v", "-m", "shift", "-m", "ctrl"]
    } else {
        &["-M", "ctrl", "-k", "v", "-m", "ctrl"]
    };
    // short delay: the popup must close and give focus back first
    let mut args: Vec<&str> = vec!["-c", "sleep 0.2; exec wtype \"$@\"", "wtype"];
    args.extend_from_slice(keys);
    ui::spawn_detached("sh", &args);
}

/// "[[ binary data 1.2 MiB png 800x600 ]]" -> ("1.2 MiB", "png", "800x600")
fn parse_img(prev: &str) -> Option<(String, String, String)> {
    let rest = prev.strip_prefix("[[ binary data ")?;
    let end = rest.find(" ]]")?;
    let toks: Vec<&str> = rest[..end].split(' ').collect();
    if toks.len() < 3 {
        return None;
    }
    let dims = toks[toks.len() - 1];
    let ext = toks[toks.len() - 2];
    let (a, b) = dims.split_once('x')?;
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|c| c.is_ascii_digit());
    if !digits(a) || !digits(b) {
        return None;
    }
    if !matches!(ext, "png" | "jpg" | "jpeg" | "bmp" | "webp" | "gif" | "tif" | "tiff") {
        return None;
    }
    Some((toks[..toks.len() - 2].join(" "), ext.to_string(), dims.to_string()))
}

struct Thumb {
    data: Vec<u8>,
    alpha: bool,
    w: i32,
    h: i32,
    stride: i32,
}

/// Runs on a worker thread: fetch + decode + shrink. Only plain bytes go back to the GTK thread.
fn load_thumb(cid: &str) -> Option<Thumb> {
    let (rc, data, _) = ui::run_bytes("cliphist", &["decode", cid], Duration::from_secs(10));
    if rc != 0 {
        return None;
    }
    let stream = gtk::gio::MemoryInputStream::from_bytes(&glib::Bytes::from(&data[..]));
    let pb = Pixbuf::from_stream(&stream, gtk::gio::Cancellable::NONE).ok()?;
    let (w, h) = (pb.width(), pb.height());
    if w <= 0 || h <= 0 {
        return None;
    }
    let r = (THUMB_W / w as f64).min(THUMB_H / h as f64).min(1.0);
    let pb = if r < 1.0 {
        pb.scale_simple(
            ((w as f64 * r) as i32).max(1),
            ((h as f64 * r) as i32).max(1),
            InterpType::Bilinear,
        )?
    } else {
        pb
    };
    Some(Thumb {
        data: pb.read_pixel_bytes().to_vec(),
        alpha: pb.has_alpha(),
        w: pb.width(),
        h: pb.height(),
        stride: pb.rowstride(),
    })
}

struct Item {
    row: gtk::ListBoxRow,
    line: String,
    cid: String,
    mime: Option<String>,
    search: String,
}

struct Clip {
    list: gtk::ListBox,
    scroll: gtk::ScrolledWindow,
    search: gtk::Entry,
    note: gtk::Label,
    items: RefCell<Vec<Rc<Item>>>,
    wipe_armed: Cell<bool>,
    pool: ui::Pool,
    max_items: usize,
}

impl Clip {
    fn make_item(self: &Rc<Self>, line: &str) -> Rc<Item> {
        let (cid, prev) = match line.split_once('\t') {
            Some((a, b)) => (a.to_string(), b.trim().to_string()),
            None => (line.to_string(), String::new()),
        };
        let outer = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let body = gtk::Box::new(gtk::Orientation::Vertical, 4);
        let mut mime = None;
        let search;

        if let Some((size, ext, dims)) = parse_img(&prev) {
            let ext = if ext == "jpg" { "jpeg".to_string() } else { ext };
            mime = Some(format!("image/{ext}"));
            search = format!("image {ext} {dims} {size}").to_lowercase();
            let img = gtk::Image::new();
            img.set_halign(gtk::Align::Start);
            img.set_size_request(-1, 40);
            body.pack_start(&img, false, false, 0);
            let cap = ui::label(&format!("image  \u{b7}  {ext}  \u{b7}  {dims}  \u{b7}  {size}"));
            ui::class(&cap, "dim");
            body.pack_start(&cap, false, false, 0);
            let c = cid.clone();
            self.pool.submit(
                move || load_thumb(&c),
                move |t| {
                    if let Some(t) = t {
                        let pb = Pixbuf::from_mut_slice(t.data, Colorspace::Rgb, t.alpha, 8, t.w, t.h, t.stride);
                        img.set_from_pixbuf(Some(&pb));
                        img.set_size_request(-1, -1);
                    }
                },
            );
        } else {
            search = prev.to_lowercase();
            let lab = ui::label(if prev.is_empty() { "(empty)" } else { prev.as_str() });
            lab.set_ellipsize(pango::EllipsizeMode::End);
            lab.set_single_line_mode(true);
            body.pack_start(&lab, false, false, 0);
        }
        outer.pack_start(&body, true, true, 0);

        let del = gtk::Button::with_label("\u{f00d}");
        ui::class(&del, "del");
        del.set_can_focus(false);
        del.set_valign(gtk::Align::Center);
        outer.pack_end(&del, false, false, 0);

        let row = gtk::ListBoxRow::new();
        row.add(&outer);
        let item = Rc::new(Item { row, line: line.to_string(), cid, mime, search });
        {
            let (me, it) = (self.clone(), item.clone());
            del.connect_clicked(move |_| me.delete(&it));
        }
        item
    }

    fn reload(self: &Rc<Self>) {
        let me = self.clone();
        ui::spawn_bg(
            || {
                let (_, out, _) = ui::run_bytes("cliphist", &["list"], Duration::from_secs(10));
                String::from_utf8_lossy(&out)
                    .lines()
                    .filter(|l| !l.trim().is_empty())
                    .map(|l| l.to_string())
                    .collect::<Vec<String>>()
            },
            move |lines| me.populate(lines),
        );
    }

    fn populate(self: &Rc<Self>, lines: Vec<String>) {
        for c in self.list.children() {
            self.list.remove(&c);
        }
        let mut items = Vec::new();
        for line in lines.iter().take(self.max_items) {
            let it = self.make_item(line);
            self.list.add(&it.row);
            items.push(it);
        }
        *self.items.borrow_mut() = items;
        self.list.show_all();
        self.on_search();
    }

    fn visible(&self) -> Vec<Rc<Item>> {
        self.items.borrow().iter().filter(|i| i.row.is_visible()).cloned().collect()
    }

    fn update_note(&self) {
        if !self.visible().is_empty() {
            self.note.hide();
        } else {
            let empty = self.search.text().is_empty();
            self.note.set_text(if empty { "Nothing copied yet" } else { "No matches" });
            self.note.show();
        }
    }

    fn on_search(self: &Rc<Self>) {
        let q = self.search.text().trim().to_lowercase();
        for it in self.items.borrow().iter() {
            it.row.set_visible(it.search.contains(&q));
        }
        if let Some(first) = self.visible().first() {
            self.select(&first.row);
        }
        self.update_note();
    }

    fn select(self: &Rc<Self>, row: &gtk::ListBoxRow) {
        self.list.select_row(Some(row));
        let (me, row) = (self.clone(), row.clone());
        glib::idle_add_local(move || {
            let a = row.allocation();
            let adj = me.scroll.vadjustment();
            let (y, bottom) = (a.y() as f64, (a.y() + a.height()) as f64);
            if y < adj.value() {
                adj.set_value(y);
            } else if bottom > adj.value() + adj.page_size() {
                adj.set_value(bottom - adj.page_size());
            }
            glib::ControlFlow::Break
        });
    }

    fn move_sel(self: &Rc<Self>, d: i32) {
        let rows = self.visible();
        if rows.is_empty() {
            return;
        }
        let cur = self.list.selected_row();
        let i = cur
            .and_then(|c| rows.iter().position(|r| r.row == c))
            .map(|p| p as i32)
            .unwrap_or(-1);
        let n = (i + d).clamp(0, rows.len() as i32 - 1) as usize;
        self.select(&rows[n].row);
    }

    fn pick(&self, row: Option<gtk::ListBoxRow>) {
        let Some(row) = row else { return };
        let found = self.items.borrow().iter().find(|i| i.row == row).cloned();
        if let Some(it) = found {
            let (rc, data, _) = ui::run_bytes("cliphist", &["decode", &it.cid], Duration::from_secs(10));
            if rc == 0 {
                let mut args: Vec<&str> = Vec::new();
                if let Some(m) = &it.mime {
                    args.push("--type");
                    args.push(m);
                }
                ui::spawn_detached_stdin("wl-copy", &args, &data);
                auto_paste();
            }
        }
        gtk::main_quit();
    }

    fn delete(self: &Rc<Self>, item: &Rc<Item>) {
        if !ui::run_input("cliphist", &["delete"], item.line.as_bytes()) {
            return;
        }
        let rows = self.visible();
        let mut nxt: Option<Rc<Item>> = None;
        if let Some(i) = rows.iter().position(|r| Rc::ptr_eq(r, item)) {
            let rest: Vec<&Rc<Item>> = rows.iter().enumerate().filter(|(k, _)| *k != i).map(|(_, r)| r).collect();
            if !rest.is_empty() {
                nxt = Some(rest[i.min(rest.len() - 1)].clone());
            }
        }
        self.list.remove(&item.row);
        self.items.borrow_mut().retain(|i| !Rc::ptr_eq(i, item));
        if let Some(n) = nxt {
            self.select(&n.row);
        }
        self.update_note();
    }

    fn on_wipe(self: &Rc<Self>, btn: &gtk::Button) {
        if !self.wipe_armed.get() {
            self.wipe_armed.set(true);
            btn.set_label("Click again to confirm");
            ui::class(btn, "confirm");
            return;
        }
        let _ = Command::new("cliphist").arg("wipe").output();
        self.wipe_armed.set(false);
        btn.set_label("Clear all");
        btn.style_context().remove_class("confirm");
        self.populate(Vec::new());
    }
}

pub fn run(conf: &Conf) {
    let width = conf.int("clipboard_width", 400);
    let height = conf.int("clipboard_height", 500); // fixed, so the popup can be placed predictably
    let gap = conf.int("clipboard_gap", 14) as f64;
    let sh = ui::shell(CSS, width, height);

    // open next to the mouse pointer (Wayland gives apps no text-caret position)
    match pointer_anchor() {
        None => {
            sh.card.set_halign(gtk::Align::Center);
            sh.card.set_valign(gtk::Align::Center);
        }
        Some((px, py, mw, mh)) => {
            let (w, h) = (width as f64, height as f64);
            let x = (px - w / 2.0).min(mw - w - 8.0).max(8.0);
            let mut y = py + gap;
            if y + h > mh - 8.0 {
                y = py - h - gap; // no room below: open above the pointer
            }
            y = y.min(mh - h - 8.0).max(8.0);
            sh.card.set_halign(gtk::Align::Start);
            sh.card.set_valign(gtk::Align::Start);
            sh.card.set_margin_start(x as i32);
            sh.card.set_margin_top(y as i32);
        }
    }

    let title = ui::label("\u{f328}  Clipboard");
    ui::class(&title, "title");
    title.set_margin_start(4);
    title.set_margin_bottom(6);
    sh.card.pack_start(&title, false, false, 0);

    let search = gtk::Entry::new();
    search.set_placeholder_text(Some("Search\u{2026}"));
    sh.card.pack_start(&search, false, false, 0);
    sh.card.pack_start(&ui::sep(), false, false, 0);

    let scroll = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .build();
    let list = gtk::ListBox::new();
    list.set_selection_mode(gtk::SelectionMode::Single);
    list.set_activate_on_single_click(true);
    let note = gtk::Label::new(Some("Loading\u{2026}"));
    ui::class(&note, "dim");
    note.set_margin_top(12);
    scroll.add(&list);
    sh.card.pack_start(&scroll, true, true, 0);
    sh.card.pack_start(&note, false, false, 0);

    sh.card.pack_start(&ui::sep(), false, false, 0);
    let foot = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let hint = ui::label("\u{2191}\u{2193} Enter copy  \u{b7}  Del remove");
    ui::class(&hint, "dim");
    foot.pack_start(&hint, true, true, 4);
    let wipe_btn = gtk::Button::with_label("Clear all");
    foot.pack_end(&wipe_btn, false, false, 0);
    let close = gtk::Button::with_label("Close");
    close.connect_clicked(|_| gtk::main_quit());
    foot.pack_end(&close, false, false, 0);
    sh.card.pack_start(&foot, false, false, 0);

    let me = Rc::new(Clip {
        list: list.clone(),
        scroll,
        search: search.clone(),
        note: note.clone(),
        items: RefCell::new(Vec::new()),
        wipe_armed: Cell::new(false),
        pool: ui::Pool::new(3),
        max_items: conf.int("clipboard_items", 60).max(1) as usize,
    });
    {
        let me = me.clone();
        search.connect_changed(move |_| me.on_search());
    }
    {
        let me = me.clone();
        list.connect_row_activated(move |_, row| me.pick(Some(row.clone())));
    }
    {
        let me = me.clone();
        wipe_btn.connect_clicked(move |b| me.on_wipe(b));
    }
    {
        let me = me.clone();
        sh.win.connect_key_press_event(move |_, ev| {
            let k = ev.keyval();
            if k == gdk::keys::constants::Escape {
                gtk::main_quit();
                return Inhibit(true);
            }
            if k == gdk::keys::constants::Down {
                me.move_sel(1);
                return Inhibit(true);
            }
            if k == gdk::keys::constants::Up {
                me.move_sel(-1);
                return Inhibit(true);
            }
            if k == gdk::keys::constants::Return || k == gdk::keys::constants::KP_Enter {
                me.pick(me.list.selected_row());
                return Inhibit(true);
            }
            if k == gdk::keys::constants::Delete && me.search.text().is_empty() {
                if let Some(row) = me.list.selected_row() {
                    let found = me.items.borrow().iter().find(|i| i.row == row).cloned();
                    if let Some(it) = found {
                        me.delete(&it);
                    }
                }
                return Inhibit(true);
            }
            Inhibit(false)
        });
    }
    sh.win.show_all();
    note.show();
    search.grab_focus();
    me.reload();
}
