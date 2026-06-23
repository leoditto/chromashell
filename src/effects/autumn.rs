use super::{Cell, Effect, TextGrid};
use crate::color::hsv_to_rgb;
use rand::Rng;

struct Leaf {
    x: f32,
    y: f32,
    speed: f32,
    drift_amp: f32,
    drift_freq: f32,
    phase: f32,
    hue: f32,
    ch: char,
}

pub struct Autumn {
    width: usize,
    height: usize,
    leaves: Vec<Leaf>,
    cells: Vec<Cell>,
    text: Option<TextGrid>,
    density: f32,
    time: f64,
}

const LEAF_ASCII: &[char] = &[',', '~', '%', '&', '@'];

impl Autumn {
    pub fn new(width: usize, height: usize) -> Self {
        let mut a = Self {
            width,
            height,
            leaves: Vec::new(),
            cells: Vec::new(),
            text: None,
            density: -1.0,
            time: 0.0,
        };
        a.resize(width, height);
        a
    }

    fn spawn_leaf(rng: &mut impl Rng, w: usize) -> Leaf {
        let hue = rng.gen_range(10.0..45.0);
        Leaf {
            x: rng.gen_range(0.0..w as f32),
            y: rng.gen_range(-3.0..0.0),
            speed: rng.gen_range(3.0..5.5),
            drift_amp: rng.gen_range(2.0..6.0),
            drift_freq: rng.gen_range(0.8..2.5),
            phase: rng.gen_range(0.0..6.28),
            hue,
            ch: LEAF_ASCII[rng.gen_range(0..LEAF_ASCII.len())],
        }
    }
}

impl Effect for Autumn {
    fn name(&self) -> &str {
        "autumn"
    }

    fn update(&mut self, dt: f64) {
        let mut rng = rand::thread_rng();
        self.time += dt;
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

        // Clear
        for cell in &mut self.cells {
            *cell = Cell { ch: ' ', fg: (0, 0, 0), bg: (0, 0, 0) };
        }

        // Draw text with warm tint
        if let Some(ref grid) = self.text {
            for row in 0..h {
                for col in 0..w {
                    if grid.has_char(col, row) {
                        let idx = row * w + col;
                        self.cells[idx] = Cell {
                            ch: grid.char_at(col, row),
                            fg: (210, 195, 170),
                            bg: (0, 0, 0),
                        };
                    }
                }
            }
        }

        // Spawn leaves
        let spawn_rate = intensity * 3.2;
        let spawns = (spawn_rate * dt as f32 * w as f32 * 0.1).max(1.0) as usize;
        for _ in 0..spawns {
            self.leaves.push(Self::spawn_leaf(&mut rng, w));
        }

        // Update and render leaves
        let dt_f = dt as f32;
        let t = self.time as f32;
        self.leaves.retain_mut(|leaf| {
            leaf.y += leaf.speed * dt_f;
            leaf.x += (t * leaf.drift_freq + leaf.phase).sin() * leaf.drift_amp * dt_f;

            if leaf.y >= h as f32 {
                return false;
            }

            let col = leaf.x as i32;
            let row = leaf.y as i32;
            if col >= 0 && col < w as i32 && row >= 0 && row < h as i32 {
                let col = col as usize;
                let row = row as usize;
                let idx = row * w + col;

                let is_text = self.text.as_ref().map_or(false, |g| g.has_char(col, row));

                if is_text {
                    let base = self.cells[idx].fg;
                    let tint = hsv_to_rgb(leaf.hue, 0.4, 0.15);
                    self.cells[idx].fg = (
                        base.0.saturating_add(tint.0 / 2),
                        base.1.saturating_add(tint.1 / 3),
                        base.2.saturating_sub(10),
                    );
                } else {
                    let fg = hsv_to_rgb(leaf.hue, 0.8, 0.7 + (t + leaf.phase).sin() * 0.15);
                    self.cells[idx] = Cell {
                        ch: leaf.ch,
                        fg,
                        bg: (0, 0, 0),
                    };
                }
            }
            true
        });
    }

    fn cell_at(&self, col: usize, row: usize) -> Cell {
        self.cells[row * self.width + col]
    }

    fn resize(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.cells = vec![Cell::blank(); width * height];
        self.leaves.clear();
    }

    fn set_text(&mut self, grid: TextGrid) {
        self.text = Some(grid);
    }

    fn set_density(&mut self, density: f32) {
        self.density = density.clamp(0.0, 1.0);
    }
}
