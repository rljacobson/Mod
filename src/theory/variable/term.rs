use std::any::Any;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::Rc;
use crate::abstractions::{IString, RcCell};
use crate::core::{OrderingValue, RcSort, Substitution};
use crate::theory::{DagNode, RcDagNode, RcSymbol, RcTerm, Term, TermMembers};
use crate::theory::term::NodeCache;
use crate::theory::variable::symbol::VariableSymbol;
use crate::theory::variable::VariableDagNode;


pub type RcVariableTerm = Rc<VariableTerm>;

pub struct VariableTerm{
  term_members: TermMembers,
  name: IString,
  pub(crate) index: i32
}

impl VariableTerm {
  pub fn sort(&self) -> RcSort {
    if let Some(v) = self.term_members.top_symbol.as_any().downcast_ref::<VariableSymbol>(){
      v.sort()
    } else {
      unreachable!("Downcast to VariableSymbol failed. This is a bug.");
    }
  }

  pub fn new(name: IString, symbol: RcSymbol) -> VariableTerm {
    let term_members = TermMembers::new(symbol);

    VariableTerm{
      term_members,
      name,
      index: -1 // What should this be?
    }
  }
}

impl Term for VariableTerm {
  fn term_members(&self) -> &TermMembers {
    &self.term_members
  }

  fn term_members_mut(&mut self) -> &mut TermMembers {
    &mut self.term_members
  }

  fn compare_term_arguments(&self, other: &dyn Term) -> Ordering {
    if let Some(other) = other.as_any().downcast_ref::<VariableTerm>(){
      self.name.cmp(&other.name)
    } else {
      Ordering::Less
    }
  }

  fn compare_dag_arguments(&self, other: &dyn DagNode) -> Ordering {
    if let Some(other) = other.as_any().downcast_ref::<VariableDagNode>(){
      self.name.cmp(&other.name)
    } else {
      Ordering::Less
    }
  }

  fn partial_compare_unstable(&self, partial_substitution: &mut Substitution, other: &dyn DagNode) -> OrderingValue {
    match partial_substitution.get(self.index) {

      None => {
        return OrderingValue::Unknown;
      }

      Some(d) => {
        d.borrow().compare(other).into()
      }
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

  fn dagify_aux(&self, _sub_dags: &mut NodeCache, _set_sort_info: bool) -> RcDagNode {
    RcCell(
      Rc::new(
        RefCell::new(
          VariableDagNode::new(
            self.symbol(),
            self.name.clone(),
            self.index
          )
        )
      )
    )
  }
}
