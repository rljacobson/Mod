#![allow(unused_imports)]

pub(crate) mod automata;
mod cached_dag;
mod local_bindings;
mod narrowing_variable_info;
mod ordering_value;
mod redex_position;
mod strategy;
mod term_bag;
mod variable_info;

pub mod condition_fragment;
pub mod hash_cons_set;
pub mod interpreter;
pub mod pre_equation;
pub mod sort;
pub mod substitution;
// mod strategy_definition; // Unimplemented

// Re-export most important modules from `interpreter` to save a few keystrokes
// Flatten single-item modules
pub(crate) use automata::BindingLHSAutomaton;
pub(crate) use cached_dag::CachedDag;
pub(crate) use interpreter::{format, module, rewrite_context};
pub(crate) use local_bindings::{Binding, LocalBindings};
pub(crate) use narrowing_variable_info::NarrowingVariableInfo;
pub(crate) use ordering_value::{numeric_ordering, numeric_ordering_value, OrderingValue};
pub(crate) use redex_position::RedexPosition;
pub(crate) use strategy::Strategy;
pub(crate) use term_bag::TermBag;
pub(crate) use variable_info::VariableInfo;

// NOT YET IMPLEMENTED
pub struct CacheableState {}
pub struct StateTransitionGraph {}
pub struct Token {}

// Won't Implement
pub struct SyntacticPreModule {}
pub struct SyntacticView {}
pub struct VisibleModule {}
