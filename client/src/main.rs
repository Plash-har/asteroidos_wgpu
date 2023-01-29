mod rendering;
mod debug;
mod interface;
mod math;

use rendering::MainRenderer;

use winit::{event::{Event, WindowEvent, KeyboardInput, ElementState, VirtualKeyCode}, event_loop::{ControlFlow, EventLoop}, window::Window};

use logger::Log;

fn main() {
    env_logger::init();

    logger::info(0, Log::Str("Hello From Main"));

    // std::env::set_var("RUST_BACKTRACE", "1");

    let (window, event_loop) = setup_window_and_event_loop();

    let mut renderer = pollster::block_on(MainRenderer::new(&window, 1., cgmath::Point2 { x: 0., y: 0. }));
    let mut world = game_logic::World::new();
    let mut interface = interface::UserInterface::new();
    event_loop.run(move |event, _, control_flow| {
        renderer.handle_event(&event); // Necessary for egui

        match event {

            Event::WindowEvent { window_id, ref event } if window.id() == window_id => match event {
    
                WindowEvent::CloseRequested | WindowEvent::KeyboardInput { input: KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::Escape),
                    ..
                }, .. } => {
                    *control_flow = ControlFlow::Exit;
                },
    
                WindowEvent::KeyboardInput { .. } => {
                    interface.update_inputs(event);
                },
    
                WindowEvent::CursorMoved { .. } => {
                    interface.update_inputs(event);
                },
                
                WindowEvent::Resized(new_size) => {
                    renderer.resize(*new_size, None);
                },
                
                WindowEvent::ScaleFactorChanged { new_inner_size, scale_factor } => {
                    renderer.resize(**new_inner_size, Some(*scale_factor));
                },
                
                _ => {}
            },
    
            Event::MainEventsCleared => {
                window.request_redraw();
            },
    
            Event::RedrawRequested(window_id) if window.id() == window_id => {
    
                let gui_context = renderer.get_gui_context();
                
                interface.update(&mut world, &mut renderer, gui_context);
                world.update();
                renderer.update(&world);
    
                match renderer.render(&window) {
                    Ok(_) => {},
                    Err(wgpu::SurfaceError::Lost) => {
                        renderer.resize(renderer.size, None);
                        eprintln!("Surface Lost !");
                    },
                    Err(wgpu::SurfaceError::OutOfMemory) => panic!("Out of memory, exiting"),
                    Err(e) => eprintln!("Surface error while rendering {:?}", e),
                }
            }
    
            _ => {}
        }
    });
}

fn setup_window_and_event_loop() -> (Window, EventLoop<()>) {
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new().build(&event_loop).unwrap();

    if cfg!(windows) {
        window.set_inner_size(winit::dpi::PhysicalSize::new(1280, 720));
    } else {
        window.set_inner_size(winit::dpi::PhysicalSize::new(1066, 600));
    }

    window.set_title("Asteroidos avec WGPU");
    
    return (window, event_loop);
}