use std::ops::{Index, IndexMut};

extern crate rand;

use std::fs::File;
use std::path::Path;
use std::path::PathBuf;
use wav;
use wav::bit_depth::BitDepth;

use crate::util::Component;
use crate::util::WAVE_TABLE_SAMPLES_PER_CYCLE;
use crate::util::WAVE_TABLE_SAMPLES_PER_CYCLE_FACTOR;
lazy_static! {
    static ref SIN_TABLE: [i16; WAVE_TABLE_SAMPLES_PER_CYCLE as usize] = {
        let mut wt = [0; WAVE_TABLE_SAMPLES_PER_CYCLE as usize];
        for i in 0..WAVE_TABLE_SAMPLES_PER_CYCLE {
            wt[i as usize] = (((((i as f64) / (WAVE_TABLE_SAMPLES_PER_CYCLE as f64))
                * std::f64::consts::TAU)
                .sin())
                * (i16::min_value() as f64)) as i16;
        }
        wt
    };
    static ref SAW_TABLE: [i16; WAVE_TABLE_SAMPLES_PER_CYCLE as usize] = {
        let mut wt = [0; WAVE_TABLE_SAMPLES_PER_CYCLE as usize];
        for i in 0..WAVE_TABLE_SAMPLES_PER_CYCLE {
            wt[i as usize] = ((i16::max_value() as f64)
                - ((u16::max_value() as f64)
                    * ((i as f64) / (WAVE_TABLE_SAMPLES_PER_CYCLE as f64))))
                as i16;
        }
        wt
    };
    static ref TRIANGLE_TABLE: [i16; WAVE_TABLE_SAMPLES_PER_CYCLE as usize] = {
        let mut wt = [0; WAVE_TABLE_SAMPLES_PER_CYCLE as usize];
        for i in 0..WAVE_TABLE_SAMPLES_PER_CYCLE {
            wt[i as usize] = if i <= WAVE_TABLE_SAMPLES_PER_CYCLE / 4 {
                ((i16::max_value() as f64) * (i as f64)
                    / ((WAVE_TABLE_SAMPLES_PER_CYCLE as f64) / 4.)) as i16
            } else if i <= WAVE_TABLE_SAMPLES_PER_CYCLE / 2 {
                ((i16::max_value() as f64)
                    * (1.
                        - ((i as f64) - ((WAVE_TABLE_SAMPLES_PER_CYCLE as f64) / 4.))
                            / ((WAVE_TABLE_SAMPLES_PER_CYCLE as f64) / 4.))) as i16
            } else if i <= WAVE_TABLE_SAMPLES_PER_CYCLE / 4 * 3 {
                ((i16::min_value() as f64)
                    * ((i as f64) - ((WAVE_TABLE_SAMPLES_PER_CYCLE as f64) / 2.))
                    / ((WAVE_TABLE_SAMPLES_PER_CYCLE as f64) / 4.)) as i16
            } else {
                ((i16::min_value() as f64)
                    * (1.
                        - ((i as f64) - ((WAVE_TABLE_SAMPLES_PER_CYCLE as f64) / 4. * 3.))
                            / ((WAVE_TABLE_SAMPLES_PER_CYCLE as f64) / 4.))) as i16
            }
        }
        wt
    };
    static ref SQUARE_TABLE: [i16; WAVE_TABLE_SAMPLES_PER_CYCLE as usize] = {
        let mut wt = [0; WAVE_TABLE_SAMPLES_PER_CYCLE as usize];
        for i in 0..WAVE_TABLE_SAMPLES_PER_CYCLE {
            wt[i as usize] = if i <= WAVE_TABLE_SAMPLES_PER_CYCLE / 2 {
                i16::max_value()
            } else {
                i16::min_value()
            }
        }
        wt
    };
    static ref WHITE_NOISE_TABLE: [i16; WAVE_TABLE_SAMPLES_PER_CYCLE as usize] = {
        let mut wt = [0i16; WAVE_TABLE_SAMPLES_PER_CYCLE as usize];
        for i in 1..WAVE_TABLE_SAMPLES_PER_CYCLE {
            wt[i as usize] = wt[(i - 1) as usize].wrapping_add(rand::random());
        }
        wt
    };
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum WaveTableChoice {
    Custom,
    Sin,
    Saw,
    Triangle,
    Square,
    WhiteNoise,
}

pub struct WaveTableOsc {
    pub counter: u32,
    pub wt_i: u32,
    pub wt: [i16; WAVE_TABLE_SAMPLES_PER_CYCLE as usize],
    pub which_table: WaveTableChoice,
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
        which_table: WaveTableChoice,
    ) -> WaveTableOsc {
        WaveTableOsc {
            wt_i: 0,
            counter: 0,
            wt: wt,
            freq: init_freq,
            modulation_idx: 0,
            modulation: 0,
            phase_offset: 0,
            out_cv: 0,
            dummy: 0,
            ipc_64_map: ipc_64_map,
            which_table: which_table,
        }
    }

    // Yes, I know, it's not great do this in the audio thread.
    pub fn load_scwf(&mut self, filename: PathBuf) {
        let mut inp_file = File::open(filename).unwrap();
        let (_header, data) = wav::read(&mut inp_file).unwrap();

        match data {
            BitDepth::Eight(_) => println!("8"),
            BitDepth::Sixteen(fwt) => {
                for i in 1..fwt.len().min(WAVE_TABLE_SAMPLES_PER_CYCLE as usize) {
                    self.wt[i as usize] = fwt[i as usize];
                }
            }
            BitDepth::TwentyFour(_) => println!("25"),
            BitDepth::ThirtyTwoFloat(_) => println!("32"),
            BitDepth::Empty => println!("0"),
        };
        self.which_table = WaveTableChoice::Custom;
    }

    pub fn sin(ipc_64_map: [u32; 256], init_freq: i16) -> WaveTableOsc {
        WaveTableOsc::new(
            [0; WAVE_TABLE_SAMPLES_PER_CYCLE as usize],
            ipc_64_map,
            init_freq,
            WaveTableChoice::Sin,
        )
    }
    pub fn saw(ipc_64_map: [u32; 256], init_freq: i16) -> WaveTableOsc {
        WaveTableOsc::new(
            [0; WAVE_TABLE_SAMPLES_PER_CYCLE as usize],
            ipc_64_map,
            init_freq,
            WaveTableChoice::Saw,
        )
    }
    pub fn triangle(ipc_64_map: [u32; 256], init_freq: i16) -> WaveTableOsc {
        WaveTableOsc::new(
            [0; WAVE_TABLE_SAMPLES_PER_CYCLE as usize],
            ipc_64_map,
            init_freq,
            WaveTableChoice::Triangle,
        )
    }
    pub fn square(ipc_64_map: [u32; 256], init_freq: i16) -> WaveTableOsc {
        WaveTableOsc::new(
            [0; WAVE_TABLE_SAMPLES_PER_CYCLE as usize],
            ipc_64_map,
            init_freq,
            WaveTableChoice::Square,
        )
    }

    pub fn white_noise(ipc_64_map: [u32; 256], init_freq: i16) -> WaveTableOsc {
        WaveTableOsc::new(
            [0; WAVE_TABLE_SAMPLES_PER_CYCLE as usize],
            ipc_64_map,
            init_freq,
            WaveTableChoice::WhiteNoise,
        )
    }
}

impl Component for WaveTableOsc {
    fn tick(&mut self) {}
    fn step(&mut self) {
        let freq_ipc = self.ipc_64_map[self.freq as usize];
        // So, in theory we could have a wt with multiple frames in it,
        // so I don't want to hardcode this right now.
        // let wt_len = (64 * self.wt.len()) as u32;
        // Setting the len to 1024 allows natural wrapping of a u32.

        // Does left shift work the way I want with signed values?
        // I am trying to use the modulation_idx as essentially as a signed Q1.7
        //println!("{} {}", self.freq_ipc, self.modulation_idx);
        let m = ((freq_ipc as i64) * (self.modulation_idx as i64)) >> 15;
        let m = (((self.modulation as i64) * m) >> 15) as u64;

        self.counter = ((self.counter as u64)
            .wrapping_add(WAVE_TABLE_SAMPLES_PER_CYCLE_FACTOR as u64)
            .wrapping_add(m)) as u32;

        while self.counter > freq_ipc {
            self.wt_i += 1;
            self.counter -= freq_ipc;
        }
        self.wt_i %= WAVE_TABLE_SAMPLES_PER_CYCLE;

        // // I need to double check that this works the way I'm expecting
        // // with the wrapping. Also need to think about how this would
        // // be implemented on a microcontroller.
        // self.counter = ((self.counter as i64).wrapping_add(m) % (i32::max_value() as i64)) as u32;
        // // Setting the len to 1024 allows natural wrapping of a u32.
        // // self.counter %= wt_len;

        // // I need to double check that this works the way I'm expecting
        // // with the wrapping. Also need to think about how this would
        // // be implemented on a microcontroller.
        // let mut i = (self.counter as i64).wrapping_add(self.phase_offset as i64);
        // // Setting the len to 1024 allows natural wrapping of a u16.
        // i /= WAVE_TABLE_SAMPLES_PER_CYCLE_FACTOR as i64;
        // i %= self.wt.len() as i64;
        // let i = i as usize;

        let i = self.wt_i as usize;
        self.out_cv = match self.which_table {
            WaveTableChoice::Custom => self.wt[i],
            WaveTableChoice::Sin => SIN_TABLE[i],
            WaveTableChoice::Saw => SAW_TABLE[i],
            WaveTableChoice::Square => SQUARE_TABLE[i],
            WaveTableChoice::Triangle => TRIANGLE_TABLE[i],
            WaveTableChoice::WhiteNoise => WHITE_NOISE_TABLE[i],
        };
    }

    fn inputs(&self) -> Vec<&'static str> {
        vec!["modulation_idx", "modulation"]
    }

    fn outputs(&self) -> Vec<&'static str> {
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
