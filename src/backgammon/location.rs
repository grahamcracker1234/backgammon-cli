use crate::backgammon::{board::BOARD_SIZE, player::Player, Error};
use std::convert::From;
use std::fmt::Display;
use std::ops::Deref;

/// Represents a range from 0 to 25 where 0 is the player's rail and the opponent's
/// bar, 1 is the player's ace, 24 is the opponent's ace, and 25 is the player's
/// bar and the opponent's rail. This position is normalized to a given player's
/// perspective and is not absolute.
#[derive(Debug, PartialEq, Eq)]
pub struct Normalized(usize, Player);

// #[allow(dead_code)]
impl Normalized {
    /// Create a `Normalized`.
    pub fn new(position: usize, player: Player) -> Result<Self, Error> {
        if player == Player::None {
            return Err(Error::InvalidNormalizedLocation(position, player));
        }

        if (0..=25).contains(&position) {
            Ok(Self(position, player))
        } else {
            Err(Error::InvalidNormalizedLocation(position, player))
        }
    }

    // /// Denormalize the given `NormalizedLocation` from a `Player` perspective.
    // pub fn denormalize(&self) -> Result<DenormalizedLocation, Error> {
    //     match self.1 {
    //         Player::Black => self.0.try_into(),
    //         Player::White => ((BOARD_SIZE + 1) - self.0).try_into(),
    //         Player::None => panic!("cannot denormalize with perspective of `Player::None`"),
    //     }
    // }

    /// Converts the given `NormalizedLocation` to an `IndexLocation` from a
    /// `Player` perspective.
    pub fn to_index(&self) -> Result<Index, Error> {
        // Here to avoid underflow errors from unsigned subtraction.
        if self.0 == 0 || self.0 == (BOARD_SIZE + 1) {
            return Err(Error::InvalidIndexLocation(self.0));
        }

        match self.1 {
            Player::Black => (self.0 - 1).try_into(),
            Player::White => ((BOARD_SIZE + 1) - self.0 - 1).try_into(),
            Player::None => panic!("cannot convert to index with perspective of `Player::None`"),
        }
    }
}

/// Displays the value inside the `NormalizedLocation`.
impl Display for Normalized {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// /// Tries to create a new `NormalizedLocation` from a `usize` validating it is
// /// contained within the range `0..=25`.
// impl TryFrom<usize> for NormalizedLocation {
//     type Error = crate::backgammon::Error;

//     fn try_from(value: usize) -> Result<Self, Self::Error> {
//         match (0..=25).contains(&value) {
//             true => Ok(NormalizedLocation(value)),
//             false => Err(Error::InvalidNormalizedLocation(value)),
//         }
//     }
// }

// /// Creates a `usize` from a `NormalizedLocation`.
// impl From<NormalizedLocation> for usize {
//     fn from(value: NormalizedLocation) -> Self {
//         value.0
//     }
// }

/// Gets the `usize` from a `NormalizedLocation`.
impl Deref for Normalized {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Represents a range from 0 to 25 where 0 is the `Player::Black` rail and the
/// `Player::White`bar, 1 is the `Player::Black` ace, 24 is the `Player::White`
/// ace, and 25 is the `Player::Black` bar and the `Player::White` rail. This
/// position is not normalized and is absolute no matter the perspective.
#[derive(Debug, PartialEq, Eq, PartialOrd, Clone, Copy)]
pub struct Denormalized(usize);

// #[allow(dead_code)]
impl Denormalized {
    /// Normalize the given `DenormalizedLocation` to a `Player` perspective.
    ///
    /// The value must be contained within range `0..=25` for `NormalizedLocation`
    /// to be instantiated from `usize`, thus when this method is called, the
    /// `DenormalizedLocation` is guaranteed to be in range `0..=25`.
    pub fn normalize(self, perspective: Player) -> Normalized {
        match perspective {
            Player::Black => Normalized::new(self.0, perspective),
            Player::White => Normalized::new((BOARD_SIZE + 1) - self.0, perspective),
            Player::None => panic!("cannot normalize with perspective of `Player::None`"),
        }
        .unwrap()
    }

    /// Converts the given `DenormalizedLocation` to an `IndexLocation`.
    pub fn to_index(self) -> Result<Index, Error> {
        // Here to avoid underflow errors from unsigned subtraction.
        if self.0 == 0 || self.0 == (BOARD_SIZE + 1) {
            return Err(Error::InvalidIndexLocation(self.0));
        }

        (self.0 - 1).try_into()
    }
}

/// Tries to create a new `DenormalizedLocation` from a `usize` validating it is
/// contained within the range `0..=25`.
impl TryFrom<usize> for Denormalized {
    type Error = crate::backgammon::Error;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if (0..=25).contains(&value) {
            Ok(Self(value))
        } else {
            Err(Error::InvalidDenormalizedLocation(value))
        }
    }
}

/// Creates a `usize` from a `DenormalizedLocation`.
impl From<Denormalized> for usize {
    fn from(value: Denormalized) -> Self {
        value.0
    }
}

/// Gets the `usize` from a `DenormalizedLocation`.
impl Deref for Denormalized {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Represents a range from 0 to 23 where 0 is the `Player::Black` ace and 23 is
/// the `Player::White` ace. This position is not normalized and is absolute no
/// matter the perspective. It is used for indexing into `Board.points`.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Index(usize);

// #[allow(dead_code)]
impl Index {
    /// Normalize the given `IndexLocation` to a `Player` perspective.
    ///
    /// The value must be contained within range `0..=23` for `IndexLocation` to
    /// be instantiated from `usize`, thus when this method is called, the
    /// `NormalizedLocation` is guaranteed to be in range `0..=25`.
    pub fn normalize(self, perspective: Player) -> Normalized {
        match perspective {
            Player::Black => Normalized::new(self.0 + 1, perspective),
            Player::White => Normalized::new((BOARD_SIZE + 1) - (self.0 + 1), perspective),
            Player::None => panic!("cannot normalize with perspective of `Player::None`"),
        }
        .unwrap()
    }

    /// Denormalize the given `IndexLocation`.
    ///
    /// The value must be contained within range `0..=23` for `IndexLocation` to
    /// be instantiated from `usize`, thus when this method is called, the
    /// `DenormalizedLocation` is guaranteed to be in range `0..=25`.
    pub fn denormalize(self) -> Denormalized {
        (self.0 + 1).try_into().unwrap()
    }
}

/// Tries to create a new `IndexLocation` from a `usize` validating it is
/// contained within the range `0..=23`.
impl TryFrom<usize> for Index {
    type Error = crate::backgammon::Error;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if (0..=23).contains(&value) {
            Ok(Self(value))
        } else {
            Err(Error::InvalidIndexLocation(value))
        }
    }
}

/// Creates a `usize` from a `IndexLocation`.
impl From<Index> for usize {
    fn from(value: Index) -> Self {
        value.0
    }
}

/// Gets the `usize` from a `IndexLocation`.
impl Deref for Index {
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
            Normalized::new(27, Player::Black),
            Err(Error::InvalidNormalizedLocation(27, Player::Black))
        );
    }

    #[test]
    fn bad_normalized_position_2() {
        assert_eq!(
            Normalized::new(209, Player::White),
            Err(Error::InvalidNormalizedLocation(209, Player::White))
        );
    }

    #[test]
    fn bad_normalized_position_3() {
        assert_eq!(
            Normalized::new(10, Player::None),
            Err(Error::InvalidNormalizedLocation(10, Player::None))
        );
    }

    #[test]
    fn bad_denormalized_position_1() {
        assert_eq!(
            Denormalized::try_from(26),
            Err(Error::InvalidDenormalizedLocation(26))
        );
    }

    #[test]
    fn bad_denormalized_position_2() {
        assert_eq!(
            Denormalized::try_from(87),
            Err(Error::InvalidDenormalizedLocation(87))
        );
    }

    #[test]
    fn bad_index_position_1() {
        assert_eq!(Index::try_from(24), Err(Error::InvalidIndexLocation(24)));
    }

    #[test]
    fn bad_index_position_2() {
        assert_eq!(Index::try_from(67), Err(Error::InvalidIndexLocation(67)));
    }

    #[test]
    fn normalized_position_to_index_1() -> Result<(), Error> {
        assert_eq!(
            Normalized::new(1, Player::Black)?.to_index(),
            Ok(Index::try_from(0)?)
        );
        Ok(())
    }

    #[test]
    fn normalized_position_to_index_2() -> Result<(), Error> {
        assert_eq!(
            Normalized::new(20, Player::Black)?.to_index(),
            Ok(Index::try_from(19)?)
        );
        Ok(())
    }

    #[test]
    fn normalized_position_to_index_3() -> Result<(), Error> {
        assert_eq!(
            Normalized::new(24, Player::White)?.to_index(),
            Ok(Index::try_from(0)?)
        );
        Ok(())
    }

    #[test]
    fn normalized_position_to_index_4() -> Result<(), Error> {
        assert_eq!(
            Normalized::new(10, Player::White)?.to_index(),
            Ok(Index::try_from(14)?)
        );
        Ok(())
    }

    #[test]
    fn bad_normalized_position_to_index_1() -> Result<(), Error> {
        assert_eq!(
            Normalized::new(25, Player::Black)?.to_index(),
            Err(Error::InvalidIndexLocation(25))
        );
        Ok(())
    }

    #[test]
    fn bad_normalized_bad_position_to_index_2() -> Result<(), Error> {
        assert_eq!(
            Normalized::new(0, Player::Black)?.to_index(),
            Err(Error::InvalidIndexLocation(0))
        );
        Ok(())
    }

    #[test]
    fn bad_normalized_position_to_index_3() -> Result<(), Error> {
        assert_eq!(
            Normalized::new(25, Player::White)?.to_index(),
            Err(Error::InvalidIndexLocation(25))
        );
        Ok(())
    }

    #[test]
    fn bad_normalized_bad_position_to_index_4() -> Result<(), Error> {
        assert_eq!(
            Normalized::new(0, Player::White)?.to_index(),
            Err(Error::InvalidIndexLocation(0))
        );
        Ok(())
    }

    #[test]
    fn index_position_to_normalized_1() -> Result<(), Error> {
        assert_eq!(
            Index::try_from(0)?.normalize(Player::Black),
            Normalized::new(1, Player::Black)?
        );
        Ok(())
    }

    #[test]
    fn index_position_to_normalized_2() -> Result<(), Error> {
        assert_eq!(
            Index::try_from(19)?.normalize(Player::Black),
            Normalized::new(20, Player::Black)?
        );
        Ok(())
    }

    #[test]
    fn index_position_to_normalized_3() -> Result<(), Error> {
        assert_eq!(
            Index::try_from(23)?.normalize(Player::White),
            Normalized::new(1, Player::White)?
        );
        Ok(())
    }

    #[test]
    fn index_position_to_normalized_4() -> Result<(), Error> {
        assert_eq!(
            Index::try_from(14)?.normalize(Player::White),
            Normalized::new(10, Player::White)?
        );
        Ok(())
    }
}
