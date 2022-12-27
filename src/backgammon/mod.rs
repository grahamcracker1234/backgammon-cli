use itertools::Itertools;
use std::cell::RefCell;

mod board;
mod turn;

use board::Board;
use turn::{Move, Turn};

pub struct Game {
    current_player: RefCell<Player>,
    board: Board,
}

impl Game {
    pub fn new() -> Self {
        Self {
            current_player: RefCell::new(Player::None),
            board: Board::new(),
        }
    }

    pub fn start(&self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let rolls = (|| loop {
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
            println!("{:?}", self.board);
            if let Ok(turn) = self.get_turn() {
                for mut r#move in turn.moves {
                    self.make_valid_move(&mut r#move);
                }
                // self.make_valid_move(&mut move2);
            } else {
                println!("Invalid input, try again.");
            }
        }
    }

    fn get_turn(&self) -> Result<Turn, &str> {
        use std::{io, io::Write};
        // println!("{:?}", self.current_player);

        print!(
            "{} to move: ",
            if *self.current_player.borrow() == Player::Black {
                "Black"
            } else if *self.current_player.borrow() == Player::White {
                "White"
            } else {
                panic!("Attempting to get moves from '{:?}'.", self.current_player)
            }
        );
        io::stdout()
            .flush()
            .expect("Failed to flush standard output.");
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");

        let moves = regex::Regex::new(r"\d+(?:/\d+)+")
            .expect("Regex is invalid")
            .find_iter(&input)
            .flat_map(|m| {
                m.as_str()
                    .split('/')
                    .map(|m| m.parse::<usize>().unwrap())
                    .tuple_windows()
                    .map(|(i, j)| {
                        Move::new(
                            *self.current_player.borrow(),
                            &self.board.points[i],
                            &self.board.points[j],
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let turn = Turn::new(moves);

        Ok(turn)
    }

    fn make_valid_move(&self, r#move: &mut Move) {
        if !self.is_move_valid(&r#move) {
            panic!("Move is invalid");
        }

        r#move.from.borrow_mut().count -= 1;
        if r#move.from.borrow().count == 0 {
            r#move.from.borrow_mut().player = Player::None;
        }

        if Player::are_opposites(r#move.to.borrow().player, r#move.player)
            && r#move.to.borrow().count == 1
        {
            if r#move.player == Player::Black {
                self.board.points[0].borrow_mut().count += 1;
            } else if r#move.player == Player::White {
                self.board.points[25].borrow_mut().count += 1;
            }
        }

        r#move.to.borrow_mut().player = r#move.player;
        r#move.to.borrow_mut().count += 1;
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
