/*!

Functions for collecting and displaying profiling statistics for module items.

In Maude, there is a subclass of `Module` called `ProfileModule`.

*/

use crate::{
  core::{
    format::{FormatStyle, Formattable},
    interpreter::module::{item::ModuleItem, module::Module},
    pre_equation::{PreEquation, PreEquationKind::*, RcPreEquation},
    rewrite_context::RewriteType,
    sort::SpecialSort,
  },
  theory::{DagNode, RcDagNode, RcSymbol, Symbol},
  NONE,
};

#[derive(Default)]
pub struct SymbolProfile {
  pub(crate) builtin_mb_rewrite_count: u64,
  pub(crate) builtin_eq_rewrite_count: u64,
  pub(crate) builtin_rl_rewrite_count: u64,
  pub(crate) memo_rewrite_count:       u64,
}

impl SymbolProfile {
  pub(crate) fn new() -> Self {
    Self::default()
  }
}

#[derive(Default)]
pub struct FragmentProfile {
  pub(crate) success_count: u64,
  pub(crate) failure_count: u64,
}

impl FragmentProfile {
  fn new() -> Self {
    Self::default()
  }
}

#[derive(Default)]
pub struct StatementProfile {
  pub(crate) rewrite_count:         u64,
  pub(crate) condition_start_count: u64,
  pub(crate) fragment_info:         Vec<FragmentProfile>,
}

impl StatementProfile {
  pub(crate) fn new() -> Self {
    Self::default()
  }

  pub(crate) fn update_fragment_info(&mut self, index: usize, success: bool) {
    if index >= self.fragment_info.len() {
      self.fragment_info.resize_with(index + 1, FragmentProfile::new);
    }
    if success {
      self.fragment_info[index].success_count += 1;
    } else {
      self.fragment_info[index].failure_count += 1;
    }
  }
}


// Used in the `show_*` functions.
pub(crate) fn format_percent(n: u64, float_total: f64) -> String {
  format!("{} ({:.2}%)", n, (100.0 * n as f64) / float_total)
}
