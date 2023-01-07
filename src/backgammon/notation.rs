use itertools::Itertools;
use regex::Regex;

use crate::backgammon::{
    board::Board,
    location::{IndexLocation, NormalizedLocation},
    player::Player,
    Error,
};

/// Represents [backgammon notation](https://en.wikipedia.org/wiki/Backgammon_notation)
/// for the given input and player.
pub(crate) struct Notation {
    /// The notation
    input: String,
    /// The player
    player: Player,
}

impl Notation {
    /// Create a `Notation`.
    pub fn new(input: String, player: Player) -> Self {
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
            .map(|group| match re.find(group) {
                Some(m) => {
                    let input = m.as_str().to_owned();
                    let notation = Notation::new(input, self.player);
                    notation.get_play_group()
                }
                None => Err(Error::InvalidNotation(group.to_owned())),
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
                        let norm = NormalizedLocation::new(pos, self.player)?;
                        let index = norm.to_index()?;
                        PositionRef::Point(index)
                    }
                })
            })
            .collect()
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct Turn(pub Vec<Play>);

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum PositionRef {
    Bar(Player),
    Rail(Player),
    Point(IndexLocation),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Play {
    pub player: Player,
    pub from: PositionRef,
    pub to: PositionRef,
}

impl Play {
    pub fn new(player: Player, from: PositionRef, to: PositionRef) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! turn {
        ($player:expr, $(($from:tt, $to:tt)),*) => {
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

    use Player::{Black, White};

    #[test]
    fn black_notation_1() {
        let notation = Notation::new("1/2".to_string(), Black);
        let turn = turn!(Black, (0, 1));
        assert_eq!(notation.turn(), Ok(turn));
    }

    #[test]
    fn black_notation_2() {
        let notation = Notation::new("3/4 13/14".to_string(), Black);
        let turn = turn!(Black, (2, 3), (12, 13));
        assert_eq!(notation.turn(), Ok(turn));
    }

    #[test]
    fn black_notation_3() {
        let notation = Notation::new("10/5/19".to_string(), Black);
        let turn = turn!(Black, (9, 4), (4, 18));
        assert_eq!(notation.turn(), Ok(turn));
    }

    #[test]
    fn black_notation_4() {
        let notation = Notation::new("8/3/14/22 7/19".to_string(), Black);
        let turn = turn!(Black, (7, 2), (2, 13), (13, 21), (6, 18));
        assert_eq!(notation.turn(), Ok(turn));
    }

    #[test]
    fn black_notation_5() {
        let notation = Notation::new("bar/1/20 7/19/off".to_string(), Black);
        let turn = turn!(Black, (bar, 0), (0, 19), (6, 18), (18, off));
        assert_eq!(notation.turn(), Ok(turn));
    }

    #[test]
    fn black_notation_6() {
        let notation = Notation::new("bar/off bar/off".to_string(), Black);
        let turn = turn!(Black, (bar, off), (bar, off));
        assert_eq!(notation.turn(), Ok(turn));
    }

    #[test]
    fn white_notation_1() {
        let notation = Notation::new("1/2".to_string(), White);
        let turn = turn!(White, (23, 22));
        assert_eq!(notation.turn(), Ok(turn));
    }

    #[test]
    fn white_notation_2() {
        let notation = Notation::new("3/4 13/14".to_string(), White);
        let turn = turn!(White, (21, 20), (11, 10));
        assert_eq!(notation.turn(), Ok(turn));
    }

    #[test]
    fn white_notation_3() {
        let notation = Notation::new("10/5/19".to_string(), White);
        let turn = turn!(White, (14, 19), (19, 5));
        assert_eq!(notation.turn(), Ok(turn));
    }

    #[test]
    fn white_notation_4() {
        let notation = Notation::new("8/3/14/22 7/19".to_string(), White);
        let turn = turn!(White, (16, 21), (21, 10), (10, 2), (17, 5));
        assert_eq!(notation.turn(), Ok(turn));
    }

    #[test]
    fn white_notation_5() {
        let notation = Notation::new("bar/1/20 7/19/off".to_string(), White);
        let turn = turn!(White, (bar, 23), (23, 4), (17, 5), (5, off));
        assert_eq!(notation.turn(), Ok(turn));
    }

    #[test]
    fn white_notation_6() {
        let notation = Notation::new("bar/off bar/off".to_string(), White);
        let turn = turn!(White, (bar, off), (bar, off));
        assert_eq!(notation.turn(), Ok(turn));
    }

    #[test]
    fn bad_notation_1() {
        let input = "test123.4abc-30".to_string();
        let notation = Notation::new(input.clone(), Black);
        assert_eq!(notation.turn(), Err(Error::InvalidNotation(input)));
    }

    #[test]
    fn bad_notation_2() {
        let input = "test123.4abc-30".to_string();
        let notation = Notation::new(input.clone(), White);
        assert_eq!(notation.turn(), Err(Error::InvalidNotation(input)));
    }

    #[test]
    fn bad_notation_3() {
        let input = "bar/bar".to_string();
        let notation = Notation::new(input.clone(), Black);
        assert_eq!(notation.turn(), Err(Error::InvalidNotation(input)));
    }

    #[test]
    fn bad_notation_4() {
        let input = "bar/bar".to_string();
        let notation = Notation::new(input.clone(), White);
        assert_eq!(notation.turn(), Err(Error::InvalidNotation(input)));
    }

    #[test]
    fn bad_notation_5() {
        let input = "1/bar/10".to_string();
        let notation = Notation::new(input.clone(), Black);
        assert_eq!(notation.turn(), Err(Error::InvalidNotation(input)));
    }

    #[test]
    fn bad_notation_6() {
        let input = "1/bar/10".to_string();
        let notation = Notation::new(input.clone(), White);
        assert_eq!(notation.turn(), Err(Error::InvalidNotation(input)));
    }

    #[test]
    fn bad_notation_7() {
        let input = "off/10/3".to_string();
        let notation = Notation::new(input.clone(), Black);
        assert_eq!(notation.turn(), Err(Error::InvalidNotation(input)));
    }

    #[test]
    fn bad_notation_8() {
        let input = "off/10/3".to_string();
        let notation = Notation::new(input.clone(), White);
        assert_eq!(notation.turn(), Err(Error::InvalidNotation(input)));
    }

    #[test]
    fn bad_notation_9() {
        let input = "19/off/21".to_string();
        let notation = Notation::new(format!("bar/10/off {input}"), Black);
        assert_eq!(notation.turn(), Err(Error::InvalidNotation(input)));
    }

    #[test]
    fn bad_notation_10() {
        let input = "19/off/21".to_string();
        let notation = Notation::new(format!("bar/10/off {input}"), White);
        assert_eq!(notation.turn(), Err(Error::InvalidNotation(input)));
    }
}
