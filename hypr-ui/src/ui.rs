//! Shared pieces: the full-screen layer-shell window with the centred "card", colours, CSS,
//! running commands, and background work that reports back on the GTK thread.

use gtk::glib::Propagation;
use gtk::glib::translate::ToGlibPtr;
use gtk::prelude::*;
use gtk::{gdk, glib};
use std::env;
use std::ffi::{c_void, CString};
use std::fs;
use std::io::{Read, Write};
use std::os::raw::c_char;
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

// ---------- gtk-layer-shell ----------
// Loaded at run time (dlopen) instead of linked, so building needs no layer-shell dev files and
// no linker tricks. The launch scripts still LD_PRELOAD the library, so it is already loaded
// before wayland-client, which gtk-layer-shell requires; dlopen then simply finds it.
extern "C" {
    fn dlopen(filename: *const c_char, flag: i32) -> *mut c_void;
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    fn setsid() -> i32;
}

type WinFn = unsafe extern "C" fn(*mut gtk::ffi::GtkWindow);
type WinFnI = unsafe extern "C" fn(*mut gtk::ffi::GtkWindow, i32);
type WinFnII = unsafe extern "C" fn(*mut gtk::ffi::GtkWindow, i32, i32);

struct LayerShell {
    init: WinFn,
    set_layer: WinFnI,
    keyboard: WinFnI,
    zone: WinFnI,
    anchor: WinFnII,
}

fn layer_shell() -> Option<LayerShell> {
    unsafe {
        let mut h: *mut c_void = std::ptr::null_mut();
        for name in ["libgtk-layer-shell.so.0", "libgtk-layer-shell.so"] {
            let c = CString::new(name).ok()?;
            h = dlopen(c.as_ptr(), 2); // RTLD_NOW
            if !h.is_null() {
                break;
            }
        }
        if h.is_null() {
            return None;
        }
        let sym = |n: &str| -> *mut c_void {
            let c = CString::new(n).unwrap();
            dlsym(h, c.as_ptr())
        };
        let init = sym("gtk_layer_shell_init_for_window");
        let layer = sym("gtk_layer_shell_set_layer");
        let kb = sym("gtk_layer_shell_set_keyboard_mode");
        let zone = sym("gtk_layer_shell_set_exclusive_zone");
        let anchor = sym("gtk_layer_shell_set_anchor");
        if [init, layer, kb, zone, anchor].iter().any(|p| p.is_null()) {
            return None;
        }
        Some(LayerShell {
            init: std::mem::transmute::<*mut c_void, WinFn>(init),
            set_layer: std::mem::transmute::<*mut c_void, WinFnI>(layer),
            keyboard: std::mem::transmute::<*mut c_void, WinFnI>(kb),
            zone: std::mem::transmute::<*mut c_void, WinFnI>(zone),
            anchor: std::mem::transmute::<*mut c_void, WinFnII>(anchor),
        })
    }
}

// ---------- colours ----------
pub const PILL: &str = "#413D3A";
pub const BORDER: &str = "#514947";
pub const GOLD: &str = "#c7ab7a";
pub const TEXT: &str = "#ded8be";
pub const DIM: &str = "#ab9c88";
pub const ERR: &str = "#e07a6b";

/// Card background: surface_container_high from the waybar colours if present.
fn card_bg() -> String {
    let home = env::var("HOME").unwrap_or_default();
    if let Ok(src) = fs::read_to_string(format!("{home}/.config/waybar/colors.css")) {
        for line in src.lines() {
            let mut it = line.split_whitespace();
            if it.next() == Some("@define-color") && it.next() == Some("surface_container_high") {
                if let Some(v) = it.next() {
                    let v = v.trim_end_matches(';');
                    if v.len() >= 7
                        && v.starts_with('#')
                        && v[1..7].bytes().all(|c| c.is_ascii_hexdigit())
                    {
                        return v[..7].to_string();
                    }
                }
            }
        }
    }
    "#282828".to_string()
}

fn paint(t: &str) -> String {
    t.replace("%BG%", &card_bg())
        .replace("%PILL%", PILL)
        .replace("%BORDER%", BORDER)
        .replace("%GOLD%", GOLD)
        .replace("%TEXT%", TEXT)
        .replace("%DIM%", DIM)
        .replace("%ERR%", ERR)
}

fn load_css(css: &str, priority: u32) {
    let prov = gtk::CssProvider::new();
    if prov.load_from_data(css.as_bytes()).is_ok() {
        if let Some(scr) = gdk::Screen::default() {
            gtk::StyleContext::add_provider_for_screen(&scr, &prov, priority);
        }
    }
}

// ---------- window ----------
pub struct Shell {
    pub win: gtk::Window,
    pub card: gtk::Box,
}

/// Full-screen transparent layer-shell window; `card` sits inside it. Clicking outside the
/// card closes the popup. `css` uses %BG% %GOLD% ... placeholders.
pub fn shell(css: &str, card_w: i32, card_h: i32) -> Shell {
    let win = gtk::Window::new(gtk::WindowType::Toplevel);
    win.set_app_paintable(true);
    if let Some(vis) = gtk::prelude::WidgetExt::screen(&win).and_then(|s| s.rgba_visual()) {
        win.set_visual(Some(&vis));
    }
    let stash = ToGlibPtr::<*mut gtk::ffi::GtkWindow>::to_glib_none(&win);
    let p: *mut gtk::ffi::GtkWindow = stash.0;
    match layer_shell() {
        Some(ls) => unsafe {
            (ls.init)(p);
            (ls.set_layer)(p, 3); // OVERLAY
            (ls.keyboard)(p, 1); // EXCLUSIVE
            (ls.zone)(p, -1);
            for edge in 0..4 {
                (ls.anchor)(p, edge, 1);
            }
        },
        None => eprintln!("hypr-ui: libgtk-layer-shell not found; the popup will be a normal window"),
    }

    load_css(&paint(css), gtk::STYLE_PROVIDER_PRIORITY_USER);
    // your own tweaks: ~/.config/hypr/popup-user.css (read every time a popup opens)
    let home = env::var("HOME").unwrap_or_default();
    if let Ok(user) = fs::read_to_string(format!("{home}/.config/hypr/popup-user.css")) {
        load_css(&user, gtk::STYLE_PROVIDER_PRIORITY_USER + 1);
    }

    let root = gtk::EventBox::new();
    root.set_visible_window(false);
    root.add_events(gdk::EventMask::BUTTON_PRESS_MASK);
    win.add(&root);

    let card = gtk::Box::new(gtk::Orientation::Vertical, 0);
    card.style_context().add_class("card");
    card.set_size_request(card_w, card_h);
    root.add(&card);

    let c = card.clone();
    root.connect_button_press_event(move |_, ev| {
        let a = c.allocation();
        let (x, y) = ev.position();
        let inside = x >= a.x() as f64
            && x <= (a.x() + a.width()) as f64
            && y >= a.y() as f64
            && y <= (a.y() + a.height()) as f64;
        if !inside {
            gtk::main_quit();
        }
        Propagation::Stop
    });
    win.connect_destroy(|_| gtk::main_quit());
    Shell { win, card }
}

// ---------- small widget helpers ----------
pub fn label(text: &str) -> gtk::Label {
    let l = gtk::Label::new(Some(text));
    l.set_xalign(0.0);
    l
}

pub fn class<W: IsA<gtk::Widget>>(w: &W, name: &str) {
    w.style_context().add_class(name);
}

pub fn sep() -> gtk::Box {
    let b = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    class(&b, "sep");
    b
}

pub fn note(text: &str) -> gtk::Label {
    let l = gtk::Label::new(Some(text));
    class(&l, "dim");
    l
}

// ---------- running commands ----------
/// Run a program, collect stdout and stderr, kill it after `timeout`.
pub fn run_bytes(prog: &str, args: &[&str], timeout: Duration) -> (i32, Vec<u8>, Vec<u8>) {
    let mut child = match Command::new(prog)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => return (1, Vec::new(), e.to_string().into_bytes()),
    };
    let mut so = child.stdout.take().unwrap();
    let mut se = child.stderr.take().unwrap();
    let h1 = thread::spawn(move || {
        let mut v = Vec::new();
        let _ = so.read_to_end(&mut v);
        v
    });
    let h2 = thread::spawn(move || {
        let mut v = Vec::new();
        let _ = se.read_to_end(&mut v);
        v
    });
    let start = Instant::now();
    let code = loop {
        match child.try_wait() {
            Ok(Some(st)) => break st.code().unwrap_or(1),
            Ok(None) => {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    break 1;
                }
                thread::sleep(Duration::from_millis(15));
            }
            Err(_) => break 1,
        }
    };
    (code, h1.join().unwrap_or_default(), h2.join().unwrap_or_default())
}

/// (exit code, stdout + stderr trimmed)
pub fn run_text(prog: &str, args: &[&str], timeout: Duration) -> (i32, String) {
    let (rc, o, e) = run_bytes(prog, args, timeout);
    let mut s = String::from_utf8_lossy(&o).into_owned();
    s.push_str(&String::from_utf8_lossy(&e));
    (rc, s.trim().to_string())
}

/// Start a program in its own session and do not wait for it.
pub fn spawn_detached(prog: &str, args: &[&str]) {
    let mut c = Command::new(prog);
    c.args(args).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    unsafe {
        c.pre_exec(|| {
            setsid();
            Ok(())
        });
    }
    let _ = c.spawn();
}

/// Same, but feed `data` to its stdin first (wl-copy).
pub fn spawn_detached_stdin(prog: &str, args: &[&str], data: &[u8]) {
    let mut c = Command::new(prog);
    c.args(args).stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::null());
    unsafe {
        c.pre_exec(|| {
            setsid();
            Ok(())
        });
    }
    if let Ok(mut child) = c.spawn() {
        if let Some(mut si) = child.stdin.take() {
            let _ = si.write_all(data);
        }
    }
}

/// Run a program with `input` on stdin and wait for it. True if it exited 0.
pub fn run_input(prog: &str, args: &[&str], input: &[u8]) -> bool {
    let mut child = match Command::new(prog)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => return false,
    };
    if let Some(mut si) = child.stdin.take() {
        let _ = si.write_all(input);
    }
    child.wait().map(|s| s.success()).unwrap_or(false)
}

// ---------- background work ----------
fn wait_for<T: 'static, D: FnOnce(T) + 'static>(rx: mpsc::Receiver<T>, done: D) {
    let mut done = Some(done);
    glib::timeout_add_local(Duration::from_millis(20), move || match rx.try_recv() {
        Ok(v) => {
            if let Some(d) = done.take() {
                d(v);
            }
            glib::ControlFlow::Break
        }
        Err(mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
        Err(_) => glib::ControlFlow::Break,
    });
}

/// Run `work` on its own thread; `done` runs on the GTK thread with the result.
pub fn spawn_bg<T, W, D>(work: W, done: D)
where
    T: Send + 'static,
    W: FnOnce() -> T + Send + 'static,
    D: FnOnce(T) + 'static,
{
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let _ = tx.send(work());
    });
    wait_for(rx, done);
}

type Job = Box<dyn FnOnce() + Send>;

/// A few worker threads (clipboard thumbnails: never decode 60 pictures at once).
pub struct Pool {
    tx: mpsc::Sender<Job>,
}

impl Pool {
    pub fn new(n: usize) -> Pool {
        let (tx, rx) = mpsc::channel::<Job>();
        let rx = Arc::new(Mutex::new(rx));
        for _ in 0..n {
            let rx = rx.clone();
            thread::spawn(move || loop {
                let job = {
                    let g = rx.lock().unwrap();
                    g.recv()
                };
                match job {
                    Ok(j) => j(),
                    Err(_) => break,
                }
            });
        }
        Pool { tx }
    }

    pub fn submit<T, W, D>(&self, work: W, done: D)
    where
        T: Send + 'static,
        W: FnOnce() -> T + Send + 'static,
        D: FnOnce(T) + 'static,
    {
        let (rtx, rrx) = mpsc::channel();
        let _ = self.tx.send(Box::new(move || {
            let _ = rtx.send(work());
        }));
        wait_for(rrx, done);
    }
}
