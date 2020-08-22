extern crate colored;
extern crate rand;

use colored::*;
use rand::{Rng, rngs::ThreadRng};
use std::io;

#[derive(PartialEq)]
enum CellValue {
    Mine,
    Water,
}

struct Cell {
    value: CellValue,
    opened: bool,
    flagged: bool,
}

#[derive(Debug, PartialEq)]
enum MinesError {
    MineOpened,
    OutOfBounds(u16, u16),
    EmptyField,
    FieldTooSmall(u16, u16),
    TooManyMines,
}

impl Cell {
    fn mine() -> Self {
        Self {
            value: CellValue::Mine,
            opened: false,
            flagged: false,
        }
    }

    fn water() -> Self {
        Self {
            value: CellValue::Water,
            opened: false,
            flagged: false,
        }
    }

    fn open(&mut self) -> Result<(), MinesError> {
        if !self.flagged {
            self.opened = true;
            match self.value {
                CellValue::Mine => Err(MinesError::MineOpened),
                CellValue::Water => Ok(()),
            }
        } else {
            Ok(())
        }
    }

    fn toggle_flag(&mut self) {
        if !self.opened {
            self.flagged = !self.flagged;
        }
    }
}

struct Field {
    cells: Vec<Vec<Cell>>,
    numbers: Vec<Vec<u8>>,
}

impl Field {
    fn with_cells(cells: Vec<Vec<Cell>>) -> Self {
        let mut numbers = vec![];
        for (x, c_col) in cells.iter().enumerate() {
            let mut col = vec![];
            for (y, _cell) in c_col.iter().enumerate() {
                col.push(count_neighbours(&cells, x as u16, y as u16).unwrap());
            }
            numbers.push(col);
        }
        Self {
            cells,
            numbers,
        }
    }

    fn generate(rng: &mut ThreadRng, width: u16, height: u16, mines: u16) -> Result<Self, MinesError> {
        let cells = generate_cells(rng, width.clone(), height.clone(), mines)?;
        Ok(Self::with_cells(cells))
    }

    fn print(&self) {
        for (x, col) in self.cells.iter().enumerate() {
            for (y, cell) in col.iter().enumerate() {
                if cell.flagged {
                    print!("F ");
                } else if !cell.opened {
                    print!("_ ");
                } else {
                    match cell.value {
                        CellValue::Mine => print!("{} ", "X".red()),
                        CellValue::Water => print!("{} ", color_number(self.numbers.get(x).unwrap().get(y).unwrap())),
                    }
                }
            }
            println!();
        }
    }

    fn flag(&mut self, x: u16, y: u16) -> Result<(), MinesError> {
        let _ = get_2d(&mut self.cells, x, y)?;
        let cell: &mut Cell = self.cells.get_mut(x as usize).unwrap().get_mut(y as usize).unwrap();
        cell.toggle_flag();
        Ok(())
    }

    fn open(&mut self, x: u16, y: u16) -> Result<(), MinesError> {
        let _ = get_2d(&self.cells, x, y)?;
        let cell: &mut Cell = self.cells.get_mut(x as usize).unwrap().get_mut(y as usize).unwrap();
        if cell.opened {
            return Ok(());
        }
        cell.open()?;
        if self.numbers.get(x as usize).unwrap().get(y as usize).unwrap().eq(&0) {
            for nx in min_coord(x)..x + 2 {
                for ny in min_coord(y)..y + 2 {
                    let _ = self.open(nx, ny);
                }
            }
        }
        Ok(())
    }

    fn chord(&mut self, x: u16, y: u16) -> Result<(), MinesError>{
        {
            let cell  = get_2d(&self.cells, x, y)?;
            if !cell.opened {
                return Ok(());
            }
        }
        let number = get_2d(&self.numbers, x, y)?;
        let mut counter = 0;
        do_with_neighbours(&self.cells, x, y, |_, _, c| {
            if c.flagged {
                counter += 1;
            }
            Ok(())
        })?;
        if number.ne(&counter) {
            return Ok(())
        }
        self.open(x, y)?;
        for x in min_coord(x)..x+2 {
            for y in min_coord(y)..y+2 {
                {
                    let cell = get_2d(&self.cells, x, y)?;
                    if cell.opened || cell.flagged {
                        continue;
                    }

                }
                self.open(x, y)?;
            }
        }
        Ok(())
    }

    fn is_won(&self) -> bool {
        for col in self.cells.iter() {
            for cell in col.iter() {
                if cell.value.eq(&CellValue::Water) && !cell.opened {
                    return false;
                }
            }
        }
        true
    }
}

fn generate_cells(rng: &mut ThreadRng, width: u16, height: u16, mines: u16) -> Result<Vec<Vec<Cell>>, MinesError> {
    if width * height < mines * 10 {
        return Err(MinesError::TooManyMines);
    }
    if width < 8 && height < 8 {
        return Err(MinesError::FieldTooSmall(width, height));
    }
    let mut bombs = vec![];
    for _ in 0..mines {
        loop {
            let coords = (
                rng.gen_range(0, width),
                rng.gen_range(0, height)
            );
            if !bombs.contains(&coords) {
                bombs.push(coords);
                break;
            }
        }
    }
    let cells = (0..width).map(|x| {
        (0..height).map(|y| {
            if bombs.contains(&(x, y)) {
                Cell::mine()
            } else {
                Cell::water()
            }
        }).collect()
    }).collect();
    Ok(cells)
}

fn min_coord(c: u16) -> u16 {
    if c > 0 {
        c - 1
    } else {
        c
    }
}

fn count_neighbours(cells: &Vec<Vec<Cell>>, x: u16, y: u16) -> Result<u8, MinesError> {
    let mut counter = 0;
    if cells.is_empty() {
        return Err(MinesError::EmptyField);
    }
    if x as usize >= cells.len() {
        return Err(MinesError::OutOfBounds(x, y));
    }
    do_with_neighbours(cells, x, y, |_, _, c| {
        if c.value.eq(&CellValue::Mine) {
            counter += 1;
        }
        Ok(())
    })?;
    Ok(counter)
}

fn do_with_neighbours<F>(cells: &Vec<Vec<Cell>>, x: u16, y: u16, mut cb: F) -> Result<(), MinesError>
    where F: FnMut(u16, u16, &Cell) -> Result<(), MinesError> {
    for curr_x in min_coord(x)..x+2 {
        for curr_y in min_coord(y)..y+2 {
            if curr_x == x && curr_y == y {
                continue;
            }
            if let Ok(cell) = get_2d(cells, curr_x, curr_y) {
                cb(curr_x, curr_y, cell)?;
            }
        }
    }
    Ok(())
}

fn do_with_neighbours_mut<F>(cells: &mut Vec<Vec<Cell>>, x: u16, y: u16, mut cb: F) -> Result<(), MinesError>
    where F: FnMut(u16, u16, &mut Cell) -> Result<(), MinesError> {
    for curr_x in min_coord(x)..x+2 {
        for curr_y in min_coord(y)..y+2 {
            if let Ok(cell) = get_2d_mut(cells, curr_x, curr_y) {
                cb(curr_x, curr_y, cell)?;
            }
        }
    }
    Ok(())
}

fn get_2d<T>(vec: &Vec<Vec<T>>, x: u16, y: u16) -> Result<&T, MinesError> {
    if let Some(col) = vec.get(x as usize) {
        if let Some(item) = col.get(y as usize) {
            return Ok(item);
        }
    }
    Err(MinesError::OutOfBounds(x, y))
}

fn get_2d_mut<T>(vec: &mut Vec<Vec<T>>, x: u16, y: u16) -> Result<&mut T, MinesError> {
    if let Some(col) = vec.get_mut(x as usize) {
        if let Some(item) = col.get_mut(y as usize) {
            return Ok(item);
        }
    }
    Err(MinesError::OutOfBounds(x, y))
}

fn color_number(num: &u8) -> ColoredString {
    let s = format!("{}", num);
    match num {
        0 => s.blue(),
        1 => s.bright_green(),
        2 => s.green(),
        3 => s.yellow(),
        4 => s.bright_red(),
        5 => s.red(),
        _ => s.magenta(),
    }
}

fn main() {
    let mut args = std::env::args();
    args.next();
    let width: u16 = args.next().unwrap().parse().unwrap();
    let height: u16 = args.next().unwrap().parse().unwrap();
    let mines = width * height / 10;

    let mut rng = rand::thread_rng();
    let field = Field::generate(&mut rng, height.clone(), width.clone(), mines);
    let mut field = match field {
        Ok(field) => field,
        Err(MinesError::TooManyMines) => panic!("Too many mines"),
        _ => panic!("Error?!"),
    };
    let mut in_buffer = String::new();
    let stdin = io::stdin();
    field.print();
    loop {
        let mut flag = false;
        let mut chord = false;
        let selection;
        loop {
            stdin.read_line(&mut in_buffer).unwrap();
            let mut input: Vec<String> = in_buffer.trim().split(" ").filter(|s| s.len() > 0).map(|s| s.into()).collect();
            if let Some(first) = input.get(0) {
                if first.trim().eq("f") {
                    flag = true;
                    input.remove(0);
                } else if first.trim().eq("c") {
                    chord = true;
                    input.remove(0);
                }
            }
            if input.len() == 2 {
                let input: Vec<Result<u16, _>> = input.iter().map(|s| s.trim()).map(|s| s.parse()).filter(|v| v.is_ok()).collect();
                let input: Vec<u16> = input.into_iter().map(|r| r.unwrap()).collect();
                if input.len() == 2 {
                    let x = input.get(0).unwrap().clone();
                    let y = input.get(1).unwrap().clone();
                    selection = (
                        if x > 0 { x - 1 } else { x },
                        if y > 0 { y - 1 } else { y },
                    );
                    break;
                } else {
                    println!("Wrong coords count ({})", input.len());
                    in_buffer.clear();
                }
            } else {
                println!("Wrong input count ({})", input.len());
                in_buffer.clear();
            }
        }
        in_buffer.clear();
        if flag {
            let _ = field.flag(selection.1, selection.0);
        } else if chord {
            match field.chord(selection.1, selection.0) {
                Err(MinesError::MineOpened) => {
                    field.print();
                    panic!("You lost!");
                },
                _ => {},
            }
        } else {
            match field.open(selection.1, selection.0) {
                Err(MinesError::MineOpened) => {
                    field.print();
                    panic!("You lost!");
                }
                _ => {},
            }
        }
        println!();
        field.print();
        println!();
        if field.is_won() {
            println!("{}", "You won!".green().bold());
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{CellValue, Cell};

    mod cell {
        use crate::{Cell, MinesError};

        #[test]
        fn open_water() {
            assert!(Cell::water().open().is_ok());
        }

        #[test]
        fn open_mine() {
            assert_eq!(Err(MinesError::MineOpened), Cell::mine().open());
        }

        #[test]
        fn toggle_flag() {
            let mut cell = Cell::mine();
            assert!(!cell.flagged);
            cell.toggle_flag();
            assert!(cell.flagged);
            cell.toggle_flag();
            assert!(!cell.flagged);
        }

        #[test]
        fn toggle_opened() {
            let mut cell = Cell::water();
            let _ = cell.open();
            cell.toggle_flag();
            assert!(!cell.flagged);
        }

        #[test]
        fn open_flagged() {
            let mut cell = Cell::water();
            cell.flagged = true;
            cell.open().unwrap();
            assert!(!cell.opened);
        }
    }

    mod field {
        use crate::{CellValue::*, Field};
        use crate::tests::cells_from_types;

        #[test]
        fn with_cells() {
            let cells = cells_from_types(vec![
                vec![Water, Water, Mine],
                vec![Mine, Water, Water],
                vec![Mine, Water, Mine],
            ]);
            let field = Field::with_cells(cells);
            let numbers = vec![
                vec![1, 2, 0],
                vec![1, 4, 2],
                vec![1, 3, 0],
            ];
            assert_eq!(numbers, field.numbers);
        }

        #[test]
        fn flag_cell() {
            let cells = cells_from_types(vec![
                vec![Water, Mine],
                vec![Water, Water],
            ]);
            let mut field = Field::with_cells(cells);
            field.flag(1, 0).unwrap();
            assert!(field.cells.get(1).unwrap().get(0).unwrap().flagged);
            field.flag(1, 0).unwrap();
            assert!(!field.cells.get(1).unwrap().get(0).unwrap().flagged);
        }
    }

    fn cells_from_types(types: Vec<Vec<CellValue>>) -> Vec<Vec<Cell>> {
        types.into_iter().map(|c|
            c.into_iter().map(|v|
                Cell {
                    value: v,
                    opened: false,
                    flagged: false,
                }
            ).collect()
        ).collect()
    }
}
