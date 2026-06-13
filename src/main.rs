#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod show;
mod shows_db;
mod shows_view;
mod ui_painter;

use ui_painter::Vcc;

use eframe::egui::{self};

struct VccApplication {
    vcc: Vcc,
}

impl VccApplication {
    fn new() -> Self {
        let vcc = Vcc::new();
        Self { vcc }
    }
}

impl eframe::App for VccApplication {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.vcc.paint_ui(ui);
    }
}

pub fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(),
        ..Default::default()
    };

    eframe::run_native(
        "vcc",
        options,
        Box::new(|_| Ok(Box::new(VccApplication::new()))),
    )
}
