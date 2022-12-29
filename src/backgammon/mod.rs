use itertools::Itertools;
use std::cell::RefCell;
use std::collections::HashMap;

mod board;
mod player;
mod roll;
mod turn;

use board::Board;
use player::Player;
use roll::Roll;
use turn::{Move, Turn};

pub struct Game {
    current_player: RefCell<Player>,
    current_roll: RefCell<Roll>,
    board: Board,
}

impl Game {
    pub fn new() -> Self {
        Self {
            current_player: RefCell::new(Player::random()),
            board: Board::new(),
            current_roll: RefCell::new(Roll::first_roll()),
        }
    }

    pub fn start(&self) {
        loop {
            println!("\n{self}\n");
            let notation = self.get_notation();
            let turn = self.get_turn(notation);
            self.take_turn(turn);
            self.change_turn();
        }
    }

    #[allow(unstable_name_collisions)]
    fn get_notation(&self) -> String {
        use std::{io, io::Write};
        print!(
            "{} to move ({}): ",
            if *self.current_player.borrow() == Player::Black {
                "Black"
            } else if *self.current_player.borrow() == Player::White {
                "White"
            } else {
                panic!("Attempting to get moves from '{:?}'.", self.current_player)
            },
            self.current_roll
                .borrow()
                .dice
                .into_iter()
                .map(|die| die.to_string())
                .intersperse("-".to_string())
                .collect::<String>()
        );

        io::stdout()
            .flush()
            .expect("Failed to flush standard output.");

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");

        input
    }

    fn get_turn(&self, notation: String) -> Turn {
        Turn::new(
            regex::Regex::new(r"\d+(?:/\d+)+")
                .expect("Regex is invalid")
                .find_iter(&notation)
                .flat_map(|m| {
                    m.as_str()
                        .split('/')
                        .map(|m| m.parse::<usize>().unwrap())
                        .tuple_windows()
                        .map(|(i, j)| {
                            let player = *self.current_player.borrow();
                            Move::new(
                                player,
                                &self.board.get_point(i - 1, player),
                                &self.board.get_point(j - 1, player),
                            )
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>(),
        )
    }

    fn take_turn(&self, turn: Turn) {
        for r#move in turn.moves {
            self.make_move(r#move);
        }
    }

    fn make_move(&self, r#move: Move) {
        let mut from = r#move.from.borrow_mut();
        let mut to = r#move.to.borrow_mut();

        from.count -= 1;
        if from.count == 0 {
            from.player = Player::None;
        }

        // if to.player == !r#move.player && to.count == 1 {
        //     if r#move.player == Player::Black {
        //         self.board.points[0].borrow_mut().count += 1;
        //     } else if r#move.player == Player::White {
        //         self.board.points[25].borrow_mut().count += 1;
        //     }
        // }

        to.player = r#move.player;
        to.count += 1;
    }

    fn change_turn(&self) {
        self.current_roll.borrow_mut().reroll();
        self.current_player.borrow_mut().switch();
    }

    fn get_valid_turn(&self, notation: String) -> Result<Turn, &'static str> {
        let turn = self.get_turn(notation);

        let current_roll = self.current_roll.borrow();

        let mut dice_freq: HashMap<u8, u8> = HashMap::new();
        if current_roll.is_double() {
            dice_freq.insert(current_roll.dice[0], 4);
        } else {
            dice_freq.insert(current_roll.dice[0], 1);
            dice_freq.insert(current_roll.dice[1], 1);
        }

        // Check each move
        for r#move in &turn.moves {
            let from_idx = r#move.from.borrow().pos;
            let to_idx = r#move.to.borrow().pos;

            // Check if move direction is correct.
            if (from_idx.cmp(&to_idx) == std::cmp::Ordering::Greater
                && r#move.player == Player::Black)
                || (from_idx.cmp(&to_idx) == std::cmp::Ordering::Less
                    && r#move.player == Player::White)
            {
                return Err("Invalid move direction.");
            }

            // Check if move distance is valid.
            let diff = from_idx.abs_diff(to_idx) as u8;
            match dice_freq.get_mut(&diff) {
                Some(count) if *count > 0 => *count -= 1,
                _ => return Err("Invalid move distance."),
            }
        }

        Ok(turn)
    }

    fn make_valid_move(&self, r#move: &mut Move) {
        if !self.is_move_valid(&r#move) {
            panic!("Move is invalid");
        }

        let mut from = r#move.from.borrow_mut();
        let mut to = r#move.to.borrow_mut();

        from.count -= 1;
        if from.count == 0 {
            from.player = Player::None;
        }

        // if !to.player == r#move.player && to.count == 1 {
        //     if r#move.player == Player::Black {
        //         self.board.points[0].borrow_mut().count += 1;
        //     } else if r#move.player == Player::White {
        //         self.board.points[25].borrow_mut().count += 1;
        //     }
        // }

        to.player = r#move.player;
        to.count += 1;
    }

    fn is_move_valid(&self, r#move: &Move) -> bool {
        // println!("1");
        if *self.current_player.borrow() != r#move.player {
            return false;
        }

        let from = r#move.from.borrow();
        let to = r#move.to.borrow();

        // println!("2");
        if from.count <= 0 {
            return false;
        }

        // println!("3");
        if from.player != r#move.player {
            return false;
        }

        // println!("4");
        if !to.player == r#move.player && to.count > 1 {
            return false;
        }

        // println!("5");
        if to.player != r#move.player && to.player != Player::None {
            return false;
        }

        // println!("6");
        return true;
    }
}

impl std::fmt::Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self.current_player.borrow() {
            Player::White => write!(f, "{:#}", self.board),
            _ => write!(f, "{}", self.board),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::board::*;
    use super::*;

    #[test]
    fn basic_move() -> Result<(), &'static str> {
        let game = Game::new();
        *game.current_player.borrow_mut() = Player::Black;

        let turn = game.get_valid_turn("1/2/3/5 1/5/7 17/19".to_string())?;
        for mut r#move in turn.moves {
            game.make_valid_move(&mut r#move);
        }

        // let mut points = Game::new().board.points;
        // *points[0].borrow_mut() = Point::new(0, Player::None);
        // *points[5].borrow_mut() = Point::new(1, Player::Black);
        // *points[5].borrow_mut() = Point::new(1, Player::Black);

        Ok(())
    }
}
