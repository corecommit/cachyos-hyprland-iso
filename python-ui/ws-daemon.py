#!/usr/bin/env python3
# Keeps the bar's 1-2-3-4 workspace buttons in sync. Listens to Hyprland's event
# socket, writes the state to $XDG_RUNTIME_DIR/ws-state and pokes waybar
# (signal 8) only when something changed. ws-state.sh reads that file.
import glob, json, os, socket, subprocess, time

RT = os.environ.get("XDG_RUNTIME_DIR", "/tmp")
STATE = os.path.join(RT, "ws-state")
EVENTS = {"workspace", "workspacev2", "createworkspace", "createworkspacev2",
          "destroyworkspace", "destroyworkspacev2", "openwindow", "closewindow",
          "movewindow", "movewindowv2", "moveworkspace", "moveworkspacev2",
          "focusedmon", "focusedmonv2"}


def hc(what):
    out = subprocess.run(["hyprctl", "-j", what], capture_output=True, text=True, timeout=3).stdout
    return json.loads(out)


def waybar_ready():
    # An RT signal sent to a waybar that is still starting up kills it, so only
    # signal one that has been running for a few seconds.
    for _ in range(8):
        try:
            out = subprocess.run(["ps", "-o", "etimes=", "-C", "waybar"],
                                 capture_output=True, text=True, timeout=2).stdout.split()
        except Exception:
            return False
        if not out:
            return False
        if min(int(x) for x in out) >= 3:
            return True
        time.sleep(0.5)
    return False


def update(last):
    try:
        active = int(hc("activeworkspace")["id"])
        occ = sorted({int(w["id"]) for w in hc("workspaces") if w.get("windows", 0) > 0})
    except Exception:
        return last
    text = 'active=%d\nocc=" %s "\n' % (active, " ".join(map(str, occ)))
    if text == last:
        return last
    tmp = STATE + ".tmp"
    with open(tmp, "w") as f:
        f.write(text)
    os.replace(tmp, STATE)
    if waybar_ready():
        subprocess.run(["pkill", "-RTMIN+8", "waybar"])
    return text


def sock_path():
    sig = os.environ.get("HYPRLAND_INSTANCE_SIGNATURE", "")
    cands = [os.path.join(RT, "hypr", sig, ".socket2.sock")] if sig else []
    cands += sorted(glob.glob(os.path.join(RT, "hypr", "*", ".socket2.sock")),
                    key=os.path.getmtime, reverse=True)
    for c in cands:
        if os.path.exists(c):
            return c
    return None


def main():
    path = None
    for _ in range(20):
        path = sock_path()
        if path:
            break
        time.sleep(0.5)
    if not path:
        return
    s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    s.connect(path)
    last, buf, dirty = "", b"", True
    while True:
        s.settimeout(0.08 if dirty else None)   # wait for a burst of events to settle
        try:
            data = s.recv(4096)
        except socket.timeout:
            last = update(last)
            dirty = False
            continue
        if not data:
            return
        buf += data
        *lines, buf = buf.split(b"\n")
        for line in lines:
            if line.split(b">>", 1)[0].decode("ascii", "ignore") in EVENTS:
                dirty = True


if __name__ == "__main__":
    main()
