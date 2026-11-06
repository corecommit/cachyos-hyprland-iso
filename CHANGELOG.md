# Changelog

## 1.0.3

- Hyprland is the default desktop again, with a polished look: a top bar with clock, system stats, workspaces and tray, a blurred dark-blue wallpaper, and the kitty terminal, fish shell and fastfetch ready to go.
- New screenshot shortcuts: **Super+Shift+S** to select an area, **Print** for the same, and **Super+Print** for the whole screen. Screenshots are copied to the clipboard and saved in `~/screenshots/`.
- Installing software now works right away once you're online. No more running `sudo pacman -Sy` first.
- Fixed the touchpad error "unknown config key input.touchpad.tap-to-click". Tap to click works.
- Smoother on low-memory machines: compressed swap in RAM (zram) is enabled, logs and crash dumps no longer eat memory, and downloaded packages are cleaned up after each install.
- Fewer background tasks running in the live session.

## 1.0.2

- Tried KDE Plasma alongside Hyprland. It made the ISO about 2.16 GB, so Hyprland is the only desktop again.
- Tested on real hardware, including the "copy to RAM" boot option.

## 1.0.1

- Fixed the boot getting stuck in an emergency shell with "Failed to load your GPU driver(s)".
- Works better in virtual machines, including slow ones and common graphics setups (virtio, VMware, Hyper-V).
- Fixed a startup error about ModemManager.
- Networking is on by default, so Wi-Fi and `nmtui` work out of the box.

## 1.0.0

- First release: an unofficial CachyOS live ISO that boots into a Hyprland desktop instead of KDE Plasma.
- Comes with Waybar, Wofi, Mako, kitty, fish, fastfetch, Firefox and the Thunar file manager.
- Graphical login screen.
- The CachyOS installer is included.
- Fixed ISOs that failed to boot with a missing kernel error.
