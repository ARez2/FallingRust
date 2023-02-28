#![deny(clippy::all)]
#![forbid(unsafe_code)]

use std::{time::Duration, borrow::{Cow, BorrowMut}, num::NonZeroU32};

use egui_wgpu::wgpu::ImageDataLayout;
use glam::IVec2;
use log::{error};
use pixels::{Error, Pixels, SurfaceTexture, wgpu::{ShaderModuleDescriptor, ShaderSource, self, TextureView}};
use winit::{
    dpi::{LogicalSize, LogicalPosition},
    event::{Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

use falling_rust::{Matrix, WIDTH, HEIGHT, SCALE, Framework, Assets, UIInfo, matrix::CHUNK_SIZE_VEC, NoiseRenderer, Color};

mod texture;
use texture::Texture;


// TODO: Add rigidbodies (https://youtu.be/prXuyMCgbTc?t=358)
// TODO: Add sprite system (https://github.com/parasyte/pixels/tree/main/examples/invaders/simple-invaders)
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

    let window_size = window.inner_size();
    let (mut pixels, mut framework) = {
        let scale_factor = window.scale_factor() as f32;
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        let mut pixels = Pixels::new(WIDTH, HEIGHT, surface_texture)?;
        pixels.set_clear_color(Color::BLACK);
        let framework = Framework::new(
            &event_loop,
            window_size.width,
            window_size.height,
            scale_factor,
            &pixels,
        );
        (pixels, framework)
    };

    let mut time = 0.0;
    let mut noise_renderer = NoiseRenderer::new(&pixels, window_size.width, window_size.height)?;
    
    let mut ui_info = UIInfo::new();
    let mut assets: Assets = Assets::new();
    let mut matrix = Matrix::new_empty(WIDTH as usize, HEIGHT as usize);
    let mut paused = false;

    let mut last_update = std::time::SystemTime::now();
    let mut frame_time = last_update;
    let start = std::time::SystemTime::now();
    let mut num_frames = 0;

    
    let diffuse_bytes = include_bytes!("../data/sprites/lamp.png");
    let diffuse_texture = texture::Texture::from_bytes(pixels.device(), pixels.queue(), diffuse_bytes, "lamp.png").unwrap();


    event_loop.run(move |event, _, control_flow| {
        // The one and only event that winit_input_helper doesn't have for us...
        let current_time = std::time::SystemTime::now();
        let frame_delta = current_time.duration_since(frame_time).unwrap();
        let update_delta = current_time.duration_since(last_update).unwrap();
        let should_update = matrix.wait_time_after_frame <= 0.0 || (update_delta >= Duration::from_millis(matrix.wait_time_after_frame as u64));
        ui_info.num_frames = num_frames as f32 / current_time.duration_since(start).unwrap().as_secs_f32();
        if let Event::RedrawRequested(_) = event {
            matrix.draw(pixels.get_frame_mut());

            // Prepare egui
            framework.prepare(&window, &mut matrix, &mut assets, &mut ui_info);

            // Render everything together
            let render_result = pixels.render_with(|encoder, render_target, context| {
                let target_copy = wgpu::ImageCopyTextureBase {
                    texture: &context.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d{x: 100, y: 100, z: 0},
                    aspect: wgpu::TextureAspect::All,
                };
                let source = diffuse_texture.texture.as_image_copy();
                encoder.copy_texture_to_texture(source, target_copy, wgpu::Extent3d {
                    width: 23, height: 57, depth_or_array_layers: 0
                });
                noise_renderer.update(&context.queue);
                noise_renderer.lights[0].position[0] = time % 2.0;
                let noise_texture = noise_renderer.get_texture_view();
                noise_renderer.render(encoder, render_target, context.scaling_renderer.clip_rect());
                // Render the world texture
                context.scaling_renderer.render(encoder, noise_texture);
                
                noise_renderer.locals.time = time;
                time += 0.01;
                
                
                
                // Render egui
                framework.render(encoder, render_target, context);
                

                Ok(())
            });

            // Basic error handling
            if let Err(err) = render_result {
                error!("pixels.render() failed: {err}");
                *control_flow = ControlFlow::Exit;
            };
        };

        if let Event::WindowEvent {event, .. } = &event {
            if framework.handle_event(event) {
                return;
            };
            if let WindowEvent::MouseWheel {delta: winit::event::MouseScrollDelta::LineDelta(_, y), ..} = event {
                if y > &0.0 {
                    matrix.brush.increase_material_index();
                } else if y < &0.0 {
                    matrix.brush.decrease_material_index();
                };
                println!("Material: {:?}", matrix.brush.get_material_from_index());
            };
        };
        
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
                matrix = Matrix::new_empty(WIDTH as usize, HEIGHT as usize);
            }
            if input.key_pressed(VirtualKeyCode::F5) {
                matrix.debug_draw = !matrix.debug_draw;
                println!("Debug: {}", matrix.debug_draw);
            }
            if input.key_pressed(VirtualKeyCode::Up) {
                matrix.brush.size = matrix.brush.size.saturating_add(1);
                println!("Brush size: {}", matrix.brush.size);
            }
            if input.key_pressed(VirtualKeyCode::Down) {
                matrix.brush.size = matrix.brush.size.saturating_sub(1);
                println!("Brush size: {}", matrix.brush.size);
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
                let pos = IVec2::new(mouse_cell.0 as i32, mouse_cell.1 as i32);
                let cp = pos / CHUNK_SIZE_VEC;
                println!("Mouse click at {:?}, In bounds: {}, Chunk: {}, Chunk in bounds: {}", mouse_cell, matrix.is_in_bounds(pos), cp, matrix.chunk_in_bounds(cp));
                matrix.draw_brush(pos, matrix.brush.get_material_from_index(), &mut assets);
            } else {
                let release = input.mouse_released(0);
                let held = input.mouse_held(0);
                // If they either released (finishing the drawing) or are still
                // in the middle of drawing, keep going.
                if release || held {
                    matrix.set_line(
                        mouse_prev_cell.0,
                        mouse_prev_cell.1,
                        mouse_cell.0,
                        mouse_cell.1,
                        matrix.brush.get_material_from_index(),
                        &mut assets
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
                if let Err(err) = noise_renderer.resize(&pixels, size.width, size.height) {
                    error!("noise_renderer.resize() failed: {err}");
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
                matrix.update(&mut assets);
                last_update = std::time::SystemTime::now();
                num_frames += 1;
            };
            window.request_redraw();
        };
        frame_time = std::time::SystemTime::now();
    });
}
    
    
    
    
    
    