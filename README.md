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
- **Finite 480x270 plane** (16:9, a quarter-scale Full HD proportion): live
  cells are stored as a `HashSet<(i64, i64)>` bounded to `x ∈ [-240, 239]`,
  `y ∈ [-135, 134]` (`simulation::WORLD_MIN/WORLD_MAX`) — no wraparound,
  cells simply can't be painted, stamped, or born past the edge (drawn as a
  red boundary line on the canvas), stepped with the standard
  neighbor-counting algorithm (`O(live cells)` per generation).
- **Starting configurations**: a "Start" dropdown + "Load" button (Board
  row) clears the board and lays out a named setup centered on the world —
  Empty board, Random soup, Single glider, Gosper glider gun, Acorn,
  R-pentomino, Diehard, Glider symphony (4 gliders), Pulsar field (3x3).
  Built from the pattern library's own cell data (`src/starts.rs`), so a fix
  to a pattern's shape automatically carries through to any start built from
  it.
- **Eraser tool**: a Draw/Eraser toggle (Board row) next to the pattern
  library. Draw is the default click/drag-to-toggle-or-paint behavior;
  Eraser forces every click or drag stroke to remove cells regardless of
  their state, with a red outline over the cell it would remove. Selecting
  a pattern from the library switches back to Draw automatically.
- **The viewport is always fully inside the map.** Panning/zooming is
  clamped (`View::clamp_to_world`) so the visible rectangle can slide right
  up to an edge but never shows empty space beyond it — you can't scroll
  off into the void. If the viewport is ever wider/taller than the map
  itself (very zoomed out on a large window), that axis is centered on the
  map instead.
- **Minimap**: bottom-right overlay showing the whole plane, a green marker
  per occupied region, and a yellow outline for the current viewport. Click
  or drag inside it to jump/pan the camera anywhere on the plane instantly —
  handy since the plane is much bigger than what's visible at once.
- **Zoom & pan**: pinch-to-zoom (or Ctrl+scroll) anchored on the cursor/gesture,
  clamped between a min and max cell size; two-finger trackpad drag pans
  freely in any direction, like scrolling a map on a touchscreen — panning
  and zooming are separate gestures and never fight each other. On-screen
  `-`/slider/`+` zoom controls, `<`/`^`/`v`/`>` pan buttons, and a "Reset
  view" button in the top bar work identically without relying on gesture
  recognition — true pinch-to-zoom via a laptop touchpad is a platform
  limitation on Linux (winit only wires up `PinchGesture`/`PanGesture` on
  macOS/iOS), so these on-screen controls are the primary way to navigate
  there, not just a fallback. A "Show input debug" checkbox overlays live
  `zoom_delta`/`scroll_delta`/touch values on the canvas for diagnosing any
  gesture that still seems to do nothing.
- **Speed control**: Play/Pause/Step, generations-per-second slider.
- **Pattern library**: 35 well-known patterns across 5 categories (still
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
  starts.rs       - named starting configurations (Board "Start" dropdown)
  view.rs         - pan/zoom camera, cell<->screen coordinate math
```

## Controls

| Action | Input |
|---|---|
| Draw / erase a cell | Left-click (Draw tool; erases instead if the stroke starts on a live cell) |
| Freehand paint a trail | Left-click-drag (Draw tool) |
| Force-erase cells | Switch to the "Eraser" tool (Board row), then click/drag — always removes, regardless of cell state |
| Place a pattern | Select it in the left panel, then click the canvas (switches back to the Draw tool) |
| Cancel pattern placement | Right-click, `Esc`, or the "Cancel" button in the panel |
| Load a starting configuration | "Start" dropdown + "Load" button (Board row) — clears the board first |
| Zoom | Pinch gesture (macOS/iOS only — see Known issues), Ctrl + scroll, `+`/`-` keys, or the `-`/slider/`+` controls in the top bar |
| Pan | Two-finger trackpad drag, arrow keys, the `<`/`^`/`v`/`>` buttons, or click/drag on the minimap |
| Jump to a distant part of the plane | Click or drag inside the minimap (bottom-right corner) |
| Reset the camera | "Reset view" button in the top bar |
| Play / Pause | `Space`, or the button in the top bar |
| Step (by the selected skip amount) | `S`, or the "Step" button |
| Choose how many generations Step advances | "Skip" dropdown (0, 5, 10, 50, 100, 500, 1000 — 0 behaves as 1) |
| Clear board | `C`, or the "Clear" button |
| Fill visible area randomly | `R`, or the "Random" button (density set by the adjacent slider) |
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

- **True pinch-to-zoom trackpad gestures don't reach the app on Linux.**
  This is a `winit` platform limitation, not a bug here: `winit`'s
  `PinchGesture`/`PanGesture`/`RotationGesture` events (which `egui-winit`
  does translate into zoom/pan) are only ever emitted on macOS/iOS — there's
  no code path that produces them on X11 or Wayland. Ctrl+scroll still
  zooms (egui synthesizes that itself from the scroll wheel), and the
  on-screen zoom/pan controls added in the top bar work everywhere
  regardless. Two-finger-scroll-to-pan *should* still work on Linux (it's
  just a regular high-resolution scroll-wheel event, not a special
  gesture) — if it doesn't on a given machine, turn on "Show input debug"
  in the top bar and see whether `scroll_delta` moves at all while
  two-finger-scrolling; that'll tell us whether it's this app, the desktop
  environment intercepting the gesture, or the touchpad driver.
- See `ROADMAP.md` for what's next.
