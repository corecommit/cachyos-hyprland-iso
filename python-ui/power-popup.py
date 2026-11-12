#!/usr/bin/env python3
import os, re, subprocess
import gi
gi.require_version("Gtk", "3.0")
gi.require_version("Gdk", "3.0")
gi.require_version("GtkLayerShell", "0.1")
from gi.repository import Gtk, Gdk, GtkLayerShell

WIDTH = 260

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
popupconf.override(globals(), "power")

try:
    css_src = open(os.path.expanduser("~/.config/waybar/colors.css")).read()
    m = re.search(r"@define-color\s+surface_container_high\s+(#[0-9a-fA-F]{6})", css_src)
    if m:
        BG = m.group(1)
except OSError:
    pass

# (icon, label, hotkey, command, dangerous)
ACTIONS = [
    ("\uf023", "Lock",      "L", ["hyprlock"], False),
    ("\uf2f5", "Log out",   "O", ["sh", "-c", "hyprctl dispatch 'hl.dsp.exit()'; hyprctl dispatch exit"], False),
    ("\uf186", "Suspend",   "S", ["systemctl", "suspend"], False),
    ("\uf021", "Reboot",    "R", ["systemctl", "reboot"], True),
    ("\uf011", "Shut down", "P", ["systemctl", "poweroff"], True),
]

CSS = f"""
window {{ background: transparent; }}
* {{ font-family: monospace, "Font Awesome 6 Free", sans-serif; font-size: 14px; color: {TEXT}; }}
.card {{ background: {BG}; border: 5px solid {BG}; border-radius: 6px; padding: 8px; }}
.title {{ color: {GOLD}; font-weight: bold; }}
.sep {{ background: {BORDER}; min-height: 1px; margin: 6px 0; }}
button {{ background: transparent; border: none; box-shadow: none; text-shadow: none; padding: 8px 10px; border-radius: 4px; }}
button:hover, button:focus {{ background: alpha({GOLD}, 0.18); }}
button:focus {{ background: {PILL}; }}
button.danger:hover label, button.danger:focus label {{ color: {ERR}; }}
.icon {{ color: {GOLD}; }}
.dim {{ color: {DIM}; font-size: 12px; }}
"""


class Popup(Gtk.Window):
    def __init__(self):
        super().__init__()

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
        self.card.set_size_request(WIDTH, -1)
        self.card.set_halign(Gtk.Align.CENTER)
        self.card.set_valign(Gtk.Align.CENTER)
        root.add(self.card)

        title = Gtk.Label(label="\uf011  Power", xalign=0)
        title.get_style_context().add_class("title")
        title.set_margin_start(4)
        title.set_margin_top(4)
        title.set_margin_bottom(2)
        self.card.pack_start(title, False, False, 0)

        sep = Gtk.Box(); sep.get_style_context().add_class("sep")
        self.card.pack_start(sep, False, False, 0)

        self.buttons = []
        self.hotkeys = {}
        for icon, label, key, cmd, danger in ACTIONS:
            btn = Gtk.Button()
            if danger:
                btn.get_style_context().add_class("danger")
            box = Gtk.Box(spacing=12)
            ic = Gtk.Label(label=icon)
            ic.get_style_context().add_class("icon")
            ic.set_size_request(20, -1)
            box.pack_start(ic, False, False, 0)
            box.pack_start(Gtk.Label(label=label, xalign=0), True, True, 0)
            hk = Gtk.Label(label=key)
            hk.get_style_context().add_class("dim")
            box.pack_end(hk, False, False, 0)
            btn.add(box)
            btn.connect("clicked", lambda *_a, c=cmd: self.run(c))
            self.card.pack_start(btn, False, False, 0)
            self.buttons.append(btn)
            self.hotkeys[key.lower()] = cmd

        hint = Gtk.Label(label="arrows + Enter, or press the letter  \u00b7  Esc closes", xalign=0)
        hint.get_style_context().add_class("dim")
        hint.set_margin_start(4)
        hint.set_margin_top(6)
        self.card.pack_start(hint, False, False, 0)

        self.connect("key-press-event", self.on_key)
        self.connect("destroy", Gtk.main_quit)
        self.show_all()
        self.buttons[0].grab_focus()

    def run(self, cmd):
        try:
            subprocess.Popen(cmd, start_new_session=True,
                            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        except OSError:
            pass
        Gtk.main_quit()

    def on_key(self, _w, ev):
        if ev.keyval == Gdk.KEY_Escape:
            Gtk.main_quit()
            return True
        name = Gdk.keyval_name(ev.keyval) or ""
        if len(name) == 1 and name.lower() in self.hotkeys:
            self.run(self.hotkeys[name.lower()])
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
