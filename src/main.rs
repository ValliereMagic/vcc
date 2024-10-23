#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod show;
mod shows_db;
mod shows_view;

use show::{AdderShow, ShowCategory};
use shows_view::{ShowsView, UiShowCategory};

use egui_winit_vulkano::{Gui, GuiConfig};

use vulkano_util::{
    context::{VulkanoConfig, VulkanoContext},
    window::{VulkanoWindows, WindowDescriptor},
};

use winit::{
    application::ApplicationHandler, error::EventLoopError, event::WindowEvent,
    event_loop::EventLoop,
};

struct VccHandler {
    context: VulkanoContext,
    windows: VulkanoWindows,
    vcc: Vcc,
    gui: Option<Gui>,
}

impl VccHandler {
    fn new() -> Self {
        let context = VulkanoContext::new(VulkanoConfig::default());
        let windows = VulkanoWindows::default();
        let vcc = Vcc::new();
        Self {
            context,
            windows,
            vcc,
            gui: None,
        }
    }
}

impl ApplicationHandler for VccHandler {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.windows.create_window(
            event_loop,
            &self.context,
            &WindowDescriptor::default(),
            |ci| {
                ci.image_format = vulkano::format::Format::B8G8R8A8_UNORM;
                ci.min_image_count = ci.min_image_count.max(2);
            },
        );

        // Create gui as main render pass (no overlay means it clears the image each frame)
        self.gui = Some({
            let renderer = self.windows.get_primary_renderer_mut().unwrap();
            Gui::new(
                event_loop,
                renderer.surface(),
                renderer.graphics_queue(),
                renderer.swapchain_format(),
                GuiConfig::default(),
            )
        });
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let renderer = self.windows.get_renderer_mut(window_id).unwrap();

        let gui = self.gui.as_mut().unwrap();

        // Update Egui integration so the UI works!
        let _ = !gui.update(&event);

        match event {
            WindowEvent::Resized(_) => {
                renderer.resize();
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                renderer.resize();
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // Set immediate UI in redraw here
                gui.immediate_ui(|gui| {
                    self.vcc.draw_gui(gui);
                });
                // Render UI
                // Acquire swapchain future
                match renderer.acquire(Some(std::time::Duration::from_millis(10)), |_| {}) {
                    Ok(future) => {
                        // Render gui
                        let after_future = self
                            .gui
                            .as_mut()
                            .unwrap()
                            .draw_on_image(future, renderer.swapchain_image_view());
                        // Present swapchain
                        renderer.present(after_future, true);
                    }
                    Err(vulkano::VulkanError::OutOfDate) => {
                        renderer.resize();
                    }
                    Err(e) => panic!("Failed to acquire swapchain future: {}", e),
                };
            }
            _ => (),
        }
    }
    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        let renderer = self.windows.get_primary_renderer_mut().unwrap();
        renderer.window().request_redraw();
    }
}

pub fn main() -> Result<(), EventLoopError> {
    let event_loop = EventLoop::new().unwrap();

    // Application state
    let mut vcc_handler = VccHandler::new();

    event_loop.run_app(&mut vcc_handler)
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

    fn draw_gui(&mut self, gui: &mut Gui) {
        let ctx = gui.context();
        egui::CentralPanel::default().show(&ctx, |ui| {
            self.search_page(ui);
            self.rows(ui);
            self.add(ui);

            for modification in self.accumulated_modifications.drain(..) {
                modification(&mut self.shows);
            }
        });
    }

    fn search_page(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let search_box_label = ui.label("Search: ");
            let search_box = ui
                .add(
                    egui::TextEdit::singleline(self.shows.search_box())
                        .desired_width(TEXT_LABEL_WIDTH),
                )
                .labelled_by(search_box_label.id);
            if search_box.changed() {
                self.shows.search();
            }

            ui.separator();

            let category_label = ui.label("Category: ");

            if ui
                .add(egui::SelectableLabel::new(
                    *self.shows.current_category() == UiShowCategory::Watching,
                    "Watching",
                ))
                .labelled_by(category_label.id)
                .clicked()
            {
                *self.shows.current_category() = UiShowCategory::Watching;
                self.shows.update_category();
            }

            if ui
                .add(egui::SelectableLabel::new(
                    *self.shows.current_category() == UiShowCategory::PlanToWatch,
                    "Plan to Watch",
                ))
                .labelled_by(category_label.id)
                .clicked()
            {
                *self.shows.current_category() = UiShowCategory::PlanToWatch;
                self.shows.update_category();
            }

            if ui
                .add(egui::SelectableLabel::new(
                    *self.shows.current_category() == UiShowCategory::Completed,
                    "Completed",
                ))
                .labelled_by(category_label.id)
                .clicked()
            {
                *self.shows.current_category() = UiShowCategory::Completed;
                self.shows.update_category();
            }

            if ui
                .add(egui::SelectableLabel::new(
                    *self.shows.current_category() == UiShowCategory::All,
                    "All",
                ))
                .labelled_by(category_label.id)
                .clicked()
            {
                *self.shows.current_category() = UiShowCategory::All;
                self.shows.update_category();
            }

            ui.separator();

            ui.label("Page Number: ");

            if ui.button("-").clicked() {
                self.shows.previous_page();
            }

            ui.label(format!(
                "{} of {}",
                self.shows.page(),
                self.shows.page_count(),
            ));

            if ui.button("+").clicked() {
                self.shows.next_page();
            }
        });

        ui.separator();
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

        egui::Grid::new("display_show_grid").show(ui, |ui| {
            for (index, show) in self.shows.iter_mut() {
                ui.horizontal(|ui| {
                    if ui.button("Del").clicked() {
                        modifications.push(Box::new(move |shows: &mut ShowsView| {
                            shows.remove(index);
                        }));
                    }
                    ui.separator();
                });

                ui.horizontal(|ui| {
                    ui.label("Name: ");
                    ui.label(show.name().as_str());
                });

                ui.horizontal(|ui| {
                    ui.separator();

                    if ui.button("-").clicked()
                        && changer(&mut show.season_number, &mut |curr| curr - 1)
                    {
                        modifications.push(Box::new(move |shows: &mut ShowsView| {
                            shows.update(index);
                        }));
                    }

                    let season_label = ui.label("Season Number: ");
                    let season_number_textbox = ui
                        .add(
                            egui::TextEdit::singleline(&mut show.season_number)
                                .desired_width(NUMBER_LABEL_WIDTH),
                        )
                        .labelled_by(season_label.id);

                    if season_number_textbox.changed()
                        && changer(&mut show.season_number, &mut |curr| curr)
                    {
                        modifications.push(Box::new(move |shows: &mut ShowsView| {
                            shows.update(index);
                        }));
                    }

                    if ui.button("+").clicked()
                        && changer(&mut show.season_number, &mut |curr| curr + 1)
                    {
                        modifications.push(Box::new(move |shows: &mut ShowsView| {
                            shows.update(index);
                        }));
                    }

                    ui.separator();
                });
                ui.horizontal(|ui| {
                    if ui.button("-").clicked()
                        && changer(&mut show.episodes_seen, &mut |curr| curr - 1)
                    {
                        modifications.push(Box::new(move |shows: &mut ShowsView| {
                            shows.update(index);
                        }));
                    }

                    let episodes_seen_label = ui.label("Episodes Seen: ");
                    let episodes_label_textbox = ui
                        .add(
                            egui::TextEdit::singleline(&mut show.episodes_seen)
                                .desired_width(NUMBER_LABEL_WIDTH),
                        )
                        .labelled_by(episodes_seen_label.id);

                    if episodes_label_textbox.changed()
                        && changer(&mut show.episodes_seen, &mut |curr| curr)
                    {
                        modifications.push(Box::new(move |shows: &mut ShowsView| {
                            shows.update(index);
                        }));
                    }

                    if ui.button("+").clicked()
                        && changer(&mut show.episodes_seen, &mut |curr| curr + 1)
                    {
                        modifications.push(Box::new(move |shows: &mut ShowsView| {
                            shows.update(index);
                        }));
                    }

                    ui.separator();
                });
                let category_label = ui.label("Category: ");
                egui::ComboBox::from_id_salt(category_label.id)
                    .selected_text(format!("{:?}", show.category))
                    .show_ui(ui, |ui| {
                        let watch = ui
                            .selectable_value(
                                &mut show.category,
                                ShowCategory::Watching,
                                "Watching",
                            )
                            .changed();
                        let plan = ui
                            .selectable_value(
                                &mut show.category,
                                ShowCategory::PlanToWatch,
                                "PlanToWatch",
                            )
                            .changed();
                        let complete = ui
                            .selectable_value(
                                &mut show.category,
                                ShowCategory::Completed,
                                "Completed",
                            )
                            .changed();

                        if watch || plan || complete {
                            modifications.push(Box::new(move |shows: &mut ShowsView| {
                                shows.update(index);
                            }));
                        }
                    });
                ui.end_row();
            }
        });
        ui.separator();
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

            let seasons_label = ui.label("Season Number: ");
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

            ui.separator();

            let category_label = ui.label("Category: ");
            egui::ComboBox::from_id_salt(category_label.id)
                .selected_text(format!("{:?}", self.adder.category))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.adder.category,
                        ShowCategory::Watching,
                        "Watching",
                    );
                    ui.selectable_value(
                        &mut self.adder.category,
                        ShowCategory::PlanToWatch,
                        "PlanToWatch",
                    );

                    ui.selectable_value(
                        &mut self.adder.category,
                        ShowCategory::Completed,
                        "Completed",
                    );
                });
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
