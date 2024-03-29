/*!

Free theory automaton.

*/

use std::{any::Any, cmp::Ordering, ops::DerefMut, rc::Rc};

// Free Theory
use super::super::{
  BoundVariable,
  FreeDagNode,
  FreeOccurrence,
  FreeSymbol,
  FreeTerm,
  FreeVariable,
  GroundAlien,
  NonGroundAlien,
  RcFreeDagNode,
  RcFreeSymbol,
  RcFreeTerm,
};
// Variable "Theory"
use crate::{
  abstractions::RcCell,
  core::{sort::SpecialSort, substitution::Substitution},
  theory::{
    variable::VariableTerm,
    BxLHSAutomaton,
    DagNode,
    ExtensionInfo,
    LHSAutomaton,
    MaybeSubproblem,
    NodeList,
    RcDagNode,
    RcLHSAutomaton,
    RcSymbol,
    RcTerm,
    SubproblemSequence,
    Symbol,
    Term,
  },
  NONE,
};


#[derive(Clone)]
pub struct FreeSubterm {
  position:   i32,
  arg_index:  i32,
  symbol:     RcSymbol,
  save_index: i32,
}


pub struct FreeLHSAutomaton {
  top_symbol: RcSymbol,

  stack:               Vec<NodeList>,
  free_subterms:       Vec<FreeSubterm>,
  uncertain_variables: Vec<FreeVariable>,
  bound_variables:     Vec<BoundVariable>,
  ground_aliens:       Vec<GroundAlien>,

  non_ground_aliens: Vec<NonGroundAlien>,
}

impl FreeLHSAutomaton {
  pub fn new(
    free_symbols: &Vec<FreeOccurrence>,
    uncertain_vars: &Vec<FreeOccurrence>,
    bound_vars: &Vec<FreeOccurrence>,
    gnd_aliens: &Vec<FreeOccurrence>,
    non_gnd_aliens: &Vec<FreeOccurrence>,
    best_sequence: &Vec<u32>,
    sub_automata: &Vec<RcLHSAutomaton>,
  ) -> Self {
    let free_symbol_count = free_symbols.len();
    let top_term = free_symbols[0].dereference_term::<FreeTerm>();
    let top_symbol = top_term.symbol();
    let mut slot_nr = 1;

    top_term.slot_index = 0;

    // Free symbol skeleton //
    // Start with 1, because 0th term is `top_term`, which we set above.
    let free_subterms = (1..free_symbol_count)
      .map(|i| {
        let oc: &FreeOccurrence = &free_symbols[i];
        let parent: &mut FreeTerm = free_symbols[oc.position as usize].dereference_term::<FreeTerm>();
        let term: &mut FreeTerm = oc.dereference_term::<FreeTerm>();
        let symbol: RcSymbol = term.symbol();
        let free_subterm = FreeSubterm {
          position:   parent.slot_index,
          arg_index:  oc.arg_index,
          symbol:     symbol.clone(),
          save_index: term.term_members().save_index,
        };

        if symbol.arity() > 0 {
          term.slot_index = slot_nr;
          slot_nr += 1;
        }

        free_subterm
      })
      .collect::<Vec<_>>();

    let stack = vec![NodeList::new(); slot_nr as usize];

    // Variables that may be bound //

    let uncertain_variables = uncertain_vars
      .iter()
      .map(|oc| {
        let parent = free_symbols[oc.position as usize].dereference_term::<FreeTerm>();
        let v = oc.dereference_term::<VariableTerm>();
        FreeVariable {
          position:  parent.slot_index,
          arg_index: oc.arg_index,
          var_index: v.index as i32,
          sort:      v.sort(),
        }
      })
      .collect::<Vec<_>>();

    // Variables that will be bound //

    let bound_variables = bound_vars
      .iter()
      .map(|oc| {
        let parent = free_symbols[oc.position as usize].dereference_term::<FreeTerm>();
        let v = oc.dereference_term::<VariableTerm>();
        BoundVariable {
          position:  parent.slot_index,
          arg_index: oc.arg_index,
          var_index: v.index as i32,
        }
      })
      .collect::<Vec<_>>();

    // Ground alien subterms //

    let ground_aliens = gnd_aliens
      .iter()
      .map(|oc| {
        let parent = free_symbols[oc.position as usize].dereference_term::<FreeTerm>();
        GroundAlien {
          position:  parent.slot_index,
          arg_index: oc.arg_index,
          alien:     oc.term,
        }
      })
      .collect::<Vec<_>>();

    // Non-ground alien subterms //

    let non_ground_aliens = best_sequence
      .iter()
      .map(|&i| {
        let occurance: &FreeOccurrence = &non_gnd_aliens[i as usize];
        let parent = free_symbols[occurance.position as usize].dereference_term::<FreeTerm>();
        NonGroundAlien {
          position:  parent.slot_index,
          arg_index: occurance.arg_index,
          automaton: sub_automata[i as usize].clone(),
        }
      })
      .collect::<Vec<_>>();

    FreeLHSAutomaton {
      top_symbol,
      stack,
      free_subterms,
      uncertain_variables,
      bound_variables,
      ground_aliens,
      non_ground_aliens,
    }
  }
}


impl LHSAutomaton for FreeLHSAutomaton {
  fn match_(
    &mut self,
    subject: RcDagNode,
    solution: &mut Substitution,
    // extension_info: Option<&mut dyn ExtensionInfo>,
  ) -> (bool, MaybeSubproblem) {
    if subject.as_ref().symbol().as_ref() != self.top_symbol.as_ref() as &dyn Symbol {
      return (false, None);
    }

    if self.top_symbol.arity() == 0 {
      return (true, None);
    }

    // Maude casts to a FreeDagNode?! Presumably because they want `match` to be a virtual function on the base class.
    let mut subject_mut = subject.borrow_mut();
    if let Some(s) = subject_mut.as_any_mut().downcast_mut::<FreeDagNode>() {
      self.stack[0] = s.dag_node_members().args.new_ref();

      let mut stack_idx: usize = 0;
      // Match free symbol skeleton.
      for i in &self.free_subterms {
        // It is important that this is _immutable_ access to the args list, because
        // a `SharedVec` is copy on write if the ref count is greater than 1.
        let d = self.stack[i.position as usize][i.arg_index as usize].clone();
        if *d.borrow().symbol() != *i.symbol {
          return (false, None);
        }

        if i.save_index != NONE {
          solution.bind(i.save_index, Some(d.clone()));
        }

        if i.symbol.arity() != 0 {
          stack_idx += 1;
          self.stack[stack_idx] = d.borrow().dag_node_members().args.new_ref();
        }
      }

      for i in &self.uncertain_variables {
        let d = self.stack[i.position as usize][i.arg_index as usize].clone();
        let v = i.var_index;
        let b = solution.get(v);
        if b.is_none() {
          assert_ne!(
            d.borrow().get_sort_index(),
            SpecialSort::Unknown as i32,
            "missing sort information (2) for {:?}",
            d.borrow().symbol().name()
          );
          if d.borrow().leq_sort(i.sort.as_ref()) {
            solution.bind(v, Some(d));
          } else {
            return (false, None);
          }
        } else {
          if !d.eq(b.as_ref().unwrap()) {
            return (false, None);
          }
        }
      }

      for i in &self.bound_variables {
        if !self.stack[i.position as usize][i.arg_index as usize].eq(solution.get(i.var_index).as_ref().unwrap()) {
          return (false, None);
        }
      }

      for i in &self.ground_aliens {
        let term: &mut dyn Term = unsafe { &mut *i.alien };
        if term
          .compare_dag_node(&*self.stack[i.position as usize][i.arg_index as usize].borrow())
          .is_ne()
        {
          return (false, None);
        }
      }

      assert!(self.non_ground_aliens.len() > 0, "no nrNonGroundAliens");
      if !self.non_ground_aliens.is_empty() {
        let mut subproblems = SubproblemSequence::new();

        for i in &mut self.non_ground_aliens {
          if let (true, subproblem) = i.automaton.borrow_mut().match_(
            self.stack[i.position as usize][i.arg_index as usize].clone(),
            solution,
            // None
          ) {
            // Destructure `subproblem`
            if let Some(sp) = subproblem {
              subproblems.add(sp);
            }
          } else {
            return (false, None);
          }
        }
        return (true, Some(subproblems.extract_subproblem()));
      }
      return (true, None);
    } else {
      panic!("FreeLHSAutomaton::match called with non Free DagNode. This is a bug.");
    }
  }
}
