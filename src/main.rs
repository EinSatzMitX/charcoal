use std::env::{self};

use color_eyre::eyre::{Ok, Result};

mod image;

use image::Image;
use ratatui::{
    DefaultTerminal,
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent},
    layout::{Constraint::*, Layout, Rect},
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
}

#[derive(Debug)]
struct App {
    mode: Mode,
    command_buffer: String,
    image: Image,
    terminal: DefaultTerminal,
}

impl App {
    fn new(image_path: String, term: DefaultTerminal) -> Self {
        Self {
            mode: Mode::Normal,
            command_buffer: String::new(),
            image: Image::new(image_path.clone()),
            terminal: term,
        }
    }

    fn run(mut self) -> Result<()> {
        loop {
            // prepare state outside of the closure to avoid multiple mutable borrows
            let cmd_line = self.command_buffer.clone();
            let file_path = self.image.path.clone();
            let img_widget = &mut self.image;

            // Draw UI using only these locals inside the closure
            self.terminal.draw(|f| {
                let size = f.area();
                let [top, main] = Layout::vertical([Length(1), Min(0)]).areas(size);
                let [title, _] = Layout::horizontal([Min(0), Length(8)]).areas(top);

                // Status line
                Text::from(format!("File: {}", file_path))
                    .centered()
                    .render(title, f.buffer_mut());

                // Image area
                f.render_widget(img_widget, main);

                // Command line
                Text::from(format!(":{}", cmd_line))
                    .left_aligned()
                    .render(title, f.buffer_mut());
            })?;

            // Input Handling
            if let Event::Key(key) = event::read()? {
                if self.mode == Mode::Normal
                    && (key.code == KeyCode::Char('q') || key.code == KeyCode::Esc)
                {
                    break;
                }
                self.handle_input(key);
            }
        }
        Ok(())
    }

    fn handle_input(&mut self, key: KeyEvent) {
        match self.mode {
            Mode::Normal => self.handle_normal_key_input(key),
            Mode::Command => self.handle_command_key_input(key),
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
                // TODO: execute command
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

        Text::from(format!(":{}", self.command_buffer))
            .left_aligned()
            .render(title, buf);
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
