use itertools::Itertools;

use crate::backgammon::{
    board::{Position, BOARD_SIZE},
    player::Player,
    Error,
};

pub(super) struct Notation(pub String);

impl Notation {
    pub fn position_to_index(
        notation_position: usize,
        perspective: Player,
    ) -> Result<usize, Error> {
        let index = match perspective {
            Player::White => (BOARD_SIZE).checked_sub(notation_position),
            _ => notation_position.checked_sub(1),
        };

        match index {
            Some(index) if index < BOARD_SIZE => Ok(index),
            _ => Err(Error::InvalidNotationPosition(notation_position)),
        }
    }

    pub fn position_from_index(index: usize, perspective: Player) -> Result<usize, Error> {
        let notation_position = match perspective {
            Player::White => (BOARD_SIZE).checked_sub(index),
            _ => index.checked_add(1),
        };

        match notation_position {
            Some(notation_position)
                if notation_position <= BOARD_SIZE && notation_position >= 1 =>
            {
                Ok(notation_position)
            }
            _ => Err(Error::InvalidIndexPosition(index)),
        }
    }

    pub fn turn(&self, player: Player) -> Result<Turn, Error> {
        let re = regex::Regex::new(r"^(?:(?:\d+)|(?:bar))(?:/\d+)*(?:/(?:(?:\d+)|(?:off)))$")
            .expect("Regex is invalid");

        // Get all play groups
        let Notation(notation) = self;
        let play_groups = notation
            .split_whitespace()
            .map(|group| match re.find(group) {
                Some(m) => Ok(Notation(m.as_str().to_owned()).get_play_group(player)?),
                None => Err(Error::InvalidNotation(group.to_owned())),
            })
            .flatten_ok()
            .collect::<Result<Vec<_>, _>>();

        match play_groups {
            Err(error) => return Err(error),
            Ok(play_groups) => Ok(Turn(play_groups)),
        }
    }

    fn get_play_group(&self, player: Player) -> Result<Vec<Play>, Error> {
        let positions = self.get_board_positions(player);

        match positions {
            Err(error) => return Err(error),
            Ok(positions) => Ok(positions
                .into_iter()
                .tuple_windows()
                .map(|(from, to)| Play::new(player, from, to))
                .collect()),
        }
    }

    fn get_board_positions(&self, player: Player) -> Result<Vec<Position>, Error> {
        let Notation(notation) = self;

        notation
            .split('/')
            .map(|m| {
                Ok(match m {
                    "bar" => Position::Bar(player),
                    "off" => Position::Rail(player),
                    pos => {
                        let index = pos.parse::<usize>().unwrap();
                        Position::Point(Notation::position_to_index(index, player)?)
                    }
                })
            })
            .collect()
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct Turn(pub Vec<Play>);

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Play {
    pub player: Player,
    pub from: Position,
    pub to: Position,
}

impl Play {
    pub fn new(player: Player, from: Position, to: Position) -> Self {
        Self { player, from, to }
    }
}

impl std::fmt::Display for Play {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}/{}",
            match self.from {
                Position::Bar(_) => "bar".to_string(),
                Position::Rail(_) => panic!("Cannot play a piece after bearing it off."),
                Position::Point(index) => format!(
                    "{:?}",
                    Notation::position_from_index(index, self.player).unwrap()
                ),
            },
            match self.to {
                Position::Bar(_) => panic!("Cannot play onto the bar."),
                Position::Rail(_) => "off".to_string(),
                Position::Point(index) => format!(
                    "{:?}",
                    Notation::position_from_index(index, self.player).unwrap()
                ),
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_to_index_1() {
        assert_eq!(Notation::position_to_index(1, Player::Black), Ok(0));
    }

    #[test]
    fn position_to_index_2() {
        assert_eq!(Notation::position_to_index(20, Player::Black), Ok(19));
    }

    #[test]
    fn position_to_index_3() {
        assert_eq!(Notation::position_to_index(24, Player::White), Ok(0));
    }

    #[test]
    fn position_to_index_4() {
        assert_eq!(Notation::position_to_index(10, Player::White), Ok(14));
    }

    #[test]
    fn bad_position_to_index_1() {
        assert_eq!(
            Notation::position_to_index(25, Player::Black),
            Err(Error::InvalidNotationPosition(25))
        );
    }

    #[test]
    fn bad_position_to_index_2() {
        assert_eq!(
            Notation::position_to_index(0, Player::Black),
            Err(Error::InvalidNotationPosition(0))
        );
    }

    #[test]
    fn bad_position_to_index_3() {
        assert_eq!(
            Notation::position_to_index(27, Player::White),
            Err(Error::InvalidNotationPosition(27))
        );
    }

    #[test]
    fn bad_position_to_index_4() {
        assert_eq!(
            Notation::position_to_index(0, Player::White),
            Err(Error::InvalidNotationPosition(0))
        );
    }

    #[test]
    fn position_from_index_1() {
        assert_eq!(Notation::position_from_index(3, Player::Black), Ok(4));
    }

    #[test]
    fn position_from_index_2() {
        assert_eq!(Notation::position_from_index(13, Player::Black), Ok(14));
    }

    #[test]
    fn position_from_index_3() {
        assert_eq!(Notation::position_from_index(21, Player::White), Ok(3));
    }

    #[test]
    fn position_from_index_4() {
        assert_eq!(Notation::position_from_index(7, Player::White), Ok(17));
    }

    #[test]
    fn bad_position_from_index_1() {
        assert_eq!(
            Notation::position_from_index(25, Player::Black),
            Err(Error::InvalidIndexPosition(25))
        );
    }

    #[test]
    fn bad_position_from_index_2() {
        assert_eq!(
            Notation::position_from_index(29, Player::White),
            Err(Error::InvalidIndexPosition(29))
        );
    }

    #[test]
    fn position_to_from_index_1() {
        let player = Player::Black;
        let position = 10;
        assert_eq!(
            Notation::position_from_index(
                Notation::position_to_index(position, player).unwrap(),
                player
            ),
            Ok(position)
        )
    }

    #[test]
    fn position_to_from_index_2() {
        let player = Player::White;
        let position = 24;
        assert_eq!(
            Notation::position_from_index(
                Notation::position_to_index(position, player).unwrap(),
                player
            ),
            Ok(position)
        )
    }

    #[test]
    fn position_to_from_index_3() {
        let player = Player::White;
        let position = 3;
        assert_eq!(
            Notation::position_from_index(
                Notation::position_to_index(position, player).unwrap(),
                player
            ),
            Ok(position)
        )
    }

    #[test]
    fn position_from_to_index_1() {
        let player = Player::Black;
        let position = 9;
        assert_eq!(
            Notation::position_to_index(
                Notation::position_from_index(position, player).unwrap(),
                player
            ),
            Ok(position)
        )
    }

    #[test]
    fn position_from_to_index_2() {
        let player = Player::White;
        let position = 1;
        assert_eq!(
            Notation::position_to_index(
                Notation::position_from_index(position, player).unwrap(),
                player
            ),
            Ok(position)
        )
    }

    #[test]
    fn position_from_to_index_3() {
        let player = Player::White;
        let position = 22;
        assert_eq!(
            Notation::position_to_index(
                Notation::position_from_index(position, player).unwrap(),
                player
            ),
            Ok(position)
        )
    }

    #[test]
    fn black_notation_1() {
        let player = Player::Black;
        let notation = Notation("1/2".to_string()).turn(player);
        let turn = Turn(vec![Play::new(
            player,
            Position::Point(0),
            Position::Point(1),
        )]);
        assert_eq!(notation, Ok(turn));
    }

    #[test]
    fn black_notation_2() {
        let player = Player::Black;
        let notation = Notation("3/4 13/14".to_string()).turn(player);
        let turn = Turn(vec![
            Play::new(player, Position::Point(2), Position::Point(3)),
            Play::new(player, Position::Point(12), Position::Point(13)),
        ]);
        assert_eq!(notation, Ok(turn));
    }

    #[test]
    fn black_notation_3() {
        let player = Player::Black;
        let notation = Notation("10/5/19".to_string()).turn(player);
        let turn = Turn(vec![
            Play::new(player, Position::Point(9), Position::Point(4)),
            Play::new(player, Position::Point(4), Position::Point(18)),
        ]);
        assert_eq!(notation, Ok(turn));
    }

    #[test]
    fn black_notation_4() {
        let player = Player::Black;
        let notation = Notation("8/3/14/22 7/19".to_string()).turn(player);
        let turn = Turn(vec![
            Play::new(player, Position::Point(7), Position::Point(2)),
            Play::new(player, Position::Point(2), Position::Point(13)),
            Play::new(player, Position::Point(13), Position::Point(21)),
            Play::new(player, Position::Point(6), Position::Point(18)),
        ]);
        assert_eq!(notation, Ok(turn));
    }

    #[test]
    fn black_notation_5() {
        let player = Player::Black;
        let notation = Notation("bar/1/20 7/19/off".to_string()).turn(player);
        let turn = Turn(vec![
            Play::new(player, Position::Bar(player), Position::Point(0)),
            Play::new(player, Position::Point(0), Position::Point(19)),
            Play::new(player, Position::Point(6), Position::Point(18)),
            Play::new(player, Position::Point(18), Position::Rail(player)),
        ]);
        assert_eq!(notation, Ok(turn));
    }

    #[test]
    fn black_notation_6() {
        let player = Player::Black;
        let notation = Notation("bar/off bar/off".to_string()).turn(player);
        let turn = Turn(vec![
            Play::new(player, Position::Bar(player), Position::Rail(player)),
            Play::new(player, Position::Bar(player), Position::Rail(player)),
        ]);
        assert_eq!(notation, Ok(turn));
    }

    #[test]
    fn bad_notation_1() {
        let player = Player::Black;
        let input = "test123.4abc-30".to_string();
        let notation = Notation(input.clone()).turn(player);
        assert_eq!(notation, Err(Error::InvalidNotation(input)));
    }

    #[test]
    fn bad_notation_2() {
        let player = Player::Black;
        let input = "bar/bar".to_string();
        let notation = Notation(input.clone()).turn(player);
        assert_eq!(notation, Err(Error::InvalidNotation(input)));
    }

    #[test]
    fn bad_notation_3() {
        let player = Player::Black;
        let input = "1/bar/10".to_string();
        let notation = Notation(input.clone()).turn(player);
        assert_eq!(notation, Err(Error::InvalidNotation(input)));
    }

    #[test]
    fn bad_notation_4() {
        let player = Player::Black;
        let input = "off/10/3".to_string();
        let notation = Notation(input.clone()).turn(player);
        assert_eq!(notation, Err(Error::InvalidNotation(input)));
    }

    #[test]
    fn bad_notation_5() {
        let player = Player::Black;
        let input = "bar/10/off 19/off/21".to_string();
        let notation = Notation(input.clone()).turn(player);
        assert_eq!(
            notation,
            Err(Error::InvalidNotation("19/off/21".to_string()))
        );
    }
}
