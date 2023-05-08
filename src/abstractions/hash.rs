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
      v2 = v2.shl(7i32).wrapping_add(v1);
    }
    self.value = hash2(v1, v2);
  }

  #[inline(always)]
  fn write_u32(&mut self, v: u32) {
    self.value = hash2(self.value, v as u64)
  }
  #[inline(always)]
  fn write_u64(&mut self, v: u64) {
    self.value = hash2(self.value, v as u64)
  }
  #[inline(always)]
  fn write_usize(&mut self, v: usize) {
    self.value = hash2(self.value, v as u64)
  }
  #[inline(always)]
  fn write_i32(&mut self, v: i32) {
    self.value = hash2(self.value, v as u64)
  }
  #[inline(always)]
  fn write_i64(&mut self, v: i64) {
    self.value = hash2(self.value, v as u64)
  }

}

pub struct FastHasherBuilder;

impl BuildHasher for FastHasherBuilder {
  type Hasher = FastHasher;

  #[inline(always)]
  fn build_hasher(&self) -> FastHasher {
    FastHasher::new()
  }
}
