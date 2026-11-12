//! Random dark wallpaper (port of genwall.py: same random numbers, so same pictures).
//!
//! usage: hypr-helper genwall png OUT.png [--style glow|aurora|mesh] [--seed N] [--size WxH]
//!
//! Three looks (picked at random unless --style is given):
//!   glow    soft coloured light blobs on a dark gradient
//!   aurora  flowing ribbons of light
//!   mesh    smooth multi-colour gradient, gently warped
//! The file is written next to its destination as OUT.tmp and moved over the old one.

use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const USAGE: &str =
    "usage: hypr-helper genwall png OUT.png [--style glow|aurora|mesh] [--seed N] [--size WxH]";

type Rgb = [f64; 3];

/// Python's random.Random (MT19937). genwall.py used it, so with the same seed this makes
/// the same picture the Python version made.
struct Rng {
    mt: [u32; 624],
    idx: usize,
}

impl Rng {
    fn init_genrand(&mut self, s: u32) {
        self.mt[0] = s;
        for i in 1..624 {
            let p = self.mt[i - 1];
            self.mt[i] = 1_812_433_253u32.wrapping_mul(p ^ (p >> 30)).wrapping_add(i as u32);
        }
        self.idx = 624;
    }

    /// Same seeding as Python for an int seed: init_by_array with the seed as 32-bit words.
    fn new(seed: u64) -> Rng {
        let mut key: Vec<u32> = vec![seed as u32];
        if (seed >> 32) != 0 {
            key.push((seed >> 32) as u32);
        }
        let mut r = Rng { mt: [0; 624], idx: 624 };
        r.init_genrand(19_650_218);
        let n = 624usize;
        let klen = key.len();
        let (mut i, mut j) = (1usize, 0usize);
        for _ in 0..n.max(klen) {
            let prev = r.mt[i - 1];
            r.mt[i] = (r.mt[i] ^ (prev ^ (prev >> 30)).wrapping_mul(1_664_525))
                .wrapping_add(key[j])
                .wrapping_add(j as u32);
            i += 1;
            j += 1;
            if i >= n {
                r.mt[0] = r.mt[n - 1];
                i = 1;
            }
            if j >= klen {
                j = 0;
            }
        }
        for _ in 0..n - 1 {
            let prev = r.mt[i - 1];
            r.mt[i] = (r.mt[i] ^ (prev ^ (prev >> 30)).wrapping_mul(1_566_083_941))
                .wrapping_sub(i as u32);
            i += 1;
            if i >= n {
                r.mt[0] = r.mt[n - 1];
                i = 1;
            }
        }
        r.mt[0] = 0x8000_0000;
        r
    }

    fn next_u32(&mut self) -> u32 {
        if self.idx >= 624 {
            for k in 0..624 {
                let y = (self.mt[k] & 0x8000_0000) | (self.mt[(k + 1) % 624] & 0x7fff_ffff);
                let mut v = self.mt[(k + 397) % 624] ^ (y >> 1);
                if y & 1 != 0 {
                    v ^= 0x9908_b0df;
                }
                self.mt[k] = v;
            }
            self.idx = 0;
        }
        let mut y = self.mt[self.idx];
        self.idx += 1;
        y ^= y >> 11;
        y ^= (y << 7) & 0x9d2c_5680;
        y ^= (y << 15) & 0xefc6_0000;
        y ^= y >> 18;
        y
    }

    /// random.random(): 0.0 <= x < 1.0
    fn f(&mut self) -> f64 {
        let a = (self.next_u32() >> 5) as f64;
        let b = (self.next_u32() >> 6) as f64;
        (a * 67_108_864.0 + b) * (1.0 / 9_007_199_254_740_992.0)
    }

    /// random.uniform(a, b)
    fn uniform(&mut self, a: f64, b: f64) -> f64 {
        a + (b - a) * self.f()
    }

    /// random._randbelow(n): 0 <= x < n
    fn below(&mut self, n: usize) -> usize {
        let k = usize::BITS - n.leading_zeros();
        loop {
            let r = (self.next_u32() >> (32 - k)) as usize;
            if r < n {
                return r;
            }
        }
    }

    /// random.randint(a, b): a..=b
    fn int(&mut self, a: usize, b: usize) -> usize {
        a + self.below(b - a + 1)
    }
}

fn hsv(h: f64, s: f64, v: f64) -> Rgb {
    let h = h.rem_euclid(1.0);
    let s = s.clamp(0.0, 1.0);
    let v = v.clamp(0.0, 1.0);
    let i = (h * 6.0).floor();
    let f = h * 6.0 - i;
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));
    let (r, g, b) = match (i as i64).rem_euclid(6) {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };
    [255.0 * r, 255.0 * g, 255.0 * b]
}

fn mix(a: Rgb, b: Rgb, t: f64) -> Rgb {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

/// (background colour, accent colours) from a random colour harmony.
fn palette(rnd: &mut Rng) -> (Rgb, Vec<Rgb>) {
    const OFFSETS: [[f64; 4]; 4] = [
        [0.0, 0.06, -0.06, 0.12], // analogous: neighbours
        [0.0, 0.5, 0.04, 0.46],   // complementary
        [0.0, 0.333, 0.667, 0.05], // triadic
        [0.0, 0.42, 0.58, 0.08],  // split complementary
    ];
    let hue = rnd.f();
    let offs = OFFSETS[rnd.int(0, 3)];
    let bg_s = rnd.uniform(0.45, 0.75);
    let bg_v = rnd.uniform(0.045, 0.09);
    let bg = hsv(hue, bg_s, bg_v);
    let mut accents: Vec<Rgb> = Vec::with_capacity(4);
    for o in offs {
        let s = rnd.uniform(0.40, 0.80);
        let v = rnd.uniform(0.60, 0.98);
        accents.push(hsv(hue + o, s, v));
    }
    for i in (1..accents.len()).rev() {
        let j = rnd.int(0, i);
        accents.swap(i, j);
    }
    (bg, accents)
}

fn style_glow(rnd: &mut Rng, w: usize, h: usize) -> Vec<Rgb> {
    let (bg, accents) = palette(rnd);
    let top = mix(bg, accents[0], 0.07);
    let (wf, hf) = (w as f64, h as f64);
    let n = rnd.int(4, 6);
    let mut blobs: Vec<(Rgb, Vec<f64>, Vec<f64>)> = Vec::new();
    for i in 0..n {
        let col = accents[i % accents.len()];
        let cx = rnd.uniform(0.05, 0.95);
        let cy = rnd.uniform(0.05, 0.95);
        let rx = rnd.uniform(0.10, 0.30);
        let ry = rnd.uniform(0.10, 0.34);
        let s = rnd.uniform(0.40, 0.85);
        let ex: Vec<f64> = (0..w)
            .map(|x| {
                let d = (x as f64 / wf - cx) * wf / hf;
                s * (-(d * d) / (2.0 * rx * rx)).exp()
            })
            .collect();
        let ey: Vec<f64> = (0..h)
            .map(|y| {
                let d = y as f64 / hf - cy;
                (-(d * d) / (2.0 * ry * ry)).exp()
            })
            .collect();
        blobs.push((col, ex, ey));
    }
    let mut img = Vec::with_capacity(w * h);
    for y in 0..h {
        let base = mix(top, bg, y as f64 / (hf - 1.0));
        for x in 0..w {
            let [mut r, mut g, mut b] = base;
            for (col, ex, ey) in &blobs {
                let k = ex[x] * ey[y];
                r += (col[0] - r) * k;
                g += (col[1] - g) * k;
                b += (col[2] - b) * k;
            }
            img.push([r, g, b]);
        }
    }
    img
}

struct Ribbon {
    col: Rgb,
    centre: Vec<f64>,
    bright: Vec<f64>,
    th: f64,
}

fn style_aurora(rnd: &mut Rng, w: usize, h: usize) -> Vec<Rgb> {
    let (bg, accents) = palette(rnd);
    let (wf, hf) = (w as f64, h as f64);
    let n = rnd.int(3, 4);
    let mut ribbons: Vec<Ribbon> = Vec::new();
    for i in 0..n {
        let col = accents[i % accents.len()];
        let cy = rnd.uniform(0.25, 0.75); // centre line height
        let amp = rnd.uniform(0.05, 0.14); // wave size
        let f1 = rnd.uniform(1.0, 2.6);
        let p1 = rnd.uniform(0.0, 6.28);
        let f2 = rnd.uniform(2.5, 5.0);
        let p2 = rnd.uniform(0.0, 6.28);
        let th = rnd.uniform(0.045, 0.10); // thickness
        let br = rnd.uniform(0.5, 1.0); // brightness
        let fb = rnd.uniform(0.8, 2.0); // slow brightness change along x
        let pb = rnd.uniform(0.0, 6.28);
        let mut centre = Vec::with_capacity(w);
        let mut bright = Vec::with_capacity(w);
        for x in 0..w {
            let t = x as f64 / wf;
            centre.push(
                cy + amp * (f1 * 6.283 * t + p1).sin() + amp * 0.4 * (f2 * 6.283 * t + p2).sin(),
            );
            bright.push(br * (0.55 + 0.45 * (fb * 6.283 * t + pb).sin()));
        }
        ribbons.push(Ribbon { col, centre, bright, th });
    }
    let mut img = Vec::with_capacity(w * h);
    for y in 0..h {
        let v = y as f64 / hf;
        let base = mix(mix(bg, accents[0], 0.05), bg, v);
        for x in 0..w {
            let [mut r, mut g, mut b] = base;
            for rb in &ribbons {
                let d = (v - rb.centre[x]) / rb.th;
                if d > 3.2 || d < -3.2 {
                    continue;
                }
                // sharper on the lower edge, soft glow upwards: like a curtain of light
                let sharp = if d > 0.0 { 0.9 } else { 0.35 };
                let k = (-d * d * sharp).exp() * rb.bright[x] * 0.8;
                r += (rb.col[0] - r) * k;
                g += (rb.col[1] - g) * k;
                b += (rb.col[2] - b) * k;
            }
            img.push([r, g, b]);
        }
    }
    img
}

fn style_mesh(rnd: &mut Rng, w: usize, h: usize) -> Vec<Rgb> {
    let (bg, accents) = palette(rnd);
    let mut corners: Vec<Rgb> = Vec::with_capacity(4);
    for i in 0..4 {
        let t = rnd.uniform(0.35, 0.75);
        corners.push(mix(bg, accents[i % accents.len()], t));
    }
    let wa = rnd.uniform(1.0, 2.2);
    let wb = rnd.uniform(1.0, 2.2);
    let pa = rnd.uniform(0.0, 6.28);
    let pb = rnd.uniform(0.0, 6.28);
    let amp = rnd.uniform(0.10, 0.22);
    let mut img = Vec::with_capacity(w * h);
    for y in 0..h {
        let v = y as f64 / (h as f64 - 1.0);
        for x in 0..w {
            let u = x as f64 / (w as f64 - 1.0);
            let uu = (u + amp * (wa * 6.283 * v + pa).sin()).clamp(0.0, 1.0);
            let vv = (v + amp * (wb * 6.283 * u + pb).sin()).clamp(0.0, 1.0);
            // smoothstep: no visible grid lines
            let uu = uu * uu * (3.0 - 2.0 * uu);
            let vv = vv * vv * (3.0 - 2.0 * vv);
            let top = mix(corners[0], corners[1], uu);
            let bot = mix(corners[2], corners[3], uu);
            let c = mix(top, bot, vv);
            // keep it dark: it is a desktop
            img.push([c[0] * 0.55, c[1] * 0.55, c[2] * 0.55]);
        }
    }
    img
}

fn crc32(parts: &[&[u8]]) -> u32 {
    let mut table = [0u32; 256];
    for i in 0..256u32 {
        let mut c = i;
        for _ in 0..8 {
            c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
        }
        table[i as usize] = c;
    }
    let mut crc = 0xFFFF_FFFFu32;
    for part in parts {
        for &b in part.iter() {
            crc = table[((crc ^ b as u32) & 0xFF) as usize] ^ (crc >> 8);
        }
    }
    !crc
}

fn chunk(out: &mut Vec<u8>, ty: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.extend_from_slice(ty);
    out.extend_from_slice(data);
    out.extend_from_slice(&crc32(&[&ty[..], data]).to_be_bytes());
}

fn render(seed: u64, style: Option<&str>, w: usize, h: usize) -> Vec<u8> {
    let mut rnd = Rng::new(seed);
    let name: &str = match style {
        Some(s) if matches!(s, "glow" | "aurora" | "mesh") => s,
        _ => ["aurora", "glow", "mesh"][rnd.int(0, 2)],
    };
    let grain: Vec<f64> = (0..4096).map(|_| rnd.uniform(-2.6, 2.6)).collect();
    let img = match name {
        "glow" => style_glow(&mut rnd, w, h),
        "aurora" => style_aurora(&mut rnd, w, h),
        _ => style_mesh(&mut rnd, w, h),
    };

    // Vignette (darker towards the corners) + fine grain so scaling up shows no colour banding.
    let cx = (w as f64 - 1.0) / 2.0;
    let cy = (h as f64 - 1.0) / 2.0;
    let mut raw: Vec<u8> = Vec::with_capacity(h * (w * 3 + 1));
    for y in 0..h {
        raw.push(0); // PNG filter type: none
        let dy = (y as f64 - cy) / cy;
        for x in 0..w {
            let px = img[y * w + x];
            let dx = (x as f64 - cx) / cx;
            let vig = 1.0 - 0.30 * ((dx * dx + dy * dy) / 2.0).max(0.0);
            let n = grain[(x * 31 + y * 17) & 4095];
            for c in px {
                raw.push((c * vig + n).clamp(0.0, 255.0) as u8);
            }
        }
    }

    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&(w as u32).to_be_bytes());
    ihdr.extend_from_slice(&(h as u32).to_be_bytes());
    ihdr.extend_from_slice(&[8, 2, 0, 0, 0]); // 8 bit, RGB

    let mut png: Vec<u8> = b"\x89PNG\r\n\x1a\n".to_vec();
    chunk(&mut png, b"IHDR", &ihdr);
    chunk(&mut png, b"IDAT", &miniz_oxide::deflate::compress_to_vec_zlib(&raw, 6));
    chunk(&mut png, b"IEND", &[]);
    png
}

fn write_atomic(path: &str, data: &[u8]) -> std::io::Result<()> {
    if let Some(dir) = Path::new(path).parent() {
        fs::create_dir_all(dir)?;
    }
    let tmp = format!("{path}.tmp");
    fs::write(&tmp, data)?;
    fs::rename(&tmp, path)
}

struct Opts {
    out: String,
    seed: u64,
    style: Option<String>,
    w: usize,
    h: usize,
}

fn parse(args: &[String]) -> Option<Opts> {
    if args.len() < 2 || args[0] != "png" {
        return None;
    }
    let mut o = Opts {
        out: args[1].clone(),
        seed: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(1),
        style: None,
        w: 512, // small on purpose: fast to make; smooth pictures scale up cleanly
        h: 288,
    };
    let mut i = 2;
    while i < args.len() {
        let val = args.get(i + 1)?;
        match args[i].as_str() {
            "--seed" => o.seed = val.parse().ok()?,
            "--style" => {
                if !matches!(val.as_str(), "glow" | "aurora" | "mesh") {
                    return None;
                }
                o.style = Some(val.clone());
            }
            "--size" => {
                let (a, b) = val.to_lowercase().split_once('x').map(|(a, b)| (a.to_string(), b.to_string()))?;
                o.w = a.parse().ok()?;
                o.h = b.parse().ok()?;
                if o.w < 2 || o.h < 2 || o.w > 16384 || o.h > 16384 {
                    return None;
                }
            }
            _ => return None,
        }
        i += 2;
    }
    Some(o)
}

pub fn run(args: &[String]) -> i32 {
    let o = match parse(args) {
        Some(o) => o,
        None => {
            eprintln!("{USAGE}");
            return 2;
        }
    };
    let data = render(o.seed, o.style.as_deref(), o.w, o.h);
    match write_atomic(&o.out, &data) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("genwall: {}: {e}", o.out);
            1
        }
    }
}
