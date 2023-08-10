/*!

This is a variation of FreeLHSAutomaton for matching whatever is left of a pattern after free
symbols have been matched.

ToDo: Merge with FreeLHSAutomaton.

*/

use std::rc::Rc;
use crate::core::format::{FormatStyle, Formattable};
use crate::core::pre_equation::{PreEquation, RcPreEquation};
use crate::NONE;
use crate::theory::free_theory::{FreeOccurrence, FreeTerm};
use crate::theory::{NodeList, RcLHSAutomaton, RcSymbol, Term};
use crate::theory::variable::VariableTerm;

use super::{
  FreeVariable,
  BoundVariable,
  GroundAlien,
  NonGroundAlien
};

pub type RcFreeRemainder = Rc<FreeRemainder>;
pub type FreeRemainderList = Vec<Option<RcFreeRemainder>>;

/// There are potentially three ways to compute the remainder, two of which are shortcut
/// optimizations.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
#[repr(i8)]
pub enum Speed {
  // > 0 super-fast; < 0 fast; = 0 slow
  Fast = -1,
  Slow =  0,
  SuperFast = 1,
}

pub struct FreeRemainder {
  //	To qualify for "fast" treatment the associated equation must:
  //	(1) have a lhs that parses into a non-error sort
  //	(2) have only free symbols in lhs
  //	(3) be left linear
  //	(4) be unconditional
  //	(5) have no "problem" variables (ones which need their bindings copied to avoid
  //	    eager evaluation of lazy subterm)
  //	(6) have the sort of each variable qualify with fastGeqSufficient()
  //	To qualify for "super-fast", additionally each variable must have a sort that
  //	is the unique maximal user sort in its component which must be error-free.
  //
  /// > 0 super-fast; < 0 fast; = 0 slow
  pub(crate) fast: Speed,
  /// remainder consists of a foreign equation that might collapse into free theory
  foreign           : bool ,
  free_variables    : Vec<FreeVariable> ,
  /// equation we are a remainder of
  equation          : RcPreEquation,
  bound_variables   : Vec<BoundVariable> ,
  ground_aliens     : Vec<GroundAlien> ,
  non_ground_aliens : Vec<NonGroundAlien> ,
}

impl FreeRemainder {

  /// Constructs a slow foreign remainder with the given equation.
  pub fn with_equation(equation: RcPreEquation) -> Self {
    FreeRemainder{
      fast: Speed::Slow,
      foreign: true,
      free_variables: vec![],
      equation,
      bound_variables: vec![],
      ground_aliens: vec![],
      non_ground_aliens: vec![],
    }
  }

  pub fn new(
    equation        : RcPreEquation,
    free_symbols    : &Vec<FreeOccurrence>,
    free_variables  : &Vec<FreeOccurrence>,
    bound_variables : &Vec<FreeOccurrence>,
    gnd_aliens      : &Vec<FreeOccurrence>,
    non_gnd_aliens  : &Vec<FreeOccurrence>,
    best_sequence   : &Vec<u32>,
    sub_automata    : &Vec<RcLHSAutomaton>,
    slot_translation: &Vec<i32>
  ) -> Self {
    // Preliminary determination of whether remainder will qualify for "fast" or "super-fast"
    // runtime treatment.
    let mut fast: Speed = if !(equation.borrow().has_condition()) { Speed::SuperFast } else { Speed::Slow };


    // Variables that will be unbound //
    let mut new_free_variables = free_variables
        .iter()
        .map(|oc| {
          let parent = free_symbols[oc.position as usize].dereference_term::<FreeTerm>();
          let v = oc.dereference_term::<VariableTerm>();
          let sort = v.sort();

          if !(sort.borrow().fast_geq_sufficient()) {
            fast = Speed::Slow; // Need slow handling for full sort check
          }  else {
            if fast == Speed::SuperFast { // Currently super-fast
              if !(sort.borrow().error_free_maximal()) {
                fast = Speed::Fast; // Downgrade to fast
              }
            }
          }

          FreeVariable {
            position : slot_translation[parent.slot_index as usize],
            arg_index: oc.arg_index,
            var_index: v.index as i32,
            sort,
          }
        })
        .collect::<Vec<_>>();

    // Pseudo variables for left to right sharing //
    for oc in free_symbols {
      let free_term: &mut FreeTerm = oc.dereference_term::<FreeTerm>();
      if free_term.term_members.save_index != NONE {
        let index  = free_term.term_members.save_index;
        let parent = free_symbols[oc.position as usize].dereference_term::<FreeTerm>();
        // format!("bad slot for {} in {}", parent.repr(FormatStyle::Simple), equation.borrow().repr(FormatStyle::Simple)).as_str()
        assert!(parent.slot_index != NONE, "bad slot for parent in equation");
        let new_free_var = FreeVariable {
          position : slot_translation[parent.slot_index as usize],
          arg_index: oc.arg_index,
          var_index: index,
          sort: free_term.connected_component()
                         .borrow()
                         .sort(0)
                         .upgrade()
                         .unwrap(),
        };
        new_free_variables.push(new_free_var);
      }
    }

    // Variables that will be bound //
    let new_bound_variables = bound_variables
        .iter()
        .map(|oc| {
          let parent = free_symbols[oc.position as usize].dereference_term::<FreeTerm>();
          // format!("bad slot for {} in {}", parent.repr(FormatStyle::Simple), equation.borrow().repr(FormatStyle::Simple)).as_str()
          assert!(parent.slot_index != NONE, "bad slot for parent in equation");
          let v = oc.dereference_term::<VariableTerm>();
          fast = Speed::Slow;  // Need slow handling if there are nonlinear variables
          BoundVariable {
            position : slot_translation[parent.slot_index as usize],
            arg_index: oc.arg_index,
            var_index: v.index,
          }
        })
        .collect::<Vec<_>>();


    // Ground alien subterms //
    let ground_aliens = gnd_aliens
        .iter()
        .map(|oc| {
          let parent = free_symbols[oc.position as usize].dereference_term::<FreeTerm>();
          fast = Speed::Slow;  // Need slow handling if there are nonlinear variables
          GroundAlien {
            position : parent.slot_index,
            arg_index: oc.arg_index,
            alien    : oc.term,
          }
        })
        .collect::<Vec<_>>();


    // Non-ground alien subterms //
    let non_ground_aliens = best_sequence
        .iter()
        .map(|&i| {
          let occurance: &FreeOccurrence = &non_gnd_aliens[i as usize];
          let parent = free_symbols[occurance.position as usize].dereference_term::<FreeTerm>();
          fast = Speed::Slow;  // Need slow handling if there are nonlinear variables
          NonGroundAlien {
            position : slot_translation[parent.slot_index as usize],
            arg_index: occurance.arg_index,
            automaton: sub_automata[i as usize].clone()
          }
        })
        .collect::<Vec<_>>();

    FreeRemainder {
      fast,
      foreign: false,
      free_variables: new_free_variables,
      equation,
      bound_variables: new_bound_variables,
      ground_aliens,
      non_ground_aliens,
    }
  }
}


