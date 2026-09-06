/// Minimal parser for the standard Life RLE pattern format.
/// Ignores header (`x = .., y = ..`) and comment (`#C`) lines; only the
/// `b`/`o`/`$`/`!` run-length body matters for placing cells.
pub fn parse(rle: &str) -> Vec<(i32, i32)> {
    let mut cells = Vec::new();
    let mut x: i32 = 0;
    let mut y: i32 = 0;
    let mut count: u32 = 0;

    for line in rle.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with('x') {
            continue;
        }
        for ch in line.chars() {
            match ch {
                '0'..='9' => {
                    count = count * 10 + (ch as u32 - '0' as u32);
                }
                'b' => {
                    x += count.max(1) as i32;
                    count = 0;
                }
                'o' => {
                    let n = count.max(1);
                    for i in 0..n {
                        cells.push((x + i as i32, y));
                    }
                    x += n as i32;
                    count = 0;
                }
                '$' => {
                    y += count.max(1) as i32;
                    x = 0;
                    count = 0;
                }
                '!' => break,
                _ => {}
            }
        }
    }

    cells
}
