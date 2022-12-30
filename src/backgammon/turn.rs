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

        let moves = notation
            .split_whitespace()
            .flat_map(|m| {
                if let Some(r#match) = re.find(m) {
                    r#match
                        .as_str()
                        .split('/')
                        .map(|m| match m {
                            "bar" => BoardPosition::Bar(&game.board.bar[&game.current_player]),
                            "off" => BoardPosition::Off(&game.board.off[&game.current_player]),
                            pos => {
                                let index = pos.parse::<usize>().unwrap() - 1;
                                BoardPosition::Point(
                                    game.board.get_point(index, game.current_player),
                                )
                            }
                        })
                        .tuple_windows()
                        .map(|(from, to)| Some(Move::new(game.current_player, from, to)))
                        .collect::<Vec<_>>()
                } else {
                    vec![None]
                }
            })
            .collect::<Vec<_>>();

        if moves.iter().any(|m| m.is_none()) {
            return Err("Invalid input.");
        }
        Ok(Turn::new(moves.into_iter().map(|m| m.unwrap()).collect()))
        // Turn::new(
        //     regex::Regex::new(r"\d+(?:/\d+)+")
        //         .expect("Regex is invalid")
        //         .find_iter(&notation)
        //         .flat_map(|m| {
        //             m.as_str()
        //                 .split('/')
        //                 .map(|m| m.parse::<usize>().unwrap())
        //                 .tuple_windows()
        //                 .map(|(i, j)| {
        //                     Move::new(
        //                         game.current_player,
        //                         BoardPosition::Point(
        //                             &game.board.get_point(i - 1, game.current_player),
        //                         ),
        //                         &game.board.get_point(j - 1, game.current_player),
        //                     )
        //                 })
        //                 .collect::<Vec<_>>()
        //         })
        //         .collect::<Vec<_>>(),
        // )
    }
}

#[derive(Debug)]
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

        println!("{}: {} -> {}", self.player, from_pos, to_pos);
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
