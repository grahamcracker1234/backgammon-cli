use itertools::Itertools;
use rand::Rng;
use std::collections::HashMap;

use super::Error;

const COUNT: usize = 2;
const SIDES: u8 = 6;

#[derive(Clone)]
pub(super) struct Roll {
    pub dice: [u8; COUNT],
    dice_freq: HashMap<u8, u8>,
}

impl Roll {
    pub fn roll() -> Self {
        if SIDES == 0 {
            panic!("Number of sides cannot be zero")
        }

        let mut rng = rand::thread_rng();

        let dice = (0..COUNT)
            .map(|_| rng.gen_range(1..=SIDES))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        let dice_freq = HashMap::new();

        let mut roll = Self { dice, dice_freq };

        for die in roll.dice {
            *roll.dice_freq.entry(die).or_insert(0) += 1;
        }

        for die in roll.dice_freq.values_mut() {
            if *die > 1 {
                *die = 1 << *die;
            }
        }

        roll
    }

    pub fn check(&self, die: u8) -> bool {
        match self.dice_freq.get(&die) {
            Some(&count) if count > 0 => true,
            _ => false,
        }
    }

    pub fn remove(&mut self, die: u8) {
        match self.dice_freq.get_mut(&die) {
            Some(count) if *count > 0 => *count -= 1,
            _ => panic!("{}", Error::InvalidPlayLength(die)),
        }
    }

    pub fn available_rolls<'a>(&'a self) -> impl Iterator<Item = u8> + 'a {
        self.dice_freq
            .iter()
            .flat_map(|(&k, &v)| vec![k; v as usize].into_iter())
    }

    pub fn any_available(&self) -> bool {
        self.dice_freq.values().any(|&count| count > 0)
    }

    pub fn reroll(&mut self) {
        *self = Roll::roll();
    }

    pub fn first_roll() -> Self {
        loop {
            let dice = Self::roll();
            if dice.dice_freq.values().all(|&count| count == 1) {
                return dice;
            }
        }
    }
}

#[allow(unstable_name_collisions)]
impl std::fmt::Display for Roll {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            &self
                .dice
                .into_iter()
                .map(|die| die.to_string())
                .intersperse("-".to_string())
                .collect::<String>(),
        )
    }
}
