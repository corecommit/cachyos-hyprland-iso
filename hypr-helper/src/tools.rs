//! Two tiny helpers that used inline python3 in shell scripts.

use crate::hypr::hc;
use serde_json::Value;

fn size(m: &Value) -> (f64, f64) {
    let mut w = m["width"].as_f64().unwrap_or(0.0);
    let mut h = m["height"].as_f64().unwrap_or(0.0);
    if m["transform"].as_i64().unwrap_or(0) % 2 != 0 {
        std::mem::swap(&mut w, &mut h);
    }
    let sc = m["scale"].as_f64().unwrap_or(1.0);
    (w / sc, h / sc)
}

/// popup-pos.sh WIDTH [TOP] -> prints "X Y": where a popup of WIDTH px must go so it sits
/// centred under the mouse pointer, TOP px from the top of the monitor, kept on screen.
/// Falls back to "100 TOP".
pub fn popup_pos(args: &[String]) -> i32 {
    let w: f64 = args.first().and_then(|v| v.parse().ok()).unwrap_or(300.0);
    let top: i64 = args.get(1).and_then(|v| v.parse().ok()).unwrap_or(46);
    let x = (|| -> Option<i64> {
        let c: Value = serde_json::from_str(&hc("cursorpos")?).ok()?;
        let mons: Value = serde_json::from_str(&hc("monitors")?).ok()?;
        let mons = mons.as_array()?;
        let (cx, cy) = (c["x"].as_f64()?, c["y"].as_f64()?);
        let mon = mons
            .iter()
            .find(|m| {
                let (sw, sh) = size(m);
                let mx = m["x"].as_f64().unwrap_or(0.0);
                let my = m["y"].as_f64().unwrap_or(0.0);
                mx <= cx && cx < mx + sw && my <= cy && cy < my + sh
            })
            .or_else(|| mons.first())?;
        let (sw, _) = size(mon);
        let mx = mon["x"].as_f64().unwrap_or(0.0);
        Some((cx - mx - w / 2.0).min(sw - w - 8.0).max(8.0) as i64)
    })()
    .unwrap_or(100);
    println!("{x} {top}");
    0
}

/// Name of the keyboard layout that is active on the main keyboard (for kb-switch.sh).
pub fn kb_name() -> i32 {
    let name = (|| -> Option<String> {
        let v: Value = serde_json::from_str(&hc("devices")?).ok()?;
        let kb = v["keyboards"]
            .as_array()?
            .iter()
            .find(|k| k["main"].as_bool().unwrap_or(false))?;
        Some(kb["active_keymap"].as_str().unwrap_or("").to_string())
    })()
    .unwrap_or_default();
    println!("{name}");
    0
}
