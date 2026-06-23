use super::{Cell, Effect, TextGrid};
use rand::Rng;

const MATRIX_CHARS: &str = "ｱｲｳｴｵｶｷｸｹｺｻｼｽｾｿﾀﾁﾂﾃﾅﾆﾇﾈﾉﾊﾋﾌﾍﾎﾏﾐﾑﾒﾓﾔﾕﾖﾗﾘﾙﾚﾛﾜﾝ0123456789";

struct Drop {
    col: usize,
    head: f32,
    speed: f32,
    length: usize,
    trail: Vec<char>,
}

pub struct Matrix {
    width: usize,
    height: usize,
    drops: Vec<Drop>,
    cells: Vec<Cell>,
    char_pool: Vec<char>,
    text: Option<TextGrid>,
    time: f64,
    density: f32,
}

impl Matrix {
    pub fn new(width: usize, height: usize) -> Self {
        let char_pool: Vec<char> = MATRIX_CHARS.chars().collect();
        let mut m = Self {
            width,
            height,
            drops: Vec::new(),
            cells: Vec::new(),
            char_pool,
            text: None,
            time: 0.0,
            density: -1.0, // -1 = use default
        };
        m.resize(width, height);
        m
    }

    fn rebuild_drops(&mut self) {
        let mut rng = rand::thread_rng();
        self.drops.clear();
        let w = self.width;
        let h = self.height;
        if w == 0 || h == 0 {
            return;
        }

        let d = if self.density < 0.0 {
            if self.text.is_some() { 0.15 } else { 1.0 }
        } else {
            self.density as f64
        };

        for col in 0..w {
            if d >= 1.0 {
                let count = rng.gen_range(1..=3);
                for _ in 0..count {
                    self.drops.push(Self::spawn_drop(&mut rng, col, h, &self.char_pool));
                }
            } else if rng.gen_bool(d) {
                self.drops.push(Self::spawn_drop(&mut rng, col, h, &self.char_pool));
            }
        }
    }

    fn spawn_drop(rng: &mut impl Rng, col: usize, height: usize, char_pool: &[char]) -> Drop {
        let length = rng.gen_range(4..height.max(5).min(30));
        let trail: Vec<char> = (0..length)
            .map(|_| char_pool[rng.gen_range(0..char_pool.len())])
            .collect();
        Drop {
            col,
            head: rng.gen_range(-(height as f32)..0.0),
            speed: rng.gen_range(8.0..22.0),
            length,
            trail,
        }
    }
}

impl Effect for Matrix {
    fn name(&self) -> &str {
        "matrix"
    }

    fn update(&mut self, dt: f64) {
        self.time += dt;
        let mut rng = rand::thread_rng();
        let w = self.width;
        let h = self.height;
        if w == 0 || h == 0 {
            return;
        }

        for cell in &mut self.cells {
            *cell = Cell::blank();
        }

        for drop in &mut self.drops {
            drop.head += drop.speed * dt as f32;

            if rng.gen_bool(0.08) {
                let idx = rng.gen_range(0..drop.trail.len());
                drop.trail[idx] = self.char_pool[rng.gen_range(0..self.char_pool.len())];
            }
        }

        let char_pool = &self.char_pool;
        for drop in &mut self.drops {
            if (drop.head as i32 - drop.length as i32) > h as i32 {
                let col = drop.col;
                *drop = Self::spawn_drop(&mut rng, col, h, char_pool);
            }
        }

        let has_text = self.text.is_some();

        for drop in &self.drops {
            let head_row = drop.head as i32;

            for (i, &trail_ch) in drop.trail.iter().enumerate() {
                let row = head_row - i as i32;
                if row < 0 || row >= h as i32 {
                    continue;
                }
                let row = row as usize;
                let col = drop.col;
                let fade = 1.0 - (i as f32 / drop.length as f32);

                let is_text_cell = self.text.as_ref().map_or(false, |g| g.has_char(col, row));
                let ch = if is_text_cell {
                    self.text.as_ref().unwrap().char_at(col, row)
                } else {
                    trail_ch
                };

                let fg = if is_text_cell {
                    if i == 0 {
                        (200, 255, 200)
                    } else if i < 4 {
                        let g = (160.0 + fade * 60.0) as u8;
                        (g / 2, g, g / 2)
                    } else {
                        let g = (fade * 160.0) as u8;
                        (g / 3, g, g / 3)
                    }
                } else if has_text {
                    if i == 0 {
                        (60, 140, 60)
                    } else {
                        let g = (fade * fade * 70.0) as u8;
                        (0, g, 0)
                    }
                } else {
                    if i == 0 {
                        (150, 255, 150)
                    } else {
                        let g = (fade * fade * 120.0) as u8;
                        (0, g, 0)
                    }
                };

                let idx = row * w + col;
                if idx < self.cells.len() && fg.1 > self.cells[idx].fg.1 {
                    self.cells[idx] = Cell {
                        ch,
                        fg,
                        bg: (0, 0, 0),
                    };
                }
            }
        }

        // Show text that isn't currently hit by rain
        if let Some(grid) = &self.text {
            for row in 0..h {
                for col in 0..w {
                    let idx = row * w + col;
                    if grid.has_char(col, row) && self.cells[idx].fg.1 < 80 {
                        self.cells[idx] = Cell {
                            ch: grid.char_at(col, row),
                            fg: (190, 200, 190),
                            bg: (0, 0, 0),
                        };
                    }
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

        self.rebuild_drops();
    }

    fn set_text(&mut self, grid: TextGrid) {
        self.text = Some(grid);
    }

    fn set_density(&mut self, density: f32) {
        self.density = density.clamp(0.0, 1.0);
        self.rebuild_drops();
    }
}
