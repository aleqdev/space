use bevy::prelude::*;

#[derive(Component)]
pub struct SelectionRectMarker;

pub mod systems {
    use bevy::prelude::*;
    use bevy_ecs_markers::params::Marker;

    use crate::space::{
        display::{BodyRef, SchematicView},
        ext::EntityOpsExt,
        scene::{
            markers::{FocusedBody, MainCamera3d},
            SelectionEvent,
        },
    };

    pub fn display_selection_rects(selected: Marker<FocusedBody>) {
        if selected.is_valid() {}
    }

    pub fn update_bodies(
        bodies: Query<&GlobalTransform, With<BodyRef>>,
        camera: Query<(&GlobalTransform, &Camera, &Projection), With<MainCamera3d>>,
        mut meshes: Query<(&mut Transform, &Parent), With<SchematicView>>,
    ) {
        const RADIUS: f32 = 30.0;

        let (camera_transform, camera, projection) = camera.single();

        let Projection::Perspective(projection) = projection else { return };

        for (mut mesh_transform, parent) in &mut meshes {
            let Ok(transform) = bodies.get(parent.get()) else { continue };

            let length = transform
                .translation()
                .distance(camera_transform.translation());

            let size = length * RADIUS / camera.logical_viewport_size().unwrap().x * projection.fov;

            mesh_transform.scale = Vec3::splat(size);
        }
    }
}
