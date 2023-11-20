#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod shows;
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
    adder: Show,
}

impl Vcc {
    fn new() -> Self {
        Vcc {
            shows: Shows::new(),
            adder: Default::default(),
        }
    }
}

impl eframe::App for Vcc {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut changes: Vec<Box<dyn Fn(&mut Shows)>> = Vec::new();

            let changer = |number: String,
                           show_name: String,
                           up_down: bool,
                           episode_count: bool,
                           changes: &mut Vec<Box<dyn Fn(&mut Shows)>>| {
                let Ok(number) = number.parse::<i64>() else {
                    return;
                };
                changes.push(Box::new(move |shows: &mut Shows| {
                    if episode_count {
                        if up_down {
                            shows.update(&show_name, None, Some(number + 1));
                        } else {
                            shows.update(&show_name, None, Some(number - 1));
                        }
                    } else {
                        if up_down {
                            shows.update(&show_name, Some(number + 1), None);
                        } else {
                            shows.update(&show_name, Some(number - 1), None);
                        }
                    }
                }));
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
                    ui.label(&show.name);

                    ui.separator();

                    if ui.button("-").clicked() {
                        changer(
                            show.season_number.to_owned(),
                            show.name.to_owned(),
                            false,
                            false,
                            &mut changes,
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
                        let Ok(season_number) = show.season_number.parse::<i64>() else {
                            return;
                        };

                        let owned_name = show.name.to_owned();
                        changes.push(Box::new(move |shows: &mut Shows| {
                            shows.update(&owned_name, Some(season_number), None);
                        }));
                    }

                    if ui.button("+").clicked() {
                        changer(
                            show.season_number.to_owned(),
                            show.name.to_owned(),
                            true,
                            false,
                            &mut changes,
                        );
                    }

                    ui.separator();

                    if ui.button("-").clicked() {
                        changer(
                            show.episodes_seen.to_owned(),
                            show.name.to_owned(),
                            false,
                            true,
                            &mut changes,
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
                        let Ok(episode_number) = show.episodes_seen.parse::<i64>() else {
                            return;
                        };
                        let owned_name = show.name.to_owned();
                        changes.push(Box::new(move |shows: &mut Shows| {
                            shows.update(&owned_name, None, Some(episode_number));
                        }))
                    }

                    if ui.button("+").clicked() {
                        changer(
                            show.episodes_seen.to_owned(),
                            show.name.to_owned(),
                            true,
                            true,
                            &mut changes,
                        );
                    }
                });
                ui.separator();
            }

            ui.label("Add new show");
            ui.horizontal(|ui| {
                let name_label = ui.label("Name: ");
                ui.add(
                    egui::TextEdit::singleline(&mut self.adder.name)
                        .desired_width(TEXT_LABEL_WIDTH),
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
                    shows.add(&owned_name, season_number, episodes_seen);
                }))
            }

            // Process accumulated changes
            for change in changes {
                change(&mut self.shows);
            }
        });
    }
}
