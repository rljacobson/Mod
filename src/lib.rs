#![feature(new_uninit)]
#![feature(strict_provenance)]
#![allow(dead_code)]
#![allow(non_snake_case)]
pub mod core;
pub mod theory;
pub mod abstractions;
pub mod parser;

// use Pratt::Parser;


#[cfg(test)]
mod tests {
    use parser::Parser;
    use super::*;


    #[test]
    fn simple_match_expr_test() {
        // set_verbosity(5);

        let mut parser = Parser::new();

        let mut pattern = "f(α, β)";
        let mut pattern_term = match parser.parse(pattern) {
            Ok(term) => term,
            Err(_err) => {
                panic!("FAILED TO PARSE.");
            }
        };

        let mut subject = "f(a, b)";
        let mut subject_term = match parser.parse(subject) {
            Ok(term) => term,
            Err(_err) => {
                panic!("FAILED TO PARSE.");
            }
        };

        println!("PATTERN: {}", pattern);
        println!("TERM: {}", pattern_term.borrow());
        println!("SUBJECT: {}", subject);
        println!("TERM: {}", subject_term.borrow());

        let mut pattern_dag = pattern_term.borrow_mut().make_dag();
        // pattern_dag.borrow_mut().


    }
}
