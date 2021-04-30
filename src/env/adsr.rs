use crate::util::Component;
use std::ops::{Index, IndexMut};

#[derive(Copy, Debug, Clone)]
enum AdsrState {
    Off,
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Debug)]
pub struct Adsr {
    attack_for: i16,
    attack_to: i16,
    decay_for: i16,
    sustain_at: i16,
    release_for: i16,
    counter: i16,
    triggered_at: u32,
    gated_at: u32,
    gate_closed_at: u32,
    dummy: i16,
    prev_trigger: i16,
    prev_gate: i16,
    trigger: i16,
    triggered: bool,
    gated: bool,
    gate: i16,
    out: i16,
    state: AdsrState,
    main_counter: u32,
}

impl Adsr {
    pub fn new() -> Self {
        Adsr {
            attack_for: 0,
            attack_to: 0,
            decay_for: 0,
            sustain_at: 0,
            release_for: 0,
            counter: 0,
            triggered_at: 0,
            gated_at: 0,
            gate_closed_at: 0,
            dummy: 0,
            out: 0,
            triggered: false,
            gated: false,
            prev_trigger: 0,
            prev_gate: 0,
            trigger: 0,
            gate: 0,
            state: AdsrState::Off,
            main_counter: 0,
        }
    }
}

impl<'a> Component<'a> for Adsr {
    fn tick(&mut self) {}
    // The floats make me cry.
    fn step(&mut self) {
        let q = i16::max_value() / 4;
        let tq = 3 * q;
        self.triggered = (!self.triggered) && (self.trigger > tq) && (self.prev_trigger < q);
        self.gated = (self.gated && (self.gate > q)) || (!self.gated && self.gate > tq);

        self.prev_trigger = self.trigger;
        self.prev_gate = self.gate;

        match (self.state, self.triggered, self.gated) {
            (AdsrState::Off, false, _) => self.counter = 0,
            (AdsrState::Off, true, _) => {
                self.state = AdsrState::Attack;
                self.counter = 0;
                self.main_counter = 0;
            }
            // What to do if we're anything and triggered?
            (_, true, _) => (),

            (AdsrState::Attack, false, true) => {
                self.out = ((self.attack_to as f64) * (self.counter as f64)
                    / (self.attack_for as f64)) as i16;
                if self.counter == self.attack_for {
                    self.state = AdsrState::Decay;
                    self.counter = 0;
                    self.main_counter = 0;
                } else {
                    self.counter = self.counter.checked_add(1).unwrap();
                }
            }
            (AdsrState::Decay, false, true) => {
                self.out = (((self.attack_to as f64) - (self.sustain_at as f64))
                    * ((self.decay_for as f64) - (self.counter as f64))
                    / (self.decay_for as f64)) as i16
                    + self.sustain_at;
                if self.counter == self.decay_for {
                    self.state = AdsrState::Sustain;
                    self.counter = 0;
                    self.main_counter = 0;
                } else {
                    self.counter = self.counter.checked_add(1).unwrap();
                }
            }
            (AdsrState::Sustain, false, true) => self.out = self.sustain_at,
            (AdsrState::Release, false, false) => {
                self.out = ((self.sustain_at as f64)
                    * ((self.release_for as f64) - (self.counter as f64))
                    / (self.release_for as f64)) as i16;
                if self.counter == self.release_for {
                    self.state = AdsrState::Off;
                    self.counter = 0;
                    self.main_counter = 0;
                } else {
                    self.counter = self.counter.checked_add(1).unwrap();
                }
            }
            // Not sure what to do here? Gated without a trigger?
            (AdsrState::Release, false, true) => {
                self.gated = false;
            }
            // Otherwise, if we lose the gate at any point, immediatly go
            // to a release state.
            (_, false, false) => {
                self.state = AdsrState::Release;
                self.counter = 0;
                self.main_counter = 0;
            }
        };
        self.triggered = false;
    }
    fn inputs(&self) -> Vec<&'a str> {
        vec![
            "attack_for",
            "attack_to",
            "decay_for",
            "sustain_at",
            "release_for",
            "trigger",
            "gate",
        ]
    }

    fn outputs(&self) -> Vec<&'a str> {
        vec!["out"]
    }
}

impl Index<&str> for Adsr {
    type Output = i16;

    fn index(&self, i: &str) -> &Self::Output {
        match i {
            "out" => &self.out,
            _ => &0,
        }
    }
}

impl IndexMut<&str> for Adsr {
    fn index_mut(&mut self, i: &str) -> &mut Self::Output {
        match i {
            "attack_for" => &mut self.attack_for,
            "attack_to" => &mut self.attack_to,
            "decay_for" => &mut self.decay_for,
            "sustain_at" => &mut self.sustain_at,
            "release_for" => &mut self.release_for,
            "trigger" => &mut self.trigger,
            "gate" => &mut self.gate,
            _ => &mut self.dummy,
        }
    }
}
