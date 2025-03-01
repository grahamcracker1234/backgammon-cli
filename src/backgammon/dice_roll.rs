use itertools::Itertools;
use rand::Rng;
use std::fmt;

use crate::backgammon::Error;

/// Number of dice used in the game
pub const COUNT: usize = 2;
/// Number of sides on each die
pub const SIDES: u8 = 6;

/// Represents the dice roll in a backgammon game
///
/// Tracks the actual values of the dice and the available moves that can be
/// made from them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiceRoll {
    /// The actual values of the dice
    dice: [u8; COUNT],
    /// Available moves that can be made from the dice, including duplicates for
    /// multiple uses, sorted in ascending order.
    available: Vec<u8>,
}

// Constructor functions

impl Default for DiceRoll {
    fn default() -> Self {
        let mut rng = rand::rng();
        let dice = [0; COUNT].map(|_| rng.random_range(1..=SIDES));
        let available = Self::calculate_available(dice);
        Self { dice, available }
    }
}

impl DiceRoll {
    /// Creates a new Dice instance with random values
    pub fn new() -> Self {
        Self::default()
    }

    /// Generates a roll where all dice show different values
    ///
    /// Used for the opening roll of the game where doubles are not allowed
    pub fn opening() -> Self {
        loop {
            let roll = Self::new();
            if roll.dice.iter().all_unique() {
                return roll;
            }
        }
    }

    /// Creates a new Dice instance with specific values
    pub fn from(values: [u8; COUNT]) -> Self {
        let available = Self::calculate_available(values);
        Self {
            dice: values,
            available,
        }
    }
}

impl DiceRoll {
    /// Calculates the available moves from dice values
    ///
    /// If there are multiple instances of the same die value,
    /// each die can be used 2^count times.
    fn calculate_available(values: [u8; COUNT]) -> Vec<u8> {
        values
            .iter()
            .counts()
            .into_iter()
            .flat_map(|(&die_value, count)| {
                let multiplier: usize = if count > 1 { 1 << count } else { 1 };
                vec![die_value; multiplier]
            })
            .sorted()
            .collect()
    }

    /// Checks if any die values are still available to be used
    pub fn any_available(&self) -> bool {
        !self.available.is_empty()
    }

    /// Checks if a specific die value is still available to be used
    pub fn contains(&self, value: u8) -> bool {
        self.available.contains(&value)
    }

    /// Consumes one usage of a specific die value
    pub fn consume(&mut self, value: u8) -> Result<(), Error> {
        if let Some(index) = self.available.iter().position(|&d| d == value) {
            self.available.remove(index);
            Ok(())
        } else {
            Err(Error::InvalidPlayLength(value))
        }
    }

    /// Returns the highest available die value
    pub fn max(&self) -> u8 {
        self.available.iter().max().copied().unwrap_or(0)
    }
}

// Iterating: `iter` and `into_iter`

impl DiceRoll {
    pub fn iter(&self) -> std::slice::Iter<u8> {
        self.available.iter()
    }
}

impl IntoIterator for DiceRoll {
    type Item = u8;
    type IntoIter = std::vec::IntoIter<u8>;

    fn into_iter(self) -> Self::IntoIter {
        self.available.into_iter()
    }
}

impl<'a> IntoIterator for &'a DiceRoll {
    type Item = &'a u8;
    type IntoIter = std::slice::Iter<'a, u8>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Implements string formatting for Dice
///
/// # Format options
/// - Default format (`{}`) shows numeric values: "4-6"
/// - Alternate format (`{:#}`) shows Unicode dice: "⚃-⚅"
impl fmt::Display for DiceRoll {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Define dice faces as a const array for better performance
        const DICE_FACES: [&str; SIDES as usize + 1] = ["\0", "⚀", "⚁", "⚂", "⚃", "⚄", "⚅"];

        let formatted: String = self
            .dice
            .iter()
            .map(|&die| {
                if f.alternate() {
                    DICE_FACES[die as usize].to_string()
                } else {
                    die.to_string()
                }
            })
            .join("-");

        f.write_str(&formatted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dice_from() {
        let dice = DiceRoll::from([3, 5]);
        assert_eq!(dice.dice, [3, 5]);
        assert_eq!(dice.available, vec![3, 5]);
    }

    #[test]
    fn test_dice_doubles() {
        let dice = DiceRoll::from([4, 4]);
        assert_eq!(dice.dice, [4, 4]);
        // For doubles, we get 4 available uses of the die value
        assert_eq!(dice.available, vec![4, 4, 4, 4]);
    }

    #[test]
    fn test_remove_die() {
        let mut dice = DiceRoll::from([2, 5]);
        assert!(dice.contains(2));
        assert!(dice.consume(2).is_ok());
        assert!(!dice.contains(2));
        assert!(dice.contains(5));
    }

    #[test]
    fn test_remove_unavailable_die() {
        let mut dice = DiceRoll::from([2, 5]);
        assert!(dice.consume(3).is_err());
    }

    #[test]
    fn test_max() {
        let dice = DiceRoll::from([2, 5]);
        assert_eq!(dice.max(), 5);

        let mut dice = DiceRoll::from([2, 5]);
        assert!(dice.consume(5).is_ok());
        assert_eq!(dice.max(), 2);

        let mut empty_dice = DiceRoll::from([2, 5]);
        assert!(empty_dice.consume(2).is_ok());
        assert!(empty_dice.consume(5).is_ok());
        assert_eq!(empty_dice.max(), 0);
    }

    #[test]
    fn test_any_available() {
        let dice = DiceRoll::from([2, 5]);
        assert!(dice.any_available());

        let mut empty_dice = DiceRoll::from([2, 5]);
        assert!(empty_dice.consume(2).is_ok());
        assert!(empty_dice.consume(5).is_ok());
        assert!(!empty_dice.any_available());
    }

    #[test]
    fn test_available_rolls() {
        let dice = DiceRoll::from([2, 5]);
        assert_eq!(dice.available, vec![2, 5]);

        let dice_doubles = DiceRoll::from([3, 3]);
        assert_eq!(dice_doubles.available, vec![3, 3, 3, 3]);
    }

    #[test]
    fn test_display() {
        let dice = DiceRoll::from([2, 5]);
        assert_eq!(format!("{}", dice), "2-5");
        assert_eq!(format!("{:#}", dice), "⚁-⚄");
    }
}
