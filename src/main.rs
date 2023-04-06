#![allow(dead_code, unused_variables, clippy::too_many_arguments, clippy::unnecessary_wraps)]

mod app;
use app::App;

use anyhow::Result;

use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event::{ElementState, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

#[rustfmt::skip]
fn main() -> Result<()> 
{
    ::std::env::set_var("RUST_LOG", "info");
    pretty_env_logger::init();

    // Window

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Vulkan Tutorial (Rust)")
        .with_inner_size(LogicalSize::new(1024, 768))
        .build(&event_loop)?;

    // App
    let mut app = unsafe { App::create(&window)? };
    let mut destroying = false;
    let mut minimized = false;
    event_loop.run(move |event, _, control_flow| 
    {
        *control_flow = ControlFlow::Poll;
        match event 
        {
            // Render a frame if our Vulkan app is not being destroyed.
            Event::MainEventsCleared if !destroying && !minimized => unsafe { app.render(&window) }.unwrap(),
            // Resize window
            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => 
            {
                if size.width == 0 || size.height == 0 
                {
                    minimized = true;
                } else
                {
                    minimized = false;
                    app.resized = true;
                }
            },
            // Keyboard Input
            Event::WindowEvent { event: WindowEvent::KeyboardInput { input, .. }, .. } =>
            {
                if input.state == ElementState::Pressed
                {
                    match input.virtual_keycode
                    {
                        Some(VirtualKeyCode::Left) if app.models > 1 => app.models -= 1,
                        Some(VirtualKeyCode::Right) if app.models < 4 => app.models += 1,
                        _ => { }
                    }
                }
            },
            // Destroy our Vulkan app.
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => 
            {
                destroying = true;
                *control_flow = ControlFlow::Exit;
                unsafe { app.destroy(); }
            }
            _ => {}
        }
    });
}