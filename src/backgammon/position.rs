use crate::backgammon::{board::BOARD_SIZE, player::Player, Error};
use std::convert::From;
use std::fmt::Display;
use std::ops::Deref;

/// Represents a range from 0 to 25 where 0 is the player's rail and the opponent's
/// bar, 1 is the player's ace, 24 is the opponent's ace, and 25 is the player's
/// bar and the opponent's rail. This position is normalized to a given player's
/// perspective and is not absolute.
#[derive(Debug, PartialEq)]
pub(crate) struct NormalizedPosition(usize, Player);

// #[allow(dead_code)]
impl NormalizedPosition {
    /// Create a `NormalizedPosition`
    pub fn new(position: usize, player: Player) -> Result<Self, Error> {
        if player == Player::None {
            return Err(Error::InvalidNormalizedPosition(position, player));
        }

        if (0..=25).contains(&position) {
            Ok(NormalizedPosition(position, player))
        } else {
            Err(Error::InvalidNormalizedPosition(position, player))
        }
    }

    /// Denormalize the given `NormalizedPosition` from a `Player` perspective.
    pub fn denormalize(&self) -> Result<DenormalizedPosition, Error> {
        match self.1 {
            Player::Black => self.0.try_into(),
            Player::White => ((BOARD_SIZE + 1) - self.0).try_into(),
            Player::None => panic!("cannot denormalize with perspective of `Player::None`"),
        }
    }

    /// Converts the given `NormalizedPosition` to an `IndexPosition` from a
    /// `Player` perspective.
    pub fn to_index(&self) -> Result<IndexPosition, Error> {
        // Here to avoid underflow errors from unsigned subtraction.
        if self.0 == 0 || self.0 == (BOARD_SIZE + 1) {
            return Err(Error::InvalidIndexPosition(self.0));
        }

        match self.1 {
            Player::Black => (self.0 - 1).try_into(),
            Player::White => ((BOARD_SIZE + 1) - self.0 - 1).try_into(),
            Player::None => panic!("cannot convert to index with perspective of `Player::None`"),
        }
    }
}

/// Displays the value inside the `NormalizedPosition`.
impl Display for NormalizedPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// /// Tries to create a new `NormalizedPosition` from a `usize` validating it is
// /// contained within the range `0..=25`.
// impl TryFrom<usize> for NormalizedPosition {
//     type Error = crate::backgammon::Error;

//     fn try_from(value: usize) -> Result<Self, Self::Error> {
//         match (0..=25).contains(&value) {
//             true => Ok(NormalizedPosition(value)),
//             false => Err(Error::InvalidNormalizedPosition(value)),
//         }
//     }
// }

// /// Creates a `usize` from a `NormalizedPosition`.
// impl From<NormalizedPosition> for usize {
//     fn from(value: NormalizedPosition) -> Self {
//         value.0
//     }
// }

// /// Gets the `usize` from a `NormalizedPosition`.
// impl Deref for NormalizedPosition {
//     type Target = usize;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

/// Represents a range from 0 to 25 where 0 is the `Player::Black` rail and the
/// `Player::White`bar, 1 is the `Player::Black` ace, 24 is the `Player::White`
/// ace, and 25 is the `Player::Black` bar and the `Player::White` rail. This
/// position is not normalized and is absolute no matter the perspective.
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub(crate) struct DenormalizedPosition(usize);

// #[allow(dead_code)]
impl DenormalizedPosition {
    /// Normalize the given `DenormalizedPosition` to a `Player` perspective.
    ///
    /// The value must be contained within range `0..=25` for `NormalizedPosition`
    /// to be instantiated from `usize`, thus when this method is called, the
    /// `DenormalizedPosition` is guaranteed to be in range `0..=25`.
    pub fn normalize(self, perspective: Player) -> NormalizedPosition {
        match perspective {
            Player::Black => NormalizedPosition::new(self.0, perspective),
            Player::White => NormalizedPosition::new((BOARD_SIZE + 1) - self.0, perspective),
            Player::None => panic!("cannot normalize with perspective of `Player::None`"),
        }
        .unwrap()
    }

    /// Converts the given `DenormalizedPosition` to an `IndexPosition`.
    pub fn to_index(self) -> Result<IndexPosition, Error> {
        // Here to avoid underflow errors from unsigned subtraction.
        if self.0 == 0 || self.0 == (BOARD_SIZE + 1) {
            return Err(Error::InvalidIndexPosition(self.0));
        }

        (self.0 - 1).try_into()
    }
}

/// Tries to create a new `DenormalizedPosition` from a `usize` validating it is
/// contained within the range `0..=25`.
impl TryFrom<usize> for DenormalizedPosition {
    type Error = crate::backgammon::Error;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if (0..=25).contains(&value) {
            Ok(DenormalizedPosition(value))
        } else {
            Err(Error::InvalidDenormalizedPosition(value))
        }
    }
}

/// Creates a `usize` from a `DenormalizedPosition`.
impl From<DenormalizedPosition> for usize {
    fn from(value: DenormalizedPosition) -> Self {
        value.0
    }
}

/// Gets the `usize` from a `DenormalizedPosition`.
impl Deref for DenormalizedPosition {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Represents a range from 0 to 23 where 0 is the `Player::Black` ace and 23 is
/// the `Player::White` ace. This position is not normalized and is absolute no
/// matter the perspective. It is used for indexing into `Board.points`.
#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct IndexPosition(usize);

// #[allow(dead_code)]
impl IndexPosition {
    /// Normalize the given `IndexPosition` to a `Player` perspective.
    ///
    /// The value must be contained within range `0..=23` for `IndexPosition` to
    /// be instantiated from `usize`, thus when this method is called, the
    /// `NormalizedPosition` is guaranteed to be in range `0..=25`.
    pub fn normalize(self, perspective: Player) -> NormalizedPosition {
        match perspective {
            Player::Black => NormalizedPosition::new(self.0 + 1, perspective),
            Player::White => NormalizedPosition::new((BOARD_SIZE + 1) - (self.0 + 1), perspective),
            Player::None => panic!("cannot normalize with perspective of `Player::None`"),
        }
        .unwrap()
    }

    /// Denormalize the given `IndexPosition`.
    ///
    /// The value must be contained within range `0..=23` for `IndexPosition` to
    /// be instantiated from `usize`, thus when this method is called, the
    /// `DenormalizedPosition` is guaranteed to be in range `0..=25`.
    pub fn denormalize(self) -> DenormalizedPosition {
        (self.0 + 1).try_into().unwrap()
    }
}

/// Tries to create a new `IndexPosition` from a `usize` validating it is
/// contained within the range `0..=23`.
impl TryFrom<usize> for IndexPosition {
    type Error = crate::backgammon::Error;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if (0..=23).contains(&value) {
            Ok(IndexPosition(value))
        } else {
            Err(Error::InvalidIndexPosition(value))
        }
    }
}

/// Creates a `usize` from a `IndexPosition`.
impl From<IndexPosition> for usize {
    fn from(value: IndexPosition) -> Self {
        value.0
    }
}

/// Gets the `usize` from a `IndexPosition`.
impl Deref for IndexPosition {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bad_normalized_position_1() {
        assert_eq!(
            NormalizedPosition::new(27, Player::Black),
            Err(Error::InvalidNormalizedPosition(27, Player::Black))
        );
    }

    #[test]
    fn bad_normalized_position_2() {
        assert_eq!(
            NormalizedPosition::new(209, Player::White),
            Err(Error::InvalidNormalizedPosition(209, Player::White))
        );
    }

    #[test]
    fn bad_normalized_position_3() {
        assert_eq!(
            NormalizedPosition::new(10, Player::None),
            Err(Error::InvalidNormalizedPosition(10, Player::None))
        );
    }

    #[test]
    fn bad_denormalized_position_1() {
        assert_eq!(
            DenormalizedPosition::try_from(26),
            Err(Error::InvalidDenormalizedPosition(26))
        );
    }

    #[test]
    fn bad_denormalized_position_2() {
        assert_eq!(
            DenormalizedPosition::try_from(87),
            Err(Error::InvalidDenormalizedPosition(87))
        );
    }

    #[test]
    fn bad_index_position_1() {
        assert_eq!(
            IndexPosition::try_from(24),
            Err(Error::InvalidIndexPosition(24))
        );
    }

    #[test]
    fn bad_index_position_2() {
        assert_eq!(
            IndexPosition::try_from(67),
            Err(Error::InvalidIndexPosition(67))
        );
    }

    #[test]
    fn normalized_position_to_index_1() -> Result<(), Error> {
        assert_eq!(
            NormalizedPosition::new(1, Player::Black)?.to_index(),
            Ok(IndexPosition::try_from(0)?)
        );
        Ok(())
    }

    #[test]
    fn normalized_position_to_index_2() -> Result<(), Error> {
        assert_eq!(
            NormalizedPosition::new(20, Player::Black)?.to_index(),
            Ok(IndexPosition::try_from(19)?)
        );
        Ok(())
    }

    #[test]
    fn normalized_position_to_index_3() -> Result<(), Error> {
        assert_eq!(
            NormalizedPosition::new(24, Player::White)?.to_index(),
            Ok(IndexPosition::try_from(0)?)
        );
        Ok(())
    }

    #[test]
    fn normalized_position_to_index_4() -> Result<(), Error> {
        assert_eq!(
            NormalizedPosition::new(10, Player::White)?.to_index(),
            Ok(IndexPosition::try_from(14)?)
        );
        Ok(())
    }

    #[test]
    fn bad_normalized_position_to_index_1() -> Result<(), Error> {
        assert_eq!(
            NormalizedPosition::new(25, Player::Black)?.to_index(),
            Err(Error::InvalidIndexPosition(25))
        );
        Ok(())
    }

    #[test]
    fn bad_normalized_bad_position_to_index_2() -> Result<(), Error> {
        assert_eq!(
            NormalizedPosition::new(0, Player::Black)?.to_index(),
            Err(Error::InvalidIndexPosition(0))
        );
        Ok(())
    }

    #[test]
    fn bad_normalized_position_to_index_3() -> Result<(), Error> {
        assert_eq!(
            NormalizedPosition::new(25, Player::White)?.to_index(),
            Err(Error::InvalidIndexPosition(25))
        );
        Ok(())
    }

    #[test]
    fn bad_normalized_bad_position_to_index_4() -> Result<(), Error> {
        assert_eq!(
            NormalizedPosition::new(0, Player::White)?.to_index(),
            Err(Error::InvalidIndexPosition(0))
        );
        Ok(())
    }

    #[test]
    fn index_position_to_normalized_1() -> Result<(), Error> {
        assert_eq!(
            IndexPosition::try_from(0)?.normalize(Player::Black),
            NormalizedPosition::new(1, Player::Black)?
        );
        Ok(())
    }

    #[test]
    fn index_position_to_normalized_2() -> Result<(), Error> {
        assert_eq!(
            IndexPosition::try_from(19)?.normalize(Player::Black),
            NormalizedPosition::new(20, Player::Black)?
        );
        Ok(())
    }

    #[test]
    fn index_position_to_normalized_3() -> Result<(), Error> {
        assert_eq!(
            IndexPosition::try_from(23)?.normalize(Player::White),
            NormalizedPosition::new(1, Player::White)?
        );
        Ok(())
    }

    #[test]
    fn index_position_to_normalized_4() -> Result<(), Error> {
        assert_eq!(
            IndexPosition::try_from(14)?.normalize(Player::White),
            NormalizedPosition::new(10, Player::White)?
        );
        Ok(())
    }
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
