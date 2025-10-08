use std::collections::VecDeque;

use rand::RngCore;
use rand::seq::SliceRandom;

use crate::Tetromino;

pub struct Queue {
    queue: VecDeque<Tetromino>,
    rng: Box<dyn RngCore>,
}

impl Queue {
    pub fn new(rng: Box<dyn RngCore>) -> Self {
        Self {
            queue: VecDeque::new(),
            rng,
        }
    }

    pub fn pop_piece(&mut self) -> Tetromino {
        let piece = self.nth_next_piece(0);
        self.queue.pop_front();
        piece
    }

    pub fn nth_next_piece(&mut self, i: usize) -> Tetromino {
        loop {
            match self.queue.get(i) {
                Some(&piece) => break piece,
                None => self.request_more_pieces(),
            }
        }
    }

    pub fn next_pieces(&mut self) -> impl Iterator<Item = Tetromino> {
        (0..).map(|i| self.nth_next_piece(i))
    }

    fn request_more_pieces(&mut self) {
        use Tetromino::*;

        let mut bag = [I, J, L, O, S, T, Z];
        bag.shuffle(&mut self.rng);
        self.queue.extend(bag);
    }
}
