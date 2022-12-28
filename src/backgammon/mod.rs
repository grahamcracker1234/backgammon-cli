use itertools::Itertools;
use std::cell::RefCell;
use std::collections::HashMap;

mod board;
mod turn;

use board::Board;
use turn::{Move, Turn};

pub struct Game {
    current_player: RefCell<Player>,
    current_roll: RefCell<[u8; 2]>,
    board: Board,
}

impl Game {
    pub fn new() -> Self {
        Self {
            current_player: RefCell::new(Player::None),
            board: Board::new(),
            current_roll: RefCell::new([0, 0]),
        }
    }

    pub fn start(&self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        *self.current_roll.borrow_mut() = (|| loop {
            let roll1: u8 = rng.gen_range(1..=6);
            let roll2: u8 = rng.gen_range(1..=6);
            if roll1 != roll2 {
                return [roll1, roll2];
            }
        })();

        *self.current_player.borrow_mut() = if rand::thread_rng().gen_bool(0.5) {
            Player::Black
        } else {
            Player::White
        };

        loop {
            println!("\n{:?}\n", self.board);
            match self.get_valid_turn(self.get_notation()) {
                Ok(turn) => {
                    for mut r#move in turn.moves {
                        self.make_valid_move(&mut r#move);
                    }
                    *self.current_roll.borrow_mut() = [rng.gen_range(1..=6), rng.gen_range(1..=6)];
                    let current_player = *self.current_player.borrow();

                    *self.current_player.borrow_mut() = match current_player {
                        Player::White => Player::Black,
                        Player::Black => Player::White,
                        Player::None => panic!("No current player."),
                    };
                }
                Err(err) => {
                    println!("{}", err);
                }
            }
        }
    }

    fn get_notation(&self) -> String {
        use std::{io, io::Write};
        let [roll1, roll2] = *self.current_roll.borrow();
        print!(
            "{} to move ({}-{}): ",
            if *self.current_player.borrow() == Player::Black {
                "Black"
            } else if *self.current_player.borrow() == Player::White {
                "White"
            } else {
                panic!("Attempting to get moves from '{:?}'.", self.current_player)
            },
            roll1,
            roll2
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
                            Move::new(
                                *self.current_player.borrow(),
                                (i, &self.board.points[i]),
                                (j, &self.board.points[j]),
                            )
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>(),
        )
    }

    fn get_valid_turn(&self, notation: String) -> Result<Turn, &'static str> {
        let turn = self.get_turn(notation);

        let current_roll = self.current_roll.borrow();

        let mut rolls: HashMap<u8, u8> = HashMap::new();
        if current_roll[0] == current_roll[1] {
            rolls.insert(current_roll[0], 4);
        } else {
            rolls.insert(current_roll[0], 1);
            rolls.insert(current_roll[1], 1);
        }

        // Check each move
        for r#move in &turn.moves {
            let from_idx = r#move.from.0;
            let to_idx = r#move.to.0;

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
            match rolls.get_mut(&diff) {
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

        let mut from = r#move.from.1.borrow_mut();
        let mut to = r#move.to.1.borrow_mut();

        from.count -= 1;
        if from.count == 0 {
            from.player = Player::None;
        }

        if Player::are_opposites(to.player, r#move.player) && to.count == 1 {
            if r#move.player == Player::Black {
                self.board.points[0].borrow_mut().count += 1;
            } else if r#move.player == Player::White {
                self.board.points[25].borrow_mut().count += 1;
            }
        }

        to.player = r#move.player;
        to.count += 1;
    }

    fn is_move_valid(&self, r#move: &Move) -> bool {
        // println!("1");
        if *self.current_player.borrow() != r#move.player {
            return false;
        }

        let (from_idx, from) = r#move.from;
        let from = from.borrow();

        let (to_idx, to) = r#move.to;
        let to = to.borrow();

        // println!("2");
        if from.count <= 0 {
            return false;
        }

        // println!("3");
        if from.player != r#move.player {
            return false;
        }

        // println!("4");
        if Player::are_opposites(to.player, r#move.player) && to.count > 1 {
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

#[derive(Clone, Copy, Eq, Debug, Hash, PartialEq)]
pub enum Player {
    Black,
    White,
    None,
}

impl Player {
    fn are_opposites(a: Player, b: Player) -> bool {
        (a == Player::Black && b == Player::White) || (a == Player::White && b == Player::Black)
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

        let mut points = Game::new().board.points;
        *points[0].borrow_mut() = Point::new(0, Player::None);
        *points[5].borrow_mut() = Point::new(1, Player::Black);
        *points[5].borrow_mut() = Point::new(1, Player::Black);

        Ok(())
    }
}
