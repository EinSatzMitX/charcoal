use std::{
    env::{self},
    time::{Duration, Instant},
};

use color_eyre::eyre::{Ok, Result};

mod image;

use image::Image;
use ratatui::{
    DefaultTerminal,
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent},
    layout::{
        Constraint::{self, *},
        Direction, Layout, Rect,
    },
    text::Text,
    widgets::Widget,
};

struct CliArgs {
    path: String,
}

/* Mode definitions for a vim-like interface */
#[derive(Debug, Default, PartialEq)]
enum Mode {
    #[default]
    Normal,
    Command,
    Quit,
}

#[derive(Debug)]
struct App {
    mode: Mode,
    command_buffer: String,
    image: Image,
    terminal: DefaultTerminal,

    footer_msg: Option<String>,
    foot_msg_expires: Option<Instant>,
}

impl App {
    fn new(image_path: String, term: DefaultTerminal) -> Self {
        Self {
            mode: Mode::Normal,
            command_buffer: String::new(),
            image: Image::new(image_path.clone()),
            terminal: term,
            footer_msg: None,
            foot_msg_expires: None,
        }
    }

    fn run(mut self) -> Result<()> {
        let tick_rate = Duration::from_millis(100);
        while self.mode != Mode::Quit {
            /* Clear footer message if ti expired */
            if let Some(expiry) = self.foot_msg_expires {
                if Instant::now() >= expiry {
                    self.footer_msg = None;
                    self.foot_msg_expires = None;
                }
            }

            // Draw UI using only these locals inside the closure
            self.terminal.draw(|f| {
                let area = f.area();

                // split into three chunks
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(1), // status bar
                        Constraint::Min(0),    // main image
                        Constraint::Length(1), // command bar
                    ])
                    .split(area);

                // pick them out by index
                let status_line = chunks[0];
                let main = chunks[1];
                let cmd_line = chunks[2];

                // 1) Status
                Text::from(format!("File: {}", self.image.path))
                    .centered()
                    .render(status_line, f.buffer_mut());

                // 2) Image
                self.image.render(main, f.buffer_mut());

                // 3) Command line
                let text = if let Some(msg) = &self.footer_msg {
                    msg.clone()
                } else if !self.command_buffer.is_empty() {
                    format!(":{}", self.command_buffer)
                } else {
                    "Press ':' to enter command mode, 'q' to quit".into()
                };
                Text::from(text)
                    .left_aligned()
                    .render(cmd_line, f.buffer_mut());
            })?;

            // Input Handling
            if event::poll(tick_rate)? {
                if let Event::Key(key) = event::read()? {
                    // quit
                    if self.mode == Mode::Normal
                        && (key.code == KeyCode::Char('q') || key.code == KeyCode::Esc)
                    {
                        break;
                    }
                    self.handle_input(key);
                }
            }
        }
        Ok(())
    }

    fn handle_input(&mut self, key: KeyEvent) {
        match self.mode {
            Mode::Normal => self.handle_normal_key_input(key),
            Mode::Command => self.handle_command_key_input(key),
            Mode::Quit => {
                /* The Program should quit now */
                std::process::exit(0);
            }
        }
    }

    fn handle_normal_key_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char(':') => {
                self.mode = Mode::Command;
                self.command_buffer.clear();
            }
            KeyCode::Esc => {
                self.mode = Mode::Normal;
            }

            _ => {}
        }
    }

    fn handle_command_key_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Backspace => {
                self.command_buffer.pop();
            }
            KeyCode::Enter => {
                let cmd = self.command_buffer.clone();
                // TODO: execute Add commands

                self.run_command(cmd);

                self.mode = Mode::Normal;
                self.command_buffer.clear();
            }
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.command_buffer.clear();
            }
            KeyCode::Char(c) => {
                self.command_buffer.push(c);
            }
            _ => {}
        };
    }

    fn run_command(&mut self, cmd: String) {
        match cmd.trim().to_lowercase().as_str() {
            "q" | "quit" => {
                self.mode = Mode::Quit;
            }
            "help" => {
                self.display_message(
                    "Commands:\n\
                    q, quit          - exit the program\n\
                    help             - show this message\n\
                    ..."
                    .to_string(),
                );
            }
            _ => {
                self.display_message(format!("Unknown command: {}", cmd));
            }
        }
    }

    fn display_message(&mut self, msg: String) {
        let now = Instant::now();
        self.footer_msg = Some(msg);
        self.foot_msg_expires = Some(now + Duration::from_secs(3));
    }
}

fn main() -> Result<()> {
    println!("Running ratatui application!");
    color_eyre::install()?;

    let args = parse_args();

    let terminal = ratatui::init();
    let app = App::new(args.path, terminal);
    let result = app.run();

    /* Call ratatui's restore function just for safety */
    ratatui::restore();
    result
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [top, main] = Layout::vertical([Length(1), Min(0)]).areas(area);
        let [title, _] = Layout::horizontal([Min(0), Length(8)]).areas(top);
        Text::from(format!("File: {}", self.image.path.clone(),))
            .centered()
            .render(title, buf);
        /* Function, which actually writes stuff to the given buffer */
        self.image.render(main, buf);

        Text::from(format!("\n Command Buffer{}", self.command_buffer.clone()))
            .left_aligned()
            .render(title, buf);

        if !self.command_buffer.is_empty() {
            Text::from(format!(":{}", self.command_buffer))
                .left_aligned()
                .render(title, buf);
        } else {
            Text::from(format!("Press ':' to enable command mode or q to exit!"))
                .left_aligned()
                .render(title, buf);
        }
    }
}

fn parse_args() -> CliArgs {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() != 1 {
        print_usage();
        std::process::exit(1);
    }

    CliArgs {
        path: args[0].clone(),
    }
}

fn print_usage() {
    println!("Usage: terminal-image-viewer path/to/file.png");
}
