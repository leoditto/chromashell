use crate::color::hsv_to_rgb;
use super::{Cell, Effect, TextGrid};

const DENSITY_CHARS: &[char] = &[' ', '·', ':', '░', '▒', '▓', '█'];

struct Ball {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    radius: f32,
    hue: f32,
}

pub struct Metaballs {
    balls: Vec<Ball>,
    time: f64,
    width: usize,
    height: usize,
    cells: Vec<Cell>,
    text: Option<TextGrid>,
    density: f32,
}

impl Metaballs {
    pub fn new() -> Self {
        Self {
            balls: vec![
                Ball { x: 0.3, y: 0.4, vx: 0.15, vy: 0.12, radius: 0.12, hue: 280.0 },
                Ball { x: 0.6, y: 0.3, vx: -0.1, vy: 0.18, radius: 0.14, hue: 320.0 },
                Ball { x: 0.5, y: 0.7, vx: 0.12, vy: -0.14, radius: 0.13, hue: 200.0 },
                Ball { x: 0.4, y: 0.5, vx: -0.18, vy: -0.1, radius: 0.1, hue: 160.0 },
                Ball { x: 0.7, y: 0.6, vx: 0.08, vy: 0.2, radius: 0.15, hue: 40.0 },
            ],
            time: 0.0,
            width: 0,
            height: 0,
            cells: Vec::new(),
            text: None,
            density: -1.0,
        }
    }
}

impl Effect for Metaballs {
    fn name(&self) -> &str {
        "metaballs"
    }

    fn update(&mut self, dt: f64) {
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
            0.4
        } else {
            1.0
        };

        for ball in &mut self.balls {
            ball.x += ball.vx * dt as f32;
            ball.y += ball.vy * dt as f32;
            if ball.x < 0.0 || ball.x > 1.0 {
                ball.vx = -ball.vx;
                ball.x = ball.x.clamp(0.0, 1.0);
            }
            if ball.y < 0.0 || ball.y > 1.0 {
                ball.vy = -ball.vy;
                ball.y = ball.y.clamp(0.0, 1.0);
            }
        }

        let aspect = w as f32 / h as f32 * 0.5;

        for row in 0..h {
            let y = row as f32 / h as f32;
            for col in 0..w {
                let x = col as f32 / w as f32;

                let mut total = 0.0f32;
                let mut weighted_hue = 0.0f32;

                for ball in &self.balls {
                    let dx = (x - ball.x) * aspect;
                    let dy = y - ball.y;
                    let dist_sq = dx * dx + dy * dy;
                    let influence = ball.radius * ball.radius / dist_sq.max(0.0001);
                    total += influence;
                    weighted_hue += ball.hue * influence;
                }

                let hue = (weighted_hue / total.max(0.001) + self.time as f32 * 20.0) % 360.0;
                let is_text = self.text.as_ref().map_or(false, |g| g.has_char(col, row));

                let (ch, fg, bg) = if is_text {
                    let ch = self.text.as_ref().unwrap().char_at(col, row);
                    if total > 0.8 {
                        // Text inside a blob: colored
                        let sat = (0.3 * intensity).min(0.6);
                        let val = 0.75 + (total - 0.8).min(0.3) * 0.25;
                        let fg = hsv_to_rgb(hue, sat, val);
                        let bg_v = ((total - 0.8) * 0.05 * intensity).min(0.05);
                        let bg = hsv_to_rgb(hue, 0.4, bg_v);
                        (ch, fg, bg)
                    } else {
                        // Text outside blobs: near-white
                        let glow = total * 0.1 * intensity;
                        let tint = hsv_to_rgb(hue, glow, 0.0);
                        (ch, (190 + tint.0 / 4, 195 + tint.1 / 8, 200), (0, 0, 0))
                    }
                } else if has_text {
                    // Background: very subtle blob outlines
                    if total > 1.0 {
                        let edge = ((total - 1.0) * intensity).min(0.5);
                        let ci = (edge * 3.0) as usize;
                        let ch = DENSITY_CHARS[ci.min(3)];
                        let fg = hsv_to_rgb(hue, 0.5, edge * 0.5);
                        (ch, fg, (0, 0, 0))
                    } else if total > 0.5 {
                        let glow = (total - 0.5) * intensity * 0.3;
                        ('·', hsv_to_rgb(hue, 0.4, glow), (0, 0, 0))
                    } else {
                        (' ', (0, 0, 0), (0, 0, 0))
                    }
                } else {
                    // Standalone mode
                    let ci = if total > 1.0 {
                        let edge = ((total - 1.0) * 3.0).min((DENSITY_CHARS.len() - 1) as f32);
                        edge as usize
                    } else {
                        let glow = (total * 2.0).min(2.0);
                        (glow as usize).min(2)
                    };
                    let ch = DENSITY_CHARS[ci.min(DENSITY_CHARS.len() - 1)];

                    if total > 1.0 {
                        let edge = (total - 1.0).min(1.0);
                        let val = 0.4 + edge * 0.6;
                        (ch, hsv_to_rgb(hue, 0.8, val), hsv_to_rgb(hue, 0.6, (edge * 0.15).min(0.15)))
                    } else {
                        let glow_v = total * 0.15;
                        (ch, hsv_to_rgb(hue, 0.5, glow_v), (0, 0, 0))
                    }
                };

                self.cells[row * w + col] = Cell { ch, fg, bg };
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
