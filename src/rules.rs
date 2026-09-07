/// A totalistic Life-like rule: how many live neighbors (0-8) are required
/// for a dead cell to be born, and for a live cell to survive.
#[derive(Clone, Copy, PartialEq)]
pub struct RuleSet {
    pub birth: [bool; 9],
    pub survive: [bool; 9],
}

impl RuleSet {
    pub fn from_counts(birth: &[u8], survive: &[u8]) -> Self {
        let mut rule = RuleSet { birth: [false; 9], survive: [false; 9] };
        for &n in birth {
            rule.birth[n as usize] = true;
        }
        for &n in survive {
            rule.survive[n as usize] = true;
        }
        rule
    }

    /// Formats as standard "B.../S..." notation.
    pub fn to_bs_string(self) -> String {
        let b: String = (0..=8).filter(|&n| self.birth[n]).map(|n| char::from(b'0' + n as u8)).collect();
        let s: String = (0..=8).filter(|&n| self.survive[n]).map(|n| char::from(b'0' + n as u8)).collect();
        format!("B{b}/S{s}")
    }
}

pub struct Preset {
    pub name: &'static str,
    pub birth: &'static [u8],
    pub survive: &'static [u8],
    /// Long-term behavior tag from random-soup testing (LifeWiki/community
    /// consensus): "stable" settles into still lifes/oscillators, "chaotic"
    /// stays turbulent without settling or unbounded growth, "explosive"
    /// grows without bound.
    pub class: &'static str,
}

pub const PRESETS: &[Preset] = &[
    Preset { name: "Conway's Life", birth: &[3], survive: &[2, 3], class: "stable" },
    Preset { name: "2x2", birth: &[3, 6], survive: &[1, 2, 5], class: "stable" },
    Preset { name: "34 Life", birth: &[3, 4], survive: &[3, 4], class: "explosive" },
    Preset { name: "Assimilation", birth: &[3, 4, 5], survive: &[4, 5, 6, 7], class: "stable" },
    Preset { name: "Coagulations", birth: &[3, 7, 8], survive: &[2, 3, 5, 6, 7, 8], class: "explosive" },
    Preset { name: "Coral", birth: &[3], survive: &[4, 5, 6, 7, 8], class: "stable" },
    Preset { name: "Day & Night", birth: &[3, 6, 7, 8], survive: &[3, 4, 6, 7, 8], class: "stable" },
    Preset { name: "Diamoeba", birth: &[3, 5, 6, 7, 8], survive: &[5, 6, 7, 8], class: "chaotic" },
    Preset { name: "Flakes", birth: &[3], survive: &[0, 1, 2, 3, 4, 5, 6, 7, 8], class: "explosive" },
    Preset { name: "Gnarl", birth: &[1], survive: &[1], class: "explosive" },
    Preset { name: "HighLife", birth: &[3, 6], survive: &[2, 3], class: "stable" },
    Preset { name: "Inverse Life", birth: &[0, 1, 2, 3, 4, 7, 8], survive: &[3, 4, 6, 7, 8], class: "explosive" },
    Preset { name: "Long Life", birth: &[3, 4, 5], survive: &[5], class: "stable" },
    Preset { name: "Maze", birth: &[3], survive: &[1, 2, 3, 4, 5], class: "explosive" },
    Preset { name: "Mazectric", birth: &[3], survive: &[1, 2, 3, 4], class: "stable" },
    Preset { name: "Move", birth: &[3, 6, 8], survive: &[2, 4, 5], class: "stable" },
    Preset { name: "Pseudo Life", birth: &[3, 5, 7], survive: &[2, 3, 8], class: "chaotic" },
    Preset { name: "Replicator", birth: &[1, 3, 5, 7], survive: &[1, 3, 5, 7], class: "explosive" },
    Preset { name: "Seeds", birth: &[2], survive: &[], class: "explosive" },
    Preset { name: "Serviettes", birth: &[2, 3, 4], survive: &[], class: "explosive" },
    Preset { name: "Stains", birth: &[3, 6, 7, 8], survive: &[2, 3, 5, 6, 7, 8], class: "stable" },
    Preset { name: "Walled Cities", birth: &[4, 5, 6, 7, 8], survive: &[2, 3, 4, 5], class: "stable" },
];

pub fn preset_rule(preset: &Preset) -> RuleSet {
    RuleSet::from_counts(preset.birth, preset.survive)
}
