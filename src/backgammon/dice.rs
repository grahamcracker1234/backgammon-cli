use itertools::Itertools;
use rand::Rng;
use std::collections::HashMap;

use crate::backgammon::Error;

const COUNT: usize = 2;
const SIDES: u8 = 6;

#[derive(Clone)]
pub(crate) struct Dice {
    dice: [u8; COUNT],
    cast_freq: HashMap<u8, u8>,
}

impl Dice {
    #[allow(dead_code)]
    pub fn from(dice: [u8; COUNT]) -> Self {
        let cast_freq = Self::cast_freq(dice);

        Self { dice, cast_freq }
    }

    fn cast_freq(dice: [u8; COUNT]) -> HashMap<u8, u8> {
        let mut cast_freq = HashMap::new();

        for die in dice {
            *cast_freq.entry(die).or_insert(0) += 1;
        }

        for die in cast_freq.values_mut() {
            if *die > 1 {
                *die = 1 << *die;
            }
        }

        cast_freq
    }

    pub fn roll() -> Self {
        let mut rng = rand::thread_rng();

        let dice = (0..COUNT)
            .map(|_| rng.gen_range(1..=SIDES))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        let cast_freq = Self::cast_freq(dice);

        Self { dice, cast_freq }
    }

    pub fn check(&self, cast: u8) -> bool {
        matches!(self.cast_freq.get(&cast), Some(&count) if count > 0)
    }

    pub fn remove(&mut self, cast: u8) {
        match self.cast_freq.get_mut(&cast) {
            Some(count) if *count > 0 => *count -= 1,
            _ => panic!("{}", Error::InvalidPlayLength(cast)),
        }
    }

    pub fn available_rolls(&self) -> impl Iterator<Item = u8> + '_ {
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

    pub fn max(&self) -> u8 {
        self.available_rolls().max().unwrap()
    }
}

#[allow(unstable_name_collisions)]
impl std::fmt::Display for Dice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            f.write_str(
                &self
                    .dice
                    .into_iter()
                    .map(|die| {
                        match die {
                            1 => "\u{2680}",
                            2 => "\u{2681}",
                            3 => "\u{2682}",
                            4 => "\u{2683}",
                            5 => "\u{2684}",
                            6 => "\u{2685}",
                            _ => "\u{1F3B2}",
                        }
                        .to_string()
                    })
                    .intersperse("-".to_string())
                    .collect::<String>(),
            )
        } else {
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
}
