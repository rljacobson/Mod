/*!

A symbol belonging to the free theory.

*/

use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

use crate::{
  abstractions::{IString, RcCell},
  core::Strategy,
  theory::{
    DagNode,
    NodeList,
    RcDagNode,
    RcTerm,
    Symbol,
    SymbolMembers,
  },
};
use crate::core::interpreter::memo_table::SourceSet;
use crate::core::rewrite_context::RewritingContext;

use super::{FreeNet, RcFreeNet, FreeDagNode, FreeTerm};

pub type RcFreeSymbol = Rc<FreeSymbol>;

pub struct FreeSymbol {
  discrimination_net: FreeNet,

  // `SymbolMembers`
  symbol_members: SymbolMembers,

  // `Strategy`
  strategy: Strategy
}

impl FreeSymbol {
  pub fn new(name: IString, arity: u32, memo_flag: bool, strategy: Strategy) -> FreeSymbol {
    FreeSymbol{
      discrimination_net: Default::default(),
      symbol_members: SymbolMembers::new(name, arity, memo_flag),
      strategy
    }
  }

  pub fn make_term_with_args(self, args: Vec<RcTerm>) -> FreeTerm {
    FreeTerm::with_args(Rc::new(self), args)
  }


  fn complex_strategy(&self, subject: RcDagNode, context: &mut RewritingContext) -> bool {
    if self.is_memoized() {
      let mut from = SourceSet::new();
      self.memo_strategy(&mut from, subject.clone(), context);
      self.memo_enter(from, subject.clone());
      return false;
    }

    let nr_args = self.arity();
    let args = subject.args_mut(); // Assuming we have a similar method

    // Execute user-supplied strategy
    let user_strategy = self.get_strategy();
    let strat_len = user_strategy.len();
    let mut seen_zero = false;
    for i in 0..strat_len {
      let mut a = user_strategy[i];
      if a == 0 {
        if !seen_zero {
          for j in 0..nr_args {
            args[j].compute_true_sort(context);
          }
          seen_zero = true;
        }
        if (i + 1 == strat_len) && self.discrimination_net.apply_replace(subject.clone(), context) ||
            self.discrimination_net.apply_replace_no_owise(subject.clone(), context) {
          return true;
        }
      } else {
        a -= 1; // real arguments start at 0 not 1
        if seen_zero {
          args[a] = args[a].copy_reducible(); // Assuming a similar method exists
          // A previous call to discrimination_net.apply_replace() may have
          // computed a true sort for our subject which will be
          // invalidated by the reduce we are about to do.
          subject.repudiate_sort_info(); // Assuming a similar method exists
        }
        args[a].reduce(context);
      }
    }
    return false;
  }

}

impl Symbol for FreeSymbol {

  #[inline(always)]
  fn symbol_members(&self) -> &SymbolMembers {
    &self.symbol_members
  }

  #[inline(always)]
  fn symbol_members_mut (&mut self) -> &mut SymbolMembers{
    &mut self.symbol_members
  }

  #[inline(always)]
  fn as_any(&self) -> &dyn Any {
    self
  }

  fn rewrite(&mut self, subject: RcDagNode, context: &mut RewritingContext) -> bool {
    // println!("attempting {}", self);
    assert!(subject.borrow().symbol().as_ref().eq(self), "bad symbol");
    if self.strategy.is_standard {
      let arg_count = self.arity() as usize;
      let mut args = subject.borrow_mut().dag_node_members_mut().args.clone();
      for arg in args.iter_mut().take(arg_count).rev() {

        context.reduce_dag_node(arg.clone())
      }
      return self.discrimination_net.apply_replace(subject.clone(), context);
    }
    return self.complex_strategy(subject, context);
  }

}

