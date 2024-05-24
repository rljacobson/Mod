/*!

The Symbol trait and its concrete implementations BinarySymbol and AssociativeSymbol.

In Maude, a Symbol subclasses
  RuleTable,
  NamedEntity,
  LineNumber,
  SortTable,
  SortConstraintTable,
  EquationTable,
  Strategy,
  MemoTable

In Maude the relationship between symbols and sort computations are strange. Symbols compute sort
information for their owning parent terms or DAG nodes. These methods more naturally belong to the
parent, not the symbol, so that's what we do. Here's a summary.

| Maude object | Method                                       | Mod object                              | Description                          |
| ------------ | -------------------------------------------- | --------------------------------------- | ------------------------------------ |
| `Symbol`     | `fillInSortInfo(Term* subject)`              | `Term::fill_in_sort_info(&mut self)`    | Virtual in base class `SortTable`    |
| `Symbol`     | `computeBaseSort(subject: &mut dyn DagNode)` | `DagNode::compute_base_sort(&mut self)` | Pure virtual, defined in each theory |
|              |                                              |                                         |                                      |
|              |                                              |                                         |                                      |
|              |                                              |                                         |                                      |

*/

use std::{
  any::Any,
  cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd},
  fmt::{Debug, Display, Formatter},
  rc::Rc,
};

use tiny_logger::{log, Channel};

use crate::{
  abstractions::{IString, Set},
  core::{
    format::{FormatStyle, Formattable},
    interpreter::module::item::ModuleItem,
    module::WeakModule,
    pre_equation::{sort_constraint_table::SortConstraintTable, PreEquationKind, RcPreEquation},
    rewrite_context::RewritingContext,
    sort::SortTable,
    Strategy,
  },
  theory::{MaybeSubproblem, RcDagNode, RcTerm},
  NONE,
  UNDEFINED,
};
use crate::core::rewrite_context::RcRewritingContext;

pub type RcSymbol = Rc<dyn Symbol>;
pub type SymbolSet = Set<dyn Symbol>;

/*
One way to deal with a lack of trait data members is to have a struct containing the shared members and then
either
  1. have a macro that implements the getters and setters, or
  2. have a trait-level getter for the struct that is implemented in every implementor, and have
     shared-implementation at the trait level by using the getter in the `impl Trait`.
We choose the second option.
*/

pub struct SymbolMembers {
  /// `NamedEntity` members
  pub name: IString,

  // `Symbol` members

  // ToDo: Can the `IString` value be used as the `hash_value`?
  // Unique integer for comparing symbols, also called order.
  // In Maude, the `order` has lower bits equal to the value of an integer that is incremented every time a symbol is
  // created and upper 8 bits (bits 24..32) equal to the arity.
  pub hash_value:        u32,
  pub unique_sort_index: i32, // Slow Case: 0, Fast Case: -1, positive for symbols that only produce an unique sort
  pub match_index:       u32, // For fast matching
  pub arity:             u32,
  pub memo_flag:         bool,

  /// `SortConstraintTable` members.
  /// It is Maude's Symbol superclass, but we use composition instead.
  pub sort_constraint_table: SortConstraintTable,

  /// `SortTable` is Maude's Symbol superclass, but we use composition instead.
  pub sort_table: SortTable,

  // `ModuleItem`
  pub(crate) index_within_parent_module: i32,
  pub(crate) parent_module:              WeakModule,

  // `Strategy`
  pub(crate) strategy: Strategy,

  // `EquationTable`
  equations: Vec<RcPreEquation>,
}

impl SymbolMembers {
  pub fn new(name: IString, arity: u32, memo_flag: bool) -> SymbolMembers {
    let mut new_symbol = SymbolMembers {
      name,
      hash_value: 0,
      unique_sort_index: UNDEFINED,
      match_index: 0,
      arity,
      memo_flag,
      sort_constraint_table: Default::default(),
      sort_table: Default::default(),
      index_within_parent_module: NONE,
      parent_module: Default::default(),
      strategy: Strategy::default(),
      equations: vec![],
    };
    // The only time the hash is computed.
    new_symbol.hash_value = new_symbol.compute_hash();

    new_symbol
  }

  fn compute_hash(&self) -> u32 {
    // In Maude, the hash value is the number (chronological order of creation) of the symbol OR'ed
    // with (arity << 24). Here we swap the "number" with the hash of the IString as defined by the
    // IString implementation.

    // ToDo: This… isn't great, because the hash is 32 bits, not 24, and isn't generated in numeric
    //       order. However, it still produces a total order on symbols in which symbols are ordered first
    //       by arity and then arbitrarily (by hash). Ordering by insertion order is just as arbitrary, so
    //       it should be ok.
    IString::get_hash(&self.name) | (self.arity << 32) // Maude: self.arity << 24
  }

  // region EquationTable methods

  fn apply_replace(&mut self, subject: RcDagNode, context: &mut RewritingContext) -> bool {
    for eq in &self.equations {
      // Destructure the equation
      if let PreEquationKind::Equation {
        rhs_term,
        ref mut rhs_builder,
        fast_variable_count,
      } = &eq.borrow().kind
      {
        if *fast_variable_count >= 0 {
          // Fast case
          context.substitution.clear_first_n(*fast_variable_count as usize);
          if let Some(lhs_automaton) = &eq.borrow().lhs_automaton {
            if let (true, sp) = lhs_automaton
              .borrow_mut()
              .match_(subject.clone(), &mut context.substitution)
            {
              if sp.is_some() || context.trace_status() {
                self.apply_replace_slow_case(subject.clone(), eq.clone(), sp, context);
              }
              if
              /* extension_info.is_none() || extension_info.matched_whole() */
              true {
                rhs_builder.replace(subject.clone(), &mut context.substitution); // Implement get_rhs_builder and
                                                                                 // replace methods
              } else {
                // ToDo: Implement `partial_replace` on `RcDagNode`, or else determine what replaces it.
                subject.borrow_mut().partial_replace(
                  // ToDo: Implement get_rhs_builder and construct methods
                  rhs_builder.construct(&mut context.substitution).unwrap(),
                  // extension_info,
                );
              }
              context.eq_count += 1;
              context.finished();
              // Memory::ok_to_collect_garbage();
              return true;
            }
          } else {
            unreachable!("LHS automaton expected. This is a bug.")
          }
        } else {
          // General case
          let nr_variables = eq.borrow().variable_info.protected_variable_count();
          context.substitution.clear_first_n(nr_variables as usize);
          if let Some(lhs_automaton) = eq.borrow().lhs_automaton.clone() {
            if let (true, sp) = lhs_automaton.borrow_mut().match_(
              subject.clone(),
              &mut context.substitution,
              // extension_info,
            ) {
              self.apply_replace_slow_case(subject.clone(), eq.clone(), sp, context /* extension_info */);
            }
          }
          context.finished();
          // MemoryCell::ok_to_collect_garbage(); // Implement ok_to_collect_garbage
        }
      } else {
        unreachable!("Destructured a nonequation as an equation. This is a bug.");
      };
    }
    false
  }

  fn apply_replace_slow_case(
    &mut self,
    subject: RcDagNode,
    eq: RcPreEquation,
    sp: MaybeSubproblem,
    context: RcRewritingContext,
    /* extension_info: &mut ExtensionInfo, */
  ) -> bool {
    #[cfg(debug_assertions)]
    log(
      Channel::Debug,
      5,
      format!(
        "EquationTable::applyReplace() slowCase:\nsubject = {}\neq = {}",
        subject.borrow(),
        eq.borrow().repr(FormatStyle::Simple)
      )
      .as_str(),
    );

    if sp.is_none() || sp.unwrap().solve(true, context) {
      if !eq.borrow().has_condition() || eq.borrow_mut().check_condition(subject, context.clone(), sp) {
        let trace = RewritingContext::get_trace_status();
        if trace {
          context.trace_pre_eq_rewrite(subject.clone(), eq, RewritingContext::NORMAL);
          if context.trace_abort() {
            context.finished();
            return false;
          }
        }
        // Destructure the equation
        if let PreEquationKind::Equation {
          rhs_term,
          ref mut rhs_builder,
          fast_variable_count,
        } = &eq.borrow().kind
        {
          // ToDo: Implement extension_info
          if
          /* extension_info.is_none() || extension_info.unwrap().matched_whole() */
          true {
            rhs_builder.replace(subject.clone(), &mut context.substitution);
          } else {
            // ToDo: Implement `partial_replace` on `RcDagNode`, or else determine what replaces it.
            //       Apparently only used by AU, ACU, S theories.
            subject.borrow_mut().partial_replace(
              rhs_builder.construct(&mut context.substitution),
              // extension_info.unwrap(),
            );
          }
          context.increment_eq_count();
          if trace {
            context.trace_post_eq_rewrite(subject.clone());
          }
          context.finished();
          // MemoryCell::ok_to_collect_garbage(); // Implement ok_to_collect_garbage if necessary
          return true;
        }
      }
    }
    false
  }
  // endregion EquationTable methods
}

pub trait Symbol {
  // region Member Getters and Setters
  /// Trait level access to members for shared implementation
  fn symbol_members(&self) -> &SymbolMembers;
  fn symbol_members_mut(&mut self) -> &mut SymbolMembers;

  #[inline(always)]
  fn name(&self) -> IString {
    self.symbol_members().name.clone()
  }

  /// Same as `get_order` or `get_hash_value`, used for "semantic hash".
  ///
  /// The semantics of a symbol are not included in the hash itself, as symbols are unique names by definition.
  #[inline(always)]
  fn semantic_hash(&self) -> u32 {
    self.symbol_members().hash_value
  }

  #[inline(always)]
  fn get_index_within_module(&self) -> usize {
    self.symbol_members().index_within_parent_module as usize
  }

  #[inline(always)]
  fn arity(&self) -> u32 {
    self.symbol_members().arity
  }

  #[inline(always)]
  fn sort_constraint_table(&self) -> &SortConstraintTable {
    &self.symbol_members().sort_constraint_table
  }

  #[inline(always)]
  fn sort_constraint_table_mut(&mut self) -> &mut SortConstraintTable {
    &mut self.symbol_members_mut().sort_constraint_table
  }

  #[inline(always)]
  fn index_within_parent(&self) -> i32 {
    self.symbol_members().index_within_parent_module
  }
  // endregion

  // Note: `compute_base_sort` is a method of *Symbol in Maude.
  // However, it takes its owning DagNode as a parameter, subject.
  // fn compute_base_sort(&self, subject: &mut dyn DagNode);

  #[inline(always)]
  fn sort_constraint_free(&self) -> bool {
    self.sort_constraint_table().sort_constraint_free()
  }

  #[inline(always)]
  fn sort_table(&self) -> &SortTable {
    &self.symbol_members().sort_table
  }

  #[inline(always)]
  fn strategy(&self) -> &Strategy {
    &self.symbol_members().strategy
  }

  #[inline(always)]
  fn strategy_mut(&mut self) -> &mut Strategy {
    &mut self.symbol_members_mut().strategy
  }

  #[inline(always)]
  fn compare(&self, other: &dyn Symbol) -> Ordering {
    // This is just std::Ord::cmp(self, other)
    // Ord::cmp(&self, other)
    self.semantic_hash().cmp(&other.semantic_hash())
  }

  fn as_any(&self) -> &dyn Any;

  #[inline(always)]
  fn is_variable(&self) -> bool {
    false
  }

  #[inline(always)]
  fn is_memoized(&self) -> bool {
    self.symbol_members().memo_flag
  }

  fn rewrite(&mut self, subject: RcDagNode, context: &mut RewritingContext) -> bool;
}

//  region Order and Equality impls
impl PartialOrd for dyn Symbol {
  #[inline(always)]
  fn partial_cmp(&self, other: &dyn Symbol) -> Option<Ordering> {
    let result = self.semantic_hash().cmp(&other.semantic_hash());
    Some(result)
  }
}

impl Ord for dyn Symbol {
  #[inline(always)]
  fn cmp(&self, other: &dyn Symbol) -> Ordering {
    self.semantic_hash().cmp(&other.semantic_hash())
  }
}

impl Eq for dyn Symbol {}

impl PartialEq for dyn Symbol {
  #[inline(always)]
  fn eq(&self, other: &dyn Symbol) -> bool {
    self.semantic_hash() == other.semantic_hash()
  }
}
// endregion


// Every `Symbol` is a `ModuleItem`
impl ModuleItem for dyn Symbol {
  #[inline(always)]
  fn get_index_within_module(&self) -> i32 {
    self.symbol_members().index_within_parent_module
  }

  #[inline(always)]
  fn set_module_information(&mut self, module: WeakModule, index_within_module: i32) {
    self.symbol_members_mut().parent_module = module;
    self.symbol_members_mut().index_within_parent_module = index_within_module;
  }

  #[inline(always)]
  fn get_module(&self) -> WeakModule {
    self.symbol_members().parent_module.clone()
  }
}

impl Formattable for dyn Symbol {
  fn repr(&self, _style: FormatStyle) -> String {
    self.symbol_members().name.to_string()
  }
}


impl Debug for dyn Symbol {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.symbol_members().name)
  }
}


/*
Deriving Traits:
BinarySymbol
*/

pub trait BinarySymbol: Symbol {
  fn get_identity(&self) -> Option<RcTerm>;
  fn get_identity_dag(&self) -> Option<RcDagNode>;
}
