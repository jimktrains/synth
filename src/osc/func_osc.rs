use std::ops::{Index, IndexMut};

use crate::util::Component;
use crate::util::WAVE_TABLE_SAMPLES_PER_CYCLE;

pub struct FuncOsc {
    pub counter: u16,
    pub f: fn(u16) -> i8,
    pub freq_ipc: u16,
    pub modulation_idx: i8,
    pub modulation: i8,
    pub phase_offset: i8,
    pub out_cv: i8,
    pub dummy: i8,
}

impl FuncOsc {
    pub fn new(f: fn(u16) -> i8, init_freq_ipc_64: u16) -> Self {
        FuncOsc {
            counter: 0,
            f: f,
            freq_ipc: init_freq_ipc_64,
            modulation_idx: 0,
            modulation: 0,
            phase_offset: 0,
            out_cv: 0,
            dummy: 0,
        }
    }

    pub fn saw(init_freq_ipc_64: u16) -> Self {
        // TODO: Remove the floating point from this
        fn f(i: u16) -> i8 {
            ((i8::max_value() as f64)
                - ((u8::max_value() as f64) * ((i as f64) / (WAVE_TABLE_SAMPLES_PER_CYCLE as f64))))
                as i8
        }

        FuncOsc::new(f, init_freq_ipc_64)
    }
    pub fn triangle(init_freq_ipc_64: u16) -> Self {
        // TODO: Remove the floating point from this
        fn f(i: u16) -> i8 {
            if i <= WAVE_TABLE_SAMPLES_PER_CYCLE / 4 {
                ((i8::max_value() as f64) * (i as f64)
                    / ((WAVE_TABLE_SAMPLES_PER_CYCLE as f64) / 4.)) as i8
            } else if i <= WAVE_TABLE_SAMPLES_PER_CYCLE / 2 {
                ((i8::max_value() as f64)
                    * (1.
                        - ((i as f64) - ((WAVE_TABLE_SAMPLES_PER_CYCLE as f64) / 4.))
                            / ((WAVE_TABLE_SAMPLES_PER_CYCLE as f64) / 4.))) as i8
            } else if i <= WAVE_TABLE_SAMPLES_PER_CYCLE / 4 * 3 {
                ((i8::min_value() as f64)
                    * ((i as f64) - ((WAVE_TABLE_SAMPLES_PER_CYCLE as f64) / 2.))
                    / ((WAVE_TABLE_SAMPLES_PER_CYCLE as f64) / 4.)) as i8
            } else {
                ((i8::min_value() as f64)
                    * (1.
                        - ((i as f64) - ((WAVE_TABLE_SAMPLES_PER_CYCLE as f64) / 4. * 3.))
                            / ((WAVE_TABLE_SAMPLES_PER_CYCLE as f64) / 4.))) as i8
            }
        }

        FuncOsc::new(f, init_freq_ipc_64)
    }
    pub fn white_noise(init_freq_ipc_64: u16) -> Self {
        // TODO: Remove the floating point from this
        fn f(i: u16) -> i8 {
            rand::random()
        }

        FuncOsc::new(f, init_freq_ipc_64)
    }

    // Pink noise notes
    // https://www.firstpr.com.au/dsp/pink-noise/
    // https://web.archive.org/web/20150701033149/home.earthlink.net/~ltrammell/tech/newpink.htm
    // https://arxiv.org/ftp/nlin/papers/0511/0511041.pdf

    pub fn square(init_freq_ipc_64: u16) -> Self {
        fn f(i: u16) -> i8 {
            if i <= WAVE_TABLE_SAMPLES_PER_CYCLE / 2 {
                i8::max_value()
            } else {
                i8::min_value()
            }
        }

        FuncOsc::new(f, init_freq_ipc_64)
    }
}

impl<'a> Component<'a> for FuncOsc {
    fn step(&mut self) {
        self.counter = self.counter.wrapping_add(self.freq_ipc);

        // Does left shift work the way I want with signed values?
        // I am trying to use the modulation_idx as essentially as a signed Q1.7
        let m = (((self.modulation as i16) * (self.modulation_idx as i16)) >> 7) as i8;

        // I need to double check that this works the way I'm expecting
        // with the wrapping. Also need to think about how this would
        // be implemented on a microcontroller.
        self.counter = (self.counter as i32).wrapping_add(m as i32) as u16;
        // self.counter %= (WAVE_TABLE_SAMPLES_PER_CYCLE * 64);

        // I need to double check that this works the way I'm expecting
        // with the wrapping. Also need to think about how this would
        // be implemented on a microcontroller.
        //let mut i = (self.counter as i32).wrapping_add(m as i32);
        let i = (self.counter as i32).wrapping_add(self.phase_offset as i32);
        //    % (WAVE_TABLE_SAMPLES_PER_CYCLE * 64) as i32;

        self.out_cv = (self.f)((i >> 6) as u16);
    }

    fn inputs(&self) -> Vec<&'a str> {
        vec!["modulation_idx", "modulation"]
    }

    fn outputs(&self) -> Vec<&'a str> {
        vec!["out"]
    }
}

impl Index<&str> for FuncOsc {
    type Output = i8;

    fn index(&self, i: &str) -> &Self::Output {
        match i {
            "out" => &self.out_cv,
            _ => &0,
        }
    }
}

impl IndexMut<&str> for FuncOsc {
    fn index_mut(&mut self, i: &str) -> &mut Self::Output {
        match i {
            "modulation" => &mut self.modulation,
            "modulation_idx" => &mut self.modulation_idx,
            // This should probably error.
            _ => &mut self.dummy,
        }
    }
}
