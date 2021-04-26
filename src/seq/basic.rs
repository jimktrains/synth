use std::ops::{Index, IndexMut};

use crate::util::quarter_point_per_32nd_node;
use crate::util::Component;

pub struct BasicSeq {
    tempo: i8,
    pub beats: [bool; 16],
    pub beat_len: [i8; 16],
    gate: i8,
    trigger: i8,
    counter: u16,
    beat: i8,
    half_beat: bool,
    dummy: i8,
}

impl BasicSeq {
    pub fn new() -> Self {
        BasicSeq {
            tempo: 0,
            beats: [false; 16],
            beat_len: [64; 16],
            gate: 0,
            trigger: 0,
            counter: 0,
            beat: 0,
            half_beat: false,
            dummy: 0,
        }
    }
}

impl<'a> Component<'a> for BasicSeq {
    fn step(&mut self) {
        self.trigger = 0;
        if self.beats[self.beat as usize] {
            let c = self.counter.wrapping_add(1);
            if c < self.counter {
                self.counter = 0;
                self.gate = 0;
            } else {
                self.counter = c;
            }
            if (1000 * (self.beat_len[self.beat as usize] as u16)) < self.counter {
                self.gate = 0;
            }
        }
    }
    fn tick(&mut self) {
        self.beat = (self.beat + 1) % 16;
        if self.beats[self.beat as usize] {
            self.gate = i8::max_value();
            self.trigger = i8::max_value();
            self.counter = 0;
        } else {
            self.gate = 0;
            self.trigger = 0;
            self.counter = 0;
        }
    }

    fn inputs(&self) -> Vec<&'a str> {
        vec!["tempo"]
    }

    fn outputs(&self) -> Vec<&'a str> {
        vec!["gate", "trigger", "beat"]
    }
}

impl Index<&str> for BasicSeq {
    type Output = i8;

    fn index(&self, i: &str) -> &Self::Output {
        match i {
            "beat" => &self.beat,
            "gate" => &self.gate,
            "trigger" => &self.trigger,
            _ => &0,
        }
    }
}

impl IndexMut<&str> for BasicSeq {
    fn index_mut(&mut self, i: &str) -> &mut Self::Output {
        match i {
            "tempo" => &mut self.tempo,
            _ => &mut self.dummy,
        }
    }
}
