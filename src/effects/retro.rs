use super::{Cell, Effect, TextGrid};
use rand::Rng;

pub struct Retro {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
    text: Option<TextGrid>,
    density: f32,
    time: f64,
    glitch_row: Option<usize>,
    glitch_timer: f32,
    static_band_y: f32,
    static_band_speed: f32,
}

impl Retro {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            cells: Vec::new(),
            text: None,
            density: -1.0,
            time: 0.0,
            glitch_row: None,
            glitch_timer: 0.0,
            static_band_y: -5.0,
            static_band_speed: 0.0,
        }
    }
}

const STATIC_CHARS: &[char] = &['░', '▒', '▓', '█', '#', '%', '&'];

impl Effect for Retro {
    fn name(&self) -> &str {
        "retro"
    }

    fn update(&mut self, dt: f64) {
        let mut rng = rand::thread_rng();
        self.time += dt;
        let w = self.width;
        let h = self.height;
        if w == 0 || h == 0 {
            return;
        }

        let dt_f = dt as f32;
        let has_text = self.text.is_some();
        let intensity = if self.density >= 0.0 {
            self.density
        } else if has_text {
            0.6
        } else {
            1.0
        };

        // Update glitch timer
        self.glitch_timer -= dt_f;
        if self.glitch_timer <= 0.0 {
            if rng.gen_bool(0.3 * intensity as f64) {
                self.glitch_row = Some(rng.gen_range(0..h));
                self.glitch_timer = rng.gen_range(0.05..0.15);
            } else {
                self.glitch_row = None;
                self.glitch_timer = rng.gen_range(0.1..0.5);
            }
        }

        // Update static band
        self.static_band_y += self.static_band_speed * dt_f;
        if self.static_band_y > h as f32 + 5.0 || self.static_band_speed == 0.0 {
            if rng.gen_bool(0.025 * intensity as f64) {
                self.static_band_y = -3.0;
                self.static_band_speed = rng.gen_range(15.0..40.0);
            }
        }

        for row in 0..h {
            // Scanline darkening (every other row)
            let scanline_dim = if row % 2 == 0 { 1.0f32 } else { 0.75 };

            // CRT curvature — slight color shift at edges
            let y_norm = (row as f32 / h as f32 - 0.5).abs();
            let edge_dim = 1.0 - y_norm * y_norm * 0.3;

            // Static band proximity
            let band_dist = (row as f32 - self.static_band_y).abs();
            let in_band = band_dist < 3.0;

            // Glitch row
            let is_glitch = self.glitch_row.map_or(false, |gr| {
                row >= gr && row < gr + 2
            });
            let glitch_offset: i32 = if is_glitch {
                rng.gen_range(-8..8)
            } else {
                0
            };

            for col in 0..w {
                let idx = row * w + col;

                // Source column with glitch offset
                let src_col = (col as i32 + glitch_offset).max(0).min(w as i32 - 1) as usize;

                let is_text = self.text.as_ref().map_or(false, |g| g.has_char(src_col, row));

                if in_band && intensity > 0.3 {
                    // Static band
                    let static_v = rng.gen_range(0.0..0.5) * intensity;
                    let ci = rng.gen_range(0..STATIC_CHARS.len());
                    let v = (static_v * 180.0) as u8;
                    self.cells[idx] = Cell {
                        ch: STATIC_CHARS[ci],
                        fg: (v, v, v),
                        bg: (v / 6, v / 6, v / 6),
                    };
                } else if is_text {
                    let ch = self.text.as_ref().unwrap().char_at(src_col, row);
                    // Phosphor green tint
                    let base_r = (140.0 * scanline_dim * edge_dim) as u8;
                    let base_g = (220.0 * scanline_dim * edge_dim) as u8;
                    let base_b = (140.0 * scanline_dim * edge_dim) as u8;

                    // Chromatic aberration on glitch
                    let (r, g, b) = if is_glitch {
                        (base_r.saturating_add(40), base_g, base_b.saturating_sub(20))
                    } else {
                        (base_r, base_g, base_b)
                    };

                    self.cells[idx] = Cell {
                        ch,
                        fg: (r, g, b),
                        bg: (0, (3.0 * scanline_dim) as u8, 0),
                    };
                } else {
                    // Empty space with faint scanlines
                    let scan_v = if row % 2 == 0 { 0u8 } else { 2 };
                    let bg_g = ((scan_v as f32 * intensity) as u8).min(4);

                    // Occasional random pixel noise
                    if rng.gen_bool(0.0025 * intensity as f64) {
                        let v = rng.gen_range(20..60);
                        self.cells[idx] = Cell {
                            ch: '·',
                            fg: (v / 2, v, v / 2),
                            bg: (0, bg_g, 0),
                        };
                    } else {
                        self.cells[idx] = Cell {
                            ch: ' ',
                            fg: (0, 0, 0),
                            bg: (0, bg_g, 0),
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
    }

    fn set_text(&mut self, grid: TextGrid) {
        self.text = Some(grid);
    }

    fn set_density(&mut self, density: f32) {
        self.density = density.clamp(0.0, 1.0);
    }
}
