/*!

Some pre-equations have conditions associated with them that must be satisfied in order for the pre-equation to
apply. The condition can have multiple parts, called fragments. These fragments are themselves terms that are matched
against. So `ConditionFragment` is like a "lite" version of `PreEquation`.

ToDo: Should this and PreEquation be unified or refactored.

*/


use std::{
  cell::RefCell,
  fmt::{Display, Formatter},
  rc::Rc,
};

use crate::{
  abstractions::{join_iter, NatSet, RcCell},
  core::{
    automata::RHSBuilder,
    format::{FormatStyle, Formattable},
    pre_equation::ConditionState,
    rewrite_context::RewritingContext,
    sort::RcSort,
    TermBag,
    VariableInfo,
  },
  theory::{LHSAutomaton, RcLHSAutomaton, RcTerm},
};

/// A `Condition` is a set of `ConditionFragments`.
pub type Conditions = Vec<RcConditionFragment>;
pub type RcConditionFragment = RcCell<ConditionFragment>;

pub enum ConditionFragment {
  Equality {
    lhs_term:  RcTerm,
    rhs_term:  RcTerm,
    builder:   RHSBuilder,
    lhs_index: i32,
    rhs_index: i32,
  },

  SortTest {
    lhs_term:  RcTerm,
    sort:      RcSort,
    builder:   RHSBuilder,
    lhs_index: i32,
  },

  Assignment {
    lhs_term:    RcTerm,
    rhs_term:    RcTerm,
    builder:     RHSBuilder,
    lhs_matcher: RcLHSAutomaton,
    rhs_index:   i32,
  },

  Rewrite {
    lhs_term:    RcTerm,
    rhs_term:    RcTerm,
    builder:     RHSBuilder,
    rhs_matcher: RcLHSAutomaton,
    lhs_index:   i32,
  },
}

use ConditionFragment::*;

use crate::{
  core::rewrite_context::{make_subcontext, Purpose, RcRewritingContext},
  theory::{find_available_terms, index_variables, term_compiler::compile_rhs},
};

impl ConditionFragment {
  pub fn check(&mut self, variable_info: &mut VariableInfo, bound_variables: &mut NatSet) {
    let mut unbound_variables = NatSet::new();

    // Handle variables in the pattern.
    match self {
      Equality { lhs_term, .. } | SortTest { lhs_term, .. } | Rewrite { lhs_term, .. } => {
        lhs_term.borrow_mut().normalize(true);
        index_variables(lhs_term.clone(), variable_info);
        variable_info.add_condition_variables(lhs_term.borrow().occurs_below());
        unbound_variables.union_in_place(lhs_term.borrow().occurs_below());
      }
      Assignment { lhs_term, .. } => {
        lhs_term.borrow_mut().normalize(true);
        index_variables(lhs_term.clone(), variable_info);
        variable_info.add_condition_variables(lhs_term.borrow().occurs_below());
      }
    }

    // assert!(
    //   !bound_variables.contains(&self.lhs.occurs_below()),
    //   "{:?}: all the variables in the left-hand side of assignment condition fragment {:?} are bound before the
    // matching takes place.",   self.lhs,
    //   self
    // );

    // Handle variables in the subject.
    match self {
      Equality { rhs_term, .. } | Assignment { rhs_term, .. } | Rewrite { rhs_term, .. } => {
        rhs_term.borrow_mut().normalize(true);
        index_variables(rhs_term.clone(), variable_info);
        variable_info.add_condition_variables(rhs_term.borrow().occurs_below());

        // Check for variables that are used before they are bound.
        unbound_variables.union_in_place(rhs_term.borrow().occurs_below());
      }
      _ => { /* noop */ }
    }

    unbound_variables.difference(bound_variables);
    variable_info.add_unbound_variables(&unbound_variables);

    // We will bind these variables.
    match &self {
      Rewrite { lhs_term, .. } | Assignment { lhs_term, .. } => {
        bound_variables.union_in_place(lhs_term.borrow().occurs_below());
      }
      _ => { /* noop */ }
    }
  }

  pub fn preprocess(&mut self) {
    match self {
      Assignment { lhs_term, rhs_term, .. } => {
        lhs_term.borrow_mut().fill_in_sort_info();
        rhs_term.borrow_mut().fill_in_sort_info();
        assert!(
          lhs_term.borrow().connected_component() == rhs_term.borrow().connected_component(),
          "component clash"
        );
        lhs_term.borrow_mut().analyse_collapses()
      }
      Equality { lhs_term, rhs_term, .. } => {
        lhs_term.borrow_mut().fill_in_sort_info();
        rhs_term.borrow_mut().fill_in_sort_info();
        assert!(
          lhs_term.borrow().connected_component() == rhs_term.borrow().connected_component(),
          "component clash"
        );
      }
      Rewrite { lhs_term, rhs_term, .. } => {
        lhs_term.borrow_mut().fill_in_sort_info();
        rhs_term.borrow_mut().fill_in_sort_info();
        assert!(
          lhs_term.borrow().connected_component() == rhs_term.borrow().connected_component(),
          "component clash"
        );
        rhs_term.borrow_mut().analyse_collapses()
      }
      SortTest { lhs_term, sort, .. } => {
        lhs_term.borrow_mut().fill_in_sort_info();
        assert!(
          lhs_term.borrow().connected_component() == sort.borrow().sort_component,
          "component clash"
        );
      }
    }
  }

  pub fn compile_build(&mut self, variable_info: &mut VariableInfo, available_terms: &mut TermBag) {
    match self {
      Equality {
        lhs_term,
        rhs_term,
        lhs_index,
        rhs_index,
        builder,
        ..
      } => {
        *lhs_index = compile_rhs(lhs_term.clone(), builder, variable_info, available_terms, true);
        *rhs_index = compile_rhs(rhs_term.clone(), builder, variable_info, available_terms, true);
        variable_info.use_index(*lhs_index);
        variable_info.use_index(*rhs_index);
        variable_info.end_of_fragment();
      }

      SortTest {
        lhs_term,
        lhs_index,
        builder,
        ..
      } => {
        *lhs_index = compile_rhs(lhs_term.clone(), builder, variable_info, available_terms, true);
        variable_info.use_index(*lhs_index);
        variable_info.end_of_fragment();
      }

      Assignment {
        lhs_term,
        rhs_term,
        rhs_index,
        builder,
        ..
      } => {
        *rhs_index = compile_rhs(rhs_term.clone(), builder, variable_info, available_terms, true);
        variable_info.use_index(*rhs_index);

        find_available_terms(lhs_term.clone(), available_terms, true, false);

        let mut lhs_term = lhs_term.borrow_mut();
        lhs_term.determine_context_variables();
        lhs_term.insert_abstraction_variables(variable_info);
        variable_info.end_of_fragment();
      }

      Rewrite {
        lhs_term,
        rhs_term,
        lhs_index,
        builder,
        ..
      } => {
        // ToDo: Why call `compile_rhs` on the lhs term?
        *lhs_index = compile_rhs(lhs_term.clone(), builder, variable_info, available_terms, true);
        variable_info.use_index(*lhs_index);

        find_available_terms(rhs_term.clone(), available_terms, true, false);

        let mut rhs_term = rhs_term.borrow_mut();
        rhs_term.determine_context_variables();
        rhs_term.insert_abstraction_variables(variable_info);
        variable_info.end_of_fragment();
      }
    }
  }

  pub fn compile_match(&mut self, variable_info: &mut VariableInfo, bound_uniquely: &mut NatSet) {
    match self {
      Equality {
        lhs_index,
        rhs_index,
        builder,
        ..
      } => {
        builder.remap_indices(variable_info);
        *lhs_index = variable_info.remap_index(*lhs_index);
        *rhs_index = variable_info.remap_index(*rhs_index);
      }

      SortTest { lhs_index, builder, .. } => {
        builder.remap_indices(variable_info);
        *lhs_index = variable_info.remap_index(*lhs_index);
      }

      Assignment {
        lhs_term,
        rhs_index,
        lhs_matcher,
        builder,
        ..
      } => {
        builder.remap_indices(variable_info);
        *rhs_index = variable_info.remap_index(*rhs_index);

        let (new_matcher, _subproblem_likely): (RcLHSAutomaton, bool) =
          lhs_term.borrow_mut().compile_lhs(false, variable_info, bound_uniquely);
        *lhs_matcher = new_matcher;

        bound_uniquely.union_in_place(lhs_term.borrow().occurs_below())
      }

      Rewrite {
        rhs_term,
        lhs_index,
        rhs_matcher,
        builder,
        ..
      } => {
        builder.remap_indices(variable_info);
        *lhs_index = variable_info.remap_index(*lhs_index);

        let (new_matcher, _subproblem_likely): (RcLHSAutomaton, bool) =
          rhs_term.borrow_mut().compile_lhs(false, variable_info, bound_uniquely);
        *rhs_matcher = new_matcher;

        bound_uniquely.union_in_place(rhs_term.borrow().occurs_below())
      }
    }
  }

  pub fn solve(&mut self, find_first: bool, solution: RcRewritingContext, state: &mut Vec<ConditionState>) -> bool {
    match self {
      Assignment { .. } => false,

      Equality {
        builder,
        lhs_index,
        rhs_index,
        ..
      } => {
        if !find_first {
          return false;
        }

        builder.safe_construct(&mut solution.borrow_mut().substitution);
        let lhs_root = solution.borrow().substitution.get(*lhs_index);
        let mut lhs_context = make_subcontext(solution.clone(), lhs_root, Purpose::ConditionEval);
        let rhs_root = solution.borrow().substitution.get(*rhs_index);
        let mut rhs_context = make_subcontext(solution.clone(), rhs_root, Purpose::ConditionEval);

        lhs_context.reduce();
        solution.borrow_mut().add_counts_from(&lhs_context);
        rhs_context.reduce();
        solution.borrow_mut().add_counts_from(&rhs_context);

        *lhs_context.root.unwrap().borrow_mut() == *rhs_context.root.unwrap().borrow_mut()
      }

      Rewrite { .. } => false,

      SortTest { .. } => false,
    }
  }
}

impl Formattable for ConditionFragment {
  fn repr(&self, style: FormatStyle) -> String {
    match self {
      ConditionFragment::Equality { lhs_term, rhs_term, .. } => {
        format!("{} = {}", lhs_term.borrow().repr(style), rhs_term.borrow().repr(style))
      }

      ConditionFragment::SortTest { lhs_term, sort, .. } => {
        format!("{} : {}", lhs_term.borrow().repr(style), sort.borrow())
      }

      ConditionFragment::Assignment { lhs_term, rhs_term, .. } => {
        format!("{} := {}", lhs_term.borrow().repr(style), rhs_term.borrow().repr(style))
      }

      ConditionFragment::Rewrite { lhs_term, rhs_term, .. } => {
        format!("{} => {}", lhs_term.borrow().repr(style), rhs_term.borrow().repr(style))
      }
    }
  }
}


pub fn repr_condition(condition: &Conditions, style: FormatStyle) -> String {
  let mut accumulator = "if ".to_string();
  accumulator.push_str(
    join_iter(condition.iter().map(|cf| cf.borrow().repr(style)), |_| {
      " ∧ ".to_string()
    })
    .collect::<String>()
    .as_str(),
  );

  accumulator
}
