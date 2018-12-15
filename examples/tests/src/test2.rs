use sysinfo::{System, SystemExt};

use std::ops;

//pub fn test2() {
//    System::new();
//}

pub struct Led{}

pub struct Leds {
    leds: [Led; 8],
}

impl ops::Index<usize> for Leds {
    type Output = Led;

    fn index(&self, i: usize) -> &Led {
        &self.leds[i]
    }
}