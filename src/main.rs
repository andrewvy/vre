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
pub const WINDOW_WIDTH: u32 = 800;
pub const WINDOW_HEIGHT: u32 = 600;

fn main() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32))
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
