# Game of Life (Rust, cross-platform)

A native desktop implementation of Conway's Game of Life and other
Life-like cellular automata, built with [egui](https://github.com/emilk/egui) /
[eframe](https://github.com/emilk/egui/tree/main/crates/eframe). Runs on
Windows, macOS and Linux from the same codebase.

## Features

- **Generic B/S rule engine** — any Life-like rule (how many live neighbors
  are required for a dead cell to be born, and for a live cell to survive)
  is expressed as a `RuleSet { birth: [bool; 9], survive: [bool; 9] }` and
  evaluated by one generic step function. No rule is hardcoded separately.
- **Rule presets**: Conway's Life, 2x2, 34 Life, Assimilation, Coagulations,
  Coral, Day & Night — plus 9 Birth / 9 Survive checkboxes to build any
  custom rule by hand.
- **Infinite sparse grid**: live cells are stored as a `HashSet<(i64, i64)>`
  (no board size limit, no wraparound), stepped with the standard
  neighbor-counting algorithm (`O(live cells)` per generation).
- **Zoom & pan**: pinch-to-zoom (or Ctrl+scroll) anchored on the cursor/gesture,
  clamped between a min and max cell size; two-finger trackpad scroll pans.
- **Speed control**: Play/Pause/Step, generations-per-second slider.
- **Pattern library**: a categorized, clickable collection of well-known
  patterns (still lifes, oscillators, spaceships, guns, methuselahs),
  decoded from standard RLE strings and stamped onto the canvas on click.
- **Freehand drawing**: click a single cell, or press-and-drag to paint (or
  erase, if the stroke starts on a live cell) a trail of cells.

## Project layout

```
src/
  main.rs        - eframe bootstrap / window setup
  app.rs          - App struct (impl eframe::App), UI panels, canvas input
  simulation.rs   - SimState: sparse live-cell set, step/tick/randomize
  rules.rs        - RuleSet, named presets, B/S formatting
  patterns.rs     - Pattern/Category, the pattern library definitions
  rle.rs          - minimal RLE decoder (b/o/$/! run-length format)
  view.rs         - pan/zoom camera, cell<->screen coordinate math
```

## Controls

| Action | Input |
|---|---|
| Draw / erase a cell | Left-click |
| Freehand paint a trail | Left-click-drag (erases instead if the stroke starts on a live cell) |
| Place a pattern | Select it in the left panel, then click the canvas |
| Cancel pattern placement | Right-click, `Esc`, or the "Cancel" button in the panel |
| Zoom | Vertical two-finger trackpad scroll, pinch gesture, or Ctrl + scroll, or `+` / `-` keys |
| Pan | Horizontal two-finger trackpad scroll |
| Play / Pause | `Space`, or the button in the top bar |
| Single step | `S`, or the "Step" button |
| Clear board | `C`, or the "Clear" button |
| Fill visible area randomly | `R`, or the "Random" button (density is fixed at 0.35 for now) |
| Change generation speed | "Speed" slider (0.5–60 gen/s) |
| Switch rule preset | "Rule" dropdown |
| Build a custom rule | Birth / Survive checkboxes (switches label to "Custom") |

## Building & running

Requires a Rust toolchain (installed here via `rustup`, stable channel).

```bash
cargo build --release
./target/release/conway_life          # Linux/macOS
# or just:
cargo run --release
```

A desktop shortcut was created at `~/Desktop/GameOfLife.desktop` pointing at
the release binary. Depending on your desktop environment you may need to
right-click it once and choose "Allow Launching" / "Trust" the first time.

The code only depends on cross-platform crates (`eframe`, `rand`) with no
OS-specific APIs, so `cargo build --release` should also produce a working
binary on Windows and macOS — only Linux has been built/run in this
environment so far.

## Known issues

- **Performance degrades during heavy freehand painting**: the canvas
  render loop iterates every live cell each frame to filter to the visible
  viewport (`app.rs`, `central_canvas`), which is `O(live cells)` per
  frame. A long drag-paint stroke (especially zoomed out, where each pixel
  of mouse movement covers many cells) combined with the simulation running
  can grow the live set quickly and make the UI feel like it "corrupts" or
  freezes momentarily. See the roadmap for planned fixes.
