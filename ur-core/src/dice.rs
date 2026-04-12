use rand::Rng;

/// The result of rolling four binary tetrahedral dice, producing a value 0–4.
///
/// Each die contributes 1 if it lands marked-side up. The total is the sum.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Dice(pub u8);

impl Dice {
    /// Generates a random dice roll using the caller-provided RNG.
    ///
    /// Simulates four independent fair binary dice (each 50/50), returning
    /// their sum — a binomial B(4, 0.5) distribution producing values 0–4.
    pub fn roll(rng: &mut impl Rng) -> Self {
        let count: u8 = (0..4).map(|_| rng.gen::<bool>() as u8).sum();
        Dice(count)
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
pub const DICE_PROBABILITIES: [f64; 5] =
    [1.0 / 16.0, 4.0 / 16.0, 6.0 / 16.0, 4.0 / 16.0, 1.0 / 16.0];

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{rngs::StdRng, SeedableRng};

    #[test]
    fn test_dice_value_always_0_to_4() {
        let mut rng = StdRng::seed_from_u64(0);
        for _ in 0..10_000 {
            let roll = Dice::roll(&mut rng);
            assert!(roll.value() <= 4, "roll {} out of range", roll.value());
        }
    }

    #[test]
    fn test_dice_probabilities_sum_to_1() {
        let sum: f64 = DICE_PROBABILITIES.iter().sum();
        assert!((sum - 1.0).abs() < 1e-10, "probabilities sum to {}, not 1.0", sum);
    }

    #[test]
    fn test_dice_probability_distribution_matches_binomial() {
        let mut rng = StdRng::seed_from_u64(42);
        let n = 100_000usize;
        let mut counts = [0usize; 5];
        for _ in 0..n {
            counts[Dice::roll(&mut rng).value() as usize] += 1;
        }
        for (val, &count) in counts.iter().enumerate() {
            let observed = count as f64 / n as f64;
            let expected = DICE_PROBABILITIES[val];
            let diff = (observed - expected).abs();
            assert!(
                diff < 0.02,
                "roll {} observed {:.4} expected {:.4} diff {:.4} > 2%",
                val, observed, expected, diff
            );
        }
    }

    #[test]
    fn test_dice_roll_is_deterministic_given_seed() {
        let mut rng_a = StdRng::seed_from_u64(999);
        let mut rng_b = StdRng::seed_from_u64(999);
        for _ in 0..1000 {
            assert_eq!(Dice::roll(&mut rng_a), Dice::roll(&mut rng_b));
        }
    }
}
