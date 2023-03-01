use spirv_builder::SpirvBuilder;

fn main() {
    let shaders = std::fs::read_dir("../../lib/shaders").expect("Error finding shaders folder")
        .map(|f| f.unwrap().path())
        .filter(|f| f.join("Cargo.toml").exists());
    for path in shaders {
        SpirvBuilder::new(path, "spirv-unknown-vulkan1.1")
            .build()
            .expect("Shader failed to compile");
    }
}