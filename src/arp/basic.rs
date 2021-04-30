use std::ops::Add;
use std::ops::{Index, IndexMut};

use crate::util::Component;
use std::convert::From;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Scale {
    CM,
}

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

// Converts a note from its semitone offset from A
impl From<u8> for TtetNote {
    fn from(v: u8) -> Self {
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

// Converts a note into its semitone offset from A
impl From<&TtetNote> for u8 {
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

impl Add<i8> for TtetNote {
    type Output = Self;

    fn add(self, other: i8) -> Self {
        let v: u8 = (&self).into();
        let x: u8 = ((v as i16 + (other as i16)) % 12) as u8;
        x.into()
    }
}

impl TtetNote {
    pub fn to_freq_cv(&self, octave: u8) -> i8 {
        let v: u8 = self.into();
        if octave < 4 {
            (69 + (((octave as i8) - 3) * (12 - v) as i8)) as i8
        } else {
            (69 + ((octave - 3) * (v))) as i8
        }
    }
    pub fn major_scale(&self) -> [TtetNote; 7] {
        let v: u8 = self.into();

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
    gate_in: i8,
    trigger_in: i8,
    counter: usize,
    pub notes: [TtetNote; 7],
    note_cv_out: i8,
    pub octave: u8,
    dummy: i8,
}

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
    fn tick(&mut self) {
        //if self.trigger_in != 0 {
        //self.counter = (self.counter + 1) % self.notes.len();
        //self.note_cv_out = self.notes[self.counter as usize].to_freq_cv(self.octave);
        //}
        //println!("{} {}", self.trigger_in, self.note_cv_out);
    }
    fn step(&mut self) {
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
    type Output = i8;

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
