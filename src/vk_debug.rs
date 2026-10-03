use std::borrow::Cow;

use vulkanalia::prelude::v1_0::*;
use vulkanalia::vk;
use vulkanalia::vk::ExtDebugUtilsExtensionInstanceCommands;

pub fn vk_debug_messenger_info() -> vk::DebugUtilsMessengerCreateInfoEXTBuilder<'static> {
    vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(
            vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE,
        )
        .message_type(
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
        )
        .user_callback(Some(vulkan_debug_callback))
}

pub fn vk_enable_debug(instance: &Instance) -> vk::DebugUtilsMessengerEXT {
    let debug_info = vk_debug_messenger_info();

    unsafe { instance.create_debug_utils_messenger_ext(&debug_info, None) }
        .expect("Debug utils messenger creation error")
}

unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
    let callback_data = unsafe { *p_callback_data };
    let message_id_number = callback_data.message_id_number;

    let message_id_name = if callback_data.message_id_name.is_null() {
        Cow::from("")
    } else {
        unsafe { std::ffi::CStr::from_ptr(callback_data.message_id_name) }.to_string_lossy()
    };

    let message = if callback_data.message.is_null() {
        Cow::from("")
    } else {
        unsafe { std::ffi::CStr::from_ptr(callback_data.message) }.to_string_lossy()
    };

    let log_level = match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => log::Level::Debug,
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => log::Level::Info,
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => log::Level::Warn,
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => log::Level::Error,
        _ => log::Level::Info,
    };

    log::log!(
        log_level,
        "{message_type:?}: {message}",
        //"{message_type:?} [{message_id_name} ({message_id_number})]: {message}",
    );

    vk::FALSE
}
