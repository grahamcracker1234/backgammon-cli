mod board;
mod dice_roll;
mod game;
mod location;
mod notation;
mod player;

pub use game::Game;

use player::Player;
use thiserror::Error;

#[derive(Debug, PartialEq, Eq, Error)]
pub enum Error {
    #[error("cannot create `NormalizedLocation` of `{0}` for `{1}`")]
    InvalidNormalizedLocation(usize, Player),

    #[error("cannot create `DenormalizedLocation` from `{0}`")]
    InvalidDenormalizedLocation(usize),

    #[error("cannot create `IndexLocation` from `{0}`")]
    InvalidIndexLocation(usize),

    #[error("notation '{0}' is not valid")]
    InvalidNotation(String),

    #[error("play of length '{0}' is not valid")]
    InvalidPlayLength(u8),

    #[error("did not use all possible plays")]
    IncompleteTurn,

    #[error("only the current player can play")]
    PlayMadeOutOfTurn,

    #[error("attempted to play a piece while there is one in the bar")]
    PlayMadeWithBarFilled,

    #[error("cannot play a piece from the rail after bearing it off")]
    PlayMadeFromRail,

    #[error("cannot play onto the bar")]
    PlayMadeToBar,

    #[error("attempted to play nonexistent piece")]
    PlayMadeFromEmptyPoint,

    #[error("attempted to play another player's piece")]
    PlayMadeWithOpposingPiece,

    #[error("attempted to play backwards")]
    InvalidPlayDirection,

    #[error("attempted to illegally play onto another player")]
    PlayMadeOntoOpposingPiece,

    #[error("attempted to bear off without all pieces in home board")]
    InvalidBearOff,

    #[error(
        "a turn must use as many moves as possible, preferring larger moves if not all can be used"
    )]
    NonMaximalTurn,
}
