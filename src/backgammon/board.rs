use colored::Colorize;
use itertools::Itertools;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::RangeInclusive;

use super::player::Player;

pub(super) const BOARD_SIZE: usize = 24;

#[derive(Clone)]
pub(super) struct Board {
    points: [RefCell<FixedPoint>; BOARD_SIZE],
    pub totals: HashMap<Player, u8>,
    pub bar: HashMap<Player, RefCell<FreePoint>>,
    pub off: HashMap<Player, RefCell<FreePoint>>,
}

impl Board {
    pub fn new() -> Self {
        let mut points: [FixedPoint; BOARD_SIZE] = (0..BOARD_SIZE)
            .map(|i| FixedPoint::new(i, 0, Player::None))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        // Sets black pieces.
        points[23] = FixedPoint::new(23, 2, Player::Black);
        points[12] = FixedPoint::new(12, 5, Player::Black);
        points[7] = FixedPoint::new(7, 3, Player::Black);
        points[5] = FixedPoint::new(5, 5, Player::Black);

        // Sets white pieces.
        let index_offset = BOARD_SIZE - 1;
        points[index_offset - 23] = FixedPoint::new(index_offset - 23, 2, Player::White);
        points[index_offset - 12] = FixedPoint::new(index_offset - 12, 5, Player::White);
        points[index_offset - 7] = FixedPoint::new(index_offset - 7, 3, Player::White);
        points[index_offset - 5] = FixedPoint::new(index_offset - 5, 5, Player::White);

        Self::from_points(
            points
                .iter()
                .map(|&p| RefCell::new(p))
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        )
    }

    fn from_points(points: [RefCell<FixedPoint>; BOARD_SIZE]) -> Self {
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
        bar.insert(
            Player::Black,
            RefCell::new(FreePoint::new(0, Player::Black)),
        );
        bar.insert(
            Player::White,
            RefCell::new(FreePoint::new(0, Player::White)),
        );

        let mut off = HashMap::new();
        off.insert(
            Player::Black,
            RefCell::new(FreePoint::new(0, Player::Black)),
        );
        off.insert(
            Player::White,
            RefCell::new(FreePoint::new(0, Player::White)),
        );

        Self {
            points,
            totals,
            bar,
            off,
        }
    }

    pub fn get_point(
        &self,
        index: usize,
        perspective: Player,
    ) -> Result<&RefCell<FixedPoint>, &'static str> {
        let error = "Position is not valid.";
        match perspective {
            Player::White => self
                .points
                .get((BOARD_SIZE - 1).checked_sub(index).ok_or(error)?),
            _ => self.points.get(index),
        }
        .ok_or(error)
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

        fn fmt_point(point: &RefCell<impl Point>) -> String {
            let str = format!("{:#02}", point.borrow().count());
            let str = match point.borrow().player() {
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
                |i| fmt_point(self.get_point(i, perspective).unwrap()),
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
            fmt_points(12..=23, !f.alternate()),
            fmt_point(&self.bar[&perspective]),
            fmt_point(&self.bar[&!perspective]),
            fmt_points(0..=11, f.alternate()),
            fmt_point(&self.off[&perspective]),
            fmt_point(&self.off[&!perspective]),
            fmt_indices(1..=12, f.alternate()),
        )?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub(super) enum BoardPosition<'a> {
    Bar(&'a RefCell<FreePoint>),
    Off(&'a RefCell<FreePoint>),
    Point(&'a RefCell<FixedPoint>),
}

impl<'a> BoardPosition<'a> {
    pub fn effective_pos(&self) -> usize {
        let error = "There is no effective position for `Player::None`.";
        match self {
            BoardPosition::Off(off) => match off.borrow().player {
                Player::Black => 0,
                Player::White => BOARD_SIZE + 1,
                _ => panic!("{error}"),
            },
            BoardPosition::Bar(bar) => match bar.borrow().player {
                Player::Black => BOARD_SIZE + 1,
                Player::White => 0,
                _ => panic!("{error}"),
            },
            BoardPosition::Point(point) => *point.borrow().pos() + 1,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct FreePoint {
    count: u8,
    player: Player,
}

impl FreePoint {
    pub fn new(count: u8, player: Player) -> Self {
        Self { count, player }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct FixedPoint {
    pos: usize,
    count: u8,
    player: Player,
}

impl FixedPoint {
    pub fn new(pos: usize, count: u8, player: Player) -> Self {
        Self { pos, count, player }
    }

    pub fn pos(&self) -> &usize {
        &self.pos
    }
}

pub(super) trait Point {
    fn count(&self) -> &u8;
    fn count_mut(&mut self) -> &mut u8;

    fn player(&self) -> &Player;
    fn player_mut(&mut self) -> &mut Player;
}

macro_rules! impl_point {
    ($type:ty) => {
        impl Point for $type {
            fn count(&self) -> &u8 {
                &self.count
            }

            fn count_mut(&mut self) -> &mut u8 {
                &mut self.count
            }

            fn player(&self) -> &Player {
                &self.player
            }

            fn player_mut(&mut self) -> &mut Player {
                &mut self.player
            }
        }
    };
}

impl_point!(FixedPoint);
impl_point!(FreePoint);

// macro_rules! point {
//     () => {};
// }

// pub(super) use point;
