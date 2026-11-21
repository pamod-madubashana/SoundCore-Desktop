//! Baked window content: opaque diagonal gradient + two radial glows and
//! vertical white sheen, masked to a rounded rect (transparent outside).
//! The OS HWND stays a full transparent host; the GPU paint is the custom
//! rounded window (matches *_window.svg without a drawn edge outline).

use std::sync::OnceLock;

use crate::platform::shell::CORNER_RADIUS;

/// Logical window size (matches shell::MAIN_W/H).
const W: usize = 400;
const H: usize = 560;
/// Render scale for crispness on HiDPI.
const SS: usize = 2;

/// Generate RGBA background at 2x, return PNG bytes. Cached.
pub fn png_bytes() -> &'static [u8] {
    static CACHE: OnceLock<Vec<u8>> = OnceLock::new();
    CACHE.get_or_init(render)
}

/// Cached GPUI image for the root window paint (gradient + glows).
pub fn image() -> &'static std::sync::Arc<gpui::Image> {
    static CACHE: OnceLock<std::sync::Arc<gpui::Image>> = OnceLock::new();
    CACHE.get_or_init(|| {
        std::sync::Arc::new(gpui::Image::from_bytes(
            gpui::ImageFormat::Png,
            png_bytes().to_vec(),
        ))
    })
}

fn render() -> Vec<u8> {
    let w = W * SS;
    let h = H * SS;
    let mut buf = vec![0u8; w * h * 4];

    // Base diagonal gradient: #0b0f1a (0%) → #141a2e (55%) → #1b1033 (100%).
    let c0 = (0x0bu8, 0x0fu8, 0x1au8);
    let c1 = (0x14u8, 0x1au8, 0x2eu8);
    let c2 = (0x1bu8, 0x10u8, 0x33u8);
    // Fully opaque — no acrylic/DWM backdrop behind the window.
    let base_a = 255u8;

    for y in 0..h {
        for x in 0..w {
            // 135° diagonal: t along top-left → bottom-right.
            let t = (x as f32 / w as f32 + y as f32 / h as f32) * 0.5;
            let (r, g, b) = if t < 0.55 {
                let u = t / 0.55;
                lerp3(c0, c1, u)
            } else {
                let u = (t - 0.55) / 0.45;
                lerp3(c1, c2, u)
            };
            let i = (y * w + x) * 4;
            buf[i] = r;
            buf[i + 1] = g;
            buf[i + 2] = b;
            buf[i + 3] = base_a;
        }
    }

    // Glow A: warm orange #ff7a45, bottom-left.
    // Glow B: cool blue #5b8cff, top-right.
    add_glow(&mut buf, w, h, 0.175, 0.875, 0.60, (0xff, 0x7a, 0x45), 0.33);
    add_glow(&mut buf, w, h, 0.875, 0.075, 0.60, (0x5b, 0x8c, 0xff), 0.35);

    // Panel sheen: white 10% → 4% top→bottom (matches panel fill).
    add_sheen(&mut buf, w, h);
    // Transparent host: zero alpha outside the rounded rect (GPU custom shape).
    mask_rounded_rect(&mut buf, w, h);

    encode_png(&buf, w, h)
}

fn lerp3(a: (u8, u8, u8), b: (u8, u8, u8), t: f32) -> (u8, u8, u8) {
    let t = t.clamp(0.0, 1.0);
    (
        (a.0 as f32 + (b.0 as f32 - a.0 as f32) * t) as u8,
        (a.1 as f32 + (b.1 as f32 - a.1 as f32) * t) as u8,
        (a.2 as f32 + (b.2 as f32 - a.2 as f32) * t) as u8,
    )
}

/// Soft radial glow composited over the buffer (screen-ish blend on RGB, add on A).
fn add_glow(
    buf: &mut [u8],
    w: usize,
    h: usize,
    cx: f32,
    cy: f32,
    radius: f32,
    color: (u8, u8, u8),
    peak: f32,
) {
    let px_c = cx * w as f32;
    let py_c = cy * h as f32;
    let pr = radius * (w as f32 + h as f32) * 0.5;

    // Bounding box for efficiency.
    let x0 = ((px_c - pr).max(0.0)) as usize;
    let x1 = ((px_c + pr).min(w as f32 - 1.0)) as usize;
    let y0 = ((py_c - pr).max(0.0)) as usize;
    let y1 = ((py_c + pr).min(h as f32 - 1.0)) as usize;

    for y in y0..=y1 {
        for x in x0..=x1 {
            let dx = x as f32 - px_c;
            let dy = y as f32 - py_c;
            let d = (dx * dx + dy * dy).sqrt() / pr;
            if d >= 1.0 {
                continue;
            }
            // Smooth falloff: (1-d)^2, softened near center.
            let fall = (1.0 - d) * (1.0 - d);
            let a = fall * peak;
            let i = (y * w + x) * 4;
            // Screen blend toward the glow color.
            let (cr, cg, cb) = color;
            let comps = [cr, cg, cb];
            for c in 0..3 {
                let base = buf[i + c] as f32 / 255.0;
                let gl = comps[c] as f32 / 255.0;
                let out = 1.0 - (1.0 - base) * (1.0 - gl * a);
                buf[i + c] = (out * 255.0) as u8;
            }
        }
    }
}

/// Vertical white sheen over the whole buffer: alpha 0.10 (top) → 0.04 (bottom).
fn add_sheen(buf: &mut [u8], w: usize, h: usize) {
    for y in 0..h {
        let t = y as f32 / h as f32;
        let a = 0.10 + (0.04 - 0.10) * t;
        for x in 0..w {
            let i = (y * w + x) * 4;
            for c in 0..3 {
                let base = buf[i + c] as f32;
                buf[i + c] = (base + (255.0 - base) * a) as u8;
            }
        }
    }
}

fn round_rect_sdf(x: usize, y: usize, r: f32, hw: f32, hh: f32) -> f32 {
    let dx = x as f32 + 0.5 - hw;
    let dy = y as f32 + 0.5 - hh;
    let qx = dx.abs() - hw + r;
    let qy = dy.abs() - hh + r;
    let ax = qx.max(0.0);
    let ay = qy.max(0.0);
    (ax * ax + ay * ay).sqrt() + qx.max(qy).min(0.0) - r
}

/// Zero alpha outside the rx=28 rounded rect; ~1 logical px AA on the edge.
/// HWND remains rectangular/transparent — only this paint defines the shape.
fn mask_rounded_rect(buf: &mut [u8], w: usize, h: usize) {
    let r = CORNER_RADIUS * SS as f32;
    let hw = w as f32 * 0.5;
    let hh = h as f32 * 0.5;
    let aa = SS as f32; // 1 logical px

    for y in 0..h {
        for x in 0..w {
            let sdf = round_rect_sdf(x, y, r, hw, hh);
            let i = (y * w + x) * 4;
            if sdf >= 0.0 {
                buf[i + 3] = 0;
            } else if sdf > -aa {
                let cover = (-sdf / aa).clamp(0.0, 1.0);
                buf[i + 3] = (buf[i + 3] as f32 * cover) as u8;
            }
        }
    }
}

/// Minimal PNG encoder (RGBA8, no filter) using the `png` crate.
fn encode_png(rgba: &[u8], w: usize, h: usize) -> Vec<u8> {
    let mut out = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut out, w as u32, h as u32);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().expect("png header");
        writer.write_image_data(rgba).expect("png data");
    }
    out
}
