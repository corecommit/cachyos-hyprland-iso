#!/usr/bin/env python3
"""Shared by the Python popups (wifi/volume/power/clipboard) of the python ISO
flavour: sizes and placement from ~/.config/hypr/popups.conf, extra styling from
~/.config/hypr/popup-user.css — the same files and keys the Rust hypr-ui reads
(hypr-ui/src/conf.rs and ui.rs), so both flavours of the ISO are configured the
same way.

popups.conf is a plain key=value file; a missing file, a missing key or a value
that is not a whole number keeps the default. The defaults are the constants at
the top of each popup script (unchanged from the pre-Rust ISO):

    top_margin=52      right_margin=150
    wifi_width=340      wifi_max_height=300
    volume_width=360    volume_max_height=420   volume_max=100
    power_width=260
    clipboard_width=400 clipboard_height=500   clipboard_gap=14
    clipboard_items=60

popup-user.css is read again every time a popup opens and applied on top of the
built-in CSS (a second provider at a slightly higher priority, so your rules
win). A CSS syntax error in it is ignored, like in the Rust hypr-ui.

Each popup calls, near the top:

    import popupconf
    popupconf.override(globals(), "wifi")      # or volume / power / clipboard

and, after registering its own CSS provider:

    popupconf.load_user_css()
"""
import os

# popups.conf key -> the constant it overrides, per popup.
# (WIDTH means the popup's own width; the conf key name carries which popup.)
_MAP = {
    "wifi": {
        "top_margin": "TOP_MARGIN",
        "right_margin": "RIGHT_MARGIN",
        "wifi_width": "WIDTH",
        "wifi_max_height": "MAX_LIST_HEIGHT",
    },
    "volume": {
        "top_margin": "TOP_MARGIN",
        "right_margin": "RIGHT_MARGIN",
        "volume_width": "WIDTH",
        "volume_max_height": "MAX_LIST_HEIGHT",
        "volume_max": "MAX_VOLUME",
    },
    "power": {
        "power_width": "WIDTH",
    },
    "clipboard": {
        "clipboard_width": "WIDTH",
        "clipboard_height": "CARD_H",
        "clipboard_gap": "GAP",
        "clipboard_items": "MAX_ITEMS",
    },
}

_CONF = None


def _read():
    """key=value lines from ~/.config/hypr/popups.conf; {} when there is none."""
    out = {}
    try:
        with open(os.path.expanduser("~/.config/hypr/popups.conf")) as f:
            for line in f:
                line = line.strip()
                if not line or line.startswith("#") or "=" not in line:
                    continue
                k, v = line.split("=", 1)
                out[k.strip()] = v.strip()
    except OSError:
        pass
    return out


def conf():
    """The parsed popups.conf (read once per popup process)."""
    global _CONF
    if _CONF is None:
        _CONF = _read()
    return _CONF


def override(g, which):
    """Rebind this popup's size constants (in its globals dict g) from
    popups.conf. Values that are not whole numbers are ignored, keeping the
    built-in default."""
    c = conf()
    for key, name in _MAP[which].items():
        if key in c:
            try:
                g[name] = int(c[key])
            except ValueError:
                pass
    return None


def load_user_css():
    """Apply ~/.config/hypr/popup-user.css on top of the popup's built-in CSS.

    Called after the popup registered its own provider; the user file wins
    because its provider is registered later at a higher priority. Read again
    every time a popup opens, exactly like the Rust hypr-ui does.
    """
    try:
        with open(os.path.expanduser("~/.config/hypr/popup-user.css")) as f:
            css = f.read()
    except OSError:
        return
    # Imported here, after the popup already did gi.require_version().
    from gi.repository import Gdk, Gtk
    try:
        prov = Gtk.CssProvider()
        prov.load_from_data(css.encode())
    except Exception:
        return          # your CSS not parsing must not break the popup
    scr = Gdk.Screen.get_default()
    if scr is not None:
        Gtk.StyleContext.add_provider_for_screen(
            scr, prov, Gtk.STYLE_PROVIDER_PRIORITY_USER + 1)
