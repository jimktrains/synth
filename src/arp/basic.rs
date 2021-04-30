use std::ops::Add;
use std::ops::{Index, IndexMut};

use crate::util::Component;
use std::convert::From;

// Twelve-tone equal temper notes, a.k.a. piano tuning.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TtetNote {
    Ab,
    A,
    As,
    Bb,
    B,
    C,
    Cs,
    Db,
    D,
    Ds,
    Eb,
    E,
    F,
    Fs,
    Gb,
    G,
    Gs,
}

// Converts a note from its semitone offset from A.
impl From<u16> for TtetNote {
    fn from(v: u16) -> Self {
        let v = v % 12;

        match v {
            0 => TtetNote::A,
            1 => TtetNote::As,
            2 => TtetNote::B,
            3 => TtetNote::C,
            4 => TtetNote::Cs,
            5 => TtetNote::D,
            6 => TtetNote::Ds,
            7 => TtetNote::E,
            8 => TtetNote::F,
            9 => TtetNote::Fs,
            10 => TtetNote::G,
            11 => TtetNote::Gs,
            _ => TtetNote::Gs,
        }
    }
}

// Converts a note into its semitone offset from A.
impl From<&TtetNote> for u16 {
    fn from(v: &TtetNote) -> Self {
        match v {
            TtetNote::A => 0,
            TtetNote::As => 1,
            TtetNote::Bb => 1,
            TtetNote::B => 2,
            TtetNote::C => 3,
            TtetNote::Cs => 4,
            TtetNote::Db => 4,
            TtetNote::D => 5,
            TtetNote::Ds => 6,
            TtetNote::Eb => 6,
            TtetNote::E => 7,
            TtetNote::F => 8,
            TtetNote::Fs => 9,
            TtetNote::Gb => 9,
            TtetNote::G => 10,
            TtetNote::Gs => 11,
            TtetNote::Ab => 11,
        }
    }
}

impl Add<i16> for TtetNote {
    type Output = Self;

    fn add(self, other: i16) -> Self {
        let v: u16 = (&self).into();
        let x: u16 = ((v as i32 + (other as i32)) % 12) as u16;
        x.into()
    }
}

impl TtetNote {
    // NB: I need to redo how I interpret control voltages <-> midi note index.
    pub fn to_freq_cv(&self, octave: u16) -> i16 {
        let v: u16 = self.into();
        if octave < 4 {
            (69 + (((octave as i16) - 3) * (12 - v) as i16)) as i16
        } else {
            (69 + ((octave - 3) * (v))) as i16
        }
    }

    // I know very little about music. I think we can construct all scales
    // based on the root and given semitone intervals.
    pub fn major_scale(&self) -> [TtetNote; 7] {
        let v: u16 = self.into();

        [
            (v + 0).into(),
            (v + 2).into(),
            (v + 4).into(),
            (v + 5).into(),
            (v + 7).into(),
            (v + 9).into(),
            (v + 11).into(),
        ]
    }
}

pub struct BasicArp {
    gate_in: i16,
    trigger_in: i16,
    counter: usize,
    pub notes: [TtetNote; 7],
    note_cv_out: i16,
    pub octave: u16,
    dummy: i16,
}

// A basic arpeggiator that simply cycles thought 7 notes, one per trigger.
impl BasicArp {
    pub fn new() -> BasicArp {
        BasicArp {
            gate_in: 0,
            trigger_in: 0,
            counter: 0,
            notes: [TtetNote::Eb; 7],
            note_cv_out: 0,
            octave: 4,
            dummy: 0,
        }
    }
}

impl<'a> Component<'a> for BasicArp {
    fn tick(&mut self) {}
    fn step(&mut self) {
        // NB I'm not sure why this isn't working in the trigger, but
        // I can track that down later.
        if self.trigger_in != 0 {
            self.counter = (self.counter + 1) % self.notes.len();
            self.note_cv_out = self.notes[self.counter as usize].to_freq_cv(self.octave);
        }
    }
    fn inputs(&self) -> Vec<&'a str> {
        vec!["trigger_in", "gate_in"]
    }

    fn outputs(&self) -> Vec<&'a str> {
        vec!["note_cv_out"]
    }
}

impl Index<&str> for BasicArp {
    type Output = i16;

    fn index(&self, i: &str) -> &Self::Output {
        match i {
            "note_cv_out" => &self.note_cv_out,
            _ => &0,
        }
    }
}

impl IndexMut<&str> for BasicArp {
    fn index_mut(&mut self, i: &str) -> &mut Self::Output {
        match i {
            "gate_in" => &mut self.gate_in,
            "trigger_in" => &mut self.trigger_in,
            _ => &mut self.dummy,
        }
    }
}
