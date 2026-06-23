use super::{Cell, Effect, TextGrid};
use rand::Rng;

struct Drop {
    x: usize,
    y: f32,
    speed: f32,
    length: usize,
    brightness: u8,
}

pub struct Rain {
    width: usize,
    height: usize,
    drops: Vec<Drop>,
    cells: Vec<Cell>,
    text: Option<TextGrid>,
    density: f32,
    time: f64,
}

impl Rain {
    pub fn new(width: usize, height: usize) -> Self {
        let mut r = Self {
            width,
            height,
            drops: Vec::new(),
            cells: Vec::new(),
            text: None,
            density: -1.0,
            time: 0.0,
        };
        r.resize(width, height);
        r
    }

    fn spawn_drop(rng: &mut impl Rng, w: usize) -> Drop {
        Drop {
            x: rng.gen_range(0..w),
            y: rng.gen_range(-8.0..0.0),
            speed: rng.gen_range(25.0..50.0),
            length: rng.gen_range(2..6),
            brightness: rng.gen_range(60..140),
        }
    }
}

impl Effect for Rain {
    fn name(&self) -> &str {
        "rain"
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

        // Draw text
        if let Some(ref grid) = self.text {
            for row in 0..h {
                for col in 0..w {
                    if grid.has_char(col, row) {
                        let idx = row * w + col;
                        self.cells[idx] = Cell {
                            ch: grid.char_at(col, row),
                            fg: (160, 175, 195),
                            bg: (0, 0, 0),
                        };
                    }
                }
            }
        }

        // Spawn
        let spawn_rate = intensity * 5.0;
        let spawns = (spawn_rate * dt as f32 * w as f32 * 0.2).max(2.0) as usize;
        for _ in 0..spawns {
            self.drops.push(Self::spawn_drop(&mut rng, w));
        }

        // Update and render
        let dt_f = dt as f32;
        self.drops.retain_mut(|d| {
            d.y += d.speed * dt_f;

            let head_row = d.y as i32;
            if head_row - d.length as i32 >= h as i32 {
                return false;
            }

            for i in 0..d.length {
                let row = head_row - i as i32;
                if row >= 0 && row < h as i32 {
                    let row = row as usize;
                    let col = d.x;
                    let idx = row * w + col;

                    let is_text = self.text.as_ref().map_or(false, |g| g.has_char(col, row));

                    if is_text {
                        let wetness = if i == 0 { 30u8 } else { 15 };
                        let base = self.cells[idx].fg;
                        self.cells[idx].fg = (
                            base.0.saturating_sub(wetness / 2),
                            base.1.saturating_sub(wetness / 4),
                            base.2.saturating_add(wetness),
                        );
                    } else {
                        let fade = 1.0 - i as f32 / d.length as f32;
                        let b = (d.brightness as f32 * fade) as u8;
                        let ch = if i == 0 { '|' }
                            else if i == 1 { '│' }
                            else { '¦' };
                        self.cells[idx] = Cell {
                            ch,
                            fg: (b / 3, b / 2, b),
                            bg: (0, 0, 0),
                        };
                    }
                }
            }

            // Splash at bottom
            if head_row >= h as i32 - 1 && head_row < h as i32 + 2 {
                let splash_row = h - 1;
                let is_text = self.text.as_ref().map_or(false, |g| g.has_char(d.x, splash_row));
                if !is_text {
                    let idx = splash_row * w + d.x;
                    self.cells[idx] = Cell {
                        ch: '~',
                        fg: (d.brightness / 3, d.brightness / 2, d.brightness),
                        bg: (0, 0, 0),
                    };
                    if d.x > 0 {
                        let li = splash_row * w + d.x - 1;
                        let is_t = self.text.as_ref().map_or(false, |g| g.has_char(d.x - 1, splash_row));
                        if !is_t && self.cells[li].ch == ' ' {
                            self.cells[li] = Cell {
                                ch: '·',
                                fg: (d.brightness / 5, d.brightness / 4, d.brightness / 2),
                                bg: (0, 0, 0),
                            };
                        }
                    }
                    if d.x + 1 < w {
                        let ri = splash_row * w + d.x + 1;
                        let is_t = self.text.as_ref().map_or(false, |g| g.has_char(d.x + 1, splash_row));
                        if !is_t && self.cells[ri].ch == ' ' {
                            self.cells[ri] = Cell {
                                ch: '·',
                                fg: (d.brightness / 5, d.brightness / 4, d.brightness / 2),
                                bg: (0, 0, 0),
                            };
                        }
                    }
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
        self.drops.clear();
    }

    fn set_text(&mut self, grid: TextGrid) {
        self.text = Some(grid);
    }

    fn set_density(&mut self, density: f32) {
        self.density = density.clamp(0.0, 1.0);
    }
}
