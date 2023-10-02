use crate::{Network, NetworkResult, Selector};
use arrayvec::ArrayVec;
use hash_core::{board::Board, mg};
use std::{cell::Cell, ops::Deref};

// This is essentially the maximum depth of the tree search
const HISTORY_CAPACITY: usize = 20;

pub struct Child {
    pub tree: Box<Tree>,
    pub probability: f32,
}

pub type Children = ArrayVec<Child, { mg::MOVES }>;

pub struct Tree {
    pub board: Board,
    pub value_sum: Cell<f32>,
    pub visits: Cell<u16>,
    pub children: Cell<Option<Children>>,
}

impl Tree {
    fn expanded(&self) -> bool {
        // SAFETY: We don't mutate in here anything.
        unsafe { self.children.as_ptr().as_ref().is_some() }
    }

    fn expand<S: Selector, N: Network>(&mut self, selector: &mut S, network: &N)
    where
        [(); N::MOVE_HISTORY]: Sized,
    {
        let mut node_progression = ArrayVec::<_, HISTORY_CAPACITY>::from_iter([self.deref()]);

        let node_to_expand = loop {
            let current_node = node_progression.last().unwrap();

            if !current_node.expanded() {
                break *current_node;
            }

            node_progression.push(selector.choose_child(current_node).unwrap());
        };

        let NetworkResult {
            mut value,
            move_probabilities,
        } = network.run(
            node_progression[(node_progression.len() - N::MOVE_HISTORY)..]
                .iter()
                .map(|tree| tree.board)
                .collect(),
        );

        node_to_expand.children.replace(Some(
            node_to_expand
                .board
                .gen_child_boards()
                .map(|(chess_move, board)| Child {
                    tree: Box::new(Tree {
                        board,
                        value_sum: Cell::new(0.0),
                        visits: Cell::new(0),
                        children: Cell::new(None),
                    }),
                    probability: move_probabilities.probability(chess_move),
                })
                .collect(),
        ));

        for node in node_progression.iter().rev() {
            node.visits.update(|visits| visits + 1);
            node.value_sum.update(|value_sum| value_sum + value);
            value = -value; // For the previous player, something good in the next position is bad,
                            // and something bad is good.
        }
    }
}
