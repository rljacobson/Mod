#![allow(unused_imports)]

mod automata;
mod cached_dag;
mod local_bindings;
mod ordering_value;
mod redex_position;
mod strategy;
mod term_bag;
mod variable_info;

pub mod substitution;
pub mod condition_fragment;
pub mod interpreter;
pub mod pre_equation;
pub mod sort;
// mod strategy_definition; // Unimplemented

// Re-export most important modules from `interpreter` to save a few keystrokes
pub(crate) use interpreter::{
  module,
  rewrite_context,
  format
};

// Flatten single-item modules
pub(crate) use automata::BindingLHSAutomaton;
pub(crate) use cached_dag::CachedDag;
pub(crate) use equation::{Equation, RcEquation};
pub(crate) use local_bindings::{Binding, LocalBindings};
pub(crate) use ordering_value::{numeric_ordering, numeric_ordering_value, OrderingValue};
pub(crate) use redex_position::RedexPosition;
pub(crate) use strategy::Strategy;
// pub(crate) use substitution::{Substitution, print_substitution, print_substitution_dag, print_substitution_with_ignored};
pub(crate) use term_bag::TermBag;
pub(crate) use variable_info::VariableInfo;
pub(crate) use rule::Rule;

// NOT YET IMPLEMENTED
pub struct CacheableState {}
pub struct NarrowingVariableInfo{} // This is not a subclass of `VariableInfo`, though it does have `index2variable`
#[derive(Default)]
pub struct RHSBuilder{}
pub struct StateTransitionGraph{}
pub struct StrategyDefinition{}
pub struct Token {}

// Won't Implement
pub struct SyntacticPreModule {}
pub struct SyntacticView {}
pub struct VisibleModule {}
