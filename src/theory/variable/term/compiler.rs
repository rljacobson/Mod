/*!

The implementation of the compiler-related methods of the `Term` trait, which belong to the `TermCompiler` subtrait.

*/



use std::cell::RefCell;
use std::rc::Rc;
use crate::abstractions::RcCell;
use crate::core::{NatSet, VariableInfo, BindingLHSAutomaton};
use crate::rc_cell;
use crate::theory::{LHSAutomaton, RcLHSAutomaton, Term};
use crate::theory::term::TermCompiler;
use crate::theory::variable::automaton::VariableLHSAutomaton;
use crate::theory::variable::VariableTerm;

impl TermCompiler for VariableTerm {
  fn compile_lhs(
    &self, match_at_top: bool,
    _variable_info: &VariableInfo,
    bound_uniquely: &mut NatSet,
    subproblem_likely: &mut bool  // Output parameter
  ) -> RcLHSAutomaton {

    bound_uniquely.insert(self.index as usize);
    *subproblem_likely = false;

    let mut a: RcLHSAutomaton =
      rc_cell!(
        VariableLHSAutomaton::new(
          self.index,
          self.sort().clone(),
          match_at_top
        )
      );

    // Assume `a` is the compiled `LHSAutomaton`
    if self.term_members.save_index != -1 /*None*/{
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
