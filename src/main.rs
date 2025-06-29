
use std::time::Instant;

type Pos = u64;

const FINAL_MASK: Pos = (1u64 << 35) - 1; // All 35 bits set
const CHOICES_COUNT: usize = 9;
const MAX_POSITIONS: usize = 384;

struct PyramidSolver {
    choices: [usize; CHOICES_COUNT],
    lengths: [usize; CHOICES_COUNT],
    all_possible_positions: [[Pos; MAX_POSITIONS]; CHOICES_COUNT],
    ptr: [usize; CHOICES_COUNT],
}

const KNOWN_DISTANCES: [[f64; 6]; 5] = [
    [1.0, 1.0, 1.0, 1.732, 2.0, 2.64575],
    [1.0, 1.0, 1.0, 1.0, 1.732, 2.0],
    [1.0, 1.0, 1.0, 1.732, 1.732, 2.64575],
    [1.0, 1.0, 1.0, 1.732, 1.732, 2.0],
    [1.0, 1.0, 1.0, 1.41421, 2.0, 2.23606],
];

const COORDS: [[f64; 3]; 35] = [
    [0.0, 0.0, 0.0],
    [1.0, 0.0, 0.0],
    [2.0, 0.0, 0.0],
    [3.0, 0.0, 0.0],
    [4.0, 0.0, 0.0],
    [0.5, 0.866, 0.0],
    [1.5, 0.866, 0.0],
    [2.5, 0.866, 0.0],
    [3.5, 0.866, 0.0],
    [1.0, 1.732, 0.0],
    [2.0, 1.732, 0.0],
    [3.0, 1.732, 0.0],
    [1.5, 2.598, 0.0],
    [2.5, 2.598, 0.0],
    [2.0, 3.464, 0.0],
    [0.5, 0.28867, 0.8165],
    [1.5, 0.28867, 0.8165],
    [2.5, 0.28867, 0.8165],
    [3.5, 0.28867, 0.8165],
    [1.0, 1.15467, 0.8165],
    [2.0, 1.15467, 0.8165],
    [3.0, 1.15467, 0.8165],
    [1.5, 2.02067, 0.8165],
    [2.5, 2.02067, 0.8165],
    [2.0, 2.88667, 0.8165],
    [1.0, 0.57734, 1.633],
    [2.0, 0.57734, 1.633],
    [3.0, 0.57734, 1.633],
    [1.5, 1.44334, 1.633],
    [2.5, 1.44334, 1.633],
    [2.0, 2.30934, 1.633],
    [1.5, 0.86601, 2.4495],
    [2.5, 0.86601, 2.4495],
    [2.0, 1.73201, 2.4495],
    [2.0, 1.15468, 3.266],
];

const PLANES: [[usize; 15]; 21] = [
    [0,1,2,3,4,5,6,7,8,9,10,11,12,13,14],
    [15,16,17,18,19,20,21,22,23,24,24,24,24,24,24],
    [25,26,27,28,29,30,30,30,30,30,30,30,30,30,30],
    [0,5,9,12,14,15,19,22,24,25,28,30,31,33,34],
    [1,6,10,13,16,20,23,26,29,32,32,32,32,32,32],
    [2,7,11,17,21,27,27,27,27,27,27,27,27,27,27],
    [0,1,2,3,4,15,16,17,18,25,26,27,31,32,34],
    [5,6,7,8,19,20,21,28,29,33,33,33,33,33,33],
    [9,10,11,22,23,30,30,30,30,30,30,30,30,30,30],
    [4,8,11,13,14,18,21,23,24,27,29,30,32,33,34],
    [3,7,10,12,17,20,22,26,28,31,31,31,31,31,31],
    [2,6,9,16,19,25,25,25,25,25,25,25,25,25,25],
    [5,6,7,8,15,16,17,18,18,18,18,18,18,18,18],
    [9,10,11,19,20,21,25,26,27,27,27,27,27,27,27],
    [12,13,22,23,28,29,31,32,32,32,32,32,32,32,32],
    [1,6,10,13,15,19,22,24,24,24,24,24,24,24,24],
    [2,7,11,16,20,23,25,28,30,30,30,30,30,30,30],
    [3,8,17,21,26,29,31,33,33,33,33,33,33,33,33],
    [3,7,10,12,18,21,23,24,24,24,24,24,24,24,24],
    [2,6,9,17,20,22,27,29,30,30,30,30,30,30,30],
    [1,5,16,19,26,28,32,33,33,33,33,33,33,33,33],
];

impl PyramidSolver {
    fn new() -> Self {
        Self {
            choices: [0; CHOICES_COUNT],
            lengths: [5, 384, 336, 96, 168, 96, 96, 96, 96],
            all_possible_positions: [[0; MAX_POSITIONS]; CHOICES_COUNT],
            ptr: [0; CHOICES_COUNT],
        }
    }

    fn set3(i: usize, j: usize, k: usize) -> Pos {
        (1u64 << i) | (1u64 << j) | (1u64 << k)
    }

    fn set4(i: usize, j: usize, k: usize, l: usize) -> Pos {
        (1u64 << i) | (1u64 << j) | (1u64 << k) | (1u64 << l)
    }

    fn in_plane(plane_idx: usize, i: usize, j: usize, k: usize, l: usize) -> bool {
        let plane = &PLANES[plane_idx];
        plane.contains(&i) && plane.contains(&j) && plane.contains(&k) && plane.contains(&l)
    }

    fn is_planar(i: usize, j: usize, k: usize, l: usize) -> bool {
        (0..21).any(|plane_idx| Self::in_plane(plane_idx, i, j, k, l))
    }

    fn distance(i: usize, j: usize) -> f64 {
        let (x1, y1, z1) = (COORDS[i][0], COORDS[i][1], COORDS[i][2]);
        let (x2, y2, z2) = (COORDS[j][0], COORDS[j][1], COORDS[j][2]);

        let dx = x1 - x2;
        let dy = y1 - y2;
        let dz = z1 - z2;

        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    fn bubble_sort_six(distances: &mut [f64; 6]) {
        for i in 1..6 {
            for j in (1..=i).rev() {
                if distances[j] < distances[j - 1] {
                    distances.swap(j, j - 1);
                } else {
                    break;
                }
            }
        }
    }

    fn about_equal(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn is_match(a: &[f64; 6], b: &[f64; 6]) -> bool {
        a.iter().zip(b.iter()).all(|(x, y)| Self::about_equal(*x, *y))
    }

    fn match_distances(&mut self, i: usize, j: usize, k: usize, l: usize) {
        let mut distances = [
            Self::distance(i, j),
            Self::distance(i, k),
            Self::distance(i, l),
            Self::distance(j, k),
            Self::distance(j, l),
            Self::distance(k, l),
        ];

        Self::bubble_sort_six(&mut distances);

        if distances[5] > 2.66 {
            return;
        }

        for (idx, known) in KNOWN_DISTANCES.iter().enumerate() {
            if Self::is_match(&distances, known) {
                let pos = Self::set4(i, j, k, l);

                if idx == 4 {
                    // Special case for the last pattern - add to multiple groups
                    for group in (idx + 1)..=(idx + 4) {
                        self.all_possible_positions[group][self.ptr[group]] = pos;
                        self.ptr[group] += 1;
                    }
                } else {
                    self.all_possible_positions[idx + 1][self.ptr[idx + 1]] = pos;
                    self.ptr[idx + 1] += 1;
                }
                return;
            }
        }
    }

    fn precompute(&mut self) {
        for i in 0..=31 {
            for j in (i + 1)..=32 {
                for k in (j + 1)..=33 {
                    for l in (k + 1)..=34 {
                        if Self::is_planar(i, j, k, l) {
                            self.match_distances(i, j, k, l);
                        }
                    }
                }
            }
        }
    }

    fn initialize(&mut self) {
        // Initialize the first group with predefined 3-element sets
        self.all_possible_positions[0][0] = Self::set3(0, 1, 2);
        self.all_possible_positions[0][1] = Self::set3(1, 2, 3);
        self.all_possible_positions[0][2] = Self::set3(5, 6, 7);
        self.all_possible_positions[0][3] = Self::set3(9, 10, 11);
        self.all_possible_positions[0][4] = Self::set3(19, 20, 21);
    }

    fn search(&mut self, level: usize, prev: Pos) -> bool {
        if level == CHOICES_COUNT {
            return prev == FINAL_MASK;
        }

        for index in 0..self.lengths[level] {
            let pos = self.all_possible_positions[level][index];
            if (prev & pos) == 0 {
                if self.search(level + 1, prev | pos) {
                    self.choices[level] = index;
                    return true;
                }
            }
        }

        false
    }

    fn display(&self) {
        let mut occupied = [0usize; 35];

        println!("Choices:");
        for choice in &self.choices {
            print!("{} ", choice);
        }
        println!();

        for (i, &choice) in self.choices.iter().enumerate() {
            let pos = self.all_possible_positions[i][choice];
            for j in 0..35 {
                if (pos & (1u64 << j)) != 0 {
                    occupied[j] = i;
                }
            }
        }

        for &occ in &occupied {
            print!("{} ", occ + 1);
        }
        println!();
    }

    fn solve(&mut self) {
        self.initialize();
        self.precompute();

        let start = Instant::now();
        self.search(0, 0);
        let duration = start.elapsed();

        println!("{:.6} sec", duration.as_secs_f64());
        self.display();
    }
}

fn main() {
    let mut solver = PyramidSolver::new();
    solver.solve();
}
