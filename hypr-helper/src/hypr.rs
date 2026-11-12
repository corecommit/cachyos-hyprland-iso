//! Hyprland sockets, shared by ws-daemon, popup-pos and kb-name.

use std::env;
use std::fs;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, SystemTime};

pub fn runtime_dir() -> String {
    env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".into())
}

/// $XDG_RUNTIME_DIR/hypr/<instance>/<file>: our own instance first, else the newest one.
pub fn sock_path(file: &str) -> Option<PathBuf> {
    let base = PathBuf::from(runtime_dir()).join("hypr");
    if let Ok(sig) = env::var("HYPRLAND_INSTANCE_SIGNATURE") {
        if !sig.is_empty() {
            let p = base.join(sig).join(file);
            if p.exists() {
                return Some(p);
            }
        }
    }
    let mut found: Vec<(SystemTime, PathBuf)> = fs::read_dir(&base)
        .ok()?
        .filter_map(|e| {
            let p = e.ok()?.path().join(file);
            let m = fs::metadata(&p).ok()?.modified().ok()?;
            Some((m, p))
        })
        .collect();
    found.sort_by(|a, b| b.0.cmp(&a.0));
    found.into_iter().next().map(|(_, p)| p)
}

/// Same as `hyprctl -j <what>`, but over the command socket (no process to start).
pub fn hc(what: &str) -> Option<String> {
    if let Some(p) = sock_path(".socket.sock") {
        if let Ok(mut s) = UnixStream::connect(p) {
            let _ = s.set_read_timeout(Some(Duration::from_secs(3)));
            if s.write_all(format!("j/{what}").as_bytes()).is_ok() {
                let mut out = Vec::new();
                if s.read_to_end(&mut out).is_ok() && !out.is_empty() {
                    return Some(String::from_utf8_lossy(&out).into_owned());
                }
            }
        }
    }
    let o = Command::new("hyprctl").args(["-j", what]).output().ok()?;
    Some(String::from_utf8_lossy(&o.stdout).into_owned())
}
