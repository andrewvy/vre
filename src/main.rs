use ash::{extensions, version::EntryV1_0, vk, Entry, Instance};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use std::{error::Error, ffi::CString, ptr};

mod utils;

struct SurfaceBundle {
    #[allow(dead_code)]
    surface_loader: extensions::khr::Surface,
    #[allow(dead_code)]
    surface: vk::SurfaceKHR,
}

struct App {
    _entry: Entry,
    #[allow(dead_code)]
    instance: Instance,
    #[allow(dead_code)]
    surface_bundle: SurfaceBundle,
}

pub const WINDOW_TITLE: &'static str = "Vulkan Tutorial";
pub const APPLICATION_VERSION: u32 = vk::make_version(1, 0, 0);
pub const ENGINE_VERSION: u32 = vk::make_version(1, 0, 0);
pub const API_VERSION: u32 = vk::make_version(1, 0, 92);
pub const VALIDATION_LAYERS: [&'static str; 1] = ["VK_LAYER_KHRONOS_validation"];

impl App {
    pub fn new(window: &Window) -> App {
        let entry = App::create_entry();
        let instance = App::create_instance(&entry, window).expect("Could not create VK Instance.");
        let surface_bundle = App::create_surface_bundle(&entry, &instance, &window)
            .expect("Could not create SurfaceBundle.");

        App {
            _entry: entry,
            instance,
            surface_bundle,
        }
    }

    fn create_entry() -> Entry {
        Entry::new().expect("Could not create Vulkan Entry.")
    }

    fn create_instance(entry: &Entry, window: &Window) -> Result<Instance, Box<dyn Error>> {
        if App::check_validation_layer_support(entry) == false {
            panic!("Validation layers requested, but not available!");
        }

        let app_name = CString::new(WINDOW_TITLE).unwrap();
        let engine_name = CString::new("Vulkan Engine").unwrap();
        let app_info = vk::ApplicationInfo {
            p_application_name: app_name.as_ptr(),
            s_type: vk::StructureType::APPLICATION_INFO,
            p_next: ptr::null(),
            application_version: APPLICATION_VERSION,
            p_engine_name: engine_name.as_ptr(),
            engine_version: ENGINE_VERSION,
            api_version: API_VERSION,
        };

        let surface_extensions = ash_window::enumerate_required_extensions(window)?;
        let instance_extensions = surface_extensions
            .iter()
            .map(|ext| ext.as_ptr())
            .collect::<Vec<_>>();

        let raw_validation_layer_names: Vec<CString> = VALIDATION_LAYERS
            .iter()
            .map(|layer_name| CString::new(*layer_name).unwrap())
            .collect();

        let validation_layer_names: Vec<*const i8> = raw_validation_layer_names
            .iter()
            .map(|layer_name| layer_name.as_ptr())
            .collect();

        let instance_desc = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&instance_extensions)
            .enabled_layer_names(&validation_layer_names);

        unsafe {
            let instance = entry.create_instance(&instance_desc, None)?;

            Ok(instance)
        }
    }

    fn create_surface_bundle(
        entry: &Entry,
        instance: &Instance,
        window: &Window,
    ) -> Result<SurfaceBundle, Box<dyn Error>> {
        // Create a surface from winit window.
        let surface = unsafe { ash_window::create_surface(entry, instance, window, None)? };
        let surface_loader = ash::extensions::khr::Surface::new(entry, instance);

        Ok(SurfaceBundle {
            surface,
            surface_loader,
        })
    }

    fn check_validation_layer_support(entry: &Entry) -> bool {
        let layer_properties = entry
            .enumerate_instance_layer_properties()
            .expect("Failed to enumerate Instance Layer Properties");

        if layer_properties.len() <= 0 {
            eprintln!("No available validation layers.");
            return false;
        }

        for required_layer_name in VALIDATION_LAYERS.iter() {
            let mut is_layer_found = false;

            for layer_property in layer_properties.iter() {
                let test_layer_name = utils::vk_to_string(&layer_property.layer_name);

                if (*required_layer_name) == test_layer_name {
                    is_layer_found = true;
                    break;
                }
            }

            if is_layer_found == false {
                return false;
            }
        }

        true
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(800.0, 600.0))
        .with_resizable(false)
        .with_title(WINDOW_TITLE)
        .build(&event_loop)
        .expect("Could not create window.");

    let _app = App::new(&window);

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
