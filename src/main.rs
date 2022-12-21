pub mod model;

use crate::model::Model;
use crate::model::Color;

use std::fs::OpenOptions;
use std::io::prelude::*;

fn main() {
    let x = 20;
    let y = 20;

    let base_states = vec![Color(255, 0, 0), Color(0, 255, 0), Color(0, 0, 255), Color(255, 255, 0), Color(255, 0, 255), Color(0, 255, 255), Color(255, 255, 255)];

    let neighbors = vec![
        vec![true, false, false, true, true, false, true],
        vec![false, true, false, true, false, true, true],
        vec![false, false, true, false, true, true, true],
        vec![true, true, false, true, true, true, true],
        vec![true, false, true, true, true, true, true],
        vec![false, true, true, true, true, true, true],
        vec![true, true, true, true, true, true, true]
    ];

    let mut m: Model = Model::new(base_states, neighbors, x, y);


    let mut c = 0;
    loop {
        let index = match m.collapse() {
            Some(index) => index,
            None => return
        };
        if c % 40 == 0 {
            let mut file = OpenOptions::new()
                .write(true)
                .append(true)
                .open("test".to_owned() + &c.to_string() + ".ppm")
                .unwrap();

            if let Err(e) = writeln!(file, "P3\n{} {}\n255\n", x, y) {
                println!("Couldn't write to file: {}", e);
                return;
            }                  

            for i in 0..(x*y) {
                writeln!(file, "{}", m.print(i));
            }
        } 
        c += 1;
    }
}

