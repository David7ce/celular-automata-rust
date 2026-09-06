use crate::rle;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Category {
    StillLife,
    Oscillator,
    Spaceship,
    Gun,
    Methuselah,
}

impl Category {
    pub const ALL: [Category; 5] = [
        Category::StillLife,
        Category::Oscillator,
        Category::Spaceship,
        Category::Gun,
        Category::Methuselah,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Category::StillLife => "Still Lifes",
            Category::Oscillator => "Oscillators",
            Category::Spaceship => "Spaceships",
            Category::Gun => "Guns",
            Category::Methuselah => "Methuselahs",
        }
    }
}

pub struct Pattern {
    pub name: &'static str,
    pub category: Category,
    pub cells: Vec<(i32, i32)>,
}

struct Def {
    name: &'static str,
    category: Category,
    rle: &'static str,
}

const DEFS: &[Def] = &[
    // Still lifes
    Def { name: "Block", category: Category::StillLife, rle: "2o$2o!" },
    Def { name: "Beehive", category: Category::StillLife, rle: "b2ob$o2bo$b2ob!" },
    Def { name: "Loaf", category: Category::StillLife, rle: "b2ob$o2bo$bobo$2bo!" },
    Def { name: "Boat", category: Category::StillLife, rle: "2ob$obo$b2o!" },
    // Oscillators
    Def { name: "Blinker", category: Category::Oscillator, rle: "3o!" },
    Def { name: "Toad", category: Category::Oscillator, rle: "b3o$3o!" },
    Def { name: "Beacon", category: Category::Oscillator, rle: "2o$2o$2b2o$2b2o!" },
    Def {
        name: "Pulsar",
        category: Category::Oscillator,
        rle: "2b3o3b3o2b$5bo3bo5b2$o4bobo4bo$o4bobo4bo$o4bobo4bo$2b3o3b3o2b2$2b3o3b3o2b$o4bobo4bo$o4bobo4bo$o4bobo4bo2$5bo3bo5b$2b3o3b3o2b!",
    },
    Def { name: "Pentadecathlon", category: Category::Oscillator, rle: "2bo4bo2b$2ob4ob2o$2bo4bo2b!" },
    // Spaceships
    Def { name: "Glider", category: Category::Spaceship, rle: "bo$2bo$3o!" },
    Def { name: "Lightweight Spaceship", category: Category::Spaceship, rle: "bo2bo$o$o3bo$4o!" },
    Def { name: "Middleweight Spaceship", category: Category::Spaceship, rle: "3bo2b$bo3bo$o5b$o3bob$4o2b!" },
    Def { name: "Heavyweight Spaceship", category: Category::Spaceship, rle: "3b2o2b$bo4bo$o6b$o4bob$4o3b!" },
    // Guns
    Def {
        name: "Gosper Glider Gun",
        category: Category::Gun,
        rle: "24bo11b$22bobo11b$12b2o6b2o12b2o$11bo3bo4b2o12b2o$2o8bo5bo3b2o14b$2o8bo3bob2o4bobo11b$10bo5bo7bo11b$11bo3bo20b$12b2o!",
    },
    // Methuselahs
    Def { name: "R-pentomino", category: Category::Methuselah, rle: "b2o$2o$bo!" },
    Def { name: "Diehard", category: Category::Methuselah, rle: "6bo$2o6b$bo3b3o!" },
    Def { name: "Acorn", category: Category::Methuselah, rle: "bo5b$3bo3b$2o2b3o!" },
];

pub fn library() -> Vec<Pattern> {
    DEFS.iter()
        .map(|def| Pattern {
            name: def.name,
            category: def.category,
            cells: rle::parse(def.rle),
        })
        .collect()
}
