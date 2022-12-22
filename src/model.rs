use std::fs::OpenOptions;
use std::io::prelude::*;
use std::ops::{AddAssign, Mul};

use num::integer::Roots;
use rand::Rng;

use image::{open, DynamicImage};

pub struct Model {
    waves: Vec<Wave>,
    entropies: Vec<f32>,
    base_states: Vec<Vec<Color>>,
    x: usize,
    y: usize,
    N: usize,
}

impl Model {
    pub fn from_image(filename: &str, N: usize) -> Option<Model> {
        let image = match open("3Bricks.png") {
            Ok(DynamicImage::ImageRgb8(i)) => i,
            _ => {
                println!("Error");
                return None;
            }
        };

        let w = image.width();
        let h = image.height();

        let mut base_states: Vec<Vec<Color>> = Vec::new();

        let mut pattern: Vec<Color> = Vec::new();

        let mut y_offset = 0u32;
        let mut x_offset = 0u32;

        while y_offset < h {
            while x_offset < w {
                for y in 0..N {
                    for x in 0..N {
                        let pixel = image.get_pixel(x as u32 + x_offset, y as u32 + y_offset);
                        pattern.push(Color::new(pixel.0[0], pixel.0[1], pixel.0[2]));
                    }
                }

                println!("pusing pattern {}", pattern.len());
                base_states.push(pattern);
                pattern = Vec::new();

                x_offset += N as u32;
            }
            x_offset = 0;
            y_offset += N as u32;
        }

        //        for (i, s) in base_states.iter().enumerate() {
        //            let name = i.to_string() + ".ppm";
        //            Model::dump_state(&s, &name);
        //        }

        Some(Model::new(base_states, N))
    }

    pub fn new(base_states: Vec<Vec<Color>>, N: usize) -> Model {
        Model {
            waves: Vec::new(),
            entropies: Vec::new(),
            base_states,
            x: 0,
            y: 0,
            N,
        }
    }

    pub fn collapse(&mut self, x: usize, y: usize) {
        self.x = x;
        self.y = y;
        self.waves = Wave::vec(self.base_states.len(), x * y);
        self.entropies = calculate_entropies(&self.waves);
        loop {
            let node = self.find_node();
            let i = match node {
                Some(i) => i,
                None => return,
            };

            let (wave, state_i, entropy) = self.observe(i);
            self.waves[i] = wave;
            self.entropies[i] = entropy;

            self.propagate(i, state_i);
        }
    }

    fn observe(&self, i: usize) -> (Wave, usize, f32) {
        let wave = &self.waves[i];

        let mut max_p = wave.c[0];
        let mut max_i = 0;
        for (i, &c) in wave.c.iter().enumerate() {
            if c > max_p {
                max_p = c;
                max_i = i;
            }
        }

        let mut ret_wave = Wave {
            c: vec![0.0; wave.c.len()],
        };

        ret_wave.c[max_i] = 1.0;
        (ret_wave, max_i, 0.0)
    }

    fn propagate(&mut self, i: usize, state_i: usize) {
        let row = i % self.x;
        let column = (i - row) / self.x;
        let row = row as isize;
        let column = column as isize;

        let dys = match column {
            0 => vec![0isize, 1],
            _ if column == (self.y - 1) as isize => vec![-1isize, 0],
            _ => vec![-1isize, 0, 1],
        };
        let dxs = match row {
            0 => vec![0isize, 1],
            _ if row == (self.x - 1) as isize => vec![-1isize, 0],
            _ => vec![-1isize, 0, 1],
        };

        for dy in &dys {
            for dx in &dxs {
                let nj = row + dx + (column + dy) * self.x as isize;
                let nj = nj as usize;
                assert!(
                    nj < self.waves.len(),
                    "nj too big {}. From {},{} and ds {},{}",
                    nj,
                    row,
                    column,
                    dx,
                    dy
                );
                let wave = &mut self.waves[nj];
                for neighbor in 0..self.base_states.len() {
                    if !Model::valid_neighbor(&self.base_states, state_i, neighbor) {
                        wave.c[neighbor] = 0.0;
                    }
                }

                self.waves[nj] = normalize(&self.waves[nj]);
                self.entropies[nj] = calculate_entropy(&self.waves[nj]);
            }
        }
    }

    fn check(&self) -> bool {
        self.entropies.iter().sum::<f32>() > 0.0
    }

    fn find_node(&self) -> Option<usize> {
        let mut imin: usize = 0;
        let mut min: f32 = f32::INFINITY;

        for (i, &e) in self.entropies.iter().enumerate() {
            if e > 0.0 && e < min {
                imin = i;
                min = e;
            }
        }

        if min == f32::INFINITY {
            None
        } else {
            Some(imin)
        }
    }

    fn valid_neighbor(neighbors: &Vec<Vec<Color>>, state: usize, neighbor_state: usize) -> bool {
        let len = neighbors.len();
        let row = state % len;
        let column = (state - row) / len;

        let neighbor_row = neighbor_state % len;
        let neighbor_column = (neighbor_state - neighbor_row) / len;

        let dx = if row < neighbor_row {
            neighbor_row - row
        } else {
            row - neighbor_row
        };

        let dy = if column < neighbor_column {
            neighbor_column - column
        } else {
            column - neighbor_column
        };

        dx <= 1 && dy <= 1
    }

    pub fn dump(&self, filename: &str) {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .append(true)
            .open(filename)
            .unwrap();

        if let Err(e) = writeln!(file, "P3\n{} {}\n255\n", self.x, self.y) {
            println!("Couldn't write to file: {}", e);
            return;
        }
        let mut y_offset = 0;
        let mut x_offset = 0;

        let pw = self.x / self.N;
        println!("pw {}", pw);

        for y in 0..self.y {
            let column = y / self.N;
            let pcolumn = y % self.N;
            for x in 0..self.x {
                let row = x / self.N;
                let prow = x % self.N;
                let i = row + column * self.N;
                
                let wave = &self.waves[i];
                
                let pi = prow + pcolumn * self.N;
                let mut c = Color::black();

                for (ci, &co) in wave.c.iter().enumerate() {
                    c += self.base_states[ci][pi] * co;
                }

                writeln!(file, "{} {} {}", c.r, c.g, c.b);
            }
        }
    }

    fn dump_state(state: &Vec<Color>, filename: &str) {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .append(true)
            .open(filename)
            .unwrap();

        let l = state.len();
        let n = l.sqrt();
        println!("{} {}", l, n);
        if let Err(e) = writeln!(file, "P3\n{} {}\n255\n", n, n) {
            println!("Couldn't write to file: {}", e);
            return;
        }

        for c in state {
            writeln!(file, "{} {} {}", c.r, c.g, c.b);
        }
    }
}

struct Wave {
    c: Vec<f32>,
}

impl Wave {
    fn new(dimensions: usize) -> Wave {
        let mut rng = rand::thread_rng();
        let c = (0..dimensions).map(|_| rng.gen_range(0.0, 1.0)).collect();
        Wave { c }
    }

    fn vec(dimensions: usize, size: usize) -> Vec<Wave> {
        (0..size).map(|_| Wave::new(dimensions)).collect()
    }
}

fn normalize(wave: &Wave) -> Wave {
    let modulus: f32 = wave.c.iter().sum();
    let modulus = modulus.sqrt();
    let inv_mod = 1.0 / modulus;
    Wave {
        c: wave.c.iter().map(|x| x * inv_mod).collect(),
    }
}

fn calculate_entropies(waves: &Vec<Wave>) -> Vec<f32> {
    let mut e: Vec<f32> = Vec::with_capacity(waves.len());
    for w in waves.iter() {
        let mut cmax = w.c[0];
        for wc in w.c.iter() {
            if wc > &cmax {
                cmax = *wc;
            }
        }
        e.push(1.0 - cmax);
    }
    e
}

fn calculate_entropy(wave: &Wave) -> f32 {
    let mut cmax = wave.c[0];
    for c in wave.c.iter() {
        if c > &cmax {
            cmax = *c;
        }
    }
    1.0 - cmax
}

#[derive(Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    fn black() -> Color {
        Color { r: 0, g: 0, b: 0 }
    }

    fn new(r: u8, g: u8, b: u8) -> Color {
        Color { r, g, b }
    }
}

impl AddAssign for Color {
    fn add_assign(&mut self, rhs: Self) {
        self.r = self.r.saturating_add(rhs.r);
        self.g = self.g.saturating_add(rhs.g);
        self.b = self.b.saturating_add(rhs.b);
    }
}

impl Mul<f32> for Color {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        let r = self.r as f32;
        let g = self.g as f32;
        let b = self.b as f32;
        let r = (r * rhs) as u8;
        let g = (g * rhs) as u8;
        let b = (b * rhs) as u8;
        Color { r, g, b }
    }
}

impl Clone for Color {
    fn clone(&self) -> Self {
        Self {
            r: self.r,
            g: self.g,
            b: self.b,
        }
    }
}
