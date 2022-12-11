pub struct Model {
    waves: Vec<Wave>,
    entropies: Vec<f32>,
    dimensions: Dimension
}

struct Dimension {
    x: usize,
    y: usize
}

impl Model {
    pub fn new(size: usize) -> Model {
        Model {
            waves: Vec::with_capacity(size),
            entropies: vec![0.8, 0.3, 0.6, 0.23, 0.67, 0.90, 0.54, 0.12, 0.54, 0.65, 0.90, 0.58, 0.14, 0.65, 0.89, 0.36],
            dimensions: Dimension { x: 4, y: 4 }
        }
    }

    pub fn collapse(&mut self) -> bool {
        let node = self.find_node();
        let i = match node {
            Some(i) => i,
            None => return self.check(),
        };

        // let (wave, entropy) = self.observe(i);
        // self.waves[i] = wave;
        // self.entropies[i] = entropy;
        //
        self.entropies[i] = 0.0;

        // self.propagate(i)
        self.check()
    }

    fn observe(&self, i: usize) -> (Wave, f32) {
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
        (ret_wave, 0.0)
    }

    fn propagate(&mut self, i: usize) -> bool {
        let row = i % self.dimensions.x;
        let column = (i - row) / self.dimensions.x;

        false
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

    pub fn print(&self) {
        let ys: Vec<usize> = (0..self.dimensions.y).collect();
        let xs: Vec<usize> = (0..self.dimensions.x).collect();
        for j in &ys {
            for i in &xs {
                print!("{} ", self.entropies[i + j*self.dimensions.x]);
            }
            println!("");
        }
    }

}

struct Wave {
    c: Vec<f32>
}

fn normalize(wave: Wave) -> Wave {
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
