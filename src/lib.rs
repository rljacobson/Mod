#![feature(new_uninit)]
#![feature(strict_provenance)]
#![feature(assert_matches)]
#![allow(dead_code)]
#![allow(non_snake_case)]
pub mod abstractions;
pub mod core;
pub mod parser;
pub mod theory;

// use Pratt::Parser;

// Sentinel Values
// ToDo: Do UNDEFINED the right way. Is this great? No. But it's convenient.
const UNDEFINED: i32 = -1;
const NONE: i32 = -1;
const ROOT_OK: i32 = -2;

#[cfg(test)]
mod tests {
  use parser::Parser;

  use super::*;
  use crate::{abstractions::NatSet, core::VariableInfo, theory::RcLHSAutomaton};


  #[test]
  fn simple_match_expr_test() {
    // set_verbosity(5);

    let mut parser = Parser::new();

    let pattern = "f(α, β)";
    let pattern_term = match parser.parse(pattern) {
      Ok(term) => term,
      Err(_err) => {
        panic!("FAILED TO PARSE.");
      }
    };

    let subject = "f(a, b)";
    let subject_term = match parser.parse(subject) {
      Ok(term) => term,
      Err(_err) => {
        panic!("FAILED TO PARSE.");
      }
    };

    println!("PATTERN: {}", pattern);
    println!("TERM: {}", pattern_term.borrow());
    println!("SUBJECT: {}", subject);
    println!("TERM: {}", subject_term.borrow());

    let (_pattern_automata, _subproblem_likely): (RcLHSAutomaton, bool) =
      pattern_term
        .borrow_mut()
        .compile_lhs(true, &VariableInfo::default(), &mut NatSet::default());
  }
}
