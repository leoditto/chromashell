use super::{Cell, Effect, TextGrid};
use crate::color::hsv_to_rgb;
use rand::Rng;

struct Star {
    x: f32,
    y: f32,
    z: f32,
    hue: f32,
}

pub struct Starfield {
    width: usize,
    height: usize,
    stars: Vec<Star>,
    cells: Vec<Cell>,
    text: Option<TextGrid>,
    density: f32,
}

impl Starfield {
    pub fn new(width: usize, height: usize) -> Self {
        let mut sf = Self {
            width,
            height,
            stars: Vec::new(),
            cells: Vec::new(),
            text: None,
            density: -1.0,
        };
        sf.resize(width, height);
        sf
    }

    fn spawn_star(rng: &mut impl Rng) -> Star {
        Star {
            x: rng.gen_range(-1.5..1.5),
            y: rng.gen_range(-1.5..1.5),
            z: rng.gen_range(0.01..1.0),
            hue: rng.gen_range(200.0..280.0),
        }
    }
}

impl Effect for Starfield {
    fn name(&self) -> &str {
        "galaxy"
    }

    fn update(&mut self, dt: f64) {
        let mut rng = rand::thread_rng();
        let speed = dt as f32 * 0.15;
        let w = self.width;
        let h = self.height;
        if w == 0 || h == 0 {
            return;
        }

        // Clear cells
        for cell in &mut self.cells {
            *cell = Cell { ch: ' ', fg: (0, 0, 0), bg: (0, 0, 2) };
        }

        // Draw text first
        if let Some(ref grid) = self.text {
            for row in 0..h {
                for col in 0..w {
                    if grid.has_char(col, row) {
                        let idx = row * w + col;
                        self.cells[idx] = Cell {
                            ch: grid.char_at(col, row),
                            fg: (190, 195, 210),
                            bg: (0, 0, 2),
                        };
                    }
                }
            }
        }

        // Render stars
        for star in &mut self.stars {
            star.z -= speed;
            if star.z <= 0.001 {
                *star = Self::spawn_star(&mut rng);
            }

            let sx = (star.x / star.z + 0.5) * w as f32;
            let sy = (star.y / star.z + 0.5) * h as f32;

            let col = sx as i32;
            let row = sy as i32;

            if col >= 0 && col < w as i32 && row >= 0 && row < h as i32 {
                let col = col as usize;
                let row = row as usize;
                let idx = row * w + col;

                let is_text = self.text.as_ref().map_or(false, |g| g.has_char(col, row));

                if is_text {
                    let brightness = (1.0 - star.z).powf(0.7);
                    let boost = (brightness * 50.0) as u8;
                    let base = self.cells[idx].fg;
                    self.cells[idx].fg = (
                        base.0.saturating_add(boost),
                        base.1.saturating_add(boost),
                        base.2.saturating_add(boost),
                    );
                } else {
                    let brightness = (1.0 - star.z).powf(0.7);
                    let ch = if star.z < 0.15 { '*' }
                        else if star.z < 0.5 { '*' }
                        else if star.z < 0.75 { '·' }
                        else { '.' };

                    let v = (brightness * 255.0) as u8;
                    let tint = hsv_to_rgb(star.hue, 0.3, brightness);
                    let fg = (
                        (v as u16 / 2 + tint.0 as u16 / 2).min(255) as u8,
                        (v as u16 / 2 + tint.1 as u16 / 2).min(255) as u8,
                        (v as u16 + tint.2 as u16 / 2).min(255) as u8,
                    );

                    self.cells[idx].ch = ch;
                    self.cells[idx].fg = fg;
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

        let star_count = if self.density >= 0.0 {
            (500.0 * self.density) as usize
        } else if self.text.is_some() {
            250
        } else {
            500
        };

        let mut rng = rand::thread_rng();
        self.stars = (0..star_count).map(|_| Self::spawn_star(&mut rng)).collect();
    }

    fn set_text(&mut self, grid: TextGrid) {
        self.text = Some(grid);
    }

    fn set_density(&mut self, density: f32) {
        self.density = density.clamp(0.0, 1.0);
    }
}
