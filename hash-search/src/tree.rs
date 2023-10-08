use crate::{Network, NetworkResult, Selector};
use hash_core::{board::Board, mg, repr::Move};
use std::{cell::Cell, ops::Deref};

pub struct Child {
    pub tree: Tree,
    pub probability: f32,
}

impl Child {
    pub fn new(board: Board, probability: f32) -> Self {
        Self {
            tree: Tree::new(board),
            probability,
        }
    }
}

pub type Children = Box<[Child]>;

pub struct Tree {
    board: Board,
    value_sum: Cell<f32>,
    visits: Cell<u16>,
    children: Cell<Option<Children>>,
}

impl Tree {
    pub fn new(board: Board) -> Self {
        Self {
            board,
            value_sum: Cell::new(0.0),
            visits: Cell::new(0),
            children: Cell::new(None),
        }
    }

    pub fn best_move(&self) -> Move {
        self.children()
            .unwrap()
            .iter()
            .zip(mg::gen_moves(&self.board))
            .max_by_key(|(child, _)| child.tree.visits())
            .unwrap()
            .1
    }

    pub fn value_sum(&self) -> f32 {
        self.value_sum.get()
    }

    pub fn visits(&self) -> u16 {
        self.visits.get()
    }

    pub fn children(&self) -> Option<&Children> {
        // SAFETY: Operations that modify the children require unique access
        unsafe { self.children.as_ptr().as_ref().unwrap() }.as_ref()
    }

    fn expanded(&self) -> bool {
        self.children().is_some()
    }

    pub fn expand<S: Selector, N: Network>(&mut self, selector: &mut S, network: &N) {
        let mut node_progression = vec![self.deref()];

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
            node_progression[node_progression
                .len()
                .saturating_sub(network.maximum_boards_expected())..]
                .iter()
                .map(|tree| tree.board)
                .collect(),
        );

        node_to_expand.children.replace(Some(
            node_to_expand
                .board
                .gen_child_boards()
                .map(|(chess_move, board)| {
                    Child::new(board, move_probabilities.probability(chess_move))
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
