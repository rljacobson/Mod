#![feature(new_uninit)]
#![feature(strict_provenance)]
#![allow(dead_code)]
#![allow(non_snake_case)]
pub mod core;
pub mod theory;
pub mod abstractions;
mod parser;

// use Pratt::Parser;


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        // let result = add(2, 2);
        // assert_eq!(result, 4);
    }
}
