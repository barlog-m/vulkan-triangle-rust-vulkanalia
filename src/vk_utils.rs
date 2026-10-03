use vulkanalia::prelude::v1_3::*;
use vulkanalia::vk::HasBuilder;
use vulkanalia_vma::Alloc;

use crate::memory_guard::MappedMemory;
use crate::rndr::MAX_FRAMES_IN_FLIGHT;
use crate::vk_models::Vertex;

pub fn vk_create_image(
    allocator: &vulkanalia_vma::Allocator,
    width: u32,
    height: u32,
    format: vk::Format,
    tiling: vk::ImageTiling,
    usage: vk::ImageUsageFlags,
    msaa_samples: vk::SampleCountFlags,
    mip_levels: u32,
) -> (vk::Image, vulkanalia_vma::Allocation) {
    let extent = vk::Extent3D {
        width,
        height,
        depth: 1,
    };

    let image_create_info = vk::ImageCreateInfo::builder()
        .image_type(vk::ImageType::_2D)
        .format(format)
        .extent(extent)
        .mip_levels(mip_levels)
        .array_layers(1)
        .samples(msaa_samples)
        .tiling(tiling)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let image_allocation_create_info = vulkanalia_vma::AllocationOptions {
        usage: vulkanalia_vma::MemoryUsage::AutoPreferDevice,
        ..Default::default()
    };

    unsafe { allocator.create_image(image_create_info, &image_allocation_create_info) }
        .expect("Failed to create image")
}

pub fn vk_create_image_view(
    device: &Device,
    image: &vk::Image,
    image_format: &vk::Format,
    aspect_flags: vk::ImageAspectFlags,
    mip_levels: u32,
) -> vk::ImageView {
    let image_view_info = vk::ImageViewCreateInfo::builder()
        .subresource_range(
            vk::ImageSubresourceRange::builder()
                .aspect_mask(aspect_flags)
                .level_count(mip_levels)
                .layer_count(1),
        )
        .image(*image)
        .format(*image_format)
        .view_type(vk::ImageViewType::_2D);

    unsafe { device.create_image_view(&image_view_info, None) }.expect("Failed to create image view")
}

pub fn vk_create_buffer(
    allocator: &vulkanalia_vma::Allocator,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    memory_usage: vulkanalia_vma::MemoryUsage,
    alloc_flags: vulkanalia_vma::AllocationCreateFlags,
) -> (vk::Buffer, vulkanalia_vma::Allocation) {
    let buffer_create_info = vk::BufferCreateInfo::builder()
        .size(size)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let alloc_create_info = vulkanalia_vma::AllocationOptions {
        flags: alloc_flags,
        usage: memory_usage,
        ..Default::default()
    };

    let (buffer, allocation) =
        unsafe { allocator.create_buffer(buffer_create_info, &alloc_create_info) }.expect("Failed to create buffer");

    (buffer, allocation)
}

pub fn vk_copy_buffer(
    device: &Device,
    queue: &vk::Queue,
    command_pool: &vk::CommandPool,
    src_buffer: &vk::Buffer,
    dst_buffer: &vk::Buffer,
    size: vk::DeviceSize,
) {
    let alloc_info = vk::CommandBufferAllocateInfo::builder()
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_pool(*command_pool)
        .command_buffer_count(1);

    let command_buffer =
        unsafe { device.allocate_command_buffers(&alloc_info) }.expect("Failed to allocate command buffer")[0];

    let begin_info = vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

    unsafe {
        device
            .begin_command_buffer(command_buffer, &begin_info)
            .expect("Begin commandbuffer failed");

        let copy_region = vk::BufferCopy::builder().size(size);
        device.cmd_copy_buffer(command_buffer, *src_buffer, *dst_buffer, &[copy_region]);

        device
            .end_command_buffer(command_buffer)
            .expect("End commandbuffer failed");

        let command_buffer_infos = [vk::CommandBufferSubmitInfo::builder().command_buffer(command_buffer)];
        let submit_info = vk::SubmitInfo2::builder().command_buffer_infos(&command_buffer_infos);

        device
            .queue_submit2(*queue, &[submit_info], vk::Fence::null())
            .expect("Queue submit failed");
        device.queue_wait_idle(*queue).expect("Queue wait idle failed");

        device.free_command_buffers(*command_pool, &[command_buffer]);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn vk_transition_image_layout(
    device: &Device,
    command_buffer: &vk::CommandBuffer,
    image: &vk::Image,
    old_layout: &vk::ImageLayout,
    new_layout: &vk::ImageLayout,
    src_access_mask: &vk::AccessFlags2,
    dst_access_mask: &vk::AccessFlags2,
    src_stage_mask: &vk::PipelineStageFlags2,
    dst_stage_mask: &vk::PipelineStageFlags2,
    image_aspect_flags: &vk::ImageAspectFlags,
    mip_levels: u32,
) {
    let subresource_range = vk::ImageSubresourceRange::builder()
        .aspect_mask(*image_aspect_flags)
        .base_mip_level(0)
        .level_count(mip_levels)
        .base_array_layer(0)
        .layer_count(1);

    let barrier = vk::ImageMemoryBarrier2::builder()
        .src_stage_mask(*src_stage_mask)
        .src_access_mask(*src_access_mask)
        .dst_stage_mask(*dst_stage_mask)
        .dst_access_mask(*dst_access_mask)
        .old_layout(*old_layout)
        .new_layout(*new_layout)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .image(*image)
        .subresource_range(subresource_range);

    let dependency_info = vk::DependencyInfo::builder().image_memory_barriers(std::slice::from_ref(&barrier));

    unsafe {
        device.cmd_pipeline_barrier2(*command_buffer, &dependency_info);
    }
}

pub fn vk_create_vertex_buffer(
    device: &Device,
    allocator: &vulkanalia_vma::Allocator,
    queue: &vk::Queue,
    command_pool: &vk::CommandPool,
    vertices: &[Vertex],
) -> (vk::Buffer, vulkanalia_vma::Allocation, u32) {
    let buffer_size = size_of_val(vertices) as vk::DeviceSize;

    let (staging_buffer, staging_alloc) = vk_create_buffer(
        allocator,
        buffer_size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vulkanalia_vma::MemoryUsage::Auto,
        vulkanalia_vma::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE,
    );

    {
        let mapped = MappedMemory::new(allocator, staging_alloc);
        mapped.as_slice_mut::<Vertex>(vertices.len()).copy_from_slice(vertices);
    }

    let (vertex_buffer, vertex_buffer_alloc) = vk_create_buffer(
        allocator,
        buffer_size,
        vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
        vulkanalia_vma::MemoryUsage::AutoPreferDevice,
        vulkanalia_vma::AllocationCreateFlags::empty(),
    );

    vk_copy_buffer(
        device,
        queue,
        command_pool,
        &staging_buffer,
        &vertex_buffer,
        buffer_size,
    );

    unsafe {
        allocator.destroy_buffer(staging_buffer, staging_alloc);
    }

    (vertex_buffer, vertex_buffer_alloc, vertices.len() as u32)
}

pub fn vk_create_index_buffer(
    device: &Device,
    allocator: &vulkanalia_vma::Allocator,
    queue: &vk::Queue,
    command_pool: &vk::CommandPool,
    indices: &[u32],
) -> (vk::Buffer, vulkanalia_vma::Allocation, u32) {
    let buffer_size = size_of_val(indices) as vk::DeviceSize;

    let (staging_buffer, staging_alloc) = vk_create_buffer(
        allocator,
        buffer_size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vulkanalia_vma::MemoryUsage::Auto,
        vulkanalia_vma::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE,
    );

    {
        let mapped = MappedMemory::new(allocator, staging_alloc);
        mapped.as_slice_mut::<u32>(indices.len()).copy_from_slice(indices);
    }

    let (index_buffer, index_buffer_alloc) = vk_create_buffer(
        allocator,
        buffer_size,
        vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
        vulkanalia_vma::MemoryUsage::AutoPreferDevice,
        vulkanalia_vma::AllocationCreateFlags::empty(),
    );

    vk_copy_buffer(device, queue, command_pool, &staging_buffer, &index_buffer, buffer_size);

    unsafe {
        allocator.destroy_buffer(staging_buffer, staging_alloc);
    }

    (index_buffer, index_buffer_alloc, indices.len() as u32)
}

pub fn vk_prepare_image_layouts(
    device: &Device,
    queue: &vk::Queue,
    command_pool: &vk::CommandPool,
    color_image: &vk::Image,
    depth_image: &vk::Image,
) {
    let alloc_info = vk::CommandBufferAllocateInfo::builder()
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_pool(*command_pool)
        .command_buffer_count(1);

    let command_buffer =
        unsafe { device.allocate_command_buffers(&alloc_info) }.expect("Failed to allocate command buffer")[0];

    let begin_info = vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

    unsafe { device.begin_command_buffer(command_buffer, &begin_info) }.expect("Begin commandbuffer failed");

    vk_transition_image_layout(
        device,
        &command_buffer,
        color_image,
        &vk::ImageLayout::UNDEFINED,
        &vk::ImageLayout::GENERAL,
        &vk::AccessFlags2::empty(),
        &vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
        &vk::PipelineStageFlags2::TOP_OF_PIPE,
        &vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
        &vk::ImageAspectFlags::COLOR,
        1,
    );

    vk_transition_image_layout(
        device,
        &command_buffer,
        depth_image,
        &vk::ImageLayout::UNDEFINED,
        &vk::ImageLayout::GENERAL,
        &vk::AccessFlags2::empty(),
        &vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE,
        &vk::PipelineStageFlags2::TOP_OF_PIPE,
        &(vk::PipelineStageFlags2::EARLY_FRAGMENT_TESTS | vk::PipelineStageFlags2::LATE_FRAGMENT_TESTS),
        &vk::ImageAspectFlags::DEPTH,
        1,
    );

    unsafe { device.end_command_buffer(command_buffer) }.expect("End commandbuffer failed");

    let command_buffer_infos = [vk::CommandBufferSubmitInfo::builder().command_buffer(command_buffer)];
    let submit_info = vk::SubmitInfo2::builder().command_buffer_infos(&command_buffer_infos);

    unsafe {
        device
            .queue_submit2(*queue, &[submit_info], vk::Fence::null())
            .expect("Queue submit failed");
        device.queue_wait_idle(*queue).expect("Queue wait idle failed");
        device.free_command_buffers(*command_pool, &[command_buffer]);
    }
}

pub fn vk_create_sync_objects(
    device: &Device,
    swapchain_images: &[vk::Image],
) -> ([vk::Semaphore; MAX_FRAMES_IN_FLIGHT], Vec<vk::Semaphore>, vk::Semaphore) {
    let semaphore_create_info = vk::SemaphoreCreateInfo::builder();

    let present_complete_semaphores: [vk::Semaphore; MAX_FRAMES_IN_FLIGHT] = std::array::from_fn(|_| {
        unsafe { device.create_semaphore(&semaphore_create_info, None) }.expect("Create semaphore failed")
    });
    let render_finished_semaphores: Vec<vk::Semaphore> = (0..swapchain_images.len())
        .map(|_| unsafe { device.create_semaphore(&semaphore_create_info, None) }.expect("Create semaphore failed"))
        .collect();

    let mut timeline_type_info = vk::SemaphoreTypeCreateInfo::builder()
        .semaphore_type(vk::SemaphoreType::TIMELINE)
        .initial_value(0);
    let timeline_semaphore_info = vk::SemaphoreCreateInfo::builder().push_next(&mut timeline_type_info);
    let timeline_semaphore = unsafe { device.create_semaphore(&timeline_semaphore_info, None) }
        .expect("Create timeline semaphore failed");

    (present_complete_semaphores, render_finished_semaphores, timeline_semaphore)
}
