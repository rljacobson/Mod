/*!

The automaton for the pattern (the LHS).

*/

use crate::theory::acu_theory::ACUDagNode;
use crate::theory::{LhsAutomaton, Term};
use crate::theory::acu_theory::dag_node::ACUDagNode;


struct GroundAlien{
  term    : Box<Term>,
  multiplicity: u32
}


pub struct ACULHSAutomaton {
  max_pattern_multiplicity: u32,
  current_multiplicity    : Vec<u32>,
  total_lower_bound       : u32,
  total_upper_bound       : u32,
  total_multiplicity      : u32,
  ground_aliens           : Vec<GroundAlien>
}

impl ACULHSAutomaton {

  fn multiplicity_checks(&mut self, subject: &ACUDagNode ) -> bool {
    // Copy argument multiplicities and check for trivial failure.
    if self.max_pattern_multiplicity  > 1 {
      //	Because failure here is common we check this first.
      let mut ok = false;
      for child in self.args {
        if child.multiplicity >= self.max_pattern_multiplicity {
          ok = true;
          break;
        }
      }
      if !ok {
        return false;
      }
    }

    // ok:
    self.current_multiplicity.resize(subject.args.len(), 0);
    let mut total_subject_multiplicity = 0;
    for (idx, arg) in subject.args.iter().enumerate() {
      self.current_multiplicity[idx] = arg.multiplicity;
      total_subject_multiplicity += arg.multiplicity;

    }

    if total_subject_multiplicity < self.total_lower_bound ||
        total_subject_multiplicity > self.total_upper_bound {
      return false;
    }

    self.total_multiplicity = total_subject_multiplicity;
    true
  }


  pub fn eliminate_ground_aliens(&mut self, subject: &ACUDagNode) -> bool {
    for &alien in self.ground_aliens {
      subject.args.
    }
    true
  }

}




#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    assert_eq!(2 + 2, 4);
  }
}
