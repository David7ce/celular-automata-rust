use std::collections::{HashMap, HashSet};

use rand::RngExt;

use crate::rules::RuleSet;

pub type Cell = (i64, i64);

/// Side length (in cells) of one spatial-index chunk. Chosen to be a bit
/// larger than a typical zoomed-in viewport in cells, so a visible-bounds
/// query touches only a handful of chunks.
const CHUNK_SIZE: i64 = 32;

fn chunk_of(cell: Cell) -> (i64, i64) {
    (cell.0.div_euclid(CHUNK_SIZE), cell.1.div_euclid(CHUNK_SIZE))
}

pub struct SimState {
    pub live: HashSet<Cell>,
    /// Spatial index mirroring `live`, bucketed by chunk, so rendering can
    /// query only the cells near the viewport instead of scanning every
    /// live cell every frame.
    chunks: HashMap<(i64, i64), HashSet<Cell>>,
    pub rule: RuleSet,
    pub running: bool,
    /// Generations per second.
    pub speed: f32,
    pub generation: u64,
    /// Cells born / that died over the most recent `step()` (or, after
    /// `step_n`, summed across that whole batch).
    pub last_births: u64,
    pub last_deaths: u64,
    accumulator: f32,
}

impl SimState {
    pub fn new(rule: RuleSet) -> Self {
        SimState {
            live: HashSet::new(),
            chunks: HashMap::new(),
            rule,
            running: false,
            speed: 8.0,
            generation: 0,
            last_births: 0,
            last_deaths: 0,
            accumulator: 0.0,
        }
    }

    fn insert_cell(&mut self, cell: Cell) {
        if self.live.insert(cell) {
            self.chunks.entry(chunk_of(cell)).or_default().insert(cell);
        }
    }

    fn remove_cell(&mut self, cell: Cell) -> bool {
        if !self.live.remove(&cell) {
            return false;
        }
        let key = chunk_of(cell);
        if let Some(bucket) = self.chunks.get_mut(&key) {
            bucket.remove(&cell);
            if bucket.is_empty() {
                self.chunks.remove(&key);
            }
        }
        true
    }

    fn rebuild_chunks(&mut self) {
        self.chunks.clear();
        for &cell in &self.live {
            self.chunks.entry(chunk_of(cell)).or_default().insert(cell);
        }
    }

    pub fn toggle_cell(&mut self, cell: Cell) {
        if !self.remove_cell(cell) {
            self.insert_cell(cell);
        }
    }

    pub fn set_cell(&mut self, cell: Cell, alive: bool) {
        if alive {
            self.insert_cell(cell);
        } else {
            self.remove_cell(cell);
        }
    }

    pub fn stamp(&mut self, cells: &[(i32, i32)], at: Cell) {
        for &(dx, dy) in cells {
            self.insert_cell((at.0 + dx as i64, at.1 + dy as i64));
        }
    }

    pub fn clear(&mut self) {
        self.live.clear();
        self.chunks.clear();
        self.generation = 0;
        self.last_births = 0;
        self.last_deaths = 0;
        self.accumulator = 0.0;
    }

    pub fn randomize(&mut self, min: Cell, max: Cell, density: f32) {
        let mut rng = rand::rng();
        for x in min.0..=max.0 {
            for y in min.1..=max.1 {
                if rng.random::<f32>() < density {
                    self.insert_cell((x, y));
                }
            }
        }
    }

    /// Live cells within `[min, max]` (inclusive), for rendering only the
    /// visible viewport instead of the whole (potentially huge) live set.
    pub fn cells_in_bounds(&self, min: Cell, max: Cell) -> Vec<Cell> {
        let mut out = Vec::new();
        let (min_cx, min_cy) = chunk_of(min);
        let (max_cx, max_cy) = chunk_of(max);
        for cx in min_cx..=max_cx {
            for cy in min_cy..=max_cy {
                if let Some(bucket) = self.chunks.get(&(cx, cy)) {
                    out.extend(
                        bucket
                            .iter()
                            .copied()
                            .filter(|&(x, y)| x >= min.0 && x <= max.0 && y >= min.1 && y <= max.1),
                    );
                }
            }
        }
        out
    }

    pub fn step(&mut self) {
        let next = next_generation(&self.live, &self.rule);
        self.last_births = next.difference(&self.live).count() as u64;
        self.last_deaths = self.live.difference(&next).count() as u64;
        self.live = next;
        self.rebuild_chunks();
        self.generation += 1;
    }

    /// Advances `n` generations at once (n.max(1)), reporting the total
    /// births/deaths across the whole batch in `last_births`/`last_deaths`
    /// rather than just the final generation's.
    pub fn step_n(&mut self, n: u32) {
        let mut total_births = 0u64;
        let mut total_deaths = 0u64;
        for _ in 0..n.max(1) {
            self.step();
            total_births += self.last_births;
            total_deaths += self.last_deaths;
        }
        self.last_births = total_births;
        self.last_deaths = total_deaths;
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
