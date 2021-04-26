use std::ops::{Index, IndexMut};

use crate::util::Component;

pub struct Mixer {
    pub a: i8,
    pub a_lvl: i8,
    pub b: i8,
    pub b_lvl: i8,
    pub out: i8,
    pub dummy: i8,
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

impl<'a> Component<'a> for Mixer {
    fn tick(&mut self) {}
    fn step(&mut self) {
        // Does left shift work the way I want with signed values?
        // I am trying to use the amp_cv as essentially as a signed Q1.7
        let a = (((self.a_lvl as i16) * (self.a as i16)) >> 7) as i8;
        let b = (((self.b_lvl as i16) * (self.b as i16)) >> 7) as i8;

        self.out = a.saturating_add(b);
    }
    fn inputs(&self) -> Vec<&'a str> {
        vec!["a", "a_lvl", "b", "b_lvl"]
    }

    fn outputs(&self) -> Vec<&'a str> {
        vec!["out"]
    }
}

impl Index<&str> for Mixer {
    type Output = i8;

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
