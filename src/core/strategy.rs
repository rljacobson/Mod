/*!

The execution strategy.

*/

use crate::core::NatSet;

#[derive(Default)]
pub struct Strategy {
  pub is_standard: bool,
  pub unevaluated_arguments: bool,
  pub strategy: Vec<i32>,
  pub eager: NatSet,
  pub evaluated: NatSet,
  pub frozen: NatSet,
}

impl Strategy {
  pub fn get_frozen(&self) -> &NatSet {
    &self.frozen
  }

  pub fn get_strategy(&self) -> &Vec<i32> {
    &self.strategy
  }

  pub fn standard_strategy(&self) -> bool {
    self.is_standard
  }

  pub fn unevaluated_arguments(&self) -> bool {
    self.unevaluated_arguments
  }

  pub fn eager_argument(&self, arg_nr: u32) -> bool {
    self.is_standard || self.eager.contains(arg_nr as usize)
  }

  pub fn evaluated_argument(&self, arg_nr: i32) -> bool {
    self.is_standard || self.evaluated.contains(arg_nr as usize)
  }
}
