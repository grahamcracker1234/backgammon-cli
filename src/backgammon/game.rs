use colored::Colorize;

use crate::backgammon::{
    board::{Board, BOARD_SIZE},
    dice::Dice,
    location::IndexLocation,
    notation::{Notation, Play, PositionRef, Turn},
    player::Player,
    Error,
};

use std::{collections::HashSet, io, io::Write};

#[derive(Clone)]
pub struct Game {
    pub(crate) current_player: Player,
    pub(crate) current_roll: Dice,
    pub(crate) board: Board,
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

            if let Err(error) = self.check_turn(&turn) {
                println!("{}", error.to_string().red().bold());
                self.board = saved_board;
                self.current_roll = saved_roll;
                continue;
            }

            self.take_turn(&turn);
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

    pub(crate) fn check_turn(&self, turn: &Turn) -> Result<(), Error> {
        let mut game = self.clone();

        let Turn(plays) = turn;
        for play in plays {
            game.check_play(play)?;
            game.make_play(play);
        }

        if game.current_roll.any_available() && !game.get_available_plays().is_empty() {
            return Err(Error::IncompleteTurn);
        }

        Ok(())
    }

    pub(crate) fn take_turn(&mut self, turn: &Turn) {
        let Turn(plays) = turn;
        for play in plays {
            self.make_play(play);
        }
    }

    /// Checks whether a given play can be made, returning a result containing the error if
    pub(crate) fn check_play(&self, play: &Play) -> Result<(), Error> {
        // Ensure current player is playing.
        if self.current_player != play.player {
            return Err(Error::PlayMadeOutOfTurn);
        }

        // Ensure that if there is a piece in the bar it is played.
        if self.board.bar(play.player).count > 0 && !matches!(play.from, PositionRef::Bar(_)) {
            return Err(Error::PlayMadeWithBarFilled);
        }

        // Ensure that piece is not taken from the rail.
        if matches!(play.from, PositionRef::Rail(_)) {
            return Err(Error::PlayMadeFromRail);
        }

        // Ensure that piece is not going to the bar.
        if matches!(play.to, PositionRef::Bar(_)) {
            return Err(Error::PlayMadeToBar);
        }

        // Ensure a piece is only borne off if all piece are in the home table
        if matches!(play.to, PositionRef::Rail(_)) && !self.board.all_in_home(play.player) {
            return Err(Error::InvalidBearOff);
        }

        let from = self.board.get(&play.from);
        let to = self.board.get(&play.to);

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
            let PositionRef::Point(index) = play.from else {
                panic!("osdjfo");
            };

            if !matches!(play.to, PositionRef::Rail(_))
                || self.board.any_behind(*index, play.player)
            {
                return Err(Error::InvalidPlayLength(len));
            }
        }

        Ok(())
    }

    pub(super) fn make_play(&mut self, play: &Play) {
        // let board_borrow = &self.board;
        let from = self.board.get(&play.from);
        let to = self.board.get(&play.to);

        // Remove possible play from the dice rolls ensuring that the proper die
        // is removed if a piece was borne off with a greater than necessary roll.
        let len = to.distance(from).try_into().expect("value was truncated");
        // let len = to.distance(&from).try_into().expect("value was truncated");
        if self.current_roll.check(len) {
            self.current_roll.remove(len);
        } else {
            // Ensure a piece can be borne off with a greater roll than necessary
            // only if there are no pieces behind it.

            assert!(matches!(play.to, PositionRef::Rail(_)));

            let index = from
                .location
                .to_index()
                .expect("location should be indexable");

            assert!(!self.board.any_behind(*index, play.player));

            self.current_roll.remove(self.current_roll.max());
        }

        // If there is a blot where the player is moving to, then send it to
        // their bar.
        if to.player == !play.player && to.count == 1 {
            let player = to.player;
            let bar = self.board.bar_mut(player);
            bar.count += 1;
        }

        // Make the play.
        let from = self.board.get_mut(&play.from);
        from.count -= 1;

        let to = self.board.get_mut(&play.to);
        to.player = play.player;
        to.count += 1;

        // Reset the player of the previous position if it is empty and not from
        // the bar
        let from = self.board.get(&play.from);
        if from.count == 0 && !matches!(play.from, PositionRef::Bar(_)) {
            let from = self.board.get_mut(&play.from);
            from.player = Player::None;
        }
    }

    fn change_turn(&mut self) {
        self.current_roll.reroll();
        self.current_player.switch();
    }

    fn get_available_plays(&self) -> HashSet<Play> {
        fn board_iter(board: &Board, player: Player) -> Box<dyn Iterator<Item = PositionRef> + '_> {
            if board.bar(player).count > 0 {
                Box::new([PositionRef::Bar(player)].into_iter())
            } else {
                Box::new(
                    (0..BOARD_SIZE)
                        .map(|i| PositionRef::Point(IndexLocation::try_from(i).unwrap()))
                        .filter(move |p| board.get(p).player == player),
                )
            }
        }

        board_iter(&self.board, self.current_player)
            .flat_map(move |board_position| {
                let rolls_iter = self
                    .current_roll
                    .available_rolls()
                    .collect::<Vec<_>>()
                    .into_iter();

                rolls_iter
                    .flat_map(|roll| {
                        let from = board_position.clone();

                        let point = self.board.get(&board_position);
                        let index = IndexLocation::try_from(match point.player {
                            Player::Black => point
                                .location
                                .checked_sub(roll as usize + 1)
                                .ok_or(Error::InvalidIndexLocation(69)),
                            Player::White => point
                                .location
                                .checked_add(roll as usize - 1)
                                .ok_or(Error::InvalidIndexLocation(69)),
                            Player::None => Err(Error::PlayMadeOutOfTurn),
                        }?)?;
                        let to = PositionRef::Point(index);
                        let play = Play::new(point.player, from, to);

                        self.check_play(&play)?;

                        Ok::<_, Error>(play)
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    fn get_available_turns(&self) -> Vec<Turn> {
        fn _get_available_turns(game: &Game) -> Vec<Vec<Play>> {
            let plays = game.get_available_plays();
            if plays.is_empty() {
                return vec![vec![]];
            }

            // .filter(|play| game.check_play(play).is_ok())
            plays
                .into_iter()
                .flat_map(|play| {
                    let mut game = game.clone();
                    game.make_play(&play);
                    let plays: Vec<Vec<Play>> = _get_available_turns(&game)
                        .into_iter()
                        .map(|mut next_plays| {
                            next_plays.splice(0..0, [play.clone()]);
                            next_plays
                        })
                        .collect();
                    plays
                })
                .collect()
        }
        // let mut game = self.clone();
        _get_available_turns(&self.clone())
            .into_iter()
            .map(Turn)
            .collect()

        // for play in game.get_available_plays() {
        //     if let Ok(_) = game.check_play(&play) {
        //         game.make_play(&play);
        //     }
        // }

        // if game.current_roll.any_available() && game.get_available_plays().count() > 0 {
        //     return Err(Error::IncompleteTurn);
        // }
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
    use crate::backgammon::notation::{plays, turn};

    #[test]
    fn black_1() {
        let player = Player::Black;
        let mut board = Board::empty();
        board.point_mut(10).set(5, player);

        let mut game = Game::from(player, Dice::from([3, 5]), board);
        let turn = turn!(player, (10, 7), (10, 5));

        println!("{game}");

        assert_eq!(game.check_turn(&turn), Ok(()));
        game.take_turn(&turn);

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
        let turn = turn!(player, (10, 4), (20, 18));

        println!("{game}");

        assert_eq!(game.check_turn(&turn), Ok(()));
        game.take_turn(&turn);

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
        let turn = turn!(player, (15, 12), (12, 11));

        println!("{game}");

        assert_eq!(game.check_turn(&turn), Ok(()));
        game.take_turn(&turn);

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
        let turn = turn!(player, (bar, 18), (7, 3));

        println!("{game}");

        assert_eq!(game.check_turn(&turn), Ok(()));
        game.take_turn(&turn);

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
        let turn = turn!(player, (bar, 23), (bar, 22));

        println!("{game}");

        assert_eq!(game.check_turn(&turn), Ok(()));
        game.take_turn(&turn);

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
        let turn = turn!(player, (17, 14), (17, 14), (14, 11), (5, 2));

        println!("{game}");

        assert_eq!(game.check_turn(&turn), Ok(()));
        game.take_turn(&turn);

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
        let turn = turn!(player, (11, 13), (11, 14));

        println!("{game}");

        assert_eq!(game.check_turn(&turn), Ok(()));
        game.take_turn(&turn);

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
        let turn = turn!(player, (20, 23), (0, 5));

        println!("{game}");

        assert_eq!(game.check_turn(&turn), Ok(()));
        game.take_turn(&turn);

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
        let turn = turn!(player, (14, 15), (15, 18));

        println!("{game}");

        assert_eq!(game.check_turn(&turn), Ok(()));
        game.take_turn(&turn);

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
        let turn = turn!(player, (bar, 5), (17, 21));

        println!("{game}");

        assert_eq!(game.check_turn(&turn), Ok(()));
        game.take_turn(&turn);

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
        let turn = turn!(player, (bar, 3), (bar, 0));

        println!("{game}");

        assert_eq!(game.check_turn(&turn), Ok(()));
        game.take_turn(&turn);

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
        let turn = turn!(player, (7, 11), (7, 11), (11, 15), (17, 21));

        println!("{game}");

        assert_eq!(game.check_turn(&turn), Ok(()));
        game.take_turn(&turn);

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
        let turn = turn!(player, (4, off), (4, 0));

        println!("{game}");

        assert_eq!(game.check_turn(&turn), Ok(()));
        game.take_turn(&turn);

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
        let turn = turn!(player, (21, off), (22, off));

        println!("{game}");

        assert_eq!(game.check_turn(&turn), Ok(()));
        game.take_turn(&turn);

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
        let turn = turn!(player, (18, off), (18, off));

        println!("{game}");

        assert_eq!(game.check_turn(&turn), Ok(()));
        game.take_turn(&turn);

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

        let game = Game::from(player, Dice::from([2, 3]), board);
        let turn = turn!(player);

        println!("{game}");

        assert_eq!(game.check_turn(&turn), Ok(()));
    }

    #[test]
    fn no_moves_2() {
        let player = Player::Black;
        let mut board = Board::empty();
        board.bar_mut(player).set(1, player);
        board.point_mut(23).set(2, Player::White);
        board.point_mut(20).set(2, Player::White);
        board.point_mut(13).set(1, player);

        let game = Game::from(player, Dice::from([1, 4]), board);
        let turn = turn!(player);

        println!("{game}");

        assert_eq!(game.check_turn(&turn), Ok(()));
    }

    #[test]
    fn invalid_play_length_1() {
        let player = Player::Black;
        let mut board = Board::empty();
        board.point_mut(10).set(2, player);

        let game = Game::from(player, Dice::from([1, 2]), board);
        let turn = turn!(player, (10, 9), (10, 7));

        println!("{game}");

        assert_eq!(game.check_turn(&turn), Err(Error::InvalidPlayLength(3)));
    }

    #[test]
    fn invalid_play_length_2() {
        let player = Player::Black;
        let mut board = Board::empty();
        board.point_mut(5).set(1, player);
        board.point_mut(4).set(2, player);
        board.point_mut(3).set(1, player);

        let game = Game::from(player, Dice::from([3, 6]), board);
        let turn = turn!(player, (4, off), (3, off));

        println!("{game}");

        assert_eq!(game.check_turn(&turn), Err(Error::InvalidPlayLength(5)));
    }

    #[test]
    fn incomplete_turn() {
        let player = Player::Black;
        let mut board = Board::empty();
        board.point_mut(10).set(2, player);

        let game = Game::from(player, Dice::from([1, 2]), board);
        let turn = turn!(player, (10, 9));

        println!("{game}");

        assert_eq!(game.check_turn(&turn), Err(Error::IncompleteTurn));
    }

    #[test]
    fn play_made_out_of_turn() {
        let player = Player::Black;
        let mut board = Board::empty();
        board.point_mut(10).set(2, player);

        let game = Game::from(Player::White, Dice::from([1, 2]), board);
        let turn = turn!(player, (10, 9), (10, 8));

        println!("{game}");

        assert_eq!(game.check_turn(&turn), Err(Error::PlayMadeOutOfTurn));
    }

    #[test]
    fn play_made_with_bar_filled() {
        let player = Player::Black;
        let mut board = Board::empty();
        board.bar_mut(player).set(1, player);
        board.point_mut(10).set(2, player);

        let game = Game::from(player, Dice::from([1, 2]), board);
        let turn = turn!(player, (10, 9), (10, 8));

        println!("{game}");

        assert_eq!(game.check_turn(&turn), Err(Error::PlayMadeWithBarFilled));
    }

    #[test]
    fn play_made_to_bar_filled() {
        let player = Player::Black;
        let mut board = Board::empty();
        board.point_mut(23).set(2, player);

        let game = Game::from(player, Dice::from([1, 2]), board);
        let turn = turn!(player, (23, bar), (23, 21));

        println!("{game}");

        assert_eq!(game.check_turn(&turn), Err(Error::PlayMadeToBar));
    }

    #[test]
    fn play_made_from_rail_filled() {
        let player = Player::Black;
        let mut board = Board::empty();
        board.rail_mut(player).set(1, player);
        board.point_mut(23).set(2, player);

        let game = Game::from(player, Dice::from([1, 2]), board);
        let turn = turn!(player, (off, 0), (23, 21));

        println!("{game}");

        assert_eq!(game.check_turn(&turn), Err(Error::PlayMadeFromRail));
    }

    #[test]
    fn play_made_from_empty_point() {
        let player = Player::Black;
        let board = Board::empty();

        let game = Game::from(player, Dice::from([1, 2]), board);
        let turn = turn!(player, (23, 21));

        println!("{game}");

        assert_eq!(game.check_turn(&turn), Err(Error::PlayMadeFromEmptyPoint));
    }

    #[test]
    fn play_made_with_opposing_piece() {
        let player = Player::Black;
        let mut board = Board::empty();
        board.point_mut(23).set(2, Player::White);

        let game = Game::from(player, Dice::from([1, 2]), board);
        let turn = turn!(player, (23, 22), (23, 21));

        println!("{game}");

        assert_eq!(
            game.check_turn(&turn),
            Err(Error::PlayMadeWithOpposingPiece)
        );
    }

    #[test]
    fn invalid_play_direction() {
        let player = Player::Black;
        let mut board = Board::empty();
        board.point_mut(10).set(2, player);

        let game = Game::from(player, Dice::from([1, 2]), board);
        let turn = turn!(player, (10, 11), (10, 12));

        println!("{game}");

        assert_eq!(game.check_turn(&turn), Err(Error::InvalidPlayDirection));
    }

    #[test]
    fn play_made_onto_opposing_piece() {
        let player = Player::Black;
        let mut board = Board::empty();
        board.point_mut(10).set(2, player);
        board.point_mut(8).set(2, Player::White);

        let game = Game::from(player, Dice::from([1, 2]), board);
        let turn = turn!(player, (10, 9), (10, 8));

        println!("{game}");

        assert_eq!(
            game.check_turn(&turn),
            Err(Error::PlayMadeOntoOpposingPiece)
        );
    }

    #[test]
    fn invalid_bear_off() {
        let player = Player::Black;
        let mut board = Board::empty();
        board.point_mut(6).set(1, player);
        board.point_mut(3).set(2, player);

        let game = Game::from(player, Dice::from([1, 3]), board);
        let turn = turn!(player, (3, 0), (0, off));

        println!("{game}");

        assert_eq!(game.check_turn(&turn), Err(Error::InvalidBearOff));
    }

    #[test]
    fn get_available_plays_1() {
        let player = Player::Black;
        let board = Board::new();
        let game = Game::from(player, Dice::from([2, 5]), board);
        let plays = plays!(player, (5, 3), (7, 2), (7, 5), (12, 7), (12, 10), (23, 21));

        println!("{game}");
        assert_eq!(plays, game.get_available_plays());
    }

    #[test]
    fn get_available_plays_2() {
        let player = Player::Black;
        let mut board = Board::empty();
        board.bar_mut(player).set(1, player);
        board.point_mut(10).set(3, player);

        let game = Game::from(player, Dice::from([4, 6]), board);
        let plays = plays!(player, (bar, 20), (bar, 18));

        println!("{game}");
        assert_eq!(plays, game.get_available_plays());
    }

    #[test]
    fn get_available_plays_3() {
        let player = Player::White;
        let mut board = Board::empty();
        board.bar_mut(player).set(1, player);
        board.point_mut(10).set(3, player);

        let game = Game::from(player, Dice::from([4, 6]), board);
        let plays = plays!(player, (bar, 3), (bar, 5));

        println!("{game}");
        assert_eq!(plays, game.get_available_plays());
    }

    #[test]
    fn get_available_plays_4() {
        let player = Player::White;
        let mut board = Board::empty();
        board.point_mut(10).set(1, player);
        board.point_mut(23).set(3, player);

        let game = Game::from(player, Dice::from([1, 4]), board);
        let plays = plays!(player, (10, 14), (10, 11));

        println!("{game}");
        assert_eq!(plays, game.get_available_plays());
    }

    #[test]
    fn get_available_plays_5() {
        let player = Player::White;
        let mut board = Board::empty();
        board.point_mut(10).set(1, player);
        board.point_mut(23).set(3, player);
        board.point_mut(11).set(2, !player);

        let game = Game::from(player, Dice::from([1, 4]), board);
        let plays = plays!(player, (10, 14));

        println!("{game}");
        assert_eq!(plays, game.get_available_plays());
    }

    #[test]
    fn get_available_plays_6() {
        let player = Player::Black;
        let mut board = Board::empty();
        board.bar_mut(player).set(1, player);
        board.point_mut(23).set(2, !player);
        board.point_mut(22).set(2, !player);
        board.point_mut(21).set(2, !player);
        board.point_mut(20).set(2, !player);
        board.point_mut(19).set(2, !player);
        board.point_mut(18).set(2, !player);

        let game = Game::from(player, Dice::from([6, 6]), board);
        let plays = plays!(player);

        println!("{game}");
        assert_eq!(plays, game.get_available_plays());
    }

    // #[test]
    // fn get_available_turns() {
    //     let player = Player::Black;
    //     let board = Board::new();
    //     let game = Game::from(player, Dice::from([2, 5]), board);
    //     println!("{game}");

    //     let turns = game.get_available_turns();
    //     for turn in turns {
    //         println!("{turn}");
    //     }

    //     panic!();
    // }
}
