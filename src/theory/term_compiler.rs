/*!

Theory independent part of compiling a term to an automaton.

These are not methods on `Term`, because a particular term might be replaced by an already compiled copy of itself.
Compilation takes not only terms but other objects as input and produces a compiled object as output. It's not just
something done to a term.

*/


use std::ops::Deref;
use crate::{
  core::{
    automata::{
      CopyRHSAutomaton,
      RHSBuilder,
    },
    automata::TrivialRHSAutomaton,
    TermBag,
    VariableInfo,
  },
  NONE,
  theory::{
    RcTerm,
    Term,
    variable::VariableTerm,
  },
};

/// Compiles the RHS automaton, returning the tuple `(rhs_automaton, subproblem_likely).
pub fn compile_rhs(
  term           : RcTerm,
  rhs_builder    : &mut RHSBuilder,
  variable_info  : &mut VariableInfo,
  available_terms: &mut TermBag,
  eager_context  : bool
) -> i32
{
  if let Some(found_term) = available_terms.find(&*term.borrow(), eager_context) {
    let mut found_term = found_term.borrow_mut();

    if found_term.term_members_mut().save_index == NONE {
      if let Some(vt) = found_term.as_any().downcast_ref::<VariableTerm>() {
        return vt.index
      }

      found_term.term_members_mut().save_index = variable_info.make_protected_variable() as i32;
    }

    return found_term.term_members_mut().save_index;
  }

  if let Some(vt) = term.borrow_mut().as_any_mut().downcast_mut::<VariableTerm>() {
    let var_index = vt.index;

    if vt.is_eager_context() {
      let index = variable_info.make_construction_index();
      rhs_builder.add_rhs_automaton(Box::new(CopyRHSAutomaton::new(var_index, index)));
      vt.term_members_mut().save_index = index;
      available_terms.insert_built_term(term, true);
      return index;
    }
    return var_index;
  }

  let index = term.borrow_mut().compile_rhs_aux(rhs_builder, variable_info, available_terms, eager_context);
  term.borrow_mut().term_members_mut().save_index = index;
  available_terms.insert_built_term(term, eager_context);
  return index;
}

pub(crate) fn compile_top_rhs(
  term           : RcTerm,
  rhs_builder    : &mut RHSBuilder,
  variable_info  : &mut VariableInfo,
  available_terms: &mut TermBag
) {
  let index = compile_rhs(term, rhs_builder, variable_info, available_terms, true);
  variable_info.use_index(index);
  //
  // If we don't have any automata we must create one, if only to do the
  // replacement.
  //
  if rhs_builder.is_empty() {
    rhs_builder.add_rhs_automaton(Box::new(TrivialRHSAutomaton::new(index)));
  }
}
