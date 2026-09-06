mod app;
mod patterns;
mod rle;
mod rules;
mod simulation;
mod view;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_inner_size([1100.0, 720.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Game of Life",
        options,
        Box::new(|_cc| Ok(Box::new(app::App::new()))),
    )
}
