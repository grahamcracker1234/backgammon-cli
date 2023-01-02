use itertools::Itertools;

use crate::backgammon::{
    board::{Position, BOARD_SIZE},
    player::Player,
    Error,
};

pub(super) struct Notation(pub String);

impl Notation {
    pub fn position_to_index(
        notation_position: usize,
        perspective: Player,
    ) -> Result<usize, Error> {
        let index = match perspective {
            Player::White => (BOARD_SIZE).checked_sub(notation_position),
            _ => notation_position.checked_sub(1),
        };

        match index {
            Some(index) if index < BOARD_SIZE => Ok(index),
            _ => Err(Error::InvalidNotationPosition(notation_position)),
        }
    }

    pub fn position_from_index(index: usize, perspective: Player) -> Result<usize, Error> {
        let notation_position = match perspective {
            Player::White => (BOARD_SIZE).checked_sub(index),
            _ => index.checked_add(1),
        };

        match notation_position {
            Some(notation_position)
                if notation_position <= BOARD_SIZE && notation_position >= 1 =>
            {
                Ok(notation_position)
            }
            _ => Err(Error::InvalidIndexPosition(index)),
        }
    }

    pub fn turn(&self, player: Player) -> Result<Turn, Error> {
        let re = regex::Regex::new(r"^(?:(?:\d+)|(?:bar))(?:/\d+)*(?:/(?:(?:\d+)|(?:off)))$")
            .expect("Regex is invalid");

        // Get all play groups
        let Notation(notation) = self;
        let play_groups = notation
            .split_whitespace()
            .map(|group| match re.find(group) {
                Some(m) => Ok(Self::get_play_group(m.as_str(), player)?),
                None => Err(Error::InvalidNotation(group.to_owned())),
            })
            .flatten_ok()
            .collect::<Result<Vec<_>, _>>();

        match play_groups {
            Err(error) => return Err(error),
            Ok(play_groups) => Ok(Turn(play_groups)),
        }
    }

    fn get_play_group(notation: &str, player: Player) -> Result<Vec<Play>, Error> {
        let positions = Self::get_board_positions(notation, player);

        match positions {
            Err(error) => return Err(error),
            Ok(positions) => Ok(positions
                .into_iter()
                .tuple_windows()
                .map(|(from, to)| Play::new(player, from, to))
                .collect()),
        }
    }

    fn get_board_positions(notation: &str, player: Player) -> Result<Vec<Position>, Error> {
        notation
            .split('/')
            .map(|m| {
                Ok(match m {
                    "bar" => Position::Bar(player),
                    "off" => Position::Rail(player),
                    pos => {
                        let index = pos.parse::<usize>().unwrap();
                        Position::Point(Notation::position_to_index(index, player)?)
                    }
                })
            })
            .collect()
    }
}

#[derive(Debug)]
pub(crate) struct Turn(pub Vec<Play>);

#[derive(Debug, Clone)]
pub(crate) struct Play {
    pub player: Player,
    pub from: Position,
    pub to: Position,
}

impl Play {
    pub fn new(player: Player, from: Position, to: Position) -> Self {
        Self { player, from, to }
    }
}

impl std::fmt::Display for Play {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}/{}",
            match self.from {
                Position::Bar(_) => "bar".to_string(),
                Position::Rail(_) => panic!("Cannot play a piece after bearing it off."),
                Position::Point(index) => format!(
                    "{:?}",
                    Notation::position_from_index(index, self.player).unwrap()
                ),
            },
            match self.to {
                Position::Bar(_) => panic!("Cannot play onto the bar."),
                Position::Rail(_) => "off".to_string(),
                Position::Point(index) => format!(
                    "{:?}",
                    Notation::position_from_index(index, self.player).unwrap()
                ),
            }
        )
    }
}
