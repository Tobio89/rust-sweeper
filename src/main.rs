use std::fmt::Debug;
use std::num::ParseIntError;
use std::{fmt, io};

use clap::{Args, Parser, Subcommand};
use colored::Colorize;
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
    Won,
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

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    game: GameOptions,
}

#[derive(Subcommand)]
enum GameOptions {
    /// 10x10 board with 10 mines
    Easy,
    /// 16x16 board with 40 mines
    Medium,
    /// 30x30 board with 99 mines
    Hard,
    /// <SIZE> <MINES>, generate a board of SIZExSIZE, with MINES number of mines
    Custom(CustomArgs),
}

#[derive(Args)]
struct CustomArgs {
    size: usize,
    num_mines: usize,
}

fn main() {
    let cli = Cli::parse();

    let mut GRID_SIZE: usize = 0;
    let mut MINE_COUNT: usize = 0;

    match cli.game {
        GameOptions::Easy => {
            GRID_SIZE = 10;
            MINE_COUNT = 10;
        }
        GameOptions::Medium => {
            GRID_SIZE = 16;
            MINE_COUNT = 40;
        }
        GameOptions::Hard => {
            GRID_SIZE = 30;
            MINE_COUNT = 99;
        }
        GameOptions::Custom(args) => {
            GRID_SIZE = args.size;
            MINE_COUNT = args.num_mines;
        }
    }

    let GRID_SIZE = GRID_SIZE;
    let MINE_COUNT = MINE_COUNT;

    let cells_to_reveal: usize = (GRID_SIZE * GRID_SIZE) - MINE_COUNT;

    let mut game_state = GameState::Playing;
    let mut cells_revealed: usize = 0;

    let mut grid: MineGrid = generate_empty_grid(GRID_SIZE);
    place_mines_in_grid(&mut grid, MINE_COUNT, GRID_SIZE);
    count_nearby_mines(&mut grid, GRID_SIZE);
    println!("{}", "Enter X and Y co-ords to reveal that cell".yellow());
    println!("{}", "e.g: 10 10".yellow());
    loop {
        print_grid(&grid);

        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).unwrap();

        let user_input = parse_input(&buffer, GRID_SIZE);

        let reveal_x: usize;
        let reveal_y: usize;

        match user_input {
            UserInput::BadInput(msg) => {
                println!("Whoops!: {}", msg);
                continue;
            }
            UserInput::Coords(x, y) => {
                reveal_x = x;
                reveal_y = y;
            }
        }

        let found_mine = reveal_cell(reveal_x, reveal_y, &mut grid);
        cells_revealed = count_cells(&grid);
        if found_mine {
            game_state = GameState::Lost;
        } else if cells_revealed >= cells_to_reveal {
            game_state = GameState::Won;
        }

        match game_state {
            GameState::Lost => {
                print_grid(&grid);
                println!("{}", "  YOU LOSE!!  ".black().on_red());
                break;
            }

            GameState::Won => {
                print_grid(&grid);
                println!(
                    "{}",
                    "  All mines safely located! YOU WIN!! ".black().on_cyan()
                );
                break;
            }

            GameState::Playing => {
                print!("{}[2J", 27 as char);
                println!(
                    "Revealed {} out of {} so far",
                    cells_revealed, cells_to_reveal
                );
                continue;
            }
        }
    }
}

enum UserInput {
    BadInput(String),
    Coords(usize, usize),
}

fn parse_input(input_buffer: &String, size: usize) -> UserInput {
    if (input_buffer.len() <= 0) {
        return UserInput::BadInput(String::from("No input"));
    }

    let parts: Vec<&str> = input_buffer.split(" ").collect();
    if parts.len() <= 1 {
        return UserInput::BadInput(String::from("Not enough co-ords"));
    }
    if parts.len() > 2 {
        return UserInput::BadInput(String::from("Too many co-ords"));
    }

    let input_y: Result<usize, ParseIntError> = parts[0].parse();
    let input_x: Result<usize, ParseIntError> = parts[1].strip_suffix("\n").unwrap().parse();

    let user_x: usize;
    let user_y: usize;

    match input_x {
        Ok(number) => {
            user_x = number;
        }
        Err(_) => {
            return UserInput::BadInput(String::from("x value was not a valid number"));
        }
    }

    match input_y {
        Ok(number) => {
            user_y = number;
        }
        Err(_) => {
            return UserInput::BadInput(String::from("y value was not a valid number"));
        }
    }

    if user_x > size || user_y > size {
        return UserInput::BadInput(String::from(
            "co-ord values must be within the size of the grid!",
        ));
    }
    if user_x <= 0 || user_y <= 0 {
        return UserInput::BadInput(String::from(
            "co-ord values must be within the size of the grid!",
        ));
    }

    return UserInput::Coords(user_x - 1, user_y - 1);
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
    let mut result = String::new();

    let size = grid.len();

    for n in 0..size + 1 {
        if n == 0 {
            result.push_str(" 0");
            continue;
        }
        if n < 10 {
            let current = format!("{}", n).red();
            result = format!("{} {}", result, current);
            continue;
        }
        let current = format!("{}", n).red();
        result = format!("{}{}", result, current);
    }
    result.push_str("\n");

    for n in 0..size {
        let row = &grid[n];
        if (n + 1) < 10 {
            let current = format!("{}", n + 1).blue();
            result = format!("{} {}", result, current);
        } else {
            let current = format!("{}", n + 1).blue();
            result = format!("{}{}", result, current);
        }

        for cell in row {
            if !cell.is_revealed {
                result.push_str("⬜");
            } else {
                match cell.value {
                    CellValue::Empty => result.push_str("  "),
                    CellValue::Mine => result.push_str("💥"),
                    CellValue::NearMine(num) => result = format!("{} {}", result, num),
                }
            }
        }
        result.push_str("\n");
    }

    println!("{}", result);
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
                CellValue::NearMine(_) => {
                    grid[i][j].is_revealed = true;
                }
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

fn count_cells(grid: &MineGrid) -> usize {
    let mut result: usize = 0;

    for row in grid {
        for cell in row {
            if cell.is_revealed {
                result += 1;
            }
        }
    }
    return result;
}
