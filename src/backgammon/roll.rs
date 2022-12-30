use rand::Rng;
use std::collections::HashMap;

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

        // println!("roll: {:?}", roll.dice_freq);

        roll
    }

    pub fn remove(&mut self, die: u8) -> Result<(), &'static str> {
        // println!("remove: {:?} (die: {die})", self.dice_freq);
        match self.dice_freq.get_mut(&die) {
            Some(die) if *die > 0 => *die -= 1,
            _ => return Err("Cannot make move of that length."),
        }
        Ok(())
    }

    pub fn available(&self) -> bool {
        // println!("available: {:?}", self.dice_freq);
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
