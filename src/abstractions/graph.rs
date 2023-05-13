/*!

Conflict graph resolved using a naive graph coloring algorithm.


*/

use std::vec::Vec;

use crate::UNDEFINED;
use crate::abstractions::NatSet;


pub struct Graph {
  adj_sets: Vec<NatSet>,
}

impl Graph {
  pub(crate) fn new(nr_nodes: usize) -> Self {
    Self {
      adj_sets: vec![NatSet::new(); nr_nodes],
    }
  }

  pub(crate) fn insert_edge(&mut self, n1: usize, n2: usize) {
    self.adj_sets[n1].insert(n2);
    self.adj_sets[n2].insert(n1);
  }

  pub(crate) fn color(&self, coloring: &mut Vec<i32>) -> i32 {
    let nr_nodes = self.adj_sets.len();
    coloring.resize(nr_nodes, UNDEFINED);
    let mut max_color = UNDEFINED;
    for i in 0..nr_nodes {
      self.color_node(i, &mut max_color, coloring);
    }
    max_color + 1
  }

  fn color_node(&self, i: usize, max_color: &mut i32, coloring: &mut Vec<i32>) {
    if coloring[i] != UNDEFINED {
      return;
    }
    let mut used = NatSet::new();
    let adj_set: &NatSet = &self.adj_sets[i];
    for j in adj_set.iter() {
      let c = coloring[j];
      if c != UNDEFINED {
        used.insert(c as usize);
      }
    }
    let mut color = 0;
    while used.contains(color) {
      color += 1;
    }
    coloring[i] = color as i32;
    if color as i32 > *max_color {
      *max_color = color as i32;
    }
    for j in adj_set.iter() {
      self.color_node(j, max_color, coloring);
    }
  }

  fn find_components(&self, components: &mut Vec<Vec<i32>>) {
    let mut visited = NatSet::new();
    let nr_nodes = self.adj_sets.len();
    for i in 0..nr_nodes {
      if !visited.contains(i) {
        let nr_components = components.len();
        components.push(Vec::new());
        self.visit(i, &mut components[nr_components], &mut visited);
      }
    }
  }

  fn visit(&self, i: usize, component: &mut Vec<i32>, visited: &mut NatSet) {
    visited.insert(i);
    component.push(i as i32);
    let adj_set = &self.adj_sets[i];
    for j in adj_set.iter() {
      if !visited.contains(j as usize) {
        self.visit(j as usize, component, visited);
      }
    }
  }
}
