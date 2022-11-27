/*!



 */


use crate::theory::dag_node::DagNode;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum RedBlackNodeFlags {
  Color = 0,

  Marked = 6,
  Collect = 7,
}


pub struct RedBlackNode {
  pub dag_node        : Box<dyn DagNode>,
  pub multiplicity    : u32,
  pub max_multiplicity: u32,
  pub left            : Option<Box<RedBlackNode>>,
  pub right           : Option<Box<RedBlackNode>>,
  pub flags           : u8
}

impl RedBlackNode {
  pub fn get_child(&self, sign: u32) -> Option<&Box<RedBlackNode>> {
    if sign < 0 {
      if let Some(&left) = self.left {
        return Some(left);
      }
    } else {
      if let Some(&right) = self.right {
        return Some(right);
      }
    }

    return None;
  }
}



#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    assert_eq!(2 + 2, 4);
  }
}
