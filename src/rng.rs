//! A small, fast pseudo-random generator (xorshift64*), seeded from the clock
//! and the process id, or from a number for repeatable results.

pub struct Rng(u64);

impl Rng {
    pub fn new() -> Rng {
        let t = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos() as u64);
        Rng::seeded(t ^ (u64::from(std::process::id()) << 32))
    }

    pub fn seeded(seed: u64) -> Rng {
        // splitmix64 step, so that small seeds give well-mixed states (never 0).
        let mut z = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        Rng((z ^ (z >> 31)) | 1)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// A number in `lo..hi` (`lo` when the range is empty).
    pub fn range(&mut self, lo: i64, hi: i64) -> i64 {
        if hi <= lo {
            return lo;
        }
        lo + (self.next_u64() % (hi - lo) as u64) as i64
    }

    pub fn below(&mut self, n: usize) -> usize {
        if n == 0 { 0 } else { (self.next_u64() % n as u64) as usize }
    }

    /// A number in 0.0..1.0.
    pub fn float(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    pub fn chance(&mut self, p: f64) -> bool {
        self.float() < p
    }

    pub fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.below(items.len())]
    }
}

impl Default for Rng {
    fn default() -> Self {
        Rng::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeatable_and_in_range() {
        let (mut a, mut b) = (Rng::seeded(7), Rng::seeded(7));
        for _ in 0..1000 {
            let x = a.range(-3, 5);
            assert_eq!(x, b.range(-3, 5));
            assert!((-3..5).contains(&x));
            assert!((0.0..1.0).contains(&a.float()));
            b.float();
        }
    }
}
