use std::ops::{Index, IndexMut};

extern crate rand;

use crate::util::Component;
use crate::util::WAVE_TABLE_SAMPLES_PER_CYCLE;

pub struct WaveTableOsc {
    pub counter: u32,
    pub wt: [i16; WAVE_TABLE_SAMPLES_PER_CYCLE as usize],
    pub freq_ipc: u32,
    pub freq: i16,
    pub modulation_idx: i16,
    pub modulation: i16,
    pub phase_offset: i16,
    pub out_cv: i16,
    pub dummy: i16,
    ipc_64_map: [u32; 256],
}

impl WaveTableOsc {
    pub fn new(
        wt: [i16; WAVE_TABLE_SAMPLES_PER_CYCLE as usize],
        ipc_64_map: [u32; 256],
        init_freq: i16,
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

    pub fn sin(ipc_64_map: [u32; 256], init_freq: i16) -> WaveTableOsc {
        let mut wto = WaveTableOsc::new(
            [0; WAVE_TABLE_SAMPLES_PER_CYCLE as usize],
            ipc_64_map,
            init_freq,
        );
        for i in 0..WAVE_TABLE_SAMPLES_PER_CYCLE {
            wto.wt[i as usize] = ((((i as f64) / (WAVE_TABLE_SAMPLES_PER_CYCLE as f64)
                * std::f64::consts::TAU)
                .sin())
                * (i16::max_value() as f64)) as i16;
        }

        wto
    }
    pub fn saw(ipc_64_map: [u32; 256], init_freq: i16) -> WaveTableOsc {
        let mut wto = WaveTableOsc::new(
            [0; WAVE_TABLE_SAMPLES_PER_CYCLE as usize],
            ipc_64_map,
            init_freq,
        );
        for i in 0..WAVE_TABLE_SAMPLES_PER_CYCLE as usize {
            wto.wt[i] = ((i16::max_value() as f64)
                - ((u16::max_value() as f64) * ((i as f64) / (WAVE_TABLE_SAMPLES_PER_CYCLE as f64))))
                as i16;
        }

        wto
    }
    pub fn triangle(ipc_64_map: [u32; 256], init_freq: i16) -> WaveTableOsc {
        let mut wto = WaveTableOsc::new(
            [0; WAVE_TABLE_SAMPLES_PER_CYCLE as usize],
            ipc_64_map,
            init_freq,
        );
        for i in 0..WAVE_TABLE_SAMPLES_PER_CYCLE as usize {
            if i <= WAVE_TABLE_SAMPLES_PER_CYCLE as usize / 4 {
                wto.wt[i] = ((i16::max_value() as f64) * (i as f64)
                    / ((WAVE_TABLE_SAMPLES_PER_CYCLE as f64) / 4.))
                    as i16;
            } else if i <= (WAVE_TABLE_SAMPLES_PER_CYCLE as usize) / 2 {
                wto.wt[i] = ((i16::max_value() as f64)
                    * (1.
                        - ((i as f64) - ((WAVE_TABLE_SAMPLES_PER_CYCLE as f64) / 4.))
                            / ((WAVE_TABLE_SAMPLES_PER_CYCLE as f64) / 4.)))
                    as i16;
            } else if i <= (WAVE_TABLE_SAMPLES_PER_CYCLE as usize) / 4 * 3 {
                wto.wt[i] = ((i16::min_value() as f64)
                    * ((i as f64) - ((WAVE_TABLE_SAMPLES_PER_CYCLE as f64) / 2.))
                    / ((WAVE_TABLE_SAMPLES_PER_CYCLE as f64) / 4.))
                    as i16;
            } else {
                wto.wt[i] = ((i16::min_value() as f64)
                    * (1.
                        - ((i as f64) - ((WAVE_TABLE_SAMPLES_PER_CYCLE as f64) / 4. * 3.))
                            / ((WAVE_TABLE_SAMPLES_PER_CYCLE as f64) / 4.)))
                    as i16;
            }
        }

        wto
    }

    pub fn square(ipc_64_map: [u32; 256], init_freq: i16) -> WaveTableOsc {
        let mut wto = WaveTableOsc::new(
            [0; WAVE_TABLE_SAMPLES_PER_CYCLE as usize],
            ipc_64_map,
            init_freq,
        );
        for i in 0..(WAVE_TABLE_SAMPLES_PER_CYCLE as usize) {
            if i <= (WAVE_TABLE_SAMPLES_PER_CYCLE as usize) / 2 {
                wto.wt[i] = i16::max_value();
            } else {
                wto.wt[i] = i16::min_value();
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
        // let wt_len = (64 * self.wt.len()) as u32;
        // Setting the len to 1024 allows natural wrapping of a u32.

        self.counter = self.counter.wrapping_add(self.freq_ipc);

        // Does left shift work the way I want with signed values?
        // I am trying to use the modulation_idx as essentially as a signed Q1.7
        //println!("{} {}", self.freq_ipc, self.modulation_idx);
        let m = (self.freq_ipc as i32) * (self.modulation_idx as i32) >> 7;
        let m = (((self.modulation as i32) * m) >> 7) as i16;

        // I need to double check that this works the way I'm expecting
        // with the wrapping. Also need to think about how this would
        // be implemented on a microcontroller.
        self.counter = (self.counter as i32).wrapping_add(m as i32) as u32;
        // Setting the len to 1024 allows natural wrapping of a u32.
        // self.counter %= wt_len;

        // I need to double check that this works the way I'm expecting
        // with the wrapping. Also need to think about how this would
        // be implemented on a microcontroller.
        let mut i = (self.counter as i32).wrapping_add(m as i32) as usize;
        i = i.wrapping_add(self.phase_offset as usize);
        // Setting the len to 1024 allows natural wrapping of a u32.
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
    type Output = i16;

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
