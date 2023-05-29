use std::rc::Rc;
use crate::core::pre_equation::PreEquation;

use super::{
  FreeVariable,
  BoundVariable,
  GroundAlien,
  NonGroundAlien
};

pub type RcFreeRemainder = Rc<FreeRemainder>;
pub type FreeRemainderList = Vec<RcFreeRemainder>;

pub struct FreeRemainder {
  //	To qualify for "fast" treatment the associated equation must:
  //	(1) have a lhs that parses into a non-error sort
  //	(2) have only free symbols in lhs
  //	(3) be left linear
  //	(4) be unconditional
  //	(5) have no "problem" variables (ones which need their bindings copied to avoid
  //	    eager evaluation of lazy subterm)
  //	(6) have the sort of each variable qualify with fastGeqSufficient()
  //	To qualify for "super-fast", additionally each variable must have a sort that
  //	is the unique maximal user sort in its component which must be error-free.
  //
  /// > 0 super-fast; < 0 fast; = 0 slow
  fast              : u8 ,
  /// remainder consists of a foreign equation that might collapse into free theory
  foreign           : bool ,
  free_variables    : Vec<FreeVariable> ,
  /// equation we are a remainder of
  equation          : Box<PreEquation>,
  bound_variables   : Vec<BoundVariable> ,
  ground_aliens     : Vec<GroundAlien> ,
  non_ground_aliens : Vec<NonGroundAlien> ,
}
