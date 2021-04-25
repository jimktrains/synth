use crate::fixed::{SQ16_0, SQ1_31, SQ32_0};
use std::ops::{Index, IndexMut};

// Points per second.
pub const RATE: u16 = 44_100;

pub const SEC_PER_TICK: SQ1_31 = SQ32_0::inv_u16(RATE);

pub const WAVE_TABLE_SAMPLES_PER_CYCLE: u16 = 1024;

// Wave Table Samples per point.
pub const WAVE_TABLE_SAMPLES_PER_POINT: f64 = (WAVE_TABLE_SAMPLES_PER_CYCLE as f64) / (RATE as f64);

pub fn quarter_point_per_32nd_node(tempo: f64) -> u16 {
    let quarter_note_len = 60. / tempo;
    let thirtysecond_note_len = quarter_note_len / 8.;
    let pp32 = (RATE as f64) * thirtysecond_note_len;
    let pp32_4 = (4 as f64 * pp32) as u16;

    pp32_4
}

pub fn cv_to_64th_wavetable_increment(cv: i8) -> (f64, u16, f64) {
    let midi_note_index = cv as f64;
    let midi_exp = (midi_note_index - 69.) / 12.;
    let delta = (2f64).powf(midi_exp);
    let f = 440. * delta;
    let ipc = WAVE_TABLE_SAMPLES_PER_POINT * f;
    let ipc_64 = (64. * ipc) as u16;
    let e = ((ipc_64 as f64) / (64. * ipc)) - 1.;

    (f, ipc_64, e)
}

pub trait Component<'a>: Index<&'a str, Output = i8> + IndexMut<&'a str> {
    fn step(&mut self);
    fn inputs(&self) -> Vec<&'a str>;
    fn outputs(&self) -> Vec<&'a str>;
}
