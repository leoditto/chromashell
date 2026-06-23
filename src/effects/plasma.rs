use crate::color::hsv_to_rgb;
use super::{Cell, Effect, TextGrid};

const CHARS: &[char] = &[' ', '·', ':', '░', '▒', '▓', '█'];

pub struct Plasma {
    time: f64,
    width: usize,
    height: usize,
    cells: Vec<Cell>,
    text: Option<TextGrid>,
    density: f32,
}

impl Plasma {
    pub fn new() -> Self {
        Self {
            time: 0.0,
            width: 0,
            height: 0,
            cells: Vec::new(),
            text: None,
            density: -1.0,
        }
    }
}

impl Effect for Plasma {
    fn name(&self) -> &str {
        "plasma"
    }

    fn update(&mut self, dt: f64) {
        self.time += dt;
        if self.width == 0 || self.height == 0 {
            return;
        }

        let t = self.time as f32;
        let w = self.width;
        let h = self.height;
        let has_text = self.text.is_some();
        let intensity = if self.density >= 0.0 {
            self.density
        } else if has_text {
            0.7
        } else {
            1.0
        };

        for row in 0..h {
            let y = row as f32 / h as f32;
            let cy = y * 8.0;
            for col in 0..w {
                let x = col as f32 / w as f32;
                let cx = x * 8.0;

                let v1 = (cx + t * 1.5).sin();
                let v2 = ((cy * 1.3 + t * 0.7).sin() + (cx * 0.8 + t).cos()) * 0.5;
                let v3 = ((cx * cx + cy * cy).sqrt() * 0.5 - t * 2.0).sin();
                let v4 = ((cx * 0.5 + cy * 0.5 + t).sin()
                    + (cx * cx * 0.01 + cy * cy * 0.01).sqrt().sin())
                    * 0.5;

                let v = ((v1 + v2 + v3 + v4) * 0.25 + 1.0) * 0.5;
                let hue = (v * 120.0 + t * 40.0 + 200.0) % 360.0;

                let is_text = self.text.as_ref().map_or(false, |g| g.has_char(col, row));

                let (ch, fg, bg) = if is_text {
                    let ch = self.text.as_ref().unwrap().char_at(col, row);
                    // Subtle color shift on text
                    let base = 180.0;
                    let shift = v * intensity * 80.0;
                    let rgb = hsv_to_rgb(hue, 0.3 * intensity, 0.75 + v * 0.15);
                    let fg = (
                        ((base + shift).min(255.0) as u8).max(rgb.0 / 2),
                        ((base + shift * 0.5).min(255.0) as u8).max(rgb.1 / 2),
                        ((base).min(255.0) as u8).max(rgb.2 / 2),
                    );
                    (ch, fg, (0, 0, 0))
                } else if has_text {
                    let vi = v * intensity;
                    if vi < 0.02 {
                        (' ', (0, 0, 0), (0, 0, 0))
                    } else {
                        let ch_idx = (vi * 4.0).min((CHARS.len() - 1) as f32) as usize;
                        let ch = CHARS[ch_idx];
                        let fg = hsv_to_rgb(hue, 0.8, vi * 0.8);
                        (ch, fg, (0, 0, 0))
                    }
                } else {
                    // Standalone mode
                    let char_idx = (v * v * (CHARS.len() - 1) as f32) as usize;
                    let ch = CHARS[char_idx.min(CHARS.len() - 1)];
                    let fg = hsv_to_rgb(hue, 0.9, (0.3 + v * 0.7).min(1.0));
                    let bg_val = v * 0.08;
                    let bg = hsv_to_rgb(hue, 0.5, bg_val.min(1.0));
                    (ch, fg, bg)
                };

                self.cells[row * w + col] = Cell { ch, fg, bg };
            }
        }
    }

    fn cell_at(&self, col: usize, row: usize) -> Cell {
        if self.width == 0 {
            return Cell::blank();
        }
        self.cells[row * self.width + col]
    }

    fn resize(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.cells = vec![Cell::blank(); width * height];
    }

    fn set_text(&mut self, grid: TextGrid) {
        self.text = Some(grid);
    }

    fn set_density(&mut self, density: f32) {
        self.density = density.clamp(0.0, 1.0);
    }
}
