mod app;
mod assert;
mod logger;
mod memory_guard;
mod mesh;
mod rndr;
mod vk_debug;
mod vk_init_core;
mod vk_init_rndr;
mod vk_models;
mod vk_utils;
mod window;

use crate::app::App;
use crate::logger::logger_init;

fn main() {
    logger_init();
    
    let mut app = App::new();
    app.run();
}
