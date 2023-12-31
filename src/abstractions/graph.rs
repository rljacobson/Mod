/*!

Conflict graph resolved using a naive graph coloring algorithm.

ToDo: Review if there are better options.

*/

use std::vec::Vec;

use crate::{abstractions::NatSet, UNDEFINED};


pub struct Graph {
  adj_sets: Vec<NatSet>,
}

impl Graph {
  pub(crate) fn new(node_count: usize) -> Self {
    Self {
      adj_sets: vec![NatSet::new(); node_count],
    }
  }

  pub(crate) fn insert_edge(&mut self, n1: usize, n2: usize) {
    self.adj_sets[n1].insert(n2);
    self.adj_sets[n2].insert(n1);
  }

  pub(crate) fn color(&self, coloring: &mut Vec<i32>) -> i32 {
    let node_count = self.adj_sets.len();
    coloring.resize(node_count, UNDEFINED);
    let mut max_color = UNDEFINED;
    for i in 0..node_count {
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
    let node_count = self.adj_sets.len();
    for i in 0..node_count {
      if !visited.contains(i) {
        let component_count = components.len();
        components.push(Vec::new());
        self.visit(i, &mut components[component_count], &mut visited);
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
