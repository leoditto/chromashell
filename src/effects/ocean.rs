use super::{Cell, Effect, TextGrid};
use crate::color::hsv_to_rgb;

const WAVE_CHARS: &[char] = &[' ', '~', '≈', '∼', '≋'];

pub struct Ocean {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
    text: Option<TextGrid>,
    density: f32,
    time: f64,
}

impl Ocean {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            cells: Vec::new(),
            text: None,
            density: -1.0,
            time: 0.0,
        }
    }
}

impl Effect for Ocean {
    fn name(&self) -> &str {
        "ocean"
    }

    fn update(&mut self, dt: f64) {
        self.time += dt;
        let w = self.width;
        let h = self.height;
        if w == 0 || h == 0 {
            return;
        }

        let t = self.time as f32;
        let has_text = self.text.is_some();
        let intensity = if self.density >= 0.0 {
            self.density
        } else if has_text {
            0.6
        } else {
            1.0
        };

        for row in 0..h {
            let y = row as f32 / h as f32;
            for col in 0..w {
                let x = col as f32 / w as f32;
                let idx = row * w + col;

                // Overlapping wave functions
                let w1 = (x * 6.0 + t * 1.2 + y * 2.0).sin();
                let w2 = (x * 10.0 - t * 0.8 + y * 3.0).sin() * 0.5;
                let w3 = (x * 3.0 + t * 0.5 + y * 5.0).cos() * 0.3;
                let wave = (w1 + w2 + w3) * 0.33 + 0.5;

                // Depth gradient
                let depth = 0.5 + y * 0.4;

                let v = wave * depth * intensity;

                // Foam on wave crests
                let is_crest = wave > 0.75;

                // Hue shifts between deep blue and teal
                let hue = 190.0 + wave * 30.0 + y * 20.0;

                let is_text = self.text.as_ref().map_or(false, |g| g.has_char(col, row));

                if is_text {
                    let ch = self.text.as_ref().unwrap().char_at(col, row);
                    let tint = hsv_to_rgb(hue % 360.0, 0.3 * intensity, 0.15 * v);
                    self.cells[idx] = Cell {
                        ch,
                        fg: (
                            (170.0 + tint.0 as f32 * 0.3) as u8,
                            (185.0 + tint.1 as f32 * 0.4) as u8,
                            (200.0 + tint.2 as f32 * 0.3) as u8,
                        ),
                        bg: (0, tint.1 / 10, tint.2 / 8),
                    };
                } else {
                    let (ch, fg, bg) = if is_crest && v > 0.25 {
                        let foam_v = ((wave - 0.75) * 4.0).min(1.0);
                        let bright = (200.0 + foam_v * 55.0) as u8;
                        ('≈', (bright, bright, bright), hsv_to_rgb(hue % 360.0, 0.5, (v * 0.15).min(0.15)))
                    } else if v > 0.08 {
                        let ci = (v * 4.0).min((WAVE_CHARS.len() - 1) as f32) as usize;
                        let ch = WAVE_CHARS[ci];
                        let fg = hsv_to_rgb(hue % 360.0, 0.7, (v * 1.0).min(0.9));
                        let bg = hsv_to_rgb(hue % 360.0, 0.5, (v * 0.12).min(0.12));
                        (ch, fg, bg)
                    } else {
                        let bg = hsv_to_rgb(200.0, 0.4, (v * 0.06).min(0.06));
                        ('~', hsv_to_rgb(210.0, 0.4, 0.15), bg)
                    };
                    self.cells[idx] = Cell { ch, fg, bg };
                }
            }
        }
    }

    fn cell_at(&self, col: usize, row: usize) -> Cell {
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
