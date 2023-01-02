use itertools::Itertools;
use rand::Rng;
use std::collections::HashMap;

use super::Error;

const COUNT: usize = 2;
const SIDES: u8 = 6;

#[derive(Clone)]
pub(super) struct Dice {
    pub dice: [u8; COUNT],
    cast_freq: HashMap<u8, u8>,
}

impl Dice {
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

        let cast_freq = HashMap::new();

        let mut roll = Self { dice, cast_freq };

        for die in roll.dice {
            *roll.cast_freq.entry(die).or_insert(0) += 1;
        }

        for die in roll.cast_freq.values_mut() {
            if *die > 1 {
                *die = 1 << *die;
            }
        }

        roll
    }

    pub fn check(&self, cast: u8) -> bool {
        match self.cast_freq.get(&cast) {
            Some(&count) if count > 0 => true,
            _ => false,
        }
    }

    pub fn remove(&mut self, cast: u8) {
        match self.cast_freq.get_mut(&cast) {
            Some(count) if *count > 0 => *count -= 1,
            _ => panic!("{}", Error::InvalidPlayLength(cast)),
        }
    }

    pub fn available_rolls<'a>(&'a self) -> impl Iterator<Item = u8> + 'a {
        self.cast_freq
            .iter()
            .flat_map(|(&k, &v)| vec![k; v as usize].into_iter())
    }

    pub fn any_available(&self) -> bool {
        self.cast_freq.values().any(|&count| count > 0)
    }

    pub fn reroll(&mut self) {
        *self = Dice::roll();
    }

    pub fn first_roll() -> Self {
        loop {
            let roll = Self::roll();
            if roll.cast_freq.values().all(|&count| count == 1) {
                return roll;
            }
        }
    }
}

#[allow(unstable_name_collisions)]
impl std::fmt::Display for Dice {
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
