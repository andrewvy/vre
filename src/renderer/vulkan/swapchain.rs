use ash::{vk, Device, Instance};

use super::{QueueFamilyIndices, SurfaceBundle};

pub struct SwapchainSupportDetails {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

impl SwapchainSupportDetails {
    pub fn new(physical_device: vk::PhysicalDevice, surface_bundle: &SurfaceBundle) -> Self {
        let capabilities = unsafe {
            surface_bundle
                .surface_loader
                .get_physical_device_surface_capabilities(physical_device, surface_bundle.surface)
                .expect("Failed to get physical device surface capabilities.")
        };
        let formats = unsafe {
            surface_bundle
                .surface_loader
                .get_physical_device_surface_formats(physical_device, surface_bundle.surface)
                .expect("Failed to get physical device surface formats.")
        };
        let present_modes = unsafe {
            surface_bundle
                .surface_loader
                .get_physical_device_surface_present_modes(physical_device, surface_bundle.surface)
                .expect("Failed to get physical device surface present modes.")
        };

        Self {
            capabilities,
            formats,
            present_modes,
        }
    }

    fn choose_format(&self) -> vk::SurfaceFormatKHR {
        for available_format in self.formats.iter() {
            if available_format.format == vk::Format::B8G8R8A8_SRGB
                && available_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            {
                return available_format.clone();
            }
        }

        return self.formats.first().unwrap().clone();
    }

    fn choose_present_mode(&self) -> vk::PresentModeKHR {
        for available_present_mode in self.present_modes.iter() {
            if *available_present_mode == vk::PresentModeKHR::MAILBOX {
                return available_present_mode.clone();
            }
        }

        return vk::PresentModeKHR::FIFO;
    }

    fn choose_extent(&self) -> vk::Extent2D {
        if self.capabilities.current_extent.width != u32::max_value() {
            self.capabilities.current_extent
        } else {
            use num::clamp;

            vk::Extent2D::builder()
                .width(clamp(
                    crate::WINDOW_WIDTH,
                    self.capabilities.min_image_extent.width,
                    self.capabilities.max_image_extent.width,
                ))
                .height(clamp(
                    crate::WINDOW_HEIGHT,
                    self.capabilities.min_image_extent.height,
                    self.capabilities.max_image_extent.height,
                ))
                .build()
        }
    }
}

pub struct SwapchainBundle {
    pub swapchain_loader: ash::extensions::khr::Swapchain,
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_format: vk::Format,
    pub swapchain_extent: vk::Extent2D,
    pub swapchain_images: Vec<vk::Image>,
}

impl SwapchainBundle {
    pub fn new(
        instance: &Instance,
        device: &Device,
        physical_device: vk::PhysicalDevice,
        surface_bundle: &SurfaceBundle,
        queue_family: QueueFamilyIndices,
    ) -> Self {
        let swapchain_details = SwapchainSupportDetails::new(physical_device, surface_bundle);
        let surface_format = swapchain_details.choose_format();
        let present_mode = swapchain_details.choose_present_mode();
        let extent = swapchain_details.choose_extent();
        let desired_image_count = swapchain_details.capabilities.min_image_count + 1;
        let desired_image_count = if swapchain_details.capabilities.max_image_count > 0 {
            desired_image_count.min(swapchain_details.capabilities.max_image_count)
        } else {
            desired_image_count
        };

        let (image_sharing_mode, queue_family_indices) =
            if queue_family.graphics_family != queue_family.present_family {
                (
                    vk::SharingMode::EXCLUSIVE,
                    vec![
                        queue_family.graphics_family.unwrap(),
                        queue_family.present_family.unwrap(),
                    ],
                )
            } else {
                (vk::SharingMode::EXCLUSIVE, vec![])
            };

        let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface_bundle.surface)
            .min_image_count(desired_image_count)
            .image_color_space(surface_format.color_space)
            .image_format(surface_format.format)
            .image_extent(extent)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(image_sharing_mode)
            .pre_transform(swapchain_details.capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .image_array_layers(1)
            .queue_family_indices(&queue_family_indices)
            .build();

        let swapchain_loader = ash::extensions::khr::Swapchain::new(instance, device);
        let swapchain = unsafe {
            swapchain_loader
                .create_swapchain(&swapchain_create_info, None)
                .expect("Failed to create swapchain.")
        };
        let swapchain_images = unsafe {
            swapchain_loader
                .get_swapchain_images(swapchain)
                .expect("Failed to get swapchain images.")
        };

        Self {
            swapchain_loader,
            swapchain,
            swapchain_format: surface_format.format,
            swapchain_extent: extent,
            swapchain_images,
        }
    }
}
