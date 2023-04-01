/*!

We need a version of `std::cmp::Ordering` that also has an `Unknown` or `Undecided` variant.
In the Maude source, this enum is called `ReturnValue`.

There are also a couple of convenience free functions for converting a number to `Ordering` or `OrderingValue`  based
on the sign of the number.

*/


use std::cmp::Ordering;

// Todo: Instead of this custom enum, should we have `Option<Ordering>` or `Result<Ordering, ()>`?
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(i8)]
pub enum OrderingValue {
  Greater = 1,
  Less = -2,
  Equal = 0,
  Unknown = -1
}

impl From<Ordering> for OrderingValue {
  fn from(value: Ordering) -> Self {
    match value {
      Ordering::Less    => {OrderingValue::Less}
      Ordering::Equal   => {OrderingValue::Equal}
      Ordering::Greater => {OrderingValue::Greater}
    }
  }
}

impl From<i32> for OrderingValue {
  fn from(value: i32) -> Self {
    if value > 0 {
      OrderingValue::Greater
    } else if value < 0 {
      OrderingValue::Less
    } else {
      OrderingValue::Equal
    }
  }
}

#[inline(always)]
pub fn numeric_ordering<T>(value: T) -> Ordering
  where T: Into<isize>
{
  let value: isize = value.into();
  if value > 0 {
    Ordering::Greater
  } else if value < 0 {
    Ordering::Less
  } else {
    Ordering::Equal
  }
}

#[inline(always)]
pub fn numeric_ordering_value<T>(value: T) -> OrderingValue
  where T: Into<isize>
{
  let value: isize = value.into();
  if value > 0 {
    OrderingValue::Greater
  } else if value < 0 {
    OrderingValue::Less
  } else {
    OrderingValue::Equal
  }
}
