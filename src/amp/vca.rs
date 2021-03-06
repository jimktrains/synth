use std::ops::{Index, IndexMut};

use crate::util::Component;
pub struct Vca {
    pub in_cv: i16,
    pub amp_cv: i16,
    pub out_cv: i16,
    pub dummy: i16,
}

impl Vca {
    pub fn new(init_amp_cv: i16) -> Vca {
        Vca {
            in_cv: 0,
            amp_cv: init_amp_cv,
            out_cv: 0,
            dummy: 0,
        }
    }
}

impl Component for Vca {
    fn tick(&mut self) {}
    fn step(&mut self) {
        // Does left shift work the way I want with signed values?
        // I am trying to use the amp_cv as essentially as a signed Q1.15
        self.out_cv = (((self.amp_cv as i32) * (self.in_cv as i32)) >> 15) as i16;
    }
    fn inputs(&self) -> Vec<&'static str> {
        vec!["amp_in", "in_cv"]
    }

    fn outputs(&self) -> Vec<&'static str> {
        vec!["out"]
    }
}

impl Index<&str> for Vca {
    type Output = i16;

    fn index(&self, i: &str) -> &Self::Output {
        match i {
            "out" => &self.out_cv,
            _ => &0,
        }
    }
}

impl IndexMut<&str> for Vca {
    fn index_mut(&mut self, i: &str) -> &mut Self::Output {
        match i {
            "amp_cv" => &mut self.amp_cv,
            "in_cv" => &mut self.in_cv,
            _ => &mut self.dummy,
        }
    }
}
