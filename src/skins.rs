//! Minecraft skins: the 64×64 texture layout, editing helpers and the local skin library.
//!
//! Every body part is a box whose six faces are unwrapped onto the texture in the same pattern:
//!
//! ```text
//!         [top][bottom]
//!   [right][front][left][back]
//! ```
//!
//! Each part has a base layer and an outer layer (hat, jacket, sleeves, trousers) drawn slightly
//! larger around it.

use std::path::PathBuf;

use crate::net::Result;
use crate::paths;

pub const SIZE: usize = 64;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Part {
    Head,
    Body,
    RightArm,
    LeftArm,
    RightLeg,
    LeftLeg,
}

impl Part {
    pub const ALL: [Part; 6] = [
        Part::Head,
        Part::Body,
        Part::RightArm,
        Part::LeftArm,
        Part::RightLeg,
        Part::LeftLeg,
    ];

    /// Width, height and depth in pixels. Slim ("Alex") arms are a pixel narrower.
    pub fn size(self, slim: bool) -> [usize; 3] {
        match self {
            Part::Head => [8, 8, 8],
            Part::Body => [8, 12, 4],
            Part::RightArm | Part::LeftArm if slim => [3, 12, 4],
            _ => [4, 12, 4],
        }
    }

    /// Top-left corner of the part's unwrapped faces, for the base and the outer layer.
    fn origin(self, outer: bool) -> (usize, usize) {
        match (self, outer) {
            (Part::Head, false) => (0, 0),
            (Part::Head, true) => (32, 0),
            (Part::Body, false) => (16, 16),
            (Part::Body, true) => (16, 32),
            (Part::RightArm, false) => (40, 16),
            (Part::RightArm, true) => (40, 32),
            (Part::LeftArm, false) => (32, 48),
            (Part::LeftArm, true) => (48, 48),
            (Part::RightLeg, false) => (0, 16),
            (Part::RightLeg, true) => (0, 32),
            (Part::LeftLeg, false) => (16, 48),
            (Part::LeftLeg, true) => (0, 48),
        }
    }

    /// The same part on the other side of the body.
    pub fn mirrored(self) -> Part {
        match self {
            Part::RightArm => Part::LeftArm,
            Part::LeftArm => Part::RightArm,
            Part::RightLeg => Part::LeftLeg,
            Part::LeftLeg => Part::RightLeg,
            other => other,
        }
    }

    /// Where the centre of the part sits on the player, in pixels, with the feet at y = 0.
    pub fn center(self, slim: bool) -> [f32; 3] {
        let arm = if slim { 5.5 } else { 6.0 };
        match self {
            Part::Head => [0.0, 28.0, 0.0],
            Part::Body => [0.0, 18.0, 0.0],
            Part::RightArm => [-arm, 18.0, 0.0],
            Part::LeftArm => [arm, 18.0, 0.0],
            Part::RightLeg => [-2.0, 6.0, 0.0],
            Part::LeftLeg => [2.0, 6.0, 0.0],
        }
    }
}

/// A box face. "Right" is the player's right, which faces the viewer's left.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Face {
    Top,
    Bottom,
    Right,
    Front,
    Left,
    Back,
}

impl Face {
    pub const ALL: [Face; 6] = [
        Face::Top,
        Face::Bottom,
        Face::Right,
        Face::Front,
        Face::Left,
        Face::Back,
    ];

    fn mirrored(self) -> Face {
        match self {
            Face::Right => Face::Left,
            Face::Left => Face::Right,
            other => other,
        }
    }
}

/// A rectangle on the texture: x, y, width, height.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Area {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

impl Area {
    pub fn contains(&self, x: usize, y: usize) -> bool {
        x >= self.x && x < self.x + self.w && y >= self.y && y < self.y + self.h
    }
}

/// Where one face of a part is drawn on the texture.
pub fn face_area(part: Part, face: Face, outer: bool, slim: bool) -> Area {
    let [w, h, d] = part.size(slim);
    let (u, v) = part.origin(outer);
    let (x, y, fw, fh) = match face {
        Face::Top => (u + d, v, w, d),
        Face::Bottom => (u + d + w, v, w, d),
        Face::Right => (u, v + d, d, h),
        Face::Front => (u + d, v + d, w, h),
        Face::Left => (u + d + w, v + d, d, h),
        Face::Back => (u + d + w + d, v + d, w, h),
    };
    Area { x, y, w: fw, h: fh }
}

/// A pixel's place on the player: which part, layer and face it belongs to, and where on that face.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Spot {
    pub part: Part,
    pub face: Face,
    pub outer: bool,
    pub col: usize,
    pub row: usize,
}

/// Every face rectangle on the texture.
pub fn all_faces(slim: bool) -> impl Iterator<Item = (Part, Face, bool, Area)> {
    [false, true].into_iter().flat_map(move |outer| {
        Part::ALL.into_iter().flat_map(move |part| {
            Face::ALL
                .into_iter()
                .map(move |face| (part, face, outer, face_area(part, face, outer, slim)))
        })
    })
}

pub fn locate(x: usize, y: usize, slim: bool) -> Option<Spot> {
    all_faces(slim)
        .find(|(_, _, _, area)| area.contains(x, y))
        .map(|(part, face, outer, area)| Spot {
            part,
            face,
            outer,
            col: x - area.x,
            row: y - area.y,
        })
}

/// The pixel on the other side of the body that mirrors (x, y), e.g. the left hand for the right.
pub fn mirror(x: usize, y: usize, slim: bool) -> Option<(usize, usize)> {
    let spot = locate(x, y, slim)?;
    let area = face_area(spot.part.mirrored(), spot.face.mirrored(), spot.outer, slim);
    Some((area.x + area.w - 1 - spot.col, area.y + spot.row))
}

#[derive(Clone, PartialEq)]
pub struct Skin {
    /// 64×64 RGBA pixels, row by row.
    pub pixels: Vec<u8>,
    pub slim: bool,
}

impl Skin {
    pub fn empty() -> Self {
        Skin {
            pixels: vec![0; SIZE * SIZE * 4],
            slim: false,
        }
    }

    pub fn get(&self, x: usize, y: usize) -> [u8; 4] {
        let i = (y * SIZE + x) * 4;
        [
            self.pixels[i],
            self.pixels[i + 1],
            self.pixels[i + 2],
            self.pixels[i + 3],
        ]
    }

    pub fn set(&mut self, x: usize, y: usize, rgba: [u8; 4]) {
        if x < SIZE && y < SIZE {
            let i = (y * SIZE + x) * 4;
            self.pixels[i..i + 4].copy_from_slice(&rgba);
        }
    }

    fn fill_area(&mut self, area: Area, rgba: [u8; 4]) {
        for y in area.y..area.y + area.h {
            for x in area.x..area.x + area.w {
                self.set(x, y, rgba);
            }
        }
    }

    /// A simple starting skin: hair, a face, a teal shirt, jeans and shoes. No outer layer.
    pub fn template() -> Self {
        let mut skin = Skin::empty();
        let skin_tone = [0xc6, 0x8e, 0x6c, 255];
        let hair = [0x3b, 0x26, 0x17, 255];
        let shirt = [0x2a, 0x9d, 0x8f, 255];
        let jeans = [0x2f, 0x4b, 0x8a, 255];
        let shoes = [0x55, 0x55, 0x5c, 255];
        for face in Face::ALL {
            skin.fill_area(face_area(Part::Head, face, false, false), skin_tone);
            skin.fill_area(face_area(Part::Body, face, false, false), shirt);
            for arm in [Part::RightArm, Part::LeftArm] {
                skin.fill_area(face_area(arm, face, false, false), skin_tone);
            }
            for leg in [Part::RightLeg, Part::LeftLeg] {
                skin.fill_area(face_area(leg, face, false, false), jeans);
            }
        }
        // Hair on top, at the back and around the forehead
        skin.fill_area(face_area(Part::Head, Face::Top, false, false), hair);
        skin.fill_area(face_area(Part::Head, Face::Back, false, false), hair);
        for face in [Face::Front, Face::Right, Face::Left] {
            let a = face_area(Part::Head, face, false, false);
            skin.fill_area(Area { h: 2, ..a }, hair);
        }
        let front = face_area(Part::Head, Face::Front, false, false);
        for (dx, rgba) in [(1, [255, 255, 255, 255]), (2, [0x3a, 0x5f, 0xc8, 255])] {
            skin.set(front.x + dx, front.y + 4, rgba);
            skin.set(front.x + 7 - dx, front.y + 4, rgba);
        }
        for dx in 3..5 {
            skin.set(front.x + dx, front.y + 6, [0x8a, 0x52, 0x3c, 255]);
        }
        // Short sleeves, then shoes
        for arm in [Part::RightArm, Part::LeftArm] {
            for face in [Face::Top, Face::Right, Face::Front, Face::Left, Face::Back] {
                let a = face_area(arm, face, false, false);
                let h = if face == Face::Top { a.h } else { 4 };
                skin.fill_area(Area { h, ..a }, shirt);
            }
        }
        for leg in [Part::RightLeg, Part::LeftLeg] {
            skin.fill_area(face_area(leg, Face::Bottom, false, false), shoes);
            for face in [Face::Right, Face::Front, Face::Left, Face::Back] {
                let a = face_area(leg, face, false, false);
                skin.fill_area(
                    Area {
                        y: a.y + a.h - 2,
                        h: 2,
                        ..a
                    },
                    shoes,
                );
            }
        }
        skin
    }

    /// Reads a skin PNG. Old 64×32 skins are converted the way the game does: the left arm and
    /// leg become mirror images of the right ones.
    pub fn from_png(data: &[u8]) -> Result<Skin> {
        let image = image::load_from_memory(data)
            .map_err(|e| format!("That isn't a PNG image Scaleri can read: {e}"))?
            .to_rgba8();
        let (w, h) = image.dimensions();
        if w != 64 || (h != 64 && h != 32) {
            return Err(format!("Skins are 64×64 pixels, but this image is {w}×{h}."));
        }
        let mut skin = Skin::empty();
        for (x, y, p) in image.enumerate_pixels() {
            skin.set(x as usize, y as usize, p.0);
        }
        if h == 32 {
            for (part, mirrored) in [(Part::RightArm, Part::LeftArm), (Part::RightLeg, Part::LeftLeg)] {
                for face in Face::ALL {
                    let from = face_area(part, face, false, false);
                    let to = face_area(mirrored, face.mirrored(), false, false);
                    for row in 0..from.h {
                        for col in 0..from.w {
                            let rgba = skin.get(from.x + col, from.y + row);
                            skin.set(to.x + to.w - 1 - col, to.y + row, rgba);
                        }
                    }
                }
            }
        } else {
            skin.slim = looks_slim(&skin);
        }
        Ok(skin)
    }

    pub fn to_png(&self) -> Vec<u8> {
        let image = image::RgbaImage::from_raw(SIZE as u32, SIZE as u32, self.pixels.clone()).expect("64×64 pixels");
        let mut png = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut png);
        image
            .write_to(&mut cursor, image::ImageFormat::Png)
            .expect("encoding a PNG in memory");
        png
    }

    /// Fills the pixels around (x, y) that have its colour, without leaving its face.
    pub fn flood_fill(&mut self, x: usize, y: usize, rgba: [u8; 4]) {
        let Some(spot) = locate(x, y, self.slim) else {
            return;
        };
        let area = face_area(spot.part, spot.face, spot.outer, self.slim);
        let target = self.get(x, y);
        if target == rgba {
            return;
        }
        let mut stack = vec![(x, y)];
        while let Some((x, y)) = stack.pop() {
            if !area.contains(x, y) || self.get(x, y) != target {
                continue;
            }
            self.set(x, y, rgba);
            stack.extend([(x + 1, y), (x.wrapping_sub(1), y), (x, y + 1), (x, y.wrapping_sub(1))]);
        }
    }

    /// The 8×8 face with the hat layer on top, as RGBA pixels.
    pub fn face_pixels(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(8 * 8 * 4);
        for y in 0..8 {
            for x in 0..8 {
                let base = self.get(8 + x, 8 + y);
                let hat = self.get(40 + x, 8 + y);
                out.extend_from_slice(if hat[3] > 0 { &hat } else { &base });
            }
        }
        out
    }

    /// The player seen from the front, outer layer included, as 16×32 RGBA pixels.
    pub fn front_view(&self) -> Vec<u8> {
        let mut out = vec![0u8; 16 * 32 * 4];
        let arm = if self.slim { 1 } else { 0 };
        let places = [
            (Part::Head, 4, 0),
            (Part::Body, 4, 8),
            (Part::RightArm, arm, 8),
            (Part::LeftArm, 12, 8),
            (Part::RightLeg, 4, 20),
            (Part::LeftLeg, 8, 20),
        ];
        for outer in [false, true] {
            for (part, ox, oy) in places {
                let area = face_area(part, Face::Front, outer, self.slim);
                for row in 0..area.h {
                    for col in 0..area.w {
                        let rgba = self.get(area.x + col, area.y + row);
                        if rgba[3] == 0 {
                            continue;
                        }
                        let i = ((oy + row) * 16 + ox + col) * 4;
                        out[i..i + 4].copy_from_slice(&rgba);
                    }
                }
            }
        }
        out
    }
}

/// Slim skins leave the outer column of each arm empty.
fn looks_slim(skin: &Skin) -> bool {
    (20..32).all(|y| skin.get(54, y)[3] == 0 && skin.get(55, y)[3] == 0)
}

// ---------------------------------------------------------------------------------------------
// The skin library: PNGs saved under skins/library/

pub fn library_dir() -> PathBuf {
    paths::data_dir().join("skins").join("library")
}

/// Saved skin names, newest first.
pub fn library() -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(library_dir()) else {
        return Vec::new();
    };
    let mut list: Vec<(std::time::SystemTime, String)> = entries
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().strip_suffix(".png")?.to_string();
            let modified = e.metadata().and_then(|m| m.modified()).ok()?;
            Some((modified, name))
        })
        .collect();
    list.sort_by_key(|entry| std::cmp::Reverse(entry.0));
    list.into_iter().map(|(_, name)| name).collect()
}

/// Turns a skin name into a safe file name.
pub fn file_name(name: &str) -> String {
    let cleaned: String = name
        .trim()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || " -_()".contains(c) {
                c
            } else {
                '_'
            }
        })
        .collect();
    let cleaned = cleaned.trim().to_string();
    if cleaned.is_empty() {
        "My skin".into()
    } else {
        cleaned
    }
}

pub fn save_to_library(name: &str, skin: &Skin) -> std::io::Result<String> {
    let name = file_name(name);
    paths::write_atomic(&library_dir().join(format!("{name}.png")), &skin.to_png())?;
    Ok(name)
}

pub fn load_from_library(name: &str) -> Result<Skin> {
    let data =
        std::fs::read(library_dir().join(format!("{name}.png"))).map_err(|e| format!("Could not open {name}: {e}"))?;
    Skin::from_png(&data)
}

pub fn delete_from_library(name: &str) -> std::io::Result<()> {
    std::fs::remove_file(library_dir().join(format!("{name}.png")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn faces_match_the_minecraft_layout() {
        assert_eq!(
            face_area(Part::Head, Face::Front, false, false),
            Area { x: 8, y: 8, w: 8, h: 8 }
        );
        assert_eq!(
            face_area(Part::Head, Face::Front, true, false),
            Area {
                x: 40,
                y: 8,
                w: 8,
                h: 8
            }
        );
        assert_eq!(
            face_area(Part::Body, Face::Back, false, false),
            Area {
                x: 32,
                y: 20,
                w: 8,
                h: 12
            }
        );
        assert_eq!(
            face_area(Part::LeftArm, Face::Front, false, true),
            Area {
                x: 36,
                y: 52,
                w: 3,
                h: 12
            }
        );
        assert_eq!(
            face_area(Part::LeftLeg, Face::Top, true, false),
            Area {
                x: 4,
                y: 48,
                w: 4,
                h: 4
            }
        );
    }

    #[test]
    fn faces_never_overlap() {
        for slim in [false, true] {
            let mut used = vec![false; SIZE * SIZE];
            for (_, _, _, area) in all_faces(slim) {
                for y in area.y..area.y + area.h {
                    for x in area.x..area.x + area.w {
                        assert!(!used[y * SIZE + x], "pixel {x},{y} is in two faces");
                        used[y * SIZE + x] = true;
                    }
                }
            }
        }
    }

    #[test]
    fn mirroring_twice_returns_the_same_pixel() {
        for slim in [false, true] {
            for (_, _, _, area) in all_faces(slim) {
                for y in area.y..area.y + area.h {
                    for x in area.x..area.x + area.w {
                        let (mx, my) = mirror(x, y, slim).unwrap();
                        assert_eq!(mirror(mx, my, slim), Some((x, y)));
                    }
                }
            }
        }
        // The right hand's front mirrors onto the left hand's front
        assert_eq!(mirror(44, 20, false), Some((39, 52)));
    }

    #[test]
    fn png_round_trip_keeps_pixels_and_model() {
        let mut skin = Skin::template();
        skin.set(50, 20, [1, 2, 3, 255]);
        let back = Skin::from_png(&skin.to_png()).unwrap();
        assert_eq!(back.pixels, skin.pixels);
        assert!(!back.slim);

        let mut slim = Skin::empty();
        slim.slim = true;
        slim.set(8, 8, [9, 9, 9, 255]);
        assert!(Skin::from_png(&slim.to_png()).unwrap().slim);
    }

    #[test]
    fn old_skins_get_mirrored_left_limbs() {
        let mut image = image::RgbaImage::new(64, 32);
        // Leftmost pixel of the right leg's front
        image.put_pixel(4, 20, image::Rgba([200, 10, 10, 255]));
        let mut png = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut png);
        image.write_to(&mut cursor, image::ImageFormat::Png).unwrap();
        let skin = Skin::from_png(&png).unwrap();
        // ...becomes the rightmost pixel of the left leg's front
        assert_eq!(skin.get(23, 52), [200, 10, 10, 255]);
    }

    #[test]
    fn fill_stays_on_one_face() {
        let mut skin = Skin::empty();
        skin.flood_fill(8, 8, [5, 5, 5, 255]);
        assert_eq!(skin.get(15, 15), [5, 5, 5, 255]);
        assert_eq!(skin.get(16, 8)[3], 0, "the next face is untouched");
        assert_eq!(skin.get(7, 8)[3], 0);
    }

    #[test]
    fn file_names_are_safe() {
        assert_eq!(file_name("  Cool/Skin?  "), "Cool_Skin_");
        assert_eq!(file_name("..."), "___");
        assert_eq!(file_name(""), "My skin");
    }
}
