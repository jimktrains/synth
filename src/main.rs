#[macro_use]
extern crate lazy_static;

use std::error::Error;
use std::sync::atomic::AtomicI16;
use std::sync::atomic::AtomicU64;
use std::sync::mpsc::channel;
use std::sync::Arc;

mod amp;
mod arp;
mod audio;
mod env;
mod fixed;
mod mix;
mod osc;
mod out;
mod seq;
mod tui_util;
mod ui;
mod util;

use crate::audio::spawn_audio;
use crate::ui::ui_loop;

fn main() -> Result<(), Box<dyn Error>> {
    let (tx, rx) = channel();
    let (tx2, rx2) = channel();

    let beat = Arc::new(AtomicI16::new(-1));
    let setbeat = Arc::clone(&beat);

    let measured_xtime = Arc::new(AtomicU64::new(0));
    let set_measured_xtime = Arc::clone(&measured_xtime);

    let target_inc = (1_000_000_000. / (util::RATE as f64)) as u128; //22675; // (1/44100 * 10^9) ns //(((util::RATE as u64) / 100) as u64;

    let cpalOut = spawn_audio(rx, tx2, setbeat, set_measured_xtime, target_inc);

    ui_loop(tx, rx2, beat, measured_xtime, target_inc)
}
