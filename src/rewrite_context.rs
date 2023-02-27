/*!

Holds state for a rewrite system.

Maude uses inheritence: A `RewritingContext` is-a `Substitution`. We use composition: A `RewritingContext` has-a `Substitution`.

*/

use crate::redex_position::RedexPosition;
use crate::Substitution;
use crate::theory::RcDagNode;


pub enum Purpose {
  ConditionEval,
  SortEval,
  Other
}

pub enum RewriteType {
  Normal,
  Builtin,
  Memoized
}


pub struct RewritingContext {
  pub solution: Substitution,

  trace_flag: bool,
  progress: bool,

  root_node: RcDagNode,

  mb_count: u64,
  eq_count: u64,
  rl_count: u64,

  narrowing_count: u64,
  variant_narrowing_count: u64,

  //	For rule rewriting
  redex_stack: Vec<RedexPosition>,
  stale_marker: u32,
  lazy_marker: u32,
  current_index: u32,
  rewrite_limit: u64,
  gas_per_node: u64,
  current_gas: u64,
}


