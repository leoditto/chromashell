use super::{Cell, Effect, TextGrid};
use rand::Rng;

struct Bolt {
    segments: Vec<(usize, usize)>,
    age: f32,
    lifetime: f32,
    brightness: f32,
}

pub struct Lightning {
    width: usize,
    height: usize,
    bolts: Vec<Bolt>,
    flash: f32,
    cells: Vec<Cell>,
    text: Option<TextGrid>,
    density: f32,
    time: f64,
    next_bolt: f64,
}

impl Lightning {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            bolts: Vec::new(),
            flash: 0.0,
            cells: Vec::new(),
            text: None,
            density: -1.0,
            time: 0.0,
            next_bolt: 0.5,
        }
    }

    fn create_bolt(rng: &mut impl Rng, w: usize, h: usize) -> Bolt {
        let mut segments = Vec::new();
        let mut x = rng.gen_range(w / 4..3 * w / 4);
        let mut y = 0usize;

        while y < h {
            segments.push((x, y));
            y += 1;
            // Jagged horizontal drift
            let drift: i32 = rng.gen_range(-2..3);
            x = (x as i32 + drift).max(0).min(w as i32 - 1) as usize;

            // Chance to fork/branch
            if rng.gen_bool(0.08) && y < h - 3 {
                let branch_len = rng.gen_range(3..8).min(h - y);
                let mut bx = x;
                let dir: i32 = if rng.gen_bool(0.5) { 1 } else { -1 };
                for by in y..y + branch_len {
                    bx = (bx as i32 + dir + rng.gen_range(-1..2)).max(0).min(w as i32 - 1) as usize;
                    segments.push((bx, by));
                }
            }

            // Chance to stop early
            if y > h / 2 && rng.gen_bool(0.05) {
                break;
            }
        }

        Bolt {
            segments,
            age: 0.0,
            lifetime: rng.gen_range(0.15..0.4),
            brightness: rng.gen_range(0.7..1.0),
        }
    }
}

impl Effect for Lightning {
    fn name(&self) -> &str {
        "lightning"
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

        let dt_f = dt as f32;

        // Spawn new bolts
        if self.time >= self.next_bolt {
            let count = rng.gen_range(1..=3);
            for _ in 0..count {
                self.bolts.push(Self::create_bolt(&mut rng, w, h));
            }
            self.flash = 0.3 + (count - 1) as f32 * 0.1;
            self.next_bolt = self.time + rng.gen_range(0.5..2.0) / intensity as f64;
        }

        // Decay flash
        self.flash = (self.flash - dt_f * 2.0).max(0.0);

        // Clear with slight flash tint
        let flash_bg = (self.flash * 15.0) as u8;
        for cell in &mut self.cells {
            *cell = Cell {
                ch: ' ',
                fg: (0, 0, 0),
                bg: (flash_bg / 3, flash_bg / 3, flash_bg),
            };
        }

        // Draw text
        if let Some(ref grid) = self.text {
            for row in 0..h {
                for col in 0..w {
                    if grid.has_char(col, row) {
                        let idx = row * w + col;
                        let flash_boost = (self.flash * 80.0) as u8;
                        self.cells[idx] = Cell {
                            ch: grid.char_at(col, row),
                            fg: (
                                (175u8).saturating_add(flash_boost),
                                (180u8).saturating_add(flash_boost),
                                (195u8).saturating_add(flash_boost),
                            ),
                            bg: (flash_bg / 3, flash_bg / 3, flash_bg),
                        };
                    }
                }
            }
        }

        // Update and render bolts
        self.bolts.retain_mut(|bolt| {
            bolt.age += dt_f;
            if bolt.age >= bolt.lifetime {
                return false;
            }

            let fade = 1.0 - (bolt.age / bolt.lifetime);
            let b = (bolt.brightness * fade * 255.0) as u8;

            for &(bx, by) in &bolt.segments {
                if bx < w && by < h {
                    let idx = by * w + bx;
                    let is_text = self.text.as_ref().map_or(false, |g| g.has_char(bx, by));

                    if is_text {
                        self.cells[idx].fg = (
                            self.cells[idx].fg.0.max(b),
                            self.cells[idx].fg.1.max(b),
                            self.cells[idx].fg.2.max(b),
                        );
                    } else {
                        let ch = if fade > 0.7 { '█' }
                            else if fade > 0.4 { '▓' }
                            else if fade > 0.2 { '│' }
                            else { '¦' };
                        self.cells[idx] = Cell {
                            ch,
                            fg: (b, b, b),
                            bg: (b / 8, b / 8, b / 4),
                        };
                    }

                    // Glow around bolt
                    if fade > 0.3 {
                        let glow = (b as f32 * 0.3) as u8;
                        for &(dx, _) in &[(-1i32, 0i32), (1, 0)] {
                            let gc = bx as i32 + dx;
                            if gc >= 0 && gc < w as i32 {
                                let gi = by * w + gc as usize;
                                let is_gt = self.text.as_ref().map_or(false, |g| g.has_char(gc as usize, by));
                                if is_gt {
                                    self.cells[gi].fg = (
                                        self.cells[gi].fg.0.saturating_add(glow / 2),
                                        self.cells[gi].fg.1.saturating_add(glow / 2),
                                        self.cells[gi].fg.2.saturating_add(glow),
                                    );
                                } else if self.cells[gi].ch == ' ' {
                                    self.cells[gi] = Cell {
                                        ch: '·',
                                        fg: (glow / 2, glow / 2, glow),
                                        bg: self.cells[gi].bg,
                                    };
                                }
                            }
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
        self.bolts.clear();
    }

    fn set_text(&mut self, grid: TextGrid) {
        self.text = Some(grid);
    }

    fn set_density(&mut self, density: f32) {
        self.density = density.clamp(0.0, 1.0);
    }
}
