#![deny(clippy::all)]
#![forbid(unsafe_code)]

use std::time::Duration;

use glam::IVec2;
use image::GenericImageView;
use log::{error};
use strum::IntoEnumIterator;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::{
    dpi::{LogicalSize, LogicalPosition},
    event::{Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

use falling_rust::{Matrix, Material, WIDTH, HEIGHT, SCALE, Framework};


// TODO: Add rigidbodies (https://youtu.be/prXuyMCgbTc?t=358)
// TODO: Add sprite system (https://github.com/parasyte/pixels/tree/main/examples/invaders/simple-invaders)
// TODO: Add fire
// TODO: Maybe add (verlet) rope physics
// TODO: Camera system
// TODO: Physics (https://parry.rs/)
// TODO: Audio (https://crates.io/crates/kira)


fn main() -> Result<(), Error> {
    //env::set_var("RUST_LOG", "falling_rust=debug");
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();

    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        let scaled_size = LogicalSize::new(WIDTH as f64 * SCALE, HEIGHT as f64 * SCALE);
        WindowBuilder::new()
            .with_title("Falling Sand Simulation")
            .with_inner_size(scaled_size)
            .with_min_inner_size(size)
            .with_position(LogicalPosition::new(2560 - (WIDTH as f64 * SCALE).round() as u32 - 50, 30))
            .build(&event_loop)
            .unwrap()
    };

    let (mut pixels, mut framework) = {
        let window_size = window.inner_size();
        let scale_factor = window.scale_factor() as f32;
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        let pixels = Pixels::new(WIDTH, HEIGHT, surface_texture)?;
        let framework = Framework::new(
            &event_loop,
            window_size.width,
            window_size.height,
            scale_factor,
            &pixels,
        );
        (pixels, framework)
    };

    let mut life = Matrix::new_empty(WIDTH as usize, HEIGHT as usize);
    let mut paused = false;

    let mut last_update = std::time::SystemTime::now();
    let mut frame_time = last_update;
    event_loop.run(move |event, _, control_flow| {
        // The one and only event that winit_input_helper doesn't have for us...
        let current_time = std::time::SystemTime::now();
        let frame_delta = current_time.duration_since(frame_time).unwrap();
        let update_delta = current_time.duration_since(last_update).unwrap();
        let should_update = life.wait_time_after_frame <= 0.0 || (update_delta >= Duration::from_millis(life.wait_time_after_frame as u64));
        // if should_update {
        //     println!("Update: {:?} -> {}", update_delta, should_update);
        // };
        //println!("Update delta: {:?}, wait time: {:?}, should update: {}, why: {}", update_delta, Duration::from_millis(life.wait_time_after_frame as u64), should_update, update_delta >= Duration::from_millis(life.wait_time_after_frame as u64));
        if let Event::RedrawRequested(_) = event {
            life.draw(pixels.get_frame_mut());

            // Prepare egui
            framework.prepare(&window, &mut life);

            // Render everything together
            let render_result = pixels.render_with(|encoder, render_target, context| {
                // Render the world texture
                context.scaling_renderer.render(encoder, render_target);

                // Render egui
                framework.render(encoder, render_target, context);

                Ok(())
            });

            // Basic error handling
            if let Err(err) = render_result {
                error!("pixels.render() failed: {err}");
                *control_flow = ControlFlow::Exit;
            }
        }

        if let Event::WindowEvent {event, .. } = &event {
            if framework.handle_event(event) {
                return;
            };
            match event {
                WindowEvent::MouseWheel { delta, ..} => match delta {
                    winit::event::MouseScrollDelta::LineDelta(x, y) => {
                        if y > &0.0 {
                            life.brush.increase_material_index();
                        } else if y < &0.0 {
                            life.brush.decrease_material_index();
                        };
                        println!("Material: {:?}", life.brush.get_material_from_index());
                    },
                    _ => (),
                },
                _ => (),
            }
        }
        
        // For everything else, for let winit_input_helper collect events to build its state.
        // It returns `true` when it is time to update our game state and request a redraw.
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }
            if input.key_pressed(VirtualKeyCode::P) {
                paused = !paused;
            }
            if input.key_pressed_os(VirtualKeyCode::Space) {
                // Space is frame-step, so ensure we're paused
                paused = true;
            }
            if input.key_pressed(VirtualKeyCode::C) {
                life = Matrix::new_empty(WIDTH as usize, HEIGHT as usize);
            }
            if input.key_pressed(VirtualKeyCode::F5) {
                life.debug_draw = !life.debug_draw;
                println!("Debug: {}", life.debug_draw);
            }
            if input.key_pressed(VirtualKeyCode::Up) {
                life.brush.size = life.brush.size.saturating_add(1);
                println!("Brush size: {}", life.brush.size);
            }
            if input.key_pressed(VirtualKeyCode::Down) {
                life.brush.size = life.brush.size.saturating_sub(1);
                println!("Brush size: {}", life.brush.size);
            }
            // Handle mouse. This is a bit involved since support some simple
            // line drawing (mostly because it makes nice looking patterns).
            let (mouse_cell, mouse_prev_cell) = input
                .mouse()
                .map(|(mx, my)| {
                    let (dx, dy) = input.mouse_diff();
                    let prev_x = mx - dx;
                    let prev_y = my - dy;
                    
                    let (mx_i, my_i) = pixels
                    .window_pos_to_pixel((mx, my))
                    .unwrap_or_else(|pos| pixels.clamp_pixel_pos(pos));
                    
                    let (px_i, py_i) = pixels
                    .window_pos_to_pixel((prev_x, prev_y))
                    .unwrap_or_else(|pos| pixels.clamp_pixel_pos(pos));
                    
                    (
                        (mx_i as isize, my_i as isize),
                        (px_i as isize, py_i as isize),
                    )
                })
                .unwrap_or_default();
                
            if input.mouse_pressed(0) {
                //println!("Mouse click at {:?}", mouse_cell);
                let pos = IVec2::new(mouse_cell.0 as i32, mouse_cell.1 as i32);
                life.draw_brush(pos, life.brush.get_material_from_index());
            } else {
                let release = input.mouse_released(0);
                let held = input.mouse_held(0);
                // If they either released (finishing the drawing) or are still
                // in the middle of drawing, keep going.
                if release || held {
                    life.set_line(
                        mouse_prev_cell.0,
                        mouse_prev_cell.1,
                        mouse_cell.0,
                        mouse_cell.1,
                        life.brush.get_material_from_index(),
                    );
                }
                // If they let go or are otherwise not clicking anymore, stop drawing.
                if release || !held {
                    //debug!("Draw end");
                }
            }
            // Resize the window
            if let Some(size) = input.window_resized() {
                if let Err(err) = pixels.resize_surface(size.width, size.height) {
                    error!("pixels.resize_surface() failed: {err}");
                    *control_flow = ControlFlow::Exit;
                    return;
                }
                framework.resize(size.width, size.height);
            }
            // Update the scale factor
            if let Some(scale_factor) = input.scale_factor() {
                framework.scale_factor(scale_factor);
            }
            if (!paused || input.key_pressed_os(VirtualKeyCode::Space)) && should_update
            {
                life.update();
                last_update = std::time::SystemTime::now();
            };
            window.request_redraw();
        };
        frame_time = std::time::SystemTime::now();
    });
}
    
    
    
    
    
    