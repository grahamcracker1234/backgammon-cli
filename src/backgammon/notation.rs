use std::fmt::Display;
use std::string::ToString;

use itertools::Itertools;
use regex::Regex;

use crate::backgammon::{
    board::Board,
    location::{Index, Normalized},
    player::Player,
    Error,
};

/// Represents [backgammon notation](https://en.wikipedia.org/wiki/Backgammon_notation)
/// for the given input and player.
pub struct Notation {
    /// The notation
    input: String,
    /// The player
    player: Player,
}

impl Notation {
    /// Create a `Notation`.
    pub const fn new(input: String, player: Player) -> Self {
        Self { input, player }
    }

    /// Tries to generate a `Turn` from itself.
    pub fn turn(&self) -> Result<Turn, Error> {
        let re = Regex::new(r"^((\d+)|(bar))(/\d+)*(/((\d+)|(off)))$")
            .expect("regex should always be valid");

        // Get all play groups, the plays of each whitespace-seperated group of
        // invididual simple notations.
        let play_groups: Result<Vec<_>, _> = self
            .input
            .split_whitespace()
            .map(|group| {
                re.find(group).map_or_else(
                    || Err(Error::InvalidNotation(group.to_owned())),
                    |m| {
                        let input = m.as_str().to_owned();
                        let notation = Self::new(input, self.player);
                        notation.get_play_group()
                    },
                )
            })
            .flatten_ok()
            .collect();

        play_groups.map(Turn)
    }

    /// Gets the group of plays of a given simple notation (notation without whitespace).
    fn get_play_group(&self) -> Result<Vec<Play>, Error> {
        let spaces = self.get_board_spaces()?;
        let plays = spaces
            .into_iter()
            .tuple_windows()
            .map(|(from, to)| Play::new(self.player, from, to))
            .collect();
        Ok(plays)
    }

    /// Converts the simple notation's (notation without whitespace) input into a list of spaces.
    fn get_board_spaces(&self) -> Result<Vec<PositionRef>, Error> {
        let spaces = self.input.split('/');
        spaces
            .map(|m| {
                Ok(match m {
                    "bar" => PositionRef::Bar(self.player),
                    "off" => PositionRef::Rail(self.player),
                    pos => {
                        let pos = pos.parse::<usize>().expect("pos should be an integer");
                        let norm = Normalized::new(pos, self.player)?;
                        let index = norm.to_index()?;
                        PositionRef::Point(index)
                    }
                })
            })
            .collect()
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Turn(pub Vec<Play>);

impl Turn {
    pub fn distance(&self, board: &Board) -> usize {
        let Self(plays) = self;
        plays
            .iter()
            .map(|play| {
                let from = board.get(&play.from);
                let to = board.get(&play.to);
                from.distance(to)
            })
            .sum()
    }
}

#[allow(unstable_name_collisions)]
impl Display for Turn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(plays) = self;
        write!(
            f,
            "Turn({})",
            plays
                .iter()
                .map(ToString::to_string)
                .intersperse(", ".to_string())
                .collect::<String>()
        )
    }
}

#[allow(unused)]
macro_rules! turn {
    ($player:expr $(, ($from:tt, $to:tt))*) => {
        Turn(vec![
            $(Play::new(
                $player,
                turn!($player, $from),
                turn!($player, $to),
            )),*
        ])
    };
    ($player:expr, bar) => { PositionRef::Bar($player) };
    ($player:expr, off) => { PositionRef::Rail($player) };
    ($player:expr, $index:literal) => { PositionRef::Point($index.try_into().unwrap()) };
}

#[allow(unused)]
pub(crate) use turn;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PositionRef {
    Bar(Player),
    Rail(Player),
    Point(Index),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Play {
    pub player: Player,
    pub from: PositionRef,
    pub to: PositionRef,
}

impl Play {
    pub const fn new(player: Player, from: PositionRef, to: PositionRef) -> Self {
        Self { player, from, to }
    }

    pub fn is_valid_direction(&self, board: &Board) -> bool {
        let from = board.get(&self.from);
        let to = board.get(&self.to);

        match self.player {
            Player::White => to.location > from.location,
            Player::Black => to.location < from.location,
            Player::None => panic!("There is no move direction for `Player::None`."),
        }
    }
}

impl std::fmt::Display for Play {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}/{}",
            match self.from {
                PositionRef::Bar(_) => "bar".to_string(),
                PositionRef::Rail(_) => panic!("Cannot play a piece after bearing it off."),
                PositionRef::Point(index) => index.normalize(self.player).to_string(),
            },
            match self.to {
                PositionRef::Bar(_) => panic!("Cannot play onto the bar."),
                PositionRef::Rail(_) => "off".to_string(),
                PositionRef::Point(index) => index.normalize(self.player).to_string(),
            }
        )
    }
}

#[allow(unused)]
macro_rules! plays {
    ($player:expr $(, ($from:tt, $to:tt))*) => {
        HashSet::from([
            $(Play::new(
                $player,
                turn!($player, $from),
                turn!($player, $to),
            )),*
        ])
    };
    ($player:expr, bar) => { PositionRef::Bar($player) };
    ($player:expr, off) => { PositionRef::Rail($player) };
    ($player:expr, $index:literal) => { PositionRef::Point($index.try_into().unwrap()) };
}

#[allow(unused)]
pub(crate) use plays;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn black_notation_1() {
        let notation = Notation::new("1/2".to_string(), Player::Black);
        let turn = turn!(Player::Black, (0, 1));
        assert_eq!(notation.turn(), Ok(turn));
    }

    #[test]
    fn black_notation_2() {
        let notation = Notation::new("3/4 13/14".to_string(), Player::Black);
        let turn = turn!(Player::Black, (2, 3), (12, 13));
        assert_eq!(notation.turn(), Ok(turn));
    }

    #[test]
    fn black_notation_3() {
        let notation = Notation::new("10/5/19".to_string(), Player::Black);
        let turn = turn!(Player::Black, (9, 4), (4, 18));
        assert_eq!(notation.turn(), Ok(turn));
    }

    #[test]
    fn black_notation_4() {
        let notation = Notation::new("8/3/14/22 7/19".to_string(), Player::Black);
        let turn = turn!(Player::Black, (7, 2), (2, 13), (13, 21), (6, 18));
        assert_eq!(notation.turn(), Ok(turn));
    }

    #[test]
    fn black_notation_5() {
        let notation = Notation::new("bar/1/20 7/19/off".to_string(), Player::Black);
        let turn = turn!(Player::Black, (bar, 0), (0, 19), (6, 18), (18, off));
        assert_eq!(notation.turn(), Ok(turn));
    }

    #[test]
    fn black_notation_6() {
        let notation = Notation::new("bar/off bar/off".to_string(), Player::Black);
        let turn = turn!(Player::Black, (bar, off), (bar, off));
        assert_eq!(notation.turn(), Ok(turn));
    }

    #[test]
    fn white_notation_1() {
        let notation = Notation::new("1/2".to_string(), Player::White);
        let turn = turn!(Player::White, (23, 22));
        assert_eq!(notation.turn(), Ok(turn));
    }

    #[test]
    fn white_notation_2() {
        let notation = Notation::new("3/4 13/14".to_string(), Player::White);
        let turn = turn!(Player::White, (21, 20), (11, 10));
        assert_eq!(notation.turn(), Ok(turn));
    }

    #[test]
    fn white_notation_3() {
        let notation = Notation::new("10/5/19".to_string(), Player::White);
        let turn = turn!(Player::White, (14, 19), (19, 5));
        assert_eq!(notation.turn(), Ok(turn));
    }

    #[test]
    fn white_notation_4() {
        let notation = Notation::new("8/3/14/22 7/19".to_string(), Player::White);
        let turn = turn!(Player::White, (16, 21), (21, 10), (10, 2), (17, 5));
        assert_eq!(notation.turn(), Ok(turn));
    }

    #[test]
    fn white_notation_5() {
        let notation = Notation::new("bar/1/20 7/19/off".to_string(), Player::White);
        let turn = turn!(Player::White, (bar, 23), (23, 4), (17, 5), (5, off));
        assert_eq!(notation.turn(), Ok(turn));
    }

    #[test]
    fn white_notation_6() {
        let notation = Notation::new("bar/off bar/off".to_string(), Player::White);
        let turn = turn!(Player::White, (bar, off), (bar, off));
        assert_eq!(notation.turn(), Ok(turn));
    }

    #[test]
    fn empty() {
        let notation = Notation::new("".to_string(), Player::White);
        let turn = turn!(Player::White);
        assert_eq!(notation.turn(), Ok(turn));
    }

    #[test]
    fn bad_notation_1() {
        let input = "test123.4abc-30".to_string();
        let notation = Notation::new(input.clone(), Player::Black);
        assert_eq!(notation.turn(), Err(Error::InvalidNotation(input)));
    }

    #[test]
    fn bad_notation_2() {
        let input = "test123.4abc-30".to_string();
        let notation = Notation::new(input.clone(), Player::White);
        assert_eq!(notation.turn(), Err(Error::InvalidNotation(input)));
    }

    #[test]
    fn bad_notation_3() {
        let input = "bar/bar".to_string();
        let notation = Notation::new(input.clone(), Player::Black);
        assert_eq!(notation.turn(), Err(Error::InvalidNotation(input)));
    }

    #[test]
    fn bad_notation_4() {
        let input = "bar/bar".to_string();
        let notation = Notation::new(input.clone(), Player::White);
        assert_eq!(notation.turn(), Err(Error::InvalidNotation(input)));
    }

    #[test]
    fn bad_notation_5() {
        let input = "1/bar/10".to_string();
        let notation = Notation::new(input.clone(), Player::Black);
        assert_eq!(notation.turn(), Err(Error::InvalidNotation(input)));
    }

    #[test]
    fn bad_notation_6() {
        let input = "1/bar/10".to_string();
        let notation = Notation::new(input.clone(), Player::White);
        assert_eq!(notation.turn(), Err(Error::InvalidNotation(input)));
    }

    #[test]
    fn bad_notation_7() {
        let input = "off/10/3".to_string();
        let notation = Notation::new(input.clone(), Player::Black);
        assert_eq!(notation.turn(), Err(Error::InvalidNotation(input)));
    }

    #[test]
    fn bad_notation_8() {
        let input = "off/10/3".to_string();
        let notation = Notation::new(input.clone(), Player::White);
        assert_eq!(notation.turn(), Err(Error::InvalidNotation(input)));
    }

    #[test]
    fn bad_notation_9() {
        let input = "19/off/21".to_string();
        let notation = Notation::new(format!("bar/10/off {input}"), Player::Black);
        assert_eq!(notation.turn(), Err(Error::InvalidNotation(input)));
    }

    #[test]
    fn bad_notation_10() {
        let input = "19/off/21".to_string();
        let notation = Notation::new(format!("bar/10/off {input}"), Player::White);
        assert_eq!(notation.turn(), Err(Error::InvalidNotation(input)));
    }
}
