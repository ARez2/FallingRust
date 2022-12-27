#![deny(clippy::all)]
#![forbid(unsafe_code)]

use glam::IVec2;
use image::GenericImageView;
use log::{error};
use strum::IntoEnumIterator;
use pixels::{Error, Pixels, SurfaceTexture, wgpu::{self}};
use winit::{
    dpi::{LogicalSize, LogicalPosition},
    event::{Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

use falling_rust::{Matrix, Material, WIDTH, HEIGHT, SCALE};


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

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };

    let mut life = Matrix::new_empty(WIDTH as usize, HEIGHT as usize);
    // life.set_cell_material(IVec2::new(100, 80), Material::Sand, false);
    // life.set_cell_material(IVec2::new(100, 30), Material::Dirt, false);
    let mut paused = false;

    event_loop.run(move |event, _, control_flow| {
        // The one and only event that winit_input_helper doesn't have for us...
        if let Event::RedrawRequested(_) = event {
            //std::thread::sleep(core::time::Duration::from_millis(200));
            life.draw(pixels.get_frame_mut());
            if pixels
                .render()
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
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
                life.brush_size = life.brush_size.saturating_add(1);
                println!("Brush size: {}", life.brush_size);
            }
            if input.key_pressed(VirtualKeyCode::Down) {
                life.brush_size = life.brush_size.saturating_sub(1);
                println!("Brush size: {}", life.brush_size);
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
                life.draw_brush(pos, life.get_material_from_brushindex());
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
                        life.get_material_from_brushindex(),
                    );
                }
                // If they let go or are otherwise not clicking anymore, stop drawing.
                if release || !held {
                    //debug!("Draw end");
                }
            }
            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
            }
            if !paused || input.key_pressed_os(VirtualKeyCode::Space) {
                life.update();
            }
            window.request_redraw();
        };

        if let Event::WindowEvent {event, .. } = event {
            match event {
                WindowEvent::MouseWheel { delta, ..} => match delta {
                    winit::event::MouseScrollDelta::LineDelta(x, y) => {
                        if y > 0.0 {
                            life.brush_material_index += 1;
                            if life.brush_material_index >= Material::iter().count() {
                                life.brush_material_index = 0;
                            };
                        } else if y < 0.0 {
                            if life.brush_material_index == 0 {
                                life.brush_material_index = Material::iter().count() - 1;
                            } else {
                                life.brush_material_index -= 1;
                            };
                        };
                        println!("Material: {:?}", life.get_material_from_brushindex());
                    },
                    _ => (),
                },
                _ => (),
            }
        }
    });
}





