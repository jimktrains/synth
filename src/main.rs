use std::fs::File;
use std::io::Error;
use std::path::Path;
use wav;

mod amp;
mod env;
mod osc;
mod seq;
mod util;

fn main() -> Result<(), Error> {
    let tempo = 45.;
    println!(
        "{} bpm = {} ",
        tempo,
        util::quarter_point_per_32nd_node(tempo)
    );
    let tempo = 60.;
    println!(
        "{} bpm = {} ",
        tempo,
        util::quarter_point_per_32nd_node(tempo)
    );
    let tempo = 90.;
    println!(
        "{} bpm = {} ",
        tempo,
        util::quarter_point_per_32nd_node(tempo)
    );
    let tempo = 120.;
    println!(
        "{} bpm = {} ",
        tempo,
        util::quarter_point_per_32nd_node(tempo)
    );

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

    println!(
        "max err = {}",
        ipc_64_e_map.iter().map(|x| x.abs()).max().unwrap()
    );
    println!(
        "avg err = {}",
        (ipc_64_e_map.iter().map(|x| x.abs()).sum::<i16>() as f64) / (ipc_64_e_map.len() as f64)
    );

    let names = vec![
        "wto1", "wto2", "vca1", "adsr1", "seq1", "vca1o", "adsr1o", "seq1o",
    ];
    let mut wto1 = osc::WaveTableOsc::saw(ipc_64_map[69]);
    wto1.modulation_idx = 32;

    let mut wto2 = osc::FuncOsc::triangle(ipc_64_map[33]);
    let mut vca1 = amp::Vca::new(i8::max_value());

    let mut adsr1 = env::Adsr::new();
    adsr1["attack_for"] = 10;
    adsr1["attack_to"] = i8::max_value();
    adsr1["decay_for"] = 10;
    adsr1["sustain_at"] = i8::max_value() / 2;
    adsr1["release_for"] = 10;

    let mut seq1 = seq::BasicSeq::new();
    seq1.beats[0] = true;
    seq1.beats[4] = true;
    seq1.beats[12] = true;
    seq1["tempo"] = 120;

    let mut vca1o = amp::Vca::new(i8::max_value());

    let mut adsr1o = env::Adsr::new();
    adsr1o["attack_for"] = 15;
    adsr1o["attack_to"] = i8::max_value() / 4 * 3;
    adsr1o["decay_for"] = 15;
    adsr1o["sustain_at"] = i8::max_value() / 2;
    adsr1o["release_for"] = 15;

    let mut seq1o = seq::BasicSeq::new();
    seq1o.beats[1] = true;
    seq1o.beats[2] = true;
    seq1o.beats[5] = true;
    seq1o.beats[6] = true;
    seq1o.beats[13] = true;
    seq1o.beats[14] = true;
    seq1o["tempo"] = 120;

    let mut components: Vec<&mut dyn util::Component> = vec![
        &mut wto1,
        &mut wto2,
        &mut vca1,
        &mut adsr1,
        &mut seq1,
        &mut vca1o,
        &mut adsr1o,
        &mut seq1o,
    ];

    // Connect the modulation input of the first oscillator to the
    // output of the second.
    let wires = vec![
        (("wto2", "out"), ("wto1", "modulation")),
        (("adsr1", "out"), ("vca1", "amp_cv")),
        (("wto1", "out"), ("vca1", "in_cv")),
        (("seq1", "trigger"), ("adsr1", "trigger")),
        (("seq1", "gate"), ("adsr1", "gate")),
        (("adsr1o", "out"), ("vca1o", "amp_cv")),
        (("wto1", "out"), ("vca1o", "in_cv")),
        (("seq1o", "trigger"), ("adsr1o", "trigger")),
        (("seq1o", "gate"), ("adsr1o", "gate")),
    ];

    // Sanity Check of the wires.
    for (src, dst) in wires.iter() {
        if let None = names.iter().position(|&x| x == src.0) {
            println!("{} not found for {:?}, {:?}", src.0, src, dst);
            return Ok(());
        }
        if let None = names.iter().position(|&x| x == dst.0) {
            println!("{} not found for {:?}, {:?}", dst.0, src, dst);
            return Ok(());
        }
    }

    let len = (util::RATE as usize) * 8;
    let mut data: Vec<u8> = vec![0; len * 10];
    let mut track_counter = 0;
    for cnt in 0..len {
        track_counter = 0;

        // Increment all the components.
        for component in components.iter_mut() {
            component.step();
            for o in component.outputs() {
                let mut d = component[o] as i16;
                // The wav lib wants an unsigned value, but audacity expects
                // a signed value.
                d += (i8::min_value() as i16).abs();
                data[(10 * cnt) + track_counter] = d as u8;
                track_counter += 1;
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
    println!("{} tracks", track_counter);
    let mut out_file = File::create(Path::new("output.wav"))?;
    let header = wav::Header::new(1, track_counter as u16, util::RATE.into(), 8);
    wav::write(header, &wav::BitDepth::Eight(data), &mut out_file)?;

    println!("Wrote");
    Ok(())
}
