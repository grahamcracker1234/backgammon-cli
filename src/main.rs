use colored::Colorize;
use itertools::Itertools;
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Clone)]
struct Board {
    points: [RefCell<Point>; 26],
    current_turn: RefCell<Color>,
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
            Color::Black,
            points
                .iter()
                .filter(|p| p.borrow().color == Color::Black)
                .map(|p| p.borrow().count)
                .sum(),
        );
        totals.insert(
            Color::White,
            points
                .iter()
                .filter(|p| p.borrow().color == Color::White)
                .map(|p| p.borrow().count)
                .sum(),
        );

        Self {
            points: points,
            current_turn: RefCell::new(Color::None),
            totals,
        }
    }

    fn start(&self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let rolls = (|| loop {
            let roll1: u8 = rng.gen_range(1..=6);
            let roll2: u8 = rng.gen_range(1..=6);
            if roll1 != roll2 {
                return [roll1, roll2];
            }
        })();

        *self.current_turn.borrow_mut() = if rand::thread_rng().gen_bool(0.5) {
            Color::Black
        } else {
            Color::White
        };

        loop {
            println!("{:?}", self);
            if let Ok(turn) = self.get_turn() {
                for mut r#move in turn.moves {
                    self.make_valid_move(&mut r#move);
                }
                // self.make_valid_move(&mut move2);
            } else {
                println!("Invalid input, try again.");
            }
        }
    }

    fn get_turn(&self) -> Result<Turn, &str> {
        use std::{io, io::Write};
        // println!("{:?}", self.current_turn);

        print!(
            "{} to move: ",
            if *self.current_turn.borrow() == Color::Black {
                "Black"
            } else if *self.current_turn.borrow() == Color::White {
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

        // println!("{:?}", input);

        let re = regex::Regex::new(r"\s*(\d+)\s*/\s*(\d+)\s*(\d+)*\s*/\s*(\d+)")
            .expect("Regex is invalid");

        // for r#match in regex::Regex::new(r"\d+(?:/\d+)+")
        //     .expect("Regex is invalid")
        //     .find_iter(&input)
        // {
        //     println!(
        //         "{:?}",
        //         r#match
        //             .as_str()
        //             .split('/')
        //             .map(|m| m.parse::<usize>().unwrap())
        //             .collect::<Vec<_>>()
        //     );
        // }

        let moves = regex::Regex::new(r"\d+(?:/\d+)+")
            .expect("Regex is invalid")
            .find_iter(&input)
            .flat_map(|m| {
                m.as_str()
                    .split('/')
                    .map(|m| m.parse::<usize>().unwrap())
                    .tuple_windows()
                    .map(|(i, j)| {
                        Move::new(
                            *self.current_turn.borrow(),
                            &self.points[i],
                            &self.points[j],
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        // let captures = re.captures(&input).ok_or("Input is not parsable")?;

        // let indices = (1..captures.len())
        //     .into_iter()
        //     .filter_map(|i| captures.get(i))
        //     .map(|m| m.as_str().parse::<usize>().unwrap())
        //     .filter(|&i| i < self.points.len())
        //     .collect::<Vec<_>>();

        // // println!("{:?}", indices);

        // let references = indices
        //     .into_iter()
        //     .map(|i| &self.points[i])
        //     .collect::<Vec<_>>();

        // let moves = vec![
        //     Move::new(*self.current_turn.borrow(), references[0], references[1]),
        //     Move::new(*self.current_turn.borrow(), references[1], references[2]),
        // ];

        let turn = Turn::new(moves);

        // println!("{}/{} {}/{}", a, b, c, d);

        // let board = &mut self.clone();

        // if !self.is_move_valid(&move1) {
        //     return Err("Move is not valid.");
        // }

        // self.make_valid_move(&mut move1);

        // // println!("{:?}", board);

        // if !self.is_move_valid(&move2) {
        //     let mut move1_rev = Move::new(move1.color, move1.to, move1.from);
        //     self.make_valid_move(&mut move1_rev);
        //     return Err("Move is not valid.");
        // }

        println!("test");

        Ok(turn)
    }

    fn make_valid_move(&self, r#move: &mut Move) {
        if !self.is_move_valid(&r#move) {
            panic!("Move is invalid");
        }

        r#move.from.borrow_mut().count -= 1;
        if r#move.from.borrow().count == 0 {
            r#move.from.borrow_mut().color = Color::None;
        }

        if Color::are_opposites(r#move.to.borrow().color, r#move.color)
            && r#move.to.borrow().count == 1
        {
            if r#move.color == Color::Black {
                self.points[0].borrow_mut().count += 1;
            } else if r#move.color == Color::White {
                self.points[25].borrow_mut().count += 1;
            }
        }

        r#move.to.borrow_mut().color = r#move.color;
        r#move.to.borrow_mut().count += 1;
    }

    fn is_move_valid(&self, r#move: &Move) -> bool {
        println!("1");
        if *self.current_turn.borrow() != r#move.color {
            return false;
        }

        println!("2");
        if r#move.from.borrow().count <= 0 {
            return false;
        }

        println!("3");
        if r#move.from.borrow().color != r#move.color {
            return false;
        }

        println!("4");
        if Color::are_opposites(r#move.to.borrow().color, r#move.color)
            && r#move.to.borrow().count > 1
        {
            return false;
        }

        println!("5");
        if r#move.to.borrow().color != r#move.color && r#move.to.borrow().color != Color::None {
            return false;
        }

        println!("6");
        return true;
    }
}

impl std::fmt::Debug for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fmt_point = |f: &mut std::fmt::Formatter<'_>, point: &RefCell<Point>| {
            let str = format!("{:#02}", point.borrow().count);
            let str = match point.borrow().color {
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

struct Turn<'a> {
    moves: Vec<Move<'a>>,
}

impl<'a> Turn<'a> {
    fn new(moves: Vec<Move<'a>>) -> Self {
        Self { moves }
    }
}

struct Move<'a> {
    color: Color,
    from: &'a RefCell<Point>,
    to: &'a RefCell<Point>,
}

impl<'a> Move<'a> {
    fn new(color: Color, from: &'a RefCell<Point>, to: &'a RefCell<Point>) -> Self {
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

#[derive(Clone, Copy, Debug)]
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
    let board = Board::new();
    board.start();
}
