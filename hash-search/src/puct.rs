use crate::tree::{Child, Selector, Tree};

pub struct PuctSelector {
    exploration_rate: f32,
}

impl PuctSelector {
    pub fn new(exploration_rate: f32) -> Self {
        Self { exploration_rate }
    }

    fn puct(&self, child: &Child) -> f32 {
        self.exploration_rate * child.probability * child.tree.value_sum().sqrt()
            / (1 + child.tree.visits()) as f32
    }
}

impl Selector for PuctSelector {
    fn choose_child<'a>(&mut self, tree: &'a Tree) -> Option<&'a Tree> {
        tree.children_ref()
            .and_then(|children| {
                children
                    .iter()
                    .max_by(|child_a, child_b| self.puct(child_a).total_cmp(&self.puct(child_b)))
            })
            .map(|child| &child.tree)
    }
}
