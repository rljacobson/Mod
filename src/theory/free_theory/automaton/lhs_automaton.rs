/*!

Free theory automaton.

*/

use std::{any::Any, ops::DerefMut};
use std::cmp::Ordering;

use crate::{
  core::{Substitution, Sort},
  theory::{
    automaton::{
      BoundVariable,
      FreeVariable,
      GroundAlien,
      LHSAutomaton,
      NonGroundAlien
    },
    DagNode,
    ExtensionInfo,
    MaybeSubproblem,
    RcDagNode,
    RcSymbol,
    Symbol,
    NodeList,
  },
};
use crate::core::SpecialSort;
use crate::theory::SubproblemSequence;
use super::super::{
  FreeDagNode,
  FreeSymbol,
  RcFreeDagNode,
  RcFreeSymbol
};


#[derive(Clone)]
struct FreeSubterm {
  position   : u32,
  arg_index  : u32,
  symbol     : RcSymbol,
  save_index : i32,
}


pub struct FreeLHSAutomaton {
  top_symbol: RcFreeSymbol,
  // ToDo: Use TinyVec or equiv. Maude stores a couple of args inline before allocating.
  args      : NodeList,

  // TODO: This is supposed to be a list of lists of RcDagNodes?
  stack               : Vec<NodeList>,
  free_subterms       : Vec<FreeSubterm>,
  uncertain_variables : Vec<FreeVariable>,
  bound_variables     : Vec<BoundVariable>,
  ground_aliens       : Vec<GroundAlien>,

  // ToDo: These are owned by `FreeLHSAutomaton`.
  non_ground_aliens: Vec<NonGroundAlien>,
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

        if i.save_index != -1 /* NONE */ {
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
              i.automaton.match_(
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
    } else {
      panic!("ACULHSAutomaton::match  called with non ACU DagNode. This is a bug.");
    }

  }
}
