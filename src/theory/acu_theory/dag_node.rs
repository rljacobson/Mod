/*!

Concrete types for the ACU theory implementing the DagNode trait.

*/

use core::panicking::panic;
use std::any::Any;
use std::borrow::BorrowMut;
use crate::Substitution;

use crate::theory::dag_node::{DagNode, DagPair};
use crate::theory::symbol::Symbol;
use crate::theory::term::{ReturnValue, Term};
use super::red_black_tree::{RedBlackTree, RedBlackNode};

pub enum NormalizationStatus {
  ///	Default: no guarantees.
  Fresh,
  ///	Node was produced by an assignment in ACU matcher:
  ///	(a) all arguments are reduced up to strategy of our symbol
  ///	   (this only holds if it was true of subject before matching);
  ///	(b) all arguments have the correct sort; and
  ///	(c) argument list in theory normal form.
  Assignment,
  ///	As above but arguments are stored in a red-black (ACU_TreeDagNode)
  ///	rather than in an ArgVec (ACU_DagNode).
  Tree
}

pub enum ACUArguments {
  List(Vec<DagPair>),
  Tree(RedBlackTree)
}

impl ACUArguments {
  pub fn size(&self) -> usize {
    match self {
      ACUArguments::List(v) => { v.len() }
      ACUArguments::Tree(t) => { t.size() }
    }
  }

  /// Alias for size()
  pub fn len(&self) -> usize {
    self.size()
  }

  /// Searches for the given term and returns its multiplicity if found.
  pub fn search_for_term(&self, term: &dyn Term) -> Option<u32> {
    match self {

      ACUArguments::List(v) => {
        v.binary_search_by((|(t, m)| t.compare(term))).map(|idx| v[idx].1).ok()
      },

      ACUArguments::Tree(t) => {
        t.node_for_key(term)
      }
    }
  }

  pub fn iter(&self) -> Box<dyn Iterator<Item=(&dyn DagNode, u32)>>{
    match self {
      ACUArguments::List(v) => {
        Box::new(v.iter().map(|pair| (&pair.0, pair.1)))
      },
      ACUArguments::Tree(t) => {
        Box::new(t.iter())
      }
    }
  }

}


pub struct ACUDagNode {
  top_symbol: Box<Symbol>,
  pub(crate) args: ACUArguments
}

impl ACUDagNode {
  ///	Returns index of argument equal key, or a -ve value pos otherwise.
  ///	In the latter case ~pos is the index of the smallest element greater
  ///	than key, and can be argArray.length() if key is greater than all elements
  ///	in the array.
  pub fn binary_search_by_term(&self, key: &dyn Term) -> usize {
    // The Maude source seems to suggest that this method is only called when the args is a vector.
    if let ACUArguments::List(args) = &self.args {

      let mut upper = args.len();
      let mut lower: usize = 0;

      loop {
        let sum = upper + lower;
        let probe = sum/2;

        let r =  key.compare_dag_node(args[probe].dag_node.as_ref());
        if r ==  0 {
          return probe;
        }
        if r < 0 {
          upper = probe - 1;
        } else {
          lower = probe + 1;
        }
        if lower > upper  {
          break;
        }
      }
    } else {
      panic!("Error: binary_search_by_term called on an ACUDagNode with tree args. This is a bug.");
    }
    -1
  }

  /// Converts self.args into `ACUArguments::List(..)` if necessary. Conversion is done in place.
  pub fn to_list_arguments(&mut self){
    if let ACUArguments::Tree(t) = &mut self.args {
      self.args = ACUArguments::List(t.vectorize())
    }
  }

  /// Matches the subject or else proves that the subject cannot match.
  pub fn eliminate_subject(
    &mut self,
    target: &mut dyn DagNode,
    multiplicity: u32,
    subject_multiplicity: &Vec<u32>
  ) -> bool
  {
    if let Some(identity) = self.top_symbol.get_identity() {
      if identity.equal(target) {
        return true;
      }
    }
    if target.symbol() == self.top_symbol {
      // Since self.top_symbol is an ACUDagNode, so must be target.
      if let Some(acu_dag_node) = target.as_any().downcast_mut::<ACUDagNode>(){
        // Todo: Why do we vectorize here?
        acu_dag_node.to_list_arguments();
        if let Some(ACUArguments::List(args)) = &target.args {
          for DagPair{ dag_node: arg_dag_node, multiplicity: arg_multiplicity } in args {
            let pos: usize = self.binary_search_by_term(arg_dag_node);
            if pos < 0 {
              return false;
            }
            subject_multiplicity[pos] -= arg_multiplicity*multiplicity;
            if subject_multiplicity[pos] < 0 {
              return false;
            }
          } // end iter over arg pairs
        } // end destructure target args
        else {
          // Should be infallible.
          panic!("ACUArguments::Tree after vectorizing. This is a bug.");
        }
      } else {
        // Should be infallible.
        panic!("DagNode could not be downcast to ACUNode despite being equal to one. This is a bug.");
      }

    } // end if self.top_symbol == target.top_symbol
    else {
      let pos = self.binary_search(target);
      if pos < 0 {
        return false;
      }
      subject_multiplicity[pos] -= multiplicity;
      if subject_multiplicity[pos] < 0 {
        return false;
      }
    }
    true
  }


  ///	Return the smallest index whose subdag is a potential match for key, given the partial substitution
  /// for key's variables. If we know that no subdag can match we return an index 1 beyond the maximal index.
  pub(crate) fn find_first_potential_match(&self, key: &dyn Term, partial: &mut Substitution) -> u32  {
    // I think self is already guaranteed to be vectorized.
    if let ACUArguments::List(args) = &self.args {
      let mut first = args.len();
      let mut upper = first - 1;
      let mut lower = 0;

      while lower <= upper {
        let mut probe = (upper + lower)/2;
        let r = key.partial_compare(partial, args[probe].dag_node.as_ref());

        match r {
          ReturnValue::Greater => { lower = probe + 1; }
          ReturnValue::Less => { upper = probe - 1; }
          ReturnValue::Equal => { return probe as u32;}
          ReturnValue::Unknown => {
            //	We need to treat probe as a potential match, and search to see if there
            //	is one with a smaller index.
            first = probe;
            upper = probe - 1;
          }
        }
      }

      first as u32
    } else {
      panic!("Error: find_first_potential_match called when self is not vectorized. This is a bug.");
    }
  }
}

/// If the given DagNode is an ACUDagNode and it has tree args, vectorize it. Modifies the node in place.
// fn to_acu_list_args(dag_node: &mut dyn DagNode) {
//   if let Some(acu_dag_node) = dag_node.as_any().downcast_mut::<ACUDagNode>(){
//     acu_dag_node.to_list_arguments();
//   }
// }

impl DagNode for ACUDagNode {
  fn symbol(&self) -> &Symbol {
    self.top_symbol.as_ref()
  }

  // Todo: Is this needed?
  fn symbol_mut(&mut self) -> &mut Symbol {
    self.top_symbol.borrow_mut()
  }

  fn iter_args(&self) -> Box<dyn Iterator<Item=(&dyn DagNode, u32)>> {
    self.args.iter()
  }

  fn len(&self) -> usize {
    self.args.len()
  }

  fn as_any(&self) -> &dyn Any{
    self
  }
}


