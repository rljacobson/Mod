/*!

`Term` implementation for the free theory.

The definition of `FreeTerm` and its implementation of the `Term` trait live in this file. The
(non-trait) `impl` for `FreeTerm` is spread over multiple files to keep the file size small enough
to navigate. In particular, the compiler for the matcher is in `compiler.rs`.

*/


mod compiler;

use std::{
  cmp::Ordering,
  any::Any,
  rc::Rc,
  cell::RefCell,
  fmt::Display,
};

use crate::{
  abstractions::{
    hash2 as term_hash,
    NatSet,
    RcCell,
  },
  core::{
    format::{FormatStyle, Formattable},
    OrderingValue,
    substitution::Substitution,
    TermBag,
    VariableInfo,
  },
  rc_cell,
  theory::{
    DagNode,
    NodeCache,
    RcDagNode,
    RcLHSAutomaton,
    RcSymbol,
    RcTerm,
    Term,
    TermFlags,
    TermMembers,
  },
};

use super::{FreeSymbol, FreeOccurrence, FreeDagNode};

pub type RcFreeTerm = RcCell<FreeTerm>;

pub struct FreeTerm {
  pub(crate) term_members: TermMembers,
  pub(crate) args        : Vec<RcTerm>,
  pub(crate) slot_index  : u32,
  pub(crate) visited     : bool
}

// Constructors
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
  //region Representation and Reduction Methods
  fn as_any(&self) -> &dyn Any {
    self
  }

  fn as_any_mut(&mut self) -> &mut dyn Any {
    self
  }

  fn as_ptr(&self) -> *const dyn Term {
    self
  }

  /// In sync with `normalize`.
  fn compute_hash(&self) -> u32 {
    let mut hash_value: u32 = self.symbol().get_hash_value();

    for arg in &self.args {
      hash_value = term_hash(
        hash_value,
        arg.borrow().compute_hash()
      );
    }

    hash_value
  }

  fn normalize(&mut self, full: bool) -> (u32, bool) {
    let mut changed: bool = false;
    let mut hash_value: u32 = self.symbol().get_hash_value();

    for arg in &self.args {
      let (child_hash, child_changed): (u32, bool) = arg.borrow_mut().normalize(full);

      changed = changed || child_changed;
      hash_value = term_hash(
        hash_value,
        child_hash
      );
    }

    (hash_value, changed)
  }

  // endregion

  // region Accessors
  #[inline(always)]
  fn term_members(&self) -> &TermMembers {
    &self.term_members
  }

  #[inline(always)]
  fn term_members_mut(&mut self) -> &mut TermMembers {
    &mut self.term_members
  }

  #[inline(always)]
  fn iter_args(&self) -> Box<dyn Iterator<Item=RcTerm> + '_>{
    Box::new(self.args.iter().cloned())
  }
  // endregion

  // region Comparison Methods

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

  // endregion

  fn dagify_aux(&self, sub_dags: &mut NodeCache, set_sort_info: bool) -> RcDagNode {
    let mut node = FreeDagNode::new(self.symbol());

    for arg in &self.args {
      node.members.args.push(arg.borrow_mut().dagify(sub_dags, set_sort_info));
    }
    // Needed to specify generic trait object.
    let node: RcCell<dyn DagNode> = rc_cell!(node);
    node
  }

  #[inline(always)]
  fn compile_lhs(
    &self,
    match_at_top     : bool,
    variable_info    : &VariableInfo,
    bound_uniquely   : &mut NatSet,
  ) -> (RcLHSAutomaton, bool)
  {
    FreeTerm::compile_lhs(self, match_at_top, variable_info, bound_uniquely)
  }

  #[inline(always)]
  fn analyse_constraint_propagation(&mut self, bound_uniquely: &mut NatSet) {
    FreeTerm::analyse_constraint_propagation(self, bound_uniquely)
  }

  #[inline(always)]
  fn find_available_terms_aux(&self, available_terms: &mut TermBag, eager_context: bool, at_top: bool) {
    FreeTerm::find_available_terms_aux(&self, available_terms, eager_context, at_top);
  }

}


impl Formattable for FreeTerm {
  fn repr(&self, style: FormatStyle) -> String {

    let mut accumulator = String::new();
    match style {

      FormatStyle::Simple => {
        accumulator.push_str(
          self.term_members.top_symbol.repr(style).as_str()
        );
      }

      | FormatStyle::Debug
      | _ => {
        accumulator.push_str(
          format!(
            "free<{}>",
            self.term_members.top_symbol.repr(style).as_str()
          ).as_str()
        );
      }
    }

    accumulator.push_str(
      format!(
        "free<{}>",
        self.term_members.top_symbol.repr(style).as_str()
      ).as_str()
    );
    if !self.args.is_empty() {
      accumulator.push('(');
      accumulator.push_str(
        self.args
            .iter()
            .map(|arg| arg.borrow().repr(style))
            .collect::<Vec<String>>()
            .join(", ")
            .as_str()
      );
      accumulator.push(')');
    }

    accumulator
  }
}
