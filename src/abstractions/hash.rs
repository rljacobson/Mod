/*!
Hash algorithm used to detect whether a term has changed, whether a DAG has already been
constructed for a (sub)term.

On 64-bit archs, Maude takes 32 bit parameters for `Term`s and 64 bit parameters for `DagNode`s.
Don't know why. Possibly a vestige of 32-bit platforms?

Maude:
 * Best function to date on empirical measurement.
 * The Term version is in symmetry with DagNode version.

 */
use std::ops::{Mul, Shr, BitXor, Shl};
use std::hash::{Hasher, BuildHasher};
use std::num::Wrapping;


#[inline(always)]
pub fn hash2<T>(v1: T, v2: T) -> T
  where
      T: Mul<Output = T> + Shr<u8, Output = T> + BitXor<Output = T> + Copy,
      Wrapping<T>: Mul<Output = Wrapping<T>>
{
  let wv1 = Wrapping(v1);
  let wv1_squared = wv1 * wv1;
  let v1_squared = wv1_squared.0;
  (v1_squared) ^ (v1 >> 16) ^ v2
}

#[inline(always)]
pub fn hash3<T>(v1: T, v2: T, v3: T) -> T
  where
      T: Mul<Output = T> + Shr<u8, Output = T> + BitXor<Output = T> + Copy,
      Wrapping<T>: Mul<Output = Wrapping<T>>
{
  let wv2   = Wrapping(v2);
  let wv3   = Wrapping(v3);
  let wv2v3 = wv2 * wv3;
  let v2v3  = wv2v3.0;

  hash2(v1, v2v3) // (v1 * v1) ^ (v1 >> 16) ^ (v2 * v3)
}


/// An implementation of the Rust `Hasher` API for the fast hash algorithms.
#[derive(Copy, Clone, Eq, PartialEq, Default, Hash, Debug)]
pub struct FastHasher {
  value: u64,
}

impl FastHasher {
  #[inline(always)]
  pub fn new() -> Self {
    FastHasher {
      value: 0,
    }
  }
}

impl Hasher for FastHasher {
  #[inline(always)]
  fn finish(&self) -> u64 {
    self.value
  }

  #[inline(always)]
  fn write(&mut self, bytes: &[u8]) {
    let mut v1 = 0u64;
    let mut v2 = 0u64;

    for byte in bytes {
      v1 = v1.wrapping_add(*byte as u64);
      v2 = v2.shl(7i32).wrapping_add(*byte as u64);
    }
    self.value = hash3(self.value, v1, v2);
  }

  #[inline(always)]
  fn write_u32(&mut self, v: u32) {
    self.value = hash3(self.value, self.value, v as u64)
  }
  #[inline(always)]
  fn write_u64(&mut self, v: u64) {
    self.value = hash3(self.value, self.value, v)
  }
  #[inline(always)]
  fn write_usize(&mut self, v: usize) {
    self.value = hash3(self.value, self.value, v as u64)
  }
  #[inline(always)]
  fn write_i32(&mut self, v: i32) {
    self.value = hash3(self.value, self.value, v as u64)
  }
  #[inline(always)]
  fn write_i64(&mut self, v: i64) {
    self.value = hash3(self.value, self.value, v as u64)
  }

}

pub type FastHasherBuilder = FastHasher;

impl BuildHasher for FastHasher {
  type Hasher = FastHasher;

  #[inline(always)]
  fn build_hasher(&self) -> FastHasher {
    FastHasher::new()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::hash::{Hash, Hasher};
  use std::num::Wrapping;

  #[test]
  fn test_hash2() {
    let result = hash2(1u64, 2u64);
    let expected = Wrapping(1u64) * Wrapping(1u64) ^ Wrapping(1u64 >> 16) ^ Wrapping(2u64);
    assert_eq!(result, expected.0);

    // test with different values
    let result = hash2(123u64, 456u64);
    let expected = Wrapping(123u64) * Wrapping(123u64) ^ Wrapping(123u64 >> 16) ^ Wrapping(456u64);
    assert_eq!(result, expected.0);
  }

  #[test]
  fn test_hash3() {
    let result = hash3(1u64, 2u64, 3u64);
    let expected = hash2(1u64, (Wrapping(2u64) * Wrapping(3u64)).0);
    assert_eq!(result, expected);
  }

  #[test]
  fn test_fast_hasher() {
    let mut hasher = FastHasher::new();
    assert_eq!(hasher.finish(), 0);

    hasher.write(&[1, 2, 3, 4]);
    let v0 = 0u64;
    let v1 = 1u64 + 2u64 + 3u64 + 4u64;
    let v2 = 0u64.shl(7u32).wrapping_add(1u64).shl(7u32).wrapping_add(2u64).shl(7u32).wrapping_add(3u64).shl(7u32).wrapping_add(4u64);
    let hash_result = hash3(v0, v1, v2);
    assert_eq!(hasher.finish(), hash_result);

    hasher.write_u32(1u32);
    let hash_result = hash3(hash_result, hash_result, 1u64);
    assert_eq!(hasher.finish(), hash_result);

    hasher.write_u64(2u64);
    let hash_result = hash3(hash_result, hash_result, 2u64);
    assert_eq!(hasher.finish(), hash_result);

    hasher.write_usize(3usize);
    let hash_result = hash3(hash_result, hash_result, 3u64);
    assert_eq!(hasher.finish(), hash_result);

    hasher.write_i32(4i32);
    let hash_result = hash3(hash_result, hash_result, 4u64);
    assert_eq!(hasher.finish(), hash_result);

    hasher.write_i64(5i64);
    let hash_result = hash3(hash_result, hash_result, 5u64);
    assert_eq!(hasher.finish(), hash_result);
  }


  #[test]
  fn test_fast_hasher_api() {
    let mut hasher = FastHasher::new();
    assert_eq!(hasher.finish(), 0);

    hasher.write(&[1, 2, 3, 4]);
    let v1 = 1u64 + 2u64 + 3u64 + 4u64;
    let v2 = 0u64.shl(7u64).wrapping_add(1u64).shl(7u64).wrapping_add(2u64).shl(7u64).wrapping_add(3u64).shl(7u64).wrapping_add(4u64);
    assert_eq!(hasher.finish(), hash2(v1, v2));

    1u32.hash(&mut hasher);
    assert_eq!(hasher.finish(), hash2(hash2(v1, v2), 1u64));
  }

  #[test]
  fn test_build_hasher() {
    let builder = FastHasherBuilder::new();
    let hasher = builder.build_hasher();

    assert_eq!(hasher.finish(), 0);
  }


  #[test]
  fn test_different_values_produce_different_hashes() {
    let mut hasher1 = FastHasher::new();
    hasher1.write(&[1, 2, 3, 4]);
    let hash1 = hasher1.finish();

    let mut hasher2 = FastHasher::new();
    hasher2.write(&[5, 6, 7, 8]);
    let hash2 = hasher2.finish();

    assert_ne!(hash1, hash2);
  }

  #[test]
  fn test_different_order_produce_different_hashes() {
    let mut hasher1 = FastHasher::new();
    hasher1.write(&[1, 2, 3, 4]);
    let hash1 = hasher1.finish();

    let mut hasher2 = FastHasher::new();
    hasher2.write(&[4, 3, 2, 1]); // reversed order
    let hash2 = hasher2.finish();

    assert_ne!(hash1, hash2);
  }

  #[test]
  fn test_hash2_different_values() {
    // this test will pass as long as hash2 does not produce collision on these specific inputs
    let hash1 = hash2(1u64, 2u64);
    let hash2 = hash2(3u64, 4u64);

    assert_ne!(hash1, hash2);
  }

  #[test]
  fn test_hello_world_hashes() {
    let mut hasher1 = FastHasher::new();
    "hello".hash(&mut hasher1);
    let hello_hash = hasher1.finish();

    let mut hasher2 = FastHasher::new();
    "world".hash(&mut hasher2);
    let world_hash = hasher2.finish();

    assert_ne!(hello_hash, world_hash);
  }

}
