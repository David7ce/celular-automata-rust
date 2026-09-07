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
- **Rule presets**: 22 built-in Life-like rules — Conway's Life, 2x2, 34 Life,
  Assimilation, Coagulations, Coral, Day & Night, Diamoeba, Flakes, Gnarl,
  HighLife, Inverse Life, Long Life, Maze, Mazectric, Move, Pseudo Life,
  Replicator, Seeds, Serviettes, Stains, Walled Cities — each tagged with its
  long-term random-soup behavior (`stable`, `chaotic`, or `explosive`) in the
  dropdown, e.g. "Diamoeba (chaotic)". Plus 9 Birth / 9 Survive checkboxes to
  build any custom rule by hand.
- **Infinite sparse grid**: live cells are stored as a `HashSet<(i64, i64)>`
  (no board size limit, no wraparound), stepped with the standard
  neighbor-counting algorithm (`O(live cells)` per generation).
- **Zoom & pan**: pinch-to-zoom (or Ctrl+scroll) anchored on the cursor/gesture,
  clamped between a min and max cell size; two-finger trackpad drag pans
  freely in any direction, like scrolling a map on a touchscreen — panning
  and zooming are separate gestures and never fight each other. On-screen
  `-`/slider/`+` zoom controls and `<`/`^`/`v`/`>` pan buttons in the top bar
  work identically without relying on gesture recognition, for touchpads
  that don't report pinch/scroll gestures to the app.
- **Speed control**: Play/Pause/Step, generations-per-second slider.
- **Pattern library**: 28 well-known patterns across 5 categories (still
  lifes, oscillators, spaceships, guns, methuselahs), decoded from standard
  RLE strings verified against LifeWiki and stamped onto the canvas on
  click. A regression test (`cargo test`) checks every pattern's cell count
  against its documented population.
- **Freehand drawing**: click a single cell, or press-and-drag to paint (or
  erase, if the stroke starts on a live cell) a trail of cells.
- **Generation skipping**: a "Skip" dropdown (0/5/10/50/100/500/1000) lets
  Step (or the `S` key) advance several generations at once instead of one.
- **Live stats**: generation count, live-cell count, and cells born/died on
  the most recent step (or across a whole skip batch).
- **Show/hide grid**: checkbox in the top bar, on by default.

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
| Zoom | Pinch gesture, Ctrl + scroll, `+`/`-` keys, or the `-`/slider/`+` controls in the top bar |
| Pan | Two-finger trackpad drag (any direction), arrow keys, or the `<`/`^`/`v`/`>` buttons in the top bar |
| Play / Pause | `Space`, or the button in the top bar |
| Step (by the selected skip amount) | `S`, or the "Step" button |
| Choose how many generations Step advances | "Skip" dropdown (0, 5, 10, 50, 100, 500, 1000 — 0 behaves as 1) |
| Clear board | `C`, or the "Clear" button |
| Fill visible area randomly | `R`, or the "Random" button (density is fixed at 0.35 for now) |
| Change generation speed | "Speed" slider (0.5–60 gen/s) |
| Switch rule preset | "Rule" dropdown (each entry tagged stable/chaotic/explosive) |
| Build a custom rule | Birth / Survive checkboxes (switches label to "Custom") |
| Show/hide the grid lines | "Show grid" checkbox (on by default) |

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

None currently tracked. (Rendering used to scan every live cell each frame
to cull to the viewport, which could bog down during heavy freehand
painting on a large board — fixed by adding a chunked spatial index in
`SimState`; see `ROADMAP.md` for details.)
