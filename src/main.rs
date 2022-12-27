use colored::Colorize;
use std::collections::HashMap;

#[derive(Clone)]
struct Board {
    points: [Point; 26],
    current_turn: Color,
    totals: HashMap<Color, u8>,
}

impl Board {
    fn new() -> Self {
        let mut points = [Point::new(0, Color::None); 26];
        // Sets black pieces.
        points[0] = Point::new(0, Color::Black);
        points[1] = Point::new(2, Color::Black);
        points[12] = Point::new(5, Color::Black);
        points[17] = Point::new(3, Color::Black);
        points[19] = Point::new(5, Color::Black);

        // Sets white pieces.
        points[25 - 0] = Point::new(0, Color::White);
        points[25 - 1] = Point::new(2, Color::White);
        points[25 - 12] = Point::new(5, Color::White);
        points[25 - 17] = Point::new(3, Color::White);
        points[25 - 19] = Point::new(5, Color::White);

        Self::from_points(points)
    }

    fn from_points(points: [Point; 26]) -> Self {
        let mut totals = HashMap::new();

        totals.insert(
            Color::Black,
            points
                .into_iter()
                .filter(|p| p.color == Color::Black)
                .map(|p| p.count)
                .sum(),
        );
        totals.insert(
            Color::White,
            points
                .into_iter()
                .filter(|p| p.color == Color::White)
                .map(|p| p.count)
                .sum(),
        );

        Self {
            points,
            current_turn: Color::None,
            totals,
        }
    }

    fn start(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let rolls = (|| loop {
            let roll1: u8 = rng.gen_range(1..=6);
            let roll2: u8 = rng.gen_range(1..=6);
            if roll1 != roll2 {
                return [roll1, roll2];
            }
        })();

        self.current_turn = if rand::thread_rng().gen_bool(0.5) {
            Color::Black
        } else {
            Color::White
        };

        // println!("test {:?}", self.current_turn);

        loop {
            println!("{:?}", self);
            unsafe {
                if let Ok([move1, move2]) = Self::get_moves(&mut *(self as *mut _)) {
                    Self::make_valid_move(&mut *(self as *mut _), move1);
                    Self::make_valid_move(&mut *(self as *mut _), move2);
                } else {
                    println!("Invalid input, try again.");
                }
            }
        }
    }

    fn get_moves(&mut self) -> Result<[Move; 2], &str> {
        use std::{io, io::Write};
        // println!("{:?}", self.current_turn);

        print!(
            "{} to move: ",
            if self.current_turn == Color::Black {
                "Black"
            } else if self.current_turn == Color::White {
                "White"
            } else {
                panic!("Attempting to get moves from '{:?}'.", self.current_turn)
            }
        );
        io::stdout()
            .flush()
            .expect("Failed to flush standard output.");
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");

        println!("{:?}", input);

        let re = regex::Regex::new(r"\s*(\d+)\s*/\s*(\d+)\s*(\d+)*\s*/\s*(\d+)")
            .expect("Regex is invalid");

        let captures = re.captures(&input).ok_or("Input is not parsable")?;

        let is_in_bounds = |i: usize| i >= 0 && i < self.points.len();

        let mut nums = (1..captures.len())
            .into_iter()
            .filter_map(|i| captures.get(i))
            .map(|m| m.as_str().parse::<usize>().unwrap())
            .filter(|&i| is_in_bounds(i))
            .collect::<Vec<_>>();

        println!("{:?}", nums);

        let [a, b, c, d]: [usize; 4] = (if nums.len() == 3 {
            nums.insert(1, *nums.get(1).unwrap());
            nums
        } else {
            nums
        })
        .try_into()
        .or(Err("Input is not properly formatted"))?;

        if (self.current_turn == Color::White && (b >= a || d >= c))
            || (self.current_turn == Color::Black && (a >= b || c >= d))
        {
            return Err("Move is not valid.");
        }

        unsafe {
            let move1 = Move::new(
                self.current_turn,
                &mut *(self.points.get_unchecked_mut(a) as *mut _),
                &mut *(self.points.get_unchecked_mut(b) as *mut _),
            );

            let move2 = Move::new(
                self.current_turn,
                &mut *(self.points.get_unchecked_mut(c) as *mut _),
                &mut *(self.points.get_unchecked_mut(d) as *mut _),
            );

            println!("{}/{} {}/{}", a, b, c, d);

            let board = &mut self.clone();

            if !Self::is_move_valid(
                &mut *(board as *mut _),
                &Move::new(
                    self.current_turn,
                    &mut *(board.points.get_unchecked_mut(a) as *mut _),
                    &mut *(board.points.get_unchecked_mut(b) as *mut _),
                ),
            ) {
                return Err("Move is not valid.");
            }

            Self::make_valid_move(
                &mut *(board as *mut _),
                Move::new(
                    self.current_turn,
                    &mut *(board.points.get_unchecked_mut(a) as *mut _),
                    &mut *(board.points.get_unchecked_mut(b) as *mut _),
                ),
            );

            println!("{:?}", board);

            if !Self::is_move_valid(
                &mut *(board as *mut _),
                &Move::new(
                    self.current_turn,
                    &mut *(board.points.get_unchecked_mut(c) as *mut _),
                    &mut *(board.points.get_unchecked_mut(d) as *mut _),
                ),
            ) {
                return Err("Move is not valid.");
            }

            println!("test");

            Ok([move1, move2])
        }
    }

    fn make_valid_move(&mut self, r#move: Move) {
        if !self.is_move_valid(&r#move) {
            panic!("Move is invalid");
        }

        r#move.from.count -= 1;
        if r#move.from.count == 0 {
            r#move.from.color = Color::None;
        }

        if Color::are_opposites(r#move.to.color, r#move.color) && r#move.to.count == 1 {
            if r#move.color == Color::Black {
                self.points[0].count += 1;
            } else if r#move.color == Color::White {
                self.points[25].count += 1;
            }
        }

        r#move.to.color = r#move.color;
        r#move.to.count += 1;
    }

    fn is_move_valid(&self, r#move: &Move) -> bool {
        println!("1");
        if self.current_turn != r#move.color {
            return false;
        }

        println!("2");
        if r#move.from.count <= 0 {
            return false;
        }

        println!("3");
        if r#move.from.color != r#move.color {
            return false;
        }

        println!("4");
        if Color::are_opposites(r#move.to.color, r#move.color) && r#move.to.count > 1 {
            return false;
        }

        println!("5");
        if r#move.to.color != r#move.color && r#move.to.color != Color::None {
            return false;
        }

        println!("6");
        return true;
    }
}

impl std::fmt::Debug for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fmt_point = |f: &mut std::fmt::Formatter<'_>, point: &Point| {
            let str = format!("{:#02}", point.count);
            let str = match point.color {
                Color::Black => str.on_black().white().bold(),
                Color::White => str.on_white().truecolor(0, 0, 0).bold(),
                Color::None => str.normal().dimmed(),
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

        write!(f, "\n{}\n", sep)?;

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

struct Move<'a> {
    color: Color,
    from: &'a mut Point,
    to: &'a mut Point,
}

impl<'a> Move<'a> {
    fn new(color: Color, from: &'a mut Point, to: &'a mut Point) -> Self {
        Self { color, from, to }
    }
}

#[derive(Clone, Copy, Eq, Debug, Hash, PartialEq)]
enum Color {
    Black,
    White,
    None,
}

impl Color {
    fn are_opposites(a: Color, b: Color) -> bool {
        (a == Color::Black && b == Color::White) || (a == Color::White && b == Color::Black)
    }
}

#[derive(Clone, Copy)]
struct Point {
    count: u8,
    color: Color,
}

impl Point {
    fn new(count: u8, color: Color) -> Self {
        Self { count, color }
    }
}

fn main() {
    let mut board = Board::new();
    board.start();
}
