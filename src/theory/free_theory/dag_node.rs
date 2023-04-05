use std::{
  any::Any,
  rc::Rc, cmp::Ordering
};

use crate::{
  theory::{
    DagNodeFlags,
    DagNode,
    RcDagNode,
    DagPair,
    RcSymbol,
    Symbol,
  },
  core::{
    RcSort,
    OrderingValue,
    numeric_ordering
  }
};

use super::RcFreeSymbol;

pub type RcFreeDagNode = Rc<FreeDagNode>;

pub struct FreeDagNode {
  pub(crate) top_symbol: RcFreeSymbol,
  pub(crate) args      : Vec<DagPair>,
  pub(crate) sort      : RcSort,
  pub(crate) flags      : DagNodeFlags,
  pub(crate) sort_index: i32,
}

impl DagNode for FreeDagNode {

  fn iter_args(&self) -> Box<dyn Iterator<Item=(RcDagNode, u32)> + '_> {
    Box::new(self.args.iter().map(|pair| (pair.dag_node.clone(), pair.multiplicity)))
  }

  fn symbol(&self) -> RcSymbol {
    self.top_symbol.clone()
  }

  // Todo: Is this needed?
  fn symbol_mut(&mut self) -> &mut dyn Symbol {
    Rc::get_mut(&mut self.top_symbol).unwrap()   //.borrow_mut()
  }

  fn compare_arguments(&self, other: &dyn DagNode) -> std::cmp::Ordering {
    match other.as_any().downcast_ref::<FreeDagNode>() {
      Some(free_dag_node) => {
        // Fail fast if lengths differ.
        let r: i32 = self.args.len() as i32 - free_dag_node.len() as i32;
        if r != 0 {
          return numeric_ordering(r as isize);
        }
        // Compare corresponding terms.
        for ((this_child, this_multiplicity), (other_child, other_multiplicity))
        in self.iter_args().zip(free_dag_node.iter_args()) {
          let r: i32 = this_multiplicity as i32 - other_multiplicity as i32;
          if r != 0 {
            return numeric_ordering(r as isize);
          }

          let r = this_child.as_ref().compare(other_child.as_ref());
          if r != Ordering::Equal {
            return r;
          }
        }
        // Identical
        return Ordering::Equal;
      }
      None => panic!("Could not downcast a Term to a FreeTerm. This is a bug."),
    };
  }

  fn get_sort(&self) -> RcSort {
    self.sort.clone()
  }

  fn set_sort_index(&mut self, sort_index: i32) {
    self.sort_index = sort_index;
  }

  fn get_sort_index(&self) -> i32 {
    self.sort_index
  }

  fn len(&self) -> usize {
    self.args.len()
  }

  fn as_any(&self) -> &dyn Any{
    self
  }

  fn as_any_mut(&mut self) -> &mut dyn Any{
    self
  }

  fn compute_base_sort(&self) -> i32 {
    todo!()
  }

  fn flags(&self) -> DagNodeFlags {
    self.flags
  }
}
