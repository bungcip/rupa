//! Deterministic pseudo-random number generator for Rupa.
//!
//! Rupa uses a documented, seedable PRNG so that rendering is reproducible.
//! The default algorithm is xoshiro256** seeded through splitmix64.

/// A deterministic PRNG.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prng {
    state: [u64; 4],
}

impl Prng {
    /// Default seed used when a document does not declare one.
    pub const DEFAULT_SEED: u64 = 0;

    /// Create a new PRNG from a seed.
    pub fn new(seed: u64) -> Self {
        let mut sm = seed;
        let mut state = [0u64; 4];
        for slot in &mut state {
            *slot = splitmix64(&mut sm);
        }
        // Avoid an all-zero state.
        if state == [0u64; 4] {
            state[0] = 1;
        }
        Self { state }
    }

    /// Return the next 64-bit value.
    pub fn next_u64(&mut self) -> u64 {
        let result = (self.state[1].wrapping_mul(5))
            .rotate_left(7)
            .wrapping_mul(9);
        let t = self.state[1] << 17;
        self.state[2] ^= self.state[0];
        self.state[3] ^= self.state[1];
        self.state[1] ^= self.state[2];
        self.state[0] ^= self.state[3];
        self.state[2] ^= t;
        self.state[3] = self.state[3].rotate_left(45);
        result
    }

    /// Return a float in `[0, 1)`.
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Return a value in `[min, max)` for floats.
    pub fn next_f64_range(&mut self, min: f64, max: f64) -> f64 {
        min + (max - min) * self.next_f64()
    }

    /// Return a value in `[min, max]` for integers.
    pub fn next_u64_range(&mut self, min: u64, max: u64) -> u64 {
        if min >= max {
            return min;
        }
        min + self.next_u64() % (max - min + 1)
    }
}

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E3779B97F4A7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111B9);
    z ^ (z >> 31)
}

/// Backwards/alternative names.
/// Alias for [`Prng`].
pub type Rng = Prng;
/// Alias for [`Prng`].
pub type Random = Prng;
