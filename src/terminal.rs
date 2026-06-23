use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    terminal::{self, ClearType},
    ExecutableCommand,
};
use std::io;
use std::time::Duration;

pub struct Terminal {
    width: u16,
    height: u16,
}

impl Terminal {
    pub fn new() -> io::Result<Self> {
        let (width, height) = terminal::size()?;
        let mut stdout = io::stdout();
        terminal::enable_raw_mode()?;
        stdout.execute(terminal::EnterAlternateScreen)?;
        stdout.execute(cursor::Hide)?;
        stdout.execute(terminal::Clear(ClearType::All))?;
        Ok(Self { width, height })
    }

    pub fn size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    pub fn refresh_size(&mut self) -> io::Result<()> {
        let (w, h) = terminal::size()?;
        self.width = w;
        self.height = h;
        Ok(())
    }

    pub fn should_quit(&mut self) -> io::Result<bool> {
        while event::poll(Duration::ZERO)? {
            match event::read()? {
                Event::Key(KeyEvent { code, .. }) => {
                    if matches!(code, KeyCode::Char('q') | KeyCode::Esc) {
                        return Ok(true);
                    }
                }
                Event::Resize(w, h) => {
                    self.width = w;
                    self.height = h;
                }
                _ => {}
            }
        }
        Ok(false)
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        let mut stdout = io::stdout();
        let _ = stdout.execute(cursor::Show);
        let _ = stdout.execute(terminal::LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}
