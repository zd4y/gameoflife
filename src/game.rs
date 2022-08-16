#[derive(Debug, PartialEq)]
enum CellKind {
    Alive,
    Dead,
}

#[derive(Debug, PartialEq)]
pub struct Cell {
    x: u16,
    y: u16,
    kind: CellKind,
}

impl Cell {
    pub fn is_alive(&self) -> bool {
        match self.kind {
            CellKind::Alive => true,
            CellKind::Dead => false,
        }
    }

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

    fn live(&mut self) {
        self.kind = CellKind::Alive;
    }

    fn die(&mut self) {
        self.kind = CellKind::Dead;
    }
}

pub struct Game {
    cells: Vec<Vec<Cell>>,
}

impl Game {
    pub fn new(width: u16, height: u16) -> Self {
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

    pub fn cells(&self) -> Vec<(&Cell, (u16, u16))> {
        let mut result = vec![];
        for (y, row) in self.cells.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                result.push((cell, (x as u16, y as u16)))
            }
        }
        result
    }

    pub fn revive_cell_at_pos(&mut self, x: u16, y: u16) -> Option<()> {
        let cell = self.find_cell_at_pos_mut(x, y)?;
        cell.live();
        Some(())
    }

    pub fn kill_cell_at_pos(&mut self, x: u16, y: u16) -> Option<()> {
        let cell = self.find_cell_at_pos_mut(x, y)?;
        cell.die();
        Some(())
    }

    pub fn resize_if_larger(&mut self, width: u16, height: u16) {
        let old_height = self.cells.len() as u16;
        let old_width = self.cells[0].len() as u16;
        if height > old_height {
            for y in old_height..height {
                let mut row = vec![];
                for x in 0..old_width {
                    let cell = Cell::new(x, y);
                    row.push(cell);
                }
                self.cells.push(row);
            }
        }

        if width > old_width {
            for (y, row) in self.cells.iter_mut().enumerate() {
                for x in old_width..width {
                    let cell = Cell::new(x, y as u16);
                    row.push(cell);
                }
            }
        }
    }

    pub fn tick(&mut self) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_resizes_if_larger() {
        let mut game = Game {
            cells: vec![
                vec![Cell::new(0, 0), Cell::new(1, 0), Cell::new(2, 0)],
                vec![Cell::new(0, 1), Cell::new(1, 1), Cell::new(2, 1)],
            ],
        };

        game.resize_if_larger(4, 3);

        assert_eq!(
            game.cells,
            vec![
                vec![
                    Cell::new(0, 0),
                    Cell::new(1, 0),
                    Cell::new(2, 0),
                    Cell::new(3, 0)
                ],
                vec![
                    Cell::new(0, 1),
                    Cell::new(1, 1),
                    Cell::new(2, 1),
                    Cell::new(3, 1)
                ],
                vec![
                    Cell::new(0, 2),
                    Cell::new(1, 2),
                    Cell::new(2, 2),
                    Cell::new(3, 2)
                ]
            ]
        )
    }
}
