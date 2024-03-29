/*!

ACU term

*/



use std::cmp::Ordering;
use std::any::Any;

use crate::core::numeric_ordering;
use crate::theory::acu_theory::ACUDagNode;
use crate::theory::DagNode;
use crate::theory::Symbol;
use crate::theory::{Term, RcTerm, Flags};

// A "Pair" struct
#[derive(Clone)]
pub struct ACUTermRecord {
  term                      : RcTerm,
  multiplicity              : u32,
  abstraction_variable_index: u32,  // If subterm could enter our theory we abstract it
  collapse_to_our_theory    : bool, // First possible reason for variable abstraction
  match_our_identity        : bool, // Second possible reason for variable abstraction
}

#[derive(Clone)]
pub struct ACUTerm {
  args      : Vec<ACUTermRecord>,
  unique_collapse_subterm_index: u32,
  top_symbol: Box<dyn Symbol>,
  // flags  : u8
}

impl ACUTerm {

}

impl Term for ACUTerm {
  fn symbol(&self) -> &dyn Symbol {
    Box::as_ref(&self.top_symbol)
  }

  fn is_stable(&self) -> bool {
    // (self.flags & Flags::Stable as u8) != 0
    // ToDo: Why not just set the flag correctly?
    true
  }

  fn compare_term_arguments(&self, other: &dyn Term) -> Ordering {
    match other.as_any().downcast_ref::<ACUTerm>() {

      Some(acu_term) => {
        // Fail fast if lengths differ.
        let r = self.args.len() - acu_term.args.len();
        if r != 0 {
          return numeric_ordering(r as usize);
        }

        // Equal
        // Compare corresponding terms.
        for (this_record, other_record) in self.args.iter().zip(acu_term.args.iter()) {
          let r: u32 = this_record.multiplicity - other_record.multiplicity;
          if r != 0 {
            return numeric_ordering(r as usize);
          }

          let r = this_record.term.compare(other_record.term.as_ref());
          if r != Ordering::Equal {
            return r;
          }
        }

        // Identical
        return Ordering::Equal;
      },

      None => panic!("Could not downcast a Term to an ACUTerm."),
    };
  }

  // Todo: Is the actual value of this function needed, or just the sign? If the latter, use OrderingValue or Ordering.
  fn compare_dag_arguments(&self, other: &dyn DagNode) -> Ordering {
    match other.as_any().downcast_ref::<ACUDagNode>() {
      Some(acu_dag_node) => {
        // Fail fast if lengths differ.
        let r: i32 = self.args.len() as i32 - acu_dag_node.len() as i32;
        if r < 0 {
          return Ordering::Less;
        } else if r > 0 {
          return Ordering::Greater;
        }
        // Equal
        // Compare corresponding terms.
        for (this_record, (other_dag_node, other_multiplicity)) in self.args.iter().zip(acu_dag_node.iter_args()) {
          let r: i32 = this_record.multiplicity as i32 - other_multiplicity as i32;
          if r < 0 {
            return Ordering::Less;
          } else if r > 0 {
            return Ordering::Greater;
          }

          let r = this_record.term.compare_dag_node(other_dag_node.as_ref());
          if r != Ordering::Equal {
            return r;
          }
        }

        // Identical
        return Ordering::Equal;
      }
      None => panic!("Could not downcast a Term to an ACUTerm."),
    };
  }


  fn as_any(&self) -> &dyn Any {
      self
  }

}
