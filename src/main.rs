use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use std::error::Error;

mod renderer;
mod utils;

use renderer::Renderer;

pub const WINDOW_TITLE: &'static str = "Vulkan Tutorial";

fn main() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(800.0, 600.0))
        .with_resizable(false)
        .with_title(WINDOW_TITLE)
        .build(&event_loop)
        .expect("Could not create window.");

    let _app = Renderer::new(&window);

    event_loop.run(move |event, _, control_flow| {
        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            *control_flow = ControlFlow::Exit;
        }
    });
}
