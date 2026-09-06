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
}

pub const PRESETS: &[Preset] = &[
    Preset { name: "Conway's Life", birth: &[3], survive: &[2, 3] },
    Preset { name: "2x2", birth: &[3, 6], survive: &[1, 2, 5] },
    Preset { name: "34 Life", birth: &[3, 4], survive: &[3, 4] },
    Preset { name: "Assimilation", birth: &[3, 4, 5], survive: &[4, 5, 6, 7] },
    Preset { name: "Coagulations", birth: &[3, 7, 8], survive: &[2, 3, 5, 6, 7, 8] },
    Preset { name: "Coral", birth: &[3], survive: &[4, 5, 6, 7, 8] },
    Preset { name: "Day & Night", birth: &[3, 6, 7, 8], survive: &[3, 4, 6, 7, 8] },
];

pub fn preset_rule(preset: &Preset) -> RuleSet {
    RuleSet::from_counts(preset.birth, preset.survive)
}
