use std::{any::Any, cell::RefCell, cmp::Ordering, rc::Rc};

use shared_vector::SharedVector;

use super::{FreeTerm, RcFreeSymbol};
use crate::{
  abstractions::RcCell,
  core::{hash_cons_set::HashConsSet, sort::SpecialSort, RedexPosition},
  rc_cell,
  theory::{
    dag_node::MaybeDagNode,
    DagNode,
    DagNodeFlag,
    DagNodeFlags,
    DagNodeMembers,
    DagPair,
    NodeList,
    RcDagNode,
    RcSymbol,
    RcTerm,
    Symbol,
  },
};


pub type RcFreeDagNode = Rc<FreeDagNode>;

pub struct FreeDagNode {
  // Base DagNode Members
  pub members: DagNodeMembers,
}

impl FreeDagNode {
  pub fn new(symbol: RcSymbol) -> FreeDagNode {
    let members = DagNodeMembers {
      top_symbol: symbol,
      args:       Default::default(),
      // sort: Default::default(),
      flags:      Default::default(),
      sort_index: 0,
      copied_rc:  None,
      hash:       0,
    };

    FreeDagNode { members }
  }
}

impl DagNode for FreeDagNode {
  #[inline(always)]
  fn dag_node_members(&self) -> &DagNodeMembers {
    &self.members
  }

  #[inline(always)]
  fn dag_node_members_mut(&mut self) -> &mut DagNodeMembers {
    &mut self.members
  }

  #[inline(always)]
  fn as_any(&self) -> &dyn Any {
    self
  }

  #[inline(always)]
  fn as_any_mut(&mut self) -> &mut dyn Any {
    self
  }

  #[inline(always)]
  fn as_ptr(&self) -> *const dyn DagNode {
    self
  }

  fn compare_arguments(&self, other: &dyn DagNode) -> Ordering {
    assert!(self.symbol() == other.symbol(), "symbols differ");

    let s = self.symbol();
    let arg_count = s.arity() as usize;

    if arg_count != 0 {
      let mut pd = self;
      let mut qd = match other.as_any().downcast_ref::<FreeDagNode>() {
        Some(v) => v,
        None => {
          // Not even the same theory. It's not clear what to return in this case, so just compare symbols.
          return s.compare(other.symbol().as_ref());
        }
      };

      loop {
        let p = &pd.dag_node_members().args;
        let q = &qd.dag_node_members().args;

        // Compare all but the last (rightmost) node.
        for i in (0..arg_count - 1).rev() {
          let r = DagNode::compare(p[i].as_ref(), q[i].as_ref());
          if r.is_ne() {
            return r;
          }
        }

        let pd2 = p[arg_count - 1].as_ref();
        let qd2 = q[arg_count - 1].as_ref();
        // Fast bail on equal pointers.
        if std::ptr::addr_of!(pd2) == std::ptr::addr_of!(qd2) {
          return Ordering::Equal; // Points to same node
        }
        // Compare symbols
        let ps = pd2.symbol();
        let qs = qd2.symbol();
        let r = ps.compare(qs.as_ref());
        if r.is_ne() {
          return r; // different symbols
        }
        // Go ahead and check all arguments
        if *ps != *s {
          return pd2.compare_arguments(qd2);
        }

        // Next iteration will compare argument lists using tail recursion elimination. See https://imgur.com/a/12dtE3j.
        pd = pd2.as_any().downcast_ref::<FreeDagNode>().unwrap();
        qd = qd2.as_any().downcast_ref::<FreeDagNode>().unwrap();
      }
    }
    // Survived all attempts at finding inequality.
    return Ordering::Equal;
  }

  fn compute_base_sort(&mut self) -> i32 {
    let symbol = self.symbol();
    // assert_eq!(self as *const _, subject.symbol() as *const _, "bad symbol");
    let arg_count = symbol.arity();
    if arg_count == 0 {
      let idx = symbol.sort_table().traverse(0, 0);
      self.set_sort_index(idx); // Maude: HACK
      return idx;
    }

    let mut state = 0;
    // enumerate is only used for assertion
    for (i, arg) in self.iter_args().enumerate() {
      let t = arg.borrow().get_sort_index();
      assert_ne!(
        t,
        SpecialSort::Unknown as i32,
        "unknown sort encounter for arg {} subject = {}",
        i,
        self as &dyn DagNode
      );
      state = symbol.sort_table().traverse(state as usize, t as usize);
    }
    self.set_sort_index(state);
    state
  }

  fn termify(&self) -> RcTerm {
    let args: Vec<RcTerm> = self.members.args.iter().map(|d| d.borrow().termify()).collect();
    rc_cell!(FreeTerm::with_args(self.symbol(), args))
  }

  fn shallow_copy(&self) -> RcDagNode {
    let fdg = FreeDagNode {
      members: DagNodeMembers {
        top_symbol: self.symbol(),
        args:       self.members.args.clone_buffer(), // Clones all elements of the `SharedVec`
        flags:      self.flags() & DagNodeFlags::RewritingFlags,
        sort_index: self.get_sort_index(),
        copied_rc:  self.members.copied_rc.clone(),
        hash:       0,
      },
    };

    rc_cell!(fdg)
  }

  fn copy_with_replacements(&self, redex_stack: &[RedexPosition], mut first_idx: usize, last_idx: usize) -> RcDagNode {
    assert!(
      first_idx <= last_idx && last_idx < redex_stack.len(),
      "bad replacement range"
    );
    let symbol = self.symbol();
    let arg_count = symbol.arity() as i32;
    assert!(
      redex_stack[first_idx].arg_index < arg_count && redex_stack[last_idx].arg_index < arg_count,
      "bad replacement arg index"
    );

    let mut new_dag_node = FreeDagNode::new(symbol);
    let args = &self.members.args;
    let new_args = &mut new_dag_node.members.args;
    let mut next_replacement_index = redex_stack[first_idx].arg_index;

    for i in 0..arg_count {
      if i == next_replacement_index {
        new_args.push(redex_stack[first_idx].dag_node.clone());
        first_idx += 1;
        next_replacement_index = if first_idx <= last_idx {
          redex_stack[first_idx].arg_index
        } else {
          -1
        };
      } else {
        new_args.push(args[i as usize].clone());
      }
    }
    rc_cell!(new_dag_node)
  }

  fn copy_with_replacement(&self, replacement: RcDagNode, arg_index: usize) -> RcDagNode {
    let symbol = self.symbol();
    let arg_count = symbol.arity() as usize;
    assert!(arg_index < arg_count, "bad argIndex");

    let mut new_dag_node = FreeDagNode::new(symbol);
    let new_args = &mut new_dag_node.members.args;
    // Because args is COW, we could just clone. But this is safer, in case someone forgets and changes args to non-COW.
    *new_args = self.members.args.clone_buffer();

    new_args[arg_index] = replacement;

    rc_cell!(new_dag_node)
  }

  fn copy_eager_upto_reduced_aux(&mut self) -> RcDagNode {
    let symbol = self.symbol();
    let mut new_dag_node = FreeDagNode::new(symbol);
    let arg_count = symbol.arity() as usize;

    if arg_count != 0 {
      if symbol.strategy().standard_strategy() {
        new_dag_node.members.args.extend(
          self
            .members
            .args
            .iter()
            .map(|v| v.borrow().copy_eager_upto_reduced().unwrap()),
        );
      } else {
        let p = &mut self.members.args;
        let mut q = &mut new_dag_node.dag_node_members_mut().args;

        for i in 0..arg_count {
          q[i] = if symbol.strategy().eager_argument(i) {
            p[i].borrow().copy_eager_upto_reduced().unwrap()
          } else {
            p[i].clone()
          };
        }
      }
    }
    rc_cell!(new_dag_node)
  }

  fn copy_all_aux(&mut self) -> RcDagNode {
    let symbol = self.symbol();
    let mut new_dag_node = FreeDagNode::new(symbol);
    let arg_count = symbol.arity() as usize;

    if arg_count != 0 {
      let p = &mut self.members.args;
      let mut q = &mut new_dag_node.members.args;
      for i in 0..arg_count {
        // ToDo: Can we justify the unwrap?
        q[i] = p[i].borrow().copy_all().unwrap();
      }
    }
    rc_cell!(new_dag_node)
  }

  fn overwrite_with_clone(&mut self, old: RcDagNode) {
    if let Some(old_dag_node) = old.borrow_mut().as_any_mut().downcast_mut::<FreeDagNode>() {
      let mut fdg = FreeDagNode::new(self.symbol());

      fdg.set_sort_index(self.get_sort_index());
      fdg.set_flags(
        self.flags()
          | DagNodeFlag::Reduced
          | DagNodeFlag::Unrewritable
          | DagNodeFlag::Unstackable
          | DagNodeFlag::Ground,
      );
      fdg.members.args = old_dag_node.members.args.clone();

      let _ = std::mem::replace(old_dag_node, fdg);
    } else {
      unreachable!("This execution path should be unreachable. This is a bug.")
    }
  }

  /// For hash consing, recursively checks child nodes to determine if a canonical copy needs to be made.
  fn make_canonical(&self, rc_dag_node: RcDagNode, hcs: &mut HashConsSet) -> RcDagNode {
    // Downcast
    if let Some(dag_node) = rc_dag_node.borrow_mut().as_any_mut().downcast_mut::<FreeDagNode>() {
      let nr_args = dag_node.members.top_symbol.arity() as usize;
      let args = &mut dag_node.members.args;

      for i in 0..nr_args {
        let d: RcDagNode = args[i].clone();
        // let c = hcs.get_canonical(hcs.insert(d));
        let (canonical_dag_node, _) = hcs.insert(d);

        if Rc::ptr_eq(&canonical_dag_node, &args[i]) {
          // The child node was already canonical.
          continue;
        }

        // Detected a non-canonical argument, need to make a new copy
        let symbol = Rc::clone(&dag_node.members.top_symbol);
        let mut new_node = FreeDagNode::new(symbol);
        new_node.members.flags.set_copied_flags(dag_node.members.flags);
        new_node.members.sort_index = dag_node.members.sort_index;

        let new_args = &mut new_node.members.args;
        for j in 0..i {
          new_args.push(args[j].clone());
        }
        new_args.push(canonical_dag_node);
        for j in i + 1..nr_args {
          let (canonical, _) = hcs.insert(args[j].clone());
          new_args.push(canonical);
        }

        return Rc::new(new_node) as RcDagNode;
      }

      rc_dag_node // Can use the original DAG node as the canonical version
    } else {
      unreachable!("This execution path should be unreachable. This is a bug.")
    }
  }
}
