use colored::Colorize;
use std::cell::RefCell;
use std::collections::HashMap;

use super::Player;

#[derive(Clone)]
pub(super) struct Board {
    pub points: [RefCell<Point>; 26],
    pub totals: HashMap<Player, u8>,
}

impl Board {
    pub fn new() -> Self {
        let mut points = [Point::new(0, Player::None); 26];
        // Sets black pieces.
        points[0] = Point::new(0, Player::Black);
        points[1] = Point::new(2, Player::Black);
        points[12] = Point::new(5, Player::Black);
        points[17] = Point::new(3, Player::Black);
        points[19] = Point::new(5, Player::Black);

        // Sets white pieces.
        points[25 - 0] = Point::new(0, Player::White);
        points[25 - 1] = Point::new(2, Player::White);
        points[25 - 12] = Point::new(5, Player::White);
        points[25 - 17] = Point::new(3, Player::White);
        points[25 - 19] = Point::new(5, Player::White);

        Self::from_points(
            points
                .iter()
                .map(|&p| RefCell::new(p))
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        )
    }

    fn from_points(points: [RefCell<Point>; 26]) -> Self {
        let mut totals = HashMap::new();

        totals.insert(
            Player::Black,
            points
                .iter()
                .filter(|p| p.borrow().player == Player::Black)
                .map(|p| p.borrow().count)
                .sum(),
        );
        totals.insert(
            Player::White,
            points
                .iter()
                .filter(|p| p.borrow().player == Player::White)
                .map(|p| p.borrow().count)
                .sum(),
        );

        Self {
            points: points,
            totals,
        }
    }
}

impl std::fmt::Debug for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fmt_point = |f: &mut std::fmt::Formatter<'_>, point: &RefCell<Point>| {
            let str = format!("{:#02}", point.borrow().count);
            let str = match point.borrow().player {
                Player::Black => str.on_black().white().bold(),
                Player::White => str.on_white().truecolor(0, 0, 0).bold(),
                Player::None => str.normal().dimmed(),
            };
            write!(f, "{} ", str)
        };

        let sep = "-- -- -- -- -- -- + -- -- -- -- -- --";

        write!(
            f,
            "{} | {}\n{}\n",
            "13 14 15 16 17 18".bold(),
            "19 20 21 22 23 24".bold(),
            sep
        )?;

        for point in &self.points[13..=18] {
            fmt_point(f, point)?;
        }

        write!(f, "| ")?;

        for point in &self.points[19..=24] {
            fmt_point(f, point)?;
        }

        write!(f, "\n{} Bar: ", sep)?;

        fmt_point(f, &self.points[0])?;
        fmt_point(f, &self.points[25])?;

        write!(f, "\n")?;

        for point in self.points[7..=12].into_iter().rev() {
            fmt_point(f, point)?;
        }

        write!(f, "| ")?;

        for point in self.points[1..=6].into_iter().rev() {
            fmt_point(f, point)?;
        }

        write!(
            f,
            "\n{}\n{} | {}",
            sep,
            "12 11 10 09 08 07".bold(),
            "06 05 04 03 02 01".bold(),
        )?;

        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Point {
    pub count: u8,
    pub player: Player,
}

impl Point {
    pub fn new(count: u8, player: Player) -> Self {
        Self { count, player }
    }
}
