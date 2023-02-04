pub mod systems {
    use bevy::prelude::*;
    use bevy_mod_raycast::RaycastMesh;

    use crate::space::{
        display::{BodyRef, CameraScale, RelativeWorldScale},
        scene::SelectionRaycastSet,
        simulation::SpaceSimulation,
    };

    pub fn update_bodies_on_enter(
        mut bodies: Query<(&BodyRef, &Children), With<BodyRef>>,
        simulation: Res<SpaceSimulation>,
        camera: Res<CameraScale>,
        mut meshes: Query<
            (&mut Transform, &Handle<StandardMaterial>),
            With<RaycastMesh<SelectionRaycastSet>>,
        >,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        for (&BodyRef(body_ref), children) in &mut bodies {
            let (mut transform, material_handle) = meshes.get_mut(children[0]).unwrap();

            transform.scale =
                Vec3::splat((simulation.bodies.radius[body_ref] * camera.scale) as f32);

            materials.get_mut(material_handle).unwrap().base_color = Color::GRAY;
        }
    }

    pub fn update_bodies(
        mut bodies: Query<(&BodyRef, &Children), With<BodyRef>>,
        simulation: Res<SpaceSimulation>,
        camera_scale: Res<CameraScale>,
        relative_world_scale: Res<RelativeWorldScale>,
        mut meshes: Query<&mut Transform, With<RaycastMesh<SelectionRaycastSet>>>,
    ) {
        let scale = camera_scale.scale * relative_world_scale.scale;
        for (&BodyRef(body_ref), children) in &mut bodies {
            let mut transform = meshes.get_mut(children[0]).unwrap();

            transform.scale = Vec3::splat((simulation.bodies.radius[body_ref] * scale) as f32);
        }
    }
}
