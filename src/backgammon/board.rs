use colored::Colorize;
use itertools::Itertools;
use std::cell::RefCell;
use std::fmt::Debug;

use crate::backgammon::{
    player::Player,
    position::{DenormalizedPosition, IndexPosition, NormalizedPosition},
};

pub(crate) const BOARD_SIZE: usize = 24;

const HOME_BOARD_INDEX: usize = 5;

#[derive(Debug, Clone)]
pub(crate) struct Board {
    points: [RefCell<Point>; BOARD_SIZE],
    bar: [RefCell<Point>; 2],
    rail: [RefCell<Point>; 2],
    // totals: [u8; 2],
}

impl Board {
    pub fn empty() -> Self {
        let points: [_; BOARD_SIZE] = (0..BOARD_SIZE)
            .map(|i| {
                RefCell::new(Point::new(
                    IndexPosition::try_from(i).unwrap().denormalize(),
                    0,
                    Player::None,
                ))
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        let bar = [
            RefCell::new(Point::new(
                DenormalizedPosition::try_from(BOARD_SIZE + 1).unwrap(),
                0,
                Player::Black,
            )),
            RefCell::new(Point::new(
                DenormalizedPosition::try_from(0).unwrap(),
                0,
                Player::White,
            )),
        ];

        let rail = [
            RefCell::new(Point::new(
                DenormalizedPosition::try_from(0).unwrap(),
                0,
                Player::Black,
            )),
            RefCell::new(Point::new(
                DenormalizedPosition::try_from(BOARD_SIZE + 1).unwrap(),
                0,
                Player::White,
            )),
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
        let board = Board::empty();

        // Sets black pieces.
        board.point(23).borrow_mut().set(2, Player::Black);
        board.point(12).borrow_mut().set(5, Player::Black);
        board.point(7).borrow_mut().set(3, Player::Black);
        board.point(5).borrow_mut().set(5, Player::Black);

        // Sets white pieces.
        let offset = BOARD_SIZE - 1;
        board.point(offset - 23).borrow_mut().set(2, Player::White);
        board.point(offset - 12).borrow_mut().set(5, Player::White);
        board.point(offset - 7).borrow_mut().set(3, Player::White);
        board.point(offset - 5).borrow_mut().set(5, Player::White);

        board
    }

    pub fn bar(&self, player: Player) -> &RefCell<Point> {
        &self.bar[player as usize]
    }

    pub fn rail(&self, player: Player) -> &RefCell<Point> {
        &self.rail[player as usize]
    }

    pub fn point(&self, index: usize) -> &RefCell<Point> {
        &self.points[index]
    }

    pub fn any_behind(&self, index: usize, player: Player) -> bool {
        match player {
            Player::Black => &self.points[index + 1..],
            Player::White => &self.points[..index],
            Player::None => panic!("no pieces behind `None`"),
        }
        .iter()
        .any(|p| p.borrow().player == player && p.borrow().count > 0)
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
        let point_format = |point: &RefCell<Point>| {
            let point = point.borrow();
            let count = point.count;

            let str = if count == 0 {
                format!("{:2}", "░░")
            } else {
                format!("{:02}", count)
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
                        *NormalizedPosition::new(i, perspective)
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
            .all(|(a, b)| *a.borrow() == *b.borrow());

        let bars_match = self
            .bar
            .iter()
            .zip(other.bar.iter())
            .all(|(a, b)| *a.borrow() == *b.borrow());

        let rails_match = self
            .rail
            .iter()
            .zip(other.rail.iter())
            .all(|(a, b)| *a.borrow() == *b.borrow());

        points_match && bars_match && rails_match
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Space {
    Bar(Player),
    Rail(Player),
    Point(IndexPosition),
}

impl Space {
    pub fn point<'a>(&self, board: &'a Board) -> &'a RefCell<Point> {
        match *self {
            Space::Bar(player) => board.bar(player),
            Space::Rail(player) => board.rail(player),
            Space::Point(index) => board.point(*index),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Point {
    pub position: DenormalizedPosition,
    pub count: u8,
    pub player: Player,
}

impl Point {
    fn new(position: DenormalizedPosition, count: u8, player: Player) -> Self {
        Self {
            position,
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

    pub fn distance(&self, to: &Point) -> usize {
        self.position.abs_diff(*to.position)
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
        let board = Board::empty();
        board.point(5).borrow_mut().set(1, player);
        println!("{board}");
        assert!(board.all_in_home(player));
    }

    #[test]
    fn all_in_home_2() {
        let player = Player::White;
        let board = Board::empty();
        board.point(18).borrow_mut().set(1, player);
        println!("{board}");
        assert!(board.all_in_home(player));
    }

    #[test]
    fn all_in_home_3() {
        let player = Player::Black;
        let board = Board::empty();
        board.point(10).borrow_mut().set(1, player);
        println!("{board}");
        assert!(!board.all_in_home(player));
    }

    #[test]
    fn all_in_home_4() {
        let player = Player::White;
        let board = Board::empty();
        board.point(13).borrow_mut().set(1, player);
        println!("{board}");
        assert!(!board.all_in_home(player));
    }
}
