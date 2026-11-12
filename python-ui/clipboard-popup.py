#!/usr/bin/env python3
import json, os, re, subprocess, threading
from concurrent.futures import ThreadPoolExecutor
import gi
gi.require_version("Gtk", "3.0")
gi.require_version("Gdk", "3.0")
gi.require_version("GtkLayerShell", "0.1")
gi.require_version("GdkPixbuf", "2.0")
from gi.repository import Gtk, Gdk, GLib, GtkLayerShell, GdkPixbuf

WIDTH = 400
CARD_H = 500         # fixed height, so the popup can be placed predictably
GAP = 14             # distance between the mouse pointer and the popup
MAX_ITEMS = 60
THUMB_W, THUMB_H = 360, 110

BG = "#282828"       # replaced by surface_container_high from colors.css if found
PILL = "#413D3A"
BORDER = "#514947"
GOLD = "#c7ab7a"
TEXT = "#ded8be"
DIM = "#ab9c88"
ERR = "#e07a6b"

# Sizes and placement also come from ~/.config/hypr/popups.conf when it
# exists — the same keys the Rust hypr-ui reads (see popupconf.py, next to
# these files). The constants above stay the defaults.
import popupconf
popupconf.override(globals(), "clipboard")

try:
    css_src = open(os.path.expanduser("~/.config/waybar/colors.css")).read()
    m = re.search(r"@define-color\s+surface_container_high\s+(#[0-9a-fA-F]{6})", css_src)
    if m:
        BG = m.group(1)
except OSError:
    pass

# what cliphist shows for binary entries: "[[ binary data 1.2 MiB png 800x600 ]]"
IMG_RE = re.compile(r"^\[\[ binary data (.+?) (png|jpg|jpeg|bmp|webp|gif|tiff?) (\d+x\d+) \]\]")

POOL = ThreadPoolExecutor(3)


def pointer_anchor():
    """Pointer position relative to its monitor: (x, y, monitor_w, monitor_h), or None."""
    try:
        c = json.loads(subprocess.check_output(["hyprctl", "-j", "cursorpos"], text=True, timeout=2))
        mons = json.loads(subprocess.check_output(["hyprctl", "-j", "monitors"], text=True, timeout=2))
    except Exception:
        return None
    for m in mons:
        w, h = m["width"], m["height"]
        if m.get("transform", 0) % 2:
            w, h = h, w
        w, h = w / m["scale"], h / m["scale"]
        if m["x"] <= c["x"] < m["x"] + w and m["y"] <= c["y"] < m["y"] + h:
            return c["x"] - m["x"], c["y"] - m["y"], w, h
    return None


CSS = f"""
window {{ background: transparent; }}
* {{ font-family: monospace, "Font Awesome 6 Free", sans-serif; font-size: 14px; color: {TEXT}; }}
.card {{ background: {BG}; border: 5px solid {BG}; border-radius: 6px; padding: 8px; }}
.title {{ color: {GOLD}; font-weight: bold; }}
.sep {{ background: {BORDER}; min-height: 1px; margin: 6px 0; }}
button {{ background: transparent; border: none; box-shadow: none; text-shadow: none; padding: 6px 8px; border-radius: 4px; }}
button:hover {{ background: alpha({GOLD}, 0.18); }}
.del label {{ color: {DIM}; font-size: 12px; }}
.del:hover label {{ color: {ERR}; }}
.confirm label {{ color: {ERR}; }}
.dim {{ color: {DIM}; font-size: 12px; }}
entry {{ background: {PILL}; border: 1px solid {BORDER}; border-radius: 4px; padding: 4px 8px; box-shadow: none; }}
entry:focus {{ border-color: {GOLD}; }}
viewport {{ background: transparent; }}
list {{ background: transparent; }}
list row {{ padding: 6px 8px; border-radius: 4px; }}
list row:hover {{ background: alpha({GOLD}, 0.18); }}
list row:selected {{ background: {PILL}; }}
scrolledwindow undershoot, scrolledwindow overshoot {{ background: none; }}
"""

TERMINALS = ("kitty", "foot", "alacritty", "wezterm", "ghostty", "konsole", "xterm", "gnome-terminal")


def active_class():
    try:
        w = json.loads(subprocess.check_output(["hyprctl", "-j", "activewindow"], text=True, timeout=2))
        return (w.get("class") or "").lower()
    except Exception:
        return ""


def auto_paste():
    """Press Ctrl+V (Ctrl+Shift+V in terminals) in the window you came from."""
    cls = active_class()
    if not cls:
        return
    if any(t in cls for t in TERMINALS):
        keys = ["-M", "ctrl", "-M", "shift", "-k", "v", "-m", "shift", "-m", "ctrl"]
    else:
        keys = ["-M", "ctrl", "-k", "v", "-m", "ctrl"]
    # short delay: the popup must close and give focus back first
    subprocess.Popen(["sh", "-c", 'sleep 0.2; exec wtype "$@"', "wtype", *keys],
           start_new_session=True,
           stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

class Item(Gtk.ListBoxRow):
    def __init__(self, popup, line):
        super().__init__()
        self.popup = popup
        self.line = line
        self.cid, _, prev = line.partition("\t")
        prev = prev.strip()
        m = IMG_RE.match(prev)
        self.mime = None
        self.img = None

        outer = Gtk.Box(spacing=8)
        body = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=4)

        if m:
            size, ext, dims = m.groups()
            ext = "jpeg" if ext == "jpg" else ext
            self.mime = f"image/{ext}"
            self.search = f"image {ext} {dims} {size}".lower()
            self.img = Gtk.Image()
            self.img.set_halign(Gtk.Align.START)
            self.img.set_size_request(-1, 40)
            body.pack_start(self.img, False, False, 0)
            cap = Gtk.Label(label=f"image  \u00b7  {ext}  \u00b7  {dims}  \u00b7  {size}", xalign=0)
            cap.get_style_context().add_class("dim")
            body.pack_start(cap, False, False, 0)
            POOL.submit(self.load_thumb)
        else:
            self.search = prev.lower()
            lab = Gtk.Label(label=prev or "(empty)", xalign=0, ellipsize=3)
            lab.set_single_line_mode(True)
            body.pack_start(lab, False, False, 0)

        outer.pack_start(body, True, True, 0)

        d = Gtk.Button(label="\uf00d")
        d.get_style_context().add_class("del")
        d.set_can_focus(False)
        d.set_valign(Gtk.Align.CENTER)
        d.connect("clicked", lambda *_: popup.delete(self))
        outer.pack_end(d, False, False, 0)
        self.add(outer)

    def load_thumb(self):
        try:
            data = subprocess.run(["cliphist", "decode", self.cid],
                                  capture_output=True, timeout=10).stdout
            loader = GdkPixbuf.PixbufLoader()
            loader.write(data)
            loader.close()
            pb = loader.get_pixbuf()
            if pb is None:
                return
            w, h = pb.get_width(), pb.get_height()
            r = min(THUMB_W / w, THUMB_H / h, 1.0)
            if r < 1.0:
                pb = pb.scale_simple(max(1, int(w * r)), max(1, int(h * r)),
                                    GdkPixbuf.InterpType.BILINEAR)
            GLib.idle_add(self.set_thumb, pb)
        except Exception:
            pass

    def set_thumb(self, pb):
        self.img.set_from_pixbuf(pb)
        self.img.set_size_request(-1, -1)
        return False


class Popup(Gtk.Window):
    def __init__(self):
        super().__init__()
        self.wipe_armed = False

        self.set_app_paintable(True)
        vis = self.get_screen().get_rgba_visual()
        if vis:
            self.set_visual(vis)

        GtkLayerShell.init_for_window(self)
        GtkLayerShell.set_layer(self, GtkLayerShell.Layer.OVERLAY)
        GtkLayerShell.set_keyboard_mode(self, GtkLayerShell.KeyboardMode.EXCLUSIVE)
        GtkLayerShell.set_exclusive_zone(self, -1)
        for edge in (GtkLayerShell.Edge.TOP, GtkLayerShell.Edge.BOTTOM,
                    GtkLayerShell.Edge.LEFT, GtkLayerShell.Edge.RIGHT):
            GtkLayerShell.set_anchor(self, edge, True)

        prov = Gtk.CssProvider()
        prov.load_from_data(CSS.encode())
        Gtk.StyleContext.add_provider_for_screen(
            Gdk.Screen.get_default(), prov, Gtk.STYLE_PROVIDER_PRIORITY_USER)

        # your own tweaks: ~/.config/hypr/popup-user.css — applied on top of
        # the built-in CSS, read again every time the popup opens (popupconf.py)
        popupconf.load_user_css()

        # full-screen transparent root: clicking outside the card closes the popup
        root = Gtk.EventBox()
        root.set_visible_window(False)
        root.add_events(Gdk.EventMask.BUTTON_PRESS_MASK)
        root.connect("button-press-event", self.on_root_click)
        self.add(root)

        self.card = Gtk.Box(orientation=Gtk.Orientation.VERTICAL)
        self.card.get_style_context().add_class("card")
        self.card.set_size_request(WIDTH, CARD_H)
        self.place_card()
        root.add(self.card)

        title = Gtk.Label(label="\uf328  Clipboard", xalign=0)
        title.get_style_context().add_class("title")
        title.set_margin_start(4)
        title.set_margin_bottom(6)
        self.card.pack_start(title, False, False, 0)

        self.search = Gtk.Entry()
        self.search.set_placeholder_text("Search\u2026")
        self.search.connect("changed", self.on_search)
        self.card.pack_start(self.search, False, False, 0)

        sep = Gtk.Box(); sep.get_style_context().add_class("sep")
        self.card.pack_start(sep, False, False, 0)

        self.scroll = Gtk.ScrolledWindow()
        self.scroll.set_policy(Gtk.PolicyType.NEVER, Gtk.PolicyType.AUTOMATIC)
        self.list = Gtk.ListBox()
        self.list.set_selection_mode(Gtk.SelectionMode.SINGLE)
        self.list.set_activate_on_single_click(True)
        self.list.connect("row-activated", lambda _l, row: self.pick(row))
        self.note = Gtk.Label(label="Loading\u2026")
        self.note.get_style_context().add_class("dim")
        self.note.set_margin_top(12)
        self.scroll.add(self.list)
        self.card.pack_start(self.scroll, True, True, 0)
        self.card.pack_start(self.note, False, False, 0)

        sep2 = Gtk.Box(); sep2.get_style_context().add_class("sep")
        self.card.pack_start(sep2, False, False, 0)
        foot = Gtk.Box(spacing=6)
        hint = Gtk.Label(label="\u2191\u2193 Enter copy  \u00b7  Del remove", xalign=0)
        hint.get_style_context().add_class("dim")
        foot.pack_start(hint, True, True, 4)
        self.wipe_btn = Gtk.Button(label="Clear all")
        self.wipe_btn.connect("clicked", self.on_wipe)
        foot.pack_end(self.wipe_btn, False, False, 0)
        close = Gtk.Button(label="Close")
        close.connect("clicked", lambda *_: Gtk.main_quit())
        foot.pack_end(close, False, False, 0)
        self.card.pack_start(foot, False, False, 0)

        self.connect("key-press-event", self.on_key)
        self.connect("destroy", Gtk.main_quit)
        self.show_all()
        self.note.show()
        self.search.grab_focus()
        self.reload()

    # ---------- placement ----------
    def place_card(self):
        """Open next to the mouse pointer (Wayland gives apps no text-caret position)."""
        a = pointer_anchor()
        if a is None:
            self.card.set_halign(Gtk.Align.CENTER)
            self.card.set_valign(Gtk.Align.CENTER)
            return
        px, py, mw, mh = a
        x = max(8, min(px - WIDTH / 2, mw - WIDTH - 8))
        y = py + GAP
        if y + CARD_H > mh - 8:          # no room below: open above the pointer
            y = py - CARD_H - GAP
        y = max(8, min(y, mh - CARD_H - 8))
        self.card.set_halign(Gtk.Align.START)
        self.card.set_valign(Gtk.Align.START)
        self.card.set_margin_start(int(x))
        self.card.set_margin_top(int(y))

    # ---------- data ----------
    def reload(self):
        def work():
            try:
                out = subprocess.run(["cliphist", "list"], capture_output=True,
                                    text=True, errors="replace", timeout=10).stdout
            except Exception:
                out = ""
            GLib.idle_add(self.populate, [l for l in out.splitlines() if l.strip()])
        threading.Thread(target=work, daemon=True).start()

    def populate(self, lines):
        for c in self.list.get_children():
            self.list.remove(c)
        for line in lines[:MAX_ITEMS]:
            self.list.add(Item(self, line))
        self.list.show_all()
        self.on_search(self.search)
        return False

    def visible_rows(self):
        return [r for r in self.list.get_children() if r.get_visible()]

    def update_note(self):
        rows = self.visible_rows()
        if rows:
            self.note.hide()
        else:
            self.note.set_text("Nothing copied yet" if not self.search.get_text()
                              else "No matches")
            self.note.show()

    def on_search(self, entry):
        q = entry.get_text().strip().lower()
        for r in self.list.get_children():
            r.set_visible(q in r.search)
        rows = self.visible_rows()
        if rows:
            self.select(rows[0])
        self.update_note()

    # ---------- selection ----------
    def select(self, row):
        self.list.select_row(row)
        GLib.idle_add(self.reveal, row)

    def reveal(self, row):
        a = row.get_allocation()
        adj = self.scroll.get_vadjustment()
        if a.y < adj.get_value():
            adj.set_value(a.y)
        elif a.y + a.height > adj.get_value() + adj.get_page_size():
            adj.set_value(a.y + a.height - adj.get_page_size())
        return False

    def move(self, d):
        rows = self.visible_rows()
        if not rows:
            return
        cur = self.list.get_selected_row()
        i = rows.index(cur) if cur in rows else -1
        self.select(rows[max(0, min(len(rows) - 1, i + d))])

    # ---------- actions ----------
    def pick(self, row):
      if row is None:
          return
      try:
          data = subprocess.run(["cliphist", "decode", row.cid],
                                capture_output=True, timeout=10).stdout
          args = ["wl-copy"] + (["--type", row.mime] if row.mime else [])
          p = subprocess.Popen(args, stdin=subprocess.PIPE, start_new_session=True,
                              stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
          p.stdin.write(data)
          p.stdin.close()
          auto_paste()
      except Exception:
          pass
      Gtk.main_quit()

    def delete(self, row):
        try:
            subprocess.run(["cliphist", "delete"], input=row.line.encode(),
                          capture_output=True, timeout=5)
        except Exception:
            return
        rows = self.visible_rows()
        nxt = None
        if row in rows:
            i = rows.index(row)
            rest = rows[:i] + rows[i + 1:]
            if rest:
                nxt = rest[min(i, len(rest) - 1)]
        self.list.remove(row)
        if nxt is not None:
            self.select(nxt)
        self.update_note()

    def on_wipe(self, btn):
        if not self.wipe_armed:
            self.wipe_armed = True
            btn.set_label("Click again to confirm")
            btn.get_style_context().add_class("confirm")
            return
        subprocess.run(["cliphist", "wipe"], capture_output=True, timeout=5)
        self.wipe_armed = False
        btn.set_label("Clear all")
        btn.get_style_context().remove_class("confirm")
        self.populate([])

    # ---------- events ----------
    def on_key(self, _w, ev):
        k = ev.keyval
        if k == Gdk.KEY_Escape:
            Gtk.main_quit()
            return True
        if k == Gdk.KEY_Down:
            self.move(1)
            return True
        if k == Gdk.KEY_Up:
            self.move(-1)
            return True
        if k in (Gdk.KEY_Return, Gdk.KEY_KP_Enter):
            self.pick(self.list.get_selected_row())
            return True
        if k == Gdk.KEY_Delete and not self.search.get_text():
            row = self.list.get_selected_row()
            if row is not None:
                self.delete(row)
            return True
        return False

    def on_root_click(self, _w, ev):
        a = self.card.get_allocation()
        if not (a.x <= ev.x <= a.x + a.width and a.y <= ev.y <= a.y + a.height):
            Gtk.main_quit()
        return True


if __name__ == "__main__":
    Popup()
    Gtk.main()
