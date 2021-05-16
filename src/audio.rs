use std::ops::{Index, IndexMut};
use std::sync::atomic::AtomicI16;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::mpsc::TryRecvError;
use std::sync::Arc;
use std::sync::RwLock;

use crate::ui::Cmd;
use crate::util;

use crate::amp;
use crate::env;
use crate::mix;
use crate::osc;
use crate::out;
use crate::rvb;
use crate::seq;
use crate::util::RATE;

use crate::util::Component;

use crate::arp;

pub const AUDIO_BUFFER_LEN: u64 = 512;

enum AvailableComponents {
    Adsr(env::Adsr),
    BasicArp(arp::BasicArp),
    BasicSeq(seq::BasicSeq),
    FuncOsc(osc::FuncOsc),
    Mixer(mix::Mixer),
    Vca(amp::Vca),
    WaveTableOsc(osc::WaveTableOsc),
    BasicReverb(rvb::BasicReverb),
}

impl AvailableComponents {
    fn step(&mut self) {
        match self {
            AvailableComponents::Adsr(x) => x.step(),
            AvailableComponents::BasicArp(x) => x.step(),
            AvailableComponents::BasicSeq(x) => x.step(),
            AvailableComponents::FuncOsc(x) => x.step(),
            AvailableComponents::Mixer(x) => x.step(),
            AvailableComponents::Vca(x) => x.step(),
            AvailableComponents::WaveTableOsc(x) => x.step(),
            AvailableComponents::BasicReverb(x) => x.step(),
        }
    }
    fn tick(&mut self) {
        match self {
            AvailableComponents::Adsr(x) => x.tick(),
            AvailableComponents::BasicArp(x) => x.tick(),
            AvailableComponents::BasicSeq(x) => x.tick(),
            AvailableComponents::FuncOsc(x) => x.tick(),
            AvailableComponents::Mixer(x) => x.tick(),
            AvailableComponents::Vca(x) => x.tick(),
            AvailableComponents::WaveTableOsc(x) => x.tick(),
            AvailableComponents::BasicReverb(x) => x.tick(),
        }
    }
}
impl Index<&str> for AvailableComponents {
    type Output = i16;

    fn index(&self, i: &str) -> &Self::Output {
        match self {
            AvailableComponents::Adsr(x) => x.index(i),
            AvailableComponents::BasicArp(x) => x.index(i),
            AvailableComponents::BasicSeq(x) => x.index(i),
            AvailableComponents::FuncOsc(x) => x.index(i),
            AvailableComponents::Mixer(x) => x.index(i),
            AvailableComponents::Vca(x) => x.index(i),
            AvailableComponents::WaveTableOsc(x) => x.index(i),
            AvailableComponents::BasicReverb(x) => x.index(i),
        }
    }
}

impl IndexMut<&str> for AvailableComponents {
    fn index_mut(&mut self, i: &str) -> &mut Self::Output {
        match self {
            AvailableComponents::Adsr(x) => x.index_mut(i),
            AvailableComponents::BasicArp(x) => x.index_mut(i),
            AvailableComponents::BasicSeq(x) => x.index_mut(i),
            AvailableComponents::FuncOsc(x) => x.index_mut(i),
            AvailableComponents::Mixer(x) => x.index_mut(i),
            AvailableComponents::Vca(x) => x.index_mut(i),
            AvailableComponents::WaveTableOsc(x) => x.index_mut(i),
            AvailableComponents::BasicReverb(x) => x.index_mut(i),
        }
    }
}

pub fn spawn_audio(
    rx: Receiver<Cmd>,
    tx2: Sender<Cmd>,
    setbeat: Arc<AtomicI16>,
    set_measured_xtime: Arc<AtomicU64>,
    target_inc: u128,
) -> Option<out::CpalOut> {
    let mut cv_note_map = [0f64; 256];
    let mut ipc_64_map = [0u32; 256];
    let mut ipc_64_e_map = [0i32; 256];

    for cv in 0..128 {
        let (f, ipc_64, e) = util::cv_to_64th_wavetable_increment(cv as i16);
        cv_note_map[cv] = f;
        ipc_64_map[cv] = ipc_64;
        ipc_64_e_map[cv] = (10_000. * e) as i32;

        //println!("cv={} f={:5.2} ipc={} err={}", cv, f, ipc_64, e);
    }

    let mut wto1 = osc::WaveTableOsc::sin(ipc_64_map, 69);
    wto1.modulation_idx = i16::max_value();

    let mut wto1o = osc::WaveTableOsc::square(ipc_64_map, 69);
    wto1.modulation_idx = i16::max_value();

    let mut wto2 = osc::WaveTableOsc::sin(ipc_64_map, 0);
    wto2.modulation_idx = i16::max_value();

    let mut vca1 = amp::Vca::new(i16::max_value());

    let mut adsr1 = env::Adsr::new();
    adsr1["attack_for"] = 2048;
    adsr1["attack_to"] = i16::max_value();
    adsr1["decay_for"] = 1024;
    adsr1["sustain_at"] = i16::max_value() / 4 * 3;
    adsr1["release_for"] = 4096;

    tx2.send(Cmd::AdsrAttackFor(adsr1["attack_for"])).unwrap();
    tx2.send(Cmd::AdsrAttackTo(adsr1["attack_to"])).unwrap();
    tx2.send(Cmd::AdsrDecayFor(adsr1["decay_for"])).unwrap();
    tx2.send(Cmd::AdsrSustainAt(adsr1["sustain_at"])).unwrap();
    tx2.send(Cmd::AdsrReleaseFor(adsr1["release_for"])).unwrap();
    println!("in audio release_for {}", adsr1["release_for"]);

    let mut beats = [false; 16];
    beats[0] = true;
    beats[4] = true;
    beats[12] = true;
    for (i, b) in beats.iter().enumerate() {
        tx2.send(Cmd::Beat(i as i16, *b)).unwrap()
    }
    let beats = Arc::new(RwLock::new(beats));
    let mut beat_len = Arc::new(RwLock::new([128; 16]));
    let mut seq1 = seq::BasicSeq::new(Arc::clone(&beats), Arc::clone(&beat_len));

    let mut vca1o = amp::Vca::new(i16::max_value());

    let mut adsr1o = env::Adsr::new();
    adsr1o["attack_for"] = 2048;
    adsr1o["attack_to"] = i16::max_value() / 4 * 3;
    adsr1o["decay_for"] = 1024;
    adsr1o["sustain_at"] = i16::max_value() / 2;
    adsr1o["release_for"] = 4096 * 4;

    let mut obeats = [false; 16];
    //obeats[2] = true;
    //obeats[6] = true;
    //obeats[10] = true;
    //obeats[14] = true;
    for (i, b) in obeats.iter().enumerate() {
        tx2.send(Cmd::Obeat(i as i16, *b)).unwrap()
    }
    let obeats = Arc::new(RwLock::new(obeats));
    let mut obeat_len = Arc::new(RwLock::new([64; 16]));
    let mut seq1o = seq::BasicSeq::new(Arc::clone(&obeats), Arc::clone(&obeat_len));

    let mut mix1 = mix::Mixer::new();
    mix1["a_lvl"] = i16::max_value();
    mix1["b_lvl"] = i16::max_value();

    let mut arp1 = arp::BasicArp::new();
    arp1.notes = arp::TtetNote::C.major_scale();
    arp1.octave = 3;
    tx2.send(Cmd::Scale(arp::TtetNote::C)).unwrap();

    let mut arp1o = arp::BasicArp::new();
    arp1o.notes = arp::TtetNote::Fs.major_scale();

    let mut rvb1 = rvb::BasicReverb::new();
    rvb1.delay[0] = (RATE / 3) as i16;
    rvb1.delay[1] = (RATE / 5) as i16;
    rvb1.delay[2] = (RATE / 8) as i16;

    let mut components: Vec<(&str, AvailableComponents)> = vec![
        ("wto1", AvailableComponents::WaveTableOsc(wto1)),
        ("wto1o", AvailableComponents::WaveTableOsc(wto1o)),
        ("wto2", AvailableComponents::WaveTableOsc(wto2)),
        ("vca1", AvailableComponents::Vca(vca1)),
        ("adsr1", AvailableComponents::Adsr(adsr1)),
        ("seq1", AvailableComponents::BasicSeq(seq1)),
        ("vca1o", AvailableComponents::Vca(vca1o)),
        ("adsr1o", AvailableComponents::Adsr(adsr1o)),
        ("seq1o", AvailableComponents::BasicSeq(seq1o)),
        ("mix1", AvailableComponents::Mixer(mix1)),
        ("arp1", AvailableComponents::BasicArp(arp1)),
        ("arp1o", AvailableComponents::BasicArp(arp1o)),
        ("rvb1", AvailableComponents::BasicReverb(rvb1)),
    ];

    // Connect the modulation input of the first oscillator to the
    // output of the second.
    let wires = vec![
        //(("wto1", "out"), ("mix1", "a")),
        // (("wto2", "out"), ("wto1", "modulation")),
        //(("wto1", "out"), ("wto2", "modulation")),
        (("adsr1", "out"), ("vca1", "amp_cv")),
        (("wto1", "out"), ("vca1", "in_cv")),
        (("seq1", "trigger"), ("adsr1", "trigger")),
        (("seq1", "gate"), ("adsr1", "gate")),
        (("vca1", "out"), ("rvb1", "cv_in")),
        (("seq1", "trigger"), ("arp1", "trigger_in")),
        (("seq1", "gate"), ("arp1", "gate_in")),
        (("arp1", "note_cv_out"), ("wto1", "freq")),
        //(("adsr1o", "out"), ("vca1o", "amp_cv")),
        //(("wto1o", "out"), ("vca1o", "in_cv")),
        //(("seq1o", "trigger"), ("adsr1o", "trigger")),
        //(("seq1o", "gate"), ("adsr1o", "gate")),
        //(("vca1o", "out"), ("mix1", "b")),
        //(("seq1o", "trigger"), ("arp1o", "trigger_in")),
        //(("seq1o", "gate"), ("arp1o", "gate_in")),
        //(("arp1o", "note_cv_out"), ("wto1o", "freq")),
    ];

    // Sanity Check of the wires.
    for (src, dst) in wires.iter() {
        if let None = components.iter().position(|x| x.0 == src.0) {
            println!("{} not found for {:?}, {:?}", src.0, src, dst);
            return None;
        }
        if let None = components.iter().position(|x| x.0 == dst.0) {
            println!("{} not found for {:?}, {:?}", dst.0, src, dst);
            return None;
        }
    }
    let tempo = 90;
    let cycles_per_16th = ((60. / ((4 * tempo) as f64)) * (util::RATE as f64)) as u64;
    let mut cycle_counter = 0;
    let next_sample = move || -> i16 {
        match rx.try_recv() {
            Ok(c) => match c {
                Cmd::AdsrAttackFor(v) => {
                    if let Some(j) = components.iter().position(|x| x.0 == "adsr1") {
                        match &mut components.get_mut(j).unwrap().1 {
                            AvailableComponents::Adsr(adsr) => {
                                adsr.attack_for = v;
                                tx2.send(c).unwrap();
                            }
                            _ => (),
                        }
                    }
                }
                Cmd::AdsrAttackTo(v) => {
                    if let Some(j) = components.iter().position(|x| x.0 == "adsr1") {
                        match &mut components.get_mut(j).unwrap().1 {
                            AvailableComponents::Adsr(adsr) => {
                                adsr.attack_to = v;
                                tx2.send(c).unwrap();
                            }
                            _ => (),
                        }
                    }
                }
                Cmd::AdsrDecayFor(v) => {
                    if let Some(j) = components.iter().position(|x| x.0 == "adsr1") {
                        match &mut components.get_mut(j).unwrap().1 {
                            AvailableComponents::Adsr(adsr) => {
                                adsr.decay_for = v;
                                tx2.send(c).unwrap();
                            }
                            _ => (),
                        }
                    }
                }
                Cmd::AdsrSustainAt(v) => {
                    if let Some(j) = components.iter().position(|x| x.0 == "adsr1") {
                        match &mut components.get_mut(j).unwrap().1 {
                            AvailableComponents::Adsr(adsr) => {
                                adsr.sustain_at = v;
                                tx2.send(c).unwrap();
                            }
                            _ => (),
                        }
                    }
                }
                Cmd::AdsrReleaseFor(v) => {
                    if let Some(j) = components.iter().position(|x| x.0 == "adsr1") {
                        match &mut components.get_mut(j).unwrap().1 {
                            AvailableComponents::Adsr(adsr) => {
                                adsr.release_for = v;
                                tx2.send(c).unwrap();
                            }
                            _ => (),
                        }
                    }
                }
                Cmd::Freq(f) => components[0].1["freq"] = f,
                Cmd::Scale(n) => {
                    if let Some(j) = components.iter().position(|x| x.0 == "arp1") {
                        match &mut components.get_mut(j).unwrap().1 {
                            AvailableComponents::BasicArp(arp) => {
                                arp.notes = n.major_scale();
                                tx2.send(Cmd::Scale(n)).unwrap();
                            }
                            _ => (),
                        }
                    }
                }
                Cmd::FileWaveTable(scwf) => {
                    if let Some(j) = components.iter().position(|x| x.0 == "wto1") {
                        match &mut components.get_mut(j).unwrap().1 {
                            AvailableComponents::WaveTableOsc(wt) => {
                                wt.load_scwf(scwf.path);
                            }
                            _ => (),
                        }
                    }
                }
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
            Err(TryRecvError::Disconnected) => return 0,
        }
        let mut tick = false;
        cycle_counter += 1;
        if cycle_counter >= cycles_per_16th {
            cycle_counter = 0;
            tick = true;
            //if let Some(j) = components.iter().position(|x| x.0 == dst.0) {
            //    components[j].1[dst.1] = components[i].1[src.1];
            //}
        }

        for component in components.iter_mut() {
            component.1.step();
        }

        if tick {
            if let Some(j) = components.iter().position(|x| x.0 == "seq1") {
                setbeat.store(components[j].1["beat"], Ordering::Relaxed);
            }
            for component in components.iter_mut() {
                component.1.tick();
            }
        }

        for (src, dst) in wires.iter() {
            if let Some(i) = components.iter().position(|x| x.0 == src.0) {
                if let Some(j) = components.iter().position(|x| x.0 == dst.0) {
                    components[j].1[dst.1] = components[i].1[src.1];
                }
            }
        }
        if let Some(j) = components.iter().position(|x| x.0 == "rvb1") {
            components[j].1["out"]
        } else {
            0
        }
    };
    Some(out::CpalOut::from_defaults(next_sample).unwrap())
}
