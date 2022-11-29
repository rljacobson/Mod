extern crate core;

mod theory;
mod rewrite_context;
mod sort;
mod substitution;

pub use sort::Sort;
pub use rewrite_context::RewritingContext;
pub use substitution::Substitution;




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
