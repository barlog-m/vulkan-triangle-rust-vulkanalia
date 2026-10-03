use std::ffi::CStr;
use sdl3_sys::error::SDL_GetError;

pub fn sdl_error() -> String {
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
        if !$e {
            log::error!("{} failed: {}", stringify!($e), crate::assert::sdl_error());
            std::process::exit(1);
        }
    };
}

pub(crate) use assert_sdl;