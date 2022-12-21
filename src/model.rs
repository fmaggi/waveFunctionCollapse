use std::ops::{AddAssign, Mul};

use rand::Rng;

#[derive(Copy)]
pub struct Color(pub u16, pub u16, pub u16);

pub struct Model {
    waves: Vec<Wave>,
    entropies: Vec<f32>,
    base_states: Vec<Color>,
    valid_neighbors: Vec<Vec<bool>>,
    x: usize,
    y: usize
}

impl Model {
    pub fn new(base_states: Vec<Color>, neighbors: Vec<Vec<bool>>, x: usize, y: usize) -> Model {
        let waves = Wave::vec(base_states.len(), x*y);
        let entropies = calculate_entropies(&waves);
        Model {
            waves,
            entropies,
            base_states,
            valid_neighbors: neighbors,
            x,
            y
        }
    }

    pub fn collapse(&mut self) -> Option<usize> {
        let node = self.find_node();
        let i = match node {
            Some(i) => i,
            None => return None,
        };

        let (wave, state_i, entropy) = self.observe(i);
        self.waves[i] = wave;
        self.entropies[i] = entropy;

        self.propagate(i, state_i);
        Some(i)
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
            c: vec![0.0; wave.c.len()]
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
            _ if column == (self.y-1) as isize => vec![-1isize, 0],
            _ => vec![-1isize, 0, 1]
        };
        let dxs = match row {
            0 => vec![0isize, 1],
            _ if row == (self.x-1) as isize => vec![-1isize, 0],
            _ => vec![-1isize, 0, 1]
        };

        for dy in &dys {
            for dx in &dxs {
                let nj = row + dx + (column + dy) * self.x as isize;
                let nj = nj as usize;
                assert!(nj < self.waves.len(), "nj too big {}. From {},{} and ds {},{}", nj, row, column, dx, dy);
                let wave = &mut self.waves[nj];
                for neighbor in 0..self.base_states.len() {
                    if !self.valid_neighbors[state_i][neighbor] {
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

    fn valid_neighbor(&self, state: usize, neighbor_state: usize) -> bool {
        self.valid_neighbors[state][neighbor_state]
    }

    pub fn print(&self, i: usize) -> String {
        let wave = &self.waves[i];
        let mut color = Color(0, 0, 0);

        for j in 0..self.base_states.len() {
            color += self.base_states[j] * wave.c[j];
        }

        format!("{} {} {}", color.0, color.1, color.2)
    }

}

struct Wave {
    c: Vec<f32>
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
    let modulus: f32 = wave.c
        .iter()
        .sum();
    let modulus = modulus.sqrt();
    let inv_mod = 1.0 / modulus;
    Wave {
        c: wave.c
            .iter()
            .map(|x| x * inv_mod)
            .collect()
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
        e.push(1.0-cmax);
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
    1.0-cmax
}

impl AddAssign for Color {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
        self.1 += rhs.1;
        self.2 += rhs.2;
    }
}

impl Mul<f32> for Color {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        let r = self.0 as f32;
        let g = self.1 as f32;
        let b = self.2 as f32;
        let r = (r * rhs) as u16;
        let g = (g * rhs) as u16;
        let b = (b * rhs) as u16;
        Color(r, g, b)        
    }
}

impl Clone for Color {
    fn clone(&self) -> Self {
        Self(self.0, self.1, self.2)
    }
}
