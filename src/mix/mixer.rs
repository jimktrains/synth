use std::ops::{Index, IndexMut};

use crate::util::Component;

pub struct Mixer {
    pub a: i16,
    pub a_lvl: i16,
    pub b: i16,
    pub b_lvl: i16,
    pub out: i16,
    pub dummy: i16,
}

impl Mixer {
    pub fn new() -> Self {
        Mixer {
            a: 0,
            a_lvl: 0,
            b: 0,
            b_lvl: 0,
            out: 0,
            dummy: 0,
        }
    }
}

impl Component for Mixer {
    fn tick(&mut self) {}
    fn step(&mut self) {
        // Does left shift work the way I want with signed values?
        // I am trying to use the amp_cv as essentially as a signed Q1.7
        let a = (((self.a_lvl as i32) * (self.a as i32)) >> 15) as i16;
        let b = (((self.b_lvl as i32) * (self.b as i32)) >> 15) as i16;

        self.out = a.saturating_add(b);
    }
    fn inputs(&self) -> Vec<&'static str> {
        vec!["a", "a_lvl", "b", "b_lvl"]
    }

    fn outputs(&self) -> Vec<&'static str> {
        vec!["out"]
    }
}

impl Index<&str> for Mixer {
    type Output = i16;

    fn index(&self, i: &str) -> &Self::Output {
        match i {
            "out" => &self.out,
            _ => &0,
        }
    }
}

impl IndexMut<&str> for Mixer {
    fn index_mut(&mut self, i: &str) -> &mut Self::Output {
        match i {
            "a" => &mut self.a,
            "a_lvl" => &mut self.a_lvl,
            "b" => &mut self.b,
            "b_lvl" => &mut self.b_lvl,
            _ => &mut self.dummy,
        }
    }
}
