use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef},
};

#[derive(AsBindGroup, TypeUuid, Default, Clone, serde::Serialize, serde::Deserialize)]
#[uuid = "96c15e62-5b6a-4c2b-9a06-fdababfcc25d"]
pub struct StarMaterial {
    #[uniform(0)]
    pub primary_color: Color,
    #[uniform(1)]
    pub secondary_color: Color,
    #[serde(skip)]
    pub alpha_mode: AlphaMode,
}

impl Material for StarMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/star.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }
}
