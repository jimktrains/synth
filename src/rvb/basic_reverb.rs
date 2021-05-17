use std::ops::{Index, IndexMut};

use crate::util::Component;
use crate::util::RATE;

pub struct BasicReverb {
    buffer: [[i16; RATE as usize]; 3],
    pub delay: [i16; 3],
    pub gain: [f32; 3],
    cv_in: i16,
    out_cv: i16,
    dummy: i16,
    counter: usize,
}

impl BasicReverb {
    pub fn new() -> BasicReverb {
        BasicReverb {
            buffer: [[0i16; RATE as usize]; 3],
            delay: [0i16; 3],
            gain: [0.25f32; 3],
            cv_in: 0i16,
            out_cv: 0i16,
            dummy: 0i16,
            counter: 0usize,
        }
    }
}

impl Component for BasicReverb {
    fn tick(&mut self) {}
    fn step(&mut self) {
        let mut inv = self.cv_in;
        for i in 0..3 {
            let delay = self.delay[i].abs() as usize;
            let gain = self.gain[i];
            let buffer = self.buffer.get_mut(i).unwrap();
            let counter = self.counter % delay;

            inv = ((inv as f32) * (1. - gain)) as i16;
            let delayed_i = (counter + 1) % delay;
            let delayed = ((1. - (gain * gain)) * (buffer[delayed_i] as f32)) as i16;
            inv = inv.saturating_add(delayed);

            buffer[counter] = self.cv_in + ((gain * (buffer[counter] as f32)) as i16);
        }

        self.out_cv = inv;
        self.counter += 1;
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
