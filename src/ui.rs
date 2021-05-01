//use std::fs::File;
//use std::io::Error;
//use std::path::Path;
//use wav;

use std::sync::mpsc::TryRecvError;
use std::time;

use std::sync::atomic::AtomicI16;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::RwLock;
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

use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

use crate::tui_util::{Config, Event, Events};

pub enum Cmd {
    Freq(i16),
    Beat(i16, bool),
    Obeat(i16, bool),
}

pub fn ui_loop(
    tx: Sender<Cmd>,
    rx2: Receiver<Cmd>,
    beat: Arc<AtomicI16>,
    measured_xtime: Arc<AtomicU64>,
    target_inc: u128,
) -> Result<(), Box<dyn Error>> {
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
    let mut beats = [0u64; 16];
    let mut obeats = [0u64; 16];
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut counter = 0;
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

        let measured_xtime = measured_xtime.load(Ordering::Relaxed);
        xtimes[counter % 100] = measured_xtime;
        terminal
            .draw(|f| {
                let xtime_avg = xtimes.iter().sum::<u64>() as f64 / xtimes.len() as f64;
                let text = vec![
                    Spans::from("  | S |   | F | G | H |   | K |"),
                    Spans::from("| Z | X | C | V | B | N | M |"),
                    Spans::from(""),
                    Spans::from(format!(
                        "{:02} {:5.4} ::::: {:10.4} {:10.4} {:10.4} ::::: {:10.4} {:10.4} {:10.4}",
                        beat,
                        comp_tempo,
                        0,
                        0,
                        0,
                        measured_xtime,
                        xtime_avg,
                        ((256 * target_inc) as f64) - xtime_avg,
                    )),
                ];
                let block = Block::default().borders(Borders::ALL).title(Span::styled(
                    "Pitch",
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ));
                let paragraph = Paragraph::new(text).block(block);
                let chunks = Layout::default()
                    .constraints(
                        [
                            Constraint::Length(6),
                            Constraint::Length(6),
                            Constraint::Min(0),
                        ]
                        .as_ref(),
                    )
                    .split(f.size());
                f.render_widget(paragraph, chunks[0]);

                let text = vec![
                    Spans::from("1 2 3 4 | 5 6 7 8"),
                    Spans::from(" q w e r | t y u i"),
                    Spans::from(" + Shift for accent"),
                ];
                let block = Block::default().borders(Borders::ALL).title(Span::styled(
                    "Beat",
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ));
                let paragraph = Paragraph::new(text).block(block);
                f.render_widget(paragraph, chunks[1]);

                let chunks = Layout::default()
                    .constraints(
                        [
                            Constraint::Length(2),
                            Constraint::Length(2),
                            Constraint::Length(2),
                            Constraint::Length(5),
                        ]
                        .as_ref(),
                    )
                    .margin(1)
                    .split(chunks[2]);

                let mut cur_beats = [0u64; 16];
                cur_beats[beat] = 1;
                let sparkline = Sparkline::default()
                    .block(Block::default().title("Current Beat:"))
                    .style(Style::default().fg(Color::Green))
                    .data(&cur_beats)
                    .bar_set(symbols::bar::NINE_LEVELS);
                f.render_widget(sparkline, chunks[0]);
                let sparkline = Sparkline::default()
                    .block(Block::default().title("Accented Beats:"))
                    .style(Style::default().fg(Color::Green))
                    .data(&beats)
                    .bar_set(symbols::bar::NINE_LEVELS);
                f.render_widget(sparkline, chunks[1]);

                let sparkline = Sparkline::default()
                    .block(Block::default().title("Beats:"))
                    .style(Style::default().fg(Color::Green))
                    .data(&obeats)
                    .bar_set(symbols::bar::NINE_LEVELS);
                f.render_widget(sparkline, chunks[2]);

                let sparkline = Sparkline::default()
                    .block(Block::default().title("loop xtime:"))
                    .style(Style::default().fg(Color::Green))
                    .data(&xtimes)
                    .bar_set(symbols::bar::NINE_LEVELS);
                f.render_widget(sparkline, chunks[3]);
            })
            .unwrap();
        counter += 1;

        match events.next().unwrap() {
            Event::Input(key) => match key {
                //Key::Char(c) => match c {
                //    // A
                //    'z' => tx.send(Cmd::Freq(69)).unwrap(),
                //    // A# / Bb
                //    's' => tx.send(Cmd::Freq(70)).unwrap(),
                //    // B
                //    'x' => tx.send(Cmd::Freq(71)).unwrap(),
                //    // C
                //    'c' => tx.send(Cmd::Freq(72)).unwrap(),
                //    // C# /tx.send(Cmd::Freq(
                //    'f' => tx.send(Cmd::Freq(73)).unwrap(),
                //    // D
                //    'v' => tx.send(Cmd::Freq(74)).unwrap(),
                //    // D# /tx.send(Cmd::Freq(
                //    'g' => tx.send(Cmd::Freq(75)).unwrap(),
                //    // E
                //    'b' => tx.send(Cmd::Freq(76)).unwrap(),
                //    // F
                //    'n' => tx.send(Cmd::Freq(77)).unwrap(),
                //    // F# /tx.send(Cmd::Freq(
                //    'j' => tx.send(Cmd::Freq(78)).unwrap(),
                //    // G
                //    'm' => tx.send(Cmd::Freq(79)).unwrap(),
                //    // G# / Ab
                //    'k' => tx.send(Cmd::Freq(80)).unwrap(),

                //    '!' => tx.send(Cmd::Beat(0, 0 == beats[0])).unwrap(),
                //    '@' => tx.send(Cmd::Beat(1, 0 == beats[1])).unwrap(),
                //    '#' => tx.send(Cmd::Beat(2, 0 == beats[2])).unwrap(),
                //    '$' => tx.send(Cmd::Beat(3, 0 == beats[3])).unwrap(),
                //    '%' => tx.send(Cmd::Beat(4, 0 == beats[4])).unwrap(),
                //    '^' => tx.send(Cmd::Beat(5, 0 == beats[5])).unwrap(),
                //    '&' => tx.send(Cmd::Beat(6, 0 == beats[6])).unwrap(),
                //    '*' => tx.send(Cmd::Beat(7, 0 == beats[7])).unwrap(),
                //    'Q' => tx.send(Cmd::Beat(8, 0 == beats[8])).unwrap(),
                //    'W' => tx.send(Cmd::Beat(9, 0 == beats[9])).unwrap(),
                //    'E' => tx.send(Cmd::Beat(10, 0 == beats[10])).unwrap(),
                //    'R' => tx.send(Cmd::Beat(11, 0 == beats[11])).unwrap(),
                //    'T' => tx.send(Cmd::Beat(12, 0 == beats[12])).unwrap(),
                //    'Y' => tx.send(Cmd::Beat(13, 0 == beats[13])).unwrap(),
                //    'U' => tx.send(Cmd::Beat(14, 0 == beats[14])).unwrap(),
                //    'I' => tx.send(Cmd::Beat(15, 0 == beats[15])).unwrap(),

                //    '1' => tx.send(Cmd::Obeat(0, 0 == obeats[0])).unwrap(),
                //    '2' => tx.send(Cmd::Obeat(1, 0 == obeats[1])).unwrap(),
                //    '3' => tx.send(Cmd::Obeat(2, 0 == obeats[2])).unwrap(),
                //    '4' => tx.send(Cmd::Obeat(3, 0 == obeats[3])).unwrap(),
                //    '5' => tx.send(Cmd::Obeat(4, 0 == obeats[4])).unwrap(),
                //    '6' => tx.send(Cmd::Obeat(5, 0 == obeats[5])).unwrap(),
                //    '7' => tx.send(Cmd::Obeat(6, 0 == obeats[6])).unwrap(),
                //    '8' => tx.send(Cmd::Obeat(7, 0 == obeats[7])).unwrap(),
                //    'q' => tx.send(Cmd::Obeat(8, 0 == obeats[8])).unwrap(),
                //    'w' => tx.send(Cmd::Obeat(9, 0 == obeats[9])).unwrap(),
                //    'e' => tx.send(Cmd::Obeat(10, 0 == obeats[10])).unwrap(),
                //    'r' => tx.send(Cmd::Obeat(11, 0 == obeats[11])).unwrap(),
                //    't' => tx.send(Cmd::Obeat(12, 0 == obeats[12])).unwrap(),
                //    'y' => tx.send(Cmd::Obeat(13, 0 == obeats[13])).unwrap(),
                //    'u' => tx.send(Cmd::Obeat(14, 0 == obeats[14])).unwrap(),
                //    'i' => tx.send(Cmd::Obeat(15, 0 == obeats[15])).unwrap(),

                //    _ => (),
                //},
                Key::Esc => break,
                Key::Up => {}
                Key::Down => {}
                Key::Left => {}
                Key::Right => {}
                _ => {}
            },
            Event::Tick => {}
        }
        //match rx2.try_recv() {
        //    Ok(c) => match c {
        //        Cmd::Freq(_) => (),
        //        Cmd::Beat(i, b) => beats[i as usize] = 1 * if b { 1 } else { 0 },
        //        Cmd::Obeat(i, b) => obeats[i as usize] = 1 * if b { 1 } else { 0 },
        //    },
        //    Err(TryRecvError::Empty) => (),
        //    Err(TryRecvError::Disconnected) => break,
        //}
        prev_beat = beat;
    }
    Ok(())
}
