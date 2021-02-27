use winit::window::Window;

mod vulkan;

use vulkan::VulkanBackend;

pub struct Renderer {
    _backend: VulkanBackend,
}

impl Renderer {
    pub fn new(window: &Window) -> Renderer {
        let backend = vulkan::VulkanBackend::new(&window);

        Renderer { _backend: backend }
    }
}
