# Changelog

## [1.2.0] - 2026-11-10

### Added
- New Wi-Fi popup. Click Wi-Fi in the bar to open a panel with an on/off switch, a refresh button and the list of nearby networks (signal bars, a lock on secured ones, "connected" on the current one). Saved networks connect with one click; new ones ask for the password right under the network. Click again, press Esc or click outside to close it. It replaces the old `nmtui` window.
- New volume popup, replacing pavucontrol. It has a slider and mute button for the output, one for the microphone, and one for every app that is playing. With more than one output device you can switch between them (playing apps move along). It follows changes made with the volume keys.
- New power menu popup: Lock, Log out, Suspend, Reboot and Shut down. Use the arrow keys and Enter, click, or press the letter shown next to each entry (L, O, S, R, P).
- Clipboard history popup. Press **Super+C** (or pick "Clipboard: History" in the command palette) to see what you copied, with text as a preview and images as thumbnails. Type to search, use the arrow keys and Enter (or click) to paste an entry into the window you were in. Delete or the ✕ removes one entry, and "Clear all" (click twice to confirm) empties the history. It opens next to the mouse pointer and stays on screen.
- Redesigned tooltips for the bar. Wi-Fi shows the network, signal in % and dBm, band, IP, gateway, interface and live speeds. Ethernet, "Not connected" and "Wi-Fi is off" got the same look. Volume shows the output device, volume and microphone level. The clock tooltip is a calendar with week numbers: scroll to change month, right-click to switch the view, middle-click to jump back to today.
- The bar now shows the Wi-Fi icon with "off" while Wi-Fi is switched off.
- "System: Change Keyboard Layout" in the command palette.
- `wtype` is now included (it lets the clipboard popup paste for you).

### Changed
- Double-clicking the clock now opens a searchable timezone list, sorted by UTC offset, with the current time in each place. "Custom offset…" at the top still lets you type `5:30`, `5`, `-8` or a name like `Asia/Tokyo`. If setting the timezone fails, the notification now shows the reason.
- The keyboard layout menu is simpler and opens as a normal window.
- The app launcher, command palette and other menus are wider (650 px) and have rounder corners.
- The Keyboard Shortcuts window (Super+F1) now lists Super+C and the new Wi-Fi and volume popups.

### Removed
- pavucontrol and the window rule that placed it under the volume button.

### Fixed
- The Wi-Fi icon disappeared from the bar when Wi-Fi was turned off, so there was no way to turn it back on. It stays visible now and the popup has the switch.
- The power menu did not show up.
- The keyboard layout menu did not work.

### Notes
- Apps can't see where the text cursor is on Wayland, so the clipboard popup opens at the mouse pointer instead.
- Clipboard history is kept in RAM and is gone after a reboot. `save-progress` does not save it. It stores everything you copy, passwords included, so use "Clear all" when needed.
- Pasting uses Ctrl+V, or Ctrl+Shift+V in terminals.
- The Wi-Fi and volume popups sit at the top right under the bar. If one looks misplaced on your screen, change `RIGHT_MARGIN` at the top of `~/.config/hypr/wifi-popup.py` or `volume-popup.py`.

## [1.1.4] - 2026-11-08

### Added
- `updateos` command: update the running system from a USB without rebooting. Run it in a terminal, then plug in a USB flashed with the new ISO, or one that has the `.iso` file on it. Your files, saved Wi-Fi networks, timezone, wallpaper and colours stay. The bar, keybinds, launcher and notification config are refreshed from the new version (your old copies are backed up in `~/.local/share/update-backup/`), and the bar restarts by itself.
- `--help` (or `-h`) for `save-progress` and `updateos`. It shows what the command does and its options, and works without a password. An unknown option now prints the help instead of being ignored.
- Brave gets a new random wallpaper on every new tab. It is made by the same generator as the desktop wallpaper and placed in `~/.config/BraveSoftware/Brave-Browser/Default/sanitized_background_images/`. The old picture is deleted each time, so only one ever exists. A new one is also made at login, on Super+Shift+W and when Brave starts.
- After a toram boot restores your saved files, it now tells you when the USB is free to unplug.
- `rsync` is now included (needed by `updateos`).

### Changed
- New top bar, made to match the reference screenshot: a flat dark bar (`#282828`) with a steel-blue frame, a small gap to the top, left and right and slightly rounded corners. Every item is a small rounded pill (`#413d3a`) with gold text and icons. Left side: disk usage, memory (percent, hover for used / total), CPU and battery. Middle: all 4 workspace buttons (the active one is filled gold). Right side: tray, keyboard layout and brightness (only when relevant), volume, Wi-Fi with live network speed, and the clock with the date. The window title is no longer in the bar.
- 4 workspaces instead of 3. Super+1 to 4, Super+Shift+1 to 4, Super+, / Super+. and Super+scroll cycle through all of them. All four are always shown in the bar.
- New colours for the whole system: primary is now gold `#c7ab7a` (was lavender `#b4befe`) and secondary is steel blue `#819299` (was pink `#f5a6d7`). They are used for the bar, window borders (active = steel blue), lock screen, notifications, the app launcher and terminal cursor/selection. Backgrounds are now `#1d1d1d` (base), `#282828` (bar) and `#413d3a` (pills). Super+P (wallpaper colours) still replaces them with colours from the picture.
- The volume, language and timezone popups now open right under the floating bar.
- Better wallpaper generator: three looks (soft glows, flowing aurora ribbons, smooth mesh gradient) in matching colour harmonies, with a light vignette and fine grain so there is no colour banding. Used for the desktop and for Brave.
- Brave: the search widget on the new tab page is now off too (top sites, clock, news, cards and sponsored images were already off).
- README, welcome message and the Keyboard Shortcuts window (Super+F1) updated.

### Removed
- The launcher, power and idle buttons from the top bar. Super+Space, Super+X and the command palette (Super+Shift+Space) still work.

### Fixed
- `updateos` stopped with `rsync: --numeric-owner: unknown option` and copied no system files. It now uses the correct `--numeric-ids`.

### Notes
- Open programs keep running the old version after `updateos` until you restart them, and a new kernel only starts after a reboot.
- Your current ISO does not have `updateos`, so boot this version normally once. Later updates can use it.
- Brave only shows the generated picture once "Use your own" is picked for the background in Brave's new tab settings (Customize). Pick it once and it stays.

## [1.1.3] - 2026-11-08

### Added
- Brave gets a new random wallpaper on its new tab page every time it is opened.
- `save-progress` now also saves your saved Wi-Fi networks and your timezone, and restores them at boot.

### Changed
- `save-progress` now lists every file it skips for being over 10 MB, with its full path and size, instead of only a count.
- When `save-progress` overwrites a save, it deletes all the old saved data and writes the new one in its place. The old save is kept until the new one is complete, and if there isn't room for both you're asked whether to delete it first.
- Corners are less rounded across the desktop: windows, the bar, the app launcher and notifications.
- README updated: Brave, the current shortcuts, and a new "Saving your progress" section.

## [1.1.2] - 2026-11-07

### Added
- `save-progress` command: run it in a terminal to save your home folder (files up to 10 MB each) to the USB you booted from. Next boot the ISO finds the saved data and restores it automatically ("FOUND DATA! TRANSFERRING OVER..."), so browser logins and files survive reboots. Works with USBs written by balenaEtcher / Rufus DD mode (creates a small save partition) and with Ventoy / Rufus ISO mode (saves a folder on the USB).
- Inter is now the UI font (bar, launcher, notifications, lock screen, GTK apps).
- Chinese text now displays correctly (WenQuanYi Micro Hei is included as the fallback font).
- The welcome box and `README.txt` now explain how to keep your files.

### Changed
- Brave now uses `--password-store=basic` so logins saved with `save-progress` open again after a reboot.

### Removed
- Brave's custom random-wallpaper new tab page and homepage override. Brave starts as normal, with its sponsored images, widgets and news still turned off.

### Notes
- The ISO is about 20 MB larger because of the new fonts.
- The save is not encrypted. Keep the USB safe. Installed programs and system settings are not saved.

## [1.1.1] - 2026-11-07

### Added
- Workspace buttons 1 2 3 in the top bar; the active workspace is highlighted and workspaces with windows are brighter.
- A new random wallpaper at every login, so it is different each time. Press Super+Shift+W for another one.
- Brave opens a plain new tab page showing a freshly generated random wallpaper on every start (the old one is deleted). Sponsored images, top sites, Brave News, the clock and cards are turned off.
- Thunar file manager, with gvfs (trash and mounts) and tumbler (thumbnails).

### Changed
- Super+Space now opens the same app launcher as the square button in the bar. Super+D does the same, and pressing it again closes it.
- Thunar replaces Nautilus as the file manager (Super+E, the command palette).
- The Keyboard Shortcuts window (Super+F1) is now a floating, colour-coded, searchable terminal window.
- Only workspaces 1 to 3 exist. Super+, / Super+. and Super+scroll cycle between them.

### Removed
- anyrun and its AUR build step, which makes builds faster.
- Nautilus.

### Fixed
- The file picker for Super+P could open tiled instead of floating on newer zenity.

## [1.1.0] - 2026-11-07

#### New
- **Wallpaper colours.** Press **Super+P**, pick any image, and it becomes your wallpaper. The top bar and the terminal are recoloured to match it. You can also run it from the command palette ("System: Wallpaper theme").
- **anyrun launcher.** **Super+Space** now opens anyrun, a fast app launcher. If anyrun can't be included in a build, Super+Space opens the command palette instead.
- **Set your timezone from the clock.** Double-click the clock in the top bar and type a UTC offset (`5:30`, `5`, `-8`) or a zone name such as `Asia/Tokyo`. Hovering over the clock shows the full date and a calendar.
- **Keyboard layout menu.** Click the layout indicator in the top bar to pick US, German, French or Spanish from a dropdown. **Super+Alt+Space** still cycles through them.
- **Your btop settings.** btop starts with the horizon colour theme and a ready-made layout.

#### Changed
- **Nautilus replaces Thunar** as the file manager (**Super+E**). Nautilus can extract and compress archives by itself, and File Roller is still included.
- **Command palette moved to Super+Shift+Space**, since Super+Space is now the launcher.
- **Top bar menus open under their button.** The power menu, Wi-Fi list, volume mixer and layout menu all appear right below the button you clicked.
- **Simpler power menu.** It now uses the same look as the app launcher and command palette.
- **Wi-Fi click** now opens a compact floating Wi-Fi list that is ready to connect. Click again to close it.
- **Workspace buttons removed** from the top bar to give it more room. Workspaces still work with the usual shortcuts.
- **The clock** no longer switches to the date when clicked, because the date is in its tooltip.
- The shortcuts cheatsheet, welcome message and terminal greeting list the new shortcuts.

#### Removed
- Thunar and its archive plugin.
- The separate power menu stylesheet.

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
