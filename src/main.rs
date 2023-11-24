#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod show;
mod shows_db;
mod shows_view;
use std::i64;

use show::AdderShow;
use shows_view::ShowsView;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("vcc", native_options, Box::new(|_| Box::new(Vcc::new())))
}

const NUMBER_LABEL_WIDTH: f32 = 40f32;
const TEXT_LABEL_WIDTH: f32 = 125f32;

type AccumulatedModifications = Vec<Box<dyn FnOnce(&mut ShowsView)>>;
struct Vcc {
    shows: ShowsView,
    adder: AdderShow,
    accumulated_modifications: AccumulatedModifications,
}

impl Vcc {
    fn new() -> Self {
        Vcc {
            shows: ShowsView::new(),
            adder: Default::default(),
            accumulated_modifications: Default::default(),
        }
    }

    fn rows(&mut self, ui: &mut egui::Ui) {
        let changer = |show_field: &mut String, updater: &mut dyn FnMut(i64) -> i64| -> bool {
            let Ok(number) = show_field.parse::<i64>() else {
                return false;
            };
            *show_field = format!("{}", updater(number));
            true
        };
        let modifications = &mut self.accumulated_modifications;

        for (index, show) in self.shows.iter() {
            ui.horizontal(|ui| {
                if ui.button("Del").clicked() {
                    modifications.push(Box::new(move |shows: &mut ShowsView| {
                        shows.remove(index);
                    }));
                }

                ui.separator();

                ui.label("Name: ");
                ui.label(&*show.name);

                ui.separator();

                if ui.button("-").clicked() {
                    if changer(&mut show.season_number, &mut |curr| curr - 1) {
                        modifications.push(Box::new(move |shows: &mut ShowsView| {
                            shows.update(index);
                        }));
                    }
                }

                let season_label = ui.label("Season Number: ");
                let season_number_textbox = ui
                    .add(
                        egui::TextEdit::singleline(&mut show.season_number)
                            .desired_width(NUMBER_LABEL_WIDTH),
                    )
                    .labelled_by(season_label.id);

                if season_number_textbox.changed() {
                    if changer(&mut show.season_number, &mut |curr| curr) {
                        modifications.push(Box::new(move |shows: &mut ShowsView| {
                            shows.update(index);
                        }));
                    }
                }

                if ui.button("+").clicked() {
                    if changer(&mut show.season_number, &mut |curr| curr + 1) {
                        modifications.push(Box::new(move |shows: &mut ShowsView| {
                            shows.update(index);
                        }));
                    }
                }

                ui.separator();

                if ui.button("-").clicked() {
                    if changer(&mut show.episodes_seen, &mut |curr| curr - 1) {
                        modifications.push(Box::new(move |shows: &mut ShowsView| {
                            shows.update(index);
                        }));
                    }
                }

                let episodes_seen_label = ui.label("Episodes Seen: ");
                let episodes_label_textbox = ui
                    .add(
                        egui::TextEdit::singleline(&mut show.episodes_seen)
                            .desired_width(NUMBER_LABEL_WIDTH),
                    )
                    .labelled_by(episodes_seen_label.id);

                if episodes_label_textbox.changed() {
                    if changer(&mut show.episodes_seen, &mut |curr| curr) {
                        modifications.push(Box::new(move |shows: &mut ShowsView| {
                            shows.update(index);
                        }));
                    }
                }

                if ui.button("+").clicked() {
                    if changer(&mut show.episodes_seen, &mut |curr| curr + 1) {
                        modifications.push(Box::new(move |shows: &mut ShowsView| {
                            shows.update(index);
                        }));
                    }
                }
            });
            ui.separator();
        }
    }

    fn add(&mut self, ui: &mut egui::Ui) {
        ui.label("Add new show");
        ui.horizontal(|ui| {
            let name_label = ui.label("Name: ");
            ui.add(
                egui::TextEdit::singleline(&mut self.adder.name).desired_width(TEXT_LABEL_WIDTH),
            )
            .labelled_by(name_label.id);

            ui.separator();

            let seasons_label = ui.label("Seasons seen: ");
            ui.add(
                egui::TextEdit::singleline(&mut self.adder.season_number)
                    .desired_width(NUMBER_LABEL_WIDTH),
            )
            .labelled_by(seasons_label.id);

            ui.separator();

            let episodes_label = ui.label("Episodes Seen: ");
            ui.add(
                egui::TextEdit::singleline(&mut self.adder.episodes_seen)
                    .desired_width(NUMBER_LABEL_WIDTH),
            )
            .labelled_by(episodes_label.id);
        });
        if ui.button("Add").clicked() {
            if self.adder.name.is_empty() {
                self.adder.clear();
                return;
            }

            let (Ok(_), Ok(_)) = (
                self.adder.season_number.parse::<i64>(),
                self.adder.episodes_seen.parse::<i64>(),
            ) else {
                self.adder.clear();
                return;
            };

            let owned_adder = self.adder.to_owned();
            self.accumulated_modifications
                .push(Box::new(move |shows: &mut ShowsView| {
                    shows.add(owned_adder);
                }));
            self.adder.clear();
        }
    }
}

impl eframe::App for Vcc {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.rows(ui);
            self.add(ui);

            for modification in self.accumulated_modifications.drain(..) {
                modification(&mut self.shows);
            }
        });
    }
}
