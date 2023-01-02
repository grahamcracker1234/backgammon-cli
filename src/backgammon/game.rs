use colored::Colorize;
use std::cell::RefCell;

use super::{
    board::{Board, BoardPosition},
    dice::Dice,
    player::Player,
    turn::{Play, Turn},
    Error,
};

#[derive(Clone)]
pub struct Game {
    pub(super) current_player: Player,
    pub(super) current_roll: RefCell<Dice>,
    pub(super) board: Board,
}

impl Game {
    pub fn new() -> Self {
        Self {
            current_player: Player::random(),
            board: Board::new(),
            current_roll: RefCell::new(Dice::first_roll()),
        }
    }

    pub fn start(&mut self) {
        loop {
            let saved_board = self.board.clone();
            let saved_roll = self.current_roll.borrow().clone();

            println!("\n{self}\n{}", self.current_roll.borrow());

            println!(
                "{:?}",
                Turn::get_available_plays(&self)
                    .map(|m| m.to_string())
                    .collect::<Vec<_>>()
            );

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
                "{} to play ({}): ",
                match self.current_player {
                    Player::Black => "Black",
                    Player::White => "White",
                    Player::None =>
                        panic!("Attempting to get plays from '{:?}'.", self.current_player),
                },
                self.current_roll.borrow()
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

    pub(super) fn take_turn(&mut self, turn: Turn) -> Result<(), Error> {
        for play in turn.plays {
            if let Err(error) = self.check_play(&play) {
                return Err(error);
            }
            self.make_play(&play);
        }

        if self.current_roll.borrow().any_available() {
            return Err(Error::IncompleteTurn);
        }

        Ok(())
    }

    pub(super) fn check_play(&self, play: &Play) -> Result<(), Error> {
        // Ensure current player is making thplay.
        if self.current_player != play.player {
            return Err(Error::PlayMadeOutOfTurn);
        }

        // Ensure that if there is a piece in the bar it is played.
        if self.board.bar(play.player).borrow().count > 0
            && !matches!(play.from, BoardPosition::Bar(_))
        {
            return Err(Error::PlayMadeWithBarFilled);
        }

        let from = match play.from {
            BoardPosition::Bar(player) => self.board.bar(player).borrow(),
            BoardPosition::Rail(_) => return Err(Error::PlayMadeFromRail),
            BoardPosition::Point(index) => self.board.point(index).borrow(),
        };

        let to = match play.to {
            BoardPosition::Bar(_) => return Err(Error::PlayMadeToBar),
            BoardPosition::Rail(player) => self.board.off(player).borrow(),
            BoardPosition::Point(index) => self.board.point(index).borrow(),
        };

        // Ensure there is a piece to play.
        if from.count == 0 {
            return Err(Error::PlayMadeFromEmptyPoint);
        }

        // Ensure the piece to play is the current player's.
        if from.player != play.player {
            return Err(Error::PlayMadeWithOpposingPiece);
        }

        // Ensure the player is moving in the correct direction.
        if !from.is_valid_direction(&to) {
            return Err(Error::InvalidPlayDirection);
        }

        // Ensure that a piece is only played onto another player's piece if the other player's piece the only piece on that space.
        if to.player == !play.player && to.count > 1 {
            return Err(Error::PlayMadeOntoOpposingPiece);
        }

        // Ensure play is possible from the dice rolls.
        let len = from.distance(&to) as u8;
        if !self.current_roll.borrow().check(len) {
            return Err(Error::InvalidPlayLength(len));
        }

        Ok(())
    }

    pub(super) fn make_play(&mut self, play: &Play) {
        let mut to = play.to.point(&self.board).borrow_mut();
        let mut from = play.from.point(&self.board).borrow_mut();

        // Ensure play is possible from the dice rolls.
        self.current_roll
            .borrow_mut()
            .remove(from.distance(&to) as u8);

        // Ensure that a piece is only played onto another player's piece if the other player's piece the only piece on that space.
        if to.player == !play.player && to.count == 1 {
            self.board.bar(to.player).borrow_mut().count += 1;
        }

        // Make the play.
        from.count -= 1;
        to.player = play.player;
        to.count += 1;

        // Reset the player of the previous position if it is empty and not from the bar
        if from.count == 0 && !matches!(play.from, BoardPosition::Bar(_)) {
            from.player = Player::None;
        }
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
