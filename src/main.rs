#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod show;
mod shows_db;
mod shows_view;
mod ui_painter;

use ui_painter::Vcc;

use egui_winit_vulkano::{Gui, GuiConfig};

use vulkano_util::{
    context::{VulkanoConfig, VulkanoContext},
    window::{VulkanoWindows, WindowDescriptor},
};

use winit::{
    application::ApplicationHandler, error::EventLoopError, event::WindowEvent,
    event_loop::EventLoop,
};

struct VccApplication {
    context: VulkanoContext,
    windows: VulkanoWindows,
    vcc: Vcc,
    gui: Option<Gui>,
}

impl VccApplication {
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

impl ApplicationHandler for VccApplication {
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
                    self.vcc.paint_ui(gui);
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
    let mut vcc_handler = VccApplication::new();

    event_loop.run_app(&mut vcc_handler)
}
