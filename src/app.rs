use eframe::egui::{self, Color32, Key, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2};

use crate::patterns::{self, Category, Pattern};
use crate::rules;
use crate::simulation::{Cell, SimState, CHUNK_SIZE, WORLD_MAX, WORLD_MIN};
use crate::starts;
use crate::view::{View, MAX_CELL_SIZE, MIN_CELL_SIZE};

/// Zoom factor applied per keyboard/button zoom-shortcut press ('+'/'-').
const KEY_ZOOM_STEP: f32 = 1.2;
/// Screen-pixel-equivalent pan distance per keyboard arrow / pan-button press.
const PAN_STEP: f32 = 60.0;
/// Options for the "generations to skip" selector next to Step.
const SKIP_OPTIONS: &[u32] = &[0, 5, 10, 50, 100, 500, 1000];
/// On-screen size of the minimap box, anchored to the canvas's bottom-right
/// corner with `MINIMAP_MARGIN` of breathing room.
const MINIMAP_SIZE: Vec2 = Vec2::new(160.0, 160.0);
const MINIMAP_MARGIN: f32 = 12.0;

/// Everything in this file is the *2D renderer*: it turns `SimState`'s cells
/// and `View`'s camera into `egui::Painter` calls, and turns pointer/keyboard
/// input into `View`/`SimState` mutations. It deliberately never reaches
/// into simulation internals beyond the public `Cell`/`SimState` API, so a
/// future 3D build could swap this whole module for a `wgpu`-based
/// voxel/instanced-cube renderer and an orbiting 3D camera (replacing
/// `View`) without `simulation.rs`/`rules.rs`/`patterns.rs` needing to
/// change beyond widening `Cell` to `(i64, i64, i64)` (see the neighbor-
/// offset note in `simulation.rs`).
pub struct App {
    sim: SimState,
    view: View,
    library: Vec<Pattern>,
    selected_pattern: Option<usize>,
    preset_name: &'static str,
    preset_class: &'static str,
    random_density: f32,
    /// Whether the current drag-paint stroke is drawing (true) or erasing (false).
    paint_value: Option<bool>,
    last_paint_cell: Option<Cell>,
    show_grid: bool,
    /// How many generations a single "Step" advances at once (0 behaves as 1).
    skip_generations: u32,
    /// Eraser tool: when on, click/drag always removes cells (instead of the
    /// default draw tool's toggle/paint-a-trail behavior) and pattern
    /// placement is disabled, mutually exclusive with `selected_pattern`.
    eraser_mode: bool,
    /// Index into `starts::START_CONFIGS`, the "Start" dropdown's selection.
    selected_start: usize,
    /// Canvas size from the last frame, used to anchor button/slider zoom on
    /// the canvas center (the mouse-based zoom anchors on the cursor instead).
    canvas_size: Vec2,
    /// Whether an in-progress drag started inside the minimap (so it keeps
    /// steering the camera even if the pointer strays outside the box).
    dragging_minimap: bool,
    /// Shows live gesture/input values in a canvas corner, for diagnosing
    /// touchpad gestures that don't behave as expected on a given machine.
    show_input_debug: bool,
}

impl App {
    pub fn new() -> Self {
        let rule = rules::preset_rule(&rules::PRESETS[0]);
        let mut sim = SimState::new(rule);
        // Seed with a glider so the canvas isn't empty on first launch.
        sim.stamp(&crate::rle::parse("bo$2bo$3o!"), (2, 2));

        App {
            sim,
            view: View::default(),
            library: patterns::library(),
            selected_pattern: None,
            preset_name: rules::PRESETS[0].name,
            preset_class: rules::PRESETS[0].class,
            random_density: 0.35,
            paint_value: None,
            last_paint_cell: None,
            show_grid: true,
            skip_generations: 0,
            eraser_mode: false,
            selected_start: 0,
            canvas_size: Vec2::new(800.0, 600.0),
            dragging_minimap: false,
            show_input_debug: false,
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let dt = ctx.input(|i| i.stable_dt);
        self.sim.tick(dt);
        if self.sim.running {
            ctx.request_repaint();
        }

        self.top_panel(ui);
        self.side_panel(ui);
        self.central_canvas(ui);
    }
}

impl App {
    fn top_panel(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("controls").show(ui, |ui| {
            ui.label(egui::RichText::new("Simulation").small().strong());
            ui.horizontal(|ui| {
                ui.label("Rule:");
                egui::ComboBox::from_id_salt("rule_preset")
                    .selected_text(format!("{} ({})", self.preset_name, self.preset_class))
                    .show_ui(ui, |ui| {
                        for preset in rules::PRESETS {
                            let label = format!("{} ({})", preset.name, preset.class);
                            if ui.selectable_label(self.preset_name == preset.name, label).clicked() {
                                self.preset_name = preset.name;
                                self.preset_class = preset.class;
                                self.sim.rule = rules::preset_rule(preset);
                            }
                        }
                    });
                ui.label(self.sim.rule.to_bs_string());

                ui.separator();

                if ui.button(if self.sim.running { "Pause" } else { "Play" }).clicked() {
                    self.sim.running = !self.sim.running;
                }
                if ui
                    .button("Step")
                    .on_hover_text("Advance by the Skip amount (1 generation if Skip is 0)")
                    .clicked()
                {
                    self.sim.step_n(self.skip_generations);
                }
                ui.label("Skip");
                egui::ComboBox::from_id_salt("skip_generations")
                    .selected_text(self.skip_generations.to_string())
                    .show_ui(ui, |ui| {
                        for &n in SKIP_OPTIONS {
                            ui.selectable_value(&mut self.skip_generations, n, n.to_string());
                        }
                    });

                ui.separator();
                ui.label("Speed");
                ui.add(egui::Slider::new(&mut self.sim.speed, 0.5..=60.0).suffix(" gen/s"));

                ui.separator();
                ui.label(format!("Gen: {}", self.sim.generation));
                ui.label(format!("Live: {}", self.sim.live.len()));
                ui.label(format!("Births: {}", self.sim.last_births))
                    .on_hover_text("Cells born on the most recent step (or summed over a Skip batch)");
                ui.label(format!("Deaths: {}", self.sim.last_deaths))
                    .on_hover_text("Cells that died on the most recent step (or summed over a Skip batch)");
            });

            ui.separator();
            ui.label(egui::RichText::new("Board").small().strong());
            ui.horizontal(|ui| {
                ui.label("Start:");
                egui::ComboBox::from_id_salt("start_config")
                    .selected_text(starts::START_CONFIGS[self.selected_start])
                    .show_ui(ui, |ui| {
                        for (idx, name) in starts::START_CONFIGS.iter().enumerate() {
                            ui.selectable_value(&mut self.selected_start, idx, *name);
                        }
                    });
                if ui
                    .button("Load")
                    .on_hover_text("Clear the board and lay out the selected starting configuration")
                    .clicked()
                {
                    let center = ((WORLD_MIN.0 + WORLD_MAX.0) / 2, (WORLD_MIN.1 + WORLD_MAX.1) / 2);
                    starts::apply(&mut self.sim, starts::START_CONFIGS[self.selected_start], &self.library, center, self.random_density);
                    self.selected_pattern = None;
                    self.view.center_on(center, self.canvas_size);
                }

                ui.separator();
                if ui.button("Clear").on_hover_text("Erase every live cell").clicked() {
                    self.sim.clear();
                }
                if ui
                    .button("Random")
                    .on_hover_text(format!("Fill the visible area at {:.0}% density", self.random_density * 100.0))
                    .clicked()
                {
                    let (min, max) = self.view.visible_bounds(ui.available_size().max(Vec2::new(400.0, 400.0)));
                    self.sim.randomize(min, max, self.random_density);
                }
                ui.add(egui::Slider::new(&mut self.random_density, 0.05..=0.9).text("density"));

                ui.separator();
                if ui.selectable_label(!self.eraser_mode, "Draw").on_hover_text("Click/drag to toggle or paint cells").clicked() {
                    self.eraser_mode = false;
                }
                if ui
                    .selectable_label(self.eraser_mode, "Eraser")
                    .on_hover_text("Click/drag to remove cells (goma de borrar)")
                    .clicked()
                {
                    self.eraser_mode = true;
                    self.selected_pattern = None;
                }

                ui.separator();
                ui.checkbox(&mut self.show_grid, "Show grid");
            });

            ui.separator();
            ui.label(egui::RichText::new("View").small().strong());
            ui.horizontal(|ui| {
                // On-screen zoom/pan controls: a reliable fallback for
                // touchpads whose pinch/scroll gestures the OS/windowing
                // layer doesn't deliver to the app (this is a real
                // limitation on Linux — see ROADMAP.md).
                let center = self.canvas_size / 2.0;
                ui.label("Zoom");
                if ui.button("-").on_hover_text("Zoom out").clicked() {
                    self.view.zoom(1.0 / KEY_ZOOM_STEP, center);
                }
                let mut cell_size = self.view.cell_size;
                if ui
                    .add(egui::Slider::new(&mut cell_size, MIN_CELL_SIZE..=MAX_CELL_SIZE).show_value(false))
                    .on_hover_text("Zoom level")
                    .changed()
                {
                    self.view.zoom(cell_size / self.view.cell_size, center);
                }
                if ui.button("+").on_hover_text("Zoom in").clicked() {
                    self.view.zoom(KEY_ZOOM_STEP, center);
                }
                ui.label(format!("{:.0}px/cell", self.view.cell_size));

                ui.separator();
                ui.label("Pan");
                if ui.button("<").on_hover_text("Pan left").clicked() {
                    self.view.pan(Vec2::new(PAN_STEP, 0.0));
                }
                if ui.button("^").on_hover_text("Pan up").clicked() {
                    self.view.pan(Vec2::new(0.0, PAN_STEP));
                }
                if ui.button("v").on_hover_text("Pan down").clicked() {
                    self.view.pan(Vec2::new(0.0, -PAN_STEP));
                }
                if ui.button(">").on_hover_text("Pan right").clicked() {
                    self.view.pan(Vec2::new(-PAN_STEP, 0.0));
                }
                if ui
                    .button("Reset view")
                    .on_hover_text("Recenter on the world and reset zoom")
                    .clicked()
                {
                    self.view = View::default();
                }

                ui.separator();
                ui.checkbox(&mut self.show_input_debug, "Show input debug")
                    .on_hover_text("Live scroll/zoom/touch values, to diagnose gestures that don't do anything");
            });

            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Custom rule — Birth:");
                let mut changed = false;
                for n in 0..=8u8 {
                    let mut on = self.sim.rule.birth[n as usize];
                    if ui.checkbox(&mut on, n.to_string()).changed() {
                        self.sim.rule.birth[n as usize] = on;
                        changed = true;
                    }
                }
                ui.label("Survive:");
                for n in 0..=8u8 {
                    let mut on = self.sim.rule.survive[n as usize];
                    if ui.checkbox(&mut on, n.to_string()).changed() {
                        self.sim.rule.survive[n as usize] = on;
                        changed = true;
                    }
                }
                if changed {
                    self.preset_name = "Custom";
                    self.preset_class = "custom";
                }
            });
        });
    }

    fn side_panel(&mut self, ui: &mut egui::Ui) {
        egui::Panel::left("patterns").min_size(220.0).show(ui, |ui| {
            ui.heading("Pattern Library");
            if self.selected_pattern.is_some() {
                ui.horizontal(|ui| {
                    ui.label("Click canvas to place. ");
                    if ui.button("Cancel").clicked() {
                        self.selected_pattern = None;
                    }
                });
            }
            egui::ScrollArea::vertical().show(ui, |ui| {
                for category in Category::ALL {
                    ui.collapsing(category.label(), |ui| {
                        for (idx, pattern) in self.library.iter().enumerate() {
                            if pattern.category != category {
                                continue;
                            }
                            ui.horizontal(|ui| {
                                let (rect, response) =
                                    ui.allocate_exact_size(Vec2::new(36.0, 36.0), Sense::click());
                                paint_pattern_preview(ui.painter(), rect, &pattern.cells);
                                let label = ui.selectable_label(
                                    self.selected_pattern == Some(idx),
                                    pattern.name,
                                );
                                if response.clicked() || label.clicked() {
                                    self.selected_pattern = Some(idx);
                                    self.eraser_mode = false;
                                }
                            });
                        }
                    });
                }
            });
        });
    }

    fn central_canvas(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        egui::CentralPanel::default().show(ui, |ui| {
            let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
            self.canvas_size = rect.size();
            let painter = ui.painter_at(rect);
            painter.rect_filled(rect, 0.0, Color32::from_gray(18));

            let pointer_local = response.hover_pos().map(|p| p - rect.min);

            // Pinch-to-zoom (touch pinch or ctrl+scroll), anchored on the gesture/pointer.
            let zoom_delta = ctx.input(|i| i.zoom_delta());
            if zoom_delta != 1.0 {
                let anchor = ctx
                    .input(|i| i.multi_touch())
                    .map(|t| t.center_pos - rect.min)
                    .or(pointer_local)
                    .unwrap_or(rect.size() / 2.0);
                self.view.zoom(zoom_delta, anchor);
            }

            // Two-finger trackpad drag pans freely in both directions, like
            // scrolling/panning a map on a phone or tablet. Zooming is a
            // separate, unambiguous gesture (pinch or Ctrl+scroll, handled
            // above via `zoom_delta`), so panning never fights with zoom.
            let scroll_delta = ctx.input(|i| i.smooth_scroll_delta);
            if scroll_delta != Vec2::ZERO {
                self.view.pan(scroll_delta);
            }
            let touch_count = ctx.input(|i| i.multi_touch().map_or(0, |t| t.num_touches));

            // Keyboard shortcuts (ignored while a widget like a text field wants
            // keyboard input, though none currently exist in this app).
            // One-shot actions (Escape/Space/S/C/R) ignore key-repeat events so
            // holding the key down doesn't spam them; zoom repeats on purpose.
            if !ctx.egui_wants_keyboard_input() {
                ctx.input(|i| {
                    for event in &i.events {
                        let &egui::Event::Key { key, pressed: true, repeat, .. } = event else {
                            continue;
                        };
                        match key {
                            Key::Escape if !repeat => self.selected_pattern = None,
                            Key::Space if !repeat => self.sim.running = !self.sim.running,
                            Key::S if !repeat => self.sim.step_n(self.skip_generations),
                            Key::C if !repeat => self.sim.clear(),
                            Key::R if !repeat => {
                                let (min, max) = self.view.visible_bounds(rect.size());
                                self.sim.randomize(min, max, self.random_density);
                            }
                            Key::Plus | Key::Equals => self.view.zoom(KEY_ZOOM_STEP, rect.size() / 2.0),
                            Key::Minus => self.view.zoom(1.0 / KEY_ZOOM_STEP, rect.size() / 2.0),
                            Key::ArrowUp => self.view.pan(Vec2::new(0.0, PAN_STEP)),
                            Key::ArrowDown => self.view.pan(Vec2::new(0.0, -PAN_STEP)),
                            Key::ArrowLeft => self.view.pan(Vec2::new(PAN_STEP, 0.0)),
                            Key::ArrowRight => self.view.pan(Vec2::new(-PAN_STEP, 0.0)),
                            _ => {}
                        }
                    }
                });
            }

            // The minimap lives in the bottom-right corner and intercepts
            // clicks/drags there for navigation instead of painting/stamping.
            let minimap_rect = Rect::from_min_size(rect.max - MINIMAP_SIZE - Vec2::splat(MINIMAP_MARGIN), MINIMAP_SIZE);
            if response.drag_started()
                && let Some(p) = response.interact_pointer_pos()
                && minimap_rect.contains(p)
            {
                self.dragging_minimap = true;
            }
            let minimap_handled = if self.dragging_minimap {
                if let Some(p) = response.interact_pointer_pos().or_else(|| response.hover_pos()) {
                    self.view.center_on(minimap_to_world(minimap_rect, p), self.canvas_size);
                }
                if response.drag_stopped() {
                    self.dragging_minimap = false;
                }
                true
            } else if response.clicked()
                && let Some(p) = response.interact_pointer_pos()
                && minimap_rect.contains(p)
            {
                self.view.center_on(minimap_to_world(minimap_rect, p), self.canvas_size);
                true
            } else {
                false
            };

            if !minimap_handled {
                match self.selected_pattern {
                    Some(idx) => {
                        // Placing a pattern: a single click stamps it once.
                        if response.clicked()
                            && let Some(pointer) = response.interact_pointer_pos()
                        {
                            let cell = self.view.screen_to_cell(rect.min, pointer);
                            let cells = self.library[idx].cells.clone();
                            self.sim.stamp(&cells, cell);
                        }
                    }
                    None => {
                        // Free drawing: press-and-drag paints (or erases) every cell the
                        // cursor passes over, like a paintbrush. When the eraser tool is
                        // active every stroke removes cells regardless of their state,
                        // instead of the draw tool's toggle/paint-a-trail behavior.
                        if response.drag_started() {
                            if let Some(pointer) = response.interact_pointer_pos() {
                                let cell = self.view.screen_to_cell(rect.min, pointer);
                                let value = if self.eraser_mode { false } else { !self.sim.live.contains(&cell) };
                                self.sim.set_cell(cell, value);
                                self.paint_value = Some(value);
                                self.last_paint_cell = Some(cell);
                            }
                        } else if response.dragged() {
                            if let (Some(pointer), Some(value)) = (response.interact_pointer_pos(), self.paint_value) {
                                let cell = self.view.screen_to_cell(rect.min, pointer);
                                if Some(cell) != self.last_paint_cell {
                                    let from = self.last_paint_cell.unwrap_or(cell);
                                    for c in line_cells(from, cell) {
                                        self.sim.set_cell(c, value);
                                    }
                                    self.last_paint_cell = Some(cell);
                                }
                            }
                        } else if response.clicked()
                            && let Some(pointer) = response.interact_pointer_pos()
                        {
                            let cell = self.view.screen_to_cell(rect.min, pointer);
                            if self.eraser_mode {
                                self.sim.set_cell(cell, false);
                            } else {
                                self.sim.toggle_cell(cell);
                            }
                        }
                        if response.drag_stopped() {
                            self.paint_value = None;
                            self.last_paint_cell = None;
                        }
                    }
                }
            }
            if response.secondary_clicked() {
                self.selected_pattern = None;
            }

            // Applied once per frame, after every pan/zoom input this frame
            // (mouse, keyboard, on-screen buttons/slider, minimap) has had
            // its say: keeps the viewport fully inside the world borders.
            self.view.clamp_to_world(self.canvas_size);

            let (min, max) = self.view.visible_bounds(rect.size());
            let cs = self.view.cell_size;

            if self.show_grid && cs > 4.0 {
                let stroke = Stroke::new(1.0, Color32::from_gray(35));
                let mut x = min.0;
                while x <= max.0 {
                    let p = self.view.cell_to_screen(rect.min, (x, 0));
                    painter.line_segment([Pos2::new(p.x, rect.min.y), Pos2::new(p.x, rect.max.y)], stroke);
                    x += 1;
                }
                let mut y = min.1;
                while y <= max.1 {
                    let p = self.view.cell_to_screen(rect.min, (0, y));
                    painter.line_segment([Pos2::new(rect.min.x, p.y), Pos2::new(rect.max.x, p.y)], stroke);
                    y += 1;
                }
            }

            for (x, y) in self.sim.cells_in_bounds(min, max) {
                let p = self.view.cell_to_screen(rect.min, (x, y));
                painter.rect_filled(
                    Rect::from_min_size(p, Vec2::splat(cs)),
                    0.0,
                    Color32::from_rgb(120, 220, 130),
                );
            }

            // Ghost preview of the pattern in hand, following the cursor.
            if let Some(idx) = self.selected_pattern
                && let Some(pointer) = response.hover_pos()
            {
                let base = self.view.screen_to_cell(rect.min, pointer);
                for &(dx, dy) in &self.library[idx].cells {
                    let p = self.view.cell_to_screen(rect.min, (base.0 + dx as i64, base.1 + dy as i64));
                    painter.rect_filled(
                        Rect::from_min_size(p, Vec2::splat(cs)),
                        0.0,
                        Color32::from_rgba_unmultiplied(255, 255, 255, 100),
                    );
                }
            } else if self.eraser_mode
                && let Some(pointer) = response.hover_pos()
            {
                // Eraser cursor: a red outline over the cell it would remove.
                let cell = self.view.screen_to_cell(rect.min, pointer);
                let p = self.view.cell_to_screen(rect.min, cell);
                painter.rect_stroke(
                    Rect::from_min_size(p, Vec2::splat(cs)),
                    0.0,
                    Stroke::new(2.0, Color32::from_rgb(220, 90, 90)),
                    StrokeKind::Outside,
                );
            }

            // The plane is finite: draw its edge wherever it's on-screen, so
            // it's clear painting/patterns stop working past this line.
            let world_screen_min = self.view.cell_to_screen(rect.min, WORLD_MIN);
            let world_screen_max = self.view.cell_to_screen(rect.min, (WORLD_MAX.0 + 1, WORLD_MAX.1 + 1));
            painter.rect_stroke(
                Rect::from_min_max(world_screen_min, world_screen_max),
                0.0,
                Stroke::new(2.0, Color32::from_rgb(190, 90, 90)),
                StrokeKind::Outside,
            );

            draw_minimap(&self.sim, &self.view, self.canvas_size, &painter, minimap_rect);

            if self.show_input_debug {
                let text = format!(
                    "zoom_delta={zoom_delta:.4}\nscroll_delta=({:.1}, {:.1})\ntouches={touch_count}\ncell_size={cs:.1}",
                    scroll_delta.x, scroll_delta.y,
                );
                painter.rect_filled(
                    Rect::from_min_size(rect.min + Vec2::splat(8.0), Vec2::new(220.0, 70.0)),
                    4.0,
                    Color32::from_black_alpha(200),
                );
                painter.text(
                    rect.min + Vec2::splat(12.0),
                    egui::Align2::LEFT_TOP,
                    text,
                    egui::FontId::monospace(13.0),
                    Color32::from_rgb(230, 230, 230),
                );
            }
        });
    }
}

/// Maps a screen point inside the minimap box to the world cell it
/// represents, for click/drag-to-navigate.
fn minimap_to_world(minimap_rect: Rect, p: Pos2) -> Cell {
    let world_w = (WORLD_MAX.0 - WORLD_MIN.0 + 1) as f32;
    let world_h = (WORLD_MAX.1 - WORLD_MIN.1 + 1) as f32;
    let local = p - minimap_rect.min;
    let fx = (local.x / minimap_rect.width()).clamp(0.0, 1.0);
    let fy = (local.y / minimap_rect.height()).clamp(0.0, 1.0);
    (
        (WORLD_MIN.0 as f32 + fx * world_w).round() as i64,
        (WORLD_MIN.1 as f32 + fy * world_h).round() as i64,
    )
}

/// Draws the bottom-right minimap: the whole (finite) plane, a coarse marker
/// per occupied spatial-index chunk (cheap: `O(occupied chunks)`, not
/// `O(live cells)`), and a rectangle showing the current viewport.
fn draw_minimap(sim: &SimState, view: &View, canvas_size: Vec2, painter: &egui::Painter, minimap_rect: Rect) {
    painter.rect_filled(minimap_rect, 4.0, Color32::from_black_alpha(215));

    let world_w = (WORLD_MAX.0 - WORLD_MIN.0 + 1) as f32;
    let world_h = (WORLD_MAX.1 - WORLD_MIN.1 + 1) as f32;
    let sx = minimap_rect.width() / world_w;
    let sy = minimap_rect.height() / world_h;

    for (cx, cy) in sim.occupied_chunks() {
        let x0 = minimap_rect.min.x + ((cx * CHUNK_SIZE) as f32 - WORLD_MIN.0 as f32) * sx;
        let y0 = minimap_rect.min.y + ((cy * CHUNK_SIZE) as f32 - WORLD_MIN.1 as f32) * sy;
        let w = (CHUNK_SIZE as f32 * sx).max(1.0);
        let h = (CHUNK_SIZE as f32 * sy).max(1.0);
        let chunk_rect = Rect::from_min_size(Pos2::new(x0, y0), Vec2::new(w, h)).intersect(minimap_rect);
        painter.rect_filled(chunk_rect, 0.0, Color32::from_rgb(90, 170, 100));
    }

    let (vmin, vmax) = view.visible_bounds(canvas_size);
    let vp_min = Pos2::new(
        minimap_rect.min.x + (vmin.0.clamp(WORLD_MIN.0, WORLD_MAX.0) as f32 - WORLD_MIN.0 as f32) * sx,
        minimap_rect.min.y + (vmin.1.clamp(WORLD_MIN.1, WORLD_MAX.1) as f32 - WORLD_MIN.1 as f32) * sy,
    );
    let vp_max = Pos2::new(
        minimap_rect.min.x + ((vmax.0 + 1).clamp(WORLD_MIN.0, WORLD_MAX.0 + 1) as f32 - WORLD_MIN.0 as f32) * sx,
        minimap_rect.min.y + ((vmax.1 + 1).clamp(WORLD_MIN.1, WORLD_MAX.1 + 1) as f32 - WORLD_MIN.1 as f32) * sy,
    );
    let viewport_rect = Rect::from_min_max(vp_min, vp_max).intersect(minimap_rect);
    painter.rect_stroke(viewport_rect, 0.0, Stroke::new(1.5, Color32::from_rgb(255, 210, 90)), StrokeKind::Outside);

    painter.rect_stroke(minimap_rect, 4.0, Stroke::new(1.0, Color32::from_gray(110)), StrokeKind::Outside);
}

/// Bresenham line between two cells, so fast drags don't leave gaps.
fn line_cells(a: Cell, b: Cell) -> Vec<Cell> {
    let mut cells = Vec::new();
    let (mut x0, mut y0) = a;
    let (x1, y1) = b;
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx: i64 = if x0 < x1 { 1 } else { -1 };
    let sy: i64 = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        cells.push((x0, y0));
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
    cells
}

fn paint_pattern_preview(painter: &egui::Painter, rect: Rect, cells: &[(i32, i32)]) {
    painter.rect_filled(rect, 2.0, Color32::from_gray(28));
    if cells.is_empty() {
        return;
    }
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
    for &(x, y) in cells {
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x);
        max_y = max_y.max(y);
    }
    let w = (max_x - min_x + 1) as f32;
    let h = (max_y - min_y + 1) as f32;
    let pad = 4.0;
    // Fit the pattern's bounding box inside the preview box, shrinking large
    // patterns (e.g. the 36-wide Gosper Glider Gun) instead of forcing at
    // least 1px/cell, which used to make them overflow the tiny icon and
    // look like an unrecognizable blob.
    let scale = ((rect.width() - pad * 2.0) / w).min((rect.height() - pad * 2.0) / h).clamp(0.3, 6.0);
    let origin = rect.min + Vec2::new(pad, pad);
    for &(x, y) in cells {
        let p = origin + Vec2::new((x - min_x) as f32 * scale, (y - min_y) as f32 * scale);
        painter.rect_filled(Rect::from_min_size(p, Vec2::splat(scale.max(1.0))), 0.0, Color32::from_rgb(120, 220, 130));
    }
}
