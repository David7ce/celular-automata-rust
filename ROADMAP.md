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
- Vertical two-finger trackpad scroll now zooms (anchored on the cursor),
  horizontal scroll pans. Ctrl+scroll and pinch gesture still zoom too — all
  three are mutually exclusive at the egui input layer (wheel input routes to
  either `smooth_scroll_delta` or `zoom_delta`, never both), so there's no
  double-handling.
- Keyboard shortcuts: `Esc` cancels pattern placement, `Space` play/pause,
  `S` step, `C` clear, `R` random fill, `+`/`-` zoom in/out (centered).
- Zoom-via-scroll sensitivity is `SCROLL_ZOOM_SPEED` in `app.rs` (currently
  0.003) — not yet tuned against real touchpad hardware, may need
  adjustment.

## Next up (priority order)

1. **Diagnose the painting slowdown/corruption.**
   - Leading hypothesis: `central_canvas`'s render loop in `app.rs` iterates
     the *entire* `sim.live` `HashSet` every frame to cull to the visible
     viewport — that's `O(live cells)`, not `O(visible cells)`. A drag
     stroke while zoomed out (each pixel of mouse motion covers several
     grid cells) can grow `live` very fast, and if the sim is also
     `running` on an explosive rule (Day & Night, Coagulations), the live
     set can blow up further each generation. The combination could look
     like freezing, tearing, or an unresponsive window.
   - To confirm: reproduce with the "Live: N" counter visible (already
     shown in the top bar) — if N spikes into the hundreds of thousands
     right before it "corrupts", that confirms it.
   - Likely fix: index live cells spatially (a coarse grid-of-chunks, or a
     `BTreeSet`/sorted structure keyed for range queries) so rendering is
     `O(visible cells)`, not `O(live cells)`. Simpler interim mitigation:
     cap how many cells a single paint stroke can add per frame, and/or
     pause simulation stepping while a paint stroke is in progress.
   - If it turns out to be a rendering/driver glitch instead (visual
     corruption rather than slowdown), try eframe's `glow` backend instead
     of the default `wgpu` backend as a quick isolation test.

2. **Random-fill density control.** Currently hardcoded to 0.35 — expose it
   as a slider next to the "Random" button.

3. **Pattern placement niceties.** Rotate/flip the selected pattern before
   stamping (R / F keys), since guns and spaceships are directional.

4. **Persistence.** Save/load the current board as RLE (export what's
   drawn, import a pattern file from disk) — currently patterns only come
   from the built-in library.

5. **Windows/macOS build verification.** Only Linux has actually been
   built and run so far. The dependency set (`eframe`, `rand`) is
   cross-platform with no OS-specific code, so this should mostly be a
   matter of running `cargo build --release` on each target and fixing
   anything that comes up (packaging/icon per OS if desired).

6. **Packaging.** Right now it's a raw binary + a hand-written `.desktop`
   file. Consider `cargo-bundle` or `cargo-packager` for a proper
   `.app`/`.exe`/`.AppImage` if this needs to be distributed beyond this
   machine.

## Nice-to-haves (not scheduled)

- More rule presets (Seeds, HighLife, Replicator, etc.) — trivial to add,
  just another entry in `rules::PRESETS`.
- More library patterns per category (currently a representative handful:
  4 still lifes, 5 oscillators, 4 spaceships, 1 gun, 3 methuselahs).
- Simkin Glider Gun as a second gun (skipped tonight — didn't want to ship
  an RLE string for it I couldn't verify from memory).
