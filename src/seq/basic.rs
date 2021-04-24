use std::ops::{Index, IndexMut};

use crate::util::quarter_point_per_32nd_node;
use crate::util::Component;

pub struct BasicSeq {
    tempo: i8,
    pub beats: [bool; 16],
    gate: i8,
    trigger: i8,
    counter: u16,
    beat: u8,
    half_beat: bool,
    dummy: i8,
}

impl BasicSeq {
    pub fn new() -> Self {
        BasicSeq {
            tempo: 0,
            beats: [false; 16],
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
        let ts = quarter_point_per_32nd_node(self.tempo as f64);
        if self.trigger != 0 {
            self.trigger = 0;
        }
        if self.counter == 0 || self.counter > ts {
            if self.counter != 0 {
                self.counter -= ts;
            }
            if self.half_beat {
                self.gate = 0;
                self.beat = (self.beat + 1) % self.beats.len() as u8;
            } else {
                if self.beats[self.beat as usize] {
                    self.gate = i8::max_value();
                    self.trigger = i8::max_value();
                }
            }
            self.half_beat = !self.half_beat;
        }
        self.counter += 4;
    }

    fn inputs(&self) -> Vec<&'a str> {
        vec!["tempo"]
    }

    fn outputs(&self) -> Vec<&'a str> {
        vec!["gate", "trigger"]
    }
}

impl Index<&str> for BasicSeq {
    type Output = i8;

    fn index(&self, i: &str) -> &Self::Output {
        match i {
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
