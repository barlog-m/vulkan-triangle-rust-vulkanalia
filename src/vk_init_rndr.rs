use vulkanalia::bytecode::Bytecode;
use vulkanalia::prelude::v1_3::*;
use vulkanalia::vk::{HasBuilder, KhrSurfaceExtensionInstanceCommands, KhrSwapchainExtensionDeviceCommands};

use crate::rndr::MAX_FRAMES_IN_FLIGHT;
use crate::vk_models::{Vertex, VkSwapchainSupportDetails};
use crate::vk_utils::{vk_create_image, vk_create_image_view};

fn vk_choose_swap_surface_format(available_formats: &[vk::SurfaceFormatKHR]) -> vk::SurfaceFormatKHR {
    for format in available_formats {
        if format.format == vk::Format::B8G8R8A8_SRGB && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR {
            return *format;
        }
    }

    available_formats[0]
}

pub fn vk_query_swapchain_support(
    physical_device: &vk::PhysicalDevice,
    surface: &vk::SurfaceKHR,
    instance: &Instance,
    window_width: u32,
    window_height: u32,
) -> (VkSwapchainSupportDetails, vk::SurfaceFormatKHR, vk::Extent2D, u32) {
    let surface_formats = unsafe { instance.get_physical_device_surface_formats_khr(*physical_device, *surface) }
        .expect("Failed to get physical device surface formats");

    let surface_format = vk_choose_swap_surface_format(&surface_formats);

    let surface_capabilities =
        unsafe { instance.get_physical_device_surface_capabilities_khr(*physical_device, *surface) }
            .expect("Failed to get physical device surface capabilities");

    let surface_extent = if surface_capabilities.current_extent.width != u32::MAX {
        surface_capabilities.current_extent
    } else {
        let min = surface_capabilities.min_image_extent;
        let max = surface_capabilities.max_image_extent;

        vk::Extent2D {
            width: window_width.clamp(min.width, max.width),
            height: window_height.clamp(min.height, max.height),
        }
    };

    let present_modes =
        unsafe { instance.get_physical_device_surface_present_modes_khr(*physical_device, *surface) }
            .expect("Failed to get physical device present modes");

    let present_mode = present_modes
        .iter()
        .cloned()
        .find(|&mode| mode == vk::PresentModeKHR::MAILBOX)
        .unwrap_or(vk::PresentModeKHR::FIFO);

    let mut image_count = surface_capabilities.min_image_count + 1;
    if surface_capabilities.max_image_count > 0 && image_count > surface_capabilities.max_image_count {
        image_count = surface_capabilities.max_image_count;
    }

    (
        VkSwapchainSupportDetails {
            surface_formats,
            surface_capabilities,
            present_modes,
            present_mode,
        },
        surface_format,
        surface_extent,
        image_count,
    )
}

pub fn vk_create_swap_chain(
    device: &Device,
    surface: &vk::SurfaceKHR,
    swapchain_support_details: &VkSwapchainSupportDetails,
    surface_format: &vk::SurfaceFormatKHR,
    surface_extent: &vk::Extent2D,
    min_image_count: u32,
    old_swapchain: vk::SwapchainKHR,
) -> (vk::SwapchainKHR, Vec<vk::Image>) {
    let pre_transform = if swapchain_support_details
        .surface_capabilities
        .supported_transforms
        .contains(vk::SurfaceTransformFlagsKHR::IDENTITY)
    {
        vk::SurfaceTransformFlagsKHR::IDENTITY
    } else {
        swapchain_support_details.surface_capabilities.current_transform
    };

    let sharing_mode = vk::SharingMode::EXCLUSIVE;

    let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
        .surface(*surface)
        .min_image_count(min_image_count)
        .image_color_space(surface_format.color_space)
        .image_format(surface_format.format)
        .image_extent(*surface_extent)
        .image_usage(
            vk::ImageUsageFlags::COLOR_ATTACHMENT
                | vk::ImageUsageFlags::TRANSFER_DST
                | vk::ImageUsageFlags::TRANSFER_SRC,
        )
        .image_sharing_mode(sharing_mode)
        .pre_transform(pre_transform)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(swapchain_support_details.present_mode)
        .clipped(true)
        .image_array_layers(1)
        .old_swapchain(old_swapchain);

    let swapchain =
        unsafe { device.create_swapchain_khr(&swapchain_create_info, None) }.expect("Failed to create swapchain");

    let swapchain_images =
        unsafe { device.get_swapchain_images_khr(swapchain) }.expect("Failed to get swapchain images");

    (swapchain, swapchain_images)
}

pub fn vk_create_swap_chain_image_views(
    device: &Device,
    swapchain_surface_format: vk::SurfaceFormatKHR,
    swapchain_images: &[vk::Image],
) -> Vec<vk::ImageView> {
    let swapchain_image_views: Vec<vk::ImageView> = swapchain_images
        .iter()
        .map(|&image| {
            let create_view_info = vk::ImageViewCreateInfo::builder()
                .view_type(vk::ImageViewType::_2D)
                .format(swapchain_surface_format.format)
                .components(vk::ComponentMapping {
                    r: vk::ComponentSwizzle::R,
                    g: vk::ComponentSwizzle::G,
                    b: vk::ComponentSwizzle::B,
                    a: vk::ComponentSwizzle::A,
                })
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                })
                .image(image);
            unsafe { device.create_image_view(&create_view_info, None) }.expect("Failed to create image view")
        })
        .collect();
    swapchain_image_views
}

pub fn vk_create_descriptor_set_layout(device: &Device) -> vk::DescriptorSetLayout {
    let layout_bindings = [
        vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX),
        vk::DescriptorSetLayoutBinding::builder()
            .binding(1)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT),
    ];

    let layout_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&layout_bindings);

    unsafe { device.create_descriptor_set_layout(&layout_info, None) }
        .expect("Failed to create descriptor set layout")
}

pub fn vk_create_graphics_pipeline(
    device: &Device,
    swapchain_surface_extent: &vk::Extent2D,
    msaa_samples: &vk::SampleCountFlags,
    color_format: vk::Format,
    depth_format: vk::Format,
    descriptor_set_layout: &vk::DescriptorSetLayout,
) -> (
    vk::Pipeline,
    vk::PipelineLayout,
    vk::ShaderModule,
    [vk::Viewport; 1],
    [vk::Rect2D; 1],
) {
    let shader_code = Bytecode::new(&include_bytes!("../shaders/shader.spv")[..])
        .expect("Failed to read shader spv file");
    let shader_info = vk::ShaderModuleCreateInfo::builder()
        .code_size(shader_code.code_size())
        .code(shader_code.code());

    let shader_module = unsafe { device.create_shader_module(&shader_info, None) }.expect("Shader module create error");

    let set_layouts = [*descriptor_set_layout];
    let layout_create_info = vk::PipelineLayoutCreateInfo::builder().set_layouts(&set_layouts);
    let pipeline_layout =
        unsafe { device.create_pipeline_layout(&layout_create_info, None) }.expect("Pipeline layout create error");

    let vertex_shader_stage_info = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::VERTEX)
        .module(shader_module)
        .name(b"vertMain\0");

    let fragment_shader_stage_info = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::FRAGMENT)
        .module(shader_module)
        .name(b"fragMain\0");

    let shader_stage_create_infos = [vertex_shader_stage_info, fragment_shader_stage_info];

    let vertex_input_binding_descriptions = Vertex::binding_description();

    let vertex_input_attribute_descriptions = Vertex::attribute_descriptions();

    let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo::builder()
        .vertex_binding_descriptions(&vertex_input_binding_descriptions)
        .vertex_attribute_descriptions(&vertex_input_attribute_descriptions);

    let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST);

    let viewports = [vk::Viewport {
        x: 0.0,
        y: 0.0,
        width: swapchain_surface_extent.width as f32,
        height: swapchain_surface_extent.height as f32,
        min_depth: 0.0,
        max_depth: 1.0,
    }];

    let scissors = [vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0 },
        extent: *swapchain_surface_extent,
    }];

    let viewport_state_info = vk::PipelineViewportStateCreateInfo::builder()
        .scissors(&scissors)
        .viewports(&viewports);

    let rasterization_info = vk::PipelineRasterizationStateCreateInfo::builder()
        .front_face(vk::FrontFace::CLOCKWISE)
        .line_width(1.0)
        .polygon_mode(vk::PolygonMode::FILL)
        .cull_mode(vk::CullModeFlags::BACK);

    let multisample_state_info = vk::PipelineMultisampleStateCreateInfo::builder()
        .rasterization_samples(*msaa_samples);

    let noop_stencil_state = vk::StencilOpState {
        fail_op: vk::StencilOp::KEEP,
        pass_op: vk::StencilOp::KEEP,
        depth_fail_op: vk::StencilOp::KEEP,
        compare_op: vk::CompareOp::ALWAYS,
        ..Default::default()
    };

    let depth_state_info = vk::PipelineDepthStencilStateCreateInfo::builder()
        .depth_test_enable(true)
        .depth_write_enable(true)
        .depth_compare_op(vk::CompareOp::LESS)
        .front(noop_stencil_state)
        .back(noop_stencil_state)
        .max_depth_bounds(1.0);

    let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState {
        blend_enable: vk::FALSE,
        src_color_blend_factor: vk::BlendFactor::SRC_COLOR,
        dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_DST_COLOR,
        color_blend_op: vk::BlendOp::ADD,
        src_alpha_blend_factor: vk::BlendFactor::ZERO,
        dst_alpha_blend_factor: vk::BlendFactor::ZERO,
        alpha_blend_op: vk::BlendOp::ADD,
        color_write_mask: vk::ColorComponentFlags::R
            | vk::ColorComponentFlags::G
            | vk::ColorComponentFlags::B
            | vk::ColorComponentFlags::A,
    }];

    let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
        .logic_op(vk::LogicOp::CLEAR)
        .attachments(&color_blend_attachment_states);

    let dynamic_state = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];

    let dynamic_state_info = vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&dynamic_state);

    let color_attachment_formats = [color_format];

    let mut pipeline_rendering_create_info = vk::PipelineRenderingCreateInfo::builder()
        .color_attachment_formats(&color_attachment_formats)
        .depth_attachment_format(depth_format)
        .stencil_attachment_format(vk::Format::UNDEFINED);

    let graphic_pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
        .stages(&shader_stage_create_infos)
        .vertex_input_state(&vertex_input_state_info)
        .input_assembly_state(&vertex_input_assembly_state_info)
        .viewport_state(&viewport_state_info)
        .rasterization_state(&rasterization_info)
        .multisample_state(&multisample_state_info)
        .depth_stencil_state(&depth_state_info)
        .color_blend_state(&color_blend_state)
        .dynamic_state(&dynamic_state_info)
        .layout(pipeline_layout)
        .push_next(&mut pipeline_rendering_create_info);

    let graphic_pipeline =
        unsafe { device.create_graphics_pipelines(vk::PipelineCache::null(), &[graphic_pipeline_info], None) }
            .expect("Unable to create graphics pipeline")
            .0[0];

    (graphic_pipeline, pipeline_layout, shader_module, viewports, scissors)
}

pub fn vk_find_supported_format(
    instance: &Instance,
    physical_device: &vk::PhysicalDevice,
    candidates: &[vk::Format],
    tiling: vk::ImageTiling,
    features: vk::FormatFeatureFlags,
) -> vk::Format {
    candidates
        .iter()
        .copied()
        .find(|&format| {
            let props = unsafe { instance.get_physical_device_format_properties(*physical_device, format) };
            (tiling == vk::ImageTiling::LINEAR && props.linear_tiling_features.contains(features))
                || (tiling == vk::ImageTiling::OPTIMAL && props.optimal_tiling_features.contains(features))
        })
        .expect("failed to find supported format")
}

pub fn vk_find_depth_format(instance: &Instance, physical_device: &vk::PhysicalDevice) -> vk::Format {
    let candidates = [
        vk::Format::D32_SFLOAT,
        vk::Format::D32_SFLOAT_S8_UINT,
        vk::Format::D24_UNORM_S8_UINT,
    ];

    let format = vk_find_supported_format(
        instance,
        physical_device,
        &candidates,
        vk::ImageTiling::OPTIMAL,
        vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
    );

    debug_assert!(
        matches!(format, vk::Format::D32_SFLOAT),
        "combined depth-stencil format not supported by this renderer"
    );

    format
}

pub fn vk_create_depth_resources(
    device: &Device,
    allocator: &vulkanalia_vma::Allocator,
    swapchain_surface_extent: &vk::Extent2D,
    depth_format: &vk::Format,
    msaa_samples: vk::SampleCountFlags,
) -> (vk::Image, vulkanalia_vma::Allocation, vk::ImageView) {
    let (depth_image, depth_image_allocation) = vk_create_image(
        allocator,
        swapchain_surface_extent.width,
        swapchain_surface_extent.height,
        *depth_format,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
        msaa_samples,
        1,
    );

    let depth_image_view = vk_create_image_view(device, &depth_image, depth_format, vk::ImageAspectFlags::DEPTH, 1);

    (depth_image, depth_image_allocation, depth_image_view)
}

pub fn vk_create_color_resources(
    device: &Device,
    allocator: &vulkanalia_vma::Allocator,
    swapchain_surface_extent: &vk::Extent2D,
    image_format: &vk::Format,
    msaa_samples: vk::SampleCountFlags,
) -> (vk::Image, vulkanalia_vma::Allocation, vk::ImageView) {
    let (color_image, color_image_allocation) = vk_create_image(
        allocator,
        swapchain_surface_extent.width,
        swapchain_surface_extent.height,
        *image_format,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::TRANSIENT_ATTACHMENT | vk::ImageUsageFlags::COLOR_ATTACHMENT,
        msaa_samples,
        1,
    );

    let color_image_view = vk_create_image_view(device, &color_image, image_format, vk::ImageAspectFlags::COLOR, 1);

    (color_image, color_image_allocation, color_image_view)
}

pub fn vk_create_command_pool(device: &Device, queue_family_index: u32) -> vk::CommandPool {
    let pool_create_info = vk::CommandPoolCreateInfo::builder()
        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
        .queue_family_index(queue_family_index);

    unsafe { device.create_command_pool(&pool_create_info, None) }.expect("Unable to create command pool")
}

pub fn vk_create_command_buffers(device: &Device, command_pool: &vk::CommandPool) -> [vk::CommandBuffer; MAX_FRAMES_IN_FLIGHT] {
    let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_buffer_count(MAX_FRAMES_IN_FLIGHT as u32)
        .command_pool(*command_pool)
        .level(vk::CommandBufferLevel::PRIMARY);

    let command_buffers = unsafe { device.allocate_command_buffers(&command_buffer_allocate_info) }
        .expect("Unable to allocate command buffers");

    command_buffers
        .try_into()
        .expect("Vulkan returned an unexpected number of command buffers")
}
