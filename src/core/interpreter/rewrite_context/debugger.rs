/*!

Debugger related code. This is a lot more of Maude's infrastructure than really should be in this library, but I will
need to understand the boundary between algorithms and supporting code better before I refactor.

*/

use crate::core::rewrite_context::context_attributes::ContextAttribute;
use crate::core::rewrite_context::RewritingContext;
use crate::core::interpreter::tui::DEFAULT_PROMPT;

impl RewritingContext {
  pub fn change_prompt(&mut self) {
    if self.debug_level == 0 {
      self.tui.set_prompt(DEFAULT_PROMPT.to_string());
      self.attributes.reset(ContextAttribute::DebugMode);

    } else {
      let prompt = format!("Debug({})> ", self.debug_level);
      self.tui.set_prompt(prompt);
      self.attributes.set(ContextAttribute::DebugMode);
    }
  }

}
