/*!

Textual user interface. Maude has IO_Manager that handles most of the functionality of this module. Some of Maude's UI
stuff is distributed throughout other bits of its code. The TUI should probably be owned by the interpreter… or vice
versa.

*/

use std::io::{Stderr, Stdin, Stdout};

pub(crate) static DEFAULT_PROMPT: &'static str = "Maude> ";

#[derive(Debug)]
pub struct TUI {
  prompt_format: String,

  // std_in : Stdin,
  // std_out: Stdout,
  // std_err: Stderr,
  // log_out: Stdout,
}

impl TUI {
  pub fn set_prompt(&mut self, prompt: String) {
    self.prompt_format = prompt;
  }
}

impl Default for TUI {
  fn default() -> Self {
    TUI{
      prompt_format: DEFAULT_PROMPT.to_string()
    }
  }
}
