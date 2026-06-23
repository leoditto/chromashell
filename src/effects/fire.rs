use super::{Cell, Effect, TextGrid};
use rand::Rng;

const FIRE_CHARS: &[char] = &[' ', '.', ':', '*', 's', 'S', '#', '$', '&', '@'];

fn idx(w: usize, x: usize, y: usize) -> usize {
    y * w + x
}

pub struct Fire {
    width: usize,
    height: usize,
    heat: Vec<f32>,
    cells: Vec<Cell>,
    text: Option<TextGrid>,
    density: f32,
}

impl Fire {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            heat: vec![0.0; width * height],
            cells: vec![Cell::blank(); width * height],
            text: None,
            density: -1.0,
        }
    }

    fn heat_to_fg(&self, v: f32) -> (u8, u8, u8) {
        if v < 0.2 {
            let t = v / 0.2;
            ((t * 80.0) as u8, 0, 0)
        } else if v < 0.5 {
            let t = (v - 0.2) / 0.3;
            (80 + (t * 175.0) as u8, (t * 40.0) as u8, 0)
        } else if v < 0.8 {
            let t = (v - 0.5) / 0.3;
            (255, 40 + (t * 200.0) as u8, (t * 30.0) as u8)
        } else {
            let t = (v - 0.8) / 0.2;
            (255, 240 + (t * 15.0) as u8, 30 + (t * 200.0) as u8)
        }
    }
}

impl Effect for Fire {
    fn name(&self) -> &str {
        "fire"
    }

    fn update(&mut self, _dt: f64) {
        let mut rng = rand::thread_rng();
        let w = self.width;
        let h = self.height;
        if w == 0 || h == 0 {
            return;
        }

        let has_text = self.text.is_some();
        let intensity = if self.density >= 0.0 {
            self.density
        } else if has_text {
            0.7
        } else {
            1.0
        };

        // Ignite bottom row
        for x in 0..w {
            self.heat[idx(w, x, h - 1)] = if rng.gen_bool(0.7) {
                rng.gen_range(0.7..1.0) * intensity
            } else {
                rng.gen_range(0.2..0.5) * intensity
            };
        }

        // Propagate upward
        for y in 0..h - 1 {
            for x in 0..w {
                let x_left = if x > 0 { x - 1 } else { x };
                let x_right = if x + 1 < w { x + 1 } else { x };

                let below = self.heat[idx(w, x, y + 1)];
                let below_left = self.heat[idx(w, x_left, y + 1)];
                let below_right = self.heat[idx(w, x_right, y + 1)];
                let below2 = if y + 2 < h {
                    self.heat[idx(w, x, y + 2)]
                } else {
                    below
                };

                let avg = (below + below_left + below_right + below2) * 0.25;
                let decay = 2.2 / h as f32;
                self.heat[idx(w, x, y)] = (avg - decay).max(0.0);
            }
        }

        // Convert heat to cells
        for y in 0..h {
            for x in 0..w {
                let v = self.heat[idx(w, x, y)];
                let is_text = self.text.as_ref().map_or(false, |g| g.has_char(x, y));

                if is_text {
                    let ch = self.text.as_ref().unwrap().char_at(x, y);
                    let fg = if v > 0.3 {
                        // Text lit by fire
                        self.heat_to_fg(v)
                    } else if v > 0.05 {
                        // Warm glow on text
                        let t = v / 0.3;
                        let r = (190.0 + t * 65.0) as u8;
                        let g = (180.0 + t * 40.0) as u8;
                        let b = (170.0 - t * 100.0) as u8;
                        (r, g, b)
                    } else {
                        // Idle text
                        (200, 195, 185)
                    };
                    let bg = if v > 0.2 {
                        ((v * 40.0) as u8, 0, 0)
                    } else {
                        (0, 0, 0)
                    };
                    self.cells[idx(w, x, y)] = Cell { ch, fg, bg };
                } else {
                    let ch = if has_text {
                        // Shell mode: use subtle chars
                        if v < 0.05 { ' ' }
                        else if v < 0.15 { '.' }
                        else if v < 0.3 { ':' }
                        else if v < 0.5 { '*' }
                        else { '#' }
                    } else {
                        let ci = (v * (FIRE_CHARS.len() - 1) as f32) as usize;
                        FIRE_CHARS[ci.min(FIRE_CHARS.len() - 1)]
                    };

                    let fg = self.heat_to_fg(v);

                    let bg_v = (v * 0.1).min(0.1);
                    let bg = ((bg_v * 150.0) as u8, 0, 0);

                    self.cells[idx(w, x, y)] = Cell { ch, fg, bg };
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
        self.heat = vec![0.0; width * height];
        self.cells = vec![Cell::blank(); width * height];
    }

    fn set_text(&mut self, grid: TextGrid) {
        self.text = Some(grid);
    }

    fn set_density(&mut self, density: f32) {
        self.density = density.clamp(0.0, 1.0);
    }
}
