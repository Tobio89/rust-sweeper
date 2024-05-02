use std::fmt;
use std::fmt::Debug;

use rand::Rng;

#[derive(Copy, Clone)]
enum CellValue {
    Mine,
    NearMine(usize),
    Empty,
}

#[derive(Debug)]
enum GameState {
    Playing,
    // Won,
    Lost,
}

impl Debug for CellValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CellValue::Empty => return write!(f, "  "),
            CellValue::NearMine(v) => return write!(f, " {}", v),
            CellValue::Mine => return write!(f, "💥"),
        };
    }
}

#[derive(Copy, Clone)]
struct Cell {
    is_revealed: bool,
    value: CellValue,
}
impl Debug for Cell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_revealed {
            return write!(f, "{:?}", self.value);
        } else {
            return write!(f, "⬜");
        }
    }
}

impl Cell {
    fn new() -> Cell {
        return Cell {
            is_revealed: false,
            value: CellValue::Empty,
        };
    }
}

type MineGrid = Vec<Vec<Cell>>;

fn main() {
    const GRID_SIZE: usize = 10;
    const MINE_COUNT: usize = 30;

    let mut _game_state = GameState::Playing;

    let mut grid: MineGrid = generate_empty_grid(GRID_SIZE);
    place_mines_in_grid(&mut grid, MINE_COUNT, GRID_SIZE);
    count_nearby_mines(&mut grid, GRID_SIZE);

    let found_mine = reveal_cell(1, 1, &mut grid);

    if found_mine {
        _game_state = GameState::Lost;
        println!("mine");
    } else {
        println!("not mine");
    }
    print_grid(&grid);
}

fn generate_empty_grid(size: usize) -> MineGrid {
    let grid: Vec<Vec<Cell>> = vec![vec![Cell::new(); size]; size];
    return grid;
}

fn place_mines_in_grid(grid: &mut MineGrid, num_mines: usize, grid_size: usize) {
    let mut rng: rand::prelude::ThreadRng = rand::thread_rng();
    let mut mines_placed: usize = 0;

    while mines_placed < num_mines {
        let random_i: usize = rng.gen_range(0..grid_size);
        let random_j: usize = rng.gen_range(0..grid_size);
        match grid[random_i][random_j].value {
            CellValue::Empty => {
                grid[random_i][random_j].value = CellValue::Mine;
                mines_placed += 1;
            }
            _ => {}
        }
    }
}

fn count_cell_mine_neighbours(
    cell_i: usize,
    cell_j: usize,
    grid_size: usize,
    grid: &MineGrid,
) -> usize {
    let i_cell_i: i64 = cell_i.try_into().unwrap();
    let i_cell_j: i64 = cell_j.try_into().unwrap();

    let i_grid_size: i64 = grid_size.try_into().unwrap();

    let mut i_min = i_cell_i - 1;
    let mut i_max = i_cell_i + 2;

    let mut j_min = i_cell_j - 1;
    let mut j_max = i_cell_j + 2;

    if i_min < 0 {
        i_min = 0;
    };
    if i_max > i_grid_size {
        i_max = i_grid_size;
    };
    if j_min < 0 {
        j_min = 0;
    };
    if j_max > i_grid_size {
        j_max = i_grid_size;
    };

    let mut result: usize = 0;

    for i in i_min..i_max {
        for j in j_min..j_max {
            if i == i_cell_i && j == i_cell_j {
                continue;
            }
            match grid[i as usize][j as usize].value {
                CellValue::Mine => {
                    result += 1;
                }
                CellValue::Empty => {}
                CellValue::NearMine(_v) => {}
            }
        }
    }
    return result;
}

fn count_nearby_mines(grid: &mut MineGrid, grid_size: usize) {
    for i in 0..grid_size {
        for j in 0..grid_size {
            let cell = &grid[i][j];
            match cell.value {
                CellValue::Mine => {}
                CellValue::Empty => {
                    let amt = count_cell_mine_neighbours(i, j, grid_size, grid);
                    if amt > 0 {
                        grid[i][j].value = CellValue::NearMine(amt);
                    }
                }
                CellValue::NearMine(amt) => {
                    let new_amt = amt + count_cell_mine_neighbours(i, j, grid_size, grid);
                    grid[i][j].value = CellValue::NearMine(new_amt);
                }
            }
        }
    }
}

fn print_grid(grid: &MineGrid) {
    for g in grid {
        println!("{:?}", g);
    }
    println!("-------");
}

fn clamp(x: i64, min: i64, max: i64) -> usize {
    if x < min {
        return min as usize;
    }
    if x > max {
        return max as usize;
    }
    x as usize
}

fn recursive_reveal(selected_x: usize, selected_y: usize, grid: &mut MineGrid) {
    let size: i64 = grid.len().try_into().unwrap();

    let i_x: i64 = selected_x.try_into().unwrap();
    let i_y: i64 = selected_y.try_into().unwrap();

    let x_min = clamp(i_x - 1, 0, size - 1);
    let x_max = clamp(i_x + 1, 0, size - 1);

    let y_min = clamp(i_y - 1, 0, size - 1);
    let y_max = clamp(i_y + 1, 0, size - 1);

    for i in x_min..x_max + 1 {
        for j in y_min..y_max + 1 {
            if i == selected_x && j == selected_y {
                continue;
            }
            if grid[i][j].is_revealed {
                continue;
            }
            match grid[i][j].value {
                CellValue::Mine => continue,
                CellValue::NearMine(_) => grid[i][j].is_revealed = true,
                CellValue::Empty => {
                    grid[i][j].is_revealed = true;
                    recursive_reveal(i, j, grid);
                }
            }
        }
    }
}

fn reveal_cell(reveal_x: usize, reveal_y: usize, grid: &mut MineGrid) -> bool {
    let mut cell = grid[reveal_x][reveal_y];
    cell.is_revealed = true;

    grid[reveal_x][reveal_y] = cell;

    match cell.value {
        CellValue::Mine => {
            return true;
        }
        _rest => {
            recursive_reveal(reveal_x, reveal_y, grid);
            return false;
        }
    }
}
