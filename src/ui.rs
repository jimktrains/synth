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

use crate::tui_util::StatefulList;
use argh::FromArgs;
use std::{error::Error, io, time::Duration};
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Span, Spans, Text},
    widgets::canvas::{Canvas, Line, Map, MapResolution, Rectangle},
    widgets::{
        Axis, BarChart, Block, Borders, Chart, Dataset, Gauge, LineGauge, List, ListItem,
        Paragraph, Row, Sparkline, Table, Tabs, Wrap,
    },
    Frame,
};
use tui::{backend::TermionBackend, Terminal};

use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

use crate::arp::TtetNote;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct SingleCycleWaveFormItem {
    pub name: String,
    pub path: PathBuf,
}

impl<'a> From<&SingleCycleWaveFormItem> for Text<'a> {
    fn from(s: &SingleCycleWaveFormItem) -> Text<'a> {
        s.name.to_owned().into()
    }
}

use crate::tui_util::{Config, Event, Events};

pub enum Cmd {
    Freq(i16),
    Beat(i16, bool),
    Obeat(i16, bool),
    FileWaveTable(SingleCycleWaveFormItem),
    Scale(TtetNote), // Major Scale only right now, and only octave 4
    AdsrAttackFor(i16),
    AdsrAttackTo(i16),
    AdsrDecayFor(i16),
    AdsrSustainAt(i16),
    AdsrReleaseFor(i16),
}

pub fn ui_loop(
    tx: Sender<Cmd>,
    rx2: Receiver<Cmd>,
    beat: Arc<AtomicI16>,
    measured_xtime: Arc<AtomicU64>,
    target_inc: u128,
    single_cycle_wave_forms: &Vec<SingleCycleWaveFormItem>,
) -> Result<(), Box<dyn Error>> {
    let mut arp1_scale = TtetNote::A;
    let mut scwf_state = StatefulList::with_items(single_cycle_wave_forms);

    let mut last_scwf_i = usize::max_value();

    let mut adsr_attack_for = 0i16;
    let mut adsr_attack_to = 0i16;
    let mut adsr_decay_for = 0i16;
    let mut adsr_sustain_at = 0i16;
    let mut adsr_release_for = 0i16;

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
                    Spans::from(format!("arp1.scale={} a=Up z=Down", arp1_scale)),
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
                            Constraint::Length(2),
                            Constraint::Length(2),
                            Constraint::Length(2),
                            Constraint::Length(15),
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

                let mut cur_beats = [0u64; 16];
                cur_beats[beat] = 1;
                let sparkline = Sparkline::default()
                    .block(Block::default().title("Current Beat:"))
                    .style(Style::default().fg(Color::Green))
                    .data(&cur_beats)
                    .bar_set(symbols::bar::NINE_LEVELS);
                f.render_widget(sparkline, chunks[2]);
                let sparkline = Sparkline::default()
                    .block(Block::default().title("Accented Beats:"))
                    .style(Style::default().fg(Color::Green))
                    .data(&beats)
                    .bar_set(symbols::bar::NINE_LEVELS);
                f.render_widget(sparkline, chunks[3]);

                let sparkline = Sparkline::default()
                    .block(Block::default().title("Beats:"))
                    .style(Style::default().fg(Color::Green))
                    .data(&obeats)
                    .bar_set(symbols::bar::NINE_LEVELS);
                f.render_widget(sparkline, chunks[4]);

                let sparkline = Sparkline::default()
                    .block(Block::default().title("loop xtime:"))
                    .style(Style::default().fg(Color::Green))
                    .data(&xtimes)
                    .bar_set(symbols::bar::NINE_LEVELS);
                f.render_widget(sparkline, chunks[5]);

                let vchunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Length(30), Constraint::Length(30)].as_ref())
                    .margin(1)
                    .split(chunks[6]);

                let sswf_list = List::new(
                    single_cycle_wave_forms
                        .iter()
                        .map(|x| ListItem::new(x))
                        .collect::<Vec<ListItem>>(),
                )
                .block(Block::default().title("List").borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
                .highlight_symbol(">>");
                f.render_stateful_widget(sswf_list, vchunks[0], &mut scwf_state.state);

                let hchunks = Layout::default()
                    .constraints(
                        [
                            Constraint::Length(1),
                            Constraint::Length(1),
                            Constraint::Length(1),
                            Constraint::Length(1),
                            Constraint::Length(1),
                        ]
                        .as_ref(),
                    )
                    .margin(1)
                    .split(vchunks[1]);

                let adsr_attack_for_guage = Gauge::default()
                    .ratio(adsr_attack_for as f64 / i16::max_value() as f64)
                    .label("Attach For (d/c)")
                    .gauge_style(Style::default().fg(Color::White));
                f.render_widget(adsr_attack_for_guage, hchunks[0]);

                let adsr_attack_to_guage = Gauge::default()
                    .ratio(adsr_attack_to as f64 / i16::max_value() as f64)
                    .label("Attach To (f/v)")
                    .gauge_style(Style::default().fg(Color::White));
                f.render_widget(adsr_attack_to_guage, hchunks[1]);

                let adsr_decay_for_guage = Gauge::default()
                    .ratio(adsr_decay_for as f64 / i16::max_value() as f64)
                    .label("Decay For (g/b)")
                    .gauge_style(Style::default().fg(Color::White));
                f.render_widget(adsr_decay_for_guage, hchunks[2]);
                let adsr_sustain_at_guage = Gauge::default()
                    .ratio(adsr_sustain_at as f64 / i16::max_value() as f64)
                    .label("Sustain at (h/n)")
                    .gauge_style(Style::default().fg(Color::White));
                f.render_widget(adsr_sustain_at_guage, hchunks[3]);
                let adsr_release_for_guage = Gauge::default()
                    .ratio(adsr_release_for as f64 / i16::max_value() as f64)
                    .label(format!("Release For (j/m) {}", adsr_release_for))
                    .gauge_style(Style::default().fg(Color::White));
                f.render_widget(adsr_release_for_guage, hchunks[4]);

                if let Some(i) = scwf_state.state.selected() {
                    if last_scwf_i != i {
                        tx.send(Cmd::FileWaveTable(single_cycle_wave_forms[i].clone()))
                            .unwrap();

                        last_scwf_i = i;
                    }
                }
            })
            .unwrap();

        counter += 1;

        match events.next().unwrap() {
            Event::Input(key) => match key {
                Key::Char(c) => match c {
                    'a' => tx.send(Cmd::Scale(arp1_scale + 1)).unwrap(),
                    'z' => tx.send(Cmd::Scale(arp1_scale - 1)).unwrap(),

                    'd' => tx
                        .send(Cmd::AdsrAttackFor(
                            adsr_attack_for.saturating_add(i16::max_value() / 100),
                        ))
                        .unwrap(),
                    'c' => tx
                        .send(Cmd::AdsrAttackFor(
                            // Casting to u16 because I want to saturate to 0.
                            (adsr_attack_for as u16).saturating_sub(i16::max_value() as u16 / 100)
                                as i16,
                        ))
                        .unwrap(),

                    'f' => tx
                        .send(Cmd::AdsrAttackTo(
                            adsr_attack_to.saturating_add(i16::max_value() / 100),
                        ))
                        .unwrap(),
                    'v' => tx
                        .send(Cmd::AdsrAttackTo(
                            // Casting to u16 because I want to saturate to 0.
                            (adsr_attack_to as u16).saturating_sub(i16::max_value() as u16 / 100)
                                as i16,
                        ))
                        .unwrap(),

                    'g' => tx
                        .send(Cmd::AdsrDecayFor(
                            adsr_decay_for.saturating_add(i16::max_value() / 100),
                        ))
                        .unwrap(),
                    'b' => tx
                        .send(Cmd::AdsrDecayFor(
                            // Casting to u16 because I want to saturate to 0.
                            (adsr_decay_for as u16).saturating_sub(i16::max_value() as u16 / 100)
                                as i16,
                        ))
                        .unwrap(),

                    'h' => tx
                        .send(Cmd::AdsrSustainAt(
                            adsr_sustain_at.saturating_add(i16::max_value() / 100),
                        ))
                        .unwrap(),
                    'n' => tx
                        .send(Cmd::AdsrSustainAt(
                            // Casting to u16 because I want to saturate to 0.
                            (adsr_sustain_at as u16).saturating_sub(i16::max_value() as u16 / 100)
                                as i16,
                        ))
                        .unwrap(),

                    'j' => tx
                        .send(Cmd::AdsrReleaseFor(
                            adsr_release_for.saturating_add(i16::max_value() / 100),
                        ))
                        .unwrap(),
                    'm' => tx
                        .send(Cmd::AdsrReleaseFor(
                            // Casting to u16 because I want to saturate to 0.
                            (adsr_release_for as u16).saturating_sub(i16::max_value() as u16 / 100)
                                as i16,
                        ))
                        .unwrap(),

                    '!' => tx.send(Cmd::Beat(0, 0 == beats[0])).unwrap(),
                    '@' => tx.send(Cmd::Beat(1, 0 == beats[1])).unwrap(),
                    '#' => tx.send(Cmd::Beat(2, 0 == beats[2])).unwrap(),
                    '$' => tx.send(Cmd::Beat(3, 0 == beats[3])).unwrap(),
                    '%' => tx.send(Cmd::Beat(4, 0 == beats[4])).unwrap(),
                    '^' => tx.send(Cmd::Beat(5, 0 == beats[5])).unwrap(),
                    '&' => tx.send(Cmd::Beat(6, 0 == beats[6])).unwrap(),
                    '*' => tx.send(Cmd::Beat(7, 0 == beats[7])).unwrap(),
                    'Q' => tx.send(Cmd::Beat(8, 0 == beats[8])).unwrap(),
                    'W' => tx.send(Cmd::Beat(9, 0 == beats[9])).unwrap(),
                    'E' => tx.send(Cmd::Beat(10, 0 == beats[10])).unwrap(),
                    'R' => tx.send(Cmd::Beat(11, 0 == beats[11])).unwrap(),
                    'T' => tx.send(Cmd::Beat(12, 0 == beats[12])).unwrap(),
                    'Y' => tx.send(Cmd::Beat(13, 0 == beats[13])).unwrap(),
                    'U' => tx.send(Cmd::Beat(14, 0 == beats[14])).unwrap(),
                    'I' => tx.send(Cmd::Beat(15, 0 == beats[15])).unwrap(),

                    '1' => tx.send(Cmd::Obeat(0, 0 == obeats[0])).unwrap(),
                    '2' => tx.send(Cmd::Obeat(1, 0 == obeats[1])).unwrap(),
                    '3' => tx.send(Cmd::Obeat(2, 0 == obeats[2])).unwrap(),
                    '4' => tx.send(Cmd::Obeat(3, 0 == obeats[3])).unwrap(),
                    '5' => tx.send(Cmd::Obeat(4, 0 == obeats[4])).unwrap(),
                    '6' => tx.send(Cmd::Obeat(5, 0 == obeats[5])).unwrap(),
                    '7' => tx.send(Cmd::Obeat(6, 0 == obeats[6])).unwrap(),
                    '8' => tx.send(Cmd::Obeat(7, 0 == obeats[7])).unwrap(),
                    'q' => tx.send(Cmd::Obeat(8, 0 == obeats[8])).unwrap(),
                    'w' => tx.send(Cmd::Obeat(9, 0 == obeats[9])).unwrap(),
                    'e' => tx.send(Cmd::Obeat(10, 0 == obeats[10])).unwrap(),
                    'r' => tx.send(Cmd::Obeat(11, 0 == obeats[11])).unwrap(),
                    't' => tx.send(Cmd::Obeat(12, 0 == obeats[12])).unwrap(),
                    'y' => tx.send(Cmd::Obeat(13, 0 == obeats[13])).unwrap(),
                    'u' => tx.send(Cmd::Obeat(14, 0 == obeats[14])).unwrap(),
                    'i' => tx.send(Cmd::Obeat(15, 0 == obeats[15])).unwrap(),

                    _ => (),
                },
                Key::Esc => break,
                Key::Left => {}
                Key::Right => {}
                Key::PageDown => scwf_state.forward(10),
                Key::PageUp => scwf_state.backward(10),
                Key::Down => scwf_state.next(),
                Key::Up => scwf_state.previous(),
                _ => {}
            },
            Event::Tick => {}
        }
        loop {
            match rx2.try_recv() {
                Ok(c) => match c {
                    Cmd::Freq(_) => (),
                    Cmd::Beat(i, b) => beats[i as usize] = 1 * if b { 1 } else { 0 },
                    Cmd::Obeat(i, b) => obeats[i as usize] = 1 * if b { 1 } else { 0 },
                    Cmd::FileWaveTable(_) => (),
                    Cmd::Scale(n) => arp1_scale = n,
                    Cmd::AdsrAttackFor(v) => adsr_attack_for = v,
                    Cmd::AdsrAttackTo(v) => adsr_attack_to = v,
                    Cmd::AdsrDecayFor(v) => adsr_decay_for = v,
                    Cmd::AdsrSustainAt(v) => adsr_sustain_at = v,
                    Cmd::AdsrReleaseFor(v) => adsr_release_for = v,
                },
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => return Ok(()),
            }
        }
        prev_beat = beat;
    }
    Ok(())
}
