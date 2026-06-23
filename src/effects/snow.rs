use super::{Cell, Effect, TextGrid};
use rand::Rng;

struct Flake {
    x: f32,
    y: f32,
    speed: f32,
    drift: f32,
    ch: char,
    brightness: u8,
}

pub struct Snow {
    width: usize,
    height: usize,
    flakes: Vec<Flake>,
    cells: Vec<Cell>,
    text: Option<TextGrid>,
    density: f32,
    time: f64,
}

const FLAKE_CHARS: &[char] = &['·', '•', '*', '❄', '❅', '❆'];

impl Snow {
    pub fn new(width: usize, height: usize) -> Self {
        let mut s = Self {
            width,
            height,
            flakes: Vec::new(),
            cells: Vec::new(),
            text: None,
            density: -1.0,
            time: 0.0,
        };
        s.resize(width, height);
        s
    }

    fn spawn_flake(rng: &mut impl Rng, w: usize) -> Flake {
        let size = rng.gen_range(0..FLAKE_CHARS.len());
        Flake {
            x: rng.gen_range(0.0..w as f32),
            y: rng.gen_range(-5.0..0.0),
            speed: rng.gen_range(4.0..8.0),
            drift: rng.gen_range(-1.5..1.5),
            ch: FLAKE_CHARS[size],
            brightness: rng.gen_range(140..255),
        }
    }
}

impl Effect for Snow {
    fn name(&self) -> &str {
        "snow"
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
                            fg: (200, 205, 215),
                            bg: (0, 0, 0),
                        };
                    }
                }
            }
        }

        // Spawn new flakes
        let spawn_rate = intensity * 3.2;
        let spawns = (spawn_rate * dt as f32 * w as f32 * 0.12).max(1.0) as usize;
        for _ in 0..spawns {
            self.flakes.push(Self::spawn_flake(&mut rng, w));
        }

        // Update and render flakes
        let dt_f = dt as f32;
        let t = self.time as f32;
        self.flakes.retain_mut(|f| {
            f.y += f.speed * dt_f;
            f.x += f.drift * dt_f + (t * 2.0 + f.x * 0.5).sin() * 0.3 * dt_f;

            if f.y >= h as f32 {
                return false;
            }

            let col = f.x as i32;
            let row = f.y as i32;
            if col >= 0 && col < w as i32 && row >= 0 && row < h as i32 {
                let col = col as usize;
                let row = row as usize;
                let idx = row * w + col;
                let is_text = self.text.as_ref().map_or(false, |g| g.has_char(col, row));

                if is_text {
                    let base = self.cells[idx].fg;
                    self.cells[idx].fg = (
                        base.0.saturating_add(20),
                        base.1.saturating_add(20),
                        base.2.saturating_add(25),
                    );
                } else {
                    let b = f.brightness;
                    self.cells[idx] = Cell {
                        ch: f.ch,
                        fg: (b, b, b),
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
        self.flakes.clear();
    }

    fn set_text(&mut self, grid: TextGrid) {
        self.text = Some(grid);
    }

    fn set_density(&mut self, density: f32) {
        self.density = density.clamp(0.0, 1.0);
    }
}
