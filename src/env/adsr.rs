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
    attack_for: i8,
    attack_to: i8,
    decay_for: i8,
    sustain_at: i8,
    release_for: i8,
    counter: i8,
    triggered_at: u16,
    gated_at: u16,
    gate_closed_at: u16,
    dummy: i8,
    prev_trigger: i8,
    prev_gate: i8,
    trigger: i8,
    triggered: bool,
    gated: bool,
    gate: i8,
    out: i8,
    state: AdsrState,
    main_counter: u16,
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
    // The floats make me cry.
    fn step(&mut self) {
        let q = i8::max_value() / 4;
        let tq = 3 * q;
        self.triggered = (!self.triggered) && (self.trigger > tq) && (self.prev_trigger < q);
        self.gated = (self.gated && (self.gate > q)) || (!self.gated && self.gate > tq);

        self.prev_trigger = self.trigger;
        self.prev_gate = self.gate;

        match (self.state, self.triggered, self.gated) {
            (AdsrState::Off, false, _) => (),
            (AdsrState::Off, true, _) => {
                self.state = AdsrState::Attack;
                self.counter = 0;
                self.main_counter = 0;
            }
            // What to do if we're anything and triggered?
            (_, true, _) => (),

            (AdsrState::Attack, false, true) => {
                self.out = ((self.attack_to as f64) * (self.main_counter as f64)
                    / ((self.attack_for as f64) * 441.)) as i8;
                if self.counter == self.attack_for {
                    self.state = AdsrState::Decay;
                    self.counter = 0;
                    self.main_counter = 0;
                } else {
                    // Inc the counter every 10ms.
                    //
                    // I want this to fail, at least now, if this overflows.
                    self.main_counter = self.main_counter.checked_add(10).unwrap();
                    if self.main_counter > (((self.counter as u16) + 1) * 441) {
                        // I want this to fail, at least now, if this overflows.
                        self.counter = self.counter.checked_add(1).unwrap();
                    }
                }
            }
            (AdsrState::Decay, false, true) => {
                self.out = (((self.attack_to as f64) - (self.sustain_at as f64))
                    * ((441. * (self.decay_for as f64)) - (self.main_counter as f64))
                    / ((self.decay_for as f64) * 441.)) as i8
                    + self.sustain_at;
                if self.counter == self.decay_for {
                    self.state = AdsrState::Sustain;
                    self.counter = 0;
                    self.main_counter = 0;
                } else {
                    // Inc the counter every 10ms.
                    //
                    // I want this to fail, at least now, if this overflows.
                    self.main_counter = self.main_counter.checked_add(10).unwrap();
                    if self.main_counter > (((self.counter as u16) + 1) * 441) {
                        // I want this to fail, at least now, if this overflows.
                        self.counter = self.counter.checked_add(1).unwrap();
                    }
                }
            }
            (AdsrState::Sustain, false, true) => self.out = self.sustain_at,
            (AdsrState::Release, false, false) => {
                self.out = ((self.sustain_at as f64)
                    * ((441. * (self.release_for as f64)) - (self.main_counter as f64))
                    / ((self.release_for as f64) * 441.)) as i8;
                if self.counter == self.release_for {
                    self.state = AdsrState::Off;
                    self.counter = 0;
                    self.main_counter = 0;
                } else {
                    // Inc the counter every 10ms.
                    //
                    // I want this to fail, at least now, if this overflows.
                    self.main_counter = self.main_counter.checked_add(10).unwrap();
                    if self.main_counter > (((self.counter as u16) + 1) * 441) {
                        // I want this to fail, at least now, if this overflows.
                        self.counter = self.counter.checked_add(1).unwrap();
                    }
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
    type Output = i8;

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