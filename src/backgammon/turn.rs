use std::fmt::Display;

use super::{
    board::{Board, BoardPosition},
    game::Game,
    player::Player,
    Error,
};
use itertools::Itertools;

#[derive(Debug)]
pub(super) struct Turn {
    pub moves: Vec<Move>,
}

impl Turn {
    pub fn new(moves: Vec<Move>) -> Self {
        Self { moves }
    }

    pub fn from(notation: String, game: &Game) -> Result<Self, Error> {
        let re = regex::Regex::new(r"^(?:(?:\d+)|(?:bar))(?:/\d+)*(?:/(?:(?:\d+)|(?:off)))$")
            .expect("Regex is invalid");

        // Get all move groups
        let move_groups = notation
            .split_whitespace()
            .map(|group| match re.find(group) {
                Some(m) => Ok(Self::get_move_group(m.as_str(), game)?),
                None => Err(Error::InvalidNotation(group.to_owned())),
            })
            .flatten_ok()
            .collect::<Result<Vec<_>, _>>();

        match move_groups {
            Err(error) => return Err(error),
            Ok(move_groups) => Ok(Turn::new(move_groups)),
        }
    }

    fn get_move_group(notation: &str, game: &Game) -> Result<Vec<Move>, Error> {
        let positions = notation
            .split('/')
            .map(|m| {
                let player = game.current_player;
                Ok(match m {
                    "bar" => BoardPosition::Bar(player),
                    "off" => BoardPosition::Off(player),
                    pos => {
                        let index = pos.parse::<usize>().unwrap();
                        BoardPosition::Point(Board::convert_index(index, player)?)
                    }
                })
            })
            .collect::<Result<Vec<_>, _>>();

        match positions {
            Err(error) => return Err(error),
            Ok(positions) => Ok(positions
                .into_iter()
                .tuple_windows()
                .map(|(from, to)| Move::new(game.current_player, from, to))
                .collect()),
        }
    }

    // pub fn get_available_moves(game: &mut Game) -> impl Iterator<Item = Move> {
    //     // let game = game.clone();
    //     let saved_board = game.board.clone();

    //     game.board.iter().flat_map(|board_position| {
    //         // game.board = saved_board.clone();
    //         game.current_roll
    //             .borrow()
    //             .available_rolls()
    //             .map(|roll| {
    //                 let from_pos = board_position.effective_pos();

    //                 let to_pos = from_pos + roll as usize;
    //                 let notation = format!(
    //                     "{}/{}",
    //                     match board_position {
    //                         BoardPosition::Bar(_) => "bar".to_string(),
    //                         BoardPosition::Off(_) =>
    //                             Err("Cannot move a piece after bearing it off.")?,
    //                         BoardPosition::Point(_) => from_pos.to_string(),
    //                     },
    //                     match game.board.get_point(to_pos, game.current_player) {
    //                         Ok(_) => to_pos,
    //                         Err(error) => Err(error)?,
    //                     }
    //                 );

    //                 let turn = Turn::from(notation, game)?;
    //                 let moves = turn.moves.clone();

    //                 game.take_turn(turn)?;
    //                 Result::<_, &'static str>::Ok(moves)
    //             })
    //             .filter_map(Result::ok)
    //             .flatten()
    //             .collect::<Vec<_>>()
    //     })
    // }
}

#[derive(Debug, Clone)]
pub(super) struct Move {
    pub player: Player,
    pub from: BoardPosition,
    pub to: BoardPosition,
}

impl Move {
    pub fn new(player: Player, from: BoardPosition, to: BoardPosition) -> Self {
        Self { player, from, to }
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}/{}",
            match self.from {
                BoardPosition::Bar(_) => "bar".to_string(),
                BoardPosition::Off(_) => panic!("Cannot move a piece after bearing it off."),
                BoardPosition::Point(index) => (index + 1).to_string(),
            },
            match self.to {
                BoardPosition::Bar(_) => panic!("Cannot move onto the bar."),
                BoardPosition::Off(_) => "off".to_string(),
                BoardPosition::Point(index) => (index + 1).to_string(),
            }
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn notation() {
        let notation = "1/2/3 10/3 4/1".to_string();
        let _ = Turn::from(notation, &Game::new());

        let notation = "bar/3/4 bar/1".to_string();
        let _ = Turn::from(notation, &Game::new());

        let notation = "3/4/off 1/off".to_string();
        let _ = Turn::from(notation, &Game::new());

        let notation = "bar/off bar/off".to_string();
        let _ = Turn::from(notation, &Game::new());
    }

    #[test]
    #[should_panic]
    fn panic0() {
        let notation = "120/12".to_string();
        assert!(matches!(Turn::from(notation, &Game::new()), Ok(_)));
    }

    #[test]
    #[should_panic]
    fn panic1() {
        let notation = "3/bar".to_string();
        let _ = Turn::from(notation, &Game::new()).unwrap();
    }

    #[test]
    #[should_panic]
    fn panic2() {
        let notation = "3/bar/5".to_string();
        let _ = Turn::from(notation, &Game::new()).unwrap();
    }

    #[test]
    #[should_panic]
    fn panic3() {
        let notation = "off/4".to_string();
        let _ = Turn::from(notation, &Game::new()).unwrap();
    }

    #[test]
    #[should_panic]
    fn panic4() {
        let notation = "off/3/5".to_string();
        let _ = Turn::from(notation, &Game::new()).unwrap();
    }
}
