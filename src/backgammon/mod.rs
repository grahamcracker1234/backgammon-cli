use colored::Colorize;
use itertools::Itertools;
use std::cell::{RefCell, RefMut};

mod board;
mod player;
mod roll;
mod turn;

use board::{Board, BoardPosition, Point};
use player::Player;
use roll::Roll;
use turn::{Move, Turn};

pub struct Game {
    current_player: Player,
    current_roll: RefCell<Roll>,
    board: Board,
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

            let notation = self.get_notation();
            let turn = match Turn::from(notation, &self) {
                Ok(turn) => turn,
                Err(error) => {
                    println!("{}", error.red().bold());
                    self.board = saved_board;
                    *self.current_roll.borrow_mut() = saved_roll;
                    continue;
                }
            };

            if let Err(error) = self.take_turn(turn) {
                println!("{}", error.red().bold());
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

    fn take_turn(&self, turn: Turn) -> Result<(), &'static str> {
        for r#move in turn.moves {
            self.make_move(r#move)?;
        }

        if self.current_roll.borrow().available() {
            return Err("Did not use all available moves.");
        }

        Ok(())
    }

    fn make_move(&self, r#move: Move) -> Result<(), &'static str> {
        // Ensure current player is making the move.
        if self.current_player != r#move.player {
            return Err("Only current player can move.");
        }

        // Ensure the player is moving in the correct direction.
        if !r#move.valid_direction() {
            return Err("Attempted to move backwards.");
        }

        // Ensures move is possible from the dice rolls.
        self.current_roll
            .borrow_mut()
            .remove(r#move.distance() as u8)?;

        // Ensure that if there is a piece in the bar it is moved.
        if *self.board.bar[&r#move.player].borrow().count() > 0
            && !matches!(r#move.from, BoardPosition::Bar(_))
        {
            return Err("Attempted to move a piece while there is one in the bar.");
        }

        let mut from: RefMut<dyn Point> = match r#move.from {
            BoardPosition::Off(off) => off.borrow_mut(),
            BoardPosition::Bar(bar) => bar.borrow_mut(),
            BoardPosition::Point(point) => point.borrow_mut(),
        };

        let mut to: RefMut<dyn Point> = match r#move.to {
            BoardPosition::Off(off) => off.borrow_mut(),
            BoardPosition::Bar(bar) => bar.borrow_mut(),
            BoardPosition::Point(point) => point.borrow_mut(),
        };

        // Ensure there is a piece to move.
        if *from.count() == 0 {
            return Err("Attempted to move nonexistent piece.");
        }

        // Ensure the piece to move is the current player's.
        if *from.player() != r#move.player {
            return Err("Attempted to move another player's piece.");
        }

        // Ensure that a piece is only moved onto another player's piece if the other player's piece the only piece on that space.
        if *to.player() == !r#move.player {
            if *to.count() == 1 {
                // Move the other player's piece to the bar.
                *self.board.bar[to.player()].borrow_mut().count_mut() += 1;
                *to.count_mut() = 0;
            } else {
                return Err("Attempted to illegally move onto another player.");
            }
        }

        // Make the move.
        *from.count_mut() -= 1;
        *to.player_mut() = r#move.player;
        *to.count_mut() += 1;

        // Reset the player of the previous position if it is empty and not from the bar
        if *from.count() == 0 && !matches!(r#move.from, BoardPosition::Bar(_)) {
            *from.player_mut() = Player::None;
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
