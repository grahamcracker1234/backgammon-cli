use colored::Colorize;
use itertools::Itertools;
use std::cell::RefCell;
use std::ops::RangeInclusive;

use super::{player::Player, Error};

pub(super) const BOARD_SIZE: usize = 24;

#[derive(Clone)]
pub(super) struct Board {
    points: [RefCell<Point>; BOARD_SIZE],
    totals: [u8; 2],
    bar: [RefCell<Point>; 2],
    off: [RefCell<Point>; 2],
}

impl Board {
    pub fn new() -> Self {
        let points: [_; BOARD_SIZE] = (0..BOARD_SIZE)
            .map(|i| RefCell::new(Point::new(i + 1, 0, Player::None)))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        // Sets black pieces.
        *points[23].borrow_mut() = Point::new(23 + 1, 2, Player::Black);
        *points[12].borrow_mut() = Point::new(12 + 1, 5, Player::Black);
        *points[7].borrow_mut() = Point::new(7 + 1, 3, Player::Black);
        *points[5].borrow_mut() = Point::new(5 + 1, 5, Player::Black);

        // Sets white pieces.
        let offset = BOARD_SIZE - 1;
        *points[offset - 23].borrow_mut() = Point::new(offset - 23 + 1, 2, Player::White);
        *points[offset - 12].borrow_mut() = Point::new(offset - 12 + 1, 5, Player::White);
        *points[offset - 7].borrow_mut() = Point::new(offset - 7 + 1, 3, Player::White);
        *points[offset - 5].borrow_mut() = Point::new(offset - 5 + 1, 5, Player::White);

        Self::from_points(points)
    }

    fn from_points(points: [RefCell<Point>; BOARD_SIZE]) -> Self {
        let totals = [
            points
                .iter()
                .filter_map(|p| match p.borrow().player {
                    Player::Black => Some(p.borrow().count),
                    _ => None,
                })
                .sum(),
            points
                .iter()
                .filter_map(|p| match p.borrow().player {
                    Player::White => Some(p.borrow().count),
                    _ => None,
                })
                .sum(),
        ];
        let bar = [
            RefCell::new(Point::new(BOARD_SIZE + 1, 0, Player::Black)),
            RefCell::new(Point::new(0, 0, Player::White)),
        ];
        let off = [
            RefCell::new(Point::new(0, 0, Player::Black)),
            RefCell::new(Point::new(BOARD_SIZE + 1, 0, Player::White)),
        ];

        Self {
            points,
            totals,
            bar,
            off,
        }
    }

    pub fn convert_index(notation_index: usize, perspective: Player) -> Result<usize, Error> {
        let index = match perspective {
            Player::White => (BOARD_SIZE).checked_sub(notation_index),
            _ => notation_index.checked_sub(1),
        };

        match index {
            Some(index) if index < BOARD_SIZE => Ok(index),
            _ => Err(Error::InvalidNotationPosition(notation_index)),
        }
    }

    pub fn convert_notation(index: usize, perspective: Player) -> Result<usize, Error> {
        let notation_index = match perspective {
            Player::White => (BOARD_SIZE).checked_sub(index),
            _ => index.checked_add(1),
        };

        match notation_index {
            Some(notation_index) if notation_index <= BOARD_SIZE && notation_index >= 1 => {
                Ok(notation_index)
            }
            _ => Err(Error::InvalidIndexPosition(index)),
        }
    }

    pub fn bar(&self, player: Player) -> &RefCell<Point> {
        &self.bar[player as usize]
    }

    pub fn off(&self, player: Player) -> &RefCell<Point> {
        &self.off[player as usize]
    }

    pub fn point(&self, index: usize) -> &RefCell<Point> {
        &self.points[index]
    }
}

impl std::fmt::Display for Board {
    #[allow(unstable_name_collisions)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let perspective = if f.alternate() {
            Player::White
        } else {
            Player::Black
        };

        fn fmt_point(point: &RefCell<Point>) -> String {
            let str = format!("{:#02}", point.borrow().count);
            let str = match point.borrow().player {
                Player::Black => str.on_black().white().bold(),
                Player::White => str.on_white().truecolor(0, 0, 0).bold(),
                Player::None => str.normal().dimmed(),
            };
            format!("{str}")
        }

        macro_rules! fmt_line {
            ($range:expr, $fn:expr, $rev:expr) => {{
                let mut vec = $range.map($fn).collect::<Vec<_>>();
                vec.insert(vec.len() / 2, '|'.to_string());
                if $rev {
                    vec.reverse();
                }
                vec.into_iter()
                    .intersperse(" ".to_string())
                    .collect::<String>()
            }};
        }

        let fmt_points = |range: RangeInclusive<usize>, rev: bool| -> String {
            fmt_line!(
                range,
                |i| fmt_point(&self.points[Self::convert_index(i, perspective).unwrap()]),
                rev
            )
        };

        fn fmt_indices(range: RangeInclusive<usize>, rev: bool) -> colored::ColoredString {
            fmt_line!(range, |i| format!("{i:#02}"), rev).bold()
        }

        let sep = "\n-- -- -- -- -- -- + -- -- -- -- -- --\n";

        write!(
            f,
            "{}{sep}{} Bar: {} {}{sep}{} Off: {} {}{sep}{}",
            fmt_indices(13..=24, !f.alternate()),
            fmt_points(13..=24, !f.alternate()),
            fmt_point(self.bar(perspective)),
            fmt_point(self.bar(!perspective)),
            fmt_points(1..=12, f.alternate()),
            fmt_point(self.off(perspective)),
            fmt_point(self.off(!perspective)),
            fmt_indices(1..=12, f.alternate()),
        )?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub(super) enum BoardPosition {
    Bar(Player),
    Off(Player),
    Point(usize),
}

impl BoardPosition {
    pub fn point<'a>(&self, board: &'a Board) -> &'a RefCell<Point> {
        match *self {
            BoardPosition::Bar(player) => board.bar(player),
            BoardPosition::Off(player) => board.off(player),
            BoardPosition::Point(index) => board.point(index),
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct Point {
    pub effective_pos: usize,
    pub count: u8,
    pub player: Player,
}

impl Point {
    pub fn new(effective_pos: usize, count: u8, player: Player) -> Self {
        Self {
            effective_pos,
            count,
            player,
        }
    }

    pub fn is_valid_direction(&self, to: &Point) -> bool {
        // println!("{}: {} -> {}", self.player, from_pos, to_pos);
        match self.player {
            Player::White => to.effective_pos > self.effective_pos,
            Player::Black => to.effective_pos < self.effective_pos,
            _ => panic!("There is no move direction for `Player::None`."),
        }
    }

    pub fn distance(&self, to: &Point) -> usize {
        self.effective_pos.abs_diff(to.effective_pos)
    }
}

// Alternate implementations, consider for the future.
// #[derive(Clone, Debug)]
// pub(super) enum PositionType {
//     Bar,
//     Off,
//     Point,
// }

// #[derive(Clone, Debug)]
// pub(super) struct Position {
//     pub effective_pos: usize,
//     pub count: u8,
//     pub player: Player,
//     pub r#type: PositionType,
// }

// impl Position {
//     pub fn new(effective_pos: usize, count: u8, player: Player, r#type: PositionType) -> Self {
//         Self {
//             effective_pos,
//             count,
//             player,
//             r#type,
//         }
//     }

//     pub fn is_valid_direction(&self, to: &Self) -> bool {
//         // println!("{}: {} -> {}", self.player, from_pos, to_pos);
//         match self.player {
//             Player::White => to.effective_pos > self.effective_pos,
//             Player::Black => to.effective_pos < self.effective_pos,
//             _ => panic!("There is no move direction for `Player::None`."),
//         }
//     }

//     pub fn distance(&self, to: &Self) -> usize {
//         self.effective_pos.abs_diff(to.effective_pos)
//     }
// }
