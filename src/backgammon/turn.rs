use std::fmt::Display;

use super::{board::BoardPosition, player::Player, Game};
use itertools::Itertools;

#[derive(Debug)]
pub(super) struct Turn<'a> {
    pub moves: Vec<Move<'a>>,
}

impl<'a> Turn<'a> {
    pub fn new(moves: Vec<Move<'a>>) -> Self {
        Self { moves }
    }

    pub fn from(notation: String, game: &'a Game) -> Result<Self, &'static str> {
        let re = regex::Regex::new(r"^(?:(?:\d+)|(?:bar))(?:/\d+)*(?:/(?:(?:\d+)|(?:off)))$")
            .expect("Regex is invalid");

        // Get all move groups
        let move_groups = notation
            .split_whitespace()
            .map(|group| match re.find(group) {
                Some(m) => Ok(Self::get_move_group(m.as_str(), game)?),
                None => Err("Invalid input."),
            })
            .flatten_ok()
            .collect::<Result<Vec<_>, _>>();

        match move_groups {
            Err(error) => return Err(error),
            Ok(move_groups) => Ok(Turn::new(move_groups)),
        }
    }

    fn get_move_group(notation: &str, game: &'a Game) -> Result<Vec<Move<'a>>, &'static str> {
        let positions = notation
            .split('/')
            .map(|m| {
                Ok(match m {
                    "bar" => BoardPosition::Bar(&game.board.bar[&game.current_player]),
                    "off" => BoardPosition::Off(&game.board.off[&game.current_player]),
                    pos => {
                        let index = pos.parse::<usize>().unwrap() - 1;
                        BoardPosition::Point(game.board.get_point(index, game.current_player)?)
                    }
                })
            })
            .collect::<Result<Vec<_>, _>>();

        match positions {
            Err(error) => return Err(error),
            Ok(positions) => Ok(positions
                .into_iter()
                .tuple_windows()
                .map(|(from, to)| Move::new(game.current_player, from, to))
                .collect()),
        }
    }

    pub fn get_available_moves(game: &'a mut Game) -> impl Iterator<Item = Move<'a>> {
        // let game = game.clone();
        let saved_board = game.board.clone();

        game.board.iter().flat_map(|board_position| {
            // game.board = saved_board.clone();
            game.current_roll
                .borrow()
                .available_rolls()
                .map(|roll| {
                    let from_pos = board_position.effective_pos();

                    let to_pos = from_pos + roll as usize;
                    let notation = format!(
                        "{}/{}",
                        match board_position {
                            BoardPosition::Bar(_) => "bar".to_string(),
                            BoardPosition::Off(_) =>
                                Err("Cannot move a piece after bearing it off.")?,
                            BoardPosition::Point(_) => from_pos.to_string(),
                        },
                        match game.board.get_point(to_pos, game.current_player) {
                            Ok(_) => to_pos,
                            Err(error) => Err(error)?,
                        }
                    );

                    let turn = Turn::from(notation, game)?;
                    let moves = turn.moves.clone();

                    game.take_turn(turn)?;
                    Result::<_, &'static str>::Ok(moves)
                })
                .filter_map(Result::ok)
                .flatten()
                .collect::<Vec<_>>()
        })
    }
}

#[derive(Debug, Clone)]
pub(super) struct Move<'a> {
    pub player: Player,
    pub from: BoardPosition<'a>,
    pub to: BoardPosition<'a>,
}

impl<'a> Move<'a> {
    pub fn new(player: Player, from: BoardPosition<'a>, to: BoardPosition<'a>) -> Self {
        if let BoardPosition::Bar(_) = to {
            panic!("Cannot move onto the bar.")
        }

        if let BoardPosition::Off(_) = from {
            panic!("Cannot move a piece after bearing it off.")
        }

        Self { player, from, to }
    }

    pub fn valid_direction(&self) -> bool {
        if self.player == Player::None {
            panic!("There is no move direction for `Player::None`.");
        }

        let from_pos = self.from.effective_pos();
        let to_pos = self.to.effective_pos();

        // println!("{}: {} -> {}", self.player, from_pos, to_pos);
        match self.player {
            Player::White => to_pos > from_pos,
            Player::Black => to_pos < from_pos,
            _ => unreachable!(),
        }
    }

    pub fn distance(&self) -> usize {
        self.from.effective_pos().abs_diff(self.to.effective_pos())
    }
}

impl Display for Move<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}/{}",
            match self.from {
                BoardPosition::Bar(_) => "bar".to_string(),
                BoardPosition::Off(_) => panic!("Cannot move a piece after bearing it off."),
                BoardPosition::Point(point) => (*point.borrow().pos() + 1).to_string(),
            },
            match self.to {
                BoardPosition::Bar(_) => panic!("Cannot move onto the bar."),
                BoardPosition::Off(_) => "off".to_string(),
                BoardPosition::Point(point) => (*point.borrow().pos() + 1).to_string(),
            }
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn notation() {
        let notation = "1/2/3 10/3 4/1".to_string();
        let _ = Turn::from(notation, &Game::new());

        let notation = "bar/3/4 bar/1".to_string();
        let _ = Turn::from(notation, &Game::new());

        let notation = "3/4/off 1/off".to_string();
        let _ = Turn::from(notation, &Game::new());

        let notation = "bar/off bar/off".to_string();
        let _ = Turn::from(notation, &Game::new());
    }

    #[test]
    #[should_panic]
    fn panic0() {
        let notation = "120/12".to_string();
        assert!(matches!(Turn::from(notation, &Game::new()), Ok(_)));
    }

    #[test]
    #[should_panic]
    fn panic1() {
        let notation = "3/bar".to_string();
        let _ = Turn::from(notation, &Game::new()).unwrap();
    }

    #[test]
    #[should_panic]
    fn panic2() {
        let notation = "3/bar/5".to_string();
        let _ = Turn::from(notation, &Game::new()).unwrap();
    }

    #[test]
    #[should_panic]
    fn panic3() {
        let notation = "off/4".to_string();
        let _ = Turn::from(notation, &Game::new()).unwrap();
    }

    #[test]
    #[should_panic]
    fn panic4() {
        let notation = "off/3/5".to_string();
        let _ = Turn::from(notation, &Game::new()).unwrap();
    }
}
