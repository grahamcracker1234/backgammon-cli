use itertools::Itertools;
use rand::Rng;

const COUNT: usize = 2;
const SIDES: u8 = 6;

pub(super) struct Roll {
    pub dice: [u8; COUNT],
}

impl Roll {
    pub fn roll() -> Self {
        if SIDES == 0 {
            panic!("Number of sides cannot be zero")
        }

        let mut rng = rand::thread_rng();

        Self {
            dice: (0..COUNT)
                .map(|_| rng.gen_range(1..=SIDES))
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        }
    }

    pub fn reroll(&mut self) {
        *self = Roll::roll();
    }

    pub fn is_double(&self) -> bool {
        self.dice.iter().all_equal()
    }

    pub fn first_roll() -> Self {
        loop {
            let dice = Self::roll();
            if !dice.is_double() {
                return dice;
            }
        }
    }
}
