/*!

This struct is used to store information about an extension of a match in an ACU
(Associative Commutative Unification) automaton. It has the following fields:

  * `valid_after_match`: a boolean flag that indicates whether the extension is valid after the match.
  * `matched_whole`: a boolean flag that indicates whether the entire subject of the extension matched.
  * `subject`: the subject of the extension, represented as a reference-counted (Rc) `ACUDagNode`.
  * `unmatched`: the unmatched part of the subject, represented as a red-black tree.
  * `unmatched_multiplicity`: a vector that stores the multiplicity of the unmatched part of the subject.
  * `upper_bound`: an unsigned 32-bit integer that represents the upper bound of the unmatched multiplicity.

*/


use std::rc::Rc;
use crate::theory::acu_theory::{ACUDagNode, RedBlackTree};
use crate::theory::{DagNode, ExtensionInfo, RcDagNode};

/// A struct that stores information about an extension of a match in an ACU automaton.
pub struct ACUExtensionInfo {
  /// A flag that indicates whether the extension is valid after the match.
  valid_after_match: bool,

  /// A flag that indicates whether the entire subject matched.
  matched_whole: bool,

  /// The subject of the extension.
  pub subject: Rc<ACUDagNode>,

  /// The unmatched part of the subject, represented as a red-black tree.
  pub unmatched: RedBlackTree,

  /// The multiplicity of the unmatched part of the subject.
  pub unmatched_multiplicity: Vec<u32>,

  /// The upper bound of the unmatched multiplicity.
  pub upper_bound: u32,
}

impl ExtensionInfo for ACUExtensionInfo {
  fn set_valid_after_match(&mut self, value: bool) {
    self.valid_after_match = value;
  }

  fn set_matched_whole(&mut self, value: bool) {
    self.matched_whole = value;
  }

  fn set_unmatched(&mut self, value: RcDagNode) {
    self.unmatched.insert(value)
  }
}


