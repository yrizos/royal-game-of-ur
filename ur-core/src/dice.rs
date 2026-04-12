use rand::Rng;

/// The result of rolling four binary tetrahedral dice, producing a value 0–4.
///
/// Each die contributes 1 if it lands marked-side up. The total is the sum.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Dice(pub u8);

impl Dice {
    /// Generates a random dice roll using the caller-provided RNG.
    ///
    /// Each of the four dice is independently 50/50, so the result follows
    /// a binomial distribution B(4, 0.5).
    pub fn roll(rng: &mut impl Rng) -> Self {
        todo!()
    }

    /// Returns the numeric value of this roll (0–4).
    pub fn value(self) -> u8 {
        self.0
    }
}

/// Probability of each dice outcome. Index is the roll value (0–4).
///
/// | Roll | Ways | Probability |
/// |------|------|-------------|
/// | 0    | 1    | 1/16        |
/// | 1    | 4    | 4/16        |
/// | 2    | 6    | 6/16        |
/// | 3    | 4    | 4/16        |
/// | 4    | 1    | 1/16        |
pub const DICE_PROBABILITIES: [f64; 5] = [
    1.0 / 16.0,
    4.0 / 16.0,
    6.0 / 16.0,
    4.0 / 16.0,
    1.0 / 16.0,
];

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{rngs::StdRng, SeedableRng};

    #[test]
    fn test_dice_value_always_0_to_4() {
        // Roll 10_000 times with a seeded RNG; every result must be in 0..=4
        todo!()
    }

    #[test]
    fn test_dice_probabilities_sum_to_1() {
        // DICE_PROBABILITIES must sum to exactly 1.0 (within floating-point tolerance)
        todo!()
    }

    #[test]
    fn test_dice_probability_distribution_matches_binomial() {
        // Roll 100_000 times; observed frequencies must be within 2% of expected.
        // Use seed 42 for reproducibility.
        todo!()
    }

    #[test]
    fn test_dice_roll_is_deterministic_given_seed() {
        // Two RNGs with the same seed must produce the same sequence of rolls
        todo!()
    }
}
