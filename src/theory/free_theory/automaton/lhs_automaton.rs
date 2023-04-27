/*!

Free theory automaton.

*/

use std::{any::Any, ops::DerefMut};
use std::cmp::Ordering;
use std::rc::Rc;

use crate::{
  core::{
    SpecialSort,
    Substitution,
    Sort
  },
  theory::{
    automaton::LHSAutomaton,
    DagNode,
    ExtensionInfo,
    MaybeSubproblem,
    RcDagNode,
    RcSymbol,
    Symbol,
    NodeList,
    RcLHSAutomaton,
    SubproblemSequence,
    Term,
    RcTerm
  },
  abstractions::RcCell,
};
use crate::theory::BxLHSAutomaton;
use super::super::{
  FreeTermOccurrence,
  VariableTermOccurrence,
  RcFreeTerm,
  BoundVariable,
  FreeOccurrence,
  FreeDagNode,
  FreeSymbol,
  RcFreeDagNode,
  RcFreeSymbol,
  FreeVariable,
  GroundAlien,
  NonGroundAlien,
  FreeTerm
};


const NONE_INDEX: i32 = -1;

#[derive(Clone)]
pub struct FreeSubterm {
  position   : u32,
  arg_index  : u32,
  symbol     : RcSymbol,
  save_index : i32,
}


pub struct FreeLHSAutomaton {
  top_symbol: RcSymbol,

  stack               : Vec<NodeList>,
  free_subterms       : Vec<FreeSubterm>,
  uncertain_variables : Vec<FreeVariable>,
  bound_variables     : Vec<BoundVariable>,
  ground_aliens       : Vec<GroundAlien>,

  non_ground_aliens: Vec<NonGroundAlien>,
}

impl FreeLHSAutomaton {
  pub fn new(
    free_symbols: &[FreeTermOccurrence],
    uncertain_vars: &[VariableTermOccurrence],
    bound_vars: &[VariableTermOccurrence],
    gnd_aliens: &[FreeTermOccurrence],
    non_gnd_aliens: &[FreeTermOccurrence],
    best_sequence: &[usize],
    sub_automata: &[BxLHSAutomaton],
  ) -> Self {
    let nr_free_symbols = free_symbols.len();
    let top_term: RcFreeTerm = free_symbols[0].term.clone();
    let top_symbol = top_term.borrow().symbol();
    let mut slot_nr = 1usize;

    top_term.borrow_mut().slot_index = 0;

    // Start with 1, because 0th term is `top_term`, which we set above.
    let mut free_subterms = (1..nr_free_symbols)
        .map(|i| {
          let oc           = &free_symbols[i];
          let parent       = free_symbols[oc.position as usize].term.clone();
          let term         = oc.term.clone();
          let symbol       = term.borrow().symbol();
          let free_subterm =
              FreeSubterm {
                position  : parent.borrow().slot_index,
                arg_index : oc.arg_index,
                symbol: symbol.clone(),
                save_index: term.borrow().term_members().save_index,
              };

          if symbol.arity() > 0 {
            term.borrow_mut().slot_index = slot_nr as u32;
            slot_nr += 1;
          }

          free_subterm
        })
        .collect::<Vec<_>>();

    let stack = vec![NodeList::new(); slot_nr];

    let uncertain_variables = uncertain_vars
        .iter()
        .map(|oc| {
          let parent = free_symbols[oc.position as usize].term.clone();
          let v = oc.term.clone();
          FreeVariable {
            position: parent.borrow().slot_index,
            arg_index: oc.arg_index,
            var_index: v.index as i32,
            sort: v.sort(),
          }
        })
        .collect::<Vec<_>>();

    let bound_variables = bound_vars
        .iter()
        .map(|oc| {
          let parent = free_symbols[oc.position as usize].term.clone();
          let v = oc.term.clone();
          BoundVariable {
            position: parent.borrow().slot_index,
            arg_index: oc.arg_index,
            var_index: v.index as i32,
          }
        })
        .collect::<Vec<_>>();

    let ground_aliens = gnd_aliens
        .iter()
        .map(|oc| {
          let parent = free_symbols[oc.position as usize].term.clone();
          GroundAlien {
            position: parent.borrow().slot_index,
            arg_index: oc.arg_index,
            alien: oc.term.clone(),
          }
        })
        .collect::<Vec<_>>();

    let non_ground_aliens = best_sequence
        .iter()
        .map(|&i| {
          let oc = &non_gnd_aliens[i];
          let parent = free_symbols[oc.position as usize].term.clone();
          NonGroundAlien {
            position: parent.borrow().slot_index,
            arg_index: oc.arg_index,
            automaton: sub_automata[i]
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
  ) -> (bool, MaybeSubproblem)
  {
    if subject.as_ref().symbol().as_ref() != self.top_symbol.as_ref() as &dyn Symbol {
      return (false, None);
    }

    if self.top_symbol.arity() == 0 {
      return (true, None);
    }

    // Maude casts to a FreeDagNode?! Presumably because they want `match` to be a virtual function on the base class.
    let mut subject_mut = subject.borrow_mut();
    if let Some(s) = subject_mut.as_any_mut()
                                .downcast_mut::<FreeDagNode>()
    {
      self.stack[0] = s.dag_node_members().args.new_ref();

      let mut stack_idx: usize = 0;
      // Match free symbol skeleton.
      for i in &self.free_subterms {
        let d = self.stack[i.position as usize][i.arg_index as usize].clone();
        if *d.borrow().symbol() != *i.symbol {
          return (false, None);
        }

        if i.save_index != NONE_INDEX {
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
        let b = solution.value(v);
        if b.is_none() {
          assert_ne!(d.borrow().get_sort_index(), SpecialSort::Unknown as i32, "missing sort information (2) for {:?}", d
              .borrow().symbol().name());
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
        if !self.stack[i.position as usize][i.arg_index as usize].eq(solution.value(i.var_index).as_ref().unwrap()) {
          return (false, None);
        }
      }

      for i in &self.ground_aliens {
        if i.alien.as_ref().compare_dag_node(&*self.stack[i.position as usize][i.arg_index as usize].borrow())
            .is_ne(){
          return (false, None);
        }
      }

      assert!(self.non_ground_aliens.len() > 0, "no nrNonGroundAliens");
      if !self.non_ground_aliens.is_empty() {
        let mut subproblems = SubproblemSequence::new();

        for i in &mut self.non_ground_aliens {

          if let (true, subproblem) =
              i.automaton.borrow_mut().match_(
                self.stack[i.position as usize][i.arg_index as usize].clone(),
                solution,
                // None
              )
          {
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
      return (true, None)
    }
    else {
      panic!("FreeLHSAutomaton::match called with non Free DagNode. This is a bug.");
    }

  }
}
