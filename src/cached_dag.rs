/*!

When a DAG is created for a `Term`, it is cached in a `CachedDag`. Likewise for instructions emitted for a DAG.

*/

use crate::theory::{RcDagNode, RcTerm};

pub struct CachedDag{
  pub(crate) term: Option<RcTerm>,
  pub(crate) dag_node: Option<RcDagNode>,
  // instruction_sequence: RcInstructionSequence
}


impl CachedDag {

}
