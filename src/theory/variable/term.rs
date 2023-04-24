use std::any::Any;
use std::cmp::Ordering;
use std::rc::Rc;
use crate::abstractions::IString;
use crate::core::RcSort;
use crate::theory::{DagNode, Term, TermMembers};
use crate::theory::variable::symbol::VariableSymbol;
use crate::theory::variable::VariableDagNode;


pub type RcVariableTerm = Rc<VariableTerm>;

pub struct VariableTerm{
  term_members: TermMembers,
  name: IString,
  pub(crate) index: u32
}

impl VariableTerm {
  pub fn sort(&self) -> RcSort {
    if let Some(v) = self.term_members.top_symbol.as_any().downcast_ref::<VariableSymbol>(){
      v.sort()
    } else {
      unreachable!("Downcast to VariableSymbol failed. This is a bug.");
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

  fn as_any(&self) -> &dyn Any {
    self
  }
}
