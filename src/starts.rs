use crate::patterns::Pattern;
use crate::simulation::{Cell, SimState};

/// Named starting layouts the user can load in one action instead of
/// stamping/painting a fresh board by hand every time. Reuses the pattern
/// library's cell data rather than duplicating RLE strings here, so a fix to
/// a pattern's shape automatically applies to any start configuration built
/// from it.
pub const START_CONFIGS: &[&str] = &[
    "Empty board",
    "Random soup",
    "Single glider",
    "Gosper glider gun",
    "Acorn",
    "R-pentomino",
    "Diehard",
    "Glider symphony (4 gliders)",
    "Pulsar field (3x3)",
];

/// Clears the board, then lays out `name` centered on `center` (typically
/// the world's center cell). `density` only matters for "Random soup".
pub fn apply(sim: &mut SimState, name: &str, library: &[Pattern], center: Cell, density: f32) {
    sim.clear();
    let stamp_centered = |sim: &mut SimState, pattern_name: &str, at: Cell| {
        if let Some(pattern) = library.iter().find(|p| p.name == pattern_name) {
            let (cx, cy) = bounding_center(&pattern.cells);
            sim.stamp(&pattern.cells, (at.0 - cx, at.1 - cy));
        }
    };

    match name {
        "Random soup" => {
            let half = 40;
            let min = (center.0 - half, center.1 - half);
            let max = (center.0 + half, center.1 + half);
            sim.randomize(min, max, density);
        }
        "Single glider" => stamp_centered(sim, "Glider", center),
        "Gosper glider gun" => stamp_centered(sim, "Gosper Glider Gun", center),
        "Acorn" => stamp_centered(sim, "Acorn", center),
        "R-pentomino" => stamp_centered(sim, "R-pentomino", center),
        "Diehard" => stamp_centered(sim, "Diehard", center),
        "Glider symphony (4 gliders)" => {
            for (dx, dy) in [(-30, -30), (30, -30), (-30, 30), (30, 30)] {
                stamp_centered(sim, "Glider", (center.0 + dx, center.1 + dy));
            }
        }
        "Pulsar field (3x3)" => {
            for row in -1..=1i64 {
                for col in -1..=1i64 {
                    stamp_centered(sim, "Pulsar", (center.0 + col * 22, center.1 + row * 22));
                }
            }
        }
        // "Empty board" (or anything unrecognized) just leaves the cleared board.
        _ => {}
    }
}

/// Rounds-toward-zero midpoint of a pattern's bounding box, used to center
/// it on a target cell instead of stamping from its RLE-origin corner.
fn bounding_center(cells: &[(i32, i32)]) -> (i64, i64) {
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
    for &(x, y) in cells {
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x);
        max_y = max_y.max(y);
    }
    (((min_x + max_x) / 2) as i64, ((min_y + max_y) / 2) as i64)
}
