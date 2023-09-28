/*!

In the current implementation this is just a shell. The actually dag nodes are stored in the module.
Thus `MemoTable` needs to be derived from `ModuleItem` in order to find the module.

*/

use crate::core::module::{ModuleItem, WeakModule};
use crate::core::rewrite_context::{ContextAttribute, RcRewritingContext, RewriteType};
use crate::theory::RcDagNode;

pub type SourceSet = Vec<i32>;

pub struct MemoTable{
  is_memoized: bool,

  // `ModuleItem`
  index_within_parent_module: i32,
  pub(crate) parent_module  : WeakModule,
}

impl ModuleItem for MemoTable {
  fn get_index_within_module(&self) -> i32 {
    self.index_within_parent_module
  }

  fn set_module_information(&mut self, module: WeakModule, index_within_module: i32) {
    self.parent_module = module;
    self.index_within_parent_module = index_within_module;
  }

  fn get_module(&self) -> WeakModule {
    self.parent_module.clone()
  }
}


impl MemoTable {
  fn memo_rewrite(&mut self, source_set: &mut SourceSet, subject: RcDagNode, context: RcRewritingContext) -> bool {
    // #if 0
    // DebugAdvisory("memoRewrite()  subject " << subject <<
    //       " at " << ((void*) subject) <<
    //       " has sort index " << subject->getSortIndex());
    // #endif
    let memo_map = self.get_module().get_memo_map(); // Assuming appropriate methods for `get_module` and `get_memo_map`.
    let subject_index = memo_map.get_from_index(subject.clone());

    if let Some(to_dag) = memo_map.get_to_dag(subject_index) {
      // #if 0
      // DebugAdvisory("memoRewrite()  toDag " << subject <<
      //     " at " << ((void*) toDag) <<
      //     " has sort index " << subject->getSortIndex());
      // #endif
      let mut context = context.borrow_mut();
      let trace = context.attribute(ContextAttribute::Trace);
      if trace {
        context.trace_pre_eq_application(Some(subject.clone()), None, RewriteType::Memoized);
        if context.trace_abort() {
          return false;
        }
      }
      to_dag.overwrite_with_clone(subject.clone()); // Assuming appropriate methods for `overwrite_with_clone`.
      context.eq_count += 1;
      if trace {
        context.trace_post_eq_rewrite(subject.clone());
      }
      return true;
    }
    source_set.append(subject_index);
    false
  }

  fn memo_enter(&mut self, source_set: &SourceSet, destination: RcDagNode) {
    // #if 0
    // DebugAdvisory("memoEnter()  destination " << destination <<
    //       " at " << ((void*) destination) <<
    //       " has sort index " << destination->getSortIndex());
    // #endif
    let memo_map = self.get_module().upgrade().unwrap().get_mut().memo_map.as_mut();
    for index in source_set.iter() {
      memo_map.assign_to_dag(*index, destination.clone()); // Assuming appropriate method for `assign_to_dag`.
    }
  }
}
