/*!

Term for free theory.

*/


use std::{
  cmp::Ordering,
  any::Any,
  rc::Rc
};
use std::cell::RefCell;


use crate::{
  theory::{
    TermMembers,
    Term,
    RcTerm,
    TermFlags,
    DagNode
  },
  abstractions::{RcCell, hash2 as term_hash}
};
use crate::core::{OrderingValue, Substitution};
use crate::theory::free_theory::FreeSymbol;
use crate::theory::{RcDagNode, RcSymbol};
use crate::theory::term::NodeCache;
use super::FreeDagNode;


pub type RcFreeTerm = RcCell<FreeTerm>;

pub struct FreeTerm{
  pub(crate) term_members: TermMembers,
  pub(crate) args        : Vec<RcTerm>,
  pub(crate) slot_index  : u32,
  pub(crate) visited     : bool
}

impl FreeTerm {
  pub fn new(symbol: RcSymbol) -> FreeTerm {
    let term_members = TermMembers::new(symbol);
    FreeTerm{
      term_members,
      args: vec![],
      slot_index: 0,
      visited: false,
    }
  }

  pub fn with_args(symbol: RcSymbol, args: Vec<RcTerm>) -> FreeTerm {
    let term_members = TermMembers::new(symbol);
    FreeTerm{
      term_members,
      args,
      slot_index: 0,
      visited: false,
    }
  }
}

impl Term for FreeTerm {
  fn term_members(&self) -> &TermMembers {
    &self.term_members
  }

  fn term_members_mut(&mut self) -> &mut TermMembers {
    &mut self.term_members
  }

  // ToDo: This method makes no use of partial_substitution except for `partial_compare_unstable` in `VariableTerm`.
  fn partial_compare_arguments(&self, partial_substitution: &mut Substitution, other: &dyn DagNode) -> OrderingValue {
    assert!(self.symbol().compare(other.symbol().as_ref()).is_eq(), "symbols differ");

    if let Some(da) = other.as_any().downcast_ref::<FreeDagNode>(){
      for (term_arg, dag_arg) in self.args.iter().zip(da.iter_args()) {
        let r = term_arg.borrow()
                        .partial_compare(partial_substitution, dag_arg.as_ref());
        if r != OrderingValue::Equal {
          return r;
        }
      }
      return OrderingValue::Equal
    }
    else {
      unreachable!("{}:{}: Could not downcast to FreeDagNode. This is a bug.", file!(), line!())
    }
  }

  fn compare_term_arguments(&self, other: &dyn Term) -> Ordering {
    assert_eq!(&self.symbol(), &other.symbol(), "symbols differ");

    if let Some(other) = other.as_any().downcast_ref::<FreeTerm>() {

      for (arg_self, arg_other) in self.args.iter().zip(other.args.iter()){
        let r = arg_self.borrow().compare(arg_other.as_ref());
        if r.is_ne() {
          return r
        }
      }
      return Ordering::Equal;

    } else {
      unreachable!("Could not downcast Term to FreeTerm. This is a bug.")
    }
  }

  fn compare_dag_arguments(&self, other: &dyn DagNode) -> Ordering {
    // assert_eq!(self.symbol(), other.symbol(), "symbols differ");
    if let Some(other) = other.as_any().downcast_ref::<FreeDagNode>() {

      for (arg_self, arg_other) in self.args.iter().zip(other.iter_args()){
        let r = arg_self.borrow().compare_dag_node(arg_other.as_ref());
        if r.is_ne() {
          return r
        }
      }
      return Ordering::Equal;

    } else {
      unreachable!("Could not downcast Term to FreeTerm. This is a bug.")
    }
  }

  fn as_any(&self) -> &dyn Any {
    self
  }

  fn as_any_mut(&mut self) -> &mut dyn Any {
    self
  }

  fn as_ptr(&self) -> *const dyn Term {
    self
  }

  fn dagify_aux(&self, sub_dags: &mut NodeCache, set_sort_info: bool) -> RcDagNode {
    let node = Rc::new(RefCell::new(FreeDagNode::new(self.symbol())));

    {
      let mut node = node.borrow_mut();
      for arg in &self.args {
        node.members.args.push(arg.borrow_mut().dagify(sub_dags, set_sort_info));
      }
    }

    RcCell(node)
  }


  fn repr(&self) -> String {
    let mut accumulator = String::new();

    accumulator.push_str(
      format!(
        "free<{}>",
        self.term_members.top_symbol.to_string().as_str()
      ).as_str()
    );
    if !self.args.is_empty() {
      accumulator.push('(');
      accumulator.push_str(
        self.args
            .iter()
            .map(|arg| arg.borrow().repr())
            .collect::<Vec<String>>()
            .join(", ")
            .as_str()
      );
      accumulator.push(')');
    }

    accumulator
  }

  /// In sync with `normalize`.
  fn compute_hash(&self) -> u32 {
    let mut hash_value: u32 = self.symbol().get_hash_value();

    for arg in &self.args {
      let mut child_hash = 0;

      child_hash = arg.borrow().compute_hash();

      hash_value = term_hash(
        hash_value,
        child_hash
      );
    }

    hash_value
  }

  fn normalize(&mut self, full: bool) -> (u32, bool) {
    let mut changed: bool = false;
    let mut hash_value: u32 = self.symbol().get_hash_value();

    for arg in &self.args {
      let mut child_hash = 0;
      let mut child_changed = false;

      (child_hash, child_changed) = arg.borrow_mut().normalize(full);
      changed = changed || child_changed;

      hash_value = term_hash(
        hash_value,
        child_hash
      );
    }

    (hash_value, changed)
  }

}
