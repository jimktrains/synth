#[macro_use]
extern crate lazy_static;

use std::error::Error;
use std::io::Result as IoResult;
use std::sync::atomic::AtomicI16;
use std::sync::atomic::AtomicU64;
use std::sync::mpsc::channel;
use std::sync::Arc;

use std::fs;
use std::fs::DirEntry;
use std::path::Path;

mod amp;
mod arp;
mod audio;
mod env;
mod fixed;
mod mix;
mod osc;
mod out;
mod rvb;
mod seq;
mod tui_util;
mod ui;
mod util;

use crate::audio::spawn_audio;
use crate::ui::ui_loop;
use crate::ui::SingleCycleWaveFormItem;

// one possible implementation of walking a directory only visiting files
fn visit_dirs(
    dir: &Path,
    cb: &dyn Fn(&DirEntry) -> Option<SingleCycleWaveFormItem>,
) -> IoResult<Vec<SingleCycleWaveFormItem>> {
    let mut files = vec![];
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                files.append(&mut visit_dirs(&path, cb)?);
            } else {
                let x = cb(&entry);
                if let Some(y) = x {
                    files.push(y);
                }
            }
        }
    }
    Ok(files)
}

fn main() -> Result<(), Box<dyn Error>> {
    let path = Path::new("/home/jim/Downloads/AKWF/");

    let mut single_cycle_wave_forms =
        visit_dirs(path, &|x: &DirEntry| -> Option<SingleCycleWaveFormItem> {
            let path = x.path();
            if path.extension().unwrap() == "wav" {
                let parent = path
                    .parent()
                    .unwrap()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();

                let mut name = path.file_name().unwrap().to_str().unwrap().to_string();
                name = name[0..name.len() - 4].to_string();
                // The prefix is still in the parent. This just dedups the
                // value.
                if name.contains(&parent) {
                    name = parent.clone() + &name.replace(&parent, "");
                } else {
                    name = parent + &name.replace("AKWF", "");
                }
                return Some(SingleCycleWaveFormItem {
                    name: name,
                    path: path,
                });
            }
            None
        })
        .unwrap();
    single_cycle_wave_forms.sort();

    let (tx, rx) = channel();
    let (tx2, rx2) = channel();

    let beat = Arc::new(AtomicI16::new(-1));
    let setbeat = Arc::clone(&beat);

    let measured_xtime = Arc::new(AtomicU64::new(0));

    let target_inc = (1_000_000_000. / (util::RATE as f64)) as u128; //22675; // (1/44100 * 10^9) ns //(((util::RATE as u64) / 100) as u64;

    let _cpal_out = spawn_audio(rx, tx2, setbeat);

    ui_loop(
        tx,
        rx2,
        beat,
        measured_xtime,
        target_inc,
        &single_cycle_wave_forms,
    )
}
