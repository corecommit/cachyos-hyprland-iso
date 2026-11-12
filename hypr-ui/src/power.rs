//! Power menu (Super+X): lock, log out, suspend, reboot, shut down.

use crate::{conf::Conf, ui};
use gtk::gdk;
use gtk::glib::Propagation;
use gtk::prelude::*;

const CSS: &str = r#"
window { background: transparent; }
* { font-family: monospace, "Font Awesome 6 Free", sans-serif; font-size: 14px; color: %TEXT%; }
.card { background: %BG%; border: 5px solid %BG%; border-radius: 6px; padding: 8px; }
.title { color: %GOLD%; font-weight: bold; }
.sep { background: %BORDER%; min-height: 1px; margin: 6px 0; }
button { background: transparent; border: none; box-shadow: none; text-shadow: none; padding: 8px 10px; border-radius: 4px; }
button:hover, button:focus { background: alpha(%GOLD%, 0.18); }
button:focus { background: %PILL%; }
button.danger:hover label, button.danger:focus label { color: %ERR%; }
.icon { color: %GOLD%; }
.dim { color: %DIM%; font-size: 12px; }
"#;

struct Action {
    icon: &'static str,
    label: &'static str,
    key: char,
    cmd: &'static [&'static str],
    danger: bool,
}

static ACTIONS: [Action; 5] = [
    Action { icon: "\u{f023}", label: "Lock", key: 'L', cmd: &["hyprlock"], danger: false },
    Action {
        icon: "\u{f2f5}",
        label: "Log out",
        key: 'O',
        cmd: &["sh", "-c", "hyprctl dispatch 'hl.dsp.exit()'; hyprctl dispatch exit"],
        danger: false,
    },
    Action { icon: "\u{f186}", label: "Suspend", key: 'S', cmd: &["systemctl", "suspend"], danger: false },
    Action { icon: "\u{f021}", label: "Reboot", key: 'R', cmd: &["systemctl", "reboot"], danger: true },
    Action { icon: "\u{f011}", label: "Shut down", key: 'P', cmd: &["systemctl", "poweroff"], danger: true },
];

fn run_action(a: &Action) {
    ui::spawn_detached(a.cmd[0], &a.cmd[1..]);
    gtk::main_quit();
}

pub fn run(conf: &Conf) {
    let sh = ui::shell(CSS, conf.int("power_width", 260), -1);
    sh.card.set_halign(gtk::Align::Center);
    sh.card.set_valign(gtk::Align::Center);

    let title = ui::label("\u{f011}  Power");
    ui::class(&title, "title");
    title.set_margin_start(4);
    title.set_margin_top(4);
    title.set_margin_bottom(2);
    sh.card.pack_start(&title, false, false, 0);
    sh.card.pack_start(&ui::sep(), false, false, 0);

    let mut buttons = Vec::new();
    for (i, a) in ACTIONS.iter().enumerate() {
        let btn = gtk::Button::new();
        if a.danger {
            ui::class(&btn, "danger");
        }
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        let ic = gtk::Label::new(Some(a.icon));
        ui::class(&ic, "icon");
        ic.set_size_request(20, -1);
        row.pack_start(&ic, false, false, 0);
        row.pack_start(&ui::label(a.label), true, true, 0);
        let hk = gtk::Label::new(Some(&a.key.to_string()));
        ui::class(&hk, "dim");
        row.pack_end(&hk, false, false, 0);
        btn.add(&row);
        btn.connect_clicked(move |_| run_action(&ACTIONS[i]));
        sh.card.pack_start(&btn, false, false, 0);
        buttons.push(btn);
    }

    let hint = ui::label("arrows + Enter, or press the letter  \u{b7}  Esc closes");
    ui::class(&hint, "dim");
    hint.set_margin_start(4);
    hint.set_margin_top(6);
    sh.card.pack_start(&hint, false, false, 0);

    sh.win.connect_key_press_event(move |_, ev| {
        let k = ev.keyval();
        if k == gdk::keys::constants::Escape {
            gtk::main_quit();
            return Propagation::Stop;
        }
        if let Some(c) = k.to_unicode() {
            let c = c.to_ascii_lowercase();
            if let Some(a) = ACTIONS.iter().find(|a| a.key.to_ascii_lowercase() == c) {
                run_action(a);
                return Propagation::Stop;
            }
        }
        Propagation::Proceed
    });

    sh.win.show_all();
    buttons[0].grab_focus();
}
