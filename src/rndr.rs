use std::mem::ManuallyDrop;

use ash::ext::debug_utils;
use ash::khr::{surface, swapchain};
use ash::vk;

use crate::mesh::Mesh;
use crate::vk_debug::vk_enable_debug;
use crate::vk_init_core::{vk_create_instance, vk_create_logical_device, vk_create_surface, vk_pick_physical_device};
use crate::vk_init_rndr::{
    vk_create_color_resources, vk_create_command_buffers, vk_create_command_pool, vk_create_depth_resources,
    vk_create_descriptor_set_layout, vk_find_depth_format, vk_create_graphics_pipeline, vk_query_swapchain_support,
    vk_create_swap_chain_image_views, vk_create_swap_chain,
};
use crate::vk_models::{VkQueueFamilyIndices, VkSwapchainSupportDetails};
use crate::vk_utils::{vk_create_sync_objects, vk_prepare_image_layouts, vk_transition_image_layout};
use crate::window::Window;

pub const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub struct Rndr {
    _entry: ash::Entry,
    pub instance: ash::Instance,
    pub device: ash::Device,
    pub allocator: ManuallyDrop<vk_mem::Allocator>,

    pub surface_loader: surface::Instance,
    pub surface: vk::SurfaceKHR,

    #[cfg(debug_assertions)]
    pub debug_utils_loader: debug_utils::Instance,
    #[cfg(debug_assertions)]
    pub debug_call_back: vk::DebugUtilsMessengerEXT,

    pub physical_device: vk::PhysicalDevice,
    pub queue_family_indices: VkQueueFamilyIndices,
    pub queue: vk::Queue,
    pub compute_queue: vk::Queue,
    pub msaa_samples: vk::SampleCountFlags,

    pub swapchain_support_details: VkSwapchainSupportDetails,
    pub swapchain_surface_format: vk::SurfaceFormatKHR,
    pub swapchain_surface_extent: vk::Extent2D,
    pub swapchain_image_count: u32,
    pub swapchain_loader: swapchain::Device,
    pub swapchain: vk::SwapchainKHR,

    pub swapchain_images: Vec<vk::Image>,
    pub swapchain_image_views: Vec<vk::ImageView>,

    pub color_format: vk::Format,
    pub color_image: vk::Image,
    pub color_image_view: vk::ImageView,
    pub color_image_allocation: vk_mem::Allocation,

    pub depth_format: vk::Format,
    pub depth_image: vk::Image,
    pub depth_image_view: vk::ImageView,
    pub depth_image_allocation: vk_mem::Allocation,

    pub descriptor_set_layout: vk::DescriptorSetLayout,

    pub command_pool: vk::CommandPool,
    pub command_buffers: [vk::CommandBuffer; MAX_FRAMES_IN_FLIGHT],

    pub present_complete_semaphores: [vk::Semaphore; MAX_FRAMES_IN_FLIGHT],
    pub render_finished_semaphores: Vec<vk::Semaphore>,

    pub timeline_semaphore: vk::Semaphore,

    pub pipeline_layout: vk::PipelineLayout,
    pub graphic_pipeline: vk::Pipeline,

    pub shader_module: vk::ShaderModule,

    pub viewports: [vk::Viewport; 1],
    pub scissors: [vk::Rect2D; 1],

    pub frame_index: usize,
    pub frame_number: u64,
}

impl Rndr {
    pub fn new(window: &Window) -> Self {
        let entry = ash::Entry::linked();
        let instance = vk_create_instance(&entry);

        #[cfg(debug_assertions)]
        let (debug_utils_loader, debug_call_back) = vk_enable_debug(&entry, &instance);

        let (surface_loader, surface) = vk_create_surface(&entry, &instance, &window.raw());

        let (physical_device, queue_family_indices, queue_family_index, msaa_samples) =
            vk_pick_physical_device(&instance, &surface_loader, &surface);

        let (device, queue, compute_queue) =
            vk_create_logical_device(&instance, &physical_device, &queue_family_indices);

        let allocator = unsafe {
            vk_mem::Allocator::new(vk_mem::AllocatorCreateInfo::new(
                &instance,
                &device,
                physical_device.clone(),
            ))
        }
        .expect("Failed to create allocator");

        let depth_format = vk_find_depth_format(&instance, &physical_device);

        let window_width = window.width;
        let window_height = window.height;

        let (swapchain_support_details, swapchain_surface_format, swapchain_surface_extent, swapchain_image_count) =
            vk_query_swapchain_support(&physical_device, &surface, &surface_loader, window_width, window_height);

        let color_format = swapchain_surface_format.format;

        let (swapchain_loader, swapchain, swapchain_images) = vk_create_swap_chain(
            &instance,
            &device,
            &surface,
            &swapchain_support_details,
            &swapchain_surface_format,
            &swapchain_surface_extent,
            swapchain_image_count,
            vk::SwapchainKHR::null(),
        );

        let swapchain_image_views =
            vk_create_swap_chain_image_views(&device, swapchain_surface_format, &swapchain_images);

        let (color_image, color_image_allocation, color_image_view) = vk_create_color_resources(
            &device,
            &allocator,
            &swapchain_surface_extent,
            &color_format,
            msaa_samples,
        );

        let (depth_image, depth_image_allocation, depth_image_view) = vk_create_depth_resources(
            &device,
            &allocator,
            &swapchain_surface_extent,
            &depth_format,
            msaa_samples,
        );

        let descriptor_set_layout = vk_create_descriptor_set_layout(&device);

        let (graphic_pipeline, pipeline_layout, shader_module, viewports, scissors) = vk_create_graphics_pipeline(
            &device,
            &swapchain_surface_extent,
            &msaa_samples,
            swapchain_surface_format.format,
            depth_format,
            &descriptor_set_layout,
        );

        let command_pool = vk_create_command_pool(&device, queue_family_index);

        let command_buffers = vk_create_command_buffers(&device, &command_pool);

        vk_prepare_image_layouts(&device, &queue, &command_pool, &color_image, &depth_image);

        let (present_complete_semaphores, render_finished_semaphores, timeline_semaphore) = 
            vk_create_sync_objects(&device, &swapchain_images);

        Self {
            _entry: entry,
            instance,
            device,
            allocator: ManuallyDrop::new(allocator),

            surface_loader,
            surface,

            #[cfg(debug_assertions)]
            debug_utils_loader,
            #[cfg(debug_assertions)]
            debug_call_back,

            physical_device,
            queue_family_indices,
            queue,
            compute_queue,
            msaa_samples,

            swapchain_support_details,
            swapchain_surface_format,
            swapchain_surface_extent,
            swapchain_image_count,
            swapchain_loader,
            swapchain,

            swapchain_images,
            swapchain_image_views,

            color_format,
            color_image,
            color_image_view,
            color_image_allocation,

            depth_format,
            depth_image,
            depth_image_view,
            depth_image_allocation,

            descriptor_set_layout,
            command_pool,

            command_buffers,

            present_complete_semaphores,
            render_finished_semaphores,
            timeline_semaphore,

            pipeline_layout,
            graphic_pipeline,

            shader_module,

            viewports,
            scissors,

            frame_index: 0,
            frame_number: 0,
        }
    }

    pub fn draw_frame(&mut self, mesh: &Mesh, window: &Window) {
        if window.is_zero_size() {
            return;
        }
        
        let frame_index = self.frame_index;

        let wait_value = self.frame_number.saturating_sub(MAX_FRAMES_IN_FLIGHT as u64 - 1);
        let wait_semaphores = [self.timeline_semaphore];
        let wait_values = [wait_value];
        let timeline_wait_info = vk::SemaphoreWaitInfo::default()
            .semaphores(&wait_semaphores)
            .values(&wait_values);
        unsafe { self.device.wait_semaphores(&timeline_wait_info, u64::MAX) }.expect("Wait timeline failed");

        let window_width = window.width;
        let window_height = window.height;

        let mut needs_recreate =
            window_width != self.swapchain_surface_extent.width
            || window_height != self.swapchain_surface_extent.height;

        // Note: in_flight_fences, present_complete_semaphore, and command_buffers are indexed by frame_index,
        //       while render_finished_semaphores is indexed by image_index
        let present_complete_semaphore = self.present_complete_semaphores[frame_index];

        let acquire_result = unsafe {
            self.swapchain_loader.acquire_next_image(
                self.swapchain,
                u64::MAX,
                present_complete_semaphore,
                vk::Fence::null(),
            )
        };

        let (image_index, acquire_suboptimal) = match acquire_result {
            Ok((index, suboptimal)) => (index, suboptimal),
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                self.recreate_swapchain(window);
                return;
            }
            Err(vk::Result::NOT_READY | vk::Result::TIMEOUT) => return,
            Err(e) => panic!("Acquire next swapchain image failed: {}", e),
        };

        needs_recreate |= acquire_suboptimal;

        let command_buffer = self.command_buffers[frame_index];

        self.record_command_buffer(command_buffer, image_index, mesh);

        // Submit
        let signal_value = self.frame_number + 1;

        let wait_semaphore_infos = [
            vk::SemaphoreSubmitInfo::default()
                .semaphore(present_complete_semaphore)
                .stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT),
            vk::SemaphoreSubmitInfo::default()
                .semaphore(self.timeline_semaphore)
                .value(self.frame_number)
                .stage_mask(vk::PipelineStageFlags2::TOP_OF_PIPE),
        ];
        let command_buffer_infos = [vk::CommandBufferSubmitInfo::default().command_buffer(command_buffer)];
        let signal_semaphore_infos = [
            vk::SemaphoreSubmitInfo::default()
                .semaphore(self.render_finished_semaphores[image_index as usize])
                .stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT),
            vk::SemaphoreSubmitInfo::default()
                .semaphore(self.timeline_semaphore)
                .value(signal_value)
                .stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT),
        ];
        let submit_info = vk::SubmitInfo2::default()
            .wait_semaphore_infos(&wait_semaphore_infos)
            .command_buffer_infos(&command_buffer_infos)
            .signal_semaphore_infos(&signal_semaphore_infos);

        unsafe {
            self.device.queue_submit2(self.queue, &[submit_info], vk::Fence::null())
        }
        .expect("Queue submit failed");

        // Present
        let wait_semaphores = [self.render_finished_semaphores[image_index as usize]];
        let swapchains = [self.swapchain];
        let image_indices = [image_index];
        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(&wait_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        let present_result = unsafe { self.swapchain_loader.queue_present(self.queue, &present_info) };

        match present_result {
            Ok(suboptimal) => needs_recreate |= suboptimal,
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => needs_recreate = true,
            Err(e) => panic!("Queue present failed: {}", e),
        }

        if needs_recreate {
            self.recreate_swapchain(&window);
        }

        self.frame_number += 1;
        self.frame_index = (self.frame_index + 1) % MAX_FRAMES_IN_FLIGHT;
    }

    fn record_command_buffer(&self, command_buffer: vk::CommandBuffer, image_index: u32, mesh: &Mesh) {
        unsafe {
            self.device
                .reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::RELEASE_RESOURCES)
                .expect("Reset command buffer failed");

            let begin_info = vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
            self.device
                .begin_command_buffer(command_buffer, &begin_info)
                .expect("Begin command buffer failed");
        }

        let swapchain_image = self.swapchain_images[image_index as usize];
        let swapchain_image_view = self.swapchain_image_views[image_index as usize];

        vk_transition_image_layout(
            &self.device,
            &command_buffer,
            &swapchain_image,
            &vk::ImageLayout::UNDEFINED,
            &vk::ImageLayout::GENERAL,
            &vk::AccessFlags2::empty(),
            &vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
            &vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
            &vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
            &vk::ImageAspectFlags::COLOR,
            1,
        );

        let color_clear_value = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        };
        let depth_clear_value = vk::ClearValue {
            depth_stencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 },
        };

        let color_attachment_infos = [vk::RenderingAttachmentInfo::default()
            .image_view(self.color_image_view)
            .image_layout(vk::ImageLayout::GENERAL)
            .resolve_mode(vk::ResolveModeFlags::AVERAGE)
            .resolve_image_view(swapchain_image_view)
            .resolve_image_layout(vk::ImageLayout::GENERAL)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::DONT_CARE)
            .clear_value(color_clear_value)];

        let depth_attachment_info = vk::RenderingAttachmentInfo::default()
            .image_view(self.depth_image_view)
            .image_layout(vk::ImageLayout::GENERAL)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::DONT_CARE)
            .clear_value(depth_clear_value);

        let rendering_info = vk::RenderingInfo::default()
            .render_area(self.swapchain_surface_extent.into())
            .layer_count(1)
            .color_attachments(&color_attachment_infos)
            .depth_attachment(&depth_attachment_info);

        unsafe {
            self.device.cmd_begin_rendering(command_buffer, &rendering_info);
            self.device
                .cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, self.graphic_pipeline);
            self.device.cmd_set_viewport(command_buffer, 0, &self.viewports);
            self.device.cmd_set_scissor(command_buffer, 0, &self.scissors);
            self.device
                .cmd_bind_vertex_buffers(command_buffer, 0, &[mesh.vertex_buffer], &[0]);
            self.device
                .cmd_bind_index_buffer(command_buffer, mesh.index_buffer, 0, vk::IndexType::UINT32);
            self.device
                .cmd_draw_indexed(command_buffer, mesh.indices_count, 1, 0, 0, 0);
            self.device.cmd_end_rendering(command_buffer);
        }

        vk_transition_image_layout(
            &self.device,
            &command_buffer,
            &swapchain_image,
            &vk::ImageLayout::GENERAL,
            &vk::ImageLayout::PRESENT_SRC_KHR,
            &vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
            &vk::AccessFlags2::empty(),
            &vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
            &vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
            &vk::ImageAspectFlags::COLOR,
            1,
        );

        unsafe { self.device.end_command_buffer(command_buffer) }.expect("End command buffer failed");
    }

    pub fn apply_window_size(&mut self, window: &Window) {
        let window_width = window.width;
        let window_height = window.height;

        let (swapchain_support_details, swapchain_surface_format, swapchain_surface_extent, swapchain_image_count) =
            vk_query_swapchain_support(
                &self.physical_device,
                &self.surface,
                &self.surface_loader,
                window_width,
                window_height,
            );

        self.swapchain_support_details = swapchain_support_details;
        self.swapchain_surface_format = swapchain_surface_format;
        self.swapchain_surface_extent = swapchain_surface_extent;
        self.swapchain_image_count = swapchain_image_count;
        self.color_format = swapchain_surface_format.format;

        self.viewports[0].width = swapchain_surface_extent.width as f32;
        self.viewports[0].height = swapchain_surface_extent.height as f32;
        self.scissors[0].extent = swapchain_surface_extent;
    }

    pub fn recreate_swapchain(&mut self, window: &Window) {
        let _ = unsafe { self.device.device_wait_idle() };

        self.apply_window_size(window);
        let old_swapchain = self.swapchain;

        self.clean_up_swap_chain();

        let (swapchain_loader, swapchain, swapchain_images) = vk_create_swap_chain(
            &self.instance,
            &self.device,
            &self.surface,
            &self.swapchain_support_details,
            &self.swapchain_surface_format,
            &self.swapchain_surface_extent,
            self.swapchain_image_count,
            old_swapchain,
        );

        unsafe {
            self.swapchain_loader.destroy_swapchain(old_swapchain, None);
        }

        self.swapchain_loader = swapchain_loader;
        self.swapchain = swapchain;
        self.swapchain_images = swapchain_images;

        self.swapchain_image_views =
            vk_create_swap_chain_image_views(&self.device, self.swapchain_surface_format, &self.swapchain_images);

        if self.render_finished_semaphores.len() != self.swapchain_images.len() {
            unsafe {
                for &s in &self.render_finished_semaphores {
                    self.device.destroy_semaphore(s, None);
                }
            }
            let info = vk::SemaphoreCreateInfo::default();
            self.render_finished_semaphores = (0..self.swapchain_images.len())
                .map(|_| unsafe { self.device.create_semaphore(&info, None) }.expect("Create semaphore failed"))
                .collect();
        }

        let (color_image, color_image_allocation, color_image_view) = vk_create_color_resources(
            &self.device,
            &self.allocator,
            &self.swapchain_surface_extent,
            &self.color_format,
            self.msaa_samples,
        );
        self.color_image = color_image;
        self.color_image_allocation = color_image_allocation;
        self.color_image_view = color_image_view;

        let (depth_image, depth_image_allocation, depth_image_view) = vk_create_depth_resources(
            &self.device,
            &self.allocator,
            &self.swapchain_surface_extent,
            &self.depth_format,
            self.msaa_samples,
        );
        self.depth_image = depth_image;
        self.depth_image_allocation = depth_image_allocation;
        self.depth_image_view = depth_image_view;

        vk_prepare_image_layouts(
            &self.device,
            &self.queue,
            &self.command_pool,
            &color_image,
            &depth_image,
        );
    }

    fn clean_up_swap_chain(&mut self) {
        unsafe {
            self.device.destroy_image_view(self.depth_image_view, None);
            self.allocator
                .destroy_image(self.depth_image, &mut self.depth_image_allocation);

            self.device.destroy_image_view(self.color_image_view, None);
            self.allocator
                .destroy_image(self.color_image, &mut self.color_image_allocation);

            for &swapchain_image_view in &self.swapchain_image_views {
                self.device.destroy_image_view(swapchain_image_view, None);
            }
            self.swapchain_images.clear();
        }
    }

    pub fn destroy(&mut self) {
        unsafe {
            let _ = self.device.device_wait_idle();
            self.device.destroy_pipeline(self.graphic_pipeline, None);
            self.device.destroy_pipeline_layout(self.pipeline_layout, None);
            self.device.destroy_shader_module(self.shader_module, None);
            self.device
                .destroy_descriptor_set_layout(self.descriptor_set_layout, None);

            for &semaphore in &self.present_complete_semaphores {
                self.device.destroy_semaphore(semaphore, None);
            }
            for &semaphore in &self.render_finished_semaphores {
                self.device.destroy_semaphore(semaphore, None);
            }
            self.device.destroy_semaphore(self.timeline_semaphore, None);

            self.clean_up_swap_chain();

            self.device.destroy_command_pool(self.command_pool, None);
            self.swapchain_loader.destroy_swapchain(self.swapchain, None);

            ManuallyDrop::drop(&mut self.allocator);

            self.device.destroy_device(None);
            self.surface_loader.destroy_surface(self.surface, None);

            #[cfg(debug_assertions)]
            self.debug_utils_loader
                .destroy_debug_utils_messenger(self.debug_call_back, None);

            self.instance.destroy_instance(None);
        }
    }
}
