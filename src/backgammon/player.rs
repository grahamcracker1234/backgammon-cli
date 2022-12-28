use std::ops::Not;

#[derive(Clone, Copy, Eq, Debug, Hash, PartialEq)]
pub(super) enum Player {
    Black,
    White,
    None,
}

// impl Player {
//     fn are_opposites(a: Player, b: Player) -> bool {
//         (a == Player::Black && b == Player::White) || (a == Player::White && b == Player::Black)
//     }
// }

impl Player {
    pub fn switch(&mut self) {
        *self = self.not();
    }

    pub fn random() -> Self {
        if rand::random::<bool>() {
            Player::Black
        } else {
            Player::White
        }
    }
}

impl Not for Player {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::Black => Self::White,
            Self::White => Self::Black,
            Self::None => Self::None,
        }
    }
}
