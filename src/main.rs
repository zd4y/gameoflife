mod game;
use game::Game;

use std::io::{stdout, Write};
use std::time::Duration;

use tokio::time::{self, Instant};
use tokio_stream::StreamExt;

use crossterm::{
    cursor,
    event::{
        self, Event, EventStream, KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEvent,
        MouseEventKind,
    },
    execute, queue,
    style::{self, Stylize},
    terminal, Result,
};

const DEFAULT_FPS: f32 = 6.0;

struct TuiGame<'a, W: Write> {
    game: Game,
    writer: &'a mut W,
}

fn terminal_size() -> (u16, u16) {
    terminal::size().unwrap_or((50, 30))
}

impl<'a, W: Write> TuiGame<'a, W> {
    fn new(writer: &'a mut W) -> Self {
        let (width, height) = terminal_size();
        let game = Game::new(width, height);
        Self { game, writer }
    }

    async fn run(&mut self) -> Result<()> {
        execute!(
            self.writer,
            cursor::Hide,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0)
        )?;
        self.render()?;
        self.run_loop().await?;
        execute!(self.writer, style::ResetColor, cursor::Show)
    }

    async fn run_loop(&mut self) -> Result<()> {
        let mut fps = DEFAULT_FPS;
        let mut playing = false;
        let mut reader = EventStream::new();
        let mut interval = self.interval(fps);

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if playing {
                        self.tick()?;
                    }
                }
                maybe_event = reader.next() => {
                    match maybe_event {
                        Some(Ok(event)) => match event {
                            Event::Mouse(MouseEvent { kind: MouseEventKind::Down(button) | MouseEventKind::Drag(button), column, row, modifiers: _ }) => match button {
                                MouseButton::Left=>{
                                    self.revive_cell_at_pos(column,row);
                                },
                                MouseButton::Right=> {
                                    self.kill_cell_at_pos(column,row);
                                },
                                MouseButton::Middle => ()
                            },
                            Event::Key(KeyEvent { code, modifiers: _, kind: KeyEventKind::Press, state: _ }) => match code {
                                KeyCode::Esc | KeyCode::Char('q') => break,
                                KeyCode::Right => {
                                    self.tick()?;
                                },
                                KeyCode::Up => {
                                    fps *= 1.2;
                                    interval = self.interval(fps);
                                },
                                KeyCode::Char('r') => {
                                    fps = DEFAULT_FPS;
                                    interval = self.interval(fps);
                                }
                                KeyCode::Down => {
                                    fps /= 1.2;
                                    interval = self.interval(fps);
                                }
                                KeyCode::Char(' ') => {
                                    playing = !playing;
                                },
                                _ => ()
                            },
                            _ => ()
                        },
                        Some(Err(err)) => return Err(err),
                        None => ()
                    }
                }
            }
        }
        Ok(())
    }

    fn interval(&self, with_fps: f32) -> time::Interval {
        let duration = if with_fps > 1.0 {
            Duration::from_secs(1) / with_fps.round() as u32
        } else {
            Duration::from_secs(1) * (1.0 / with_fps).round() as u32
        };
        time::interval_at(Instant::now() + duration, duration)
    }

    fn tick(&mut self) -> Result<()> {
        self.game.tick();
        self.render()
    }

    fn render(&mut self) -> Result<()> {
        let (width, height) = terminal_size();
        self.game.resize_if_larger(width, height);
        execute!(self.writer, cursor::MoveTo(0, 0))?;

        for (cell, (x, y)) in self.game.cells() {
            let content = match cell.is_alive() {
                true => " ".on_white(),
                false => " ".on_black(),
            };
            queue!(
                self.writer,
                cursor::MoveTo(x, y),
                style::PrintStyledContent(content)
            )?;
        }
        self.writer.flush()?;

        Ok(())
    }

    fn revive_cell_at_pos(&mut self, x: u16, y: u16) -> Option<()> {
        self.game.revive_cell_at_pos(x, y)?;

        execute!(
            self.writer,
            cursor::MoveTo(x, y),
            style::PrintStyledContent(" ".on_white())
        )
        .unwrap();

        Some(())
    }

    fn kill_cell_at_pos(&mut self, x: u16, y: u16) -> Option<()> {
        self.game.kill_cell_at_pos(x, y)?;

        execute!(
            self.writer,
            cursor::MoveTo(x, y),
            style::PrintStyledContent(" ".on_black())
        )
        .unwrap();

        Some(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    terminal::enable_raw_mode()?;

    let mut stdout = stdout();
    execute!(
        stdout,
        terminal::EnterAlternateScreen,
        event::EnableMouseCapture
    )?;

    TuiGame::new(&mut stdout).run().await?;

    execute!(
        stdout,
        terminal::LeaveAlternateScreen,
        event::DisableMouseCapture
    )?;

    terminal::disable_raw_mode()
}
