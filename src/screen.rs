#[derive(Clone, Copy)]
pub struct ScreenCell {
    pub ch: char,
    pub fg: (u8, u8, u8),
}

impl ScreenCell {
    fn blank() -> Self {
        Self {
            ch: ' ',
            fg: (200, 200, 200),
        }
    }
}

pub struct Screen {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<ScreenCell>,
    cursor_x: usize,
    cursor_y: usize,
    // ANSI parser state
    esc_buf: String,
    in_esc: bool,
    in_csi: bool,
    current_fg: (u8, u8, u8),
}

impl Screen {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![ScreenCell::blank(); width * height],
            cursor_x: 0,
            cursor_y: 0,
            esc_buf: String::new(),
            in_esc: false,
            in_csi: false,
            current_fg: (200, 200, 200),
        }
    }

    pub fn cursor_pos(&self) -> (usize, usize) {
        (self.cursor_x, self.cursor_y)
    }

    pub fn cell_at(&self, col: usize, row: usize) -> ScreenCell {
        if col < self.width && row < self.height {
            self.cells[row * self.width + col]
        } else {
            ScreenCell::blank()
        }
    }

    pub fn feed(&mut self, data: &[u8]) {
        for &byte in data {
            self.process_byte(byte as char);
        }
    }

    fn process_byte(&mut self, ch: char) {
        if self.in_esc {
            if self.in_csi {
                self.esc_buf.push(ch);
                // CSI sequence ends with a letter (0x40-0x7E)
                if ch.is_ascii_alphabetic() || ch == '@' || ch == '`' {
                    self.handle_csi();
                    self.in_esc = false;
                    self.in_csi = false;
                    self.esc_buf.clear();
                }
            } else if ch == '[' {
                self.in_csi = true;
                self.esc_buf.clear();
            } else if ch == ']' {
                // OSC sequence - skip until BEL or ST
                self.in_esc = false;
                // We'll just drop OSC sequences
            } else {
                self.in_esc = false;
            }
            return;
        }

        match ch {
            '\x1b' => {
                self.in_esc = true;
                self.in_csi = false;
            }
            '\n' => {
                self.cursor_y += 1;
                if self.cursor_y >= self.height {
                    self.scroll_up();
                    self.cursor_y = self.height - 1;
                }
            }
            '\r' => {
                self.cursor_x = 0;
            }
            '\x08' => {
                // Backspace
                if self.cursor_x > 0 {
                    self.cursor_x -= 1;
                }
            }
            '\t' => {
                self.cursor_x = (self.cursor_x + 8) & !7;
                if self.cursor_x >= self.width {
                    self.cursor_x = self.width - 1;
                }
            }
            '\x07' => {} // BEL - ignore
            _ if ch >= ' ' => {
                if self.cursor_x < self.width && self.cursor_y < self.height {
                    let idx = self.cursor_y * self.width + self.cursor_x;
                    self.cells[idx] = ScreenCell {
                        ch,
                        fg: self.current_fg,
                    };
                    self.cursor_x += 1;
                    if self.cursor_x >= self.width {
                        self.cursor_x = 0;
                        self.cursor_y += 1;
                        if self.cursor_y >= self.height {
                            self.scroll_up();
                            self.cursor_y = self.height - 1;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_csi(&mut self) {
        let seq = &self.esc_buf;
        let last = seq.chars().last().unwrap_or(' ');
        let params_str: String = seq.chars().take_while(|c| !c.is_ascii_alphabetic() && *c != '@' && *c != '`').collect();
        let params: Vec<usize> = params_str
            .split(';')
            .filter(|s| !s.is_empty())
            .map(|s| s.parse().unwrap_or(0))
            .collect();

        match last {
            'H' | 'f' => {
                // Cursor position
                let row = params.first().copied().unwrap_or(1).saturating_sub(1);
                let col = params.get(1).copied().unwrap_or(1).saturating_sub(1);
                self.cursor_y = row.min(self.height - 1);
                self.cursor_x = col.min(self.width - 1);
            }
            'A' => {
                let n = params.first().copied().unwrap_or(1).max(1);
                self.cursor_y = self.cursor_y.saturating_sub(n);
            }
            'B' => {
                let n = params.first().copied().unwrap_or(1).max(1);
                self.cursor_y = (self.cursor_y + n).min(self.height - 1);
            }
            'C' => {
                let n = params.first().copied().unwrap_or(1).max(1);
                self.cursor_x = (self.cursor_x + n).min(self.width - 1);
            }
            'D' => {
                let n = params.first().copied().unwrap_or(1).max(1);
                self.cursor_x = self.cursor_x.saturating_sub(n);
            }
            'J' => {
                let mode = params.first().copied().unwrap_or(0);
                match mode {
                    0 => {
                        // Clear from cursor to end
                        let start = self.cursor_y * self.width + self.cursor_x;
                        for i in start..self.cells.len() {
                            self.cells[i] = ScreenCell::blank();
                        }
                    }
                    1 => {
                        // Clear from start to cursor
                        let end = self.cursor_y * self.width + self.cursor_x;
                        for i in 0..=end.min(self.cells.len() - 1) {
                            self.cells[i] = ScreenCell::blank();
                        }
                    }
                    2 | 3 => {
                        // Clear entire screen
                        for cell in &mut self.cells {
                            *cell = ScreenCell::blank();
                        }
                    }
                    _ => {}
                }
            }
            'K' => {
                let mode = params.first().copied().unwrap_or(0);
                let row_start = self.cursor_y * self.width;
                match mode {
                    0 => {
                        for x in self.cursor_x..self.width {
                            self.cells[row_start + x] = ScreenCell::blank();
                        }
                    }
                    1 => {
                        for x in 0..=self.cursor_x.min(self.width - 1) {
                            self.cells[row_start + x] = ScreenCell::blank();
                        }
                    }
                    2 => {
                        for x in 0..self.width {
                            self.cells[row_start + x] = ScreenCell::blank();
                        }
                    }
                    _ => {}
                }
            }
            'm' => {
                self.handle_sgr(&params);
            }
            _ => {}
        }
    }

    fn handle_sgr(&mut self, params: &[usize]) {
        if params.is_empty() {
            self.current_fg = (200, 200, 200);
            return;
        }

        let mut i = 0;
        while i < params.len() {
            match params[i] {
                0 => self.current_fg = (200, 200, 200),
                1 => {} // Bold - we could brighten
                30 => self.current_fg = (0, 0, 0),
                31 => self.current_fg = (205, 49, 49),
                32 => self.current_fg = (13, 188, 121),
                33 => self.current_fg = (229, 229, 16),
                34 => self.current_fg = (36, 114, 200),
                35 => self.current_fg = (188, 63, 188),
                36 => self.current_fg = (17, 168, 205),
                37 => self.current_fg = (200, 200, 200),
                39 => self.current_fg = (200, 200, 200),
                90 => self.current_fg = (100, 100, 100),
                91 => self.current_fg = (255, 80, 80),
                92 => self.current_fg = (80, 255, 80),
                93 => self.current_fg = (255, 255, 80),
                94 => self.current_fg = (80, 80, 255),
                95 => self.current_fg = (255, 80, 255),
                96 => self.current_fg = (80, 255, 255),
                97 => self.current_fg = (255, 255, 255),
                38 => {
                    // Extended fg color
                    if i + 1 < params.len() && params[i + 1] == 5 && i + 2 < params.len() {
                        // 256 color - just store as gray for now
                        i += 2;
                    } else if i + 1 < params.len() && params[i + 1] == 2 && i + 4 < params.len() {
                        self.current_fg = (params[i + 2] as u8, params[i + 3] as u8, params[i + 4] as u8);
                        i += 4;
                    }
                }
                _ => {} // Ignore bg colors, other attrs
            }
            i += 1;
        }
    }

    fn scroll_up(&mut self) {
        let w = self.width;
        self.cells.drain(0..w);
        self.cells.extend(std::iter::repeat(ScreenCell::blank()).take(w));
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.cells = vec![ScreenCell::blank(); width * height];
        self.cursor_x = 0;
        self.cursor_y = 0;
    }
}
