use super::{Cell, Effect, TextGrid};
use rand::Rng;

struct Firefly {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    phase: f32,
    freq: f32,
    brightness: f32,
}

pub struct Fireflies {
    width: usize,
    height: usize,
    flies: Vec<Firefly>,
    cells: Vec<Cell>,
    text: Option<TextGrid>,
    density: f32,
    time: f64,
}

impl Fireflies {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            flies: Vec::new(),
            cells: Vec::new(),
            text: None,
            density: -1.0,
            time: 0.0,
        }
    }

    fn spawn_flies(rng: &mut impl Rng, count: usize, w: usize, h: usize) -> Vec<Firefly> {
        if w == 0 || h == 0 {
            return Vec::new();
        }
        (0..count)
            .map(|_| Firefly {
                x: rng.gen_range(0.0..w as f32),
                y: rng.gen_range(0.0..h as f32),
                vx: rng.gen_range(-2.0..2.0),
                vy: rng.gen_range(-1.5..1.5),
                phase: rng.gen_range(0.0..6.28),
                freq: rng.gen_range(0.5..2.0),
                brightness: rng.gen_range(0.3..1.0),
            })
            .collect()
    }
}

impl Effect for Fireflies {
    fn name(&self) -> &str {
        "fireflies"
    }

    fn update(&mut self, dt: f64) {
        self.time += dt;
        let w = self.width;
        let h = self.height;
        if w == 0 || h == 0 {
            return;
        }

        let t = self.time as f32;
        let dt_f = dt as f32;

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
                            fg: (180, 185, 175),
                            bg: (0, 0, 0),
                        };
                    }
                }
            }
        }

        // Update and render fireflies
        let wf = w as f32;
        let hf = h as f32;
        for fly in &mut self.flies {
            // Gentle wandering
            fly.vx += (t * 0.7 + fly.phase).sin() * 1.5 * dt_f;
            fly.vy += (t * 0.5 + fly.phase + 1.0).cos() * 1.0 * dt_f;
            fly.vx *= 0.98;
            fly.vy *= 0.98;
            fly.x += fly.vx * dt_f;
            fly.y += fly.vy * dt_f;

            // Wrap around
            if fly.x < 0.0 { fly.x += wf; }
            if fly.x >= wf { fly.x -= wf; }
            if fly.y < 0.0 { fly.y += hf; }
            if fly.y >= hf { fly.y -= hf; }

            // Pulsing glow
            let pulse = ((t * fly.freq + fly.phase).sin() * 0.5 + 0.5) * fly.brightness;

            if pulse < 0.1 {
                continue;
            }

            let col = fly.x as usize;
            let row = fly.y as usize;
            if col >= w || row >= h {
                continue;
            }

            let idx = row * w + col;
            let is_text = self.text.as_ref().map_or(false, |g| g.has_char(col, row));

            if is_text {
                let boost = (pulse * 60.0) as u8;
                let base = self.cells[idx].fg;
                self.cells[idx].fg = (
                    base.0.saturating_add(boost),
                    base.1.saturating_add(boost / 2),
                    base.2,
                );
            } else {
                let v = (pulse * 255.0) as u8;
                let ch = if pulse > 0.7 { '*' }
                    else if pulse > 0.4 { '·' }
                    else { '.' };
                self.cells[idx] = Cell {
                    ch,
                    fg: (v, (v as f32 * 0.85) as u8, (v as f32 * 0.3) as u8),
                    bg: (0, 0, 0),
                };

                // Soft glow around bright fireflies
                if pulse > 0.5 {
                    let glow = (pulse * 40.0) as u8;
                    for &(dx, dy) in &[(-1i32, 0i32), (1, 0), (0, -1), (0, 1)] {
                        let gc = col as i32 + dx;
                        let gr = row as i32 + dy;
                        if gc >= 0 && gc < w as i32 && gr >= 0 && gr < h as i32 {
                            let gi = gr as usize * w + gc as usize;
                            let is_gt = self.text.as_ref().map_or(false, |g| g.has_char(gc as usize, gr as usize));
                            if is_gt {
                                let base = self.cells[gi].fg;
                                self.cells[gi].fg = (
                                    base.0.saturating_add(glow / 2),
                                    base.1.saturating_add(glow / 3),
                                    base.2,
                                );
                            } else if self.cells[gi].ch == ' ' {
                                self.cells[gi] = Cell {
                                    ch: '.',
                                    fg: (glow, (glow as f32 * 0.7) as u8, (glow as f32 * 0.2) as u8),
                                    bg: (0, 0, 0),
                                };
                            }
                        }
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

        let has_text = self.text.is_some();
        let count = if self.density >= 0.0 {
            (200.0 * self.density) as usize
        } else if has_text {
            80
        } else {
            150
        };

        let mut rng = rand::thread_rng();
        self.flies = Self::spawn_flies(&mut rng, count, width, height);
    }

    fn set_text(&mut self, grid: TextGrid) {
        self.text = Some(grid);
    }

    fn set_density(&mut self, density: f32) {
        self.density = density.clamp(0.0, 1.0);
    }
}
