use colored::Colorize;
use itertools::Itertools;
use std::cell::RefCell;

use super::{
    board::{Board, BoardPosition},
    player::Player,
    roll::Roll,
    turn::{Move, Turn},
    Error,
};

#[derive(Clone)]
pub struct Game {
    pub(super) current_player: Player,
    pub(super) current_roll: RefCell<Roll>,
    pub(super) board: Board,
}

impl Game {
    pub fn new() -> Self {
        Self {
            current_player: Player::random(),
            board: Board::new(),
            current_roll: RefCell::new(Roll::first_roll()),
        }
    }

    pub fn start(&mut self) {
        loop {
            let saved_board = self.board.clone();
            let saved_roll = self.current_roll.borrow().clone();

            println!("\n{self}\n");

            // println!(
            //     "{:?}",
            //     Turn::get_available_moves(self)
            //         .map(|m| m.to_string())
            //         .collect::<Vec<_>>()
            // );

            let notation = self.get_notation();
            let turn = match Turn::from(notation, self) {
                Ok(turn) => turn,
                Err(error) => {
                    println!("{}", error.to_string().red().bold());
                    // self.board = saved_board;
                    // *self.current_roll.borrow_mut() = saved_roll;
                    continue;
                }
            };

            if let Err(error) = self.take_turn(turn) {
                println!("{}", error.to_string().red().bold());
                self.board = saved_board;
                *self.current_roll.borrow_mut() = saved_roll;
                continue;
            }

            self.change_turn();
        }
    }

    #[allow(unstable_name_collisions)]
    fn get_notation(&self) -> String {
        use std::{io, io::Write};
        print!(
            "{}",
            format!(
                "{} to move ({}): ",
                match self.current_player {
                    Player::Black => "Black",
                    Player::White => "White",
                    Player::None =>
                        panic!("Attempting to get moves from '{:?}'.", self.current_player),
                },
                self.current_roll
                    .borrow()
                    .dice
                    .into_iter()
                    .map(|die| die.to_string())
                    .intersperse("-".to_string())
                    .collect::<String>()
            )
            .green()
            .italic()
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

    fn take_turn(&mut self, turn: Turn) -> Result<(), Error> {
        for r#move in turn.moves {
            self.make_move(r#move)?;
        }

        if self.current_roll.borrow().any_available() {
            return Err(Error::IncompleteTurn);
        }

        Ok(())
    }

    fn make_move(&mut self, r#move: Move) -> Result<(), Error> {
        // Ensure current player is making the move.
        if self.current_player != r#move.player {
            return Err(Error::MoveMadeOutOfTurn);
        }

        // Ensure that if there is a piece in the bar it is moved.
        if self.board.bar(r#move.player).borrow().count > 0
            && !matches!(r#move.from, BoardPosition::Bar(_))
        {
            return Err(Error::MoveMadeWithBarFilled);
        }

        let mut from = match r#move.from {
            BoardPosition::Bar(player) => self.board.bar(player).borrow_mut(),
            BoardPosition::Off(_) => return Err(Error::MoveMadeFromBearingTable),
            BoardPosition::Point(index) => self.board.point(index).borrow_mut(),
        };

        let mut to = match r#move.to {
            BoardPosition::Bar(_) => return Err(Error::MoveMadeToBar),
            BoardPosition::Off(player) => self.board.off(player).borrow_mut(),
            BoardPosition::Point(index) => self.board.point(index).borrow_mut(),
        };

        // Ensure there is a piece to move.
        if from.count == 0 {
            return Err(Error::MoveMadeFromEmptyPoint);
        }

        // Ensure the piece to move is the current player's.
        if from.player != r#move.player {
            return Err(Error::MoveMadeWithOpposingPiece);
        }

        // Ensure the player is moving in the correct direction.
        if !from.is_valid_direction(&to) {
            return Err(Error::InvalidMoveDirection);
        }

        // Ensures move is possible from the dice rolls.
        self.current_roll
            .borrow_mut()
            .remove(from.distance(&to) as u8)?;

        // Ensure that a piece is only moved onto another player's piece if the other player's piece the only piece on that space.
        if to.player == !r#move.player {
            if to.count == 1 {
                // Move the other player's piece to the bar.
                self.board.bar(to.player).borrow_mut().count += 1;
                to.count = 0;
            } else {
                return Err(Error::MoveMadeOntoOpposingPiece);
            }
        }

        // Make the move.
        from.count -= 1;
        to.player = r#move.player;
        to.count += 1;

        // Reset the player of the previous position if it is empty and not from the bar
        if from.count == 0 && !matches!(r#move.from, BoardPosition::Bar(_)) {
            from.player = Player::None;
        }

        Ok(())
    }

    fn change_turn(&mut self) {
        self.current_roll.borrow_mut().reroll();
        self.current_player.switch();
    }
}

impl std::fmt::Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.current_player {
            Player::White => write!(f, "{:#}", self.board),
            _ => write!(f, "{}", self.board),
        }
    }
}
