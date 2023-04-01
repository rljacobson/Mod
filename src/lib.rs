#![feature(new_uninit)]
#![allow(dead_code)]
#![allow(non_snake_case)]

extern crate core;

mod theory;
mod rewrite_context;
mod sort;
mod substitution;
mod local_bindings;
mod redex_position;
mod ordering_value;
mod cached_dag;
mod sort_constraint;

pub use sort::Sort;
pub use rewrite_context::RewritingContext;
pub use substitution::Substitution;



pub(crate) use ordering_value::OrderingValue;
pub(crate) use bit_set::BitSet as NatSet;



pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
