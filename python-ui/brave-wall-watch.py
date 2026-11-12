#!/usr/bin/env python3
"""Make a new Brave wallpaper every time a new tab opens.

Brave opens the picture in sanitized_background_images each time it shows a new tab page.
This watches that folder (inotify) and, after each open, swaps the picture for a new random
one with brave-wall.sh, so the next new tab looks different. The old picture is replaced.
"""
import ctypes, ctypes.util, os, struct, subprocess, time

D = os.path.expanduser("~/.config/BraveSoftware/Brave-Browser/Default/sanitized_background_images")
SCRIPT = os.path.expanduser("~/.config/hypr/brave-wall.sh")
IN_OPEN = 0x20
EVT = struct.Struct("iIII")


def main():
    libc = ctypes.CDLL(ctypes.util.find_library("c") or "libc.so.6", use_errno=True)
    fd = libc.inotify_init()
    if fd < 0:
        return
    wd = -1
    while True:
        if wd < 0:
            if os.path.isdir(D):
                wd = libc.inotify_add_watch(fd, D.encode(), IN_OPEN)
            if wd < 0:
                time.sleep(5)
                continue
        data = os.read(fd, 4096)
        i, new_tab = 0, False
        while i + EVT.size <= len(data):
            _, _, _, ln = EVT.unpack_from(data, i)
            name = data[i + EVT.size:i + EVT.size + ln].split(b"\0", 1)[0]
            i += EVT.size + ln
            if name and not name.endswith(b".tmp"):
                new_tab = True
        if new_tab:
            time.sleep(1.0)                      # let Brave finish reading the current picture
            subprocess.run(["sh", SCRIPT], check=False)
            time.sleep(0.5)
            # drop the opens caused by Brave noticing the new file, so one tab = one picture
            os.set_blocking(fd, False)
            try:
                while os.read(fd, 4096):
                    pass
            except BlockingIOError:
                pass
            os.set_blocking(fd, True)


if __name__ == "__main__":
    main()
