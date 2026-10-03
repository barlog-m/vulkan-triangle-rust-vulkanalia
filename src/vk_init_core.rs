#[cfg(debug_assertions)]
use std::os::raw::c_char;

use ash::ext::debug_utils;
use ash::khr::surface::Instance as SurfaceInstance;
use ash::khr::{surface, swapchain};
use ash::vk;
use ash::vk::Handle;
use ash::vk::TaggedStructure;

use sdl3_sys::video::SDL_Window;
use sdl3_sys::vulkan::{SDL_Vulkan_CreateSurface, SDL_Vulkan_GetInstanceExtensions};

use crate::assert::assert_sdl;
#[cfg(debug_assertions)]
use crate::vk_debug::vk_debug_messenger_info;
use crate::vk_models::VkQueueFamilyIndices;

pub fn vk_create_surface(
    entry: &ash::Entry,
    instance: &ash::Instance,
    sdl_window: &*mut SDL_Window,
) -> (surface::Instance, vk::SurfaceKHR){
    let mut surface = vk::SurfaceKHR::null();
    let instance_raw = instance.handle().as_raw() as sdl3_sys::vulkan::VkInstance;
    let surface_loader = surface::Instance::load(entry, instance);

    unsafe {
        assert_sdl!(SDL_Vulkan_CreateSurface(
            *sdl_window,
            instance_raw,
            std::ptr::null(),
            &mut surface as *mut _ as *mut sdl3_sys::vulkan::VkSurfaceKHR,
        ));
    }

    assert_ne!(surface, vk::SurfaceKHR::null(), "SDL returned null surface");

    (surface_loader, surface)
}

pub fn vk_create_instance(entry: &ash::Entry) -> ash::Instance {
    #[cfg(debug_assertions)]
    let layer_names = [c"VK_LAYER_KHRONOS_validation"];
    #[cfg(debug_assertions)]
    let layers_names_raw: Vec<*const c_char> = layer_names.iter().map(|raw_name| raw_name.as_ptr()).collect();


    let mut sdl_extension_count: u32 = 0;
    let sdl_extensions = unsafe { SDL_Vulkan_GetInstanceExtensions(&mut sdl_extension_count) };
    assert_sdl!(!sdl_extensions.is_null());

    let mut extension_names =
        unsafe { std::slice::from_raw_parts(sdl_extensions, sdl_extension_count as usize) }.to_vec();
    extension_names.push(debug_utils::NAME.as_ptr());

    const APP_NAME: &std::ffi::CStr = c"vulkan-triangle-rust-ash";
    
    let appinfo = vk::ApplicationInfo::default()
        .application_name(APP_NAME)
        .application_version(vk::make_api_version(0, 0, 1, 0))
        .engine_name(APP_NAME)
        .engine_version(vk::make_api_version(0, 0, 1, 0))
        .api_version(vk::API_VERSION_1_4);

    let create_flags = if cfg!(any(target_os = "macos", target_os = "ios")) {
        vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
    } else {
        vk::InstanceCreateFlags::default()
    };

    #[cfg(debug_assertions)]
    let enabled_validation_features = [
        vk::ValidationFeatureEnableEXT::SYNCHRONIZATION_VALIDATION,
        vk::ValidationFeatureEnableEXT::BEST_PRACTICES,
    ];
    #[cfg(debug_assertions)]
    let mut validation_features =
        vk::ValidationFeaturesEXT::default().enabled_validation_features(&enabled_validation_features);

    #[cfg(debug_assertions)]
    let mut debug_messenger_info = vk_debug_messenger_info();
    #[cfg(debug_assertions)]
    let create_info = vk::InstanceCreateInfo::default()
        .application_info(&appinfo)
        .enabled_layer_names(&layers_names_raw)
        .enabled_extension_names(&extension_names)
        .flags(create_flags)
        .push(&mut debug_messenger_info)
        .push(&mut validation_features);

    #[cfg(not(debug_assertions))]
    let create_info = vk::InstanceCreateInfo::default()
        .application_info(&appinfo)
        .enabled_extension_names(&extension_names)
        .flags(create_flags);

    let instance = unsafe { entry.create_instance(&create_info, None) }.expect("Instance creation error");

    instance
}

fn vk_get_max_usable_sample_count(
    instance: &ash::Instance,
    physical_device: &vk::PhysicalDevice,
) -> vk::SampleCountFlags {
    let physical_device_properties = unsafe { instance.get_physical_device_properties(*physical_device) };

    let counts = physical_device_properties.limits.framebuffer_color_sample_counts
        & physical_device_properties.limits.framebuffer_depth_sample_counts;

    if counts.contains(vk::SampleCountFlags::TYPE_64) {
        return vk::SampleCountFlags::TYPE_64;
    }
    if counts.contains(vk::SampleCountFlags::TYPE_32) {
        return vk::SampleCountFlags::TYPE_32;
    }
    if counts.contains(vk::SampleCountFlags::TYPE_16) {
        return vk::SampleCountFlags::TYPE_16;
    }
    if counts.contains(vk::SampleCountFlags::TYPE_8) {
        return vk::SampleCountFlags::TYPE_8;
    }
    if counts.contains(vk::SampleCountFlags::TYPE_4) {
        return vk::SampleCountFlags::TYPE_4;
    }
    if counts.contains(vk::SampleCountFlags::TYPE_2) {
        return vk::SampleCountFlags::TYPE_2;
    }

    vk::SampleCountFlags::TYPE_1
}

fn vk_is_physical_device_suitable(instance: &ash::Instance, physical_device: &vk::PhysicalDevice) -> bool {
    let mut mesh_shader_features = vk::PhysicalDeviceMeshShaderFeaturesEXT::default();
    let mut unified_layouts_features = vk::PhysicalDeviceUnifiedImageLayoutsFeaturesKHR::default();
    let mut vulkan11_features = vk::PhysicalDeviceVulkan11Features::default();
    let mut vulkan12_features = vk::PhysicalDeviceVulkan12Features::default();
    let mut vulkan13_features = vk::PhysicalDeviceVulkan13Features::default();
    let mut vulkan14_features = vk::PhysicalDeviceVulkan14Features::default();

    let mut device_features2 = vk::PhysicalDeviceFeatures2::default()
        .push(&mut vulkan14_features)
        .push(&mut vulkan13_features)
        .push(&mut vulkan12_features)
        .push(&mut vulkan11_features)
        .push(&mut unified_layouts_features)
        .push(&mut mesh_shader_features);

    let mut device_properties2 = vk::PhysicalDeviceProperties2::default();
    unsafe { instance.get_physical_device_properties2(*physical_device, &mut device_properties2) };
    unsafe { instance.get_physical_device_features2(*physical_device, &mut device_features2) };

    device_properties2.properties.api_version >= vk::API_VERSION_1_4
        && device_features2.features.geometry_shader == vk::TRUE
        && device_features2.features.sampler_anisotropy == vk::TRUE
        && vulkan11_features.shader_draw_parameters == vk::TRUE
        && vulkan12_features.buffer_device_address == vk::TRUE
        && vulkan12_features.draw_indirect_count == vk::TRUE
        && vulkan12_features.timeline_semaphore == vk::TRUE
        && vulkan13_features.synchronization2 == vk::TRUE
        && vulkan13_features.dynamic_rendering == vk::TRUE
        && vulkan14_features.host_image_copy == vk::TRUE
        && vulkan14_features.push_descriptor == vk::TRUE
        && unified_layouts_features.unified_image_layouts == vk::TRUE
        && mesh_shader_features.task_shader == vk::TRUE
        && mesh_shader_features.mesh_shader == vk::TRUE
}

fn vk_physical_device_type_score(instance: &ash::Instance, physical_device: &vk::PhysicalDevice) -> u32 {
    let properties = unsafe { instance.get_physical_device_properties(*physical_device) };
    match properties.device_type {
        vk::PhysicalDeviceType::DISCRETE_GPU => 400,
        vk::PhysicalDeviceType::INTEGRATED_GPU => 300,
        vk::PhysicalDeviceType::VIRTUAL_GPU => 200,
        vk::PhysicalDeviceType::CPU => 100,
        _ => 0,
    }
}

fn vk_is_physical_device_has_graphics_and_present_family(
    instance: &ash::Instance,
    surface_loader: &SurfaceInstance,
    physical_device: &vk::PhysicalDevice,
    surface: &vk::SurfaceKHR,
) -> bool {
    unsafe { instance.get_physical_device_queue_family_properties(*physical_device) }
        .iter()
        .enumerate()
        .filter(|(_, queue_family_properties)| queue_family_properties.queue_flags.contains(vk::QueueFlags::GRAPHICS))
        .any(|(queue_family_index, _)| unsafe {
            surface_loader
                .get_physical_device_surface_support(*physical_device, queue_family_index as u32, *surface)
                .unwrap_or(false)
        })
}

pub fn vk_pick_physical_device(
    instance: &ash::Instance,
    surface_loader: &SurfaceInstance,
    surface: &vk::SurfaceKHR,
) -> (vk::PhysicalDevice, VkQueueFamilyIndices, u32, vk::SampleCountFlags) {
    let physical_devices = unsafe { instance.enumerate_physical_devices() }.expect("Physical device error");

    let physical_device = *physical_devices
        .iter()
        .filter(|physical_device| vk_is_physical_device_suitable(instance, *physical_device))
        .filter(|physical_device| {
            vk_is_physical_device_has_graphics_and_present_family(instance, surface_loader, *physical_device, surface)
        })
        .max_by_key(|physical_device| vk_physical_device_type_score(instance, *physical_device))
        .expect("Couldn't find suitable device");

    let device_properties = unsafe { instance.get_physical_device_properties(physical_device) };
    let device_name = unsafe { std::ffi::CStr::from_ptr(device_properties.device_name.as_ptr()) };
    println!(
        "Selected physical device: {} ({:?})",
        device_name.to_str().unwrap_or("<invalid UTF-8>"),
        device_properties.device_type
    );

    let msaa_samples = vk_get_max_usable_sample_count(&instance, &physical_device);
    let queue_family_indices = vk_find_best_queue_families(&instance, &surface_loader, &physical_device, &surface);
    let queue_family_index = queue_family_indices
        .graphics_family
        .expect("graphics queue family must be present");

    (physical_device, queue_family_indices, queue_family_index, msaa_samples)
}

fn vk_find_best_queue_families(
    instance: &ash::Instance,
    surface_instance: &SurfaceInstance,
    physical_device: &vk::PhysicalDevice,
    surface: &vk::SurfaceKHR,
) -> VkQueueFamilyIndices {
    let queue_families = unsafe { instance.get_physical_device_queue_family_properties(*physical_device) };

    let mut indices = VkQueueFamilyIndices::default();

    let mut best_graphics_score: u32 = 0;
    let mut best_compute_score: u32 = 0;
    let mut best_transfer_score: u32 = 0;

    for (i, queue_family) in queue_families.iter().enumerate() {
        let flags = queue_family.queue_flags;
        let queue_count = queue_family.queue_count;
        let i = i as u32;

        // Graphics queue (highest priority)
        if flags.contains(vk::QueueFlags::GRAPHICS) {
            let present_support = unsafe {
                surface_instance
                    .get_physical_device_surface_support(*physical_device, i, *surface)
                    .unwrap_or(false)
            };

            if present_support {
                let mut score = queue_count;
                if flags.contains(vk::QueueFlags::COMPUTE) {
                    score += 10;
                }
                if flags.contains(vk::QueueFlags::TRANSFER) {
                    score += 5;
                }

                if indices.graphics_family.is_none() || score > best_graphics_score {
                    indices.graphics_family = Some(i);
                    best_graphics_score = score;
                }
            }
        }

        // Compute queue (prefer dedicated)
        if flags.contains(vk::QueueFlags::COMPUTE) {
            let mut score = queue_count;
            if !flags.contains(vk::QueueFlags::GRAPHICS) {
                score += 20;
            }

            if indices.compute_family.is_none() || score > best_compute_score {
                indices.compute_family = Some(i);
                best_compute_score = score;
            }
        }

        // Transfer queue (prefer dedicated)
        if flags.contains(vk::QueueFlags::TRANSFER) {
            let mut score = queue_count;
            if !flags.contains(vk::QueueFlags::GRAPHICS) && !flags.contains(vk::QueueFlags::COMPUTE) {
                score += 30;
            }

            if indices.transfer_family.is_none() || score > best_transfer_score {
                indices.transfer_family = Some(i);
                best_transfer_score = score;
            }
        }
    }

    indices
}

pub fn vk_create_logical_device(
    instance: &ash::Instance,
    physical_device: &vk::PhysicalDevice,
    queue_family_indices: &VkQueueFamilyIndices,
) -> (ash::Device, vk::Queue, vk::Queue) {
    const QUEUE_PRIORITY: f32 = 0.5;
    let priorities = [QUEUE_PRIORITY];

    let queue_family = queue_family_indices
        .graphics_family
        .expect("graphics queue family must be present");

    let queue_info = vk::DeviceQueueCreateInfo::default()
        .queue_family_index(queue_family)
        .queue_priorities(&priorities);

    let mut queue_create_infos = vec![queue_info];

    if let Some(compute_family) = queue_family_indices.compute_family {
        if compute_family != queue_family {
            let compute_queue_info = vk::DeviceQueueCreateInfo::default()
                .queue_family_index(compute_family)
                .queue_priorities(&priorities);
            queue_create_infos.push(compute_queue_info);
        }
    }

    let device_extension_names_raw = [
        swapchain::NAME.as_ptr(),
        ash::ext::mesh_shader::NAME.as_ptr(),
        ash::khr::unified_image_layouts::NAME.as_ptr(),
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        ash::khr::portability_subset::NAME.as_ptr(),
    ];

    let features = vk::PhysicalDeviceFeatures {
        sampler_anisotropy: vk::TRUE,
        pipeline_statistics_query: vk::TRUE,
        inherited_queries: vk::TRUE,
        shader_clip_distance: vk::TRUE,
        ..Default::default()
    };

    let mut mesh_shader_features = vk::PhysicalDeviceMeshShaderFeaturesEXT::default()
        .task_shader(true)
        .mesh_shader(true);

    let mut unified_layouts_features =
        vk::PhysicalDeviceUnifiedImageLayoutsFeaturesKHR::default().unified_image_layouts(true);

    let mut vulkan11_features = vk::PhysicalDeviceVulkan11Features::default().shader_draw_parameters(true);

    let mut vulkan12_features = vk::PhysicalDeviceVulkan12Features::default()
        .draw_indirect_count(true)
        .timeline_semaphore(true)
        .buffer_device_address(true);

    let mut vulkan13_features = vk::PhysicalDeviceVulkan13Features::default()
        .synchronization2(true)
        .dynamic_rendering(true);

    let mut vulkan14_features = vk::PhysicalDeviceVulkan14Features::default()
        .host_image_copy(true)
        .push_descriptor(true);

    let device_create_info = vk::DeviceCreateInfo::default()
        .queue_create_infos(&queue_create_infos)
        .enabled_extension_names(&device_extension_names_raw)
        .enabled_features(&features)
        .push(&mut vulkan14_features)
        .push(&mut vulkan13_features)
        .push(&mut vulkan12_features)
        .push(&mut vulkan11_features)
        .push(&mut unified_layouts_features)
        .push(&mut mesh_shader_features);

    let device: ash::Device = unsafe {
        instance
            .create_device(*physical_device, &device_create_info, None)
            .expect("Create logical device")
    };

    let queue = unsafe { device.get_device_queue(queue_family, 0) };

    let compute_queue = match queue_family_indices.compute_family {
        Some(compute_family) => unsafe { device.get_device_queue(compute_family, 0) },
        None => queue,
    };

    (device, queue, compute_queue)
}
