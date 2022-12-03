/*!

ACU term

*/


use std::any::Any;
use crate::Substitution;
use crate::theory::acu_theory::dag_node::ACUDagNode;
use crate::theory::dag_node::DagNode;
use crate::theory::symbol::Symbol;
use crate::theory::term::{OrderingValue, Term, Flags};

// A "Pair" struct
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ACUTermRecord {
  term                      : Box<dyn Term>,
  multiplicity              : u32,
  abstraction_variable_index: u32,  // If subterm could enter our theory we abstract it
  collapse_to_our_theory    : bool, // First possible reason for variable abstraction
  match_our_identity        : bool, // Second possible reason for variable abstraction
}

#[derive(Clone)]
pub struct ACUTerm {
  args      : Vec<ACUTermRecord>,
  unique_collapse_subterm_index: u32,
  top_symbol: Box<Symbol>,
  // flags     : u8
}

impl ACUTerm {

}

impl Term for ACUTerm {
  fn symbol(&self) -> &Symbol {
    self.top_symbol.as_ref()
  }

  fn is_stable(&self) -> bool {
    // (self.flags & Flags::Stable as u8) != 0
    true
  }

  // Returns zero if the terms are the same.
  fn compare_term_arguments(&self, other: &dyn Term) -> u32 {
    match other.as_any().downcast_ref::<ACUTerm>() {
      Some(acu_term) => {
        // Fail fast if lengths differ.
        let r = self.args.len() - acu_term.args.len();
        if r != 0 {
          return r;
        }
        // Compare corresponding terms.
        for (this_record, other_record) in self.args.iter().zip(acu_term.args.iter()) {
          let r = this_record.multiplicity - other_record.multiplicity;
          if r!= 0 {
            return r;
          }
          r = this_record.term.compare(other_record.term);
          if r!=0 {
            return r;
          }
        }
        // Identical
        return 0;
      },
      None => panic!("Could not downcast a Term to an ACUTerm."),
    };
  }

  fn compare_dag_arguments(&self, other: &dyn DagNode) -> u32 {
    match other.as_any().downcast_ref::<ACUDagNode>() {
      Some(acu_dag_node) => {
        // Fail fast if lengths differ.
        let r = self.args.len() - acu_dag_node.len();
        if r != 0 {
          return r as u32;
        }
        // Compare corresponding terms.
        for (this_record, (other_dag_node, other_multiplicity)) in self.args.iter().zip(acu_dag_node.iter()) {
          let r = this_record.multiplicity - other_multiplicity;
          if r!= 0 {
            return r;
          }
          r = this_record.term.compare(other_dag_node);
          if r!=0 {
            return r;
          }
        }
        // Identical
        return 0;
      }
      None => panic!("Could not downcast a Term to an ACUTerm."),
    };
  }


}
