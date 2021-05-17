use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use termion::event::Key;
use termion::input::TermRead;

use tui::widgets::ListState;

pub struct StatefulList {
    pub state: ListState,
    pub item_len: usize,
}

impl StatefulList {
    pub fn with_items<T>(items: &Vec<T>) -> StatefulList {
        StatefulList {
            state: ListState::default(),
            item_len: items.len(),
        }
    }
    pub fn forward(&mut self, f: usize) {
        self.state.select(match self.state.selected() {
            Some(i) => Some(i.saturating_add(f).min(self.item_len - 1)),
            None => Some(f),
        });
    }
    pub fn backward(&mut self, f: usize) {
        self.state.select(match self.state.selected() {
            Some(i) => Some(i.saturating_sub(f)),
            None => Some(0),
        });
    }

    pub fn next(&mut self) {
        self.forward(1);
    }

    pub fn previous(&mut self) {
        self.backward(1);
    }

    //    pub fn unselect(&mut self) {
    //        self.state.select(None);
    //    }
}

pub enum Event<I> {
    Input(I),
    Tick,
}

/// A small event handler that wrap termion input and tick events. Each event
/// type is handled in its own thread and returned to a common `Receiver`
pub struct Events {
    rx: mpsc::Receiver<Event<Key>>,

    #[allow(dead_code)]
    input_handle: thread::JoinHandle<()>,

    #[allow(dead_code)]
    tick_handle: thread::JoinHandle<()>,
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub exit_key: Key,
    pub tick_rate: Duration,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            exit_key: Key::Char('q'),
            tick_rate: Duration::from_millis(250),
        }
    }
}

impl Events {
    pub fn with_config(config: Config) -> Events {
        let (tx, rx) = mpsc::channel();
        let input_handle = {
            let tx = tx.clone();
            thread::spawn(move || {
                let stdin = io::stdin();
                for evt in stdin.keys() {
                    if let Ok(key) = evt {
                        if let Err(err) = tx.send(Event::Input(key)) {
                            eprintln!("{}", err);
                            return;
                        }
                        //if !ignore_exit_key.load(Ordering::Relaxed) && key == config.exit_key {
                        //    return;
                        //}
                    }
                }
            })
        };
        let tick_handle = {
            thread::spawn(move || loop {
                if tx.send(Event::Tick).is_err() {
                    break;
                }
                thread::sleep(config.tick_rate);
            })
        };
        Events {
            rx,
            input_handle,
            tick_handle,
        }
    }

    pub fn next(&self) -> Result<Event<Key>, mpsc::RecvError> {
        self.rx.recv()
    }
}
