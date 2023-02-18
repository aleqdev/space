use bevy::prelude::*;

#[derive(Component)]
pub struct SchematicView;

pub mod systems {
    use bevy::prelude::*;

    use crate::space::{
        display::{BodyRef, RealisticView, SchematicView},
        scene::markers::MainCamera3d,
    };

    pub fn update_bodies_on_enter(
        mut schematic_meshes: Query<&mut Visibility, (With<SchematicView>, Without<RealisticView>)>,
        mut realistic_meshes: Query<&mut Visibility, With<RealisticView>>,
    ) {
        for mut mesh in &mut schematic_meshes {
            mesh.is_visible = true;
        }
        for mut mesh in &mut realistic_meshes {
            mesh.is_visible = false;
        }
    }

    pub fn update_bodies(
        bodies: Query<&GlobalTransform, With<BodyRef>>,
        camera: Query<(&GlobalTransform, &Camera, &Projection), With<MainCamera3d>>,
        mut meshes: Query<(&mut Transform, &Parent), With<SchematicView>>,
    ) {
        const RADIUS: f32 = 31.0;

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
