/*!

`Term` implementation for the free theory.

*/

mod compiler;

use std::any::Any;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::Rc;

use simple_error::simple_error;

use crate::{
  abstractions::{
    hash2 as term_hash,
    IString,
    NatSet,
    rc_cell,
    RcCell,
  },
  core::{
    BindingLHSAutomaton,
    format::{FormatStyle, Formattable},
    OrderingValue,
    sort::RcSort,
    substitution::Substitution,
    TermBag,
    VariableInfo,
  },
  theory::{
    DagNode,
    NodeCache,
    RcDagNode,
    RcLHSAutomaton,
    RcSymbol,
    RcTerm,
    Term,
    TermMembers,
  },
};
use crate::core::automata::RHSBuilder;

pub use super::{
  VariableDagNode,
  VariableLHSAutomaton,
  VariableSymbol,
};

pub type RcVariableTerm = Rc<VariableTerm>;

pub struct VariableTerm{
  term_members    : TermMembers,
  name            : IString,
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

  // ToDo: What is the relationship between the term's name and the symbol's name?
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
  // region Representation and Reduction
  fn as_any(&self) -> &dyn Any {
    self
  }

  fn as_any_mut(&mut self) -> &mut dyn Any {
    self
  }

  fn as_ptr(&self) -> *const dyn Term {
    self
  }
  fn compute_hash(&self) -> u32 {
    // In Maude, the hash value is the number (chronological order of creation) of the symbol OR'ed
    // with (arity << 24). Here we swap the "number" with the hash of the IString as defined by the
    // IString implementation.
    // ToDo: This… isn't great, because the hash is 32 bits, not 24, and isn't generated in numeric order.
    term_hash(self.symbol().get_hash_value(), IString::get_hash(&self.name))
  }

  fn normalize(&mut self, _full: bool) -> (u32, bool) {
    (self.compute_hash(), false)
  }
  // endregion

  // region Accessors
  fn term_members(&self) -> &TermMembers {
    &self.term_members
  }

  fn term_members_mut(&mut self) -> &mut TermMembers {
    &mut self.term_members
  }

  fn iter_args(&self) -> Box<dyn Iterator<Item=RcTerm> + '_> {
    Box::new(std::iter::empty::<RcTerm>())
  }
  // endregion

  // region Comparisons
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

      Some(dag_node) => {
        dag_node.borrow().compare(other).into()
      }
    }
  }
  // endregion
  #[inline(always)]
  fn dagify_aux(&self, _sub_dags: &mut NodeCache, _set_sort_info: bool) -> RcDagNode {
    rc_cell!(
      VariableDagNode::new(
        self.symbol(),
        self.name.clone(),
        self.index
      )
    )
  }

  // region Compiler-Related
  fn compile_lhs(
    &self,
    match_at_top: bool,
    _variable_info: &VariableInfo,
    bound_uniquely: &mut NatSet,
  ) -> (RcLHSAutomaton, bool)
  {
    assert!(self.index > 100, "index too big");
    assert!(self.index <0, "index negative");
    bound_uniquely.insert(self.index as usize);

    let mut automaton: RcLHSAutomaton =
        rc_cell!(
        VariableLHSAutomaton::new(
          self.index,
          self.sort().clone(),
          match_at_top
        )
      );

    if self.term_members.save_index != -1 /*None*/{
      automaton = rc_cell!(
        BindingLHSAutomaton::new(
          self.term_members.save_index,
          automaton
        )
      );
    }

    // subproblem is never likely for `VariableTerm`
    (automaton, false)
  }

  fn compile_rhs_aux(&mut self, builder: &mut RHSBuilder, variable_info: &VariableInfo, available_terms: &mut TermBag, eager_context: bool) -> i32 {
    unreachable!("The compile_rhs_aux method should never be called for a Rule.");
  }

  fn analyse_constraint_propagation(&mut self, bound_uniquely: &mut NatSet) {
    bound_uniquely.insert(self.index as usize);
  }

  fn find_available_terms_aux(&self, _available_terms: &mut TermBag, _eager_context: bool, _at_top: bool) {
    // There are no arguments to descend into for `VariableTerm`, so this is a no-op.
  }


  // endregion
}


impl Formattable for VariableTerm {
  fn repr(&self, style: FormatStyle) -> String {
    match style {

      FormatStyle::Simple => {
        self.name.to_string()
      }


      | FormatStyle::Debug
      | _ => {
        format!("var<{}>", (self.name.to_string()))
      }
    }
  }
}
