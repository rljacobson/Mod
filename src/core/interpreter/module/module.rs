use string_cache::DefaultAtom as IString;
use tiny_logger::{Channel::Debug, log};

use crate::{core::{
  module::{StatementProfile, SymbolProfile},
  pre_equation::RcPreEquation,
  sort::{RcConnectedComponent, SortSet},
}, NONE, theory::RcSymbol};
use crate::core::format::{FormatStyle, Formattable};
use crate::core::module::FragmentProfile;
use crate::core::module::item::ModuleItem;
use crate::core::pre_equation::{Equation, PreEquation, Rule, SortConstraint};
use crate::core::rewrite_context::RewriteType;
use crate::core::sort::SpecialSort;
use crate::theory::RcDagNode;

#[derive(Copy, Clone, Default)]
pub enum ModuleStatus {
  #[default]
  Open,
  SortSetClosed,
  SignatureClosed,
  FixUpsClosed,
  TheoryClosed,
  StackMachineCompiled,
}

// Local alias for convenience.
type Status = ModuleStatus;

#[derive(Default)]
pub struct Module {
  // environment: RcEnvironment ,  // pointer to some object in which module exists
  pub status: ModuleStatus,

  // TODO: Does a module own its `Sorts`?
  pub sorts:               SortSet,
  pub connectedComponents: Vec<RcConnectedComponent>,
  pub symbols:             Vec<RcSymbol>,
  pub sort_constraints:    Vec<RcPreEquation>,
  pub equations:           Vec<RcPreEquation>,
  pub rules:               Vec<RcPreEquation>,

  // strategies: Vec<RcRewriteStrategy> ,
  // strategyDefinitions: Vec<RcStrategyDefinition> ,
  // sortBdds: RcSortBdds ,
  pub(crate) minimum_substitution_size: i32,

  // pub memo_map: BxMemoMap ,  // Memoization map for all symbols in module

  // NamedEntity members
  /// An ID, a name given by the user.
  pub name: IString,

  // ProfileModule members
  symbol_info: Vec<SymbolProfile>,
  mb_info    : Vec<StatementProfile>, // Membership
  eq_info    : Vec<StatementProfile>, // Equation
  rl_info    : Vec<StatementProfile>, // Rule
  // sd_info    : Vec<StatementProfile>, // Strategy Definition
}

impl Module {
  // The `name` parameter is an ID, a name given by the user. It is called `id` in Maude. Here we use an interned
  // string.

  #[inline(always)]
  pub fn new(name: IString) -> Module {
    Module {
      name,
      ..Module::default()
    }
  }

  #[inline(always)]
  pub fn notify_substitution_size(&mut self, minimum_size: i32) {
    if minimum_size > self.minimum_substitution_size {
      log(
        Debug,
        5,
        format!(
          "minimumSubstitutionSize for {:?} increased from {} to {}",
          self.name, self.minimum_substitution_size, minimum_size
        )
        .as_str(),
      );
      self.minimum_substitution_size = minimum_size;
    }
  }


  fn clear_profile(&mut self) {
    self.symbol_info.clear();
    self.mb_info.clear();
    self.eq_info.clear();
    self.rl_info.clear();
    // self.sd_info.clear();
  }

  pub(crate) fn profile_mb_rewrite(&mut self, redex: RcDagNode, membership_axiom: Option<&PreEquation>) {
    if let Some(membership_axiom) = membership_axiom {
      let index = membership_axiom.get_index_within_module() as usize;
      if index >= self.mb_info.len() {
        self.mb_info.resize_with((index + 1), StatementProfile::new);
      }
      self.mb_info[index].rewrite_count += 1;
    } else {
      let index = redex.borrow().symbol().get_index_within_module();
      if index >= self.symbol_info.len() {
        self.symbol_info.resize_with((index + 1), SymbolProfile::new);
      }
      self.symbol_info[index].builtin_mb_rewrite_count += 1;
    }
  }

  pub(crate) fn profile_eq_rewrite(&mut self, redex: RcDagNode, eq: Option<&PreEquation>, rewrite_type: RewriteType) {
    if let Some(eq) = eq {
      let index = eq.get_index_within_module() as usize;
      if index >= self.eq_info.len() {
        self.eq_info.resize_with((index + 1) as usize, StatementProfile::new);
      }

      self.eq_info[index].rewrite_count += 1;
    } else {
      let index = redex.borrow().symbol().get_index_within_module() as usize;
      if index >= self.symbol_info.len() {
        self.symbol_info.resize_with((index + 1) as usize, SymbolProfile::new);
      }

      match rewrite_type {
        RewriteType::Builtin => {
          self.symbol_info[index].builtin_eq_rewrite_count += 1;
        }

        RewriteType::Memoized => {
          self.symbol_info[index].memo_rewrite_count += 1;
        }

        _ => { /* pass */ }
      }
    }
  }

  pub(crate) fn profile_rl_rewrite(&mut self, redex: RcDagNode, rl: Option<&PreEquation>) {
    if let Some(rl) = rl {
      let index = rl.get_index_within_module() as usize;
      if index >= self.rl_info.len() {
        self.rl_info.resize_with(index + 1, StatementProfile::new);
      }
      self.rl_info[index].rewrite_count += 1;
    } else {
      let index = redex.borrow().symbol().get_index_within_module() as usize;
      if index >= self.symbol_info.len() {
        self.symbol_info.resize_with((index + 1) as usize, SymbolProfile::new);
      }
      self.symbol_info[index].builtin_rl_rewrite_count += 1;
    }
  }

  /*
  fn profile_sd_rewrite(&mut self, _: RcDagNode, sd: Option<&StrategyDefinition>) {
    // There are no built-in strategy definitions
    let sd = sd.unwrap();
    let index = sd.get_index_within_module() as usize;
    if index >= self.sd_info.len() {
      self.sd_info.resize_with(index + 1, StatementProfile::new);
    }
    self.sd_info[index].rewrite_count += 1;
  }
  */
  pub(crate) fn profile_condition_start(&mut self, item: &PreEquation) {
    let mut info = match item.kind {
      Equation { .. } => &mut self.eq_info,
      Rule { .. } => &mut self.rl_info,

      SortConstraint { .. } => &mut self.mb_info,

      _ => {
        unimplemented!()
      }
    };


    let index = item.get_index_within_module();
    assert!(index >= 0, "UNDEFINED index");

    if index as usize >= info.len() {
      info.resize_with((index + 1) as usize, StatementProfile::new);
    }
    info[index as usize].condition_start_count += 1;
  }

  /*
    #[inline(always)]
    fn profile_mb_condition_start(&mut self, mb: &SortConstraint) {
      self.profile_condition_start(mb, &mut self.mb_info);
    }

    #[inline(always)]
    pub(crate) fn profile_eq_condition_start(&mut self, eq: &Equation) {
      self.profile_condition_start(eq, &mut self.eq_info);
    }

    #[inline(always)]
    pub(crate) fn profile_rl_condition_start(&mut self, rl: &Rule) {
      self.profile_condition_start(rl, &mut self.rl_info);
    }

    #[inline(always)]
    fn profile_sd_condition_start(&mut self, sdef: &StrategyDefinition) {
      self.profile_condition_start(sdef, &mut self.sd_info);
    }
  */

  pub(crate) fn profile_fragment(&mut self, pre_equation: &PreEquation, fragment_index: usize, success: bool) {
    // Check that the pre_equation's module is self.
    assert!(pre_equation.get_module().upgrade().unwrap().borrow().name == self.name);
    let index = pre_equation.get_index_within_module();
    // Check that its index is defined.
    assert_ne!(index, NONE);
    let index = index as usize;

    fn update_fragment_info(
      pre_equations: &Vec<RcPreEquation>,
      item_info: &mut Vec<StatementProfile>,
      pre_equation: &PreEquation,
      index: usize,
      fragment_index: usize,
      success: bool,
    ) {
      if index < pre_equations.len()
          && pre_equations[index as usize].as_ptr().cast_const() == std::ptr::addr_of!(*pre_equation)
      {
        item_info[index].update_fragment_info(fragment_index, success);
        return;
      }
    }

    update_fragment_info(
      &self.sort_constraints,
      &mut self.mb_info,
      pre_equation,
      index,
      fragment_index,
      success,
    );
    update_fragment_info(
      &self.equations,
      &mut self.eq_info,
      pre_equation,
      index,
      fragment_index,
      success,
    );
    update_fragment_info(
      &self.rules,
      &mut self.rl_info,
      pre_equation,
      index,
      fragment_index,
      success,
    );
    // update_fragment_info(&self.strategy_definitions, &mut self.sd_info, pre_equation, index, fragment_index,
    // success);

    // Must be a top-level pattern fragment
  }

  fn show_pre_equations(
    &self,
    pre_equations: &Vec<&PreEquation>,
    info: &Vec<StatementProfile>,
    s: &mut dyn std::io::Write,
    float_total: f64,
  ) {
    for (i, p) in info.iter().enumerate() {
      if p.condition_start_count > 0 {
        writeln!(s, "{}", pre_equations[i].repr(FormatStyle::Simple)).unwrap();
        writeln!(
          s,
          "lhs matches: {}\trewrites: {}",
          p.condition_start_count,
          crate::core::interpreter::module::profile::format_percent(p.rewrite_count, float_total)
        )
            .unwrap();
        Self::show_fragment_profile(s, &p.fragment_info, p.condition_start_count);
        writeln!(s, "").unwrap();
      } else if p.rewrite_count > 0 {
        writeln!(s, "{}", pre_equations[i].repr(FormatStyle::Simple)).unwrap();
        writeln!(s, "rewrites: {}", crate::core::interpreter::module::profile::format_percent(p.rewrite_count, float_total)).unwrap();
        writeln!(s, "").unwrap();
      }
    }
  }

  fn show_profile(&self, f: &mut dyn std::io::Write) {
    let float_total: f64;
    {
      let mut total = 0;
      for p in &self.symbol_info {
        total += p.builtin_mb_rewrite_count;
        total += p.builtin_eq_rewrite_count;
        total += p.builtin_rl_rewrite_count;
        total += p.memo_rewrite_count;
      }
      for p in &self.mb_info {
        total += p.rewrite_count;
      }
      for p in &self.eq_info {
        total += p.rewrite_count;
      }
      for p in &self.rl_info {
        total += p.rewrite_count;
      }
      // for p in &self.sd_info {
      //   total += p.rewrite_count;
      // }
      float_total = total as f64;
    }

    for (i, p) in self.symbol_info.iter().enumerate() {
      if p.builtin_mb_rewrite_count + p.builtin_eq_rewrite_count + p.builtin_rl_rewrite_count + p.memo_rewrite_count > 0
      {
        Self::show_symbol(f, self.symbols[i].clone());
        let mut g = "";
        if p.builtin_mb_rewrite_count > 0 {
          writeln!(
            f,
            "built-in mb rewrites: {}",
            crate::core::interpreter::module::profile::format_percent(p.builtin_mb_rewrite_count, float_total)
          )
              .unwrap();
          g = "\t";
        }
        if p.builtin_eq_rewrite_count > 0 {
          writeln!(
            f,
            "{}built-in eq rewrites: {}",
            g,
            crate::core::interpreter::module::profile::format_percent(p.builtin_eq_rewrite_count, float_total)
          )
              .unwrap();
          g = "\t";
        }
        if p.builtin_rl_rewrite_count > 0 {
          writeln!(
            f,
            "{}built-in rl rewrites: {}",
            g,
            crate::core::interpreter::module::profile::format_percent(p.builtin_rl_rewrite_count, float_total)
          )
              .unwrap();
          g = "\t";
        }
        if p.memo_rewrite_count > 0 {
          writeln!(
            f,
            "{}memo rewrites: {}",
            g,
            crate::core::interpreter::module::profile::format_percent(p.memo_rewrite_count, float_total)
          )
              .unwrap();
        }
        writeln!(f, "").unwrap();
      }
    }

    fn process_pre_equations(
      pre_equations: &Vec<RcPreEquation>,
      info: &Vec<StatementProfile>,
      s: &mut dyn std::io::Write,
      float_total: f64,
    ) {
      for (i, p) in info.iter().enumerate() {
        if p.condition_start_count > 0 {
          writeln!(s, "{}", pre_equations[i].borrow().repr(FormatStyle::Simple)).unwrap();
          //(p.nrRewrites) << " (" << ((100 * p.nrRewrites) / floatTotal) << "%)"
          writeln!(
            s,
            "lhs matches: {}\trewrites: {} ({:.2}%)",
            p.condition_start_count,
            p.rewrite_count,
            (100 * p.rewrite_count) as f64 / float_total
          )
              .unwrap();
          Module::show_fragment_profile(s, &p.fragment_info, p.condition_start_count);
          writeln!(s, "").unwrap();
        } else if p.rewrite_count > 0 {
          writeln!(s, "{}", pre_equations[i].borrow().repr(FormatStyle::Simple)).unwrap();
          writeln!(
            s,
            "rewrites: {} ({:.2}%)",
            p.rewrite_count,
            (100 * p.rewrite_count) as f64 / float_total
          )
              .unwrap();
          writeln!(s, "").unwrap();
        }
      }
    }

    process_pre_equations(&self.sort_constraints, &self.mb_info, f, float_total);
    process_pre_equations(&self.equations, &self.eq_info, f, float_total);
    process_pre_equations(&self.rules, &self.rl_info, f, float_total);
    // =process_pre_equations(&self.strategy_definitions, &self.sd_info, s, float_total);
  }

  fn show_symbol(f: &mut dyn std::io::Write, op: RcSymbol) {
    write!(f, "op {} : ", op.repr(FormatStyle::Simple)).unwrap();
    let arg_count = op.arity();

    for domain_component in op.sort_table().domain_components_iter() {
      write!(
        f,
        "{} ",
        domain_component
            .borrow()
            .sort(SpecialSort::Kind as i32)
            .upgrade()
            .unwrap()
            .borrow()
      )
          .unwrap();
    }

    writeln!(
      f,
      "-> {} .",
      op.sort_table()
        .range_component()
        .borrow()
        .sort(SpecialSort::Kind as i32)
        .upgrade()
        .unwrap()
        .borrow()
    )
        .unwrap();
  }

  fn show_fragment_profile(f: &mut dyn std::io::Write, fragment_info: &[FragmentProfile], mut first_count: u64) {
    let fragment_count = fragment_info.len();
    writeln!(f, "Fragment\tInitial tries\tResolve tries\tSuccesses\tFailures").unwrap();

    for (i, fragment) in fragment_info.iter().enumerate() {
      let success_count = fragment.success_count;
      let failure_count = fragment.failure_count;
      let attempt_count = success_count + failure_count;
      let backtrack_count = attempt_count - first_count;
      writeln!(
        f,
        "{}\t\t{}\t\t{}\t\t{}\t\t{}",
        i + 1,
        first_count,
        backtrack_count,
        success_count,
        failure_count
      )
          .unwrap();
      first_count = success_count; // for next fragment
    }
  }
}
