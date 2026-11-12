//! Make a new Brave wallpaper every time a new tab opens (port of brave-wall-watch.py).
//!
//! Brave opens the picture in sanitized_background_images each time it shows a new tab page.
//! This watches that folder (inotify) and, after each open, swaps the picture for a new random
//! one with brave-wall.sh, so the next new tab looks different.

use std::env;
use std::ffi::CString;
use std::fs::File;
use std::io::{ErrorKind, Read};
use std::os::fd::FromRawFd;
use std::os::raw::c_char;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;

extern "C" {
    fn inotify_init() -> i32;
    fn inotify_add_watch(fd: i32, path: *const c_char, mask: u32) -> i32;
    fn fcntl(fd: i32, cmd: i32, ...) -> i32;
}

const IN_OPEN: u32 = 0x20;
const F_SETFL: i32 = 4; // x86_64 Linux
const O_NONBLOCK: i32 = 0o4000; // x86_64 Linux

pub fn run() -> i32 {
    let home = PathBuf::from(env::var("HOME").unwrap_or_default());
    let dir = home.join(".config/BraveSoftware/Brave-Browser/Default/sanitized_background_images");
    let script = home.join(".config/hypr/brave-wall.sh");
    let cdir = match CString::new(dir.to_string_lossy().as_bytes()) {
        Ok(c) => c,
        Err(_) => return 1,
    };

    let fd = unsafe { inotify_init() };
    if fd < 0 {
        return 1;
    }
    let mut f = unsafe { File::from_raw_fd(fd) };
    let mut wd = -1;
    let mut buf = [0u8; 4096];
    loop {
        if wd < 0 {
            if dir.is_dir() {
                wd = unsafe { inotify_add_watch(fd, cdir.as_ptr(), IN_OPEN) };
            }
            if wd < 0 {
                thread::sleep(Duration::from_secs(5));
                continue;
            }
        }
        let n = match f.read(&mut buf) {
            Ok(0) => {
                thread::sleep(Duration::from_secs(1));
                continue;
            }
            Ok(n) => n,
            Err(e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(_) => return 1,
        };
        // struct inotify_event { int wd; u32 mask; u32 cookie; u32 len; char name[]; }
        let mut i = 0usize;
        let mut new_tab = false;
        while i + 16 <= n {
            let len = u32::from_ne_bytes([buf[i + 12], buf[i + 13], buf[i + 14], buf[i + 15]]) as usize;
            let end = (i + 16 + len).min(n);
            let name = buf[i + 16..end].split(|&c| c == 0).next().unwrap_or(&[]);
            i += 16 + len;
            if !name.is_empty() && !name.ends_with(b".tmp") {
                new_tab = true;
            }
        }
        if new_tab {
            thread::sleep(Duration::from_millis(1000)); // let Brave finish reading the current picture
            let _ = Command::new("sh").arg(&script).status();
            thread::sleep(Duration::from_millis(500));
            // drop the opens caused by Brave noticing the new file, so one tab = one picture
            unsafe { fcntl(fd, F_SETFL, O_NONBLOCK) };
            while let Ok(k) = f.read(&mut buf) {
                if k == 0 {
                    break;
                }
            }
            unsafe { fcntl(fd, F_SETFL, 0) };
        }
    }
}
