use crate::effects::Effect;
use std::io::{self, Write};

#[derive(Clone, Copy, PartialEq)]
pub enum ColorMode {
    TrueColor,
    Ansi256,
}

pub struct Renderer {
    buf: Vec<u8>,
    color_mode: ColorMode,
}

impl Renderer {
    pub fn new(color_mode: ColorMode) -> Self {
        Self {
            buf: Vec::with_capacity(1024 * 64),
            color_mode,
        }
    }

    fn write_fg(&mut self, r: u8, g: u8, b: u8) {
        match self.color_mode {
            ColorMode::TrueColor => {
                write!(self.buf, "\x1b[38;2;{r};{g};{b}m").unwrap();
            }
            ColorMode::Ansi256 => {
                let idx = crate::color::rgb_to_256(r, g, b);
                write!(self.buf, "\x1b[38;5;{idx}m").unwrap();
            }
        }
    }

    fn write_bg(&mut self, r: u8, g: u8, b: u8) {
        match self.color_mode {
            ColorMode::TrueColor => {
                write!(self.buf, "\x1b[48;2;{r};{g};{b}m").unwrap();
            }
            ColorMode::Ansi256 => {
                let idx = crate::color::rgb_to_256(r, g, b);
                write!(self.buf, "\x1b[48;5;{idx}m").unwrap();
            }
        }
    }

    pub fn render(&mut self, effect: &dyn Effect, width: u16, height: u16) -> io::Result<()> {
        self.render_inner(effect, width, height, None)
    }

    pub fn render_with_cursor(&mut self, effect: &dyn Effect, width: u16, height: u16, cursor: (usize, usize)) -> io::Result<()> {
        self.render_inner(effect, width, height, Some(cursor))
    }

    fn render_inner(&mut self, effect: &dyn Effect, width: u16, height: u16, cursor: Option<(usize, usize)>) -> io::Result<()> {
        self.buf.clear();
        self.buf.extend_from_slice(b"\x1b[H");

        let mut prev_fg = (255u8, 255u8, 255u8);
        let mut prev_bg = (255u8, 255u8, 255u8);

        for row in 0..height as usize {
            for col in 0..width as usize {
                let cell = effect.cell_at(col, row);

                let is_cursor = cursor.map_or(false, |(cx, cy)| col == cx && row == cy);

                let (fg, bg) = if is_cursor {
                    // Invert colors at cursor
                    ((0, 0, 0), (200, 200, 200))
                } else {
                    (cell.fg, cell.bg)
                };

                if bg != prev_bg {
                    self.write_bg(bg.0, bg.1, bg.2);
                    prev_bg = bg;
                }
                if fg != prev_fg {
                    self.write_fg(fg.0, fg.1, fg.2);
                    prev_fg = fg;
                }

                let mut char_buf = [0u8; 4];
                let s = cell.ch.encode_utf8(&mut char_buf);
                self.buf.extend_from_slice(s.as_bytes());
            }

            if row + 1 < height as usize {
                self.buf.extend_from_slice(b"\x1b[0m\r\n");
                prev_fg = (255, 255, 255);
                prev_bg = (255, 255, 255);
            }
        }

        self.buf.extend_from_slice(b"\x1b[0m");

        let mut stdout = io::stdout().lock();
        stdout.write_all(&self.buf)?;
        stdout.flush()
    }
}
