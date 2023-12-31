/*!

A `RedexPosition` holds position information of a potential redex.

*/

use crate::theory::RcDagNode;


#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum RedexPositionFlags {
  Stale = 1,
  Eager = 2,
}
// Local convenience
use RedexPositionFlags::{Eager, Stale};

pub struct RedexPosition {
  pub dag_node:     RcDagNode,
  pub parent_index: i32,
  pub arg_index:    i32,
  pub flags:        u8,
}

impl RedexPosition {
  pub fn is_stale(&self) -> bool {
    (self.flags & Stale as u8) == (Stale as u8)
  }

  pub fn is_eager(&self) -> bool {
    (self.flags & Eager as u8) == (Eager as u8)
  }

  pub fn set_stale(&mut self, value: bool) {
    if value {
      self.flags |= Stale as u8;
    } else {
      self.flags &= !(Stale as u8);
    }
  }

  pub fn set_eager(&mut self, value: bool) {
    if value {
      self.flags |= Eager as u8;
    } else {
      self.flags &= !(Eager as u8);
    }
  }
}
