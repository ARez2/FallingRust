use pixels::{Pixels, wgpu::ComputePipeline};


pub mod compute;


pub fn setup_compute(device: &pixels::wgpu::Device) -> pixels::wgpu::ComputePipeline {
    let shader = crate::compute::create_shader_module(device);
    let render_pipeline_layout = crate::compute::create_pipeline_layout(device);
    
    device.create_compute_pipeline(&pixels::wgpu::ComputePipelineDescriptor {
        label: Some("compute pipeline"),
        layout: Some(&render_pipeline_layout),
        module: &shader,
        entry_point: "compute_main",
    })
}