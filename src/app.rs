use sdl3_sys::events::{SDL_EVENT_KEY_DOWN, SDL_EVENT_QUIT, SDL_Event, SDL_PollEvent};
use sdl3_sys::init::SDL_Quit;
use sdl3_sys::keycode::SDLK_ESCAPE;

use crate::mesh::Mesh;
use crate::rndr::Rndr;
use crate::window::Window;

pub struct App {
    window: Window,
    rndr: Rndr,
    mesh: Mesh,
    is_running: bool, 
}

impl App {
    pub fn new() -> Self {
        let window = Window::new();
        
        let rndr = Rndr::new(&window);

        let mesh = Mesh::new(
            &rndr.device,
            &rndr.allocator,
            &rndr.queue,
            &rndr.command_pool,
        );
        
        Self {
            window,
            rndr,
            mesh,
            is_running: true,
        }
    }

    pub fn run(&mut self) {
        while self.is_running {
            let mut ev = SDL_Event::default();

            while unsafe { SDL_PollEvent(&mut ev) } {
                match ev.event_type() {
                    SDL_EVENT_QUIT => {
                        self.is_running = false;
                    }
                    SDL_EVENT_KEY_DOWN => {
                        if unsafe { ev.key.key } == SDLK_ESCAPE {
                            self.is_running = false;
                        }
                    }
                    _ => {}
                }
            }

            self.window.update_size();
            self.rndr.draw_frame(&self.mesh, &self.window);
        }
    }
    
    
}

impl Drop for App {
    fn drop(&mut self) {
        self.mesh.destroy(&self.rndr.device, &self.rndr.allocator);
        self.rndr.destroy();
        self.window.destroy();
        unsafe { SDL_Quit(); }
    }
}

