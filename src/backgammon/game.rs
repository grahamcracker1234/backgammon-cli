use colored::Colorize;

use crate::backgammon::{
    board::{Board, Space, BOARD_SIZE},
    dice::Dice,
    notation::{Notation, Play, Turn},
    player::Player,
    position::IndexPosition,
    Error,
};

use std::{io, io::Write};

#[derive(Clone)]
pub struct Game {
    pub(super) current_player: Player,
    pub(super) current_roll: Dice,
    pub(super) board: Board,
}

impl Game {
    #[must_use]
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

            let notation = match self.get_notation() {
                Ok(notation) => notation,
                Err(error) => {
                    println!("{}", error.to_string().red().bold());
                    continue;
                }
            };

            let turn = match notation.turn() {
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
    fn get_notation(&self) -> io::Result<Notation> {
        let prompt = format!("{} to play ({}): ", self.current_player, self.current_roll);
        print!("{}", prompt.green().italic());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        Ok(Notation::new(input, self.current_player))
    }

    pub(crate) fn take_turn(&mut self, turn: Turn) -> Result<(), Error> {
        let Turn(plays) = turn;
        for play in plays {
            self.check_play(&play)?;
            self.make_play(&play);
        }

        if self.current_roll.any_available() && self.get_available_plays().count() > 0 {
            return Err(Error::IncompleteTurn);
        }

        Ok(())
    }

    /// Checks whether a given play can be made, returning a result containing the error if
    pub(crate) fn check_play(&self, play: &Play) -> Result<(), Error> {
        // Ensure current player is playing.
        if self.current_player != play.player {
            return Err(Error::PlayMadeOutOfTurn);
        }

        // Ensure that if there is a piece in the bar it is played.
        if self.board.bar(play.player).count > 0 && !matches!(play.from, Space::Bar(_)) {
            return Err(Error::PlayMadeWithBarFilled);
        }

        // Ensure that piece is not taken from the rail.
        if matches!(play.from, Space::Rail(_)) {
            return Err(Error::PlayMadeFromRail);
        }

        // Ensure that piece is not going to the bar.
        if matches!(play.to, Space::Bar(_)) {
            return Err(Error::PlayMadeToBar);
        }

        // Ensure a piece is only borne off if all piece are in the home table
        if matches!(play.to, Space::Rail(_)) && !self.board.all_in_home(play.player) {
            return Err(Error::InvalidBearOff);
        }

        let from = play.from.point(&self.board);
        let to = play.to.point(&self.board);

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
        let len = to.distance(from).try_into().expect("value was truncated");
        if !self.current_roll.check(len) {
            // Ensure a piece can be borne off with a greater roll than necessary only if there are no pieces behind it.
            let Space::Point(index) = play.from else {
                panic!("osdjfo");
            };

            if !matches!(play.to, Space::Rail(_)) || self.board.any_behind(*index, play.player) {
                return Err(Error::InvalidPlayLength(len));
            }
        }

        Ok(())
    }

    pub(super) fn make_play(&mut self, play: &Play) {
        // let board_borrow = &self.board;
        let to = play.to.point(&self.board);
        let from = play.from.point(&self.board);

        // Remove possible play from the dice rolls ensuring that the proper die
        // is removed if a piece was borne off with a greater than necessary roll.
        let len = to.distance(from).try_into().expect("value was truncated");
        // let len = to.distance(&from).try_into().expect("value was truncated");
        if self.current_roll.check(len) {
            self.current_roll.remove(len);
        } else {
            // Ensure a piece can be borne off with a greater roll than necessary
            //only if there are no pieces behind it.
            let Space::Point(index) = play.from else {
                panic!("osdjfo");
            };

            if matches!(play.to, Space::Rail(_)) && !self.board.any_behind(*index, play.player) {
                self.current_roll.remove(self.current_roll.max());
            }
        }

        // If there is a blot where the player is moving to, then send it to
        // their bar.
        if to.player == !play.player && to.count == 1 {
            let player = to.player;
            let bar = self.board.bar_mut(player);
            bar.count += 1;
        }

        // Make the play.
        let from = play.from.point_mut(&mut self.board);
        from.count -= 1;

        let to = play.to.point_mut(&mut self.board);
        to.player = play.player;
        to.count += 1;

        // Reset the player of the previous position if it is empty and not from
        // the bar
        let from = play.from.point(&self.board);
        if from.count == 0 && !matches!(play.from, Space::Bar(_)) {
            play.from.point_mut(&mut self.board).player = Player::None;
        }
    }

    fn change_turn(&mut self) {
        self.current_roll.reroll();
        self.current_player.switch();
    }

    fn get_available_plays(&self) -> impl Iterator<Item = Play> + '_ {
        fn board_iter(board: &Board, player: Player) -> Box<dyn Iterator<Item = Space> + '_> {
            if board.bar(player).count > 0 {
                Box::new([Space::Bar(player)].into_iter())
            } else {
                Box::new(
                    (0..BOARD_SIZE)
                        .map(|i| Space::Point(IndexPosition::try_from(i).unwrap()))
                        .filter(move |p| p.point(board).player == player),
                )
            }
        }

        board_iter(&self.board, self.current_player).flat_map(move |board_position| {
            let rolls_iter = self
                .current_roll
                .available_rolls()
                .collect::<Vec<_>>()
                .into_iter();

            rolls_iter
                .flat_map(|roll| {
                    let from = board_position.clone();

                    let point = board_position.point(&self.board);
                    // if self.current_player != point.player {
                    //     return Err(Error::PlayMadeOutOfTurn);
                    // }
                    // let index = point.position.to_index()?;
                    // let to = Space::Point(index);
                    // let play = Play::new(point.player, from, to);

                    let index = IndexPosition::try_from(match point.player {
                        Player::Black => point
                            .position
                            .checked_sub(roll as usize + 1)
                            .ok_or(Error::InvalidIndexPosition(69)),
                        Player::White => point
                            .position
                            .checked_add(roll as usize - 1)
                            .ok_or(Error::InvalidIndexPosition(69)),
                        Player::None => Err(Error::PlayMadeOutOfTurn),
                    }?)?;
                    let to = Space::Point(index);
                    let play = Play::new(point.player, from, to);

                    self.check_play(&play)?;

                    Ok::<_, Error>(play)
                })
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

impl Default for Game {
    fn default() -> Self {
        Game::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn black_1() {
        let player = Player::Black;
        let mut board = Board::empty();
        board.point_mut(10).set(5, player);

        let mut game = Game::from(player, Dice::from([3, 5]), board);

        let turn = Turn(vec![
            Play::new(
                player,
                Space::Point(10.try_into().unwrap()),
                Space::Point(7.try_into().unwrap()),
            ),
            Play::new(
                player,
                Space::Point(10.try_into().unwrap()),
                Space::Point(5.try_into().unwrap()),
            ),
        ]);

        println!("{game}");

        assert!(matches!(game.take_turn(turn), Ok(())));

        println!("{game}");

        let mut board = Board::empty();
        board.point_mut(10).set(3, player);
        board.point_mut(7).set(1, player);
        board.point_mut(5).set(1, player);

        println!("{board}");

        assert_eq!(game.board, board);
    }

    #[test]
    fn black_2() {
        let player = Player::Black;
        let mut board = Board::empty();
        board.point_mut(10).set(5, player);
        board.point_mut(20).set(3, player);
        board.point_mut(4).set(3, player);

        let mut game = Game::from(player, Dice::from([2, 6]), board);

        let turn = Turn(vec![
            Play::new(
                player,
                Space::Point(10.try_into().unwrap()),
                Space::Point(4.try_into().unwrap()),
            ),
            Play::new(
                player,
                Space::Point(20.try_into().unwrap()),
                Space::Point(18.try_into().unwrap()),
            ),
        ]);

        println!("{game}");

        assert!(matches!(game.take_turn(turn), Ok(())));

        println!("{game}");

        let mut board = Board::empty();
        board.point_mut(10).set(4, player);
        board.point_mut(20).set(2, player);
        board.point_mut(18).set(1, player);
        board.point_mut(4).set(4, player);

        println!("{board}");

        assert_eq!(game.board, board);
    }

    #[test]
    fn black_3() {
        let player = Player::Black;
        let mut board = Board::empty();
        board.point_mut(15).set(7, player);

        let mut game = Game::from(player, Dice::from([1, 3]), board);

        let turn = Turn(vec![
            Play::new(
                player,
                Space::Point(15.try_into().unwrap()),
                Space::Point(12.try_into().unwrap()),
            ),
            Play::new(
                player,
                Space::Point(12.try_into().unwrap()),
                Space::Point(11.try_into().unwrap()),
            ),
        ]);

        println!("{game}");

        assert!(matches!(game.take_turn(turn), Ok(())));

        println!("{game}");

        let mut board = Board::empty();
        board.point_mut(15).set(6, player);
        board.point_mut(11).set(1, player);

        println!("{board}");

        assert_eq!(game.board, board);
    }

    #[test]
    fn black_from_bar_1() {
        let player = Player::Black;
        let mut board = Board::empty();
        board.bar_mut(player).set(1, player);
        board.point_mut(7).set(2, player);

        let mut game = Game::from(player, Dice::from([4, 6]), board);

        let turn = Turn(vec![
            Play::new(
                player,
                Space::Bar(player),
                Space::Point(18.try_into().unwrap()),
            ),
            Play::new(
                player,
                Space::Point(7.try_into().unwrap()),
                Space::Point(3.try_into().unwrap()),
            ),
        ]);

        println!("{game}");

        assert!(matches!(game.take_turn(turn), Ok(())));

        println!("{game}");

        let mut board = Board::empty();
        board.point_mut(18).set(1, player);
        board.point_mut(7).set(1, player);
        board.point_mut(3).set(1, player);

        println!("{board}");

        assert_eq!(game.board, board);
    }

    #[test]
    fn black_from_bar_2() {
        let player = Player::Black;
        let mut board = Board::empty();
        board.bar_mut(player).set(2, player);
        board.point_mut(23).set(2, player);
        board.point_mut(4).set(8, player);

        let mut game = Game::from(player, Dice::from([1, 2]), board);

        let turn = Turn(vec![
            Play::new(
                player,
                Space::Bar(player),
                Space::Point(23.try_into().unwrap()),
            ),
            Play::new(
                player,
                Space::Bar(player),
                Space::Point(22.try_into().unwrap()),
            ),
        ]);

        println!("{game}");

        assert!(matches!(game.take_turn(turn), Ok(())));

        println!("{game}");

        let mut board = Board::empty();
        board.point_mut(23).set(3, player);
        board.point_mut(22).set(1, player);
        board.point_mut(4).set(8, player);

        println!("{board}");

        assert_eq!(game.board, board);
    }

    #[test]
    fn black_doubles_1() {
        let player = Player::Black;
        let mut board = Board::empty();
        board.point_mut(17).set(2, player);
        board.point_mut(5).set(8, player);

        let mut game = Game::from(player, Dice::from([3, 3]), board);

        let turn = Turn(vec![
            Play::new(
                player,
                Space::Point(17.try_into().unwrap()),
                Space::Point(14.try_into().unwrap()),
            ),
            Play::new(
                player,
                Space::Point(17.try_into().unwrap()),
                Space::Point(14.try_into().unwrap()),
            ),
            Play::new(
                player,
                Space::Point(14.try_into().unwrap()),
                Space::Point(11.try_into().unwrap()),
            ),
            Play::new(
                player,
                Space::Point(5.try_into().unwrap()),
                Space::Point(2.try_into().unwrap()),
            ),
        ]);

        println!("{game}");

        assert!(matches!(game.take_turn(turn), Ok(())));

        println!("{game}");

        let mut board = Board::empty();
        board.point_mut(14).set(1, player);
        board.point_mut(11).set(1, player);
        board.point_mut(5).set(7, player);
        board.point_mut(2).set(1, player);

        println!("{board}");

        assert_eq!(game.board, board);
    }

    #[test]
    fn white_1() {
        let player = Player::White;
        let mut board = Board::empty();
        board.point_mut(11).set(5, player);

        let mut game = Game::from(player, Dice::from([2, 3]), board);

        let turn = Turn(vec![
            Play::new(
                player,
                Space::Point(11.try_into().unwrap()),
                Space::Point(13.try_into().unwrap()),
            ),
            Play::new(
                player,
                Space::Point(11.try_into().unwrap()),
                Space::Point(14.try_into().unwrap()),
            ),
        ]);

        println!("{game}");

        assert!(matches!(game.take_turn(turn), Ok(())));

        println!("{game}");

        let mut board = Board::empty();
        board.point_mut(11).set(3, player);
        board.point_mut(13).set(1, player);
        board.point_mut(14).set(1, player);

        println!("{board}");

        assert_eq!(game.board, board);
    }

    #[test]
    fn white_2() {
        let player = Player::White;
        let mut board = Board::empty();
        board.point_mut(20).set(5, player);
        board.point_mut(0).set(3, player);
        board.point_mut(5).set(3, player);

        let mut game = Game::from(player, Dice::from([3, 5]), board);

        let turn = Turn(vec![
            Play::new(
                player,
                Space::Point(20.try_into().unwrap()),
                Space::Point(23.try_into().unwrap()),
            ),
            Play::new(
                player,
                Space::Point(0.try_into().unwrap()),
                Space::Point(5.try_into().unwrap()),
            ),
        ]);

        println!("{game}");

        assert!(matches!(game.take_turn(turn), Ok(())));

        println!("{game}");

        let mut board = Board::empty();
        board.point_mut(20).set(4, player);
        board.point_mut(0).set(2, player);
        board.point_mut(23).set(1, player);
        board.point_mut(5).set(4, player);

        println!("{board}");

        assert_eq!(game.board, board);
    }

    #[test]
    fn white_3() {
        let player = Player::White;
        let mut board = Board::empty();
        board.point_mut(14).set(4, player);

        let mut game = Game::from(player, Dice::from([1, 3]), board);

        let turn = Turn(vec![
            Play::new(
                player,
                Space::Point(14.try_into().unwrap()),
                Space::Point(15.try_into().unwrap()),
            ),
            Play::new(
                player,
                Space::Point(15.try_into().unwrap()),
                Space::Point(18.try_into().unwrap()),
            ),
        ]);

        println!("{game}");

        assert!(matches!(game.take_turn(turn), Ok(())));

        println!("{game}");

        let mut board = Board::empty();
        board.point_mut(14).set(3, player);
        board.point_mut(18).set(1, player);

        println!("{board}");

        assert_eq!(game.board, board);
    }

    #[test]
    fn white_from_bar_1() {
        let player = Player::White;
        let mut board = Board::empty();
        board.bar_mut(player).set(1, player);
        board.point_mut(17).set(2, player);

        let mut game = Game::from(player, Dice::from([6, 4]), board);

        let turn = Turn(vec![
            Play::new(
                player,
                Space::Bar(player),
                Space::Point(5.try_into().unwrap()),
            ),
            Play::new(
                player,
                Space::Point(17.try_into().unwrap()),
                Space::Point(21.try_into().unwrap()),
            ),
        ]);

        println!("{game}");

        assert!(matches!(game.take_turn(turn), Ok(())));

        println!("{game}");

        let mut board = Board::empty();
        board.point_mut(17).set(1, player);
        board.point_mut(21).set(1, player);
        board.point_mut(5).set(1, player);

        println!("{board}");

        assert_eq!(game.board, board);
    }

    #[test]
    fn white_from_bar_2() {
        let player = Player::White;
        let mut board = Board::empty();
        board.bar_mut(player).set(2, player);
        board.point_mut(23).set(2, player);
        board.point_mut(3).set(8, player);

        let mut game = Game::from(player, Dice::from([4, 1]), board);

        let turn = Turn(vec![
            Play::new(
                player,
                Space::Bar(player),
                Space::Point(3.try_into().unwrap()),
            ),
            Play::new(
                player,
                Space::Bar(player),
                Space::Point(0.try_into().unwrap()),
            ),
        ]);

        println!("{game}");

        assert!(matches!(game.take_turn(turn), Ok(())));

        println!("{game}");

        let mut board = Board::empty();
        board.point_mut(23).set(2, player);
        board.point_mut(3).set(9, player);
        board.point_mut(0).set(1, player);

        println!("{board}");

        assert_eq!(game.board, board);
    }

    #[test]
    fn white_doubles_1() {
        let player = Player::White;
        let mut board = Board::empty();
        board.point_mut(7).set(10, player);
        board.point_mut(15).set(3, player);
        board.point_mut(17).set(3, player);

        let mut game = Game::from(player, Dice::from([4, 4]), board);

        let turn = Turn(vec![
            Play::new(
                player,
                Space::Point(7.try_into().unwrap()),
                Space::Point(11.try_into().unwrap()),
            ),
            Play::new(
                player,
                Space::Point(7.try_into().unwrap()),
                Space::Point(11.try_into().unwrap()),
            ),
            Play::new(
                player,
                Space::Point(11.try_into().unwrap()),
                Space::Point(15.try_into().unwrap()),
            ),
            Play::new(
                player,
                Space::Point(17.try_into().unwrap()),
                Space::Point(21.try_into().unwrap()),
            ),
        ]);

        println!("{game}");

        assert!(matches!(game.take_turn(turn), Ok(())));

        println!("{game}");

        let mut board = Board::empty();
        board.point_mut(7).set(8, player);
        board.point_mut(11).set(1, player);
        board.point_mut(15).set(4, player);
        board.point_mut(17).set(2, player);
        board.point_mut(21).set(1, player);

        println!("{board}");

        assert_eq!(game.board, board);
    }

    #[test]
    fn bear_off_1() {
        let player = Player::Black;
        let mut board = Board::empty();
        board.point_mut(4).set(3, player);

        let mut game = Game::from(player, Dice::from([5, 4]), board);

        let turn = Turn(vec![
            Play::new(
                player,
                Space::Point(4.try_into().unwrap()),
                Space::Rail(player),
            ),
            Play::new(
                player,
                Space::Point(4.try_into().unwrap()),
                Space::Point(0.try_into().unwrap()),
            ),
        ]);

        println!("{game}");

        assert_eq!(game.take_turn(turn), Ok(()));

        println!("{game}");

        let mut board = Board::empty();
        board.point_mut(4).set(1, player);
        board.rail_mut(player).set(1, player);
        board.point_mut(0).set(1, player);

        println!("{board}");

        assert_eq!(game.board, board);
    }

    #[test]
    fn bear_off_2() {
        let player = Player::White;
        let mut board = Board::empty();
        board.point_mut(21).set(3, player);
        board.point_mut(22).set(3, player);

        let mut game = Game::from(player, Dice::from([2, 3]), board);

        let turn = Turn(vec![
            Play::new(
                player,
                Space::Point(21.try_into().unwrap()),
                Space::Rail(player),
            ),
            Play::new(
                player,
                Space::Point(22.try_into().unwrap()),
                Space::Rail(player),
            ),
        ]);

        println!("{game}");

        assert_eq!(game.take_turn(turn), Ok(()));

        println!("{game}");

        let mut board = Board::empty();
        board.point_mut(21).set(2, player);
        board.point_mut(22).set(2, player);
        board.rail_mut(player).set(2, player);

        println!("{board}");

        assert_eq!(game.board, board);
    }

    #[test]
    fn bear_off_3() {
        let player = Player::White;
        let mut board = Board::empty();
        board.point_mut(18).set(3, player);

        let mut game = Game::from(player, Dice::from([6, 5]), board);

        let turn = Turn(vec![
            Play::new(
                player,
                Space::Point(18.try_into().unwrap()),
                Space::Rail(player),
            ),
            Play::new(
                player,
                Space::Point(18.try_into().unwrap()),
                Space::Rail(player),
            ),
        ]);

        println!("{game}");

        assert_eq!(game.take_turn(turn), Ok(()));

        println!("{game}");

        let mut board = Board::empty();
        board.point_mut(18).set(1, player);
        board.rail_mut(player).set(2, player);

        println!("{board}");

        assert_eq!(game.board, board);
    }

    #[test]
    fn no_moves_1() {
        let player = Player::Black;
        let mut board = Board::empty();
        board.point_mut(10).set(2, Player::White);
        board.point_mut(11).set(2, Player::White);
        board.point_mut(13).set(2, player);

        let mut game = Game::from(player, Dice::from([2, 3]), board);

        let turn = Turn(vec![]);

        println!("{game}");

        assert_eq!(game.take_turn(turn), Ok(()));
    }

    #[test]
    fn no_moves_2() {
        let player = Player::Black;
        let mut board = Board::empty();
        board.bar_mut(player).set(1, player);
        board.point_mut(23).set(2, Player::White);
        board.point_mut(20).set(2, Player::White);
        board.point_mut(13).set(1, player);

        let mut game = Game::from(player, Dice::from([1, 4]), board);

        let turn = Turn(vec![]);

        println!("{game}");

        assert_eq!(game.take_turn(turn), Ok(()));
    }

    #[test]
    fn invalid_play_length_1() {
        let player = Player::Black;
        let mut board = Board::empty();
        board.point_mut(10).set(2, player);

        let mut game = Game::from(player, Dice::from([1, 2]), board);

        let turn = Turn(vec![
            Play::new(
                player,
                Space::Point(10.try_into().unwrap()),
                Space::Point(9.try_into().unwrap()),
            ),
            Play::new(
                player,
                Space::Point(10.try_into().unwrap()),
                Space::Point(7.try_into().unwrap()),
            ),
        ]);

        println!("{game}");

        assert_eq!(game.take_turn(turn), Err(Error::InvalidPlayLength(3)));
    }

    #[test]
    fn invalid_play_length_2() {
        let player = Player::Black;
        let mut board = Board::empty();
        board.point_mut(5).set(1, player);
        board.point_mut(4).set(2, player);
        board.point_mut(3).set(1, player);

        let mut game = Game::from(player, Dice::from([3, 6]), board);

        let turn = Turn(vec![
            Play::new(
                player,
                Space::Point(4.try_into().unwrap()),
                Space::Rail(player),
            ),
            Play::new(
                player,
                Space::Point(3.try_into().unwrap()),
                Space::Rail(player),
            ),
        ]);

        println!("{game}");

        assert_eq!(game.take_turn(turn), Err(Error::InvalidPlayLength(5)));
    }

    #[test]
    fn incomplete_turn() {
        let player = Player::Black;
        let mut board = Board::empty();
        board.point_mut(10).set(2, player);

        let mut game = Game::from(player, Dice::from([1, 2]), board);

        let turn = Turn(vec![Play::new(
            player,
            Space::Point(10.try_into().unwrap()),
            Space::Point(9.try_into().unwrap()),
        )]);

        println!("{game}");

        assert_eq!(game.take_turn(turn), Err(Error::IncompleteTurn));
    }

    #[test]
    fn play_made_out_of_turn() {
        let player = Player::Black;
        let mut board = Board::empty();
        board.point_mut(10).set(2, player);

        let mut game = Game::from(Player::White, Dice::from([1, 2]), board);

        let turn = Turn(vec![
            Play::new(
                player,
                Space::Point(10.try_into().unwrap()),
                Space::Point(9.try_into().unwrap()),
            ),
            Play::new(
                player,
                Space::Point(10.try_into().unwrap()),
                Space::Point(8.try_into().unwrap()),
            ),
        ]);

        println!("{game}");

        assert_eq!(game.take_turn(turn), Err(Error::PlayMadeOutOfTurn));
    }

    #[test]
    fn play_made_with_bar_filled() {
        let player = Player::Black;
        let mut board = Board::empty();
        board.bar_mut(player).set(1, player);
        board.point_mut(10).set(2, player);

        let mut game = Game::from(player, Dice::from([1, 2]), board);

        let turn = Turn(vec![
            Play::new(
                player,
                Space::Point(10.try_into().unwrap()),
                Space::Point(9.try_into().unwrap()),
            ),
            Play::new(
                player,
                Space::Point(10.try_into().unwrap()),
                Space::Point(8.try_into().unwrap()),
            ),
        ]);

        println!("{game}");

        assert_eq!(game.take_turn(turn), Err(Error::PlayMadeWithBarFilled));
    }

    #[test]
    fn play_made_to_bar_filled() {
        let player = Player::Black;
        let mut board = Board::empty();
        board.point_mut(23).set(2, player);

        let mut game = Game::from(player, Dice::from([1, 2]), board);

        let turn = Turn(vec![
            Play::new(
                player,
                Space::Point(23.try_into().unwrap()),
                Space::Bar(player),
            ),
            Play::new(
                player,
                Space::Point(23.try_into().unwrap()),
                Space::Point(21.try_into().unwrap()),
            ),
        ]);

        println!("{game}");

        assert_eq!(game.take_turn(turn), Err(Error::PlayMadeToBar));
    }

    #[test]
    fn play_made_from_rail_filled() {
        let player = Player::Black;
        let mut board = Board::empty();
        board.rail_mut(player).set(1, player);
        board.point_mut(23).set(2, player);

        let mut game = Game::from(player, Dice::from([1, 2]), board);

        let turn = Turn(vec![
            Play::new(
                player,
                Space::Rail(player),
                Space::Point(0.try_into().unwrap()),
            ),
            Play::new(
                player,
                Space::Point(23.try_into().unwrap()),
                Space::Point(21.try_into().unwrap()),
            ),
        ]);

        println!("{game}");

        assert_eq!(game.take_turn(turn), Err(Error::PlayMadeFromRail));
    }

    #[test]
    fn play_made_from_empty_point() {
        let player = Player::Black;
        let board = Board::empty();

        let mut game = Game::from(player, Dice::from([1, 2]), board);

        let turn = Turn(vec![Play::new(
            player,
            Space::Point(23.try_into().unwrap()),
            Space::Point(21.try_into().unwrap()),
        )]);

        println!("{game}");

        assert_eq!(game.take_turn(turn), Err(Error::PlayMadeFromEmptyPoint));
    }

    #[test]
    fn play_made_with_opposing_piece() {
        let player = Player::Black;
        let mut board = Board::empty();
        board.point_mut(23).set(2, Player::White);

        let mut game = Game::from(player, Dice::from([1, 2]), board);

        let turn = Turn(vec![
            Play::new(
                player,
                Space::Point(23.try_into().unwrap()),
                Space::Point(22.try_into().unwrap()),
            ),
            Play::new(
                player,
                Space::Point(23.try_into().unwrap()),
                Space::Point(21.try_into().unwrap()),
            ),
        ]);

        println!("{game}");

        assert_eq!(game.take_turn(turn), Err(Error::PlayMadeWithOpposingPiece));
    }

    #[test]
    fn invalid_play_direction() {
        let player = Player::Black;
        let mut board = Board::empty();
        board.point_mut(10).set(2, player);

        let mut game = Game::from(player, Dice::from([1, 2]), board);

        let turn = Turn(vec![
            Play::new(
                player,
                Space::Point(10.try_into().unwrap()),
                Space::Point(11.try_into().unwrap()),
            ),
            Play::new(
                player,
                Space::Point(10.try_into().unwrap()),
                Space::Point(12.try_into().unwrap()),
            ),
        ]);

        println!("{game}");

        assert_eq!(game.take_turn(turn), Err(Error::InvalidPlayDirection));
    }

    #[test]
    fn play_made_onto_opposing_piece() {
        let player = Player::Black;
        let mut board = Board::empty();
        board.point_mut(10).set(2, player);
        board.point_mut(8).set(2, Player::White);

        let mut game = Game::from(player, Dice::from([1, 2]), board);

        let turn = Turn(vec![
            Play::new(
                player,
                Space::Point(10.try_into().unwrap()),
                Space::Point(9.try_into().unwrap()),
            ),
            Play::new(
                player,
                Space::Point(10.try_into().unwrap()),
                Space::Point(8.try_into().unwrap()),
            ),
        ]);

        println!("{game}");

        assert_eq!(game.take_turn(turn), Err(Error::PlayMadeOntoOpposingPiece));
    }

    #[test]
    fn invalid_bear_off() {
        let player = Player::Black;
        let mut board = Board::empty();
        board.point_mut(6).set(1, player);
        board.point_mut(3).set(2, player);

        let mut game = Game::from(player, Dice::from([1, 3]), board);

        let turn = Turn(vec![
            Play::new(
                player,
                Space::Point(3.try_into().unwrap()),
                Space::Point(0.try_into().unwrap()),
            ),
            Play::new(
                player,
                Space::Point(0.try_into().unwrap()),
                Space::Rail(player),
            ),
        ]);

        println!("{game}");

        assert_eq!(game.take_turn(turn), Err(Error::InvalidBearOff));
    }
}
