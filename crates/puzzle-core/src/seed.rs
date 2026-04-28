use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

/// FNV-1a offset basis for 64-bit hash.
const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;

/// FNV-1a prime for 64-bit hash.
const FNV_PRIME: u64 = 0x100000001b3;

/// Hash a seed string to a deterministic u64 using FNV-1a.
///
/// This is a portable, deterministic hash that produces the same
/// value across all Rust versions and platforms.
///
/// CRITICAL: Do NOT replace with `std::hash::DefaultHasher` — it is
/// NOT portable across Rust compiler versions.
pub fn hash_seed(s: &str) -> u64 {
    let mut hash: u64 = FNV_OFFSET_BASIS;
    for byte in s.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// Hash a seed string + an unordered piece pair to one bit. Used to
/// pick a deterministic knob direction per *edge identity*, not per
/// generation order — without this, knob directions flicker during
/// interactive drags because Voronoi cell ordering shifts as Lloyd
/// re-relaxes seeds. Hashing on the piece pair makes the choice
/// invariant to insertion order while still randomized by seed.
///
/// Pieces are sorted into (lo, hi) so `(a, b)` and `(b, a)` produce
/// the same hash — every edge has one canonical direction.
pub fn hash_pair_bit(seed: u64, a: usize, b: usize) -> bool {
    let lo = a.min(b) as u64;
    let hi = a.max(b) as u64;
    let mut hash = seed;
    for chunk in [lo, hi] {
        for byte in chunk.to_le_bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(FNV_PRIME);
        }
    }
    hash & 1 == 0
}

/// Create a deterministic RNG from a seed string.
///
/// Hashes the string to u64 via FNV-1a, then seeds a ChaCha8Rng.
/// Same string always produces the same RNG sequence across platforms.
pub fn create_rng(seed_string: &str) -> ChaCha8Rng {
    let hash = hash_seed(seed_string);
    ChaCha8Rng::seed_from_u64(hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::RngExt;

    #[test]
    fn test_hash_seed_deterministic() {
        let h1 = hash_seed("birthday");
        let h2 = hash_seed("birthday");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_hash_seed_different_inputs() {
        let h1 = hash_seed("birthday");
        let h2 = hash_seed("christmas");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_hash_seed_empty_is_offset_basis() {
        assert_eq!(hash_seed(""), FNV_OFFSET_BASIS);
    }

    #[test]
    fn test_create_rng_deterministic_sequence() {
        let mut rng = create_rng("test");
        let b1: bool = rng.random_bool(0.5);
        let b2: bool = rng.random_bool(0.5);
        let b3: bool = rng.random_bool(0.5);

        // Verify same sequence on second creation
        let mut rng2 = create_rng("test");
        assert_eq!(rng2.random_bool(0.5), b1);
        assert_eq!(rng2.random_bool(0.5), b2);
        assert_eq!(rng2.random_bool(0.5), b3);
    }

    #[test]
    fn test_create_rng_identical_instances() {
        let mut rng1 = create_rng("test");
        let mut rng2 = create_rng("test");

        // Generate a longer sequence and verify identity
        for _ in 0..100 {
            let v1: f64 = rng1.random();
            let v2: f64 = rng2.random();
            assert_eq!(v1, v2);
        }
    }

    #[test]
    fn test_different_seeds_different_sequences() {
        let mut rng1 = create_rng("seed-a");
        let mut rng2 = create_rng("seed-b");

        // At least one value in 10 should differ
        let mut any_different = false;
        for _ in 0..10 {
            let v1: f64 = rng1.random();
            let v2: f64 = rng2.random();
            if v1 != v2 {
                any_different = true;
                break;
            }
        }
        assert!(
            any_different,
            "Different seeds should produce different sequences"
        );
    }
}
