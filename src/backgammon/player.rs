#[derive(Clone, Copy, Eq, Debug, Hash, PartialEq)]
pub(super) enum Player {
    Black = 0,
    White = 1,
    None = 2,
}

impl Player {
    pub fn switch(&mut self) {
        *self = !*self;
    }

    pub fn random() -> Self {
        if rand::random::<bool>() {
            Player::Black
        } else {
            Player::White
        }
    }
}

impl std::ops::Not for Player {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::Black => Self::White,
            Self::White => Self::Black,
            Self::None => Self::None,
        }
    }
}

impl std::fmt::Display for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Player::Black => "Black",
            Player::White => "White",
            Player::None => "None",
        })
    }
}
