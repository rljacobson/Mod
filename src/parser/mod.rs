/*

Provides the parser for the expression language. The expression language is defined by this grammar:

    expr := symbol | variable | application ;
    symbol : = [a-z][a-zA-Z]* ;
    variable := [A-Z][a-zA-Z]* ;
    application := '(' (expr_list | ε) ')' ;

    expr_list := expr ',' expr_list | expr ;

The parser itself is implemented in the Pratt library. This module transforms the expression tree from the Pratt
parser into a tree of `Term`s.

*/

use std::cell::RefCell;
use std::rc::Rc;
use std::error::Error;

use simple_error::simple_error;
use unicode_blocks;

use pratt::{Atom, Parser as ParserCore};
use crate::abstractions::{IString, RcCell};
use crate::core::Strategy;
use crate::rc_cell;
use crate::theory::{RcSymbol, RcTerm, variable::{VariableSymbol}};
use crate::theory::free_theory::{FreeSymbol, FreeTerm};
use crate::theory::variable::VariableTerm;

static OPERATOR_TABLE_PATH: &str = "resources/operators.csv";


pub(crate) struct Parser<'t>(ParserCore<'t>);

impl Parser {
  pub fn new() -> Parser {
    Parser(ParserCore::with_operator_file(OPERATOR_TABLE_PATH))
  }


  pub fn parse(&mut self, text: &str) -> Result<RcTerm, Box<dyn Error>> {
    match self.0.parse_str(text) {
      Ok(atom) => Ok(termify_atom(atom)),
      Err(())   => { Err(Box::new(simple_error!("Parse failed.") ))}
    }


  }
}


fn is_greek_letter(s: char) -> bool {
  unicode_blocks::ANCIENT_GREEK_MUSICAL_NOTATION.contains(s)
  || unicode_blocks::ANCIENT_GREEK_NUMBERS.contains(s)
  || unicode_blocks::GREEK_AND_COPTIC.contains(s)
  || unicode_blocks::GREEK_EXTENDED.contains(s)
  || unicode_blocks::PHONETIC_EXTENSIONS.contains(s)
  || [
        0x1DBF, // MODIFIER LETTER SMALL THETA
        0x2126, // OHM SIGN
        0xAB65, // GREEK LETTER SMALL CAPITAL OMEGA
        0x101A0 // GREEK SYMBOL TAU RHO
     ].contains(&(s as i32))
}

fn termify_atom(atom: Atom) -> RcTerm {
  // let term: RcTerm = // the following match
  match atom {
        Atom::String(_)
        | Atom::Integer(_)
        | Atom::Real(_) => {
          // No literals implemented for this simple matching.
          unimplemented!("Literals are not implemented.");
        }

        Atom::Symbol(name) => {
          let (is_variable, symbol) = name_to_symbol(IString::from(name.as_str()), 0);

          // Variable
          if is_variable {
            rc_cell!(VariableTerm::new(IString::from(name.as_str()), symbol))
          }

          // Symbol (nonvariable)
          else {
            rc_cell!(FreeTerm::new(symbol))
          }
        }

        Atom::SExpression(children) => {
          let mut child_iter = Rc::into_inner(children).unwrap().into_iter();
          let head           = child_iter.next().unwrap();
          // Destructure
          if let Atom::Symbol(name) = head {
            let rest  = child_iter.map(|a| termify_atom(a) ).collect::<Vec<_>>();
            let arity = rest.len() as u32;

            // ToDo: How do I represent a "function variable"?
            let (_is_variable, symbol) = name_to_symbol(IString::from(name.as_str()), arity);
            rc_cell!(FreeTerm::with_args(symbol, rest))
          } else {
            unreachable!("Could not destructure head as a symbol. This is a bug.");
          }
        }
      }
}


/// In our language, an identifier is a variable if it is uppercase or Greek and a regular
/// symbol otherwise.
fn name_to_symbol(name: IString, arity: u32) -> (bool, RcSymbol) {
  let first_char = name.chars().next().unwrap();
  if first_char.is_ascii_uppercase() || is_greek_letter(first_char) {
    // A variable
    (true, Rc::new(VariableSymbol::new(IString::from(name))))
  }
  else {
    // Nonvariable symbol
    (false, Rc::new(FreeSymbol::new( IString::from(name), arity, false, Strategy::default() )))
  }
}




#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_symbol_expr_test() {
    // set_verbosity(5);

    let mut parser = Parser::new();
    let text = "f(x(a), G(α, x(a)), β)";
    let term = match parser.parse(text) {
      Ok(term) => term,
      Err(_err) => {
        panic!("FAILED TO PARSE.");
      }
    };
    println!("TEXT: {}", text);
    println!("TERM: {}", term.borrow());

    let dag = term.borrow_mut().make_dag();
    println!("DAG: {}", dag.borrow());
  }

}
