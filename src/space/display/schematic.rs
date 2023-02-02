pub mod systems {
    use bevy::prelude::*;
    use bevy_mod_raycast::RaycastMesh;

    use crate::space::{
        display::BodyRef,
        scene::{markers::MainCamera3d, SelectionRaycastSet},
    };

    pub fn update_bodies_on_enter(
        bodies: Query<&Children, With<BodyRef>>,
        meshes: Query<&Handle<StandardMaterial>, With<RaycastMesh<SelectionRaycastSet>>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        for children in &bodies {
            materials
                .get_mut(meshes.get(children[0]).unwrap())
                .unwrap()
                .base_color = Color::GRAY;
        }
    }

    pub fn update_bodies(
        bodies: Query<(&GlobalTransform, &Children), (With<BodyRef>, Without<MainCamera3d>)>,
        camera: Query<(&GlobalTransform, &Camera, &Projection), With<MainCamera3d>>,
        mut meshes: Query<
            &mut Transform,
            (
                With<RaycastMesh<SelectionRaycastSet>>,
                Without<MainCamera3d>,
                Without<BodyRef>,
            ),
        >,
    ) {
        let (camera_transform, camera, projection) = camera.single();

        let Projection::Perspective(projection) = projection else { return };

        for (transform, children) in &bodies {
            const RADIUS: f32 = 30.0;

            let mut mesh_transform = meshes.get_mut(children[0]).unwrap();

            let length = transform
                .translation()
                .distance(camera_transform.translation());

            info!("{length} : {}", transform.translation());

            let size = length * RADIUS / camera.logical_viewport_size().unwrap().x * projection.fov;

            mesh_transform.scale = Vec3::splat(size);
        }
    }
}
