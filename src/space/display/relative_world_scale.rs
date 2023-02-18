use bevy::prelude::*;

#[derive(Resource)]
pub struct RelativeWorldScale {
    pub scale: f64,
}

#[derive(Component)]
pub struct RelativeLightIntensivity(pub f64);

pub mod systems {
    use crate::space::scene::markers::{FocusedBody, MainCamera3d};
    use bevy::prelude::*;
    use bevy_dolly::prelude::Rig;
    use bevy_ecs_markers::params::Marker;

    use super::RelativeWorldScale;

    pub fn update_world_scale(
        mut relative_world_scale: ResMut<RelativeWorldScale>,
        bodies: Query<&GlobalTransform, Without<MainCamera3d>>,
        mut camera: Query<(&mut Transform, &GlobalTransform), (With<MainCamera3d>, With<Camera>)>,
        mut camera_rig: Query<&mut Rig, (With<MainCamera3d>, Without<Camera>)>,
        focused_body: Marker<FocusedBody>,
    ) {
        use FocusedBody::*;

        use bevy_dolly::prelude::Arm;

        let Ok(focused) = bodies.get(focused_body[Primary]) else { return };

        let mut camera_rig = camera_rig.single_mut();
        let (mut camera_transform, camera_global_trasnform) = camera.single_mut();

        let distance = camera_global_trasnform
            .translation()
            .distance(focused.translation());

        if !(distance < 0.25 || distance > 4.0) {
            return;
        }

        let scaling = 1.0 / distance as f64;

        relative_world_scale.scale *= scaling;
        camera_transform.translation *= scaling as f32;
        camera_rig.driver_mut::<Arm>().offset.z *= scaling as f32;
    }
}
