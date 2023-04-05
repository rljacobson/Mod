/*!

Concrete types for the ACU theory implementing the DagNode trait.

*/

use std::{
  any::Any,
  borrow::BorrowMut,
  cmp::Ordering
};

use crate::{
  sort::{
    SpecialSort,
    RcSort
  },
  Substitution,
  theory::{
    RcDagNode,
    DagNode,
    DagPair,
    Term,
    Symbol,
    BinarySymbol,
    dag_node::{DagNodeFlags, DagNodeFlag}
  },
  ordering_value::{
    OrderingValue,
    numeric_ordering
  }, Sort
};

use super::{
  RcRedBlackTree,
  symbol::RcACUSymbol,
  red_black_tree::RedBlackTree
};


pub type RcACUDagNode = RcCell<ACUDagNode>;

#[derive(Copy, Clone, Eq, PartialEq)]
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
      ACUArguments::Tree(t) => { t.size }
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
        v.binary_search_by(
          |pair| term.compare_dag_node(pair.dag_node.as_ref())  // Supposed to be term.compare_dag_node? or pair.dag_node.compare?
        ).map(
          |idx| v[idx].multiplicity
        ).ok()
      },

      ACUArguments::Tree(t) => {
        // find_term(..) returns `None` if nothing is found, so unwrapping the result of cursor.get never panics.
        t.find_term(term).map(|c| *c.multiplicity.borrow())
      }
    }
  }

  pub fn iter(&self) -> Box<dyn Iterator<Item=(RcDagNode, u32)>>{
    match self {
      ACUArguments::List(v) => {
        Box::new(v.iter().map(|pair| (pair.dag_node.clone(), pair.multiplicity)))
      },
      ACUArguments::Tree(t) => {
        Box::new(t.iter().map(|(dn, m)| (dn.clone(), m) ))
      }
    }
  }

}


pub struct ACUDagNode {
  pub(crate) top_symbol: RcACUSymbol,
  pub(crate) args      : ACUArguments,
  pub(crate) sort      : RcSort,
  pub(crate) flags      : DagNodeFlags,
  pub(crate) is_reduced: bool,
  pub(crate) sort_index: i32,
  pub(crate) normalization_status: NormalizationStatus,
}



impl ACUDagNode {

  pub const ASSIGNMENT: NormalizationStatus = NormalizationStatus::Assignment;

  pub fn new(symbol: RcACUSymbol, size: usize, normalization_status: NormalizationStatus) -> Self {
    ACUDagNode {
      top_symbol: symbol,
      args: ACUArguments::List(Vec::with_capacity(size)),
      normalization_status,
      sort: Strong::new(Sort::default()),
      flags: DagNodeFlag::Reduced.into(),
      is_reduced: true,
      sort_index: 0,

    }
  }

  pub fn new_tree(symbol: RcACUSymbol, tree: RedBlackTree) -> Self {
    ACUDagNode {
      top_symbol: symbol,
      args: ACUArguments::Tree(tree),
      sort: Strong::new(Sort::default()),
      flags: DagNodeFlag::Reduced.into(),
      is_reduced: true,
      sort_index: 0,
      normalization_status: NormalizationStatus::Fresh,
    }
  }

  /// Computes the base sort index for self.
  pub fn compute_base_sort(&self) -> i32 {
    // Todo: Implement uniform_sort()
    let s = self.symbol();
    if let Some(uni_sort) = s.uniform_sort() {
      // If symbol has a uniform sort structure do a fast sort computation.
      // Todo: Implement component(), error_free()
      if !uni_sort.component().error_free() {
        let mut last_index = SpecialSort::Unknown as i32;

        for (dag_node, multiplicity) in self.args.iter() {
          let index = dag_node.get_sort_index();
          assert!(index >= (SpecialSort::ErrorSort as i32));
          if index != last_index {
            if index >= uni_sort {
              return SpecialSort::ErrorSort as i32;
            }
            last_index = index;
          }
        }
      }
      return uni_sort.index();
    }

    // Standard sort calculation.
    let mut arg_iter = self.iter_args();
    // The initial value is a special case.
    let mut sort_index = {
      let (dag_node, multiplicity) = arg_iter.next().unwrap();
      let index = dag_node.get_sort_index();
      assert!(index >= SpecialSort::ErrorSort as i32);
      // The first case subtracts 1 from node.multiplicity.
      // Todo: Implement `Symbol::compute_multisort_index(..)`
      s.compute_multisort_index(index, index, multiplicity - 1)
    };
    // Now do the remaining args.
    for (dag_node, multiplicity) in arg_iter {
      let index = dag_node.get_sort_index();
      assert!(index >= SpecialSort::ErrorSort as i32);
      sort_index = s.compute_multisort_index(index, index, multiplicity);
    }

    sort_index
  }

  ///	Returns index of argument equal key, or a -ve value pos otherwise.
  ///	In the latter case ~pos is the index of the smallest element greater
  ///	than key, and can be argArray.length() if key is greater than all elements
  ///	in the array.
  pub fn binary_search_by_term(&self, key: &dyn Term) -> isize {
    // The Maude source seems to suggest that this method is only called when the args is a vector.
    if let ACUArguments::List(args) = &self.args {

      let mut upper = args.len();
      let mut lower: usize = 0;

      loop {
        let sum = upper + lower;
        let probe = sum/2;

        let r =  key.compare_dag_node(args[probe].dag_node.as_ref());
        match r {
          Ordering::Equal => {
            return probe as isize;
          }
          Ordering::Less => {
            upper = probe - 1;
          }
          Ordering::Greater => {
            lower = probe + 1;
          }
        }

        if lower > upper  {
          break;
        }
      }
      return !(lower as isize);
    } else {
      panic!("Error: binary_search_by_term called on an ACUDagNode with tree args. This is a bug.");
    }
  }


  ///	Returns index of argument equal key, or a -ve value pos otherwise.
  ///	In the latter case ~pos is the index of the smallest element greater
  ///	than key, and can be argArray.length() if key is greater than all elements
  ///	in the array.
  pub fn binary_search_by_dagnode(&self, key: &dyn DagNode) -> isize {
    // Todo: This method is identical to `binary_search_by_term` except for the type of `key`.

    // The Maude source seems to suggest that this method is only called when the args is a vector.
    if let ACUArguments::List(args) = &self.args {

      let mut upper = args.len();
      let mut lower: usize = 0;

      loop {
        let sum = upper + lower;
        let probe = sum/2;

        let r =  key.compare(args[probe].dag_node.as_ref());
        match r {
          Ordering::Equal => {
            return probe as isize;
          }
          Ordering::Less => {
            upper = probe - 1;
          }
          Ordering::Greater => {
            lower = probe + 1;
          }
        }

        if lower > upper  {
          break;
        }
      }
      return !(lower as isize);
    } else {
      panic!("Error: binary_search_by_dagnode called on an ACUDagNode with tree args. This is a bug.");
    }
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
    target: &dyn DagNode,
    multiplicity: u32,
    subject_multiplicity: &mut Vec<i32>
  ) -> bool
  {
    if let Some(identity) = Rc::as_ref(&self.top_symbol).get_identity() {
      if identity.compare_dag_node(target) == Ordering::Equal{
        return true;
      }
    }
    if target.symbol() == Rc::as_ref(&self.top_symbol) as &dyn Symbol {
      // Since self.top_symbol is an ACUDagNode, so must be target.
      if let Some(acu_dag_node) = target.as_any().downcast_ref::<ACUDagNode>(){
        // Todo: Why do we vectorize here?
        acu_dag_node.to_list_arguments();
        if let ACUDagNode{args: ACUArguments::List(args), ..} = acu_dag_node {
          for (arg_dag_node, arg_multiplicity) in acu_dag_node.iter_args() {
            let pos = self.binary_search_by_dagnode(arg_dag_node.as_ref());
            if pos < 0 {
              return false;
            }
            subject_multiplicity[pos as usize] -= (arg_multiplicity * multiplicity) as i32;
            if subject_multiplicity[pos as usize] < 0 {
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
      let pos = self.binary_search_by_dagnode(target);
      if pos < 0 {
        return false;
      }
      subject_multiplicity[pos as usize] -= multiplicity as i32;
      if subject_multiplicity[pos as usize] < 0 {
        return false;
      }
    }
    true
  }


  ///	Return the smallest index whose subdag is a potential match for key, given the partial substitution
  /// for key's variables. If we know that no subdag can match we return an index 1 beyond the maximal index.
  ///
  /// There are two versions of this function: One on `RedBlackTree` and one on `DagNode`. The `DagNode` version
  /// operates on `ACUArguments::List(args)` , while the `RedBlackTree` obviously operates on trees.
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
          OrderingValue::Greater => { lower = probe + 1; }
          OrderingValue::Less => { upper = probe - 1; }
          OrderingValue::Equal => { return probe as u32;}
          OrderingValue::Unknown => {
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
  fn symbol(&self) -> &dyn Symbol {
    // Rc::as_ref(&self.top_symbol) as &dyn Symbol
    self.top_symbol.clone()
  }

  // Todo: Is this needed?
  fn symbol_mut(&mut self) -> &mut dyn Symbol {
    Rc::get_mut(&mut self.top_symbol).unwrap()   //.borrow_mut()
  }

  fn iter_args(&self) -> Box<dyn Iterator<Item=(RcDagNode, u32)>> {
    Box::new(self.args.iter())
  }

  fn compare_arguments(&self, other: &dyn DagNode) -> Ordering {
    match other.as_any().downcast_ref::<ACUDagNode>() {
      Some(acu_dag_node) => {
        // Fail fast if lengths differ.
        let r: i32 = self.args.len() as i32 - acu_dag_node.len() as i32;
        if r != 0 {
          return numeric_ordering(r as usize);
        }
        // Compare corresponding terms.
        for ((this_child, this_multiplicity), (other_child, other_multiplicity))
        in self.iter_args().zip(acu_dag_node.iter_args()) {
          let r: i32 = this_multiplicity as i32 - other_multiplicity as i32;
          if r != 0 {
            return numeric_ordering(r as usize);
          }

          let r = this_child.compare(other_child.as_ref());
          if r != Ordering::Equal {
            return r;
          }
        }
        // Identical
        return Ordering::Equal;
      }
      None => panic!("Could not downcast a Term to an ACUTerm."),
    };
  }

  fn get_sort(&self) -> RcSort {
    self.sort.clone()
  }

  fn set_sort_index(&mut self, sort_index: i32) {
    self.sort_index = sort_index;
  }

  fn get_sort_index(&self) -> i32 {
    self.sort_index
  }

  fn len(&self) -> usize {
    self.args.len()
  }

  fn as_any(&self) -> &dyn Any{
    self
  }

  fn compute_base_sort(&self) -> i32 {
    todo!()
  }

  fn flags(&self) -> DagNodeFlags {
    self.flags
  }
}


