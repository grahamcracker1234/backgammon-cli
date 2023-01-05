use itertools::Itertools;

use crate::backgammon::{
    board::{Board, Space, BOARD_SIZE},
    player::Player,
    position::{IndexPosition, NormalizedPosition},
    Error,
};

pub(super) struct Notation(pub String);

impl Notation {
    // pub fn normalized_position_to_index(
    //     notation_position: usize,
    //     perspective: Player,
    // ) -> Result<usize, Error> {
    //     let index = match perspective {
    //         Player::White => (BOARD_SIZE).checked_sub(notation_position),
    //         _ => notation_position.checked_sub(1),
    //     };

    //     match index {
    //         Some(index) if index < BOARD_SIZE => Ok(index),
    //         _ => Err(Error::InvalidNotationPosition(notation_position)),
    //     }
    // }

    // pub fn position_from_index(index: usize, perspective: Player) -> Result<usize, Error> {
    //     let notation_position = match perspective {
    //         Player::White => (BOARD_SIZE).checked_sub(index),
    //         _ => index.checked_add(1),
    //     };

    //     match notation_position {
    //         Some(notation_position) if (1..=BOARD_SIZE).contains(&notation_position) => {
    //             Ok(notation_position)
    //         }
    //         _ => Err(Error::InvalidIndexPosition(index)),
    //     }
    // }

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
            Err(error) => Err(error),
            Ok(play_groups) => Ok(Turn(play_groups)),
        }
    }

    fn get_play_group(&self, player: Player) -> Result<Vec<Play>, Error> {
        let positions = self.get_board_positions(player)?;

        Ok(positions
            .into_iter()
            .tuple_windows()
            .map(|(from, to)| Play::new(player, from, to))
            .collect())
    }

    fn get_board_positions(&self, player: Player) -> Result<Vec<Space>, Error> {
        let Notation(notation) = self;

        notation
            .split('/')
            .map(|m| {
                Ok(match m {
                    "bar" => Space::Bar(player),
                    "off" => Space::Rail(player),
                    pos => {
                        let norm = NormalizedPosition::new(
                            pos.parse::<usize>().expect("pos should be an integer"),
                            player,
                        )?;
                        Space::Point(norm.to_index()?)
                        // Space::Point(Notation::normalized_position_to_index(index, player)?)
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
                Space::Point(index) => index.normalize(self.player).to_string()
                // format!(
                //     "{:?}",
                //     Notation::position_from_index(index, self.player).unwrap()
                // ),
            },
            match self.to {
                Space::Bar(_) => panic!("Cannot play onto the bar."),
                Space::Rail(_) => "off".to_string(),
                Space::Point(index) => index.normalize(self.player).to_string()

                // format!(
                //     "{:?}",
                //     Notation::position_from_index(index, self.player).unwrap()
                // ),
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    //     #[test]
    //     fn black_notation_1() {
    //         let player = Player::Black;
    //         let notation = Notation("1/2".to_string()).turn(player);
    //         let turn = Turn(vec![Play::new(player, Space::Point(0), Space::Point(1))]);
    //         assert_eq!(notation, Ok(turn));
    //     }
    // ::new
    //     #[t.unwrap()est]
    //     fn black_notation_2() {
    //         let player = Player::Black;
    //         let notation = Notation("3/4 13/14".to_string()).turn(player);
    //         let turn = Turn(vec![
    //             Play::new(player, Space::Point(2), Space::Point(3)),
    //             Play::new(player, Space::Point(12), Space::Point(13)),
    //         ]);
    //         assert_eq!(notation, Ok(turn));
    //     }
    // ::new
    //     #[t.unwrap()est]
    //     fn black_notation_3() {
    //         let player = Player::Black;
    //         let notation = Notation("10/5/19".to_string()).turn(player);
    //         let turn = Turn(vec![
    //             Play::new(player, Space::Point(9), Space::Point(4)),
    //             Play::new(player, Space::Point(4), Space::Point(18)),
    //         ]);
    //         assert_eq!(notation, Ok(turn));
    //     }
    // ::new
    //     #[t.unwrap()est]
    //     fn black_notation_4() {
    //         let player = Player::Black;
    //         let notation = Notation("8/3/14/22 7/19".to_string()).turn(player);
    //         let turn = Turn(vec![
    //             Play::new(player, Space::Point(7), Space::Point(2)),
    //             Play::new(player, Space::Point(2), Space::Point(13)),
    //             Play::new(player, Space::Point(13), Space::Point(21)),
    //             Play::new(player, Space::Point(6), Space::Point(18)),
    //         ]);
    //         assert_eq!(notation, Ok(turn));
    //     }
    // ::new
    //     #[t.unwrap()est]
    //     fn black_notation_5() {
    //         let player = Player::Black;
    //         let notation = Notation("bar/1/20 7/19/off".to_string()).turn(player);
    //         let turn = Turn(vec![
    //             Play::new(player, Space::Bar(player), Space::Point(0)),
    //             Play::new(player, Space::Point(0), Space::Point(19)),
    //             Play::new(player, Space::Point(6), Space::Point(18)),
    //             Play::new(player, Space::Point(18), Space::Rail(player)),
    //         ]);
    //         assert_eq!(notation, Ok(turn));
    //     }
    // ::new
    //     #[t.unwrap()est]
    //     fn black_notation_6() {
    //         let player = Player::Black;
    //         let notation = Notation("bar/off bar/off".to_string()).turn(player);
    //         let turn = Turn(vec![
    //             Play::new(player, Space::Bar(player), Space::Rail(player)),
    //             Play::new(player, Space::Bar(player), Space::Rail(player)),
    //         ]);
    //         assert_eq!(notation, Ok(turn));
    //     }
    // ::new
    //     #[t.unwrap()est]
    //     fn bad_notation_1() {
    //         let player = Player::Black;
    //         let input = "test123.4abc-30".to_string();
    //         let notation = Notation(input.clone()).turn(player);
    //         assert_eq!(notation, Err(Error::InvalidNotation(input)));
    //     }
    // ::new
    //     #[t.unwrap()est]
    //     fn bad_notation_2() {
    //         let player = Player::Black;
    //         let input = "bar/bar".to_string();
    //         let notation = Notation(input.clone()).turn(player);
    //         assert_eq!(notation, Err(Error::InvalidNotation(input)));
    //     }
    // ::new
    //     #[t.unwrap()est]
    //     fn bad_notation_3() {
    //         let player = Player::Black;
    //         let input = "1/bar/10".to_string();
    //         let notation = Notation(input.clone()).turn(player);
    //         assert_eq!(notation, Err(Error::InvalidNotation(input)));
    //     }
    // ::new
    //     #[t.unwrap()est]
    //     fn bad_notation_4() {
    //         let player = Player::Black;
    //         let input = "off/10/3".to_string();
    //         let notation = Notation(input.clone()).turn(player);
    //         assert_eq!(notation, Err(Error::InvalidNotation(input)));
    //     }
    // ::new
    //     #[t.unwrap()est]
    //     fn bad_notation_5() {
    //         let player = Player::Black;
    //         let input = "bar/10/off 19/off/21".to_string();
    //         let notation = Notation(input.clone()).turn(player);
    //         assert_eq!(
    //             notation::new,
    // .unwrap()            Err(Error::InvalidNotation("19/off/21".to_string()))
    //         );
    //     }
}
