//use std::fs::File;
//use std::io::Error;
//use std::path::Path;
//use wav;

use std::sync::mpsc::TryRecvError;
use std::time;

use std::cell::Cell;
use std::sync::atomic::AtomicI8;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;

use argh::FromArgs;
use std::{error::Error, io, time::Duration};
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Span, Spans},
    widgets::canvas::{Canvas, Line, Map, MapResolution, Rectangle},
    widgets::{
        Axis, BarChart, Block, Borders, Chart, Dataset, Gauge, LineGauge, List, ListItem,
        Paragraph, Row, Sparkline, Table, Tabs, Wrap,
    },
    Frame,
};
use tui::{backend::TermionBackend, Terminal};

mod amp;
mod env;
mod fixed;
mod mix;
mod osc;
mod out;
mod seq;
mod tui_util;
mod util;
use std::sync::mpsc::channel;

use crate::tui_util::{Config, Event, Events};

fn main() -> Result<(), Box<dyn Error>> {
    let (tx, rx) = channel();
    //let (beat_tx, beat_rx) = channel();
    let beat = Arc::new(AtomicI8::new(-1));
    let setbeat = Arc::clone(&beat);

    let measured_delay = Arc::new(AtomicU64::new(0));
    let set_measured_delay = Arc::clone(&measured_delay);

    let measured_xtime = Arc::new(AtomicU64::new(0));
    let set_measured_xtime = Arc::clone(&measured_xtime);

    let delay_delta = Arc::new(AtomicU64::new(0));
    let set_delay_delta = Arc::clone(&delay_delta);

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

    let mut seq1 = seq::BasicSeq::new();
    seq1.beats[0] = true;
    seq1.beats[4] = true;
    seq1.beats[12] = true;
    seq1["tempo"] = 120;

    let mut vca1o = amp::Vca::new(i8::max_value());

    let mut adsr1o = env::Adsr::new();
    adsr1o["attack_for"] = 50;
    adsr1o["attack_to"] = i8::max_value() / 2;
    adsr1o["decay_for"] = 15;
    adsr1o["sustain_at"] = i8::max_value() / 2;
    adsr1o["release_for"] = 15;

    let mut seq1o = seq::BasicSeq::new();
    seq1o.beats[2] = true;
    seq1o.beats[6] = true;
    seq1o.beats[10] = true;
    seq1o.beats[14] = true;
    seq1o["tempo"] = 90;

    let mut mix1 = mix::Mixer::new();
    mix1["a_lvl"] = i8::max_value();
    mix1["b_lvl"] = i8::max_value();

    let mut out1 = out::CpalOut::from_defaults().unwrap();
    let cv_in_r = Arc::new(mix1["out"]);
    //out1.cv_in_r = Some(cv_in_r);

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
            return Ok(());
        }
        if let None = names.iter().position(|&x| x == dst.0) {
            println!("{} not found for {:?}, {:?}", dst.0, src, dst);
            return Ok(());
        }
    }

    #[derive(Debug, FromArgs)]
    #[argh(description = "options")]
    struct Cli {
        /// time in ms between two ticks.
        #[argh(option, default = "65", description = "tick rate in ms")]
        tick_rate: u64,
        /// whether unicode symbols are used to improve the overall look of the app
        #[argh(option, default = "true", description = "unicode?")]
        enhanced_graphics: bool,
    }

    let cli: Cli = argh::from_env();

    let events = Events::with_config(Config {
        tick_rate: Duration::from_millis(cli.tick_rate),
        ..Config::default()
    });

    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    thread::spawn(move || {
        let mut counter = 0;
        let mut delays = [0u64; 100];
        let mut xtimes = [0u64; 100];
        let mut beat_start = time::Instant::now();
        let mut comp_tempo = 0.;
        let mut prev_beat = 15;
        loop {
            let beat = (beat.load(Ordering::Relaxed) as usize) % 16;

            if beat == 0 && prev_beat != 0 {
                beat_start = time::Instant::now();
            } else if beat == 4 && prev_beat != 4 {
                comp_tempo = 60. * 1_000_000_000. / (beat_start.elapsed().as_nanos() as f64);
            }

            let measured_delay = measured_delay.load(Ordering::Relaxed);
            let measured_xtime = measured_xtime.load(Ordering::Relaxed);
            delays[counter % 100] = measured_delay;
            xtimes[counter % 100] = measured_xtime;
            terminal.draw(|f| {
                let delays_avg = delays.iter().sum::<u64>() as f64 / delays.len() as f64;
                let xtime_avg = xtimes.iter().sum::<u64>() as f64 / delays.len() as f64;
                let text = vec![
                    Spans::from("  | S |   | F | G | H |   | K |"),
                    Spans::from("| Z | X | C | V | B | N | M |"),
                    Spans::from(format!(
                        "{:02} {:5.4} ::::: {:10.4} {:10.4} {:10.4} ::::: {:10.4} {:10.4} {:10.4}",
                        beat,
                        comp_tempo,
                        measured_delay,
                        delays_avg,
                        22675. - delays_avg,
                        measured_xtime,
                        xtime_avg,
                        22675. - xtime_avg,
                    )),
                ];
                let block = Block::default().borders(Borders::ALL).title(Span::styled(
                    "Footer",
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ));
                let paragraph = Paragraph::new(text).block(block);
                let chunks = Layout::default()
                    .constraints([Constraint::Length(6), Constraint::Min(0)].as_ref())
                    .split(f.size());
                let area = chunks[0];
                f.render_widget(paragraph, area);

                let chunks = Layout::default()
                    .constraints(
                        [
                            Constraint::Length(3),
                            Constraint::Length(3),
                            Constraint::Length(3),
                        ]
                        .as_ref(),
                    )
                    .margin(1)
                    .split(chunks[1]);

                let mut beats = [1u64; 16];
                beats[beat] = 2;
                let sparkline = Sparkline::default()
                    .block(Block::default().title("Sparkline:"))
                    .style(Style::default().fg(Color::Green))
                    .data(&beats)
                    .bar_set(symbols::bar::NINE_LEVELS);
                f.render_widget(sparkline, chunks[0]);

                let sparkline = Sparkline::default()
                    .block(Block::default().title("loop time:"))
                    .style(Style::default().fg(Color::Green))
                    .data(&delays)
                    .bar_set(symbols::bar::NINE_LEVELS);
                f.render_widget(sparkline, chunks[1]);

                let sparkline = Sparkline::default()
                    .block(Block::default().title("loop xtime:"))
                    .style(Style::default().fg(Color::Green))
                    .data(&xtimes)
                    .bar_set(symbols::bar::NINE_LEVELS);
                f.render_widget(sparkline, chunks[2]);
            });
            counter += 1;

            match events.next().unwrap() {
                Event::Input(key) => match key {
                    Key::Char(c) => match c {
                        // A
                        'z' => tx.send(69).unwrap(),
                        // A# / Bb
                        's' => tx.send(70).unwrap(),
                        // B
                        'x' => tx.send(71).unwrap(),
                        // C
                        'c' => tx.send(72).unwrap(),
                        // C# /tx.send(
                        'f' => tx.send(73).unwrap(),
                        // D
                        'v' => tx.send(74).unwrap(),
                        // D# /tx.send(
                        'g' => tx.send(75).unwrap(),
                        // E
                        'b' => tx.send(76).unwrap(),
                        // F
                        'n' => tx.send(77).unwrap(),
                        // F# /tx.send(
                        'j' => tx.send(78).unwrap(),
                        // G
                        'm' => tx.send(79).unwrap(),
                        // G# / Ab
                        'k' => tx.send(80).unwrap(),

                        _ => (),
                    },
                    Key::Esc => break,
                    Key::Up => {}
                    Key::Down => {}
                    Key::Left => {}
                    Key::Right => {}
                    _ => {}
                },
                Event::Tick => {}
            }
            prev_beat = beat;
        }
        Ok::<(), String>(())
    });

    let mut exit = false;
    let target_inc = 22675u128; // (1/44100 * 10^9) ns //(((util::RATE as u64) / 100) as u64;
    let mut delay = 100;
    let mut start = time::Instant::now();
    let tempo = 90;
    let ns_per_16th = ((60. / ((4 * tempo) as f64)) * 1_000_000_000.) as u128;
    let mut tempo_start = time::Instant::now();
    while !exit {
        if start.elapsed().as_nanos() > 1750 {
            //(target_inc - 2 * delay) {

            start = time::Instant::now();

            let tick = tempo_start.elapsed().as_nanos() > (ns_per_16th - 500);
            if tick {
                tempo_start = start;
            }

            set_measured_delay.store(
                ((delay as f64) / (target_inc as f64) * 1000.) as u64,
                Ordering::Relaxed,
            );
            match rx.try_recv() {
                Ok(f) => components[0]["freq"] = f,
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => exit = true,
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

            // println!("{:?}", start.elapsed());
            //println!("{:?}", (end.elapsed() - start.elapsed()));
            set_measured_xtime.store(start.elapsed().as_nanos() as u64, Ordering::Relaxed);
            delay = start.elapsed().as_nanos();
        }
    }
    Ok(())
}
