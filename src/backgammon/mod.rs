mod board;
mod game;
mod player;
mod roll;
mod turn;

pub use game::Game;

#[derive(Debug)]
enum Error {
    InvalidNotationPosition(usize),
    InvalidIndexPosition(usize),
    InvalidNotation(String),
    InvalidMoveLength(u8),
    IncompleteTurn,
    MoveMadeOutOfTurn,
    MoveMadeWithBarFilled,
    MoveMadeToBar,
    MoveMadeFromBearingTable,
    MoveMadeFromEmptyPoint,
    MoveMadeWithOpposingPiece,
    InvalidMoveDirection,
    MoveMadeOntoOpposingPiece,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidNotationPosition(pos) => write!(f, "position '{pos}' is not valid"),
            Error::InvalidIndexPosition(pos) => write!(f, "position '{pos}' is not valid"),
            Error::InvalidNotation(notation) => write!(f, "notation '{notation}' is not valid"),
            Error::InvalidMoveLength(len) => write!(f, "move of length '{len}' is not valid"),
            Error::IncompleteTurn => write!(f, "did not use all possible moves"),
            Error::MoveMadeOutOfTurn => write!(f, "only the current player can move"),
            Error::MoveMadeWithBarFilled => {
                write!(f, "attempted to move a piece while there is one in the bar")
            }
            Error::MoveMadeFromBearingTable => {
                write!(f, "cannot move a piece after bearing it off")
            }
            Error::MoveMadeToBar => write!(f, "cannot move onto the bar"),
            Error::MoveMadeFromEmptyPoint => write!(f, "attempted to move nonexistent piece"),
            Error::MoveMadeWithOpposingPiece => {
                write!(f, "attempted to move another player's piece")
            }
            Error::InvalidMoveDirection => write!(f, "attempted to move backwards"),
            Error::MoveMadeOntoOpposingPiece => {
                write!(f, "attempted to illegally move onto another player")
            }
        }
    }
}

impl std::error::Error for Error {}
