extern crate termion;

use std::io::{stdin, stdout, Read, Write};
use termion::input::{MouseTerminal, TermRead};
use termion::raw::IntoRawMode;
use termion::{clear, color, cursor, style};
use termion::{
    event::{Event, Key, MouseButton, MouseEvent},
    screen::AlternateScreen,
};

#[derive(Debug)]
enum CellKind {
    Alive,
    Dead,
}

#[derive(Debug)]
struct Cell {
    x: u16,
    y: u16,
    kind: CellKind,
}

impl Cell {
    fn new(x: u16, y: u16) -> Self {
        Self {
            x,
            y,
            kind: CellKind::Dead,
        }
    }

    fn new_alive(x: u16, y: u16) -> Self {
        Self {
            x,
            y,
            kind: CellKind::Alive,
        }
    }

    fn is_alive(&self) -> bool {
        match self.kind {
            CellKind::Alive => true,
            CellKind::Dead => false,
        }
    }

    fn live(&mut self) {
        self.kind = CellKind::Alive;
    }

    fn die(&mut self) {
        self.kind = CellKind::Dead;
    }
}

struct Game {
    cells: Vec<Vec<Cell>>,
}

impl Game {
    fn new(width: u16, height: u16) -> Self {
        let mut cells = vec![];
        for y in 0..height {
            let mut row = vec![];
            for x in 0..width {
                let cell = Cell::new(x, y);
                row.push(cell);
            }
            cells.push(row);
        }

        Self::with_cells(cells)
    }

    fn with_cells(cells: Vec<Vec<Cell>>) -> Self {
        Self { cells }
    }

    fn find_cell_at_pos_mut(&mut self, x: u16, y: u16) -> Option<&mut Cell> {
        let x = x as usize;
        let y = y as usize;
        let row = self.cells.get_mut(y)?;
        row.get_mut(x)
    }

    fn get_neighbours_count_at_pos(&self, x: u16, y: u16) -> u8 {
        let prev_y = y.saturating_sub(1) as usize;
        let current_y = y as usize;
        let next_y = y.saturating_add(1) as usize;
        let prev_x = x.saturating_sub(1) as usize;
        let next_x = x.saturating_add(1) as usize;
        let until = next_x - prev_x + 1;
        let filter = |c: &Cell| c.is_alive();

        let prev_count = if prev_y != current_y {
            slice_len_2d(&self.cells, prev_y, prev_x, until, filter)
        } else {
            0
        };
        let current_count = slice_len_2d(&self.cells, current_y, prev_x, until, |c| {
            c.is_alive() && (c.x, c.y) != (x, y)
        });
        let next_count = slice_len_2d(&self.cells, next_y, prev_x, until, filter);

        (prev_count + current_count + next_count) as u8
    }

    fn revive_cell_at_pos(&mut self, x: u16, y: u16) -> Option<()> {
        let cell = self.find_cell_at_pos_mut(x, y)?;
        cell.live();
        Some(())
    }

    fn kill_cell_at_pos(&mut self, x: u16, y: u16) -> Option<()> {
        let cell = self.find_cell_at_pos_mut(x, y)?;
        cell.die();
        Some(())
    }

    fn tick(&mut self) {
        let mut new_cells = vec![];

        for (y, row) in self.cells.iter().enumerate() {
            let mut new_row = vec![];
            for (x, cell) in row.iter().enumerate() {
                let x = x as u16;
                let y = y as u16;
                let neighbours_count = self.get_neighbours_count_at_pos(x, y);
                match (cell.is_alive(), neighbours_count) {
                    (true, 2..=3) | (false, 3) => {
                        let new_cell = Cell::new_alive(x, y);
                        new_row.push(new_cell);
                    }
                    (true, _) | (false, _) => {
                        let new_cell = Cell::new(x, y);
                        new_row.push(new_cell);
                    }
                }
            }
            new_cells.push(new_row);
        }

        self.cells = new_cells
    }
}

struct TuiGame<'a> {
    game: Game,
    stdin: Option<Box<dyn Read>>,
    stdout: &'a mut dyn Write,
}

impl<'a> TuiGame<'a> {
    fn new<R, W>(stdin: R, stdout: &'a mut W) -> Self
    where
        R: Read + 'static,
        W: Write,
    {
        writeln!(
            stdout,
            "{}{}{}",
            cursor::Hide,
            clear::All,
            cursor::Goto(1, 1)
        )
        .unwrap();
        stdout.flush().unwrap();

        let (width, height) = termion::terminal_size().unwrap();
        let game = Game::new(width, height - 1);
        Self {
            game,
            stdin: Some(Box::new(stdin)),
            stdout,
        }
    }

    fn start(&mut self) {
        self.render();
        self.stdout.flush().unwrap();
        self.listen_events();
        write!(self.stdout, "{}{}", style::Reset, cursor::Show).unwrap();
    }

    fn listen_events(&mut self) {
        let events = self.stdin.take().unwrap().events();
        for event in events {
            let event = event.unwrap();
            match event {
                Event::Key(Key::Char('q')) => break,
                Event::Mouse(MouseEvent::Press(MouseButton::Left, a, b))
                | Event::Mouse(MouseEvent::Hold(a, b)) => {
                    let x = a - 1;
                    let y = b - 1;
                    self.revive_cell_at_pos(x, y).unwrap();
                }
                Event::Mouse(MouseEvent::Press(MouseButton::Right, a, b)) => {
                    let x = a - 1;
                    let y = b - 1;
                    self.kill_cell_at_pos(x, y).unwrap();
                }
                Event::Key(Key::Right) => self.tick(),
                _ => (),
            }
            self.stdout.flush().unwrap();
        }
    }

    fn tick(&mut self) {
        self.game.tick();
        self.render();
        self.stdout.flush().unwrap();
    }

    fn render(&mut self) {
        for (a, row) in self.game.cells.iter().enumerate() {
            let y = (a + 1) as u16;
            for (b, cell) in row.iter().enumerate() {
                let x = (b + 1) as u16;
                let color = match cell.is_alive() {
                    true => color::Rgb(255, 255, 255),
                    false => color::Rgb(0, 0, 0),
                };
                write!(self.stdout, "{}{} ", cursor::Goto(x, y), color::Bg(color)).unwrap();
            }
            write!(self.stdout, "{}\n\r", style::Reset).unwrap();
        }
    }

    fn revive_cell_at_pos(&mut self, x: u16, y: u16) -> Option<()> {
        self.game.revive_cell_at_pos(x, y)?;

        write!(
            self.stdout,
            "{}{} ",
            cursor::Goto(x + 1, y + 1),
            color::Bg(color::White)
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
            color::Bg(color::White)
        )
        .unwrap();

        Some(())
    }
}

fn slice_until<T: std::fmt::Debug>(a: &[T], from: usize, until: usize) -> Option<&[T]> {
    a.get(from..).and_then(|a| {
        let until = until.min(a.len());
        a.get(..until)
    })
}

fn slice_len_2d<T, F>(
    a: &[Vec<T>],
    y_index: usize,
    from: usize,
    until: usize,
    mut filter: F,
) -> usize
where
    T: std::fmt::Debug,
    F: FnMut(&T) -> bool,
{
    a.get(y_index)
        .and_then(|slice| slice_until(slice, from, until))
        .map_or(0, |slice| slice.iter().filter(|t| filter(*t)).count())
}

fn main() {
    let stdin = stdin();
    let stdout = stdout().into_raw_mode().unwrap();
    let stdout = MouseTerminal::from(stdout);
    let mut stdout = AlternateScreen::from(stdout);

    let mut game = TuiGame::new(stdin, &mut stdout);
    game.start();
}