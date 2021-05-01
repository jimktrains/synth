use std::ops::{Index, IndexMut};
use std::sync::Arc;
use std::sync::RwLock;

use crate::util::Component;

pub struct BasicSeq {
    tempo: i16,
    beats: Arc<RwLock<[bool; 16]>>,
    beat_len: Arc<RwLock<[i16; 16]>>,
    gate: i16,
    trigger: i16,
    counter: u32,
    beat: i16,
    half_beat: bool,
    dummy: i16,
}

impl BasicSeq {
    pub fn new(beats: Arc<RwLock<[bool; 16]>>, beat_len: Arc<RwLock<[i16; 16]>>) -> Self {
        BasicSeq {
            tempo: 0,
            beats: beats,
            beat_len: beat_len,
            gate: 0,
            trigger: 0,
            counter: 0,
            beat: 0,
            half_beat: false,
            dummy: 0,
        }
    }
}

impl Component for BasicSeq {
    fn step(&mut self) {
        self.trigger = 0;
        if self.beats.read().unwrap()[self.beat as usize] {
            let c = self.counter.wrapping_add(1);
            if c < self.counter {
                self.counter = 0;
                self.gate = 0;
            } else {
                self.counter = c;
            }
            if (1000 * (self.beat_len.read().unwrap()[self.beat as usize] as u32)) < self.counter {
                self.gate = 0;
            }
        }
    }
    fn tick(&mut self) {
        self.beat = (self.beat + 1) % 16;
        if self.beats.read().unwrap()[self.beat as usize] {
            self.gate = i16::max_value();
            self.trigger = i16::max_value();
            self.counter = 0;
        } else {
            self.gate = 0;
            self.trigger = 0;
            self.counter = 0;
        }
    }

    fn inputs(&self) -> Vec<&'static str> {
        vec!["tempo"]
    }

    fn outputs(&self) -> Vec<&'static str> {
        vec!["gate", "trigger", "beat"]
    }
}

impl Index<&str> for BasicSeq {
    type Output = i16;

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
