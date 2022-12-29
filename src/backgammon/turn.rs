use super::{board::Point, player::Player};
use std::cell::RefCell;

pub(super) struct Turn<'a> {
    pub moves: Vec<Move<'a>>,
}

impl<'a> Turn<'a> {
    pub fn new(moves: Vec<Move<'a>>) -> Self {
        Self { moves }
    }
}

pub(super) struct Move<'a> {
    pub player: Player,
    pub from: &'a RefCell<Point>,
    pub to: &'a RefCell<Point>,
}

impl<'a> Move<'a> {
    pub fn new(player: Player, from: &'a RefCell<Point>, to: &'a RefCell<Point>) -> Self {
        Self { player, from, to }
    }

    pub fn valid_direction(&self) -> bool {
        match self.player {
            Player::White => self.to.borrow().pos < self.from.borrow().pos,
            Player::Black => self.to.borrow().pos > self.from.borrow().pos,
            Player::None => panic!("There is no move direction for `Player::None`."),
        }
    }

    pub fn distance(&self) -> usize {
        self.from.borrow().pos.abs_diff(self.to.borrow().pos)
    }
}
