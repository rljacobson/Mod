#![feature(new_uninit)]
#![allow(dead_code)]
#![allow(non_snake_case)]
pub mod core;
pub mod theory;





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
