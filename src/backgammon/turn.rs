use itertools::Itertools;

use super::{
    board::{Board, BoardPosition, BOARD_SIZE},
    game::Game,
    player::Player,
    Error,
};

#[derive(Debug)]
pub(super) struct Turn {
    pub moves: Vec<Move>,
}

impl Turn {
    pub fn new(moves: Vec<Move>) -> Self {
        Self { moves }
    }

    pub fn from(notation: String, game: &Game) -> Result<Self, Error> {
        let re = regex::Regex::new(r"^(?:(?:\d+)|(?:bar))(?:/\d+)*(?:/(?:(?:\d+)|(?:off)))$")
            .expect("Regex is invalid");

        // Get all move groups
        let move_groups = notation
            .split_whitespace()
            .map(|group| match re.find(group) {
                Some(m) => Ok(Self::get_move_group(m.as_str(), game)?),
                None => Err(Error::InvalidNotation(group.to_owned())),
            })
            .flatten_ok()
            .collect::<Result<Vec<_>, _>>();

        match move_groups {
            Err(error) => return Err(error),
            Ok(move_groups) => Ok(Turn::new(move_groups)),
        }
    }

    fn get_move_group(notation: &str, game: &Game) -> Result<Vec<Move>, Error> {
        let positions = Self::get_board_positions(notation, game);

        match positions {
            Err(error) => return Err(error),
            Ok(positions) => Ok(positions
                .into_iter()
                .tuple_windows()
                .map(|(from, to)| Move::new(game.current_player, from, to))
                .collect()),
        }
    }

    fn get_board_positions(notation: &str, game: &Game) -> Result<Vec<BoardPosition>, Error> {
        notation
            .split('/')
            .map(|m| {
                let player = game.current_player;
                Ok(match m {
                    "bar" => BoardPosition::Bar(player),
                    "off" => BoardPosition::Off(player),
                    pos => {
                        let index = pos.parse::<usize>().unwrap();
                        BoardPosition::Point(Board::convert_index(index, player)?)
                    }
                })
            })
            .collect()
    }

    pub fn get_available_moves<'a>(mut game: Game) -> impl Iterator<Item = Move> + 'a {
        // let game = game.clone();
        let saved_game = game.clone();
        let board_iter = (0..BOARD_SIZE)
            .map(|index| BoardPosition::Point(index))
            .chain([
                BoardPosition::Bar(Player::Black),
                BoardPosition::Bar(Player::White),
            ])
            .map(|x| x.clone());

        board_iter.flat_map(move |board_position| {
            let rolls_iter = game
                .current_roll
                .borrow()
                .available_rolls()
                .map(|x| x.clone())
                .collect::<Vec<_>>()
                .into_iter();

            rolls_iter
                .map(|roll| {
                    let from = board_position.clone();

                    let point = board_position.point(&game.board).borrow();
                    if game.current_player != point.player {
                        return Err(Error::MoveMadeOutOfTurn);
                    }

                    let r#move = match point.player {
                        Player::Black => match point.effective_pos.checked_sub(roll as usize + 1) {
                            Some(index) if index < BOARD_SIZE => {
                                let to = BoardPosition::Point(index);
                                Move::new(point.player, from, to)
                            }
                            _ => return Err(Error::InvalidNotationPosition(point.effective_pos)),
                        },
                        Player::White => match point.effective_pos.checked_add(roll as usize - 1) {
                            Some(index) if index < BOARD_SIZE => {
                                let to = BoardPosition::Point(index);
                                Move::new(point.player, from, to)
                            }
                            _ => return Err(Error::InvalidNotationPosition(point.effective_pos)),
                        },
                        _ => return Err(Error::MoveMadeOutOfTurn),
                    };

                    // let Some(index) = point.effective_pos.checked_sub(roll as usize + 1) else {
                    //     return Err(Error::InvalidNotationPosition(point.effective_pos));
                    // };

                    // let to = BoardPosition::Point(index);

                    // let to = match point.player {
                    //     Player::None => return Err(Error::MoveMadeOutOfTurn),
                    //     Player::Black => {
                    //         let Some(index) = point.effective_pos.checked_sub(1) else {
                    //             return Err(Error::InvalidNotationPosition(point.effective_pos));
                    //         };
                    //         let Some(index) = index.checked_sub(roll as usize) else {
                    //             return Err(Error::InvalidNotationPosition(point.effective_pos));
                    //         };
                    //         BoardPosition::Point(index)
                    //     }
                    //     Player::White => {
                    //         let Some(index) = point.effective_pos.checked_sub(1) else {
                    //             return Err(Error::InvalidNotationPosition(point.effective_pos));
                    //         };
                    //         let index = index + roll as usize;
                    //         BoardPosition::Point(index)
                    //     }
                    // };

                    // let r#move = Move::new(point.player, from, to);
                    // let notation = format!(
                    //     "{}/{}",
                    //     match board_position {
                    //         BoardPosition::Bar(_) => "bar".to_string(),
                    //         BoardPosition::Off(_) => Err(Error::MoveMadeFromBearingTable)?,
                    //         BoardPosition::Point(_) => from_pos.to_string(),
                    //     },
                    //     match Board::convert_index(to_pos, game.current_player) {
                    //         Ok(_) => to_pos,
                    //         Err(error) => Err(error)?,
                    //     }
                    // );

                    // let turn = Turn::from(notation, &game)?;
                    // let r#move = turn.moves[0].clone();

                    // game.take_turn(turn)?;
                    drop(point);
                    println!("{}", r#move);
                    if let Err(error) = game.check_move(&r#move) {
                        // println!("{error}");
                        return Err(error);
                    }
                    game = saved_game.clone();

                    Ok::<_, Error>(r#move)
                })
                // .filter_map(Result::ok)
                // .flatten()
                // .collect::<Vec<_>>();
                .flatten()
                .collect::<Vec<_>>()
        })
        // .flatten()
        // .filter_map(Result::ok)
        // .flatten()
        // .collect::<Vec<_>>();
    }
}

#[derive(Debug, Clone)]
pub(super) struct Move {
    pub player: Player,
    pub from: BoardPosition,
    pub to: BoardPosition,
}

impl Move {
    pub fn new(player: Player, from: BoardPosition, to: BoardPosition) -> Self {
        Self { player, from, to }
    }
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}/{}",
            match self.from {
                BoardPosition::Bar(_) => "bar".to_string(),
                BoardPosition::Off(_) => panic!("Cannot move a piece after bearing it off."),
                BoardPosition::Point(index) =>
                    format!("{:?}", Board::convert_notation(index, self.player).unwrap()),
            },
            match self.to {
                BoardPosition::Bar(_) => panic!("Cannot move onto the bar."),
                BoardPosition::Off(_) => "off".to_string(),
                BoardPosition::Point(index) =>
                    format!("{:?}", Board::convert_notation(index, self.player).unwrap()),
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
