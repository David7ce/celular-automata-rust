use eframe::egui::{Pos2, Vec2};

use crate::simulation::Cell;

pub const MIN_CELL_SIZE: f32 = 2.0;
pub const MAX_CELL_SIZE: f32 = 60.0;

/// Camera over the infinite grid: `offset` is the world-space point (in cell
/// units) shown at the top-left of the canvas; `cell_size` is the on-screen
/// pixel size of one cell (the zoom level).
pub struct View {
    pub offset: Vec2,
    pub cell_size: f32,
}

impl Default for View {
    fn default() -> Self {
        View { offset: Vec2::new(-20.0, -15.0), cell_size: 16.0 }
    }
}

impl View {
    pub fn screen_to_cell(&self, screen_origin: Pos2, p: Pos2) -> Cell {
        let local = p - screen_origin;
        let wx = self.offset.x + local.x / self.cell_size;
        let wy = self.offset.y + local.y / self.cell_size;
        (wx.floor() as i64, wy.floor() as i64)
    }

    pub fn cell_to_screen(&self, screen_origin: Pos2, cell: Cell) -> Pos2 {
        let wx = cell.0 as f32 - self.offset.x;
        let wy = cell.1 as f32 - self.offset.y;
        screen_origin + Vec2::new(wx * self.cell_size, wy * self.cell_size)
    }

    pub fn pan(&mut self, screen_delta: Vec2) {
        self.offset -= screen_delta / self.cell_size;
    }

    /// Zooms so that the world point currently under `anchor` (screen-space,
    /// relative to the canvas origin) stays fixed.
    pub fn zoom(&mut self, factor: f32, anchor_local: Vec2) {
        let world_x = self.offset.x + anchor_local.x / self.cell_size;
        let world_y = self.offset.y + anchor_local.y / self.cell_size;

        self.cell_size = (self.cell_size * factor).clamp(MIN_CELL_SIZE, MAX_CELL_SIZE);

        self.offset.x = world_x - anchor_local.x / self.cell_size;
        self.offset.y = world_y - anchor_local.y / self.cell_size;
    }

    /// Visible cell bounds (inclusive) for a canvas of the given pixel size.
    pub fn visible_bounds(&self, canvas_size: Vec2) -> (Cell, Cell) {
        let min = (self.offset.x.floor() as i64, self.offset.y.floor() as i64);
        let max = (
            (self.offset.x + canvas_size.x / self.cell_size).ceil() as i64,
            (self.offset.y + canvas_size.y / self.cell_size).ceil() as i64,
        );
        (min, max)
    }

    /// Re-centers the view on `cell` (keeping the current zoom level), for
    /// minimap click/drag navigation.
    pub fn center_on(&mut self, cell: Cell, canvas_size: Vec2) {
        self.offset = Vec2::new(cell.0 as f32, cell.1 as f32) - (canvas_size / 2.0) / self.cell_size;
    }
}
