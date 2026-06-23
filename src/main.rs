mod color;
mod effects;
mod pty;
mod renderer;
mod screen;
mod terminal;

use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use std::time::{Duration, Instant};

#[derive(Parser)]
#[command(name = "chromashell", version, about = "Live shader effects for your terminal")]
struct Args {
    /// Effect to run (plasma, fire, matrix, galaxy, metaballs, snow, rain, autumn)
    effect: Option<String>,

    /// Text file to overlay the effect on
    #[arg(short, long)]
    file: Option<String>,

    /// Run in standalone mode (no shell, effect only)
    #[arg(long)]
    standalone: bool,

    /// List all available effects
    #[arg(short, long)]
    list: bool,

    /// Target frames per second
    #[arg(long, default_value = "30")]
    fps: u32,

    /// Auto-quit after N seconds
    #[arg(long)]
    duration: Option<f64>,

    /// Use 24-bit truecolor (default: 256-color for compatibility)
    #[arg(long)]
    truecolor: bool,

    /// Effect density 0.0-1.0 (default: 0.15 for shell, 1.0 for standalone)
    #[arg(short, long)]
    density: Option<f32>,
}

fn run_standalone(args: Args) {
    let effect_name = args.effect.unwrap_or_else(|| {
        use rand::seq::SliceRandom;
        let names = effects::effect_names();
        names.choose(&mut rand::thread_rng()).unwrap().to_string()
    });

    if effects::create_effect(&effect_name, 1, 1).is_none() {
        eprintln!("Unknown effect: {effect_name}");
        eprintln!("Available: {}", effects::effect_names().join(", "));
        std::process::exit(1);
    }

    let input_text = args.file.and_then(|path| {
        std::fs::read_to_string(&path)
            .map_err(|e| eprintln!("Failed to read {path}: {e}"))
            .ok()
    });

    let mut term = terminal::Terminal::new().expect("failed to initialize terminal");
    let (w, h) = term.size();

    let mut effect = effects::create_effect(&effect_name, w as usize, h as usize).unwrap();

    if let Some(d) = args.density {
        effect.set_density(d);
    }

    if let Some(text) = input_text {
        let grid = effects::TextGrid::from_text(&text, w as usize, h as usize);
        effect.set_text(grid);
    }

    let color_mode = if args.truecolor {
        renderer::ColorMode::TrueColor
    } else {
        renderer::ColorMode::Ansi256
    };
    let mut renderer = renderer::Renderer::new(color_mode);
    let frame_time = Duration::from_secs_f64(1.0 / args.fps as f64);
    let start = Instant::now();
    let mut last = Instant::now();

    loop {
        let now = Instant::now();
        let dt = now.duration_since(last).as_secs_f64();
        last = now;

        if let Some(dur) = args.duration {
            if now.duration_since(start).as_secs_f64() >= dur {
                break;
            }
        }

        if term.should_quit().unwrap_or(true) {
            break;
        }

        let _ = term.refresh_size();
        let (cw, ch) = term.size();

        effect.update(dt);
        if renderer.render(effect.as_ref(), cw, ch).is_err() {
            break;
        }

        let elapsed = now.elapsed();
        if elapsed < frame_time {
            std::thread::sleep(frame_time - elapsed);
        }
    }
}

fn run_shell(args: Args) {
    let effect_name = args.effect.unwrap_or("matrix".to_string());

    if effects::create_effect(&effect_name, 1, 1).is_none() {
        eprintln!("Unknown effect: {effect_name}");
        eprintln!("Available: {}", effects::effect_names().join(", "));
        std::process::exit(1);
    }

    let mut term = terminal::Terminal::new().expect("failed to initialize terminal");
    let (w, h) = term.size();

    let mut shell = pty::Pty::spawn_shell(w, h).expect("failed to spawn shell");
    let mut screen = screen::Screen::new(w as usize, h as usize);
    let mut effect = effects::create_effect(&effect_name, w as usize, h as usize).unwrap();

    if let Some(d) = args.density {
        effect.set_density(d);
    }

    let color_mode = if args.truecolor {
        renderer::ColorMode::TrueColor
    } else {
        renderer::ColorMode::Ansi256
    };
    let mut renderer = renderer::Renderer::new(color_mode);
    let frame_time = Duration::from_secs_f64(1.0 / args.fps as f64);
    let mut last = Instant::now();
    let mut pty_buf = [0u8; 8192];

    loop {
        let now = Instant::now();
        let dt = now.duration_since(last).as_secs_f64();
        last = now;

        if !shell.is_alive() {
            break;
        }

        // Read all available shell output
        loop {
            match shell.read(&mut pty_buf) {
                Ok(n) if n > 0 => {
                    screen.feed(&pty_buf[..n]);
                }
                _ => break,
            }
        }

        // Read user input and forward to shell
        while event::poll(Duration::ZERO).unwrap_or(false) {
            match event::read() {
                Ok(Event::Key(key)) => {
                    // Ctrl+\ to quit glowr (standard SIGQUIT key)
                    if key.code == KeyCode::Char('\\') && key.modifiers.contains(KeyModifiers::CONTROL) {
                        return;
                    }

                    let bytes = key_to_bytes(&key);
                    if !bytes.is_empty() {
                        let _ = shell.write_input(&bytes);
                    }
                }
                Ok(Event::Resize(nw, nh)) => {
                    shell.resize(nw, nh);
                    screen.resize(nw as usize, nh as usize);
                    effect.resize(nw as usize, nh as usize);
                }
                _ => {}
            }
        }

        // Update effect with screen content
        let grid = screen_to_grid(&screen);
        effect.set_text(grid);
        effect.update(dt);

        let cursor = screen.cursor_pos();
        let _ = term.refresh_size();
        let (cw, ch) = term.size();
        if renderer.render_with_cursor(effect.as_ref(), cw, ch, cursor).is_err() {
            break;
        }

        let elapsed = now.elapsed();
        if elapsed < frame_time {
            std::thread::sleep(frame_time - elapsed);
        }
    }
}

fn screen_to_grid(screen: &screen::Screen) -> effects::TextGrid {
    let w = screen.width;
    let h = screen.height;
    let mut chars = Vec::with_capacity(h);
    for row in 0..h {
        let mut line = Vec::with_capacity(w);
        for col in 0..w {
            line.push(screen.cell_at(col, row).ch);
        }
        chars.push(line);
    }
    effects::TextGrid { chars, width: w, height: h }
}

fn key_to_bytes(key: &crossterm::event::KeyEvent) -> Vec<u8> {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

    match key.code {
        KeyCode::Char(c) => {
            if ctrl {
                // Ctrl+A = 1, Ctrl+B = 2, etc.
                let byte = (c as u8).wrapping_sub(b'a').wrapping_add(1);
                vec![byte]
            } else {
                let mut buf = [0u8; 4];
                let s = c.encode_utf8(&mut buf);
                s.as_bytes().to_vec()
            }
        }
        KeyCode::Enter => vec![b'\r'],
        KeyCode::Backspace => vec![0x7f],
        KeyCode::Tab => vec![b'\t'],
        KeyCode::Esc => vec![0x1b],
        KeyCode::Up => vec![0x1b, b'[', b'A'],
        KeyCode::Down => vec![0x1b, b'[', b'B'],
        KeyCode::Right => vec![0x1b, b'[', b'C'],
        KeyCode::Left => vec![0x1b, b'[', b'D'],
        KeyCode::Home => vec![0x1b, b'[', b'H'],
        KeyCode::End => vec![0x1b, b'[', b'F'],
        KeyCode::Delete => vec![0x1b, b'[', b'3', b'~'],
        _ => vec![],
    }
}

fn main() {
    let args = Args::parse();

    if args.list {
        println!("Available effects:");
        for name in effects::effect_names() {
            println!("  {name}");
        }
        return;
    }

    if args.standalone {
        run_standalone(args);
    } else {
        run_shell(args);
    }
}
