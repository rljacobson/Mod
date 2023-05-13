use std::{
  any::Any,
  rc::Rc, cmp::Ordering
};
use std::cell::RefCell;

use crate::{theory::{
  dag_node_flags,
  DagNode,
  RcDagNode,
  DagPair,
  RcSymbol,
  Symbol,
}, core::{
  RcSort,
  OrderingValue,
  numeric_ordering
}, abstractions::RcCell, rc_cell};
use crate::core::{RedexPosition, Sort, SpecialSort};
use crate::theory::dag_node::DagNodeMembers;
use crate::theory::free_theory::FreeTerm;
use crate::theory::{DagNodeFlag, DagNodeFlags, NodeList, RcTerm};

use super::RcFreeSymbol;

pub type RcFreeDagNode = Rc<FreeDagNode>;

pub struct FreeDagNode {
  // Base DagNode Members
  pub members: DagNodeMembers
}

impl FreeDagNode {
  pub fn new(symbol: RcSymbol) -> FreeDagNode {
    let members = DagNodeMembers{
      top_symbol: symbol,
      args: Default::default(),
      // sort: Default::default(),
      flags: Default::default(),
      sort_index: 0,
    };

    FreeDagNode{
      members,
    }
  }
}

impl DagNode for FreeDagNode {
  #[inline(always)]
  fn dag_node_members(&self) -> &DagNodeMembers {
    &self.members
  }

  #[inline(always)]
  fn dag_node_members_mut(&mut self) -> &mut DagNodeMembers {
    &mut self.members
  }

  #[inline(always)]
  fn as_any(&self) -> &dyn Any {
    self
  }

  #[inline(always)]
  fn as_any_mut(&mut self) -> &mut dyn Any {
    self
  }

  #[inline(always)]
  fn as_ptr(&self) -> *const dyn DagNode {
    self
  }

  fn compare_arguments(&self, other: &dyn DagNode) -> Ordering {
    assert!(self.symbol() == other.symbol(), "symbols differ");

    let s = self.symbol();
    let nr_args = s.arity() as usize;

    if nr_args != 0 {
      let mut pd = self;
      let mut qd = match other.as_any().downcast_ref::<FreeDagNode>() {
        Some(v) => v,
        None => {
          // Not even the same theory. It's not clear what to return in this case, so just compare symbols.
          return s.compare(other.symbol().as_ref());
        }
      };

      loop {

        let p = &pd.dag_node_members().args;
        let q = &qd.dag_node_members().args;

        // Compare all but the last (rightmost) node.
        for i in (0..nr_args - 1).rev() {
          let r = DagNode::compare(p[i].as_ref(), q[i].as_ref());
          if r.is_ne() {
            return r;
          }
        }

        let pd2 = p[nr_args - 1].as_ref();
        let qd2 = q[nr_args - 1].as_ref();
        // Fast bail on equal pointers.
        if std::ptr::addr_of!(pd2) == std::ptr::addr_of!(qd2) {
          return Ordering::Equal; // Points to same node
        }
        // Compare symbols
        let ps = pd2.symbol();
        let qs = qd2.symbol();
        let r = ps.compare(qs.as_ref());
        if r.is_ne() {
          return r  // different symbols
        }
        // Go ahead and check all arguments
        if *ps != *s {
          return pd2.compare_arguments(qd2);
        }

        // Next iteration will compare argument lists using tail recursion elimination. See https://imgur.com/a/12dtE3j.
        pd = pd2.as_any().downcast_ref::<FreeDagNode>().unwrap();
        qd = qd2.as_any().downcast_ref::<FreeDagNode>().unwrap();
      }
    }
    // Survived all attempts at finding inequality.
    return Ordering::Equal;
  }

  fn compute_base_sort(&mut self) -> i32{

    let symbol = self.symbol();
    // assert_eq!(self as *const _, subject.symbol() as *const _, "bad symbol");
    let nr_args = symbol.arity();
    if nr_args == 0 {
      let idx = symbol.sort_table().traverse(0, 0);
      self.set_sort_index(idx); // Maude: HACK
      return idx;
    }

    let mut state = 0;
    // enumerate is only used for assertion
    for (i, arg) in self.iter_args().enumerate() {
      let t = arg.borrow().get_sort_index();
      assert_ne!(
        t,
        SpecialSort::Unknown as i32,
        "unknown sort encounter for arg {} subject = {}",
        i,
        self as &dyn DagNode
      );
      state = symbol.sort_table().traverse(state as usize, t as usize);
    }
    self.set_sort_index(state);
    state
  }

  fn termify(&self) -> RcTerm {
    let args: Vec<RcTerm>
        = self.members
              .args
              .iter()
              .map(
                |d|{
                  d.borrow().termify()
                }
              )
              .collect();
    rc_cell!(FreeTerm::with_args(self.symbol(), args))
  }

  fn shallow_copy(&self) -> RcDagNode {
    let fdg = FreeDagNode{
      members: DagNodeMembers{
        top_symbol: self.symbol(),
        args      : self.members.args.clone_buffer(), // Clones all elements of the `SharedVec`
        flags     : self.flags() & DagNodeFlags::RewritingFlags,
        sort_index: self.get_sort_index(),
      }
    };

    rc_cell!(fdg)
  }


  fn copy_with_replacements(&self, redex_stack: &[RedexPosition], mut first_idx: usize, last_idx: usize) -> RcDagNode {
    assert!(first_idx <= last_idx && last_idx < redex_stack.len(), "bad replacement range");
    let symbol = self.symbol();
    let nr_args = symbol.arity() as i32;
    assert!(
      redex_stack[first_idx].arg_index < nr_args && redex_stack[last_idx].arg_index < nr_args,
      "bad replacement arg index"
    );

    let mut new_dag_node = FreeDagNode::new(symbol);
    let args = &self.members.args;
    let new_args = &mut new_dag_node.members.args;
    let mut next_replacement_index = redex_stack[first_idx].arg_index;

    for i in 0..nr_args {
      if i == next_replacement_index {
        new_args.push(redex_stack[first_idx].dag_node.clone());
        first_idx += 1;
        next_replacement_index = if first_idx <= last_idx {
          redex_stack[first_idx].arg_index
        } else {
          -1
        };
      } else {
        new_args.push(args[i as usize].clone());
      }
    }
    rc_cell!(new_dag_node)
  }

  fn copy_with_replacement(&self, replacement: RcDagNode, arg_index: usize) -> RcDagNode{
    let symbol = self.symbol();
    let nr_args = symbol.arity() as usize;
    assert!(arg_index < nr_args, "bad argIndex");

    let mut new_dag_node = FreeDagNode::new(symbol);
    let new_args = &mut new_dag_node.members.args;
    // Because args is COW, we could just clone. But this is safer, in case someone forgets and changes args to non-COW.
    *new_args = self.members.args.clone_buffer();

    new_args[arg_index] = replacement;

    rc_cell!(new_dag_node)
  }
}
