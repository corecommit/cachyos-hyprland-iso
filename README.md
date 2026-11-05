<div align="center">

# CachyOS Hyprland ISO

**An unofficial CachyOS live ISO that boots into a tiling Hyprland desktop instead of KDE Plasma.**

Built automatically from the upstream CachyOS-Live-ISO by a GitHub Actions workflow.

<br>

[![Build](https://img.shields.io/github/actions/workflow/status/corecommit/custom-linux-iso/cachyos-hyprland.yml?style=for-the-badge&logo=githubactions&logoColor=white&label=BUILD&labelColor=1e1e2e)](https://github.com/corecommit/custom-linux-iso/actions/workflows/cachyos-hyprland.yml)
[![Release](https://img.shields.io/github/v/release/corecommit/custom-linux-iso?style=for-the-badge&logo=github&logoColor=white&label=RELEASE&labelColor=1e1e2e&color=b4befe)](https://github.com/corecommit/custom-linux-iso/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/corecommit/custom-linux-iso/total?style=for-the-badge&logo=download&logoColor=white&label=DOWNLOADS&labelColor=1e1e2e&color=a6e3a1)](https://github.com/corecommit/custom-linux-iso/releases)
[![Stars](https://img.shields.io/github/stars/corecommit/custom-linux-iso?style=for-the-badge&logo=github&logoColor=white&label=STARS&labelColor=1e1e2e&color=f9e2af)](https://github.com/corecommit/custom-linux-iso/stargazers)

![Arch Linux](https://img.shields.io/badge/BASE-ARCH_LINUX-1793D1?style=for-the-badge&logo=archlinux&logoColor=white&labelColor=1e1e2e)
![CachyOS](https://img.shields.io/badge/DISTRO-CACHYOS-08A283?style=for-the-badge&logo=archlinux&logoColor=white&labelColor=1e1e2e)
![Hyprland](https://img.shields.io/badge/WM-HYPRLAND-58E1FF?style=for-the-badge&logo=hyprland&logoColor=white&labelColor=1e1e2e)
![Wayland](https://img.shields.io/badge/DISPLAY-WAYLAND-FFBC00?style=for-the-badge&logo=wayland&logoColor=white&labelColor=1e1e2e)

[Download](https://github.com/corecommit/custom-linux-iso/releases/latest) |
[How it works](#how-it-works) |
[Shortcuts](#keyboard-shortcuts) |
[Warnings](#warnings-read-before-booting) |
[Build your own](#build-your-own)

</div>

---

## Overview

The official CachyOS live ISO ships KDE Plasma. This project takes that ISO profile, strips out Plasma and a long list of tools a live session does not need, and replaces them with:

| Component | What it is |
|---|---|
| Compositor | Hyprland, started by greetd with autologin |
| Top bar | Waybar: clock, temperature, RAM, workspaces, window title, tray, Wi-Fi, volume, battery, power menu |
| Terminal | kitty with fish and fastfetch |
| Launcher | wofi |
| File manager | Thunar |
| Browser | Firefox |
| Notifications | mako |
| Lock screen / idle | hyprlock, hypridle |
| Kernel | linux-cachyos (the LTS kernels are removed) |
| Installer | The CachyOS installer is kept so you can install from the live session |

Nothing here is a fork of CachyOS. The workflow downloads the upstream profile on every build and patches it, so upstream updates flow in automatically.

## Warnings (read before booting)

> **This is an unofficial, community-built ISO. It is not affiliated with or supported by the CachyOS project. Do not ask the CachyOS team for help with it.**

**Memory**

- **4 GB of RAM is the practical minimum.** The live session keeps all changes (installed packages, downloads, settings) in RAM. A browser plus a few apps can fill it quickly.
- **8 GB or more is recommended**, especially if you plan to use Firefox, install packages in the live session, or run the installer while doing other things.
- If the system starts freezing or killing programs, you are almost certainly out of memory. Close Firefox first.
- Everything done in the live session is lost on reboot. Nothing is saved to the USB stick.

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

1. Download the latest ISO and its `.sha256` file from the [Releases](https://github.com/corecommit/custom-linux-iso/releases/latest) page.
2. Verify it:

   ```bash
   sha256sum -c *.iso.sha256
   ```

3. Write it to a USB stick. This erases the stick.

   ```bash
   sudo dd if=cachyos-hyprland.iso of=/dev/sdX bs=4M status=progress oflag=sync
   ```

   Replace `/dev/sdX` with your USB device (check with `lsblk`). Using the wrong device destroys data. Graphical tools such as Rufus (DD mode) or balenaEtcher also work.

4. Disable Secure Boot, boot from the stick, and wait for the desktop to appear.

### Connecting to Wi-Fi

Click the network item in the top bar, which opens `nmtui`. Or from a terminal:

```bash
nmcli device wifi connect "SSID" password "PASSWORD"
```

## Keyboard shortcuts

`Super` is the Windows key. Press `Super + F1` at any time to open the full config in a terminal.

| Keys | Action |
|---|---|
| `Super + Enter` | Open terminal (kitty) |
| `Super + Space` or `Super + D` | Open launcher (wofi) |
| `Super + Q` | Close window |
| `Super + F` | Fullscreen on or off |
| `Super + V` | Floating on or off |
| `Super + E` | File manager (Thunar) |
| `Super + B` | Firefox |
| `Super + L` | Lock screen |
| `Super + Arrow keys` | Move focus |
| `Super + Shift + Arrow keys` | Move window |
| `Super + Ctrl + Arrow keys` | Resize window (hold to repeat) |
| `Super + 1-9` | Go to workspace |
| `Super + Shift + 1-9` | Send window to workspace |
| `Super + .` and `Super + ,` | Next or previous workspace |
| `Super + Scroll` | Next or previous workspace |
| `Super + Left-drag` | Move window with the mouse |
| `Super + Right-drag` | Resize window with the mouse |
| `Super + Y` | Pin a floating window to all workspaces |
| `Print` | Select a region and copy a screenshot to the clipboard |
| `Super + Shift + R` | Reload Hyprland config |
| `Super + Shift + E` | Exit Hyprland |

Volume and brightness keys work as normal media keys.

## How it works

Everything happens in one GitHub Actions workflow: `.github/workflows/cachyos-hyprland.yml`. It runs inside an `archlinux:base-devel` container on a GitHub runner.

```
Clone upstream  ->  Patch profile  ->  Verify  ->  Build  ->  Check  ->  Publish
```

1. **Prepare.** Frees disk space on the runner, installs `archiso` and friends, sets up the CachyOS keyring and repositories, and creates a non-root `builder` user.
2. **Clone upstream.** Fetches [CachyOS/CachyOS-Live-ISO](https://github.com/CachyOS/CachyOS-Live-ISO). The commit used is printed in the log and the run summary so a good build can be pinned.
3. **Patch the profile.** Each patch is its own step and fails loudly if upstream changed and a change did nothing (a small `sedx` helper enforces this).
   - Package list: add Hyprland, Waybar, kitty and friends; remove Plasma, KDE apps, LTS kernels, guest tools, network tools and other things a live session does not need.
   - Display manager: swap Plasma's login manager for greetd, with an autologin straight into Hyprland.
   - Airootfs: write the Hyprland, Waybar, kitty and fish configs into `/etc/skel`, generate a wallpaper, and remove Plasma-specific config files.
   - Boot robustness: remove dangling links to removed services, raise timeouts for slow or emulated machines, and make sure NetworkManager is enabled.
   - Boot entries: point GRUB, systemd-boot and syslinux at `linux-cachyos`, and remove the LTS entries.
   - Session launcher: a small script copies the default config into the home folder if it is missing, so the desktop always starts with these settings.
4. **Verify.** Checks that Plasma and the LTS kernel are really gone, that required config files exist, and that no boot entry still points at a removed kernel.
5. **Build.** Runs the upstream `buildiso.sh` as the builder user and keeps a full log, which is uploaded as an artifact if the build fails.
6. **Check.** Confirms every kernel named in a boot config exists in the final ISO, and checks the ISO size against the 2 GiB GitHub release limit.
7. **Publish.** Uploads the ISO as a workflow artifact (kept 30 days) and, for tags starting with `cachyos-v`, attaches it to a GitHub Release.

### Boot test workflow

`.github/workflows/cachyos-hyprland-boottest.yml` boots an already built ISO in QEMU with UEFI firmware. It takes screenshots over QMP and checks that a desktop appears and stays up. The screenshots are uploaded as an artifact, so you can see what the machine showed without owning the hardware. It does not rebuild anything, so it finishes in minutes.

## Build your own

1. Fork this repository.
2. Open the **Actions** tab, choose **Build CachyOS Hyprland ISO**, and press **Run workflow**.
3. Download the ISO from the run's artifacts when it finishes.

Manual run options:

| Input | Default | Effect |
|---|---|---|
| `upstream_ref` | empty | CachyOS-Live-ISO commit, branch or tag to build from. Empty uses `UPSTREAM_PIN`, or the default branch if that is empty too |
| `keep_installer` | on | Keep the CachyOS installers in the ISO |
| `xz_squashfs` | off | Experimental: compress the image with xz for a smaller ISO |
| `strip_docs` | off | Experimental: skip man pages, docs and info files |

To pin the upstream so builds are reproducible, copy the commit SHA from a good run's log into `UPSTREAM_PIN` at the top of the workflow.

To make a release, push a tag beginning with `cachyos-v`, for example `cachyos-v1.0`. ISOs of 2 GiB or more cannot be attached to a GitHub Release, so the release step is skipped and the ISO stays available as an artifact only.

### Customising

Edit the arrays and heredocs in the **Patch Package List** and **Patch airootfs** steps:

- Add or remove packages in the `ADD` and `REMOVE` arrays.
- Change the desktop look in the Waybar `config.jsonc` and `style.css` blocks.
- Change shortcuts in the Hyprland config block.

## Troubleshooting

| Problem | Try this |
|---|---|
| Screen is black after the boot messages | Wait a couple of minutes on slow media. In a VM, enable virtio-gpu with 3D |
| Stock Hyprland welcome screen, no top bar | The config was not loaded. Run `cp -rn /etc/skel/. ~/` and `rm -f ~/.config/hypr/hyprland.lua`, then log out and back in |
| System freezes or apps get killed | Out of RAM. Close Firefox and other heavy apps |
| No Wi-Fi networks listed | Try `nmcli device wifi rescan`. Some chipsets need firmware that is not in the image |
| Dropped to GRUB with errors | Re-download the ISO and verify the checksum. A partial download is the most common cause |

## Credits

- [CachyOS](https://cachyos.org) for the live ISO profile this builds on
- [Hyprland](https://hypr.land) and the Hyprland ecosystem
- [Waybar](https://github.com/Alexays/Waybar), [kitty](https://sw.kovidgoyal.net/kitty/), [fish](https://fishshell.com), [wofi](https://hg.sr.ht/~scoopta/wofi)
- [archiso](https://wiki.archlinux.org/title/Archiso)

## Disclaimer

This project is provided as is, with no warranty. It is a personal, unofficial build and is not endorsed by the CachyOS or Hyprland projects. You are responsible for backing up your data before installing any operating system.
