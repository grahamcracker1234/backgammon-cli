use super::{board::Point, Player};
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
    pub from: (usize, &'a RefCell<Point>),
    pub to: (usize, &'a RefCell<Point>),
}

impl<'a> Move<'a> {
    pub fn new(
        player: Player,
        from: (usize, &'a RefCell<Point>),
        to: (usize, &'a RefCell<Point>),
    ) -> Self {
        Self { player, from, to }
    }
}
