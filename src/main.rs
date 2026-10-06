use eframe::egui;
use tug_of_war::ui;

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([1000.0, 700.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Tug-of-War DP Simulator",
        options,
        Box::new(|_cc| Box::new(ui::TugOfWarApp::default())),
    ).expect("Failed to start application");
}