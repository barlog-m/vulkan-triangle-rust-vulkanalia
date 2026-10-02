use std::ffi::CStr;

use sdl3_sys::error::SDL_GetError;
use sdl3_sys::events::{SDL_EVENT_KEY_DOWN, SDL_EVENT_QUIT, SDL_Event, SDL_PollEvent};
use sdl3_sys::init::{SDL_INIT_EVENTS, SDL_INIT_VIDEO, SDL_Init, SDL_Quit};
use sdl3_sys::keycode::SDLK_ESCAPE;
use sdl3_sys::surface::{SDL_FillSurfaceRect, SDL_MapSurfaceRGB, SDL_Surface};
use sdl3_sys::timer::SDL_Delay;
use sdl3_sys::video::{
    SDL_CreateWindow, SDL_DestroyWindow, SDL_DestroyWindowSurface, SDL_GetWindowSurface, SDL_SetWindowPosition,
    SDL_ShowWindow, SDL_UpdateWindowSurface, SDL_WINDOW_HIDDEN, SDL_WINDOW_RESIZABLE, SDL_WINDOW_VULKAN,
    SDL_WINDOWPOS_CENTERED, SDL_Window,
};

fn sdl_error() -> String {
    unsafe {
        let p = SDL_GetError();
        if p.is_null() {
            "Unknown SDL error".to_string()
        } else {
            CStr::from_ptr(p).to_string_lossy().into_owned()
        }
    }
}

macro_rules! assert_sdl {
    ($e:expr) => {
        assert!($e, "{} failed: {}", stringify!($e), crate::sdl_error())
    };
}

struct App {
    window: *mut SDL_Window,
    surface: *mut SDL_Surface,
    is_running: bool,
}

impl App {
    fn new() -> Self {
        Self {
            window: std::ptr::null_mut(),
            surface: std::ptr::null_mut(),
            is_running: false,
        }
    }

    fn init(&mut self) {
        unsafe {
            assert_sdl!(SDL_Init(SDL_INIT_VIDEO | SDL_INIT_EVENTS));

            let window_flags = SDL_WINDOW_VULKAN | SDL_WINDOW_RESIZABLE | SDL_WINDOW_HIDDEN;

            let app_name = concat!(env!("CARGO_PKG_NAME"), "\0");

            let window = SDL_CreateWindow(app_name.as_ptr().cast(), 1920, 1080, window_flags);
            assert_sdl!(!window.is_null());

            SDL_SetWindowPosition(window, SDL_WINDOWPOS_CENTERED, SDL_WINDOWPOS_CENTERED);
            SDL_ShowWindow(window);

            self.window = window;
            self.is_running = true;

            let surface = SDL_GetWindowSurface(window);
            assert_sdl!(!surface.is_null());

            let black = SDL_MapSurfaceRGB(surface, 0, 0, 0);
            assert_sdl!(SDL_FillSurfaceRect(surface, std::ptr::null(), black));
            assert_sdl!(SDL_UpdateWindowSurface(window));

            self.surface = surface;
        }
    }

    fn run(&mut self) {
        unsafe {
            while self.is_running {
                let mut ev = SDL_Event::default();

                while SDL_PollEvent(&mut ev) {
                    match ev.event_type() {
                        SDL_EVENT_QUIT => {
                            self.is_running = false;
                        }
                        SDL_EVENT_KEY_DOWN => {
                            if ev.key.key == SDLK_ESCAPE {
                                self.is_running = false;
                            }
                        }
                        _ => {}
                    }
                }

                SDL_UpdateWindowSurface(self.window);
                SDL_Delay(16);
            }
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        unsafe {
            if !self.window.is_null() {
                SDL_DestroyWindowSurface(self.window);
                SDL_DestroyWindow(self.window);
                self.window = std::ptr::null_mut();
                self.surface = std::ptr::null_mut();
            }
            SDL_Quit();
        }
    }
}

fn main() {
    let mut app = App::new();

    app.init();
    app.run();
}
