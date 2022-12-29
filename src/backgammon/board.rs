use colored::Colorize;
use itertools::Itertools;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::RangeInclusive;

use super::Player;

const BOARD_SIZE: usize = 24;

#[derive(Clone)]
pub(super) struct Board {
    points: [RefCell<Point>; BOARD_SIZE],
    pub totals: HashMap<Player, u8>,
    pub bar: HashMap<Player, RefCell<Point>>,
    pub off: HashMap<Player, RefCell<Point>>,
}

impl Board {
    pub fn new() -> Self {
        let mut points: [Point; BOARD_SIZE] = (0..BOARD_SIZE)
            .map(|i| Point::new(i, 0, Player::None))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        // Sets black pieces.
        points[23] = Point::new(23, 2, Player::Black);
        points[12] = Point::new(12, 5, Player::Black);
        points[7] = Point::new(7, 3, Player::Black);
        points[5] = Point::new(5, 5, Player::Black);

        // Sets white pieces.
        let index_offset = BOARD_SIZE - 1;
        points[index_offset - 23] = Point::new(index_offset - 23, 2, Player::White);
        points[index_offset - 12] = Point::new(index_offset - 12, 5, Player::White);
        points[index_offset - 7] = Point::new(index_offset - 7, 3, Player::White);
        points[index_offset - 5] = Point::new(index_offset - 5, 5, Player::White);

        Self::from_points(
            points
                .iter()
                .map(|&p| RefCell::new(p))
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        )
    }

    fn from_points(points: [RefCell<Point>; BOARD_SIZE]) -> Self {
        let mut totals = HashMap::new();

        totals.insert(
            Player::Black,
            points
                .iter()
                .filter(|&p| p.borrow().player == Player::Black)
                .map(|p| p.borrow().count)
                .sum(),
        );
        totals.insert(
            Player::White,
            points
                .iter()
                .filter(|&p| p.borrow().player == Player::White)
                .map(|p| p.borrow().count)
                .sum(),
        );

        let mut bar = HashMap::new();
        bar.insert(Player::Black, RefCell::new(Point::new(0, 0, Player::Black)));
        bar.insert(Player::White, RefCell::new(Point::new(0, 0, Player::White)));

        let mut off = HashMap::new();
        off.insert(Player::Black, RefCell::new(Point::new(0, 0, Player::Black)));
        off.insert(Player::White, RefCell::new(Point::new(0, 0, Player::White)));

        Self {
            points,
            totals,
            bar,
            off,
        }
    }

    pub fn get_point(&self, index: usize, perspective: Player) -> &RefCell<Point> {
        match perspective {
            Player::White => &self.points[BOARD_SIZE - 1 - index],
            _ => &self.points[index],
        }
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

        let fmt_point = |point: &RefCell<Point>| {
            let str = format!("{:#02}", point.borrow().count);
            let str = match point.borrow().player {
                Player::Black => str.on_black().white().bold(),
                Player::White => str.on_white().truecolor(0, 0, 0).bold(),
                Player::None => str.normal().dimmed(),
            };
            format!("{}", str)
        };

        let fmt_points = |range: RangeInclusive<usize>, rev: bool| {
            let iter = range.map(|i| fmt_point(self.get_point(i, perspective)));
            if rev {
                iter.rev().intersperse(" ".to_string()).collect::<String>()
            } else {
                iter.intersperse(" ".to_string()).collect::<String>()
            }
        };

        let sep = "\n-- -- -- -- -- -- + -- -- -- -- -- --\n";

        write!(
            f,
            "{} | {}{sep}{} | {} Bar: {} {}{sep}{} | {} Off: {} {}{sep}{} | {}",
            "13 14 15 16 17 18".bold(),
            "19 20 21 22 23 24".bold(),
            fmt_points(12..=17, false),
            fmt_points(18..=23, false),
            fmt_point(&self.bar[&perspective]),
            fmt_point(&self.bar[&!perspective]),
            fmt_points(6..=11, true),
            fmt_points(0..=5, true),
            fmt_point(&self.off[&perspective]),
            fmt_point(&self.off[&!perspective]),
            "12 11 10 09 08 07".bold(),
            "06 05 04 03 02 01".bold(),
        )?;

        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Point {
    pub pos: usize,
    pub count: u8,
    pub player: Player,
}

impl Point {
    pub fn new(pos: usize, count: u8, player: Player) -> Self {
        Self { pos, count, player }
    }
}
