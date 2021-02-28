use std::{error::Error, ffi::CString, ptr};
use std::{ffi::CStr, os::raw::c_void};

use ash::extensions::ext::DebugUtils;
use ash::{
    extensions,
    version::{DeviceV1_0, EntryV1_0, InstanceV1_0},
    vk, Device, Entry, Instance,
};

use winit::window::Window;

use self::swapchain::SwapchainBundle;

mod device;
mod swapchain;

pub const APPLICATION_VERSION: u32 = vk::make_version(1, 0, 0);
pub const ENGINE_VERSION: u32 = vk::make_version(1, 0, 0);
pub const API_VERSION: u32 = vk::make_version(1, 0, 92);
pub const VALIDATION_LAYERS: [&'static str; 1] = ["VK_LAYER_KHRONOS_validation"];
pub const REQUIRED_DEVICE_EXTENSIONS: [&'static str; 1] = ["VK_KHR_swapchain"];

unsafe extern "system" fn vulkan_debug_utils_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut c_void,
) -> vk::Bool32 {
    let severity = match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => "[Verbose]",
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => "[Warning]",
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => "[Error]",
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => "[Info]",
        _ => "[Unknown]",
    };
    let types = match message_type {
        vk::DebugUtilsMessageTypeFlagsEXT::GENERAL => "[General]",
        vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "[Performance]",
        vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION => "[Validation]",
        _ => "[Unknown]",
    };
    let message = CStr::from_ptr((*p_callback_data).p_message);
    println!("[Debug]{}{}{:?}", severity, types, message);

    vk::FALSE
}

pub struct QueueFamilyIndices {
    graphics_family: Option<u32>,
    present_family: Option<u32>,
}

impl QueueFamilyIndices {
    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
}

pub struct SurfaceBundle {
    #[allow(dead_code)]
    surface_loader: extensions::khr::Surface,
    #[allow(dead_code)]
    surface: vk::SurfaceKHR,
}

pub struct VulkanBackend {
    _entry: Entry,
    #[allow(dead_code)]
    instance: Instance,
    #[allow(dead_code)]
    surface_bundle: SurfaceBundle,

    #[allow(dead_code)]
    debug_utils_loader: ash::extensions::ext::DebugUtils,
    #[allow(dead_code)]
    debug_messenger: vk::DebugUtilsMessengerEXT,
    #[allow(dead_code)]
    physical_device: vk::PhysicalDevice,
    #[allow(dead_code)]
    logical_device: Device,
    #[allow(dead_code)]
    graphics_queue: vk::Queue,
    #[allow(dead_code)]
    present_queue: vk::Queue,

    #[allow(dead_code)]
    swapchain_bundle: SwapchainBundle,
}

impl VulkanBackend {
    pub fn new(window: &Window) -> VulkanBackend {
        let entry = VulkanBackend::create_entry();
        let instance =
            VulkanBackend::create_instance(&entry, window).expect("Could not create VK Instance.");
        let (debug_utils_loader, debug_messenger) =
            VulkanBackend::setup_debug_utils(&entry, &instance);
        let surface_bundle = VulkanBackend::create_surface_bundle(&entry, &instance, &window)
            .expect("Could not create SurfaceBundle.");
        let physical_device = VulkanBackend::get_physical_device(&instance, &surface_bundle);
        let (logical_device, indices) =
            device::create_logical_device(&instance, physical_device, &surface_bundle);

        let graphics_queue =
            unsafe { logical_device.get_device_queue(indices.graphics_family.unwrap(), 0) };
        let present_queue =
            unsafe { logical_device.get_device_queue(indices.present_family.unwrap(), 0) };

        let swapchain_bundle = SwapchainBundle::new(
            &instance,
            &logical_device,
            physical_device,
            &surface_bundle,
            indices,
        );

        VulkanBackend {
            _entry: entry,
            instance,
            surface_bundle,
            debug_utils_loader,
            debug_messenger,
            physical_device,
            logical_device,
            graphics_queue,
            present_queue,
            swapchain_bundle,
        }
    }

    pub fn get_physical_device(
        instance: &Instance,
        surface_bundle: &SurfaceBundle,
    ) -> vk::PhysicalDevice {
        VulkanBackend::devices(instance)
            .expect("Could not fetch devices.")
            .into_iter()
            .find(|device| VulkanBackend::is_device_suitable(instance, *device, surface_bundle))
            .expect("Could not find a suitable PhysicalDevice!")
    }

    pub fn devices(instance: &Instance) -> Result<Vec<vk::PhysicalDevice>, Box<dyn Error>> {
        let devices = unsafe { instance.enumerate_physical_devices()? };
        Ok(devices)
    }

    fn is_device_suitable(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
        surface_bundle: &SurfaceBundle,
    ) -> bool {
        let device_properties = unsafe { instance.get_physical_device_properties(physical_device) };
        let is_required_extensions_supported =
            device::check_device_extension_support(instance, physical_device);
        let is_swapchain_supported = if is_required_extensions_supported {
            let swapchain_details =
                swapchain::SwapchainSupportDetails::new(physical_device, surface_bundle);
            !swapchain_details.formats.is_empty() && !swapchain_details.present_modes.is_empty()
        } else {
            false
        };
        let is_queue_family_supported =
            device::find_queue_family(instance, physical_device, surface_bundle).is_complete();

        device_properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU
            && is_queue_family_supported
            && is_required_extensions_supported
            && is_swapchain_supported
    }

    fn create_entry() -> Entry {
        Entry::new().expect("Could not create Vulkan Entry.")
    }

    fn create_instance(entry: &Entry, window: &Window) -> Result<Instance, Box<dyn Error>> {
        let has_validation_layer_support = VulkanBackend::check_validation_layer_support(entry);

        if !has_validation_layer_support {
            eprintln!("Validation layers requested, but not available!");
        }

        let app_name = CString::new(crate::WINDOW_TITLE).unwrap();
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

        let debug_utils_messenger_info = create_debug_utils_messenger_info();

        let mut surface_extensions = ash_window::enumerate_required_extensions(window)?;

        if has_validation_layer_support {
            surface_extensions.push(DebugUtils::name());
        }

        let instance_extensions = surface_extensions
            .iter()
            .map(|ext| ext.as_ptr())
            .collect::<Vec<_>>();

        let raw_validation_layer_names: Vec<CString> = VALIDATION_LAYERS
            .iter()
            .map(|layer_name| CString::new(*layer_name).unwrap())
            .collect();

        let validation_layer_names: Vec<*const i8> = if has_validation_layer_support {
            raw_validation_layer_names
                .iter()
                .map(|layer_name| layer_name.as_ptr())
                .collect()
        } else {
            Vec::new()
        };

        let mut instance_desc = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&instance_extensions)
            .enabled_layer_names(&validation_layer_names);

        if has_validation_layer_support {
            instance_desc.p_next = &debug_utils_messenger_info
                as *const vk::DebugUtilsMessengerCreateInfoEXT
                as *const c_void;
        }

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
                let test_layer_name = crate::utils::vk_to_string(&layer_property.layer_name);

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

    fn setup_debug_utils(
        entry: &ash::Entry,
        instance: &ash::Instance,
    ) -> (ash::extensions::ext::DebugUtils, vk::DebugUtilsMessengerEXT) {
        let has_validation_layer_support = VulkanBackend::check_validation_layer_support(entry);
        let debug_utils_loader = ash::extensions::ext::DebugUtils::new(entry, instance);

        if has_validation_layer_support {
            (debug_utils_loader, ash::vk::DebugUtilsMessengerEXT::null())
        } else {
            let messenger_ci = create_debug_utils_messenger_info();

            let utils_messenger = unsafe {
                debug_utils_loader
                    .create_debug_utils_messenger(&messenger_ci, None)
                    .expect("Debug Utils Callback")
            };

            (debug_utils_loader, utils_messenger)
        }
    }
}

fn create_debug_utils_messenger_info() -> vk::DebugUtilsMessengerCreateInfoEXT {
    vk::DebugUtilsMessengerCreateInfoEXT {
        s_type: vk::StructureType::DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
        p_next: ptr::null(),
        flags: vk::DebugUtilsMessengerCreateFlagsEXT::empty(),
        message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
            | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
            | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
            | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
        message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
            | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
            | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
        pfn_user_callback: Some(vulkan_debug_utils_callback),
        p_user_data: ptr::null_mut(),
    }
}
