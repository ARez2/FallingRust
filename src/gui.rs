use egui::{ClippedPrimitive, Context, TexturesDelta, TextureHandle, ColorImage, widgets::ImageButton};
use egui_wgpu::renderer::{Renderer, ScreenDescriptor};
use strum::IntoEnumIterator;

use crate::{Material, Matrix, Assets};

use pixels::{wgpu, PixelsContext};
use winit::event_loop::EventLoopWindowTarget;
use winit::window::Window;



/// Manages all state required for rendering egui over `Pixels`.
pub struct Framework {
    // State for egui.
    egui_ctx: Context,
    egui_state: egui_winit::State,
    screen_descriptor: ScreenDescriptor,
    renderer: Renderer,
    paint_jobs: Vec<ClippedPrimitive>,
    textures: TexturesDelta,

    // State for the GUI
    gui: Gui,
}

impl Framework {
    /// Create egui.
    pub fn new<T>(
        event_loop: &EventLoopWindowTarget<T>,
        width: u32,
        height: u32,
        scale_factor: f32,
        pixels: &pixels::Pixels,
    ) -> Self {
        let max_texture_size = pixels.device().limits().max_texture_dimension_2d as usize;

        let egui_ctx = Context::default();
        let mut egui_state = egui_winit::State::new(event_loop);
        egui_state.set_max_texture_side(max_texture_size);
        egui_state.set_pixels_per_point(scale_factor);
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [width, height],
            pixels_per_point: scale_factor,
        };
        let renderer = Renderer::new(pixels.device(), pixels.render_texture_format(), None, 1);
        let textures = TexturesDelta::default();
        let gui = Gui::new();

        Self {
            egui_ctx,
            egui_state,
            screen_descriptor,
            renderer,
            paint_jobs: Vec::new(),
            textures,
            gui,
        }
    }

    /// Handle input events from the window manager.
    pub fn handle_event(&mut self, event: &winit::event::WindowEvent) -> bool {
        let r = self.egui_state.on_event(&self.egui_ctx, event);
        r.consumed
    }

    /// Resize egui.
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.screen_descriptor.size_in_pixels = [width, height];
        }
    }

    /// Update scaling factor.
    pub fn scale_factor(&mut self, scale_factor: f64) {
        self.screen_descriptor.pixels_per_point = scale_factor as f32;
    }

    /// Prepare egui.
    pub fn prepare(&mut self, window: &Window, matrix: &mut Matrix, assets: &mut Assets) {
        // Run the egui frame and create all paint jobs to prepare for rendering.
        let raw_input = self.egui_state.take_egui_input(window);

        let output = self.egui_ctx.run(raw_input, |egui_ctx| {
            // Draw the demo application.
            self.gui.ui(egui_ctx, matrix, assets);
        });

        self.textures.append(output.textures_delta);
        self.egui_state
            .handle_platform_output(window, &self.egui_ctx, output.platform_output);
        self.paint_jobs = self.egui_ctx.tessellate(output.shapes);
    }

    /// Render egui.
    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        render_target: &wgpu::TextureView,
        context: &PixelsContext,
    ) {
        // Upload all resources to the GPU.
        for (id, image_delta) in &self.textures.set {
            self.renderer
                .update_texture(&context.device, &context.queue, *id, image_delta);
        }
        self.renderer.update_buffers(
            &context.device,
            &context.queue,
            encoder,
            &self.paint_jobs,
            &self.screen_descriptor,
        );

        // Render egui with WGPU
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: render_target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            self.renderer
                .render(&mut rpass, &self.paint_jobs, &self.screen_descriptor);
        }

        // Cleanup
        let textures = std::mem::take(&mut self.textures);
        for id in &textures.free {
            self.renderer.free_texture(id);
        }
    }
}



/// Example application state. A real application will need a lot more state than this.
struct Gui {
    /// Only show the egui window when true.
    window_open: bool,
    material_textures: Vec<(TextureHandle, Material)>,
}
impl Gui {
    fn new() -> Self {
        let material_textures = vec![];
        Self {
            window_open: true,
            material_textures,
        }
    }

    fn ui(&mut self, ctx: &Context, matrix: &mut Matrix, assets: &mut Assets) {
        egui::TopBottomPanel::top("menubar_container").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Menu", |ui| {
                    if ui.button("Material Selection").clicked() {
                        self.window_open = true;
                        ui.close_menu();
                    }
                });
                ui.checkbox(&mut matrix.brush.place_fire, "Ignite Materials");
                ui.add(egui::widgets::Slider::new(&mut matrix.wait_time_after_frame, 0.0..=1000.0).text("Frame wait time"))
            });
        });
        egui::Window::new("Material Selection")
        .open(&mut self.window_open)
        .show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                if self.material_textures.len() != Material::iter().count() {
                    for mat in Material::iter() {
                        assets.add_material_texture_instance(mat);
                        let texture = assets.get_texture_for_material(mat);
                        if let Some(texture) = texture {
                            let (w, h) = (texture.texture.width() as usize, texture.texture.height() as usize);
                            let matname = format!("{:?}", mat);
                            let col_image = ColorImage::from_rgb([w, h], &texture.pixels);
                            let tex = ctx.load_texture(matname, col_image, Default::default());
                            self.material_textures.push((tex, mat));
                        };
                    };
                };

                for (mattex, mat) in self.material_textures.iter() {
                    let resp = ui.add(ImageButton::new(mattex, (32.0, 32.0)))
                        .on_hover_text(format!("{:?}", mat));
                    if resp.clicked() {
                        matrix.brush.material_index = Material::iter().position(|x| &x == mat).unwrap();
                    };
                };
            });
        });
    }
}