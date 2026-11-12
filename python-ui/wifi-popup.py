#!/usr/bin/env python3
import os, re, subprocess, threading
from concurrent.futures import ThreadPoolExecutor
import gi
gi.require_version("Gtk", "3.0")
gi.require_version("Gdk", "3.0")
gi.require_version("GtkLayerShell", "0.1")
from gi.repository import Gtk, Gdk, GLib, GtkLayerShell

# placement: distance from the top-right corner of the screen
TOP_MARGIN = 52      # gap below the bar
RIGHT_MARGIN = 150   # increase to move the box left
WIDTH = 340
MAX_LIST_HEIGHT = 300

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
popupconf.override(globals(), "wifi")

try:
    css_src = open(os.path.expanduser("~/.config/waybar/colors.css")).read()
    m = re.search(r"@define-color\s+surface_container_high\s+(#[0-9a-fA-F]{6})", css_src)
    if m:
        BG = m.group(1)
except OSError:
    pass


def run(*args, timeout=40):
    try:
        p = subprocess.run(["nmcli", *args], capture_output=True, text=True, timeout=timeout)
        return p.returncode, (p.stdout + p.stderr).strip()
    except Exception as e:
        return 1, str(e)


def split_terse(line):
    out, cur, esc = [], "", False
    for ch in line:
        if esc:
            cur += ch; esc = False
        elif ch == "\\":
            esc = True
        elif ch == ":":
            out.append(cur); cur = ""
        else:
            cur += ch
    out.append(cur)
    return out


def get_networks():
    # the three queries run in parallel: much faster than one after another
    with ThreadPoolExecutor(3) as ex:
        f_radio = ex.submit(run, "radio", "wifi")
        f_saved = ex.submit(run, "-t", "-f", "NAME,TYPE", "con", "show")
        f_list = ex.submit(run, "-t", "-f", "IN-USE,SIGNAL,SECURITY,SSID",
                           "dev", "wifi", "list", "--rescan", "no")
        radio, saved_out, out = f_radio.result()[1], f_saved.result()[1], f_list.result()[1]
    on = radio.strip() == "enabled"
    if not on:
        return on, []
    saved = {split_terse(l)[0] for l in saved_out.splitlines() if "wireless" in l}
    best = {}
    for line in out.splitlines():
        f = split_terse(line)
        if len(f) < 4 or not f[3]:
            continue
        net = {"active": f[0].strip() == "*", "signal": int(f[1] or 0),
               "secure": f[2] not in ("", "--"), "ssid": f[3], "saved": f[3] in saved}
        old = best.get(net["ssid"])
        if not old or net["active"] or (not old["active"] and net["signal"] > old["signal"]):
            best[net["ssid"]] = net
    nets = sorted(best.values(), key=lambda n: (not n["active"], -n["signal"]))
    return on, nets


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
.sep {{ background: {BORDER}; min-height: 1px; margin: 6px 0; }}
button {{ background: transparent; border: none; box-shadow: none; text-shadow: none; padding: 6px 8px; border-radius: 4px; }}
button:hover {{ background: alpha({GOLD}, 0.18); }}
.row.active {{ background: {PILL}; }}
.bars {{ color: {GOLD}; }}
.dim {{ color: {DIM}; font-size: 12px; }}
.err {{ color: {ERR}; font-size: 12px; }}
entry {{ background: {PILL}; border: 1px solid {BORDER}; border-radius: 4px; padding: 4px 8px; box-shadow: none; }}
entry:focus {{ border-color: {GOLD}; }}
.connect {{ background: {GOLD}; }}
.connect label {{ color: {BG}; font-weight: bold; }}
.connect:hover {{ background: alpha({GOLD}, 0.8); }}
scrolledwindow undershoot, scrolledwindow overshoot {{ background: none; }}
"""


class Popup(Gtk.Window):
    def __init__(self):
        super().__init__()
        self.open_rev = None
        self._setting = False

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
        title = Gtk.Label(label="\uf1eb  Wi-Fi", xalign=0)
        title.get_style_context().add_class("title")
        head.pack_start(title, True, True, 4)
        refresh = Gtk.Button(label="\uf021")
        refresh.connect("clicked", lambda *_: self.refresh(True))
        head.pack_start(refresh, False, False, 0)
        self.switch = Gtk.Switch()
        self.switch.set_valign(Gtk.Align.CENTER)
        self.switch.connect("notify::active", self.on_switch)
        head.pack_start(self.switch, False, False, 4)
        self.card.pack_start(head, False, False, 0)

        sep = Gtk.Box(); sep.get_style_context().add_class("sep")
        self.card.pack_start(sep, False, False, 0)

        self.scroll = Gtk.ScrolledWindow()
        self.scroll.set_policy(Gtk.PolicyType.NEVER, Gtk.PolicyType.AUTOMATIC)
        self.scroll.set_propagate_natural_height(True)
        self.scroll.set_max_content_height(MAX_LIST_HEIGHT)
        self.list = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=2)
        self.list.pack_start(self.note("Scanning\u2026"), False, False, 8)   # shown instantly
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

        self.refresh(False)                       # cached list right away
        GLib.timeout_add(100, lambda: (self.refresh(True), False)[1])   # then a fresh scan

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

    def on_switch(self, sw, _p):
        if self._setting:
            return
        want = sw.get_active()

        def work():
            if want:
                subprocess.run(["rfkill", "unblock", "wifi"], capture_output=True)
                run("radio", "wifi", "on")
                GLib.usleep(1500000)
                run("dev", "wifi", "rescan")
                GLib.usleep(1500000)
            else:
                run("radio", "wifi", "off")
            return get_networks()
        bg(work, self.populate)

    def refresh(self, rescan):
        def work():
            if rescan:
                run("dev", "wifi", "rescan")
                GLib.usleep(2000000)          # give the scan a moment to finish
            return get_networks()
        bg(work, self.populate)

    def populate(self, result):
        if self.open_rev is not None:   # don't wipe a password box while typing
            return
        on, nets = result
        self._setting = True
        self.switch.set_active(on)
        self._setting = False
        for c in self.list.get_children():
            self.list.remove(c)
        if not on:
            self.list.pack_start(self.note("Wi-Fi is turned off"), False, False, 8)
        elif not nets:
            self.list.pack_start(self.note("No networks found"), False, False, 8)
        for n in nets:
            self.list.pack_start(self.make_row(n), False, False, 0)
        self.list.show_all()

    def note(self, text):
        l = Gtk.Label(label=text)
        l.get_style_context().add_class("dim")
        return l

    def make_row(self, n):
        wrap = Gtk.Box(orientation=Gtk.Orientation.VERTICAL)
        btn = Gtk.Button()
        ctx = btn.get_style_context()
        ctx.add_class("row")
        if n["active"]:
            ctx.add_class("active")
        box = Gtk.Box(spacing=10)
        bars = Gtk.Label(label="\u2582\u2584\u2586\u2588"[min(3, n["signal"] // 25)])
        bars.get_style_context().add_class("bars")
        box.pack_start(bars, False, False, 0)
        box.pack_start(Gtk.Label(label=n["ssid"], xalign=0, ellipsize=3), True, True, 0)
        if n["active"]:
            tag = Gtk.Label(label="connected")
            tag.get_style_context().add_class("dim")
            box.pack_end(tag, False, False, 0)
        elif n["secure"]:
            lock = Gtk.Label(label="\uf023")
            lock.get_style_context().add_class("dim")
            box.pack_end(lock, False, False, 0)
        btn.add(box)
        wrap.pack_start(btn, False, False, 0)

        # password / status area directly below this row
        rev = Gtk.Revealer()
        rev.set_transition_type(Gtk.RevealerTransitionType.SLIDE_DOWN)
        rev.set_transition_duration(120)
        inner = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=4)
        inner.set_margin_start(8); inner.set_margin_end(8)
        inner.set_margin_top(4); inner.set_margin_bottom(6)
        erow = Gtk.Box(spacing=6)
        entry = Gtk.Entry()
        entry.set_visibility(False)
        entry.set_placeholder_text("Password")
        go = Gtk.Button(label="Connect")
        go.get_style_context().add_class("connect")
        erow.pack_start(entry, True, True, 0)
        erow.pack_start(go, False, False, 0)
        erow.set_no_show_all(True)     # hidden until a password is needed
        status = Gtk.Label(label="", xalign=0)
        status.get_style_context().add_class("dim")
        status.set_line_wrap(True)
        inner.pack_start(erow, False, False, 0)
        inner.pack_start(status, False, False, 0)
        rev.add(inner)
        wrap.pack_start(rev, False, False, 0)

        ui = dict(rev=rev, erow=erow, entry=entry, go=go, status=status)
        btn.connect("clicked", lambda *_: self.on_row(n, ui))
        entry.connect("activate", lambda *_: self.submit(n, ui))
        go.connect("clicked", lambda *_: self.submit(n, ui))
        return wrap

    def show_password(self, ui):
        # GTK3 ignores show_all() on a no_show_all widget, so clear the flag first
        ui["erow"].set_no_show_all(False)
        ui["erow"].show_all()
        ui["entry"].grab_focus()

    def on_row(self, n, ui):
        if n["active"]:
            return
        if self.open_rev is not None and self.open_rev is not ui["rev"]:
            self.open_rev.set_reveal_child(False)
        self.open_rev = ui["rev"]
        ui["status"].set_text("")
        ui["rev"].set_reveal_child(True)
        if n["secure"] and not n["saved"]:
            self.show_password(ui)
        else:
            self.connect_to(n, None, ui)

    def submit(self, n, ui):
        pw = ui["entry"].get_text()
        if n["secure"] and len(pw) < 8:
            self.set_status(ui, "Password must be at least 8 characters", True)
            return
        self.connect_to(n, pw or None, ui)

    def set_status(self, ui, text, err=False):
        ctx = ui["status"].get_style_context()
        ctx.remove_class("err"); ctx.remove_class("dim")
        ctx.add_class("err" if err else "dim")
        ui["status"].set_text(text)

    def connect_to(self, n, pw, ui):
        self.set_status(ui, "Connecting\u2026")
        ui["entry"].set_sensitive(False)
        ui["go"].set_sensitive(False)

        def work():
            if pw and n["saved"]:
                run("con", "delete", "id", n["ssid"])   # replace the old saved password
            if pw:
                rc, out = run("dev", "wifi", "connect", n["ssid"], "password", pw)
            elif n["saved"]:
                rc, out = run("con", "up", "id", n["ssid"])
            else:
                rc, out = run("dev", "wifi", "connect", n["ssid"])
            if rc != 0:
                run("con", "delete", "id", n["ssid"])   # don't leave a broken profile
            return rc, out

        def done(res):
            rc, out = res
            if rc == 0:
                subprocess.Popen(["notify-send", "-t", "3000", "Wi-Fi", f"Connected to {n['ssid']}"])
                Gtk.main_quit()
                return
            ui["entry"].set_sensitive(True)
            ui["go"].set_sensitive(True)
            if n["secure"]:
                self.set_status(ui, "Could not connect. Check the password.", True)
                self.show_password(ui)
            else:
                self.set_status(ui, "Could not connect.", True)
        bg(work, done)


if __name__ == "__main__":
    Popup()
    Gtk.main()
