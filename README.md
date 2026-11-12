<div align="center">

# CachyOS Hyprland ISO

**An unofficial CachyOS live ISO that boots into a tiling Hyprland desktop instead of KDE Plasma.**

Built automatically from the upstream CachyOS-Live-ISO by a GitHub Actions workflow.

<br>

[![Build](https://img.shields.io/github/actions/workflow/status/corecommit/cachyos-hyprland-iso/cachyos-hyprland.yml?style=for-the-badge&logo=githubactions&logoColor=white&label=BUILD&labelColor=1e1e2e)](https://github.com/corecommit/cachyos-hyprland-iso/actions/workflows/cachyos-hyprland.yml)
[![Release](https://img.shields.io/github/v/release/corecommit/cachyos-hyprland-iso?style=for-the-badge&logo=github&logoColor=white&label=RELEASE&labelColor=1e1e2e&color=b4befe)](https://github.com/corecommit/cachyos-hyprland-iso/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/corecommit/cachyos-hyprland-iso/total?style=for-the-badge&logo=download&logoColor=white&label=DOWNLOADS&labelColor=1e1e2e&color=a6e3a1)](https://github.com/corecommit/cachyos-hyprland-iso/releases)
[![Stars](https://img.shields.io/github/stars/corecommit/cachyos-hyprland-iso?style=for-the-badge&logo=github&logoColor=white&label=STARS&labelColor=1e1e2e&color=f9e2af)](https://github.com/corecommit/cachyos-hyprland-iso/stargazers)

![Arch Linux](https://img.shields.io/badge/BASE-ARCH_LINUX-1793D1?style=for-the-badge&logo=archlinux&logoColor=white&labelColor=1e1e2e)
![CachyOS](https://img.shields.io/badge/DISTRO-CACHYOS-08A283?style=for-the-badge&logo=archlinux&logoColor=white&labelColor=1e1e2e)
![Hyprland](https://img.shields.io/badge/WM-HYPRLAND-58E1FF?style=for-the-badge&logo=hyprland&logoColor=white&labelColor=1e1e2e)
![Wayland](https://img.shields.io/badge/DISPLAY-WAYLAND-FFBC00?style=for-the-badge&logo=wayland&logoColor=white&labelColor=1e1e2e)

[Download](https://github.com/corecommit/cachyos-hyprland-iso/releases/latest) |
[Two flavours](#two-flavours-rust-and-python) |
[Shortcuts](#keyboard-shortcuts) |
[Saving your progress](#saving-your-progress) |
[Updating](#updating-without-rebooting) |
[Warnings](#warnings-read-before-booting) |
[How it works](#how-it-works) |
[Build your own](#build-your-own)

</div>

---

## Overview

The official CachyOS live ISO ships KDE Plasma. This project takes that ISO profile, strips out Plasma and a long list of tools a live session does not need, and replaces them with:

| Component | What it is |
|---|---|
| Compositor | Hyprland, started by greetd with autologin |
| Top bar | Waybar, a flat floating bar of small rounded pills. Left: disk, memory, CPU, battery. Middle: workspaces 1-4. Right: tray, keyboard layout and brightness (only when relevant), volume, Wi-Fi with live speed, clock with date |
| Popups | Wi-Fi, volume mixer, power menu and clipboard history, opened from the bar or with a shortcut (see [Popups](#popups)) |
| Terminal | kitty with fish and fastfetch |
| Launcher | wofi, plus a command palette (`Super + Shift + Space`) for apps, audio, display, clipboard, Wi-Fi and system actions |
| File manager | Thunar |
| Browser | Brave, with a new random wallpaper on its new tab page every time it is opened |
| Wallpaper and colours | A new random dark wallpaper at every login (`Super + Shift + W` for another). Gold and steel blue by default; `Super + P` picks a wallpaper and recolours the bar and terminal from it |
| Notifications | mako |
| Lock screen / idle | hyprlock, hypridle |
| Kernel | linux-cachyos (the LTS kernels are removed) |
| Saving | `save-progress` keeps your files, Wi-Fi networks and timezone on the USB stick between boots |
| Updating | `updateos` updates the running system from a USB stick without rebooting |
| Installer | The CachyOS installer is kept so you can install from the live session |

Nothing here is a fork of CachyOS. The workflow downloads the upstream profile on every build and patches it, so upstream updates flow in automatically.

## Two flavours: Rust and Python

Every release has two ISOs. They are the same system, built from the same workflow. The only difference is the small programs behind the popups and helpers.

| | `rust` (the default) | `python` |
|---|---|---|
| File | `cachyos-hyprland-<version>-rust-x86_64.iso` | `cachyos-hyprland-<version>-python-x86_64.iso` |
| Popups and helpers | Two small compiled programs, `hypr-ui` and `hypr-helper` | The original Python programs, reached through shell wrappers with the same names |
| Python on the ISO | Only if another package pulls it in | Included (`python`, `python-gobject`) |
| Size and speed | Smaller, popups open quickly | Larger, popups open a little slower because the interpreter starts first |

Everything else is identical: the bar, keybinds, themes, `save-progress` and `updateos`. If you are not sure which to pick, take **rust**. The Python flavour exists so a change can still ship in Python while the Rust port catches up, and so you can read and edit the popup code without rebuilding the ISO (the files are in `/usr/share/cachyos-hyprland/py/`).

The release notes list both ISOs with their sizes and SHA-256 checksums. If one flavour failed to build, or was too big to attach, the notes say so.

## Warnings (read before booting)

> **This is an unofficial, community-built ISO. It is not affiliated with or supported by the CachyOS project. Do not ask the CachyOS team for help with it.**

**Memory**

- **4 GB of RAM is the practical minimum.** The live session keeps all changes (installed packages, downloads, settings) in RAM. A browser plus a few apps can fill it quickly.
- **8 GB or more is recommended**, especially if you plan to use Brave, install packages in the live session, or run the installer while doing other things.
- If the system starts freezing or killing programs, you are almost certainly out of memory. Close Brave first.
- Everything done in the live session is lost on reboot unless you run `save-progress` (see [Saving your progress](#saving-your-progress)). Installed packages and files over 10 MB are never saved.

**Storage and boot speed**

- The image is read from the USB stick while you use it. A slow stick makes the whole system slow, including app launches. Use a USB 3.0 stick on a USB 3.0 port if you can.
- The first boot can take a while on old hardware or slow media.

**Graphics**

- Hyprland needs working hardware graphics acceleration. Intel and AMD GPUs are the safest choice.
- NVIDIA and hybrid-graphics laptops are untested. If the desktop does not appear, try another boot entry or report an issue with your GPU model.
- In a virtual machine, use a virtio-gpu display with 3D acceleration if available. Without it, expect a slow or blank screen.

**Boot settings**

- Disable Secure Boot. The ISO is not signed.
- Both UEFI and BIOS boot entries exist, but UEFI is the most tested path.

**Installing to disk**

- Back up your data. The installer can erase disks. Check the target disk twice.
- The installed system is set up by the CachyOS installer, so it may not carry over the Waybar, kitty and Hyprland setup from the live session. This has not been fully verified. If the installed desktop looks different, copy `/etc/skel/.config` from the live session into your new home folder.

## Quick start

1. Download an ISO and its `.sha256` file from the [Releases](https://github.com/corecommit/cachyos-hyprland-iso/releases/latest) page. Take the `rust` ISO unless you have a reason to want the `python` one (see [Two flavours](#two-flavours-rust-and-python)).
2. Verify it:

   ```bash
   sha256sum -c cachyos-hyprland-*-rust-x86_64.iso.sha256
   ```

3. Write it to a USB stick. This erases the stick.

   ```bash
   sudo dd if=cachyos-hyprland-<version>-rust-x86_64.iso of=/dev/sdX bs=4M status=progress oflag=sync
   ```

   Replace `/dev/sdX` with your USB device (check with `lsblk`). Using the wrong device destroys data. Graphical tools such as Rufus (DD mode) or balenaEtcher also work.

4. Disable Secure Boot, boot from the stick, and wait for the desktop to appear.

### Connecting to Wi-Fi

Click Wi-Fi in the top bar. The popup has an on/off switch, a refresh button and the list of nearby networks (signal bars, a lock on secured ones, "connected" on the current one). Saved networks connect with one click; a new one asks for its password right under the network.

Or from a terminal (run `save-progress` afterwards to keep the network for next time):

```bash
nmcli device wifi connect "SSID" password "PASSWORD"
```

## Keyboard shortcuts

`Super` is the Windows key. Press `Super + F1` at any time to open the cheatsheet.

| Keys | Action |
|---|---|
| `Super + Enter` | Open terminal (kitty) |
| `Super + Space` or `Super + D` | Open launcher (wofi) |
| `Super + Shift + Space` | Command palette (apps, audio, display, clipboard, Wi-Fi, system) |
| `Super + B` | Browser (Brave) |
| `Super + E` | File manager (Thunar) |
| `Super + C` | Clipboard history |
| `Super + P` | Pick a wallpaper and recolour the bar and terminal |
| `Super + Shift + W` | New random wallpaper |
| `Super + Q` | Close window |
| `Super + F` | Fullscreen on or off |
| `Super + V` | Floating on or off |
| `Super + Arrow keys` | Move focus |
| `Super + Shift + Arrow keys` | Move window |
| `Super + Left-drag` | Move window with the mouse |
| `Super + Right-drag` | Resize window with the mouse |
| `Super + 1-4` | Go to workspace |
| `Super + Shift + 1-4` | Send window to workspace |
| `Super + .` and `Super + ,` | Next or previous workspace |
| `Super + Scroll` | Next or previous workspace |
| `Super + L` | Lock screen |
| `Super + X` | Power menu |
| `Super + Alt + Space` | Switch keyboard layout (us, de, fr, es) |
| `Print` or `Super + Shift + S` | Select a region: copy to clipboard and save to `~/screenshots` |
| `Super + Print` | Full-screen screenshot to `~/screenshots` |
| `Super + Shift + R` | Reload Hyprland config |
| `Super + Shift + E` | Exit Hyprland |

Volume and brightness keys work as normal media keys. Double-click the clock in the bar to set the timezone: a searchable list sorted by UTC offset, with the current time in each place. "Custom offset..." at the top lets you type `5:30`, `5`, `-8` or a name like `Asia/Kolkata`.

## Popups

| Popup | How to open | What it does |
|---|---|---|
| Wi-Fi | Click Wi-Fi in the bar | On/off switch, refresh, nearby networks, password entry |
| Volume | Click volume in the bar | Output and microphone sliders and mute, one slider per playing app, output device switch. Follows the volume keys |
| Power menu | `Super + X` | Lock, Log out, Suspend, Reboot, Shut down. Arrow keys and Enter, a click, or the letter shown next to each entry |
| Clipboard history | `Super + C` | Text as a preview, images as thumbnails. Type to search, arrow keys and Enter (or a click) to paste into the window you were in. Delete removes one entry; "Clear all" (click twice) empties the history |

Click again, press Esc or click outside to close a popup. The clipboard popup opens next to the mouse pointer, because apps cannot see the text cursor on Wayland.

Clipboard history lives in RAM only. It is gone after a reboot, `save-progress` does not save it, and it stores everything you copy, passwords included. Use "Clear all" when needed. Pasting uses `Ctrl + V`, or `Ctrl + Shift + V` in terminals.

**Moving and styling the popups.** Two files in `~/.config/hypr/` control them, and there is nothing to restart: change a value, save, and open the popup again.

- `popups.conf`: sizes and placement as `key=value` lines. Keys: `top_margin`, `right_margin`, `wifi_width`, `wifi_max_height`, `volume_width`, `volume_max_height`, `volume_max`, `power_width`, `clipboard_width`, `clipboard_height`, `clipboard_gap`, `clipboard_items`. Delete a line to use its default. If the Wi-Fi or volume popup looks misplaced on your screen, change `right_margin`.
- `popup-user.css`: your own CSS, applied on top of the built-in look every time a popup opens.

Both flavours read the same files with the same keys.

## Saving your progress

The live session runs from RAM, so by default everything is gone after a reboot. `save-progress` writes your files onto the USB stick you booted from and brings them back at the next boot.

```bash
save-progress
```

1. Plug in the USB you booted from. The command waits for it, and asks you to pick it if it cannot tell which one it is.
2. If the stick was written as a raw image (`dd`, balenaEtcher, Rufus "DD Image"), it offers to create a small `CPSAVE` partition in the free space after the ISO (at least 64 MiB free is needed). If the stick was made with Ventoy or Rufus "ISO mode", it saves into a `cachyos-hyprland-save` folder on the stick's biggest normal partition instead.
3. Close Brave when asked, so your logins are saved safely.

**What is saved**

- Your home folder, except caches and any file bigger than 10 MB. Every skipped file is listed at the end with its full path and size.
- Your saved Wi-Fi networks.
- Your timezone.

**Overwriting.** If the stick already holds a save, you are asked before it is replaced. The new save is written first, and only when it is complete is the old data (files, Wi-Fi, timezone and version) deleted and replaced, so a failed save never destroys the old one. If there is not enough room for both, you are asked whether to delete the old save first and then write the new one.

**Restoring.** At the next boot with the same stick, a message says `FOUND DATA! TRANSFERRING OVER...` and everything is put back before the desktop starts. When it is done, the message tells you the USB is free to unplug. If the save came from an older ISO version, your saved files are restored but the new ISO's Hyprland and Waybar config is kept.

Things to know:

- Saved Wi-Fi passwords are stored as NetworkManager keeps them, unencrypted. Anyone who can read the stick can read them.
- Installed packages are not saved.
- On a FAT-formatted stick a single save cannot be bigger than 4 GB.

## Updating without rebooting

`updateos` updates the running system from a USB stick without a reboot.

```bash
updateos
```

1. Plug in a USB stick flashed with the new ISO, or a normal stick that has the new `.iso` file on it. With no argument, `updateos` waits for it and lists the versions it finds. You can also pass an ISO file, a device or a folder.
2. It shows the running and the new version, checks that there is enough room, and asks before it changes anything (`-y` skips the questions).
3. It copies the new system files over the running ones. Your files, saved Wi-Fi networks, timezone, wallpaper and colours stay. The bar, keybinds, launcher, notification and terminal config are refreshed (your old copies are backed up in `~/.local/share/update-backup/`), and the bar, workspace buttons and Brave wallpaper watcher restart by themselves.

Things to know:

- Programs that are already open keep running the old version until you restart them. A new kernel only starts after a reboot.
- Packages you installed with pacman since boot are removed, just as a reboot would do.
- **Switching flavours works.** The version list shows the flavour, for example `1.2.1 (python)`, and the same version in the other flavour is offered as an update. When the flavour changes, the open popups and helpers of the old one are closed first, then the new ones take over.
- Images older than 1.2.1 do not record their flavour; they show only a version number and still update normally.
- It needs the live ISO, and it was added in 1.1.4, so an older ISO has to be booted normally once first.

## How it works

Everything happens in one GitHub Actions workflow: `.github/workflows/cachyos-hyprland.yml`. It has three jobs:

```
prepare  ->  build (rust | python, in parallel)  ->  finish
```

- **prepare** resolves the version and tag, warns if the Python code has changed more recently than the Rust code (see [Repository layout](#repository-layout)), and creates the GitHub Release when one is wanted. Doing this once, before the builds, means the two builds never race to create it.
- **build** runs twice, once per flavour, each on its own runner inside an `archlinux:base-devel` container. If one flavour fails, the other still builds and is released.
- **finish** writes the release notes once both are done.

Each build goes through these steps:

```
Clone upstream  ->  Patch profile  ->  Verify  ->  Build  ->  Check  ->  Publish
```

1. **Prepare.** Frees disk space on the runner, restores the pacman package cache, installs `archiso` and friends, sets up the CachyOS keyring and repositories, and creates a non-root `builder` user. For the rust flavour it also compiles `hypr-helper` and `hypr-ui`.
2. **Clone upstream.** Fetches [CachyOS/CachyOS-Live-ISO](https://github.com/CachyOS/CachyOS-Live-ISO). The commit used is printed in the log and the run summary so a good build can be pinned.
3. **Patch the profile.** Each patch is its own step and fails loudly if upstream changed and a change did nothing (a small `sedx` helper enforces this).
   - Package list: add Hyprland, Waybar, kitty and friends; remove Plasma, KDE apps, LTS kernels, guest tools, network tools and other things a live session does not need. The python flavour also adds `python` and `python-gobject`.
   - Display manager: swap Plasma's login manager for greetd, with an autologin straight into Hyprland.
   - Airootfs: write the Hyprland, Waybar, kitty and fish configs into `/etc/skel`, install the flavour's popups and helpers, generate a wallpaper, and remove Plasma-specific config files. A small `brave` wrapper makes a new wallpaper for Brave each time it is started.
   - Saving and updating: install `save-progress`, `updateos` and a boot service that restores your saved data before the desktop starts.
   - Boot robustness: remove dangling links to removed services, raise timeouts for slow or emulated machines, and make sure NetworkManager is enabled.
   - Boot entries: point GRUB, systemd-boot and syslinux at `linux-cachyos`, and remove the LTS entries.
   - Session launcher: a small script copies the default config into the home folder if it is missing, so the desktop always starts with these settings.
   - Size: the image is compressed with xz, and man pages, docs and info files are left out.
   - Stamp: the version and the flavour are written to `/etc/cachyos-hyprland-release`.
4. **Verify.** Checks that Plasma and the LTS kernel are really gone, that required config files and the flavour's programs exist with the right permissions, and that no boot entry still points at a removed kernel. The rust flavour also checks that no shipped script uses Python; the python flavour checks that every Python file compiles.
5. **Build.** Runs the upstream `buildiso.sh` as the builder user and keeps a full log, which is uploaded as an artifact if the build fails.
6. **Check.** Confirms every kernel named in a boot config exists in the final ISO, and checks the ISO size against the 2 GiB GitHub release limit.
7. **Publish.** When releasing, uploads the ISO and its checksum to the release. Otherwise, or if the ISO is 2 GiB or larger, it is kept as a workflow artifact for 1 day. If `CHANGELOG.md` has a section for the version (a `## [1.2.1]` heading, for example), the finish job uses it as the release description, followed by a table listing both ISOs.

### Boot test workflow

`.github/workflows/cachyos-hyprland-boottest.yml` boots an already built ISO in QEMU with UEFI firmware. It takes screenshots over QMP and checks that a desktop appears and stays up. The screenshots are uploaded as an artifact, so you can see what the machine showed without owning the hardware. It does not rebuild anything, so it finishes in minutes.

## Build your own

1. Fork this repository.
2. Open the **Actions** tab, choose **Build CachyOS Hyprland ISO**, and press **Run workflow**.
3. Download the two ISOs from the run's artifacts when it finishes (they are kept for 1 day), or tick `release` to publish them.

Manual run options:

| Input | Default | Effect |
|---|---|---|
| `upstream_ref` | empty | CachyOS-Live-ISO commit, branch or tag to build from. Empty uses `UPSTREAM_PIN`, or the default branch if that is empty too |
| `keep_installer` | on | Keep the CachyOS installers in the ISO |
| `version` | `1.0.0` | Version as `major.minor.patch`. Used in the ISO file names and the release tag (`v<version>`) |
| `release` | off | Publish the ISOs to a GitHub Release |
| `prerelease` | off | Mark the release as a pre-release |

To pin the upstream so builds are reproducible, copy the commit SHA from a good run's log into `UPSTREAM_PIN` at the top of the workflow.

To make a release from a tag, push a tag beginning with `cachyos-v`, for example `cachyos-v1.2.1`. The version comes from the tag, and tag pushes always release. ISOs of 2 GiB or more cannot be attached to a GitHub Release, so that ISO is skipped in the release (the notes say so) and stays available as an artifact only.

### Repository layout

| Path | What it is |
|---|---|
| `.github/workflows/cachyos-hyprland.yml` | The build and release workflow |
| `.github/workflows/cachyos-hyprland-boottest.yml` | The QEMU boot test |
| `hypr-helper/` | Rust crate, no GTK: wallpaper generator, workspace-button daemon, Brave wallpaper watcher, popup placement, keyboard layout name |
| `hypr-ui/` | Rust crate, GTK3: the Wi-Fi, volume, power and clipboard popups |
| `python-ui/` | The Python flavour: the original popups and helpers, the two wrapper scripts and a small shared module that reads `popups.conf` and `popup-user.css` |
| `CHANGELOG.md` | Release history. The section for a version becomes its release description |

The Python flavour only keeps up if changes are ported by hand: nothing can translate Python to Rust automatically. The workflow prints a warning when `python-ui/` was changed more recently than `hypr-ui/` or `hypr-helper/`, so a Python-only change gets noticed and ported.

### Customising

Edit the arrays and heredocs in the **Patch Package List** and **Patch airootfs** steps:

- Add or remove packages in the `ADD` and `REMOVE` arrays.
- Change the desktop look in the Waybar `config.jsonc` and `style.css` blocks.
- Change shortcuts in the Hyprland config block.
- Change corner roundness: `rounding` in the Hyprland block, and the `border-radius` values in the Waybar, wofi and mako blocks.

Changing what a popup *does* (not how it looks or where it sits) means editing the Rust crates and building a new ISO, or, in the python flavour, editing the files in `python-ui/`.

## Troubleshooting

| Problem | Try this |
|---|---|
| Screen is black after the boot messages | Wait a couple of minutes on slow media. In a VM, enable virtio-gpu with 3D |
| Stock Hyprland welcome screen, no top bar | The config was not loaded. Run `cp -rn /etc/skel/. ~/` and `rm -f ~/.config/hypr/hyprland.lua`, then log out and back in |
| System freezes or apps get killed | Out of RAM. Close Brave and other heavy apps |
| No Wi-Fi networks listed | Try the refresh button in the Wi-Fi popup or `nmcli device wifi rescan`. Some chipsets need firmware that is not in the image |
| A popup opens in the wrong place | Change `top_margin` or `right_margin` in `~/.config/hypr/popups.conf`, save, and open the popup again |
| Saved data did not come back | Boot from the same USB you saved to. Check that `save-progress` finished with `Saved ...` and `Done!`. On Ventoy sticks the `cachyos-hyprland-save` folder must still be on the stick |
| Brave wallpaper did not change | The new wallpaper is made when Brave starts, and only if Brave was not already running. Close it fully and start it with `Super + B`. Brave only shows the picture once "Use your own" is chosen for the background in Brave's new tab settings (Customize); choose it once and it stays |
| `updateos` cannot find the new version | Make sure the USB is plugged in and has either a flashed ISO or a file named `cachyos-hyprland-*.iso` within two folders of the top. Or pass the file or device directly: `updateos /path/to/file.iso` |
| Dropped to GRUB with errors | Re-download the ISO and verify the checksum. A partial download is the most common cause |

## Credits

- [CachyOS](https://cachyos.org) for the live ISO profile this builds on
- [Hyprland](https://hypr.land) and the Hyprland ecosystem
- [Waybar](https://github.com/Alexays/Waybar), [kitty](https://sw.kovidgoyal.net/kitty/), [fish](https://fishshell.com), [wofi](https://hg.sr.ht/~scoopta/wofi)
- [archiso](https://wiki.archlinux.org/title/Archiso)

## Disclaimer

This project is provided as is, with no warranty. It is a personal, unofficial build and is not endorsed by the CachyOS or Hyprland projects. You are responsible for backing up your data before installing any operating system.
