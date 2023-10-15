use crate::network::{Network, NetworkResult};
use hash_core::{board::Board, mg, repr::Move};
use std::{cell::Cell, ops::Deref};

pub struct Child {
    pub tree: Tree,
    pub probability: f32,
    pub chess_move: Move,
}

impl Child {
    pub fn new(board: Board, probability: f32, chess_move: Move) -> Self {
        Self {
            tree: Tree::new(board),
            probability,
            chess_move,
        }
    }
}

pub struct Tree {
    pub board: Board,
    value_sum: Cell<f32>,
    visits: Cell<u16>,
    children: Cell<Option<Box<[Child]>>>,
}

pub trait Selector {
    fn choose_child<'a>(&mut self, tree: &'a Tree) -> Option<&'a Tree>;
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
        self.children_ref()
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

    pub fn children(self) -> Option<Box<[Child]>> {
        self.children.into_inner()
    }

    pub fn children_ref(&self) -> Option<&[Child]> {
        // SAFETY: Operations that modify the children require unique access
        unsafe { self.children.as_ptr().as_ref().unwrap() }
            .as_ref()
            .map(|child| child.as_ref())
    }

    fn expanded(&self) -> bool {
        self.children_ref().is_some()
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
                    Child::new(
                        board,
                        move_probabilities.get_probability(chess_move),
                        chess_move,
                    )
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
