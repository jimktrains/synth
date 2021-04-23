use std::ops::{Index, IndexMut};

// Points per second.
const RATE: f64 = 44_100.;

// Wave Table Samples per point.
const WAVE_TABLE_SAMPLES_PER_POINT: f64 = 1024. / RATE;

fn quarter_point_per_32nd_node(tempo: f64) -> u16 {
    let quarter_note_len = 60. / tempo;
    let thirtysecond_note_len = quarter_note_len / 8.;
    let pp32 = RATE * thirtysecond_note_len;
    let pp32_4 = (4 as f64 * pp32) as u16;

    pp32_4
}

fn cv_to_64th_wavetable_increment(cv: i8) -> (f64, u16, f64) {
    let midi_note_index = cv as f64;
    let midi_exp = (midi_note_index - 69.) / 12.;
    let delta = (2f64).powf(midi_exp);
    let f = 440. * delta;
    let ipc = WAVE_TABLE_SAMPLES_PER_POINT * f;
    let ipc_64 = (64. * ipc) as u16;
    let e = ((ipc_64 as f64) / (64. * ipc)) - 1.;

    (f, ipc_64, e)
}

struct WaveTableOsc {
    counter: u16,
    wt: [i8; 1024],
    freq_ipc: u16,
    modulation_idx: i8,
    modulation: i8,
    phase_offset: i8,
    out_cv: i8,
    dummy: i8,
}

impl WaveTableOsc {
    pub fn step(&mut self) {
        // So, in theory we could have a wt with multiple frames in it,
        // so I don't want to hardcode this right now.
        let wt_len = self.wt.len() as u16;

        self.counter = self.counter.wrapping_add(self.freq_ipc);

        // Does left shift work the way I want with signed values?
        // I am trying to use the modulation_idx as essentially as a signed Q1.7
        let m = (((self.modulation as i16) * (self.modulation_idx as i16)) >> 7) as i8;

        // I need to double check that this works the way I'm expecting
        // with the wrapping. Also need to think about how this would
        // be implemented on a microcontroller.
        self.counter = (self.counter as i32).wrapping_add(m as i32) as u16;
        self.counter %= wt_len;

        // I need to double check that this works the way I'm expecting
        // with the wrapping. Also need to think about how this would
        // be implemented on a microcontroller.
        let i = (self.counter as i32).wrapping_add(m as i32) as u16 % wt_len;

        self.out_cv = self.wt[i as usize >> 6];
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
            "modulation" => &mut self.modulation,
            "modulation_idx" => &mut self.modulation_idx,
            // This should probably error.
            _ => &mut self.dummy,
        }
    }
}

fn main() {
    let tempo = 45.;
    println!("{} bpm = {} ", tempo, quarter_point_per_32nd_node(tempo));
    let tempo = 60.;
    println!("{} bpm = {} ", tempo, quarter_point_per_32nd_node(tempo));
    let tempo = 90.;
    println!("{} bpm = {} ", tempo, quarter_point_per_32nd_node(tempo));
    let tempo = 120.;
    println!("{} bpm = {} ", tempo, quarter_point_per_32nd_node(tempo));

    let mut cv_note_map = [0f64; 256];
    let mut ipc_64_map = [0u16; 256];
    let mut ipc_64_e_map = [0i16; 256];
    for cv in 0..128 {
        let (f, ipc_64, e) = cv_to_64th_wavetable_increment(cv as i8);
        cv_note_map[cv] = f;
        ipc_64_map[cv] = ipc_64;
        ipc_64_e_map[cv] = (10_000. * e) as i16;

        //println!("cv={} f={:5.2} ipc={} err={}", cv, f, ipc_64, e);
    }

    println!(
        "max err = {}",
        ipc_64_e_map.iter().map(|x| x.abs()).max().unwrap()
    );
    println!(
        "avg err = {}",
        (ipc_64_e_map.iter().map(|x| x.abs()).sum::<i16>() as f64) / (ipc_64_e_map.len() as f64)
    );

    let mut sin_wt = [0i8; 1024];
    let mut tr_wt = [0i8; 1024];
    let mut sq_wt = [0i8; 1024];
    let mut saw_wt = [0i8; 1024];
    for i in 0..1024 {
        sin_wt[i] =
            ((((i as f64) / 1024. * std::f64::consts::TAU).sin()) * (i8::max_value() as f64)) as i8;

        saw_wt[i] =
            ((i8::max_value() as f64) - ((u8::max_value() as f64) * ((i as f64) / 1024.))) as i8;

        if i <= 1024 / 4 {
            tr_wt[i] = ((i8::max_value() as f64) * (i as f64) / (1024. / 4.)) as i8;
        } else if i <= 1024 / 2 {
            tr_wt[i] = ((i8::max_value() as f64)
                * (1. - ((i as f64) - (1024. / 4.)) / (1024. / 4.))) as i8;
        } else if i <= 1024 / 4 * 3 {
            tr_wt[i] =
                ((i8::min_value() as f64) * ((i as f64) - (1024. / 2.)) / (1024. / 4.)) as i8;
        } else {
            tr_wt[i] = ((i8::min_value() as f64)
                * (1. - ((i as f64) - (1024. / 4. * 3.)) / (1024. / 4.)))
                as i8;
        }

        if i <= 1024 / 2 {
            sq_wt[i] = i8::max_value();
        } else {
            sq_wt[i] = i8::min_value();
        }

        let names = vec!["wto", "wto2"];
        let mut components = vec![
            WaveTableOsc {
                counter: 0,
                wt: sin_wt.clone(),
                freq_ipc: ipc_64_map[69],
                modulation_idx: 0,
                modulation: 0,
                phase_offset: 0,
                out_cv: 0,
                dummy: 0,
            },
            WaveTableOsc {
                counter: 0,
                wt: sin_wt.clone(),
                freq_ipc: ipc_64_map[69],
                modulation_idx: 0,
                modulation: 0,
                phase_offset: 0,
                out_cv: 0,
                dummy: 0,
            },
        ];

        // Connect the modulation input of the first oscillator to the
        // output of the second.
        let wires = vec![(("wto2", "out"), ("wto", "modulation"))];

        for _ in 0..1 {
            // Increment all the components.
            for component in components.iter_mut() {
                component.step();
            }
            // Update all inputs and outputs as defined by the wires.
            for (src, dst) in wires.iter() {
                // Need to actually check this and report errors....
                if let Some(i) = names.iter().position(|&x| x == src.0) {
                    if let Some(j) = names.iter().position(|&x| x == dst.0) {
                        components[j][dst.1] = components[i][src.1];
                    }
                }
            }
        }
    }
}
