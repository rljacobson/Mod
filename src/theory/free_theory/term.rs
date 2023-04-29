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
  abstractions::RcCell
};
use crate::theory::RcDagNode;
use crate::theory::term::NodeCache;
use super::FreeDagNode;


pub type RcFreeTerm = RcCell<FreeTerm>;

pub struct FreeTerm{
  pub(crate) term_members: TermMembers,
  pub(crate) args        : Vec<RcTerm>,
  pub(crate) slot_index  : u32,
  pub(crate) visited     : bool
}

impl Term for FreeTerm {
  fn term_members(&self) -> &TermMembers {
    &self.term_members
  }

  fn term_members_mut(&mut self) -> &mut TermMembers {
    &mut self.term_members
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
}
