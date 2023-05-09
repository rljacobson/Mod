/*!

The implementation of the compiler-related methods of the `Term` trait, which belong to the `TermCompiler` subtrait.

 */



use std::cell::RefCell;
use std::rc::Rc;
use crate::abstractions::{RcCell, rc_cell};
use crate::abstractions::rccell::rc_cell;
use crate::core::{NatSet, VariableInfo, BindingLHSAutomaton};
use crate::theory::{LHSAutomaton, RcLHSAutomaton, Term};
use crate::theory::free_theory::FreeTerm;
use crate::theory::term::TermCompiler;

impl TermCompiler for FreeTerm {
  fn compile_lhs(
    &self, match_at_top: bool,
    variable_info: &VariableInfo,
    bound_uniquely: &mut NatSet,
    subproblem_likely: &mut bool,  // Output parameter
  ) -> RcLHSAutomaton {
    let (mut free_symbols, mut other_symbols) = (Vec::new(), Vec::new());
    self.scan_free_skeleton(&mut free_symbols, &mut other_symbols);

    let (mut bound_variables, mut uncertain_variables, mut ground_aliens, mut non_ground_aliens) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new());

    for occurrence in other_symbols {
      let term = occurrence.term();
      if let Some(v) = term.as_variable_term() {
        let index = v.get_index();
        if bound_uniquely.contains(index) {
          bound_variables.push(occurrence);
        } else {
          bound_uniquely.insert(index);
          uncertain_variables.push(occurrence);
        }
      } else {
        if term.ground() {
          ground_aliens.push(occurrence);
        } else {
          non_ground_aliens.push(occurrence);
        }
      }
    }

    let mut best_sequence = CP_Sequence::new();
    let nr_aliens = non_ground_aliens.len();
    let mut sub_automata = vec![None; nr_aliens];
    *subproblem_likely = false;

    if nr_aliens > 0 {
      self.find_constraint_propagation_sequence(&non_ground_aliens, bound_uniquely, &mut best_sequence);

      for (i, seq_index) in best_sequence.sequence.iter().enumerate() {
        let mut spl = false;
        sub_automata[i] = Some(
          non_ground_aliens[*seq_index].term().compile_lhs2(false, variable_info, bound_uniquely, &mut spl)
        );
        *subproblem_likely = *subproblem_likely || spl;
      }
      assert!(bound_uniquely == best_sequence.bound, "bound clash");
    }

    Box::new(FreeLhsAutomaton::new(
      free_symbols,
      uncertain_variables,
      bound_variables,
      ground_aliens,
      non_ground_aliens,
      best_sequence.sequence,
      sub_automata,
    ));


    // Assume `a` is the compiled `LHSAutomaton`
    if self.term_members.save_index != -1 {
      a = rc_cell!(
      BindingLHSAutomaton::new(
      self.term_members.save_index,
  a
  )
  );
    }
    return a;
  }
}
