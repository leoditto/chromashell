use super::{Cell, Effect, TextGrid};
use crate::color::hsv_to_rgb;

pub struct Aurora {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
    text: Option<TextGrid>,
    density: f32,
    time: f64,
}

impl Aurora {
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

impl Effect for Aurora {
    fn name(&self) -> &str {
        "aurora"
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
            0.85
        } else {
            1.0
        };

        for row in 0..h {
            let y = row as f32 / h as f32;
            for col in 0..w {
                let x = col as f32 / w as f32;
                let idx = row * w + col;

                // Multiple overlapping curtains
                let c1 = ((x * 3.0 + t * 0.4).sin() * 0.5 + 0.5)
                    * ((y * 2.0 - t * 0.2).cos() * 0.5 + 0.5);
                let c2 = ((x * 5.0 - t * 0.3 + 1.0).sin() * 0.5 + 0.5)
                    * ((y * 3.0 + t * 0.15).cos() * 0.5 + 0.5);
                let c3 = ((x * 2.0 + t * 0.5 + 2.5).sin() * 0.5 + 0.5)
                    * ((y * 1.5 - t * 0.25 + 1.0).cos() * 0.5 + 0.5);

                // Spread across more of the screen
                let vert_fade = (1.0 - y * 0.8).max(0.0);
                let vert_fade = vert_fade.sqrt();

                let v = (c1 * 0.4 + c2 * 0.35 + c3 * 0.25) * vert_fade * intensity;

                // Shift between green, blue, purple
                let hue = 120.0 + c1 * 80.0 + c2 * 60.0 + (t * 8.0) % 360.0 * 0.1;

                let is_text = self.text.as_ref().map_or(false, |g| g.has_char(col, row));

                if is_text {
                    let ch = self.text.as_ref().unwrap().char_at(col, row);
                    if v > 0.03 {
                        let tint = hsv_to_rgb(hue % 360.0, 0.5 * v, 0.35 * v);
                        self.cells[idx] = Cell {
                            ch,
                            fg: (
                                (180.0 + tint.0 as f32 * 0.4) as u8,
                                (190.0 + tint.1 as f32 * 0.4) as u8,
                                (195.0 + tint.2 as f32 * 0.3) as u8,
                            ),
                            bg: (tint.0 / 8, tint.1 / 8, tint.2 / 8),
                        };
                    } else {
                        self.cells[idx] = Cell {
                            ch,
                            fg: (185, 190, 200),
                            bg: (0, 0, 0),
                        };
                    }
                } else if v > 0.02 {
                    let sat = 0.7 + v * 0.3;
                    let val = (v * 1.2).min(1.0);
                    let fg = hsv_to_rgb(hue % 360.0, sat, val);
                    let bg = hsv_to_rgb(hue % 360.0, sat * 0.5, (val * 0.15).min(0.15));
                    let ch = if v > 0.3 { '▒' }
                        else if v > 0.15 { '░' }
                        else if v > 0.06 { '·' }
                        else { ' ' };
                    self.cells[idx] = Cell { ch, fg, bg };
                } else {
                    self.cells[idx] = Cell {
                        ch: ' ',
                        fg: (0, 0, 0),
                        bg: (0, 0, 0),
                    };
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
