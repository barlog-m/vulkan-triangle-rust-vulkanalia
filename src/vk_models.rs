use ash::vk;
use glam::f32::{Vec2, Vec3};

#[derive(Debug, Default)]
pub struct VkQueueFamilyIndices {
    pub graphics_family: Option<u32>,
    pub compute_family: Option<u32>,
    pub transfer_family: Option<u32>,
}

#[derive(Debug)]
pub struct VkSwapchainSupportDetails {
    pub surface_formats: Vec<vk::SurfaceFormatKHR>,
    pub surface_capabilities: vk::SurfaceCapabilitiesKHR,
    pub present_modes: Vec<vk::PresentModeKHR>,
    pub present_mode: vk::PresentModeKHR,
}

#[derive(Clone, Debug, Copy)]
pub struct Vertex {
    pub position: Vec2,
    pub color: Vec3,
}

impl Vertex {
    pub const fn binding_description() -> [vk::VertexInputBindingDescription; 1] {
        [vk::VertexInputBindingDescription {
            binding: 0,
            stride: size_of::<Self>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }]
    }

    pub const fn attribute_descriptions() -> [vk::VertexInputAttributeDescription; 2] {
        [
            vk::VertexInputAttributeDescription {
                location: 0,
                binding: 0,
                format: vk::Format::R32G32_SFLOAT,
                offset: std::mem::offset_of!(Self, position) as u32,
            },
            vk::VertexInputAttributeDescription {
                location: 1,
                binding: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: std::mem::offset_of!(Self, color) as u32,
            },
        ]
    }
}
