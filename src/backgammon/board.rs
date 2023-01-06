use colored::Colorize;
use itertools::Itertools;
use std::fmt::Debug;

use crate::backgammon::{
    location::{DenormalizedLocation, IndexLocation, NormalizedLocation},
    notation::PositionRef,
    player::Player,
};

pub(crate) const BOARD_SIZE: usize = 24;

const HOME_BOARD_INDEX: usize = 5;

#[derive(Debug, Clone)]
pub(crate) struct Board {
    points: [Position; BOARD_SIZE],
    bar: [Position; 2],
    rail: [Position; 2],
    // totals: [u8; 2],
}

impl Board {
    pub fn empty() -> Self {
        let points: [_; BOARD_SIZE] = (0..BOARD_SIZE)
            .map(|i| {
                Position::new(
                    IndexLocation::try_from(i).unwrap().denormalize(),
                    0,
                    Player::None,
                )
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        let bar = [
            Position::new(
                DenormalizedLocation::try_from(BOARD_SIZE + 1).unwrap(),
                0,
                Player::Black,
            ),
            Position::new(DenormalizedLocation::try_from(0).unwrap(), 0, Player::White),
        ];

        let rail = [
            Position::new(DenormalizedLocation::try_from(0).unwrap(), 0, Player::Black),
            Position::new(
                DenormalizedLocation::try_from(BOARD_SIZE + 1).unwrap(),
                0,
                Player::White,
            ),
        ];

        // let totals = [
        //     points
        //         .iter()
        //         .filter_map(|p| match p.borrow().player {
        //             Player::Black => Some(p.borrow().count),
        //             _ => None,
        //         })
        //         .sum(),
        //     points
        //         .iter()
        //         .filter_map(|p| match p.borrow().player {
        //             Player::White => Some(p.borrow().count),
        //             _ => None,
        //         })
        //         .sum(),
        // ];

        Self {
            points,
            bar,
            rail,
            // totals,
        }
    }

    pub fn new() -> Self {
        let mut board = Board::empty();

        // Sets black pieces.
        board.point_mut(23).set(2, Player::Black);
        board.point_mut(12).set(5, Player::Black);
        board.point_mut(7).set(3, Player::Black);
        board.point_mut(5).set(5, Player::Black);

        // Sets white pieces.
        let offset = BOARD_SIZE - 1;
        board.point_mut(offset - 23).set(2, Player::White);
        board.point_mut(offset - 12).set(5, Player::White);
        board.point_mut(offset - 7).set(3, Player::White);
        board.point_mut(offset - 5).set(5, Player::White);

        board
    }

    pub fn bar(&self, player: Player) -> &Position {
        &self.bar[player as usize]
    }

    pub fn rail(&self, player: Player) -> &Position {
        &self.rail[player as usize]
    }

    pub fn point(&self, index: usize) -> &Position {
        &self.points[index]
    }

    pub fn bar_mut(&mut self, player: Player) -> &mut Position {
        &mut self.bar[player as usize]
    }

    pub fn rail_mut(&mut self, player: Player) -> &mut Position {
        &mut self.rail[player as usize]
    }

    pub fn point_mut(&mut self, index: usize) -> &mut Position {
        &mut self.points[index]
    }
    // impl Space {
    //     pub fn position<'a>(&self, board: &'a Board) -> &'a Position {
    //         match *self {
    //             Space::Bar(player) => board.bar(player),
    //             Space::Rail(player) => board.rail(player),
    //             Space::Point(index) => board.point(*index),
    //         }
    //     }

    //     pub fn point_mut<'a>(&self, board: &'a mut Board) -> &'a mut Position {
    //         match *self {
    //             Space::Bar(player) => board.bar_mut(player),
    //             Space::Rail(player) => board.rail_mut(player),
    //             Space::Point(index) => board.point_mut(*index),
    //         }
    //     }
    // }
    pub fn get(&self, space: &PositionRef) -> &Position {
        match *space {
            PositionRef::Bar(player) => self.bar(player),
            PositionRef::Rail(player) => self.rail(player),
            PositionRef::Point(index) => self.point(*index),
        }
    }

    pub fn get_mut(&mut self, space: &PositionRef) -> &mut Position {
        match *space {
            PositionRef::Bar(player) => self.bar_mut(player),
            PositionRef::Rail(player) => self.rail_mut(player),
            PositionRef::Point(index) => self.point_mut(*index),
        }
    }

    pub fn any_behind(&self, index: usize, player: Player) -> bool {
        match player {
            Player::Black => &self.points[index + 1..],
            Player::White => &self.points[..index],
            Player::None => panic!("no pieces behind `None`"),
        }
        .iter()
        .any(|p| p.player == player && p.count > 0)
    }

    pub fn all_in_home(&self, player: Player) -> bool {
        let index = match player {
            Player::Black => HOME_BOARD_INDEX,
            Player::White => BOARD_SIZE - 1 - HOME_BOARD_INDEX,
            Player::None => panic!("no pieces behind `None`"),
        };

        !self.any_behind(index, player)
    }
}

impl std::fmt::Display for Board {
    #[allow(unstable_name_collisions)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let index_format = |i| format!("{i:02}");
        let point_format = |point: &Position| {
            let count = point.count;

            let str = if count == 0 {
                format!("{:2}", "░░")
            } else {
                format!("{count:02}")
            };

            let str = match point.player {
                Player::Black => str.on_black().white().bold(),
                Player::White => str.on_white().black().bold(),
                Player::None => str.normal().dimmed(),
            };
            format!("{str}")
        };

        let perspective = if f.alternate() {
            Player::White
        } else {
            Player::Black
        };

        macro_rules! bkgm_format {
            ($range:expr $(, $rev:ident)?) => {
                (bkgm_format!(index_format, $range $(, $rev)?), bkgm_format!(|i| {
                    point_format(&self.points[
                        *NormalizedLocation::new(i, perspective)
                            .unwrap()
                            .to_index()
                            .unwrap()
                    ])
                }, $range $(, $rev)?))
            };
            ($fmt:expr, $range:expr $(, $rev:ident)?) => {
                ($range).map($fmt)$(.$rev())?.intersperse(" ".to_string()).collect::<String>()
            };
        }

        if f.alternate() {
            let player_home = bkgm_format!(1..=6);
            let player_outer = bkgm_format!(7..=12);
            let opponent_outer = bkgm_format!(13..=18, rev);
            let opponent_home = bkgm_format!(19..=24, rev);

            write!(
                f,
                "┏━━━━┳━━━━━━━━━━━━━━━━━━━┳━━━━┳━━━━━━━━━━━━━━━━━━━┳━━━━┓\n\
                 ┃    ┃ {               } ┃    ┃ {               } ┃    ┃\n\
                 ┃    ┣━━━━━━━━━━━━━━━━━━━┫    ┣━━━━━━━━━━━━━━━━━━━┫    ┃\n\
                 ┃ {} ┃ {               } ┃ {} ┃ {               } ┃    ┃\n\
                 ┃    ┣━━━━━━━━━━━━━━━━━━━┫    ┣━━━━━━━━━━━━━━━━━━━┫    ┃\n\
                 ┃ {} ┃ {               } ┃ {} ┃ {               } ┃    ┃\n\
                 ┃    ┣━━━━━━━━━━━━━━━━━━━┫    ┣━━━━━━━━━━━━━━━━━━━┫    ┃\n\
                 ┃    ┃ {               } ┃    ┃ {               } ┃    ┃\n\
                 ┗━━━━┻━━━━━━━━━━━━━━━━━━━┻━━━━┻━━━━━━━━━━━━━━━━━━━┻━━━━┛",
                opponent_home.0,
                opponent_outer.0,
                point_format(self.rail(!perspective)),
                opponent_home.1,
                point_format(self.bar(perspective)),
                opponent_outer.1,
                point_format(self.rail(perspective)),
                player_home.1,
                point_format(self.bar(!perspective)),
                player_outer.1,
                player_home.0,
                player_outer.0,
            )
        } else {
            let player_home = bkgm_format!(1..=6, rev);
            let player_outer = bkgm_format!(7..=12, rev);
            let opponent_outer = bkgm_format!(13..=18);
            let opponent_home = bkgm_format!(19..=24);

            write!(
                f,
                "┏━━━━┳━━━━━━━━━━━━━━━━━━━┳━━━━┳━━━━━━━━━━━━━━━━━━━┳━━━━┓\n\
                 ┃    ┃ {               } ┃    ┃ {               } ┃    ┃\n\
                 ┃    ┣━━━━━━━━━━━━━━━━━━━┫    ┣━━━━━━━━━━━━━━━━━━━┫    ┃\n\
                 ┃    ┃ {               } ┃ {} ┃ {               } ┃ {} ┃\n\
                 ┃    ┣━━━━━━━━━━━━━━━━━━━┫    ┣━━━━━━━━━━━━━━━━━━━┫    ┃\n\
                 ┃    ┃ {               } ┃ {} ┃ {               } ┃ {} ┃\n\
                 ┃    ┣━━━━━━━━━━━━━━━━━━━┫    ┣━━━━━━━━━━━━━━━━━━━┫    ┃\n\
                 ┃    ┃ {               } ┃    ┃ {               } ┃    ┃\n\
                 ┗━━━━┻━━━━━━━━━━━━━━━━━━━┻━━━━┻━━━━━━━━━━━━━━━━━━━┻━━━━┛",
                opponent_outer.0,
                opponent_home.0,
                opponent_outer.1,
                point_format(self.bar(perspective)),
                opponent_home.1,
                point_format(self.rail(!perspective)),
                player_outer.1,
                point_format(self.bar(!perspective)),
                player_home.1,
                point_format(self.rail(perspective)),
                player_outer.0,
                player_home.0,
            )
        }
    }
}

impl PartialEq for Board {
    fn eq(&self, other: &Self) -> bool {
        let points_match = self
            .points
            .iter()
            .zip(other.points.iter())
            .all(|(a, b)| a == b);

        let bars_match = self.bar.iter().zip(other.bar.iter()).all(|(a, b)| a == b);

        let rails_match = self.rail.iter().zip(other.rail.iter()).all(|(a, b)| a == b);

        points_match && bars_match && rails_match
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Position {
    pub location: DenormalizedLocation,
    pub count: u8,
    pub player: Player,
}

impl Position {
    fn new(location: DenormalizedLocation, count: u8, player: Player) -> Self {
        Self {
            location,
            count,
            player,
        }
    }

    // pub fn is_valid_direction(&self, to: &Point) -> bool {
    //     match self.player {
    //         Player::White => to.position > self.position,
    //         Player::Black => to.position < self.position,
    //         _ => panic!("There is no move direction for `Player::None`."),
    //     }
    // }

    pub fn distance(&self, to: &Position) -> usize {
        self.location.abs_diff(*to.location)
    }

    pub fn set(&mut self, count: u8, player: Player) {
        self.count = count;
        self.player = player;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_in_home_1() {
        let player = Player::Black;
        let mut board = Board::empty();
        board.point_mut(5).set(1, player);
        println!("{board}");
        assert!(board.all_in_home(player));
    }

    #[test]
    fn all_in_home_2() {
        let player = Player::White;
        let mut board = Board::empty();
        board.point_mut(18).set(1, player);
        println!("{board}");
        assert!(board.all_in_home(player));
    }

    #[test]
    fn all_in_home_3() {
        let player = Player::Black;
        let mut board = Board::empty();
        board.point_mut(10).set(1, player);
        println!("{board}");
        assert!(!board.all_in_home(player));
    }

    #[test]
    fn all_in_home_4() {
        let player = Player::White;
        let mut board = Board::empty();
        board.point_mut(13).set(1, player);
        println!("{board}");
        assert!(!board.all_in_home(player));
    }
}
