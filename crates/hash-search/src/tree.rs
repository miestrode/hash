use std::{
    mem::MaybeUninit,
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use burn::tensor::backend::Backend;
use hash_core::{board::Board, mg, repr::ChessMove};
use hash_network::model::H0;
use ringbuffer::{AllocRingBuffer, RingBuffer};

use crate::puct;

type TreeNodeIndex = usize;

#[derive(Clone, Copy)]
pub(crate) struct TreeNodeMetadata {
    pub(crate) value_sum: f32,
    pub(crate) visits: u32,
    pub(crate) probability: f32,
    chess_move: ChessMove,
}

#[derive(Clone, Copy)]
pub struct TreeNode {
    metadata: MaybeUninit<TreeNodeMetadata>,
    children_info: Option<(TreeNodeIndex, TreeNodeIndex)>,
}

impl TreeNode {
    fn is_expanded(&self) -> bool {
        self.children_info.is_some()
    }
}

pub struct Tree {
    nodes: boxcar::Vec<RwLock<TreeNode>>,
    root_index: TreeNodeIndex,
    root_board: Board,
}

#[derive(thiserror::Error, Debug)]
pub enum AdvanceTreeError {
    #[error("root is not expanded")]
    NotExpandedError,
    #[error("move is illegal in the root board")]
    IllegalMove,
}

impl Tree {
    fn get(&self, index: TreeNodeIndex) -> RwLockReadGuard<TreeNode> {
        self.nodes[index].read().expect("rwlock is poisoned")
    }

    fn get_mut(&self, index: TreeNodeIndex) -> RwLockWriteGuard<TreeNode> {
        self.nodes[index].write().expect("rwlock is poisoned")
    }

    pub fn new(board: Board) -> Tree {
        Self {
            nodes: boxcar::vec![RwLock::new(TreeNode {
                children_info: None,
                metadata: MaybeUninit::uninit()
            })],
            root_index: 0,
            root_board: board,
        }
    }

    pub fn root(&self) -> RwLockReadGuard<TreeNode> {
        self.get(self.root_index)
    }

    fn get_children_metadata<'a>(
        &'a self,
        tree_node: &'a TreeNode,
    ) -> Option<impl Iterator<Item = (TreeNodeIndex, TreeNodeMetadata)> + 'a> {
        let (start, end) = tree_node.children_info?;

        Some((start..end).map(|child_index| {
            (child_index, unsafe {
                self.get(child_index).metadata.assume_init()
            })
        }))
    }

    pub fn try_advance(&mut self, chess_move: ChessMove) -> Result<(), AdvanceTreeError> {
        let next_root_index = self
            .get_children_metadata(&self.root())
            .ok_or(AdvanceTreeError::NotExpandedError)?
            .find(|(_, child_metadata)| child_metadata.chess_move == chess_move)
            .ok_or(AdvanceTreeError::IllegalMove)?
            .0;

        self.root_index = next_root_index;

        self.root_board.make_move(chess_move).unwrap();

        Ok(())
    }

    pub fn best_move(&self) -> Option<ChessMove> {
        self.get_children_metadata(&self.root())
            .and_then(|children| {
                Some(
                    children
                        .max_by_key(|(_, child_metadata)| child_metadata.visits)?
                        .1
                        .chess_move,
                )
            })
    }

    fn select_child(&self, tree_node: &TreeNode, exploration_rate: f32) -> Option<TreeNodeIndex> {
        self.get_children_metadata(tree_node)?
            .map(|(child_index, child_metadata)| {
                (child_index, puct::puct(&child_metadata, exploration_rate))
            })
            .max_by(|(_, a), (_, b)| a.total_cmp(b))
            .map(|(child_index, _)| child_index)
    }

    pub(crate) fn expand(
        &self,
        node_index: TreeNodeIndex,
        move_probabilities: &[(f32, ChessMove)],
    ) {
        let next_node_index = self.nodes.count();
        let mut node_to_expand = self.get_mut(node_index);

        if node_to_expand.is_expanded() {
            return;
        }

        node_to_expand.children_info =
            Some((next_node_index, move_probabilities.len() + next_node_index));

        self.nodes.reserve(move_probabilities.len());

        for child in move_probabilities.iter().map(|&(probability, chess_move)| {
            RwLock::new(TreeNode {
                children_info: None,
                metadata: MaybeUninit::new(TreeNodeMetadata {
                    value_sum: 0.0,
                    visits: 0,
                    probability,
                    chess_move,
                }),
            })
        }) {
            self.nodes.push(child);
        }
    }

    pub(crate) fn select(
        &self,
        exploration_rate: f32,
        move_history: usize,
    ) -> (Box<[TreeNodeIndex]>, Box<[Board]>) {
        let mut history = AllocRingBuffer::new(move_history);
        history.push(self.root_board);

        let mut nodes = vec![];
        let mut last_node = self.get(self.root_index);

        loop {
            if last_node.is_expanded() {
                break;
            }

            nodes.push(self.select_child(&last_node, exploration_rate).unwrap());
            last_node = self.get(*nodes.last().unwrap());

            let mut current_board = *history.back().unwrap();
            current_board
                .make_move(
                    unsafe {
                        // SAFETY: Children always have initialized metadata
                        last_node.metadata.assume_init_ref()
                    }
                    .chess_move,
                )
                .unwrap();
            history.push(current_board);
        }

        (nodes.into(), history.into_iter().collect())
    }

    pub(crate) unsafe fn backpropagate(&self, value: f32, nodes: &[TreeNodeIndex]) {
        for &node in nodes {
            let mut node = self.get_mut(node);

            unsafe {
                // SAFETY: Children always have initialized metadata
                let metadata = node.metadata.assume_init_mut();

                metadata.value_sum += value;
                metadata.visits += 1;
            }
        }
    }

    pub fn grow<B: Backend>(&self, network: &H0<B>, exploration_rate: f32) {
        let (path, boards) = self.select(exploration_rate, network.move_history());
        let end_board = boards.last().unwrap();
        let network_result = &network.process(vec![&boards[..]])[0];

        self.expand(
            *path.last().unwrap(),
            &mg::gen_moves(end_board)
                .into_iter()
                .map(|chess_move| (network_result.move_probabilities[chess_move], chess_move))
                .collect::<Vec<_>>(),
        );

        // SAFETY: The path was obtained from `Tree::select`
        unsafe { self.backpropagate(network_result.value, &path) };
    }
}
