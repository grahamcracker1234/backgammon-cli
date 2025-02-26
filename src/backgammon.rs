mod board;
mod dice;
mod game;
mod location;
mod notation;
mod player;

pub use game::Game;

use player::Player;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    InvalidNormalizedLocation(usize, Player),
    InvalidDenormalizedLocation(usize),
    InvalidIndexLocation(usize),
    InvalidNotation(String),
    InvalidPlayLength(u8),
    IncompleteTurn,
    PlayMadeOutOfTurn,
    PlayMadeWithBarFilled,
    PlayMadeToBar,
    PlayMadeFromRail,
    PlayMadeFromEmptyPoint,
    PlayMadeWithOpposingPiece,
    InvalidPlayDirection,
    PlayMadeOntoOpposingPiece,
    InvalidBearOff,
    NonMaximalTurn,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidNormalizedLocation(pos, player) => {
                write!(
                    f,
                    "cannot create `NormalizedLocation` of `{pos}` for `{player}`"
                )
            }
            Self::InvalidDenormalizedLocation(pos) => {
                write!(f, "cannot create `DenormalizedLocation` from `{pos}`")
            }
            Self::InvalidIndexLocation(pos) => {
                write!(f, "cannot create `IndexLocation` from `{pos}`")
            }
            Self::InvalidNotation(notation) => write!(f, "notation '{notation}' is not valid"),
            Self::InvalidPlayLength(len) => write!(f, "play of length '{len}' is not valid"),
            Self::IncompleteTurn => write!(f, "did not use all possible plays"),
            Self::PlayMadeOutOfTurn => write!(f, "only the current player can play"),
            Self::PlayMadeWithBarFilled => {
                write!(f, "attempted to play a piece while there is one in the bar")
            }
            Self::PlayMadeFromRail => {
                write!(f, "cannot play a piece from the rail after bearing it off")
            }
            Self::PlayMadeToBar => write!(f, "cannot play onto the bar"),
            Self::PlayMadeFromEmptyPoint => write!(f, "attempted to play nonexistent piece"),
            Self::PlayMadeWithOpposingPiece => {
                write!(f, "attempted to play another player's piece")
            }
            Self::InvalidPlayDirection => write!(f, "attempted to play backwards"),
            Self::PlayMadeOntoOpposingPiece => {
                write!(f, "attempted to illegally play onto another player")
            }
            Self::InvalidBearOff => {
                write!(f, "attempted to bear off without all pieces in home board")
            }
            Self::NonMaximalTurn => write!(f, "a turn must use as many moves as possible preferring larger moves if not all can be used")
        }
    }
}

impl std::error::Error for Error {}
