# Changelog

## [1.0.5] - 2026-11-07

#### New
- **Shortcuts cheatsheet.** A window with every keyboard shortcut pops up the first time you log in. Press **Super+F1** any time to open it again.
- **Switch keyboard layouts.** US, German, French and Spanish layouts are ready to use. Press **Super+Alt+Space**, or click the layout indicator in the top bar, to cycle through them.
- **Automatic timezone.** On first boot (with internet) the clock sets itself to your timezone instead of staying on UTC. If it can't tell, you get a notification with the command to set it yourself.
- **New lock screen look.** **Super+L** now shows a blurred wallpaper with a big clock and the date. There is no password on the live session, so just press Enter to unlock.
- **Nicer notifications.** Pop-ups match the dark theme and appear in the bottom-right corner.
- **Welcome message and README.** The login message explains how to try and install the system (`sudo calamares`), and a short README is included with the ISO.

#### Improved top bar
- **All 9 workspaces are shown.** Before, only the first five appeared, so workspaces 6 to 9 were invisible. The buttons are clickable.
- **More at a glance.** The bar now shows brightness, CPU usage, memory usage and your keyboard layout, plus a button that keeps the screen awake.
- **Scroll over the brightness indicator** to change brightness.
- **Click Wi-Fi** to open a proper network window instead of a text menu.
- **Click the volume** to open a full audio mixer where you can pick speakers and test your microphone. Right-click the volume to mute.

#### New apps included
- Mousepad (text editor), imv (image viewer) and btop (system monitor)
- Pavucontrol (audio mixer) and the Network Connections editor
- Swappy, to draw on and annotate screenshots
- File Roller, so you can open and extract archives straight from the file manager
- Emoji support everywhere
- Your Documents, Downloads and other standard folders are created for you
- Copied text stays available after you close the app you copied it from

#### Other changes
- Removed zram swap. The live system already runs from RAM, so extra compressed swap wasn't helping. Logs, the package cache and background timers are still kept small.
- The ISO is slightly larger because of the new apps and fonts.

## [1.0.4] - 2026-11-06

#### New
- **Brave browser** replaces Firefox. Open it with **Super+B** or from the app launcher. It starts in dark mode, runs natively on Wayland, and uses a small cache and fewer processes to go easy on RAM.
- **Power menu.** Press **Super+X**, click the power button in the top bar, or pick "Power menu" in the command palette to lock, log out, reboot or shut down from a clean icon menu.

#### Improved
- **Redesigned top bar.** Rounded panels with a launcher button, the current window title, workspace buttons, memory usage, a clock with the date, and the power button.
- **Dark everywhere.** GTK apps such as the file manager now use the dark theme, and the app launcher has a new dark, rounded look.
- **Command palette** (**Super+Space**) now has an icon in front of every entry and a clearer search prompt.
- **Terminal** uses a plain black background with a lighter, cleaner prompt.

#### Removed
- Firefox.

## [1.0.3]

#### New
- **Hyprland is the default desktop again**, with a polished look: a top bar with clock, system stats, workspaces and tray, a blurred dark-blue wallpaper, and the kitty terminal, fish shell and fastfetch ready to go.
- **Screenshot shortcuts.** **Super+Shift+S** or **Print** selects an area, and **Super+Print** captures the whole screen. Screenshots are copied to the clipboard and saved in `~/screenshots/`.

#### Improved
- Installing software works right away once you're online. No more running `sudo pacman -Sy` first.
- Smoother on low-memory machines: compressed swap in RAM (zram) is enabled, logs and crash dumps no longer eat memory, and downloaded packages are cleaned up after each install.
- Fewer background tasks running in the live session.

#### Fixed
- The touchpad error "unknown config key input.touchpad.tap-to-click". Tap to click works.

## [1.0.2]

#### Changed
- Tried KDE Plasma alongside Hyprland. It made the ISO about 2.16 GB, so Hyprland is the only desktop again.

#### Tested
- Real hardware, including the "copy to RAM" boot option.

## [1.0.1]

#### Improved
- Works better in virtual machines, including slow ones and common graphics setups (virtio, VMware, Hyper-V).
- Networking is on by default, so Wi-Fi and `nmtui` work out of the box.

#### Fixed
- The boot getting stuck in an emergency shell with "Failed to load your GPU driver(s)".
- A startup error about ModemManager.

## [1.0.0]

#### First release
- An unofficial CachyOS live ISO that boots into a Hyprland desktop instead of KDE Plasma.
- Comes with Waybar, Wofi, Mako, kitty, fish, fastfetch, Firefox and the Thunar file manager.
- Graphical login screen.
- The CachyOS installer is included.

#### Fixed
- ISOs that failed to boot with a missing kernel error.
