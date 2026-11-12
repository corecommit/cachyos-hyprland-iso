//! Keeps the bar's 1-2-3-4 workspace buttons in sync (port of ws-daemon.py).
//!
//! Listens to Hyprland's event socket, writes the state to $XDG_RUNTIME_DIR/ws-state and
//! pokes waybar (signal 8) only when something changed. ws-state.sh reads that file.

use crate::hypr::{hc, runtime_dir, sock_path};
use std::fs;
use std::io::{ErrorKind, Read};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;

const EVENTS: [&str; 14] = [
    "workspace",
    "workspacev2",
    "createworkspace",
    "createworkspacev2",
    "destroyworkspace",
    "destroyworkspacev2",
    "openwindow",
    "closewindow",
    "movewindow",
    "movewindowv2",
    "moveworkspace",
    "moveworkspacev2",
    "focusedmon",
    "focusedmonv2",
];

type Obj = Vec<(String, i64)>;

/// Numeric fields of every top-level `{...}` object in a hyprctl JSON reply.
/// Tolerant on purpose: it never fails on odd strings the way a strict parser can.
fn objects(json: &str) -> Vec<Obj> {
    let b = json.as_bytes();
    let mut out: Vec<Obj> = Vec::new();
    let mut cur: Option<Obj> = None;
    let mut depth = 0usize;
    let mut i = 0usize;
    while i < b.len() {
        match b[i] {
            b'"' => {
                let start = i + 1;
                i += 1;
                while i < b.len() && b[i] != b'"' {
                    if b[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
                let end = i.min(b.len());
                let s = json.get(start..end).unwrap_or("");
                i += 1; // past the closing quote
                let mut j = i;
                while j < b.len() && b[j].is_ascii_whitespace() {
                    j += 1;
                }
                if depth == 1 && j < b.len() && b[j] == b':' {
                    j += 1;
                    while j < b.len() && b[j].is_ascii_whitespace() {
                        j += 1;
                    }
                    let ns = j;
                    while j < b.len() && (b[j] == b'-' || b[j].is_ascii_digit()) {
                        j += 1;
                    }
                    if j > ns {
                        if let (Some(o), Ok(n)) = (cur.as_mut(), json[ns..j].parse::<i64>()) {
                            o.push((s.to_string(), n));
                        }
                    }
                }
                continue;
            }
            b'{' => {
                depth += 1;
                if depth == 1 {
                    cur = Some(Vec::new());
                }
            }
            b'}' => {
                if depth == 1 {
                    if let Some(o) = cur.take() {
                        out.push(o);
                    }
                }
                depth = depth.saturating_sub(1);
            }
            _ => {}
        }
        i += 1;
    }
    out
}

fn field(o: &Obj, key: &str) -> Option<i64> {
    o.iter().find(|(k, _)| k == key).map(|(_, v)| *v)
}

/// An RT signal sent to a waybar that is still starting up kills it, so only
/// signal one that has been running for a few seconds.
fn waybar_ready() -> bool {
    for _ in 0..8 {
        let out = match Command::new("ps").args(["-o", "etimes=", "-C", "waybar"]).output() {
            Ok(o) => o,
            Err(_) => return false,
        };
        let secs: Vec<u64> = String::from_utf8_lossy(&out.stdout)
            .split_whitespace()
            .filter_map(|x| x.parse().ok())
            .collect();
        match secs.iter().min() {
            None => return false,
            Some(&m) if m >= 3 => return true,
            _ => {}
        }
        thread::sleep(Duration::from_millis(500));
    }
    false
}

fn update(last: &str) -> String {
    let active = match hc("activeworkspace")
        .and_then(|j| objects(&j).into_iter().next())
        .and_then(|o| field(&o, "id"))
    {
        Some(v) => v,
        None => return last.to_string(),
    };
    let ws_json = match hc("workspaces") {
        Some(j) => j,
        None => return last.to_string(),
    };
    let mut occ: Vec<i64> = objects(&ws_json)
        .iter()
        .filter(|o| field(o, "windows").unwrap_or(0) > 0)
        .filter_map(|o| field(o, "id"))
        .collect();
    occ.sort_unstable();
    occ.dedup();
    let list: Vec<String> = occ.iter().map(|n| n.to_string()).collect();
    let text = format!("active={}\nocc=\" {} \"\n", active, list.join(" "));
    if text == last {
        return last.to_string();
    }
    let state = PathBuf::from(runtime_dir()).join("ws-state");
    let tmp = PathBuf::from(runtime_dir()).join("ws-state.tmp");
    if fs::write(&tmp, &text).is_ok() {
        let _ = fs::rename(&tmp, &state);
    }
    if waybar_ready() {
        let _ = Command::new("pkill").args(["-RTMIN+8", "waybar"]).status();
    }
    text
}

pub fn run() -> i32 {
    let mut path = None;
    for _ in 0..20 {
        path = sock_path(".socket2.sock");
        if path.is_some() {
            break;
        }
        thread::sleep(Duration::from_millis(500));
    }
    let path = match path {
        Some(p) => p,
        None => return 0,
    };
    let mut s = match UnixStream::connect(&path) {
        Ok(s) => s,
        Err(_) => return 1,
    };
    let mut last = String::new();
    let mut buf: Vec<u8> = Vec::new();
    let mut dirty = true;
    let mut chunk = [0u8; 4096];
    loop {
        // wait for a burst of events to settle
        let _ = s.set_read_timeout(if dirty { Some(Duration::from_millis(80)) } else { None });
        match s.read(&mut chunk) {
            Ok(0) => return 0,
            Ok(n) => {
                buf.extend_from_slice(&chunk[..n]);
                while let Some(pos) = buf.iter().position(|&c| c == b'\n') {
                    let line: Vec<u8> = buf.drain(..=pos).collect();
                    let name = line.split(|&c| c == b'>').next().unwrap_or(&[]);
                    if let Ok(nm) = std::str::from_utf8(name) {
                        if EVENTS.contains(&nm) {
                            dirty = true;
                        }
                    }
                }
            }
            Err(e) if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                last = update(&last);
                dirty = false;
            }
            Err(e) if e.kind() == ErrorKind::Interrupted => {}
            Err(_) => return 1,
        }
    }
}
