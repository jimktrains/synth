use std::ops::{Index, IndexMut};

extern crate rand;

use crate::util::Component;
use crate::util::WAVE_TABLE_SAMPLES_PER_CYCLE;

pub struct WaveTableOsc {
    pub counter: u16,
    pub wt: [i8; WAVE_TABLE_SAMPLES_PER_CYCLE as usize],
    pub freq_ipc: u16,
    pub freq: i8,
    pub modulation_idx: i8,
    pub modulation: i8,
    pub phase_offset: i8,
    pub out_cv: i8,
    pub dummy: i8,
    ipc_64_map: [u16; 256],
}

impl WaveTableOsc {
    pub fn new(
        wt: [i8; WAVE_TABLE_SAMPLES_PER_CYCLE as usize],
        ipc_64_map: [u16; 256],
        init_freq: i8,
    ) -> WaveTableOsc {
        WaveTableOsc {
            counter: 0,
            wt: wt,
            freq_ipc: ipc_64_map[init_freq as usize],
            freq: init_freq,
            modulation_idx: 0,
            modulation: 0,
            phase_offset: 0,
            out_cv: 0,
            dummy: 0,
            ipc_64_map: ipc_64_map,
        }
    }

    pub fn sin(ipc_64_map: [u16; 256], init_freq: i8) -> WaveTableOsc {
        let mut wto = WaveTableOsc::new(
            [0; WAVE_TABLE_SAMPLES_PER_CYCLE as usize],
            ipc_64_map,
            init_freq,
        );
        for i in 0..WAVE_TABLE_SAMPLES_PER_CYCLE {
            wto.wt[i as usize] = ((((i as f64) / (WAVE_TABLE_SAMPLES_PER_CYCLE as f64)
                * std::f64::consts::TAU)
                .sin())
                * (i8::max_value() as f64)) as i8;
        }

        wto
    }
    pub fn saw(ipc_64_map: [u16; 256], init_freq: i8) -> WaveTableOsc {
        let mut wto = WaveTableOsc::new(
            [0; WAVE_TABLE_SAMPLES_PER_CYCLE as usize],
            ipc_64_map,
            init_freq,
        );
        for i in 0..WAVE_TABLE_SAMPLES_PER_CYCLE as usize {
            wto.wt[i] = ((i8::max_value() as f64)
                - ((u8::max_value() as f64) * ((i as f64) / (WAVE_TABLE_SAMPLES_PER_CYCLE as f64))))
                as i8;
        }

        wto
    }
    pub fn triangle(ipc_64_map: [u16; 256], init_freq: i8) -> WaveTableOsc {
        let mut wto = WaveTableOsc::new(
            [0; WAVE_TABLE_SAMPLES_PER_CYCLE as usize],
            ipc_64_map,
            init_freq,
        );
        for i in 0..WAVE_TABLE_SAMPLES_PER_CYCLE as usize {
            if i <= WAVE_TABLE_SAMPLES_PER_CYCLE as usize / 4 {
                wto.wt[i] = ((i8::max_value() as f64) * (i as f64)
                    / ((WAVE_TABLE_SAMPLES_PER_CYCLE as f64) / 4.))
                    as i8;
            } else if i <= (WAVE_TABLE_SAMPLES_PER_CYCLE as usize) / 2 {
                wto.wt[i] = ((i8::max_value() as f64)
                    * (1.
                        - ((i as f64) - ((WAVE_TABLE_SAMPLES_PER_CYCLE as f64) / 4.))
                            / ((WAVE_TABLE_SAMPLES_PER_CYCLE as f64) / 4.)))
                    as i8;
            } else if i <= (WAVE_TABLE_SAMPLES_PER_CYCLE as usize) / 4 * 3 {
                wto.wt[i] = ((i8::min_value() as f64)
                    * ((i as f64) - ((WAVE_TABLE_SAMPLES_PER_CYCLE as f64) / 2.))
                    / ((WAVE_TABLE_SAMPLES_PER_CYCLE as f64) / 4.))
                    as i8;
            } else {
                wto.wt[i] = ((i8::min_value() as f64)
                    * (1.
                        - ((i as f64) - ((WAVE_TABLE_SAMPLES_PER_CYCLE as f64) / 4. * 3.))
                            / ((WAVE_TABLE_SAMPLES_PER_CYCLE as f64) / 4.)))
                    as i8;
            }
        }

        wto
    }

    pub fn square(ipc_64_map: [u16; 256], init_freq: i8) -> WaveTableOsc {
        let mut wto = WaveTableOsc::new(
            [0; WAVE_TABLE_SAMPLES_PER_CYCLE as usize],
            ipc_64_map,
            init_freq,
        );
        for i in 0..(WAVE_TABLE_SAMPLES_PER_CYCLE as usize) {
            if i <= (WAVE_TABLE_SAMPLES_PER_CYCLE as usize) / 2 {
                wto.wt[i] = i8::max_value();
            } else {
                wto.wt[i] = i8::min_value();
            }
        }

        wto
    }
}

impl<'a> Component<'a> for WaveTableOsc {
    fn tick(&mut self) {}
    fn step(&mut self) {
        self.freq_ipc = self.ipc_64_map[self.freq as usize];
        // So, in theory we could have a wt with multiple frames in it,
        // so I don't want to hardcode this right now.
        // let wt_len = (64 * self.wt.len()) as u16;
        // Setting the len to 1024 allows natural wrapping of a u16.

        self.counter = self.counter.wrapping_add(self.freq_ipc);

        // Does left shift work the way I want with signed values?
        // I am trying to use the modulation_idx as essentially as a signed Q1.7
        //println!("{} {}", self.freq_ipc, self.modulation_idx);
        let m = (self.freq_ipc as i32) * (self.modulation_idx as i32) >> 7;
        let m = (((self.modulation as i32) * m) >> 7) as i8;

        // I need to double check that this works the way I'm expecting
        // with the wrapping. Also need to think about how this would
        // be implemented on a microcontroller.
        self.counter = (self.counter as i32).wrapping_add(m as i32) as u16;
        // Setting the len to 1024 allows natural wrapping of a u16.
        // self.counter %= wt_len;

        // I need to double check that this works the way I'm expecting
        // with the wrapping. Also need to think about how this would
        // be implemented on a microcontroller.
        let mut i = (self.counter as i32).wrapping_add(m as i32) as usize;
        i = i.wrapping_add(self.phase_offset as usize);
        // Setting the len to 1024 allows natural wrapping of a u16.
        i >>= 6;
        i %= self.wt.len();

        self.out_cv = self.wt[i as usize];
    }

    fn inputs(&self) -> Vec<&'a str> {
        vec!["modulation_idx", "modulation"]
    }

    fn outputs(&self) -> Vec<&'a str> {
        vec!["out"]
    }
}

impl Index<&str> for WaveTableOsc {
    type Output = i8;

    fn index(&self, i: &str) -> &Self::Output {
        match i {
            "out" => &self.out_cv,
            _ => &0,
        }
    }
}

impl IndexMut<&str> for WaveTableOsc {
    fn index_mut(&mut self, i: &str) -> &mut Self::Output {
        match i {
            "freq" => &mut self.freq,
            "modulation" => &mut self.modulation,
            "modulation_idx" => &mut self.modulation_idx,
            // This should probably error.
            _ => &mut self.dummy,
        }
    }
}
