use bevy::prelude::*;

#[derive(Component)]
pub struct RealisticView;

pub mod systems {
    use bevy::prelude::*;

    use crate::space::{
        display::{BodyRef, CameraScale, RealisticView, RelativeWorldScale, SchematicView},
        simulation::SpaceSimulation,
    };

    pub fn update_bodies_on_enter(
        mut schematic_meshes: Query<&mut Visibility, (With<SchematicView>, Without<RealisticView>)>,
        mut realistic_meshes: Query<&mut Visibility, With<RealisticView>>,
    ) {
        for mut mesh in &mut schematic_meshes {
            mesh.is_visible = false;
        }
        for mut mesh in &mut realistic_meshes {
            mesh.is_visible = true;
        }
    }

    pub fn update_bodies(
        bodies: Query<&BodyRef>,
        simulation: Res<SpaceSimulation>,
        camera_scale: Res<CameraScale>,
        relative_world_scale: Res<RelativeWorldScale>,
        mut meshes: Query<(&mut Transform, &Parent), With<RealisticView>>,
    ) {
        let scale = camera_scale.scale * relative_world_scale.scale;

        for (mut transform, parent) in &mut meshes {
            let Ok(&BodyRef(body_ref)) = bodies.get(parent.get()) else { continue };

            transform.scale = Vec3::splat((simulation.bodies.radius[body_ref] * scale) as f32);
        }
    }
}
