/*!
A discrimination net for the Free theory.

A discrimination net is a data structure used to optimize rule-based pattern matching. It improves
efficiency by organizing the conditions of the rules in a tree-like structure, enabling the system
to check many rules simultaneously. An input data item is classified by conducting a series of
individual predicate tests. The internal nodes of the tree are the conditions that are tested, while
the end nodes of the net symbolize the outcomes for the different possible predicate sequences.

*/

use std::{collections::HashSet, default};
use std::rc::Rc;

use crate::{abstractions::{
  WeakCell,
  FastHasher
}, core::module::ModuleItem, NONE, theory::{
  DagNode,
  RcDagNode,
  RcSymbol,
  NodeList,
}};
use crate::core::pre_equation::{Equation, RcPreEquation};
use crate::core::rewrite_context::RewritingContext;
use crate::theory::free_theory::{FreeSymbol, FreeTerm};
use crate::theory::free_theory::remainder::Speed;

use super::{FreeRemainder, RcFreeRemainder, FreeRemainderList};

pub type PatternSet = HashSet<i32, FastHasher>;
pub type RcFreeNet = WeakCell<FreeNet>;


struct Triple {
  symbol: RcSymbol,
  slot: i32,
  subtree: i32,
}

// region Ordering of Triples
impl Eq for Triple {}

impl PartialEq for Triple {
  fn eq(&self, other: &Self) -> bool {
    self.symbol.get_index_within_module() == other.symbol.get_index_within_module()
  }
}

impl PartialOrd for Triple {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.cmp(&other))
  }
}

impl Ord for Triple {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.symbol
        .get_index_within_module()
        .cmp(&other.symbol.get_index_within_module())
  }
}
// endregion

#[derive(Copy, Clone, Default)]
struct TestNode {
  /// Index of next test node to take for > and < cases (-ve encodes index of applicable list, 0 encodes failure)
  not_equal   : (i32, i32),
  /// Itack slot to get free dagnode argument list from (-1 indicates use old argument)
  position    : i32,
  /// Index of argument to test
  arg_index   : i32,
  /// Index within module of symbol we test against
  symbol_index: i32,
  /// Index of stack slot to store free dagnode argument list in (-1 indicates do not store)
  slot        : i32,
  /// Index of next test node to take for == case (-ve encode index of applicable list)
  equal       : i32,
}

#[derive(Default)]
pub struct FreeNet {
  stack          : Vec<NodeList>,
  net            : Vec<TestNode>,
  fast_applicable: Vec<FreeRemainderList>,
  remainders     : FreeRemainderList,
  applicable     : Vec<PatternSet>,
  fast           : bool,
}

impl FreeNet {
  pub fn new() -> Self {
    FreeNet {
      fast: true,
      ..FreeNet::default()
    }
  }

  pub fn fast_handling(&self) -> bool {
    self.fast
  }


  fn allocate_node(&mut self, nr_match_arcs: usize) -> usize {
    let len = self.net.len();
    self.net.resize(len + nr_match_arcs, TestNode::default());
    len
  }

  fn fill_out_node(
    &mut self,
    mut node_index: usize,
    position      : i32,
    arg_index     : i32,
    symbols       : &Vec<RcSymbol>,
    targets       : &Vec<i32>,
    slots         : &Vec<i32>,
    neq_target    : i32,
  ) {
    let symbol_count = symbols.len();
    let mut triples = Vec::with_capacity(symbol_count);

    for i in 0..symbol_count {
      triples.push(Triple {
        symbol: symbols[i].clone(),
        slot: slots[i],
        subtree: targets[i],
      });
    }

    triples.sort_by(|a, b| a.symbol.partial_cmp(&b.symbol).unwrap());
    self.build_ternary_tree(&mut node_index, &mut triples, 0, symbol_count - 1, neq_target, position, arg_index);
  }


  fn add_remainder_list(&mut self, live_set: PatternSet) -> i32 {
    let index = self.applicable.len();
    self.applicable.push(live_set);
    !(index as i32)
  }


  fn translate_slots(&mut self, nr_real_slots: usize, slot_translation: &Vec<i32>) {
    self.stack.resize(nr_real_slots, NodeList::new());

    for node in &mut self.net {
      node.slot     = if node.slot == NONE { NONE } else { slot_translation[node.slot as usize] };
      node.position = if node.position == NONE { NONE } else { slot_translation[node.position as usize] };
    }
  }


  fn build_remainders(
    &mut self,
    equations       : &Vec<RcPreEquation>,
    patterns_used   : &PatternSet,
    slot_translation: &Vec<i32>,
  ) {
    let nr_equations = equations.len();
    self.remainders.resize(nr_equations, None);

    for i in patterns_used {
      let e = equations[*i as usize].clone();

      if let Some(free_term) = e.borrow_mut().lhs_term.borrow_mut().as_any_mut().downcast_mut::<FreeTerm>() {
        let remainder = free_term.compile_remainder(e, slot_translation);
        self.remainders[*i as usize] = Some(remainder.clone());

        // If a remainder doesn't have fast handling, neither can the discrimination net.
        self.fast = (remainder.fast != Speed::Slow);
      } else {
        self.remainders[*i as usize] = Some(Rc::new(FreeRemainder::with_equation(e)));
        self.fast = false;  // A foreign equation always disables fast handling for the net
      }

    }
    // Build null terminated pointer version of applicable for added speed.
    let nr_applicables = self.applicable.len();
    self.fast_applicable.resize(nr_applicables, Vec::new());

    for i in 0..nr_applicables {
      let live_set = &self.applicable[i];
      let remainders = &mut self.fast_applicable[i];
      remainders.resize(live_set.len() + 1, None);

      for (j, rem) in live_set.iter().enumerate() {
        remainders[j] = self.remainders[*rem as usize].clone();
      }
    }
  }


  fn build_ternary_tree(
    &mut self,
    node_index: &mut usize,
    triples: &mut Vec<Triple>,
    first: usize,
    last: usize,
    default_subtree: i32,
    position: i32,
    arg_index: i32,
  ) {

    // Pick a middle element as the test symbol. If the sum of the first and last eligible indices
    // is odd we have a choice of middle elements and we try to break the tie in a smart way.
    let sum = first + last;
    let mut test_symbol = sum / 2;
    if sum & 1 != 0 && self.more_important(&triples[test_symbol + 1].symbol, &triples[test_symbol].symbol) {
      test_symbol += 1;
    }

    // Fill out a new node.
    let i = *node_index;
    *node_index += 1;
    self.net[i].position = position;
    self.net[i].arg_index = arg_index;
    self.net[i].symbol_index = triples[test_symbol].symbol.index_within_parent();
    self.net[i].slot = triples[test_symbol].slot;
    self.net[i].equal = triples[test_symbol].subtree;

    // If there are any symbols remaining to the left of the test symbol, build a subtree for them.
    if first < test_symbol {
      self.net[i].not_equal.0 = *node_index as i32;
      self.build_ternary_tree(node_index, triples, first, test_symbol - 1, default_subtree, NONE, NONE);
    } else {
      self.net[i].not_equal.0 = default_subtree;
    }

    // If there are any symbols remaining to the right of the test symbol, build a subtree for them.
    if last > test_symbol {
      self.net[i].not_equal.1 = *node_index as i32;
      self.build_ternary_tree(node_index, triples, test_symbol + 1, last, default_subtree, NONE, NONE);
    } else {
      self.net[i].not_equal.1 = default_subtree;
    }
  }


  /// Heuristic to decide which symbol is more important and thus should have the fastest matching.
  /// Returns true if first symbol is considered more important.
  ///
  /// The current heuristic favors free symbols over non-free symbols and high arity symbols over
  /// low arity symbols.
  fn more_important(&self, first: &RcSymbol, second: &RcSymbol) -> bool {
    let f = first.as_any().downcast_ref::<FreeSymbol>();
    let s = second.as_any().downcast_ref::<FreeSymbol>();

    match (f, s) {
      (Some(_), None) => true,
      (None, Some(_)) => false,
      _ => first.arity() > second.arity(),
    }
  }

  /// This is the inlined gaurd for `apply_replace` that provides a fast path in the case that
  /// the term cannot be applied.
  #[inline(always)]
  pub(crate) fn apply_replace(&self, subject: RcDagNode, context: &mut RewritingContext) -> bool {
    if !self.applicable.is_empty() {
      self.apply_replace_aux(subject, context)
    } else {
      false
    }
  }

  pub fn apply_replace_aux(&self, subject: RcDagNode, context: &mut RewritingContext) -> bool {

    false
  }


}
