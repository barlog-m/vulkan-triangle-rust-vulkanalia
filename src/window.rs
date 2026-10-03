use sdl3_sys::everything::{SDL_CreateWindow, SDL_WINDOW_RESIZABLE, SDL_WINDOW_VULKAN};
use sdl3_sys::init::{SDL_Init, SDL_INIT_VIDEO, SDL_INIT_EVENTS};
use sdl3_sys::video::{SDL_DestroyWindow, SDL_GetWindowSizeInPixels, SDL_Window};

use crate::assert::assert_sdl;

const WINDOW_WIDTH: i32 = 1920;
const WINDOW_HEIGHT: i32 = 1080;

pub struct Window {
    sdl_window: *mut SDL_Window,
    pub width: u32,
    pub height: u32,
}

impl Window {
    pub fn new() -> Self {
        unsafe {
            assert_sdl!(SDL_Init(SDL_INIT_VIDEO | SDL_INIT_EVENTS));
        }

        let window_flags = SDL_WINDOW_VULKAN | SDL_WINDOW_RESIZABLE;

            let app_name = concat!(env!("CARGO_PKG_NAME"), "\0");

            let window = unsafe { SDL_CreateWindow(app_name.as_ptr().cast(), WINDOW_WIDTH, WINDOW_HEIGHT, window_flags) };
            assert_sdl!(!window.is_null());

            let mut r = Self {
                sdl_window: window,
                width: WINDOW_WIDTH as u32,
                height: WINDOW_HEIGHT as u32,
            };
            
            r.update_size();
            
            r
    }
    
    pub fn update_size(&mut self) {
        let mut width: std::ffi::c_int = 0;
        let mut height: std::ffi::c_int = 0;
        
        unsafe {
            SDL_GetWindowSizeInPixels(self.sdl_window, &mut width, &mut height);
        }
        
        self.width = width as u32;
        self.height = height as u32;
    }

    pub fn is_zero_size(&self) -> bool {
        self.width == 0 || self.height == 0
    }
    
    pub fn raw(&self) -> *mut SDL_Window {
        self.sdl_window
    }
    
    pub fn destroy(&mut self) {
        unsafe {
            SDL_DestroyWindow(self.sdl_window);
        }
        self.sdl_window = std::ptr::null_mut();
    }
}