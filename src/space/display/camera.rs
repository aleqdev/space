use bevy::prelude::*;

#[derive(Resource)]
pub struct CameraScale {
    pub scale: f64,
}

pub mod systems {
    use bevy::prelude::*;

    use crate::space::scene::markers::{CubemapCamera3d, MainCamera3d};

    pub fn sync_cubemap_camera(
        mut cubemap_camera: Query<&mut Transform, (With<CubemapCamera3d>, Without<MainCamera3d>)>,
        main_camera: Query<&Transform, With<MainCamera3d>>,
    ) {
        let mut cubemap_camera = cubemap_camera.single_mut();
        let main_camera = main_camera.single();

        cubemap_camera.rotation = main_camera.rotation;
    }
}
