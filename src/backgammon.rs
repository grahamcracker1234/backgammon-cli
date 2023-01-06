mod board;
mod dice;
mod game;
mod location;
mod notation;
mod player;

pub use game::Game;

use player::Player;

#[derive(Debug, PartialEq)]
pub(crate) enum Error {
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
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidNormalizedLocation(pos, player) => {
                write!(
                    f,
                    "cannot create `NormalizedLocation` of `{pos}` for `{player}`"
                )
            }
            Error::InvalidDenormalizedLocation(pos) => {
                write!(f, "cannot create `DenormalizedLocation` from `{pos}`")
            }
            Error::InvalidIndexLocation(pos) => {
                write!(f, "cannot create `IndexLocation` from `{pos}`")
            }
            Error::InvalidNotation(notation) => write!(f, "notation '{notation}' is not valid"),
            Error::InvalidPlayLength(len) => write!(f, "play of length '{len}' is not valid"),
            Error::IncompleteTurn => write!(f, "did not use all possible plays"),
            Error::PlayMadeOutOfTurn => write!(f, "only the current player can play"),
            Error::PlayMadeWithBarFilled => {
                write!(f, "attempted to play a piece while there is one in the bar")
            }
            Error::PlayMadeFromRail => {
                write!(f, "cannot play a piece from the rail after bearing it off")
            }
            Error::PlayMadeToBar => write!(f, "cannot play onto the bar"),
            Error::PlayMadeFromEmptyPoint => write!(f, "attempted to play nonexistent piece"),
            Error::PlayMadeWithOpposingPiece => {
                write!(f, "attempted to play another player's piece")
            }
            Error::InvalidPlayDirection => write!(f, "attempted to play backwards"),
            Error::PlayMadeOntoOpposingPiece => {
                write!(f, "attempted to illegally play onto another player")
            }
            Error::InvalidBearOff => {
                write!(f, "attempted to bear off without all pieces in home board")
            }
        }
    }
}

impl std::error::Error for Error {}
