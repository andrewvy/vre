use std::collections::HashSet;

use ash::{extensions::khr::Swapchain, version::InstanceV1_0, vk, Device, Instance};

use crate::utils;

use super::{QueueFamilyIndices, SurfaceBundle};

pub fn check_device_extension_support(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
) -> bool {
    let available_extensions = unsafe {
        instance
            .enumerate_device_extension_properties(physical_device)
            .expect("Failed to get device extension properties.")
    };

    let mut available_extension_names = vec![];

    for extension in available_extensions.iter() {
        let extension_name = utils::vk_to_string(&extension.extension_name);
        available_extension_names.push(extension_name);
    }

    let mut required_extensions = HashSet::new();
    for extension in super::REQUIRED_DEVICE_EXTENSIONS.iter() {
        required_extensions.insert(extension.to_string());
    }

    for extension_name in available_extension_names.iter() {
        required_extensions.remove(extension_name);
    }

    return required_extensions.is_empty();
}

pub fn create_logical_device(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    surface_bundle: &SurfaceBundle,
) -> (Device, QueueFamilyIndices) {
    let indices = find_queue_family(instance, physical_device, surface_bundle);
    let priorities = [1.0];
    let enabled_extension_names = [Swapchain::name().as_ptr()];
    let queue_info = vk::DeviceQueueCreateInfo::builder()
        .queue_family_index(indices.graphics_family.unwrap())
        .queue_priorities(&priorities)
        .build();
    let device_create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&[queue_info])
        .enabled_extension_names(&enabled_extension_names)
        .build();

    let device = unsafe {
        instance
            .create_device(physical_device, &device_create_info, None)
            .expect("Could not create Vulkan Device.")
    };

    (device, indices)
}

pub fn find_queue_family(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    surface_bundle: &SurfaceBundle,
) -> QueueFamilyIndices {
    let queue_families =
        unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

    let mut indices = QueueFamilyIndices {
        graphics_family: None,
        present_family: None,
    };

    let mut index = 0;
    for queue_family in queue_families.iter() {
        if queue_family.queue_count > 0
            && queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
        {
            indices.graphics_family = Some(index);
        }

        let has_present_support = unsafe {
            surface_bundle
                .surface_loader
                .get_physical_device_surface_support(physical_device, index, surface_bundle.surface)
                .expect("Could not check physical device for surface support.")
        };

        if queue_family.queue_count > 0 && has_present_support {
            indices.present_family = Some(index);
        }

        if indices.is_complete() {
            break;
        }

        index += 1;
    }

    indices
}
