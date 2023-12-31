use std::{assert_matches::assert_matches, cmp::Ordering};

use tiny_logger::{log, Channel};

use super::{PreEquation as SortConstraint, PreEquationKind, RcPreEquation as RcSortConstraint};
use crate::{
  core::{
    rewrite_context::{trace::trace_status, RewriteType, RewritingContext},
    sort::{index_leq_sort, sort_leq_index},
  },
  theory::{DagNode, MaybeSubproblem, RcDagNode, RcSubproblem, Subproblem},
};

#[derive(Default)]
pub struct SortConstraintTable {
  constraints: Vec<Option<RcSortConstraint>>,
  complete:    bool,
}

impl SortConstraintTable {
  #[inline(always)]
  pub fn offer_sort_constraint(&mut self, sort_constraint: RcSortConstraint) {
    if self.accept_sort_constraint(sort_constraint.clone()) {
      self.constraints.push(Some(sort_constraint));
    }
  }

  #[inline(always)]
  pub fn sort_constraint_free(&self) -> bool {
    self.constraints.is_empty()
  }

  // This is identical to `constrain_to_smaller_sort`.
  // #[inline(always)]
  // pub fn constrain_to_exact_sort(&mut self, subject: RcDagNode, context: &mut RewritingContext) {
  //   if !self.sort_constraint_free() {
  //     self.constrain_to_smaller_sort(subject, context);
  //   }
  // }


  #[inline(always)]
  pub fn safe_to_inspect_sort_constraints(&self) -> bool {
    self.complete
  }

  // Sort constraints are sorted in the order: largest index (smallest sort) first
  fn sort_constraint_lt(lhs: &SortConstraint, rhs: &SortConstraint) -> Ordering {
    if let PreEquationKind::SortConstraint { sort: lhs_sort, .. } = &lhs.kind {
      if let PreEquationKind::SortConstraint { sort: rhs_sort, .. } = &rhs.kind {
        // reverse order: large index --> small sort
        return rhs_sort.borrow().sort_index.cmp(&lhs_sort.borrow().sort_index);
        // IDEA: might want to weaken comparison and do a stable_sort()
        // BUT: stable_sort() requires a strict weak ordering - much stronger that
        // the partial ordering we have on sorts.
      }
    }
    unreachable!("Non SortConstraint PreEquation used in SortConstraint context. This is a bug.")
  }

  fn order_sort_constraints(&mut self) {
    // sort_constraints may contain sort constraints with variable lhs which have
    // too low a sort to ever match our symbol. However the sort of our symbol
    // is itself affected by sort constraints. So we "comb" out usable sort
    // constraints in successive passes; this is inefficient but we expect the number
    // of sort constraints to be very small so it's not worth doing anything smarter.
    self.complete = true; // not really complete until we've finished, but pretend it is
    let sort_constraint_count = self.constraints.len();
    if sort_constraint_count == 0 {
      return;
    }
    let mut all = std::mem::take(&mut self.constraints);
    let mut added_sort_constraint;
    loop {
      added_sort_constraint = false;
      for i in 0..sort_constraint_count {
        if let Some(sc) = &all[i] {
          // Because we set table_complete = true; accept_sort_constraint() may
          // inspect the table of sort_constraints accepted so far and make
          // a finer distinction than it could in offer_sort_constraint().
          if self.accept_sort_constraint(sc.clone()) {
            self.constraints.push(Some(sc.clone()));
            added_sort_constraint = true;
          } else {
            all[i] = Some(sc.clone());
          }
        }
      }
      if !added_sort_constraint {
        break;
      }
    }
    self
      .constraints
      .sort_by(|a, b| Self::sort_constraint_lt(a.unwrap().as_ref(), b.unwrap().as_ref()));
  }

  #[inline(always)]
  fn compile_sort_constraints(&mut self) {
    for constraint in self.constraints {
      constraint.unwrap().borrow_mut().compile(true);
    }
  }

  // Placeholder for the actual implementations of these methods
  fn accept_sort_constraint(&self, _sort_constraint: RcSortConstraint) -> bool {
    unimplemented!()
  }

  pub(crate) fn constrain_to_smaller_sort(&mut self, subject: RcDagNode, context: &mut RewritingContext) {
    if self.sort_constraint_free() {
      return;
    }

    if context.is_limited() {
      // Limited rewriting contexts don't support sort constraint application and
      // are only used for functionality that doesn't support sort constraints.
      log(
        Channel::Warning,
        1,
        format!(
          "ignoring sort constraints for {} because context is limited",
          subject.borrow()
        )
        .as_str(),
      );
      return;
    }

    let mut current_sort_index = subject.borrow().get_sort_index();

    // We try sort constraints, smallest sort first until one applies or
    // all remaining sort constraints have sort >= than our current sort.
    // Whenever we succeed in applying a sort constraint we start again
    // with the new sort, because earlier sort constraints (via collapse
    // or variable lhs patterns) may be able to test this new sort.
    'retry: loop {
      for sort_constraint in self.constraints {
        let mut sort_constraint = sort_constraint.unwrap().borrow_mut();

        if let PreEquationKind::SortConstraint { sort, .. } = &sort_constraint.kind {
          if index_leq_sort(current_sort_index, sort.as_ref()) {
            // Done!
            return;
          }

          if sort_leq_index(sort.as_ref(), current_sort_index) {
            // not equal because of previous test
            let variable_count = sort_constraint.variable_info.protected_variable_count();
            context.substitution.clear_first_n(variable_count as usize);

            if let (true, mut subproblem) = sort_constraint
              .lhs_automaton
              .unwrap()
              .borrow_mut()
              .match_(subject.clone(), &mut context.substitution)
            {
              if subproblem.is_none() || subproblem.as_mut().unwrap().solve(true, context) {
                // `subproblem` needs to be repackaged for `check_condition_simple`.
                let mut subproblem = if let Some(mut boxed_sp) = subproblem {
                  Some(boxed_sp.as_mut())
                } else {
                  None
                };

                if !sort_constraint.has_condition()
                  || sort_constraint.check_condition_simple(subject.clone(), context, subproblem)
                {
                  subproblem.take(); // equivalent to delete sp in C++
                  if trace_status() {
                    context.trace_pre_eq_application(
                      Some(subject.clone()),
                      Some(&*sort_constraint),
                      RewriteType::Normal,
                    );
                    if context.trace_abort() {
                      context.finished();
                      return;
                    }
                  }
                  context.mb_count += 1;
                  context.finished();

                  current_sort_index = sort.borrow().sort_index;
                  subject.borrow_mut().set_sort_index(current_sort_index);
                  continue 'retry;
                }
              }
            }
          }
          context.finished();
        } else {
          unreachable!("Found a non SortConstraint. This is a bug.");
        }
      }

      break;
    }
  }
}
