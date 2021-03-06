use std::ops::{Index, IndexMut};

// Points per second.
pub const RATE: u32 = 44_100;

// pub const SEC_PER_TICK: SQ1_31 = SQ32_0::inv_u32(RATE);

pub const WAVE_TABLE_SAMPLES_PER_CYCLE: u32 = 600;
pub const WAVE_TABLE_SAMPLES_PER_CYCLE_FACTOR: u32 = 256;

// Wave Table Samples per point.
pub const WAVE_TABLE_SAMPLES_PER_POINT: f64 = (RATE as f64) / (WAVE_TABLE_SAMPLES_PER_CYCLE as f64);

pub fn cv_to_64th_wavetable_increment(cv: i16) -> (f64, u32, f64) {
    let midi_note_index = cv as f64;
    let midi_exp = (midi_note_index - 69.) / 12.;
    let delta = (2f64).powf(midi_exp);
    let f = 440. * delta;
    let ipc = WAVE_TABLE_SAMPLES_PER_POINT / f;
    let ipc_64 = ((WAVE_TABLE_SAMPLES_PER_CYCLE_FACTOR as f64) * ipc) as u32;
    let e = ((ipc_64 as f64) / ((WAVE_TABLE_SAMPLES_PER_CYCLE_FACTOR as f64) * ipc)) - 1.;

    (f, ipc_64, e)
}

pub trait Component:
    Index<&'static str, Output = i16> + IndexMut<&'static str> + Send + Sync
{
    fn step(&mut self);
    fn tick(&mut self);
    fn inputs(&self) -> Vec<&'static str>;
    fn outputs(&self) -> Vec<&'static str>;
}
