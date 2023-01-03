use colored::Colorize;

use crate::backgammon::{
    board::{Board, Position},
    dice::Dice,
    notation::{Notation, Play, Turn},
    player::Player,
    Error,
};

use super::board::BOARD_SIZE;

#[derive(Clone)]
pub struct Game {
    pub(super) current_player: Player,
    pub(super) current_roll: Dice,
    pub(super) board: Board,
}

impl Game {
    pub fn new() -> Self {
        Self {
            current_player: Player::random(),
            board: Board::new(),
            current_roll: Dice::first_roll(),
        }
    }

    #[allow(dead_code)]
    fn from(current_player: Player, current_roll: Dice, board: Board) -> Self {
        Self {
            current_player,
            current_roll,
            board,
        }
    }

    pub fn start(&mut self) {
        loop {
            let saved_board = self.board.clone();
            let saved_roll = self.current_roll.clone();

            println!("\n{self}\n");

            // println!(
            //     "{:?}",
            //     Turn::get_available_plays(&self)
            //         .map(|m| m.to_string())
            //         .collect::<Vec<_>>()
            // );

            let notation = self.get_notation();
            let turn = match notation.turn(self.current_player) {
                Ok(turn) => turn,
                Err(error) => {
                    println!("{}", error.to_string().red().bold());
                    continue;
                }
            };

            if let Err(error) = self.take_turn(turn) {
                println!("{}", error.to_string().red().bold());
                self.board = saved_board;
                self.current_roll = saved_roll;
                continue;
            }

            self.change_turn();
        }
    }

    #[allow(unstable_name_collisions)]
    fn get_notation(&self) -> Notation {
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
                self.current_roll
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

        Notation(input)
    }

    pub(super) fn take_turn(&mut self, turn: Turn) -> Result<(), Error> {
        let Turn(plays) = turn;
        for play in plays {
            if let Err(error) = self.check_play(&play) {
                return Err(error);
            }
            self.make_play(&play);
        }

        if self.current_roll.any_available()
            && self.get_available_plays().collect::<Vec<_>>().len() > 0
        {
            return Err(Error::IncompleteTurn);
        }

        Ok(())
    }

    pub(super) fn check_play(&self, play: &Play) -> Result<(), Error> {
        // Ensure current player is playing.
        if self.current_player != play.player {
            return Err(Error::PlayMadeOutOfTurn);
        }

        // Ensure that if there is a piece in the bar it is played.
        if self.board.bar(play.player).borrow().count > 0 && !matches!(play.from, Position::Bar(_))
        {
            return Err(Error::PlayMadeWithBarFilled);
        }

        // Ensure that piece is not taken from the rail.
        if matches!(play.from, Position::Rail(_)) {
            return Err(Error::PlayMadeFromRail);
        }

        // Ensure that piece is not going to the bar.
        if matches!(play.to, Position::Bar(_)) {
            return Err(Error::PlayMadeToBar);
        }

        let from = play.from.point(&self.board).borrow();
        let to = play.to.point(&self.board).borrow();

        // Ensure there is a piece to play.
        if from.count == 0 {
            return Err(Error::PlayMadeFromEmptyPoint);
        }

        // Ensure the piece to play is the current player's.
        if from.player != play.player {
            return Err(Error::PlayMadeWithOpposingPiece);
        }

        // Ensure the player is moving in the correct direction.
        if !play.is_valid_direction(&self.board) {
            return Err(Error::InvalidPlayDirection);
        }

        // Ensure that a piece is only played onto another player's piece if the other player's piece the only piece on that space.
        if to.player == !play.player && to.count > 1 {
            return Err(Error::PlayMadeOntoOpposingPiece);
        }

        // Ensure play is possible from the dice rolls.
        let len = to.distance(&from) as u8;
        if !self.current_roll.check(len) {
            return Err(Error::InvalidPlayLength(len));
        }

        Ok(())
    }

    pub(super) fn make_play(&mut self, play: &Play) {
        let mut to = play.to.point(&self.board).borrow_mut();
        let mut from = play.from.point(&self.board).borrow_mut();

        // Remove possible play from the dice rolls.
        let len = to.distance(&from) as u8;
        self.current_roll.remove(len);

        // If there is a blot where the player is moving to, then send it to their bar.
        if to.player == !play.player && to.count == 1 {
            self.board.bar(to.player).borrow_mut().count += 1;
        }

        // Make the play.
        from.count -= 1;
        to.player = play.player;
        to.count += 1;

        // Reset the player of the previous position if it is empty and not from the bar
        if from.count == 0 && !matches!(play.from, Position::Bar(_)) {
            from.player = Player::None;
        }
    }

    fn change_turn(&mut self) {
        self.current_roll.reroll();
        self.current_player.switch();
    }

    fn get_available_plays<'a>(&'a self) -> impl Iterator<Item = Play> + 'a {
        let board_iter = (0..BOARD_SIZE)
            .map(|index| Position::Point(index))
            .chain([Position::Bar(Player::Black), Position::Bar(Player::White)])
            .map(|x| x.clone());

        board_iter.flat_map(move |board_position| {
            let rolls_iter = self
                .current_roll
                .available_rolls()
                .map(|x| x.clone())
                .collect::<Vec<_>>()
                .into_iter();

            rolls_iter
                .map(|roll| {
                    let from = board_position.clone();

                    let point = board_position.point(&self.board).borrow();
                    if self.current_player != point.player {
                        return Err(Error::PlayMadeOutOfTurn);
                    }

                    let play = match point.player {
                        Player::Black => match point.position.checked_sub(roll as usize + 1) {
                            Some(index) if index < BOARD_SIZE => {
                                let to = Position::Point(index);
                                Play::new(point.player, from, to)
                            }
                            _ => return Err(Error::InvalidNotationPosition(point.position)),
                        },
                        Player::White => match point.position.checked_add(roll as usize - 1) {
                            Some(index) if index < BOARD_SIZE => {
                                let to = Position::Point(index);
                                Play::new(point.player, from, to)
                            }
                            _ => return Err(Error::InvalidNotationPosition(point.position)),
                        },
                        _ => return Err(Error::PlayMadeOutOfTurn),
                    };
                    drop(point);
                    println!("{}", play);
                    if let Err(error) = self.check_play(&play) {
                        return Err(error);
                    }

                    Ok::<_, Error>(play)
                })
                .flatten()
                .collect::<Vec<_>>()
        })
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn black_1() {
        let player = Player::Black;
        let board = Board::empty();
        board.point(10).borrow_mut().set(5, player);

        let mut game = Game::from(player, Dice::from([3, 5]), board);

        let turn = Turn(vec![
            Play::new(player, Position::Point(10), Position::Point(7)),
            Play::new(player, Position::Point(10), Position::Point(5)),
        ]);

        println!("{game}");

        assert!(matches!(game.take_turn(turn), Ok(())));

        println!("{game}");

        let board = Board::empty();
        board.point(10).borrow_mut().set(3, player);
        board.point(7).borrow_mut().set(1, player);
        board.point(5).borrow_mut().set(1, player);

        println!("{board}");

        assert_eq!(game.board, board);
    }

    #[test]
    fn black_2() {
        let player = Player::Black;
        let board = Board::empty();
        board.point(10).borrow_mut().set(5, player);
        board.point(20).borrow_mut().set(3, player);
        board.point(4).borrow_mut().set(3, player);

        let mut game = Game::from(player, Dice::from([2, 6]), board);

        let turn = Turn(vec![
            Play::new(player, Position::Point(10), Position::Point(4)),
            Play::new(player, Position::Point(20), Position::Point(18)),
        ]);

        println!("{game}");

        assert!(matches!(game.take_turn(turn), Ok(())));

        println!("{game}");

        let board = Board::empty();
        board.point(10).borrow_mut().set(4, player);
        board.point(20).borrow_mut().set(2, player);
        board.point(18).borrow_mut().set(1, player);
        board.point(4).borrow_mut().set(4, player);

        println!("{board}");

        assert_eq!(game.board, board);
    }

    #[test]
    fn black_3() {
        let player = Player::Black;
        let board = Board::empty();
        board.point(15).borrow_mut().set(7, player);

        let mut game = Game::from(player, Dice::from([1, 3]), board);

        let turn = Turn(vec![
            Play::new(player, Position::Point(15), Position::Point(12)),
            Play::new(player, Position::Point(12), Position::Point(11)),
        ]);

        println!("{game}");

        assert!(matches!(game.take_turn(turn), Ok(())));

        println!("{game}");

        let board = Board::empty();
        board.point(15).borrow_mut().set(6, player);
        board.point(11).borrow_mut().set(1, player);

        println!("{board}");

        assert_eq!(game.board, board);
    }

    #[test]
    fn black_from_bar_1() {
        let player = Player::Black;
        let board = Board::empty();
        board.bar(player).borrow_mut().set(1, player);
        board.point(7).borrow_mut().set(2, player);

        let mut game = Game::from(player, Dice::from([4, 6]), board);

        let turn = Turn(vec![
            Play::new(player, Position::Bar(player), Position::Point(18)),
            Play::new(player, Position::Point(7), Position::Point(3)),
        ]);

        println!("{game}");

        assert!(matches!(game.take_turn(turn), Ok(())));

        println!("{game}");

        let board = Board::empty();
        board.point(18).borrow_mut().set(1, player);
        board.point(7).borrow_mut().set(1, player);
        board.point(3).borrow_mut().set(1, player);

        println!("{board}");

        assert_eq!(game.board, board);
    }

    #[test]
    fn black_from_bar_2() {
        let player = Player::Black;
        let board = Board::empty();
        board.bar(player).borrow_mut().set(2, player);
        board.point(23).borrow_mut().set(2, player);
        board.point(4).borrow_mut().set(8, player);

        let mut game = Game::from(player, Dice::from([1, 2]), board);

        let turn = Turn(vec![
            Play::new(player, Position::Bar(player), Position::Point(23)),
            Play::new(player, Position::Bar(player), Position::Point(22)),
        ]);

        println!("{game}");

        assert!(matches!(game.take_turn(turn), Ok(())));

        println!("{game}");

        let board = Board::empty();
        board.point(23).borrow_mut().set(3, player);
        board.point(22).borrow_mut().set(1, player);
        board.point(4).borrow_mut().set(8, player);

        println!("{board}");

        assert_eq!(game.board, board);
    }

    #[test]
    fn black_doubles_1() {
        let player = Player::Black;
        let board = Board::empty();
        board.point(17).borrow_mut().set(2, player);
        board.point(5).borrow_mut().set(8, player);

        let mut game = Game::from(player, Dice::from([3, 3]), board);

        let turn = Turn(vec![
            Play::new(player, Position::Point(17), Position::Point(14)),
            Play::new(player, Position::Point(17), Position::Point(14)),
            Play::new(player, Position::Point(14), Position::Point(11)),
            Play::new(player, Position::Point(5), Position::Point(2)),
        ]);

        println!("{game}");

        assert!(matches!(game.take_turn(turn), Ok(())));

        println!("{game}");

        let board = Board::empty();
        board.point(14).borrow_mut().set(1, player);
        board.point(11).borrow_mut().set(1, player);
        board.point(5).borrow_mut().set(7, player);
        board.point(2).borrow_mut().set(1, player);

        println!("{board}");

        assert_eq!(game.board, board);
    }

    #[test]
    fn white_1() {
        let player = Player::White;
        let board = Board::empty();
        board.point(11).borrow_mut().set(5, player);

        let mut game = Game::from(player, Dice::from([2, 3]), board);

        let turn = Turn(vec![
            Play::new(player, Position::Point(11), Position::Point(13)),
            Play::new(player, Position::Point(11), Position::Point(14)),
        ]);

        println!("{game}");

        assert!(matches!(game.take_turn(turn), Ok(())));

        println!("{game}");

        let board = Board::empty();
        board.point(11).borrow_mut().set(3, player);
        board.point(13).borrow_mut().set(1, player);
        board.point(14).borrow_mut().set(1, player);

        println!("{board}");

        assert_eq!(game.board, board);
    }

    #[test]
    fn white_2() {
        let player = Player::White;
        let board = Board::empty();
        board.point(20).borrow_mut().set(5, player);
        board.point(0).borrow_mut().set(3, player);
        board.point(5).borrow_mut().set(3, player);

        let mut game = Game::from(player, Dice::from([3, 5]), board);

        let turn = Turn(vec![
            Play::new(player, Position::Point(20), Position::Point(23)),
            Play::new(player, Position::Point(0), Position::Point(5)),
        ]);

        println!("{game}");

        assert!(matches!(game.take_turn(turn), Ok(())));

        println!("{game}");

        let board = Board::empty();
        board.point(20).borrow_mut().set(4, player);
        board.point(0).borrow_mut().set(2, player);
        board.point(23).borrow_mut().set(1, player);
        board.point(5).borrow_mut().set(4, player);

        println!("{board}");

        assert_eq!(game.board, board);
    }

    #[test]
    fn white_3() {
        let player = Player::White;
        let board = Board::empty();
        board.point(14).borrow_mut().set(4, player);

        let mut game = Game::from(player, Dice::from([1, 3]), board);

        let turn = Turn(vec![
            Play::new(player, Position::Point(14), Position::Point(15)),
            Play::new(player, Position::Point(15), Position::Point(18)),
        ]);

        println!("{game}");

        assert!(matches!(game.take_turn(turn), Ok(())));

        println!("{game}");

        let board = Board::empty();
        board.point(14).borrow_mut().set(3, player);
        board.point(18).borrow_mut().set(1, player);

        println!("{board}");

        assert_eq!(game.board, board);
    }

    #[test]
    fn white_from_bar_1() {
        let player = Player::White;
        let board = Board::empty();
        board.bar(player).borrow_mut().set(1, player);
        board.point(17).borrow_mut().set(2, player);

        let mut game = Game::from(player, Dice::from([6, 4]), board);

        let turn = Turn(vec![
            Play::new(player, Position::Bar(player), Position::Point(5)),
            Play::new(player, Position::Point(17), Position::Point(21)),
        ]);

        println!("{game}");

        assert!(matches!(game.take_turn(turn), Ok(())));

        println!("{game}");

        let board = Board::empty();
        board.point(17).borrow_mut().set(1, player);
        board.point(21).borrow_mut().set(1, player);
        board.point(5).borrow_mut().set(1, player);

        println!("{board}");

        assert_eq!(game.board, board);
    }

    #[test]
    fn white_from_bar_2() {
        let player = Player::White;
        let board = Board::empty();
        board.bar(player).borrow_mut().set(2, player);
        board.point(23).borrow_mut().set(2, player);
        board.point(3).borrow_mut().set(8, player);

        let mut game = Game::from(player, Dice::from([4, 1]), board);

        let turn = Turn(vec![
            Play::new(player, Position::Bar(player), Position::Point(3)),
            Play::new(player, Position::Bar(player), Position::Point(0)),
        ]);

        println!("{game}");

        assert!(matches!(game.take_turn(turn), Ok(())));

        println!("{game}");

        let board = Board::empty();
        board.point(23).borrow_mut().set(2, player);
        board.point(3).borrow_mut().set(9, player);
        board.point(0).borrow_mut().set(1, player);

        println!("{board}");

        assert_eq!(game.board, board);
    }

    #[test]
    fn white_doubles_1() {
        let player = Player::White;
        let board = Board::empty();
        board.point(7).borrow_mut().set(10, player);
        board.point(15).borrow_mut().set(3, player);
        board.point(17).borrow_mut().set(3, player);

        let mut game = Game::from(player, Dice::from([4, 4]), board);

        let turn = Turn(vec![
            Play::new(player, Position::Point(7), Position::Point(11)),
            Play::new(player, Position::Point(7), Position::Point(11)),
            Play::new(player, Position::Point(11), Position::Point(15)),
            Play::new(player, Position::Point(17), Position::Point(21)),
        ]);

        println!("{game}");

        assert!(matches!(game.take_turn(turn), Ok(())));

        println!("{game}");

        let board = Board::empty();
        board.point(7).borrow_mut().set(8, player);
        board.point(11).borrow_mut().set(1, player);
        board.point(15).borrow_mut().set(4, player);
        board.point(17).borrow_mut().set(2, player);
        board.point(21).borrow_mut().set(1, player);

        println!("{board}");

        assert_eq!(game.board, board);
    }
}
