/*!

ACU term

*/


use std::any::Any;
use crate::theory::acu_theory::dag_node::ACUDagNode;
use crate::theory::dag_node::DagNode;
use crate::theory::symbol::Symbol;
use crate::theory::term::Term;

// A "Pair" struct
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ACUTermRecord {
  term                      : Box<dyn Term>,
  multiplicity              : u32,
  abstraction_variable_index: u32,  // If subterm could enter our theory we abstract it
  collapse_to_our_theory    : bool, // First possible reason for variable abstraction
  match_our_identity        : bool, // Second possible reason for variable abstraction
}

pub struct ACUTerm {
  args: Vec<ACUTermRecord>,
  unique_collapse_subterm_index: u32
  top_symbol: Box<Symbol>
}

impl Term for ACUTerm {
  fn symbol(&self) -> &Symbol {
    self.top_symbol.as_ref()
  }

  fn compare_dag_arguments(&self, other: &dyn DagNode) -> u32 {
    match other.as_any().downcast_ref::<ACUDagNode>() {
      Some(acu_dag_node) => {
        let tree = acu_dag_node.tree;
      },
      None => panic!("&a isn't a B!"),
    };
  }
  }
}
