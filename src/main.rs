#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod shows;
use std::rc::Rc;

use shows::Show;
use shows::Shows;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("vcc", native_options, Box::new(|_| Box::new(Vcc::new())))
}

const NUMBER_LABEL_WIDTH: f32 = 40f32;
const TEXT_LABEL_WIDTH: f32 = 125f32;

struct Vcc {
    shows: Shows,
    adder: Show<String>,
}

impl Vcc {
    fn new() -> Self {
        Vcc {
            shows: Shows::new(),
            adder: Default::default(),
        }
    }

    fn rows(&mut self, changes: &mut Vec<Box<dyn FnOnce(&mut Shows)>>, ui: &mut egui::Ui) {
        let changer =
            |number: &str, show_name: Rc<String>, updater: &mut dyn FnMut(i64, Rc<String>)| {
                let Ok(number) = number.parse::<i64>() else {
                    return;
                };
                updater(number, show_name);
            };

        for show in self.shows.iter() {
            ui.horizontal(|ui| {
                if ui.button("Del").clicked() {
                    let owned_name = show.name.to_owned();
                    changes.push(Box::new(move |shows: &mut Shows| {
                        shows.remove(&owned_name);
                    }));
                }

                ui.separator();

                ui.label("Name: ");
                ui.label(&*show.name);

                ui.separator();

                if ui.button("-").clicked() {
                    changer(
                        &show.season_number,
                        show.name.to_owned(),
                        &mut |number: i64, name: Rc<String>| {
                            changes.push(Box::new(move |shows: &mut Shows| {
                                shows.update(&name, Some(number - 1), None);
                            }));
                        },
                    );
                }

                let season_label = ui.label("Season Number: ");
                let season_number_textbox = ui
                    .add(
                        egui::TextEdit::singleline(&mut show.season_number)
                            .desired_width(NUMBER_LABEL_WIDTH),
                    )
                    .labelled_by(season_label.id);

                if season_number_textbox.changed() {
                    changer(
                        &show.season_number,
                        show.name.to_owned(),
                        &mut |number: i64, name: Rc<String>| {
                            changes.push(Box::new(move |shows: &mut Shows| {
                                shows.update(&name, Some(number), None);
                            }));
                        },
                    );
                }

                if ui.button("+").clicked() {
                    changer(
                        &show.season_number,
                        show.name.to_owned(),
                        &mut |number: i64, name: Rc<String>| {
                            changes.push(Box::new(move |shows: &mut Shows| {
                                shows.update(&name, Some(number + 1), None);
                            }));
                        },
                    );
                }

                ui.separator();

                if ui.button("-").clicked() {
                    changer(
                        &show.episodes_seen,
                        show.name.to_owned(),
                        &mut |number: i64, name: Rc<String>| {
                            changes.push(Box::new(move |shows: &mut Shows| {
                                shows.update(&name, None, Some(number - 1));
                            }));
                        },
                    );
                }

                let episodes_seen_label = ui.label("Episodes Seen: ");
                let episodes_label_textbox = ui
                    .add(
                        egui::TextEdit::singleline(&mut show.episodes_seen)
                            .desired_width(NUMBER_LABEL_WIDTH),
                    )
                    .labelled_by(episodes_seen_label.id);

                if episodes_label_textbox.changed() {
                    changer(
                        &show.episodes_seen,
                        show.name.to_owned(),
                        &mut |number: i64, name: Rc<String>| {
                            changes.push(Box::new(move |shows: &mut Shows| {
                                shows.update(&name, None, Some(number));
                            }));
                        },
                    );
                }

                if ui.button("+").clicked() {
                    changer(
                        &show.episodes_seen,
                        show.name.to_owned(),
                        &mut |number: i64, name: Rc<String>| {
                            changes.push(Box::new(move |shows: &mut Shows| {
                                shows.update(&name, None, Some(number + 1));
                            }));
                        },
                    );
                }
            });
            ui.separator();
        }
    }

    fn add(&mut self, changes: &mut Vec<Box<dyn FnOnce(&mut Shows)>>, ui: &mut egui::Ui) {
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
                return;
            }

            let (Ok(season_number), Ok(episodes_seen)) = (
                self.adder.season_number.parse::<i64>(),
                self.adder.episodes_seen.parse::<i64>(),
            ) else {
                return;
            };

            let owned_name = self.adder.name.to_owned();
            self.adder.clear();
            changes.push(Box::new(move |shows: &mut Shows| {
                shows.add(owned_name, season_number, episodes_seen);
            }));
        }
    }
}

impl eframe::App for Vcc {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut changes: Vec<Box<dyn FnOnce(&mut Shows)>> = Vec::new();

            self.rows(&mut changes, ui);
            self.add(&mut changes, ui);

            // Process accumulated changes
            for change in changes {
                change(&mut self.shows);
            }
        });
    }
}
