use winit::window::Window;
use winit::{event::*, event_loop::EventLoop, window::WindowBuilder};

mod camera;
mod chunk;
mod engine;
mod window;
mod world;
use crate::window::WindowState;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// use crate::engine::test_render_initialization;

// we figure out what this is very early on in runtime
// we only ever need to borrow_mut this variable
// when we resize the window,
// currently we don't have to account for multithreading
static mut WINDOW_STATE: Option<WindowState> = None;

// these functions aren't the most sound,
// they're just more for ergonomics
async fn init_window_state(window: Window) {
    unsafe {
        WINDOW_STATE = Some(WindowState::new(window).await);
    }
}
pub fn window_state<'a>() -> &'a WindowState {
    unsafe {
        WINDOW_STATE
            .as_ref()
            .expect("Window state is uninitialized.")
    }
}
pub fn window_state_mut<'a>() -> &'a mut WindowState {
    unsafe {
        WINDOW_STATE
            .as_mut()
            .expect("Window state is uninitialized.")
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    // set logging
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }

    // init window
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    init_window_state(window).await;

    // init game logic
    let mut game_state = world::init();
    // let state = state::GameState::<(), ()>::new(renderer, ());

    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        use winit::dpi::PhysicalSize;
        window.set_inner_size(PhysicalSize::new(450, 400));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm")?;
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    let mut now = instant::now();

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            window_id,
            ref event,
        } if window_id == window_state().window().id() => {
            // calc frame delta

            if true {
                // !window_state.input(event) {
                match event {
                    WindowEvent::CloseRequested => control_flow.set_exit(),
                    WindowEvent::Resized(physical_size) => game_state.resize(*physical_size),
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        game_state.resize(**new_inner_size)
                    }
                    _ => {}
                }
            }
        }
        Event::RedrawRequested(window_id) if window_id == window_state().window().id() => {
            game_state.data.delta = instant::now() - now;
            now = instant::now();

            game_state.process_events();
            game_state.queue_event(world::Event::Tick);

            match game_state.renderer.render() {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost) => {
                    let size = window_state().size;
                    game_state.resize(size);
                }
                Err(wgpu::SurfaceError::OutOfMemory) => control_flow.set_exit(),
                Err(e) => eprintln!("{:?}", e),
            }
        }
        Event::MainEventsCleared => {
            window_state().window().request_redraw();
        }
        _ => {}
    })
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
