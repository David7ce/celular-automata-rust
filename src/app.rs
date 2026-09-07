use eframe::egui::{self, Color32, Key, Pos2, Rect, Sense, Stroke, Vec2};

use crate::patterns::{self, Category, Pattern};
use crate::rules;
use crate::simulation::{Cell, SimState};
use crate::view::View;

/// Zoom factor applied per keyboard zoom-shortcut press ('+'/'-').
const KEY_ZOOM_STEP: f32 = 1.2;

pub struct App {
    sim: SimState,
    view: View,
    library: Vec<Pattern>,
    selected_pattern: Option<usize>,
    preset_name: &'static str,
    random_density: f32,
    /// Whether the current drag-paint stroke is drawing (true) or erasing (false).
    paint_value: Option<bool>,
    last_paint_cell: Option<Cell>,
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
            random_density: 0.35,
            paint_value: None,
            last_paint_cell: None,
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
            ui.horizontal(|ui| {
                ui.label("Rule:");
                egui::ComboBox::from_id_salt("rule_preset")
                    .selected_text(self.preset_name)
                    .show_ui(ui, |ui| {
                        for preset in rules::PRESETS {
                            if ui.selectable_label(self.preset_name == preset.name, preset.name).clicked() {
                                self.preset_name = preset.name;
                                self.sim.rule = rules::preset_rule(preset);
                            }
                        }
                    });
                ui.label(self.sim.rule.to_bs_string());

                ui.separator();

                if ui.button(if self.sim.running { "Pause" } else { "Play" }).clicked() {
                    self.sim.running = !self.sim.running;
                }
                if ui.button("Step").clicked() {
                    self.sim.step();
                }
                if ui.button("Clear").clicked() {
                    self.sim.clear();
                }
                if ui.button("Random").clicked() {
                    let (min, max) = self.view.visible_bounds(ui.available_size().max(Vec2::new(400.0, 400.0)));
                    self.sim.randomize(min, max, self.random_density);
                }

                ui.separator();
                ui.label("Speed");
                ui.add(egui::Slider::new(&mut self.sim.speed, 0.5..=60.0).suffix(" gen/s"));

                ui.separator();
                ui.label(format!("Gen: {}", self.sim.generation));
                ui.label(format!("Live: {}", self.sim.live.len()));
            });

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
                            Key::S if !repeat => self.sim.step(),
                            Key::C if !repeat => self.sim.clear(),
                            Key::R if !repeat => {
                                let (min, max) = self.view.visible_bounds(rect.size());
                                self.sim.randomize(min, max, self.random_density);
                            }
                            Key::Plus | Key::Equals => self.view.zoom(KEY_ZOOM_STEP, rect.size() / 2.0),
                            Key::Minus => self.view.zoom(1.0 / KEY_ZOOM_STEP, rect.size() / 2.0),
                            _ => {}
                        }
                    }
                });
            }

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
                    // cursor passes over, like a paintbrush.
                    if response.drag_started() {
                        if let Some(pointer) = response.interact_pointer_pos() {
                            let cell = self.view.screen_to_cell(rect.min, pointer);
                            let value = !self.sim.live.contains(&cell);
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
                        self.sim.toggle_cell(cell);
                    }
                    if response.drag_stopped() {
                        self.paint_value = None;
                        self.last_paint_cell = None;
                    }
                }
            }
            if response.secondary_clicked() {
                self.selected_pattern = None;
            }

            let (min, max) = self.view.visible_bounds(rect.size());
            let cs = self.view.cell_size;

            if cs > 4.0 {
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
            }

        });
    }
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
    let scale = ((rect.width() - pad * 2.0) / w).min((rect.height() - pad * 2.0) / h).max(1.0);
    let origin = rect.min + Vec2::new(pad, pad);
    for &(x, y) in cells {
        let p = origin + Vec2::new((x - min_x) as f32 * scale, (y - min_y) as f32 * scale);
        painter.rect_filled(Rect::from_min_size(p, Vec2::splat(scale.max(1.0))), 0.0, Color32::from_rgb(120, 220, 130));
    }
}
