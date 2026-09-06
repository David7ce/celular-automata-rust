use std::collections::{HashMap, HashSet};

use rand::RngExt;

use crate::rules::RuleSet;

pub type Cell = (i64, i64);

pub struct SimState {
    pub live: HashSet<Cell>,
    pub rule: RuleSet,
    pub running: bool,
    /// Generations per second.
    pub speed: f32,
    pub generation: u64,
    accumulator: f32,
}

impl SimState {
    pub fn new(rule: RuleSet) -> Self {
        SimState {
            live: HashSet::new(),
            rule,
            running: false,
            speed: 8.0,
            generation: 0,
            accumulator: 0.0,
        }
    }

    pub fn toggle_cell(&mut self, cell: Cell) {
        if !self.live.remove(&cell) {
            self.live.insert(cell);
        }
    }

    pub fn set_cell(&mut self, cell: Cell, alive: bool) {
        if alive {
            self.live.insert(cell);
        } else {
            self.live.remove(&cell);
        }
    }

    pub fn stamp(&mut self, cells: &[(i32, i32)], at: Cell) {
        for &(dx, dy) in cells {
            self.live.insert((at.0 + dx as i64, at.1 + dy as i64));
        }
    }

    pub fn clear(&mut self) {
        self.live.clear();
        self.generation = 0;
        self.accumulator = 0.0;
    }

    pub fn randomize(&mut self, min: Cell, max: Cell, density: f32) {
        let mut rng = rand::rng();
        for x in min.0..=max.0 {
            for y in min.1..=max.1 {
                if rng.random::<f32>() < density {
                    self.live.insert((x, y));
                }
            }
        }
    }

    pub fn step(&mut self) {
        self.live = next_generation(&self.live, &self.rule);
        self.generation += 1;
    }

    /// Advances the simulation according to elapsed time, if running.
    pub fn tick(&mut self, dt: f32) {
        if !self.running || self.speed <= 0.0 {
            return;
        }
        self.accumulator += dt * self.speed;
        while self.accumulator >= 1.0 {
            self.step();
            self.accumulator -= 1.0;
        }
    }
}

fn next_generation(live: &HashSet<Cell>, rule: &RuleSet) -> HashSet<Cell> {
    let mut counts: HashMap<Cell, u8> = HashMap::with_capacity(live.len() * 4);
    for &(x, y) in live {
        for dx in -1..=1i64 {
            for dy in -1..=1i64 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                *counts.entry((x + dx, y + dy)).or_insert(0) += 1;
            }
        }
    }

    counts
        .into_iter()
        .filter(|&(cell, n)| {
            if live.contains(&cell) {
                rule.survive[n as usize]
            } else {
                rule.birth[n as usize]
            }
        })
        .map(|(cell, _)| cell)
        .collect()
}
