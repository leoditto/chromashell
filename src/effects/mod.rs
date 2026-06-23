pub mod aurora;
pub mod autumn;
pub mod fire;
pub mod fireflies;
pub mod lightning;
pub mod matrix;
pub mod metaballs;
pub mod ocean;
pub mod plasma;
pub mod rain;
pub mod retro;
pub mod snow;
pub mod starfield;

#[derive(Clone, Copy)]
pub struct Cell {
    pub ch: char,
    pub fg: (u8, u8, u8),
    pub bg: (u8, u8, u8),
}

impl Cell {
    pub fn blank() -> Self {
        Self {
            ch: ' ',
            fg: (0, 0, 0),
            bg: (0, 0, 0),
        }
    }
}

#[derive(Clone)]
pub struct TextGrid {
    pub chars: Vec<Vec<char>>,
    pub width: usize,
    pub height: usize,
}

impl TextGrid {
    pub fn from_text(text: &str, width: usize, height: usize) -> Self {
        let mut chars = Vec::with_capacity(height);
        let lines: Vec<&str> = text.lines().collect();

        for row in 0..height {
            let mut line_chars = Vec::with_capacity(width);
            let line = lines.get(row).unwrap_or(&"");
            let line_chars_iter: Vec<char> = line.chars().collect();
            for col in 0..width {
                line_chars.push(
                    line_chars_iter.get(col).copied().unwrap_or(' ')
                );
            }
            chars.push(line_chars);
        }

        Self { chars, width, height }
    }

    pub fn char_at(&self, col: usize, row: usize) -> char {
        if row < self.chars.len() && col < self.chars[row].len() {
            self.chars[row][col]
        } else {
            ' '
        }
    }

    pub fn has_char(&self, col: usize, row: usize) -> bool {
        self.char_at(col, row) != ' '
    }
}

pub trait Effect {
    fn name(&self) -> &str;
    fn update(&mut self, dt: f64);
    fn cell_at(&self, col: usize, row: usize) -> Cell;
    fn resize(&mut self, width: usize, height: usize);
    fn set_text(&mut self, grid: TextGrid);
    fn set_density(&mut self, density: f32);
}

pub fn effect_names() -> Vec<&'static str> {
    vec!["plasma", "fire", "matrix", "galaxy", "metaballs", "snow", "rain", "autumn", "aurora", "fireflies", "ocean", "lightning", "retro"]
}

pub fn create_effect(name: &str, width: usize, height: usize) -> Option<Box<dyn Effect>> {
    let mut effect: Box<dyn Effect> = match name {
        "plasma" => Box::new(plasma::Plasma::new()),
        "fire" => Box::new(fire::Fire::new(width, height)),
        "matrix" => Box::new(matrix::Matrix::new(width, height)),
        "galaxy" => Box::new(starfield::Starfield::new(width, height)),
        "metaballs" => Box::new(metaballs::Metaballs::new()),
        "snow" => Box::new(snow::Snow::new(width, height)),
        "rain" => Box::new(rain::Rain::new(width, height)),
        "autumn" => Box::new(autumn::Autumn::new(width, height)),
        "aurora" => Box::new(aurora::Aurora::new()),
        "fireflies" => Box::new(fireflies::Fireflies::new()),
        "ocean" => Box::new(ocean::Ocean::new()),
        "lightning" => Box::new(lightning::Lightning::new()),
        "retro" => Box::new(retro::Retro::new()),
        _ => return None,
    };
    effect.resize(width, height);
    Some(effect)
}
