#!/usr/bin/env python3
import json, os, re, subprocess, threading
from concurrent.futures import ThreadPoolExecutor
import gi
gi.require_version("Gtk", "3.0")
gi.require_version("Gdk", "3.0")
gi.require_version("GtkLayerShell", "0.1")
from gi.repository import Gtk, Gdk, GLib, GtkLayerShell

# placement: distance from the top-right corner of the screen
TOP_MARGIN = 52      # gap below the bar
RIGHT_MARGIN = 150   # increase to move the box left
WIDTH = 360
MAX_LIST_HEIGHT = 420
MAX_VOLUME = 100     # set to 150 to allow over-amplification

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
popupconf.override(globals(), "volume")

try:
    css_src = open(os.path.expanduser("~/.config/waybar/colors.css")).read()
    m = re.search(r"@define-color\s+surface_container_high\s+(#[0-9a-fA-F]{6})", css_src)
    if m:
        BG = m.group(1)
except OSError:
    pass


def pactl(*args, timeout=5):
    try:
        p = subprocess.run(["pactl", *args], capture_output=True, text=True, timeout=timeout)
        return p.returncode, p.stdout.strip()
    except Exception as e:
        return 1, str(e)


def pactl_bg(*args):
    """fire and forget"""
    try:
        subprocess.Popen(["pactl", *args], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    except OSError:
        pass


def jlist(kind):
    rc, out = pactl("-f", "json", "list", kind)
    if rc != 0:
        return []
    try:
        return json.loads(out)
    except ValueError:
        return []


def vol_pct(d):
    for v in (d.get("volume") or {}).values():
        try:
            return int(str(v.get("value_percent", "0%")).strip().rstrip("%"))
        except ValueError:
            return 0
    return 0


def get_audio():
    # all queries in parallel
    with ThreadPoolExecutor(5) as ex:
        f_sinks = ex.submit(jlist, "sinks")
        f_srcs = ex.submit(jlist, "sources")
        f_inputs = ex.submit(jlist, "sink-inputs")
        f_dsink = ex.submit(pactl, "get-default-sink")
        f_dsrc = ex.submit(pactl, "get-default-source")
        sinks, srcs, inputs = f_sinks.result(), f_srcs.result(), f_inputs.result()
        dsink, dsrc = f_dsink.result()[1], f_dsrc.result()[1]

    outs = [{"name": s["name"], "desc": s.get("description") or s["name"],
            "vol": vol_pct(s), "mute": bool(s.get("mute"))} for s in sinks]
    ins = [{"name": s["name"], "desc": s.get("description") or s["name"],
            "vol": vol_pct(s), "mute": bool(s.get("mute"))}
          for s in srcs if not s["name"].endswith(".monitor")]
    apps = []
    for i in inputs:
        props = i.get("properties") or {}
        label = (props.get("application.name") or props.get("media.name")
                or i.get("name") or "Stream")
        apps.append({"id": str(i["index"]), "label": label,
                    "vol": vol_pct(i), "mute": bool(i.get("mute"))})
    return {"sinks": outs, "sources": ins, "apps": apps, "dsink": dsink, "dsrc": dsrc}


def bg(work, done):
    def t():
        res = work()
        GLib.idle_add(done, res)
    threading.Thread(target=t, daemon=True).start()


CSS = f"""
window {{ background: transparent; }}
* {{ font-family: monospace, "Font Awesome 6 Free", sans-serif; font-size: 14px; color: {TEXT}; }}
.card {{ background: {BG}; border: 5px solid {BG}; border-radius: 6px; padding: 8px; }}
.title {{ color: {GOLD}; font-weight: bold; }}
.section {{ color: {GOLD}; font-size: 12px; font-weight: bold; }}
.sep {{ background: {BORDER}; min-height: 1px; margin: 6px 0; }}
button {{ background: transparent; border: none; box-shadow: none; text-shadow: none; padding: 6px 8px; border-radius: 4px; }}
button:hover {{ background: alpha({GOLD}, 0.18); }}
.device.active {{ background: {PILL}; }}
.muted label {{ color: {ERR}; }}
.dim {{ color: {DIM}; font-size: 12px; }}
.pct {{ color: {DIM}; font-size: 12px; }}
scale trough {{ background: {PILL}; border: 1px solid {BORDER}; border-radius: 4px; min-height: 6px; }}
scale highlight {{ background: {GOLD}; border-radius: 4px; border: none; }}
scale slider {{ background: {TEXT}; border: none; box-shadow: none; border-radius: 8px; min-width: 14px; min-height: 14px; margin: -5px; }}
scale slider:hover {{ background: {GOLD}; }}
scale.muted highlight {{ background: {DIM}; }}
scrolledwindow undershoot, scrolledwindow overshoot {{ background: none; }}
"""


class VolRow(Gtk.Box):
    """mute button + label + slider + percentage"""

    def __init__(self, popup, kind, ident, label, vol, mute, icons):
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        self.popup, self.kind, self.ident = popup, kind, ident
        self.icon_on, self.icon_off = icons
        self.mute = mute
        self._updating = False
        self._pending = None

        top = Gtk.Box(spacing=6)
        self.mbtn = Gtk.Button(label=self.icon_on)
        self.mbtn.connect("clicked", self.on_mute)
        top.pack_start(self.mbtn, False, False, 0)
        name = Gtk.Label(label=label, xalign=0, ellipsize=3)
        top.pack_start(name, True, True, 0)
        self.pct = Gtk.Label(label="0%")
        self.pct.get_style_context().add_class("pct")
        top.pack_end(self.pct, False, False, 4)
        self.pack_start(top, False, False, 0)

        self.scale = Gtk.Scale.new_with_range(Gtk.Orientation.HORIZONTAL, 0, MAX_VOLUME, 1)
        self.scale.set_draw_value(False)
        self.scale.set_margin_start(10)
        self.scale.set_margin_end(10)
        self.scale.set_margin_bottom(6)
        self.scale.add_events(Gdk.EventMask.BUTTON_PRESS_MASK | Gdk.EventMask.BUTTON_RELEASE_MASK)
        self.scale.connect("button-press-event", self.on_press)
        self.scale.connect("button-release-event", self.on_release)
        self.scale.connect("value-changed", self.on_changed)
        self.pack_start(self.scale, False, False, 0)

        self.set_state(vol, mute)

    def set_state(self, vol, mute):
        self._updating = True
        self.scale.set_value(vol)
        self._updating = False
        self.mute = mute
        self.pct.set_text(f"{vol}%")
        self.mbtn.set_label(self.icon_off if mute else self.icon_on)
        for w in (self.mbtn, self.scale):
            ctx = w.get_style_context()
            (ctx.add_class if mute else ctx.remove_class)("muted")

    def on_press(self, *_):
        self.popup.dragging = True
        return False

    def on_release(self, *_):
        self.popup.dragging = False
        return False

    def on_changed(self, scale):
        v = int(scale.get_value())
        self.pct.set_text(f"{v}%")
        if self._updating:
            return
        if self._pending:
            GLib.source_remove(self._pending)
        self._pending = GLib.timeout_add(40, self.send, v)

    def send(self, v):
        self._pending = None
        pactl_bg(f"set-{self.kind}-volume", self.ident, f"{v}%")
        return False

    def on_mute(self, _b):
        pactl_bg(f"set-{self.kind}-mute", self.ident, "toggle")
        self.set_state(int(self.scale.get_value()), not self.mute)   # instant feedback
        self.popup.refresh_soon()


class Popup(Gtk.Window):
    def __init__(self):
        super().__init__()
        self.dragging = False
        self.sig = None
        self.rows = {}

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
        self.card.set_halign(Gtk.Align.END)
        self.card.set_valign(Gtk.Align.START)
        self.card.set_margin_top(TOP_MARGIN)
        self.card.set_margin_end(RIGHT_MARGIN)
        root.add(self.card)

        head = Gtk.Box(spacing=6)
        title = Gtk.Label(label="\uf028  Volume", xalign=0)
        title.get_style_context().add_class("title")
        head.pack_start(title, True, True, 4)
        self.card.pack_start(head, False, False, 0)

        sep = Gtk.Box(); sep.get_style_context().add_class("sep")
        self.card.pack_start(sep, False, False, 0)

        self.scroll = Gtk.ScrolledWindow()
        self.scroll.set_policy(Gtk.PolicyType.NEVER, Gtk.PolicyType.AUTOMATIC)
        self.scroll.set_propagate_natural_height(True)
        self.scroll.set_max_content_height(MAX_LIST_HEIGHT)
        self.list = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=2)
        self.list.pack_start(self.note("Loading\u2026"), False, False, 8)
        self.scroll.add(self.list)
        self.card.pack_start(self.scroll, False, False, 0)

        sep2 = Gtk.Box(); sep2.get_style_context().add_class("sep")
        self.card.pack_start(sep2, False, False, 0)
        foot = Gtk.Box()
        close = Gtk.Button(label="Close")
        close.connect("clicked", lambda *_: Gtk.main_quit())
        foot.pack_end(close, False, False, 0)
        self.card.pack_start(foot, False, False, 0)

        self.connect("key-press-event", self.on_key)
        self.connect("destroy", Gtk.main_quit)
        self.show_all()

        self.refresh()
        GLib.timeout_add(1500, self.tick)     # keep in sync with changes made elsewhere

    # ---------- events ----------
    def on_key(self, _w, ev):
        if ev.keyval == Gdk.KEY_Escape:
            Gtk.main_quit()
            return True
        return False

    def on_root_click(self, _w, ev):
        a = self.card.get_allocation()
        if not (a.x <= ev.x <= a.x + a.width and a.y <= ev.y <= a.y + a.height):
            Gtk.main_quit()
        return True

    def tick(self):
        if not self.dragging:
            self.refresh()
        return True

    def refresh_soon(self):
        GLib.timeout_add(200, lambda: (self.refresh(), False)[1])

    def refresh(self):
        bg(get_audio, self.apply)

    # ---------- building ----------
    def note(self, text):
        l = Gtk.Label(label=text)
        l.get_style_context().add_class("dim")
        return l

    def section(self, text):
        l = Gtk.Label(label=text.upper(), xalign=0)
        l.get_style_context().add_class("section")
        l.set_margin_start(8)
        l.set_margin_top(8)
        l.set_margin_bottom(2)
        return l

    def apply(self, d):
        if self.dragging:
            return
        sig = (d["dsink"], d["dsrc"],
              tuple(s["name"] for s in d["sinks"]),
              tuple(s["name"] for s in d["sources"]),
              tuple((a["id"], a["label"]) for a in d["apps"]))
        if sig != self.sig:
            self.sig = sig
            self.rebuild(d)
        else:
            # same layout: just update values in place (keeps focus and avoids flicker)
            for key, row in self.rows.items():
                kind, ident = key
                src = {"sink": d["sinks"], "source": d["sources"]}.get(kind)
                if kind == "sink-input":
                    item = next((a for a in d["apps"] if a["id"] == ident), None)
                else:
                    item = next((s for s in src if s["name"] == ident), None)
                if item:
                    row.set_state(item["vol"], item["mute"])

    def rebuild(self, d):
        for c in self.list.get_children():
            self.list.remove(c)
        self.rows = {}

        sinks, sources = d["sinks"], d["sources"]
        cur = next((s for s in sinks if s["name"] == d["dsink"]), sinks[0] if sinks else None)
        cur_src = next((s for s in sources if s["name"] == d["dsrc"]), sources[0] if sources else None)

        # output
        self.list.pack_start(self.section("Output"), False, False, 0)
        if cur:
            r = VolRow(self, "sink", cur["name"], cur["desc"], cur["vol"], cur["mute"],
                      ("\uf028", "\uf6a9"))
            self.rows[("sink", cur["name"])] = r
            self.list.pack_start(r, False, False, 0)
        else:
            self.list.pack_start(self.note("No output device"), False, False, 6)
        if len(sinks) > 1:
            for s in sinks:
                self.list.pack_start(self.device_button(s, s["name"] == (cur or {}).get("name")),
                                    False, False, 0)

        # input
        if cur_src:
            self.list.pack_start(self.section("Input"), False, False, 0)
            r = VolRow(self, "source", cur_src["name"], cur_src["desc"], cur_src["vol"],
                      cur_src["mute"], ("\uf130", "\uf131"))
            self.rows[("source", cur_src["name"])] = r
            self.list.pack_start(r, False, False, 0)

        # apps
        self.list.pack_start(self.section("Applications"), False, False, 0)
        if not d["apps"]:
            self.list.pack_start(self.note("Nothing is playing"), False, False, 6)
        for a in d["apps"]:
            r = VolRow(self, "sink-input", a["id"], a["label"], a["vol"], a["mute"],
                      ("\uf028", "\uf6a9"))
            self.rows[("sink-input", a["id"])] = r
            self.list.pack_start(r, False, False, 0)

        self.list.show_all()

    def device_button(self, s, active):
        btn = Gtk.Button()
        ctx = btn.get_style_context()
        ctx.add_class("device")
        if active:
            ctx.add_class("active")
        box = Gtk.Box(spacing=8)
        mark = Gtk.Label(label="\uf00c" if active else " ")
        mark.get_style_context().add_class("dim")
        box.pack_start(mark, False, False, 0)
        box.pack_start(Gtk.Label(label=s["desc"], xalign=0, ellipsize=3), True, True, 0)
        btn.add(box)
        btn.connect("clicked", lambda *_: self.set_default(s["name"]))
        return btn

    def set_default(self, name):
        def work():
            pactl("set-default-sink", name)
            # move playing streams to the new device too
            for i in jlist("sink-inputs"):
                pactl("move-sink-input", str(i["index"]), name)
            return get_audio()
        self.sig = None
        bg(work, self.apply)


if __name__ == "__main__":
    Popup()
    Gtk.main()
