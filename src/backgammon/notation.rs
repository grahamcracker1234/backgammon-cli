use itertools::Itertools;
use regex::Regex;

use crate::backgammon::{
    board::{Board, Space},
    player::Player,
    position::NormalizedPosition,
    Error,
};

pub(super) struct Notation {
    input: String,
    player: Player,
}

impl Notation {
    pub fn new(input: String, player: Player) -> Self {
        Self { input, player }
    }

    pub fn turn(&self) -> Result<Turn, Error> {
        let re = Regex::new(r"^(?:(?:\d+)|(?:bar))(?:/\d+)*(?:/(?:(?:\d+)|(?:off)))$")
            .expect("regex should always be valid");

        // Get all play groups
        let play_groups = self
            .input
            .split_whitespace()
            .map(|group| match re.find(group) {
                Some(m) => {
                    let input = m.as_str().to_owned();
                    let notation = Notation::new(input, self.player);
                    notation.get_play_group()
                }
                None => Err(Error::InvalidNotation(group.to_owned())),
            })
            .flatten_ok()
            .collect();

        match play_groups {
            Err(error) => Err(error),
            Ok(play_groups) => Ok(Turn(play_groups)),
        }
    }

    fn get_play_group(&self) -> Result<Vec<Play>, Error> {
        let spaces = self.get_board_spaces()?;
        let plays = spaces
            .into_iter()
            .tuple_windows()
            .map(|(from, to)| Play::new(self.player, from, to))
            .collect();
        Ok(plays)
    }

    fn get_board_spaces(&self) -> Result<Vec<Space>, Error> {
        let spaces = self.input.split('/');
        spaces
            .map(|m| {
                Ok(match m {
                    "bar" => Space::Bar(self.player),
                    "off" => Space::Rail(self.player),
                    pos => {
                        let pos = pos.parse::<usize>().expect("pos should be an integer");
                        let norm = NormalizedPosition::new(pos, self.player)?;
                        let index = norm.to_index()?;
                        Space::Point(index)
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
    pub from: Space,
    pub to: Space,
}

impl Play {
    pub fn new(player: Player, from: Space, to: Space) -> Self {
        Self { player, from, to }
    }

    pub fn is_valid_direction(&self, board: &Board) -> bool {
        let to = self.to.point(board).borrow();
        let from = self.from.point(board).borrow();

        match self.player {
            Player::White => to.position > from.position,
            Player::Black => to.position < from.position,
            Player::None => panic!("There is no move direction for `Player::None`."),
        }
    }
}

impl std::fmt::Display for Play {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}/{}",
            match self.from {
                Space::Bar(_) => "bar".to_string(),
                Space::Rail(_) => panic!("Cannot play a piece after bearing it off."),
                Space::Point(index) => index.normalize(self.player).to_string(),
            },
            match self.to {
                Space::Bar(_) => panic!("Cannot play onto the bar."),
                Space::Rail(_) => "off".to_string(),
                Space::Point(index) => index.normalize(self.player).to_string(),
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Turn(vec![
    //     Play::new(
    //         player,
    //         Space::Point(2.try_into().unwrap()),
    //         Space::Point(3.try_into().unwrap()),
    //     ),
    //     Play::new(
    //         player,
    //         Space::Point(12.try_into().unwrap()),
    //         Space::Point(13.try_into().unwrap()),
    //     ),
    // ]);
    macro_rules! turn {
        ($player:expr, $(($from:tt, $to:tt)),*) => {
            Turn(vec![
                $(Play::new(
                    $player,
                    turn!($player, $from),
                    turn!($player, $to),
                )),*
            ])
        };
        ($player:expr, bar) => { Space::Bar($player) };
        ($player:expr, off) => { Space::Rail($player) };
        ($player:expr, $index:literal) => { Space::Point($index.try_into().unwrap()) };
    }

    //     #[test]
    //     fn position_from_index_1() {
    //         assert_eq!(Notation::position_from_index(3, Player::Black), Ok(4));
    //     }

    //         #[test]
    //     fn position_from_index_2() {
    //         assert_eq!(Notation::position_from_index(13, Player::Black), Ok(14));
    //     }

    //         #[test]
    //     fn position_from_index_3() {
    //         assert_eq!(Notation::position_from_index(21, Player::White), Ok(3));
    //     }

    //     #[test]
    //     fn position_from_index_4() {
    //         assert_eq!(Notation::position_from_index(7, Player::White), Ok(17));
    //     }

    //     #[test]
    //     fn bad_position_from_index_1() {
    //         assert_eq!(
    //             Notation::new::p.unwrap()osition_from_index(25, Player::Black),
    //             Err(Error::InvalidIndexPosition(25))
    //         );
    //     }

    //     #[test]
    //     fn bad_position_from_index_2() {
    //         assert_eq!(
    //             Notation::new::p.unwrap()osition_from_index(29, Player::White),
    //             Err(Error::InvalidIndexPosition(29))
    //         );
    //     }

    //     #[test]
    //     fn position_to_from_index_1() {
    //         let player = Player::Black;
    //         let position = 10;
    //         assert_eq!(
    //             Notation::new::p.unwrap()osition_from_index(
    //                 Notation::normalized_position_to_index(position, player).unwrap(),
    //                 player
    //             ),
    //             Ok(position)
    //         )
    //     }

    //     #[test]
    //     fn position_to_from_index_2() {
    //         let player = Player::White;
    //         let position = 24;
    //         assert_eq!(
    //             Notation::new::p.unwrap()osition_from_index(
    //                 Notation::normalized_position_to_index(position, player).unwrap(),
    //                 player
    //             ),
    //             Ok(position)
    //         )
    //     }

    //     #[test]
    //     fn position_to_from_index_3() {
    //         let player = Player::White;
    //         let position = 3;
    //         assert_eq!(
    //             Notation::new::p.unwrap()osition_from_index(
    //                 Notation::normalized_position_to_index(position, player).unwrap(),
    //                 player
    //             ),
    //             Ok(position)
    //         )
    //     }

    //     #[test]
    //     fn position_from_to_index_1() {
    //         let player = Player::Black;
    //         let position = 9;
    //         assert_eq!(
    //             Notation::new::p.unwrap()osition_to_index(
    //                 Notation::position_from_index(position, player).unwrap(),
    //                 player
    //             ),
    //             Ok(position)
    //         )
    //     }

    //     #[test]
    //     fn position_from_to_index_2() {
    //         let player = Player::White;
    //         let position = 1;
    //         assert_eq!(
    //             Notation::new::p.unwrap()osition_to_index(
    //                 Notation::position_from_index(position, player).unwrap(),
    //                 player
    //             ),
    //             Ok(position)
    //         )
    //     }

    //     #[test]
    //     fn position_from_to_index_3() {
    //         let player = Player::White;
    //         let position = 22;
    //         assert_eq!(
    //             Notation::new::p.unwrap()osition_to_index(
    //                 Notation::position_from_index(position, player).unwrap(),
    //                 player
    //             ),
    //             Ok(position)
    //         )
    //     }

    #[test]
    fn black_notation_1() {
        let player = Player::Black;
        let notation = Notation::new("1/2".to_string(), player);
        // let turn = Turn(vec![Play::new(
        //     player,
        //     Space::Point(0.try_into().unwrap()),
        //     Space::Point(1.try_into().unwrap()),
        // )]);

        let turn = turn!(player, (0, 1));
        assert_eq!(notation.turn(), Ok(turn));
    }

    #[test]
    fn black_notation_2() {
        let player = Player::Black;
        let notation = Notation::new("3/4 13/14".to_string(), player);
        let turn = turn!(player, (2, 3), (12, 13));

        assert_eq!(notation.turn(), Ok(turn));
    }

    #[test]
    fn black_notation_3() {
        let player = Player::Black;
        let notation = Notation::new("10/5/19".to_string(), player);
        let turn = turn!(player, (9, 4), (4, 18));
        assert_eq!(notation.turn(), Ok(turn));
    }

    #[test]
    fn black_notation_4() {
        let player = Player::Black;
        let notation = Notation::new("8/3/14/22 7/19".to_string(), player);
        let turn = turn!(player, (7, 2), (2, 13), (13, 21), (6, 18));
        assert_eq!(notation.turn(), Ok(turn));
    }

    #[test]
    fn black_notation_5() {
        let player = Player::Black;
        let notation = Notation::new("bar/1/20 7/19/off".to_string(), player);
        let turn = turn!(player, (bar, 0), (0, 19), (6, 18), (18, off));
        assert_eq!(notation.turn(), Ok(turn));
    }

    #[test]
    fn black_notation_6() {
        let player = Player::Black;
        let notation = Notation::new("bar/off bar/off".to_string(), player);
        let turn = turn!(player, (bar, off), (bar, off));
        assert_eq!(notation.turn(), Ok(turn));
    }

    #[test]
    fn bad_notation_1() {
        let player = Player::Black;
        let input = "test123.4abc-30".to_string();
        let notation = Notation::new(input.clone(), player);
        assert_eq!(notation.turn(), Err(Error::InvalidNotation(input)));
    }

    #[test]
    fn bad_notation_2() {
        let player = Player::Black;
        let input = "bar/bar".to_string();
        let notation = Notation::new(input.clone(), player);
        assert_eq!(notation.turn(), Err(Error::InvalidNotation(input)));
    }

    #[test]
    fn bad_notation_3() {
        let player = Player::Black;
        let input = "1/bar/10".to_string();
        let notation = Notation::new(input.clone(), player);
        assert_eq!(notation.turn(), Err(Error::InvalidNotation(input)));
    }

    #[test]
    fn bad_notation_4() {
        let player = Player::Black;
        let input = "off/10/3".to_string();
        let notation = Notation::new(input.clone(), player);
        assert_eq!(notation.turn(), Err(Error::InvalidNotation(input)));
    }

    #[test]
    fn bad_notation_5() {
        let player = Player::Black;
        let input = "19/off/21".to_string();
        let notation = Notation::new(format!("bar/10/off {input}"), player);
        assert_eq!(notation.turn(), Err(Error::InvalidNotation(input)));
    }
}
