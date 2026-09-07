# Roadmap

## Status as of 2026-09-06

The app is functional end to end: rule engine (presets + custom B/S),
zoom/pan, speed control, and a categorized pattern library that stamps
patterns onto an infinite sparse grid. Built clean in release mode
(`cargo build --release`, binary at `target/release/conway_life`). A
desktop shortcut exists at `~/Desktop/GameOfLife.desktop`.

Interaction model went through a few rounds of live feedback tonight and
landed here:
- Left-click = toggle one cell (or stamp the selected pattern).
- Left-click-drag = freehand paint/erase a trail (Bresenham-interpolated so
  fast drags don't leave gaps).
- Pinch / Ctrl+scroll = zoom, anchored on the gesture/cursor.
- Two-finger trackpad scroll = pan.

**Reported but not yet investigated**: the app felt like it "corrupted"
after painting for a while. Not yet reproduced/diagnosed carefully — see
"Next up" below for the leading hypothesis and how to confirm it.

## Update — 2026-09-06 (later)

Added touchpad zoom and keyboard shortcuts:
- Keyboard shortcuts: `Esc` cancels pattern placement, `Space` play/pause,
  `S` step, `C` clear, `R` random fill, `+`/`-` zoom in/out (centered).

## Update — 2026-09-07

Reworked touchpad gestures after feedback that scroll-to-zoom felt wrong —
using vertical scroll for zoom fought with wanting to pan up/down, unlike
mobile touch behavior. Now:
- Two-finger trackpad drag always pans, in whichever direction you move your
  fingers (`smooth_scroll_delta` fed straight into `View::pan`), matching
  how panning works on a phone/tablet.
- Zooming is only ever a pinch gesture or Ctrl+scroll (`zoom_delta`),
  anchored on the pinch center or cursor. This is a distinct egui input
  channel from trackpad scrolling, so pan and zoom can't fight each other or
  trigger accidentally from the wrong axis.

## Update — 2026-09-07 (later)

Fixed the painting slowdown from item 1 below: `SimState` now keeps a
spatial index (`chunks: HashMap<(i64,i64), HashSet<Cell>>`, 32x32-cell
buckets) alongside `live`, updated incrementally on every insert/remove
(`toggle_cell`, `set_cell`, `stamp`, `randomize`) and rebuilt once per
generation in `step`. `central_canvas`'s render loop now calls
`sim.cells_in_bounds(min, max)`, which only visits chunks overlapping the
viewport, instead of scanning the entire `live` set every frame. Rendering
is now `O(visible cells + overlapping chunks)` rather than `O(live cells)`,
so a long paint stroke or a large live set no longer costs more per frame
than what's actually on screen. Not independently re-verified by hand under
heavy load (per the "no testing tonight" instruction from the prior
session) — worth confirming next time the app is run for a while.

## Update — 2026-09-07 (touchpad-only + rule pack + stats)

Addressed feedback that the user has no mouse, only a touchpad, plus a
larger feature batch:

- **Rule presets expanded from 7 to 22.** Added Diamoeba, Flakes, Gnarl,
  HighLife, Inverse Life, Long Life, Maze, Mazectric, Move, Pseudo Life,
  Replicator, Seeds, Serviettes, Stains, Walled Cities — B/S strings
  verified against LifeWiki/Wikipedia via web search rather than typed from
  memory (20 rules is too many to risk misremembering). Every preset
  (including the original 7) now carries a `class: "stable" | "chaotic" |
  "explosive"` tag reflecting its documented long-term random-soup behavior,
  shown in the rule dropdown as e.g. "Diamoeba (chaotic)".
- **Generation skipping.** New "Skip" dropdown (0/5/10/50/100/500/1000,
  default 0). Step (button or `S` key) now calls `SimState::step_n`, which
  runs that many generations in one call and reports total births/deaths
  across the whole batch — useful for fast-forwarding to see a rule's
  long-term behavior without waiting through, or rendering, every
  intermediate generation.
- **Births/Deaths counters.** `SimState::step` now diffs the live set
  before/after each generation and stores `last_births`/`last_deaths`;
  shown in the top bar next to Gen/Live.
- **Show/hide grid.** Checkbox in the top bar, on by default; grid lines in
  `central_canvas` are now gated on it (still also requires `cell_size >
  4.0` as before, so it doesn't force grid lines back on when zoomed out
  past visibility).
- Two-finger trackpad panning (added in the update above) already covers
  the "no mouse" gesture requirement — reconfirmed as the intended way to
  navigate, no mouse-only interaction was reintroduced.

Not independently re-verified live (per the standing "don't test, it's
slow in this environment" guidance) — worth a hands-on pass next session,
especially confirming pinch-zoom and two-finger pan actually arrive as
`zoom_delta`/`smooth_scroll_delta` events from this specific touchpad/driver
combination, since gesture routing varies by OS and windowing backend.

## Update — 2026-09-07 (gesture fallback + pattern fixes + regression test)

The user confirmed the touchpad gestures from the update above don't
register on their hardware ("right now in touchbar does not work" — this
resolves the "worth confirming" note above: it doesn't work, at least not
on this machine). Rather than continue debugging gesture routing blind (no
way to test interactively in this environment), added on-screen controls
that work regardless of gesture support:
- Zoom: `-` button, a slider bound to `cell_size`, `+` button — all anchored
  on the canvas center via a new `App::canvas_size` field (captured each
  frame in `central_canvas`, one frame stale when used from `top_panel`,
  which doesn't matter visually).
- Pan: `<`/`^`/`v`/`>` buttons, plus arrow keys as a keyboard equivalent
  (`Key::ArrowUp/Down/Left/Right` in the existing keyboard-shortcut match
  arm), both driving `View::pan` with a fixed `PAN_STEP`.
- Gesture-based zoom/pan are unchanged and still active alongside these —
  this is a fallback, not a replacement.

Also investigated "Gosper Glider Gun does not show its form": the gun's
RLE was actually correct (independently verified, bounding box and
cell-by-cell layout match the canonical LifeWiki pattern), but the small
preview icon in the pattern-library side panel had a real bug —
`paint_pattern_preview`'s scale calculation used `.max(1.0)`, which forced
at least 1px per cell and prevented the preview from ever *shrinking* a
pattern to fit the tiny 36x36 icon box. For the 36-cell-wide gun this meant
the preview overflowed the box almost entirely, showing what looked like a
meaningless blob instead of the gun's shape — even though stamping it onto
the actual canvas placed the correct pattern all along. Fixed by clamping
to `[0.3, 6.0]` instead of flooring at `1.0`.

**Pattern library grew from 21 to 28** (Tub, Ship, Pond, Clock, Queen Bee
Shuttle, Loafer, Simkin Glider Gun, B-heptomino added), with every new RLE
string pulled from conwaylife.com's mirror at copy.sh (LifeWiki itself
returns 403 to fetches) rather than typed from memory. Also added a
`#[cfg(test)] known_population_counts` test in `patterns.rs` that checks
every single library pattern's parsed cell count against its documented
LifeWiki population — cheap, high-signal regression coverage for a format
(RLE) where a single wrong digit silently produces a different, wrong
shape.

That test immediately caught **three real pre-existing bugs**, all
predating this session (shipped in the very first build):
1. **"Boat" was defined with the Ship's 6-cell shape**, not its own 5-cell
   shape — an exact duplicate of the (now separately added) Ship pattern.
2. **Pulsar had two extra spurious rows** (`5bo3bo5b`, appearing twice)
   adding 4 phantom cells not part of the real 48-cell pattern — these
   would have shown as two extra floating dots and likely broken or altered
   the oscillation.
3. **Middleweight and Heavyweight Spaceship both had malformed tails** — the
   final two rows encoded a 4-wide (resp. 5-wide) solid block one column
   short of and shifted from the correct 5-wide (resp. 6-wide) tail, i.e. a
   different, likely non-spaceship shape rather than the real MWSS/HWSS.

All three are fixed now, sourced from copy.sh's canonical RLEs. Two of my
own *expected* population figures in the new test were also wrong on the
first pass (Beacon: guessed 6, actually 8; Pentadecathlon: guessed 10,
actually 12) — corrected against the same canonical source rather than
trusting memory either way.

## Next up (priority order)

1. **Random-fill density control.** Currently hardcoded to 0.35 — expose it
   as a slider next to the "Random" button.

2. **Pattern placement niceties.** Rotate/flip the selected pattern before
   stamping (R / F keys), since guns and spaceships are directional.

3. **Persistence.** Save/load the current board as RLE (export what's
   drawn, import a pattern file from disk) — currently patterns only come
   from the built-in library.

4. **Windows/macOS build verification.** Only Linux has actually been
   built and run so far. The dependency set (`eframe`, `rand`) is
   cross-platform with no OS-specific code, so this should mostly be a
   matter of running `cargo build --release` on each target and fixing
   anything that comes up (packaging/icon per OS if desired).

5. **Packaging.** Right now it's a raw binary + a hand-written `.desktop`
   file. Consider `cargo-bundle` or `cargo-packager` for a proper
   `.app`/`.exe`/`.AppImage` if this needs to be distributed beyond this
   machine.

## Nice-to-haves (not scheduled)

- More library patterns per category (currently a representative handful:
  4 still lifes, 5 oscillators, 4 spaceships, 1 gun, 3 methuselahs).
- Simkin Glider Gun as a second gun (skipped tonight — didn't want to ship
  an RLE string for it I couldn't verify from memory).
