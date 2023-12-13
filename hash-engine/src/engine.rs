use std::{
    io::{BufRead, StdinLock},
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use hash_core::{board::Board, repr::ChessMove};
use hash_search::tree::{Child, Tree};

const UPDATE_CAPACITY: usize = 60;

struct Timings {
    time_left: Duration,
    increment: Duration,
    opponent_time_left: Duration,
    opponent_increment: Duration,
}

impl FromStr for Timings {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s
            .split(' ')
            .map(|time| time.parse::<u64>().map(Duration::from_nanos))
            .try_collect::<Vec<_>>()
            .map_err(|_| "Could not parse time integer")?;

        if parts.len() != 4 {
            Err("Timing information must consist of 4 space-separated parts")
        } else {
            Ok(Self {
                time_left: parts[0],
                increment: parts[1],
                opponent_time_left: parts[2],
                opponent_increment: parts[3],
            })
        }
    }
}

struct InitialUpdate {
    timings: Timings,
    board: Board,
}

struct RegularUpdate {
    time_left: Duration,
    opponent_time_left: Duration,
    played_move: ChessMove,
}

enum Update {
    Initial(InitialUpdate),
    Regular(RegularUpdate),
}

pub struct Engine<'a> {
    tree: Tree,
    stdin_handle: StdinLock<'a>,
    timings: Timings,
}

impl<'a> Engine<'a> {
    pub fn new(mut stdin_handle: StdinLock<'a>) -> Self {
        let mut update = String::with_capacity(UPDATE_CAPACITY);
        stdin_handle
            .read_line(&mut update)
            .expect("Failed to read update");
        update.pop(); // Remove trailing newline
        let update_parts = update.split(' ').collect::<Vec<_>>();

        Self {
            tree: Tree::new(
                Board::from_str(&update_parts[4..].join(" ")).expect("Update FEN is invalid"),
            ),
            stdin_handle,
            time_left: Duration::from_nanos(update_parts[0].parse::<u64>().unwrap()),
            increment: Duration::from_nanos(update_parts[1].parse::<u64>().unwrap()),
            opponent_time_left: Duration::from_nanos(update_parts[2].parse::<u64>().unwrap()),
            opponent_increment: Duration::from_nanos(update_parts[3].parse::<u64>().unwrap()),
        }
    }

    fn start_search(tree: Tree) -> impl FnOnce() -> thread::Result<Tree> {
        let stop_search = Arc::new(AtomicBool::new(false));

        let stop_search_clone = stop_search.clone();
        let join_handle = thread::spawn(move || hash_search::search(tree, stop_search_clone));

        move || {
            stop_search.store(true, Ordering::Relaxed);
            join_handle.join()
        }
    }

    fn search(mut self, time: Duration) -> Self {
        let stop_search = Self::start_search(self.tree);

        // It's fine to block command handling because the SCCS spec doesn't allow for commands to
        // arrive while making a move
        thread::sleep(time);

        self.tree = stop_search().expect("Could not join search thread");

        self
    }

    fn advance_tree(mut self, advancing_move: ChessMove) -> Self {
        self.tree = self
            .tree
            .children()
            .unwrap()
            .into_iter()
            .find(|Child { chess_move, .. }| *chess_move == advancing_move)
            .unwrap()
            .tree;
        self
    }

    // When pondering, instead of having a fixed time-frame, we want to naturally stop as soon as
    // the opponent has made their move. This means that while the search thread runs, we must
    // check for updates as to wether the opponent made their move
    fn ponder(mut self) -> Self {
        let stop_search = Self::start_search(self.tree);

        let mut update = String::with_capacity(UPDATE_CAPACITY);
        self.stdin_handle
            .read_line(&mut update)
            .expect("Failed to read update");
        update.pop(); // Remove trailing newline
        let update_parts = update.split(' ').collect::<Vec<_>>();

        self.time_left = Duration::from_nanos(update_parts[0].parse::<u64>().unwrap());
        self.opponent_time_left = Duration::from_nanos(update_parts[1].parse::<u64>().unwrap());

        let played_move =
            ChessMove::from_str(update_parts[2]).expect("Received update move is invalid");

        self.tree = stop_search().expect("Couldn't join searching thread");
        self = self.advance_tree(played_move);

        self
    }

    fn send_move(self) -> Self {
        let best_move = self.tree.best_move();
        println!("{}", best_move);
        self.advance_tree(best_move)
    }

    pub fn run(mut self) {
        loop {
            self = self.search(Duration::from_secs(5));
            self = self.send_move();
            self = self.ponder();
        }
    }
}
