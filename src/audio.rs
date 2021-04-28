use std::sync::mpsc::TryRecvError;
use std::time;

use std::sync::atomic::AtomicI8;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;

use std::time::Duration;

use crate::ui::Cmd;
use crate::util;

use crate::amp;
use crate::env;
use crate::mix;
use crate::osc;
use crate::out;
use crate::seq;

use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

pub const AUDIO_BUFFER_LEN: u64 = 512;

pub fn spawn_audio(
    rx: Receiver<Cmd>,
    tx2: Sender<Cmd>,
    setbeat: Arc<AtomicI8>,
    set_measured_xtime: Arc<AtomicU64>,
    target_inc: u128,
) {
    thread::spawn(move || {
        let mut cv_note_map = [0f64; 256];
        let mut ipc_64_map = [0u16; 256];
        let mut ipc_64_e_map = [0i16; 256];

        for cv in 0..128 {
            let (f, ipc_64, e) = util::cv_to_64th_wavetable_increment(cv as i8);
            cv_note_map[cv] = f;
            ipc_64_map[cv] = ipc_64;
            ipc_64_e_map[cv] = (10_000. * e) as i16;

            //println!("cv={} f={:5.2} ipc={} err={}", cv, f, ipc_64, e);
        }

        let mut wto1 = osc::WaveTableOsc::sin(ipc_64_map, 69);
        wto1.modulation_idx = 126;

        let mut wto2 = osc::FuncOsc::square(ipc_64_map[69]);
        wto2.modulation_idx = 126;

        let mut vca1 = amp::Vca::new(i8::max_value());

        let mut adsr1 = env::Adsr::new();
        adsr1["attack_for"] = 10;
        adsr1["attack_to"] = i8::max_value();
        adsr1["decay_for"] = 127;
        adsr1["sustain_at"] = i8::max_value() / 2;
        adsr1["release_for"] = 127;

        let mut beats = [false; 16];
        beats[0] = true;
        beats[4] = true;
        beats[12] = true;
        for (i, b) in beats.iter().enumerate() {
            tx2.send(Cmd::Beat(i as i8, *b)).unwrap()
        }
        let beats = Arc::new(RwLock::new(beats));
        let mut beat_len = Arc::new(RwLock::new([32; 16]));
        let mut seq1 = seq::BasicSeq::new(Arc::clone(&beats), Arc::clone(&beat_len));

        let mut vca1o = amp::Vca::new(i8::max_value());

        let mut adsr1o = env::Adsr::new();
        adsr1o["attack_for"] = 50;
        adsr1o["attack_to"] = i8::max_value() / 2;
        adsr1o["decay_for"] = 15;
        adsr1o["sustain_at"] = i8::max_value() / 2;
        adsr1o["release_for"] = 15;

        let mut obeats = [false; 16];
        obeats[2] = true;
        obeats[6] = true;
        obeats[10] = true;
        obeats[14] = true;
        for (i, b) in obeats.iter().enumerate() {
            tx2.send(Cmd::Obeat(i as i8, *b)).unwrap()
        }
        let obeats = Arc::new(RwLock::new(obeats));
        let mut obeat_len = Arc::new(RwLock::new([32; 16]));
        let mut seq1o = seq::BasicSeq::new(Arc::clone(&obeats), Arc::clone(&obeat_len));

        let mut mix1 = mix::Mixer::new();
        mix1["a_lvl"] = i8::max_value();
        mix1["b_lvl"] = i8::max_value();

        let mut out1 = out::CpalOut::from_defaults().unwrap();

        let names = vec![
            "wto1", "wto2", "vca1", "adsr1", "seq1", "vca1o", "adsr1o", "seq1o", "mix1", "out1",
        ];
        let mut components: Vec<&mut dyn util::Component> = vec![
            &mut wto1,
            &mut wto2,
            &mut vca1,
            &mut adsr1,
            &mut seq1,
            &mut vca1o,
            &mut adsr1o,
            &mut seq1o,
            &mut mix1,
            &mut out1,
        ];

        // Connect the modulation input of the first oscillator to the
        // output of the second.
        let wires = vec![
            //(("wto2", "out"), ("wto1", "modulation")),
            //(("wto1", "out"), ("wto2", "modulation")),
            (("adsr1", "out"), ("vca1", "amp_cv")),
            (("wto1", "out"), ("vca1", "in_cv")),
            (("seq1", "trigger"), ("adsr1", "trigger")),
            (("seq1", "gate"), ("adsr1", "gate")),
            (("adsr1o", "out"), ("vca1o", "amp_cv")),
            (("wto1", "out"), ("vca1o", "in_cv")),
            (("seq1o", "trigger"), ("adsr1o", "trigger")),
            (("seq1o", "gate"), ("adsr1o", "gate")),
            (("vca1", "out"), ("mix1", "a")),
            (("vca1o", "out"), ("mix1", "b")),
            (("mix1", "out"), ("out1", "cv_in")),
        ];

        // Sanity Check of the wires.
        for (src, dst) in wires.iter() {
            if let None = names.iter().position(|&x| x == src.0) {
                println!("{} not found for {:?}, {:?}", src.0, src, dst);
                return;
            }
            if let None = names.iter().position(|&x| x == dst.0) {
                println!("{} not found for {:?}, {:?}", dst.0, src, dst);
                return;
            }
        }
        let mut exit = false;
        let mut start;
        let tempo = 90;
        let cycles_per_16th = ((60. / ((4 * tempo) as f64)) * (util::RATE as f64)) as u64;
        let mut cycle_counter = 0;
        while !exit {
            //(target_inc - 2 * delay) {

            start = time::Instant::now();

            match rx.try_recv() {
                Ok(c) => match c {
                    Cmd::Freq(f) => components[0]["freq"] = f,
                    Cmd::Beat(i, b) => {
                        beats.write().unwrap()[i as usize] = b;
                        tx2.send(Cmd::Beat(i, beats.read().unwrap()[i as usize]))
                            .unwrap();
                    }
                    Cmd::Obeat(i, b) => {
                        obeats.write().unwrap()[i as usize] = b;
                        tx2.send(Cmd::Obeat(i, obeats.read().unwrap()[i as usize]))
                            .unwrap();
                    }
                },
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => exit = true,
            }
            for _ in 0..AUDIO_BUFFER_LEN {
                let mut tick = false;
                cycle_counter += 1;
                if cycle_counter >= cycles_per_16th {
                    cycle_counter = 0;
                    tick = true;
                }
                // Increment all the components.
                for component in components.iter_mut() {
                    component.step();
                }

                if tick {
                    setbeat.store(components[7]["beat"], Ordering::Relaxed);
                    for component in components.iter_mut() {
                        component.tick();
                    }
                }
                // Update all inputs and outputs as defined by the wires.
                for (src, dst) in wires.iter() {
                    if let Some(i) = names.iter().position(|&x| x == src.0) {
                        if let Some(j) = names.iter().position(|&x| x == dst.0) {
                            components[j][dst.1] = components[i][src.1];
                        }
                    }
                }
            }

            let xtime = start.elapsed();
            set_measured_xtime.store(xtime.as_nanos() as u64, Ordering::Relaxed);
            if (1 * xtime.as_nanos()) < (target_inc * AUDIO_BUFFER_LEN as u128) {
                thread::sleep(
                    Duration::from_nanos((target_inc as u64) * AUDIO_BUFFER_LEN as u64)
                        - (1 * xtime),
                );
            }
        }
    });
}
