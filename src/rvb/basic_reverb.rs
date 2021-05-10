use std::ops::{Index, IndexMut};

use crate::util::Component;
use crate::util::RATE;

pub struct BasicReverb {
    buffer: [i16; RATE as usize],
    pub delay: i16,
    cv_in: i16,
    out_cv: i16,
    dummy: i16,
    counter: usize,
    block_counter: usize,
}

impl BasicReverb {
    pub fn new() -> BasicReverb {
        BasicReverb {
            buffer: [0i16; RATE as usize],
            delay: 0i16,
            cv_in: 0i16,
            out_cv: 0i16,
            dummy: 0i16,
            counter: 0usize,
            block_counter: 0usize,
        }
    }
}

impl Component for BasicReverb {
    fn tick(&mut self) {}
    fn step(&mut self) {
        let delay = self.delay.abs() as usize;

        self.buffer[self.counter] = self.cv_in;

        self.out_cv = 0;
        let j = self.counter;
        self.out_cv = self
            .out_cv
            //.saturating_add(self.buffer[(j - (9 * delay)) % (9 * delay)] / 32)
            //.saturating_add(self.buffer[(j - (8 * delay)) % (9 * delay)] / 32)
            //.saturating_add(self.buffer[(j - (7 * delay)) % (9 * delay)] / 32)
            //.saturating_add(self.buffer[(j - (6 * delay)) % (9 * delay)] / 32)
            //.saturating_add(self.buffer[(j - (5 * delay)) % (9 * delay)] / 8)
            //.saturating_add(self.buffer[(j - (4 * delay)) % (9 * delay)] / 8)
            //.saturating_add(self.buffer[(j - (3 * delay)) % (9 * delay)] / 8)
            //.saturating_add(self.buffer[(j - (2 * delay)) % (9 * delay)] / 8)
            .saturating_add(self.buffer[(j.wrapping_sub((1 * delay))) % (9 * delay)] / 8)
            .saturating_add(self.buffer[(j.wrapping_sub((0 * delay))) % (9 * delay)]);

        self.counter = (self.counter + 1) % (9 * delay);
    }

    fn inputs(&self) -> Vec<&'static str> {
        vec!["cv_in"]
    }

    fn outputs(&self) -> Vec<&'static str> {
        vec!["out"]
    }
}

impl Index<&str> for BasicReverb {
    type Output = i16;

    fn index(&self, i: &str) -> &Self::Output {
        match i {
            "out" => &self.out_cv,
            _ => &0,
        }
    }
}

impl IndexMut<&str> for BasicReverb {
    fn index_mut(&mut self, i: &str) -> &mut Self::Output {
        match i {
            "cv_in" => &mut self.cv_in,
            _ => &mut self.dummy,
        }
    }
}
