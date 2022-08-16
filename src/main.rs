mod game;
use game::Game;

use std::io::{self, stdout, Write};

use termion::event::{Event, Key, MouseButton, MouseEvent};
use termion::input::{MouseTerminal, TermRead};
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use termion::{async_stdin, clear, color, cursor, style, AsyncReader};

struct TuiGame<'a> {
    game: Game,
    stdin: Option<&'a mut AsyncReader>,
    stdout: &'a mut dyn Write,
}

const BLACK_COLOR: color::Rgb = color::Rgb(0, 0, 0);
const WHITE_COLOR: color::Rgb = color::Rgb(255, 255, 255);

fn terminal_size() -> (u16, u16) {
    termion::terminal_size().unwrap_or((50, 50))
}

impl<'a> TuiGame<'a> {
    fn new<W: Write>(stdin: &'a mut AsyncReader, stdout: &'a mut W) -> Self {
        let (width, height) = terminal_size();
        let game = Game::new(width, height);
        Self {
            game,
            stdin: Some(stdin),
            stdout,
        }
    }

    fn run(&mut self) -> io::Result<()> {
        writeln!(
            self.stdout,
            "{}{}{}",
            cursor::Hide,
            clear::All,
            cursor::Goto(1, 1)
        )?;
        self.render()?;
        self.stdout.flush()?;
        self.start_loop()?;
        // self.listen_events()?;
        writeln!(self.stdout, "{}{}", style::Reset, cursor::Show)
    }

    fn start_loop(&mut self) -> io::Result<()> {
        let mut playing = false;
        'outer: loop {
            let stdin = self.stdin.take().unwrap();
            for event in stdin.events() {
                let event = event?;
                match event {
                    Event::Key(Key::Char('q')) | Event::Key(Key::Esc) => break 'outer,
                    Event::Mouse(MouseEvent::Press(MouseButton::Left, a, b))
                    | Event::Mouse(MouseEvent::Hold(a, b)) => {
                        let x = a - 1;
                        let y = b - 1;
                        self.revive_cell_at_pos(x, y);
                    }
                    Event::Mouse(MouseEvent::Press(MouseButton::Right, a, b)) if !playing => {
                        let x = a - 1;
                        let y = b - 1;
                        self.kill_cell_at_pos(x, y);
                    }
                    Event::Key(Key::Right) if !playing => {
                        self.tick()?;
                    }
                    Event::Key(Key::Char(' ')) => {
                        playing = !playing;
                    }
                    _ => (),
                }
            }
            self.stdin = Some(stdin);

            if playing {
                self.tick()?;
            }

            self.stdout.flush()?;
            std::thread::sleep(std::time::Duration::from_millis(80));
        }
        Ok(())
    }

    fn tick(&mut self) -> io::Result<()> {
        self.game.tick();
        self.render()
    }

    fn render(&mut self) -> io::Result<()> {
        let (width, height) = terminal_size();
        self.game.resize_if_larger(width, height);
        writeln!(self.stdout, "{}", cursor::Goto(1, 1))?;

        for (cell, (x, y)) in self.game.cells() {
            let color = match cell.is_alive() {
                true => WHITE_COLOR,
                false => BLACK_COLOR,
            };
            write!(
                self.stdout,
                "{}{} ",
                cursor::Goto(x + 1, y + 1),
                color::Bg(color)
            )?;
        }

        Ok(())
    }

    fn revive_cell_at_pos(&mut self, x: u16, y: u16) -> Option<()> {
        self.game.revive_cell_at_pos(x, y)?;

        write!(
            self.stdout,
            "{}{} ",
            cursor::Goto(x + 1, y + 1),
            color::Bg(WHITE_COLOR)
        )
        .unwrap();

        Some(())
    }

    fn kill_cell_at_pos(&mut self, x: u16, y: u16) -> Option<()> {
        self.game.kill_cell_at_pos(x, y)?;

        write!(
            self.stdout,
            "{}{} ",
            cursor::Goto(x + 1, y + 1),
            color::Bg(BLACK_COLOR)
        )
        .unwrap();

        Some(())
    }
}

fn main() -> io::Result<()> {
    let mut stdin = async_stdin();
    let stdout = stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let mut stdout = AlternateScreen::from(stdout);

    let mut game = TuiGame::new(&mut stdin, &mut stdout);
    game.run()
}
