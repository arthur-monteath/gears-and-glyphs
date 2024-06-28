use std::{fmt::Error};

use serenity::all::UserId;

struct Player {
    id: UserId,
    health: u32,
}

impl Player {
    pub fn damage(&mut self, amount: u32) {
        self.health -= amount;
    }
}

pub struct Duel {
    inviter: Player, // P1
    invitee: Player, // P2
    board: Board,
}

pub enum Action {
    Attack,
    Move,
    Use,
}

impl Duel {
    pub fn new(_inviter: UserId, _invitee: UserId) -> Duel {
        Duel {
            inviter: Player {
                id: _inviter,
                health: 100
            },

            invitee: Player {
                id: _invitee,
                health: 100
            },

            board: Board::new()
        }
    }

    pub fn get_board(&self) -> String {

        return self.board.get_string()
    }

    pub fn play(player: bool, action: Action) {

        match action {
            Action::Attack => return,
            Action::Move => return,
            Action::Use => return,
        }
    }
}

struct Board {
    tiles: [[Slot; 12]; 7],
}

impl Board {

    fn new() -> Board {
        let mut board = Board {
            tiles: [[Slot::Empty; 12]; 7],
        };

        board.tiles[3][1] = Slot::Player1;
        board.tiles[3][10] = Slot::Player2;

        board
    }

    fn get_string(&self) -> String {
        let mut board = String::new();

        // Add top-left corner and column headers
        board.push_str(":blue_square:");
        for col in 0..12 {
            let col_header = match col {
                0 => ":regional_indicator_a:",
                1 => ":regional_indicator_b:",
                2 => ":regional_indicator_c:",
                3 => ":regional_indicator_d:",
                4 => ":regional_indicator_e:",
                5 => ":regional_indicator_f:",
                6 => ":regional_indicator_g:",
                7 => ":regional_indicator_h:",
                8 => ":regional_indicator_i:",
                9 => ":regional_indicator_j:",
                10 => ":regional_indicator_k:",
                11 => ":regional_indicator_l:",
                _ => unreachable!(),
            };
            board.push_str(col_header);
        }
        board.push('\n');

        // Add rows with row headers
        for (i, row) in self.tiles.iter().enumerate() {
            // Add row label
            let row_header = match i {
                0 => ":one:",
                1 => ":two:",
                2 => ":three:",
                3 => ":four:",
                4 => ":five:",
                5 => ":six:",
                6 => ":seven:",
                _ => unreachable!(),
            };
            board.push_str(row_header);
            for &tile in row {
                let tile_str = match tile {
                    Slot::Player1 => ":sunglasses:",
                    Slot::Player2 => ":sunglasses:",
                    Slot::Empty => ":brown_square:",
                    Slot::Trap => ":brown_square:",
                    Slot::Wall => ":black_large_square:",
                };
                board.push_str(tile_str);
            }
            board.push('\n');
        }

        board
    }

    fn move_player(&mut self, player: bool, pos: &str) -> Result<(), String>
    {
        // Turn number to 0-based index
        let col = (pos.to_uppercase().chars().next().unwrap() as u8 - 'A' as u8) as usize;
        let row: usize = match pos[1..].parse::<usize>() {
            Ok(num) => num - 1,
            Err(e) => return Err(e.to_string()),
        };

        // Check if the position is within bounds
        if row >= 7 || col >= 12 {
            return Err("Position out of bounds!".to_string());
        }

        // Check if the tile is empty
        if self.tiles[row][col] != Slot::Empty {
            return Err("Position already occupied!".to_string());
        }

        let current_pos = self.find_player(player);
        if let Some((cur_row, cur_col)) = current_pos {
            // Check if movement is within player's move distance
            let distance = ((cur_row as isize - row as isize).pow(2) + (cur_col as isize - col as isize).pow(2)) as u32;
            if distance > 4 {
                return Err("Position is too far!".to_string());
            }
        }

        // Plus, check if the tile player is trying to move is slot::Empty

        self.tiles[row][col] = if player { Slot::Player1 } else { Slot::Player2 };

        return Ok(());
    }

    fn find_player(&self, player: bool) -> Option<(usize, usize)> {
        let target = if player { Slot::Player1 } else { Slot::Player2 };
        for (i, row) in self.tiles.iter().enumerate() {
            for (j, &tile) in row.iter().enumerate() {
                if tile == target {
                    return Some((i, j));
                }
            }
        }
        None
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Slot { // Use the commands as what they should be as String
    Player1,// :sunglasses:
    Player2,// :sunglasses:
    Empty,  // :brown_square:
    Trap,   // :brown_square:
    Wall,   // :black_large_square:
}