//! Procedural pixel art: the hero landscape, block icons and the player head.
//! Everything is drawn from code, so Scaleri ships no texture files except its logo.

use egui::{pos2, Color32, Painter, Pos2, Rect, Vec2};

/// Small deterministic hash, so the art looks the same on every repaint.
fn hash(x: i32, y: i32) -> u32 {
    let mut h = (x as u32)
        .wrapping_mul(374_761_393)
        .wrapping_add((y as u32).wrapping_mul(668_265_263));
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    h ^ (h >> 16)
}

/// Smooth 1D value noise in [0, 1].
fn noise1d(x: f32, seed: i32) -> f32 {
    let i = x.floor() as i32;
    let f = x - i as f32;
    let a = (hash(i, seed) & 0xffff) as f32 / 65535.0;
    let b = (hash(i + 1, seed) & 0xffff) as f32 / 65535.0;
    let t = f * f * (3.0 - 2.0 * f);
    a + (b - a) * t
}

fn rgb(hex: u32) -> Color32 {
    Color32::from_rgb((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
}

fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let l = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t.clamp(0.0, 1.0)).round() as u8;
    Color32::from_rgb(l(a.r(), b.r()), l(a.g(), b.g()), l(a.b(), b.b()))
}

fn jitter(c: Color32, h: u32, amount: i32) -> Color32 {
    let d = (h % (2 * amount as u32 + 1)) as i32 - amount;
    let f = |v: u8| (v as i32 + d).clamp(0, 255) as u8;
    Color32::from_rgb(f(c.r()), f(c.g()), f(c.b()))
}

fn darker(c: Color32, factor: f32) -> Color32 {
    let f = |v: u8| (v as f32 / factor) as u8;
    Color32::from_rgb(f(c.r()), f(c.g()), f(c.b()))
}

#[allow(clippy::too_many_arguments)]
fn block(p: &Painter, origin: Pos2, b: f32, x: i32, y: i32, w: i32, h: i32, c: Color32) {
    let min = origin + Vec2::new(x as f32 * b, y as f32 * b);
    p.rect_filled(Rect::from_min_size(min, Vec2::new(w as f32 * b, h as f32 * b)), 0.0, c);
}

/// A dusk landscape in 12px blocks: banded sky, stars, a square moon, mountains, forest and grass.
pub fn paint_landscape(p: &Painter, rect: Rect) {
    let b = 12.0;
    let o = rect.min;
    let cols = (rect.width() / b) as i32 + 1;
    let rows = (rect.height() / b) as i32 + 1;

    let sky_top = rgb(0x161833);
    let sky_mid = rgb(0x2e2a5c);
    let sky_low = rgb(0x7a4b6e);
    let far_hills = rgb(0x25243e);
    let mid_hills = rgb(0x1d302a);
    let grass = rgb(0x5fae34);
    let grass_top = rgb(0x8bd44a);
    let dirt = rgb(0x6b4a2f);
    let stone = rgb(0x555658);

    for y in 0..rows {
        let t = y as f32 / (rows - 1).max(1) as f32;
        let row = if t < 0.55 {
            mix(sky_top, sky_mid, t / 0.55)
        } else {
            mix(sky_mid, sky_low, (t - 0.55) / 0.45)
        };
        for x in 0..cols {
            block(p, o, b, x, y, 1, 1, jitter(row, hash(x, y), 3));
        }
    }

    for y in 0..rows / 2 {
        for x in 0..cols {
            let h = hash(x, y + 1000);
            if h.is_multiple_of(23) {
                let s = if (h >> 8).is_multiple_of(3) { 4.0 } else { 2.0 };
                let px = o.x + x as f32 * b + ((h >> 4) % (b - s) as u32) as f32;
                let py = o.y + y as f32 * b + ((h >> 12) % (b - s) as u32) as f32;
                p.rect_filled(Rect::from_min_size(pos2(px, py), Vec2::splat(s)), 0.0, rgb(0xe8e6ff));
            }
        }
    }

    let moon = cols - 9;
    block(p, o, b, moon, 2, 3, 3, rgb(0xf2eed8));
    block(p, o, b, moon + 1, 3, 1, 1, rgb(0xd8d3b8));
    block(p, o, b, moon + 2, 2, 1, 1, rgb(0xe2ddc4));

    let ridge = |scale: f32, seed: i32, base: f32, amp: f32, x: i32| {
        let n = noise1d(x as f32 / scale, seed) * 0.7 + noise1d(x as f32 / (scale / 3.0), seed + 7) * 0.3;
        (rows as f32 * base - n * amp) as i32
    };

    for x in 0..cols {
        let top = ridge(14.0, 1, 0.78, rows as f32 * 0.45, x);
        for y in top..rows {
            block(p, o, b, x, y, 1, 1, jitter(far_hills, hash(x, y + 2000), 2));
        }
    }

    for x in 0..cols {
        let top = ridge(9.0, 2, 0.92, rows as f32 * 0.32, x);
        for y in top..rows {
            block(p, o, b, x, y, 1, 1, jitter(mid_hills, hash(x, y + 3000), 3));
        }
        if hash(x, 4000).is_multiple_of(9) {
            block(
                p,
                o,
                b,
                x,
                top - 2,
                1,
                2,
                darker(jitter(mid_hills, hash(x, 4001), 2), 1.15),
            );
            block(p, o, b, x - 1, top - 4, 3, 2, jitter(mid_hills, hash(x, 4002), 4));
        }
    }

    for x in 0..cols {
        let top = rows - 3 - (noise1d(x as f32 / 6.0, 3) * 2.5) as i32;
        for y in top..rows {
            let c = if y == top {
                jitter(grass, hash(x, y + 5000), 6)
            } else if y - top > 3 && hash(x, y + 5001).is_multiple_of(4) {
                jitter(stone, hash(x, y + 5002), 6)
            } else {
                jitter(dirt, hash(x, y + 5003), 7)
            };
            block(p, o, b, x, y, 1, 1, c);
        }
        p.rect_filled(
            Rect::from_min_size(o + Vec2::new(x as f32 * b, top as f32 * b), Vec2::new(b, 3.0)),
            0.0,
            grass_top,
        );
    }

    // Darken the left side and the bottom so text and buttons stay readable
    let shade = Color32::from_rgb(18, 18, 19);
    let steps = 48;
    for i in 0..steps {
        let t = i as f32 / steps as f32;
        let alpha = if t < 0.45 {
            235.0 - (235.0 - 170.0) * t / 0.45
        } else {
            170.0 - 150.0 * (t - 0.45) / 0.55
        };
        let x0 = rect.min.x + rect.width() * t;
        let strip = Rect::from_min_max(
            pos2(x0, rect.min.y),
            pos2(x0 + rect.width() / steps as f32 + 1.0, rect.max.y),
        );
        p.rect_filled(
            strip,
            0.0,
            Color32::from_rgba_unmultiplied(shade.r(), shade.g(), shade.b(), alpha as u8),
        );
    }
    for i in 0..24 {
        let t = i as f32 / 24.0;
        let y0 = rect.center().y + rect.height() * 0.5 * t;
        let strip = Rect::from_min_max(pos2(rect.min.x, y0), pos2(rect.max.x, y0 + rect.height() / 48.0 + 1.0));
        p.rect_filled(
            strip,
            0.0,
            Color32::from_rgba_unmultiplied(shade.r(), shade.g(), shade.b(), (t * 160.0) as u8),
        );
    }
    p.rect_filled(
        Rect::from_min_max(pos2(rect.min.x, rect.max.y - 2.0), rect.max),
        0.0,
        rgb(0x121213),
    );
}

/// Draws an 8x8 pixel block icon.
pub fn paint_block_icon(p: &Painter, rect: Rect, kind: &str) {
    let px = rect.width() / 8.0;
    let seed = kind
        .bytes()
        .fold(7i32, |a, c| a.wrapping_mul(31).wrapping_add(c as i32));
    for y in 0..8 {
        for x in 0..8 {
            let h = hash(x + seed, y * 13 + seed);
            let c = block_pixel(kind, x, y, h);
            let Some(mut c) = c else { continue };
            // Light from the top left, like an inventory item
            if x == 7 || y == 7 {
                c = darker(c, 1.25);
            }
            let min = rect.min + Vec2::new(x as f32 * px, y as f32 * px);
            p.rect_filled(Rect::from_min_size(min, Vec2::splat(px + 0.5)), 0.0, c);
        }
    }
}

fn block_pixel(kind: &str, x: i32, y: i32, h: u32) -> Option<Color32> {
    let dirt = || jitter(rgb(0x79553a), h, 10);
    Some(match kind {
        "grass" => {
            if y == 0 || (y == 1 && !h.is_multiple_of(3)) || (y == 2 && h.is_multiple_of(5)) {
                jitter(rgb(0x5fae34), h >> 3, 12)
            } else {
                dirt()
            }
        }
        "snow" => {
            if y < 2 || (y == 2 && h.is_multiple_of(2)) {
                jitter(rgb(0xf0f6fa), h >> 3, 6)
            } else {
                dirt()
            }
        }
        "stone" => {
            if h.is_multiple_of(7) {
                rgb(0x6a6a6a)
            } else {
                jitter(rgb(0x8a8a8a), h, 12)
            }
        }
        "planks" => {
            if y % 2 == 1 && (x + y) % 4 != 0 {
                rgb(0x6e5230)
            } else if (y % 4 == 0 && x == 5) || (y % 4 == 2 && x == 1) {
                rgb(0x5e4426)
            } else {
                jitter(rgb(0xa8834e), h, 8)
            }
        }
        "sand" => jitter(rgb(0xdccf9c), h, 10),
        "diamond" => {
            let ore = matches!(
                (x, y),
                (1, 1) | (2, 1) | (2, 2) | (5, 2) | (5, 3) | (6, 3) | (1, 5) | (2, 5) | (4, 6) | (5, 6)
            );
            if ore {
                if h.is_multiple_of(2) {
                    rgb(0x5decf5)
                } else {
                    rgb(0x2bb4c0)
                }
            } else {
                jitter(rgb(0x7f7f7f), h, 12)
            }
        }
        "tnt" => {
            if y == 3 || y == 4 {
                if (y == 3 && matches!(x, 1 | 2 | 3 | 5 | 6)) || (y == 4 && matches!(x, 2 | 5)) {
                    rgb(0x2a2a2a)
                } else {
                    rgb(0xe8e8e8)
                }
            } else if x % 2 == 0 {
                jitter(rgb(0xdb2b1d), h, 8)
            } else {
                jitter(rgb(0xa8180f), h, 8)
            }
        }
        "obsidian" => {
            if h.is_multiple_of(6) {
                rgb(0x3b2754)
            } else {
                jitter(rgb(0x14101e), h, 6)
            }
        }
        "netherrack" => jitter(
            if h.is_multiple_of(4) {
                rgb(0x5e1f1f)
            } else {
                rgb(0x7a2f2f)
            },
            h >> 2,
            10,
        ),
        "glass" => {
            if x == 0 || y == 0 || x == 7 || y == 7 {
                rgb(0xcfeef2)
            } else if x + y == 5 || x + y == 6 {
                Color32::from_rgba_unmultiplied(255, 255, 255, 140)
            } else {
                return None;
            }
        }
        _ => jitter(rgb(0x808080), h, 10),
    })
}

/// A Steve-style 8x8 face used for the account avatar.
pub fn paint_head(p: &Painter, rect: Rect, seed: &str) {
    let skin_tones = [0xc69c6d, 0xb5835a, 0x8d5a3b, 0xe0b48b];
    let hair_tones = [0x3b2a1a, 0x1f1a14, 0x6b4a2a, 0xa0763a];
    let s = seed
        .bytes()
        .fold(0u32, |a, c| a.wrapping_mul(31).wrapping_add(c as u32));
    let skin = rgb(skin_tones[(s % 4) as usize]);
    let hair = rgb(hair_tones[((s >> 3) % 4) as usize]);
    let px = rect.width() / 8.0;
    for y in 0..8 {
        for x in 0..8 {
            let h = hash(x, y + s as i32);
            let c = if y < 2 || (y == 2 && (x == 0 || x == 7)) {
                jitter(hair, h, 6)
            } else if y == 4 && (x == 1 || x == 6) {
                Color32::WHITE
            } else if y == 4 && (x == 2 || x == 5) {
                rgb(0x4a5ec8)
            } else if y == 6 && (3..=4).contains(&x) {
                darker(skin, 1.6)
            } else if y == 5 && (3..=4).contains(&x) {
                darker(skin, 1.2)
            } else {
                jitter(skin, h, 5)
            };
            let min = rect.min + Vec2::new(x as f32 * px, y as f32 * px);
            p.rect_filled(Rect::from_min_size(min, Vec2::splat(px + 0.5)), 0.0, c);
        }
    }
}

/// Small pixel glyphs for the sidebar.
#[derive(Clone, Copy)]
pub enum Glyph {
    Plus,
    Gear,
    Chest,
    Grass,
    Brush,
}

pub fn paint_glyph(p: &Painter, rect: Rect, glyph: Glyph) {
    let rows: [&str; 8] = match glyph {
        Glyph::Plus => [
            "........", "...gg...", "...gg...", ".gggggg.", ".GGGGGG.", "...GG...", "...GG...", "........",
        ],
        Glyph::Gear => [
            "...ss...", ".s.ss.s.", "..ssss..", "sss..sss", "SSS..SSS", "..SSSS..", ".S.SS.S.", "...SS...",
        ],
        Glyph::Chest => [
            "........", "dddddddd", "dwwwwwwd", "dddyyddd", "dwwyywwd", "dwwwwwwd", "dddddddd", "........",
        ],
        Glyph::Grass => [
            "gggggggg", "GgGggGgG", "dGddGddG", "dddddddd", "dwdddwdd", "dddddddd", "ddwddddw", "dddddddd",
        ],
        Glyph::Brush => [
            "......ss", ".....sSs", "....sSs.", "...sSs..", "..wws...", ".ywww...", "yyyw....", "yy......",
        ],
    };
    let px = rect.width() / 8.0;
    for (y, row) in rows.iter().enumerate() {
        for (x, ch) in row.bytes().enumerate() {
            let c = match ch {
                b'g' => rgb(0x8bd44a),
                b'G' => rgb(0x5fae34),
                b's' => rgb(0xd0d0d4),
                b'S' => rgb(0x9a9aa0),
                b'd' => rgb(0x4a3018),
                b'w' => rgb(0xa8783e),
                b'y' => rgb(0xfcd34c),
                _ => continue,
            };
            let min = rect.min + Vec2::new(x as f32 * px, y as f32 * px);
            p.rect_filled(Rect::from_min_size(min, Vec2::splat(px + 0.5)), 0.0, c);
        }
    }
}
