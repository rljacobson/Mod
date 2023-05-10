/*!

The implementation of the compiler-related methods of the `Term` trait, which belong to the `TermCompiler` subtrait.

 */



use std::cell::RefCell;
use std::rc::Rc;

use crate::{
  abstractions::{
    NatSet,
    RcCell,
    rc_cell,
  },
  core::{
    BindingLHSAutomaton,
    VariableInfo,
  },
  theory::{
    free_theory::{
      FreeOccurrence,
      FreeOccurrences
    },
    LHSAutomaton,
    RcLHSAutomaton,
    RcSymbol,
    Term,
    variable::VariableTerm
  },
};

use super::super::{FreeTerm, FreeLHSAutomaton};

// Only used locally. Other theories will have their own local version.
#[derive(Default)]
struct ConstraintPropagationSequence {
  sequence   : Vec<u32>,
  bound      : NatSet,
  cardinality: i32
}

impl FreeTerm {
  fn scan_free_skeleton(
    &self,
    free_symbols : &mut Vec<FreeOccurrence>,
    other_symbols: &mut Vec<FreeOccurrence>,
    parent       : u32,
    arg_index    : u32
  )
  {
    let our_position = free_symbols.len() as u32;
    let oc = FreeOccurrence::new(parent, arg_index, self.as_ptr().cast_mut());
    free_symbols.push(oc);

    for (i, t) in self.args.iter().enumerate() {
      if let Some(f) = t.borrow_mut().as_any_mut().downcast_mut::<FreeTerm>() {
        f.scan_free_skeleton(free_symbols, other_symbols, our_position, i as u32);
      } else {
        let oc2 = FreeOccurrence::new(our_position, i as u32, t.as_ptr());
        other_symbols.push(oc2);
      }
    }
  }


  fn find_constraint_propagation_sequence(
    aliens        : &Vec<FreeOccurrence>,
    bound_uniquely: &mut NatSet,
    best_sequence : &mut ConstraintPropagationSequence
  ) {
    let mut current_sequence: Vec<u32> = (0..aliens.len() as u32).collect();
    best_sequence.cardinality = -1;

    Self::find_constraint_propagation_sequence_helper(aliens, &mut current_sequence, bound_uniquely, 0, best_sequence);
    assert!(best_sequence.cardinality >= 0, "didn't find a sequence");
  }

  fn remaining_aliens_contain(
    aliens               : &Vec<FreeOccurrence>,
    current_sequence     : &Vec<u32>,
    step                 : usize,
    us                   : usize,
    interesting_variables: &NatSet
  ) -> bool
  {
    if interesting_variables.is_empty() {
      return false;
    }
    for i in step..aliens.len() {
      if i != us
          && !interesting_variables.is_disjoint(
          &aliens[current_sequence[i] as usize].term().occurs_below()
              )
      {
        return true;
      }
    }
    false
  }


  fn find_constraint_propagation_sequence_helper(
    aliens          : &Vec<FreeOccurrence>,
    current_sequence: &mut Vec<u32>,
    bound_uniquely  : &NatSet,
    mut step        : usize,
    best_sequence   : &mut ConstraintPropagationSequence,
  ) {
    let nr_aliens = aliens.len();

    // Add any alien that will "ground out match" to current sequence.
    // By matching these early we maximize the chance of early match failure,
    // and avoid wasted work at match time.
    for i in step..nr_aliens {
      if aliens[current_sequence[i] as usize].term().will_ground_out_match(bound_uniquely) {
        current_sequence.swap(step, i);
        step += 1;
      }
    }
    if step < nr_aliens {
      // Now we search over possible ordering of remaining NGAs.

      let mut new_bounds: Vec<NatSet> = Vec::with_capacity(nr_aliens);
      // debug_advisory(&format!(
      //   "FreeTerm::findConstraintPropagationSequence(): phase 1 step = {}",
      //   step
      // ));

      for i in step..nr_aliens {
        new_bounds[i] = bound_uniquely.clone();
        let t = aliens[current_sequence[i] as usize].term();
        t.analyse_constraint_propagation(&mut new_bounds[i]);

        // We now check if t has the potential to benefit from delayed matching.
        let mut unbound = t.occurs_below().clone();
        unbound.difference_with(&new_bounds[i]);
        if !Self::remaining_aliens_contain(
          &aliens,
          &current_sequence,
          step,
          i,
          &unbound,
        ) {
          // No, so commit to matching it here.

          // debug_advisory(&format!(
          //   "FreeTerm::findConstraintPropagationSequence(): step = {} committed to {}",
          //   step, t
          // ));

          current_sequence.swap(step, i);
          Self::find_constraint_propagation_sequence_helper(
            aliens,
            current_sequence,
            &new_bounds[i],
            step + 1,
            best_sequence,
          );

          return;
        }
      }

      // We didn't find a NGA that we could commit to matching without possibly missing a better sequence.
      // Now go over the NGAs again. This time we need to consider expanding multiple branches in the
      // search tree.
      // debug_advisory(&format!(
      //   "FreeTerm::findConstraintPropagationSequence(): phase 2 step = {}",
      //   step
      // ));
      let mut expanded_at_least_one_branch = false;

      for i in step..nr_aliens {
        //
        // We expand this branch if it binds something that could help another NGA.
        //
        let mut newly_bound_uniquely: NatSet = new_bounds[i].clone();
        newly_bound_uniquely.difference_with(bound_uniquely);
        if Self::remaining_aliens_contain(
          &aliens,
          &current_sequence,
          step,
          i,
          &newly_bound_uniquely,
        ) {
          // Explore this path.

          // debug_advisory(&format!(
          //   "FreeTerm::findConstraintPropagationSequence(): step = {} exploring {}",
          //   step, aliens[current_sequence[i]].term()
          // ));
          current_sequence.swap(step, i);
          Self::find_constraint_propagation_sequence_helper(
            aliens,
            current_sequence,
            &new_bounds[i],
            step + 1,
            best_sequence,
          );
          current_sequence.swap(step, i);
          expanded_at_least_one_branch = true;
        }
      }
      if expanded_at_least_one_branch {
        return;
      }

      //	If we get here, none of the remaining NGAs can bind a variable that could affect
      //	the ability of other NGAs to bind variables, so there is no point pursuing further
      //	exploration. But we still need to union any other variable they may bind and score
      //	the result by making a recursive call to our leaf case.

      // debug_advisory(&format!(
      //   "FreeTerm::findConstraintPropagationSequence(): phase 3 step = {}",
      //   step
      // ));
      let mut new_bound_union = NatSet::new();
      for i in step..nr_aliens {
        new_bound_union.union_with(&new_bounds[i]);
      }

      Self::find_constraint_propagation_sequence_helper(
        aliens,
        current_sequence,
        &new_bound_union,
        nr_aliens,
        best_sequence,
      );
      return;
    }

    // Leaf of search tree.
    let n = bound_uniquely.len() as i32;
    if n > best_sequence.cardinality {
      best_sequence.sequence = current_sequence.clone(); // deep copy
      best_sequence.bound = bound_uniquely.clone(); // deep copy
      best_sequence.cardinality = n;
    }
  }


  fn compile_lhs(
    &self,
    _match_at_top     : bool,
    variable_info    : &VariableInfo,
    bound_uniquely   : &mut NatSet,
  ) -> (RcLHSAutomaton, bool)
  {
    // We bin the arg terms according to the following categories.
    // First gather all symbols lying in or directly under free skeleton.
    let mut free_symbols  = FreeOccurrences::new();
    let mut other_symbols = FreeOccurrences::new();
    // See if we can fail on the free skeleton.
    self.scan_free_skeleton(&mut free_symbols, &mut other_symbols, 0, 0);

    // Now classify occurrences of non Free-Theory symbols into 4 types
    let mut bound_variables     = FreeOccurrences::new(); // guaranteed bound when matched against
    let mut uncertain_variables = FreeOccurrences::new(); // status when matched against uncertain
    let mut ground_aliens       = FreeOccurrences::new(); // ground alien subterms
    let mut non_ground_aliens   = FreeOccurrences::new(); // non-ground alien subterms



    for occurrence in other_symbols {

      if let Some(v) = occurrence.try_dereference_term::<VariableTerm>()  {
        let index = v.index as usize;
        if bound_uniquely.contains(index) {
          bound_variables.push(occurrence);
        } else {
          bound_uniquely.insert(index);
          uncertain_variables.push(occurrence);
        }
      } else {
        let term: &mut dyn Term = occurrence.term();
        if term.ground() {
          ground_aliens.push(occurrence);
        } else {
          non_ground_aliens.push(occurrence);
        }
      }
    }

    // Now reorder the non-ground alien args in an order most likely to fail fast.
    // Now we have to find a best sequence in which to match the
    //	non-ground alien subterms and generate subautomata for them

    let mut best_sequence     = ConstraintPropagationSequence::default();
    let mut sub_automata      = Vec::with_capacity(non_ground_aliens.len());
    let mut subproblem_likely = false;

    if non_ground_aliens.len() > 0 {
      Self::find_constraint_propagation_sequence(&non_ground_aliens, bound_uniquely, &mut best_sequence);

      for &sequence_index in &best_sequence.sequence {
        let (automata, spl): (RcLHSAutomaton, bool)
            = non_ground_aliens[sequence_index as usize]
                .term()
                .compile_lhs(false, variable_info, bound_uniquely);
        sub_automata.push(automata);
        subproblem_likely = subproblem_likely || spl;
      }
      assert!(*bound_uniquely == best_sequence.bound, "Bound clash. This is a bug.");
    }

    let mut automaton: RcCell<dyn LHSAutomaton> = rc_cell!(
      FreeLHSAutomaton::new(
        free_symbols,
        uncertain_variables,
        bound_variables,
        ground_aliens,
        non_ground_aliens,
        best_sequence.sequence,
        sub_automata,
      )
    );

    if self.term_members.save_index != -1 /* None */ {
      automaton = rc_cell!(BindingLHSAutomaton::new(self.term_members.save_index, automaton));
    }


    return (automaton, subproblem_likely);
  }


  fn analyse_constraint_propagation(&mut self, bound_uniquely: &mut NatSet) {
    // First gather all symbols lying in or directly under free skeleton.
    let mut free_symbols = Vec::new();
    let mut other_symbols = Vec::new();
    self.scan_free_skeleton(&mut free_symbols, &mut other_symbols, 0, 0);

    // Now extract the non-ground aliens and update BoundUniquely with variables
    // that lie directly under the free skeleton and thus will receive an unique binding.
    let mut non_ground_aliens = Vec::new();
    for occurrence in &other_symbols {
      let t = occurrence.term();
      if let Some(variable_term) = t.as_any_mut().downcast_mut::<VariableTerm>() {
        bound_uniquely.insert(variable_term.index as usize);
      } else if !t.ground() {
        non_ground_aliens.push(occurrence.clone());
      }
    }

    if !non_ground_aliens.is_empty() {
      // debug_advisory(&format!(
      //   "FreeTerm::analyseConstraintPropagation() : looking at {} and saw {} nonground aliens",
      //   self,
      //   non_ground_aliens.len()
      // ));

      // Now we have to find a best sequence in which to match the non-ground alien subterms. Sequences that pin down
      // unique values for variables allow those values to be propagated.
      let mut best_sequence = ConstraintPropagationSequence::default();

      Self::find_constraint_propagation_sequence_helper(
        &non_ground_aliens,
        &mut vec![],
        &bound_uniquely,
        0,
        &mut best_sequence,
      );

      bound_uniquely.union_with(&best_sequence.bound);
    }
  }


}
