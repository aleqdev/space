use bevy::prelude::*;

#[derive(Component)]
pub struct RealisticView;

pub mod systems {
    use bevy::prelude::*;

    use crate::space::{
        display::{
            custom_params::ComputedScale, BodyRef, RealisticView, RelativeLightIntensivity,
            SchematicView,
        },
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
        scale: ComputedScale,
        mut meshes: Query<(&mut Transform, &Parent), With<RealisticView>>,
        mut lights: Query<(&mut PointLight, &Parent, &RelativeLightIntensivity)>,
        mut previous_scale: Local<f64>,
    ) {
        let scale = scale.get_scale();

        for (mut transform, parent) in &mut meshes {
            let Ok(BodyRef(body_ref)) = bodies.get(parent.get()) else { continue };

            let index = simulation.bodies.get_index(body_ref);

            let body_rotation = &simulation.bodies.rotations()[index];
            transform.rotation = simulation.calculate_body_rotation(
                &body_rotation.initial,
                body_rotation.sideral_rotation_offset,
                body_rotation.sideral_rotation_speed,
            );

            if *previous_scale != scale {
                transform.scale = Vec3::splat((simulation.bodies.radiuses()[index] * scale) as f32);
            }
        }

        if *previous_scale != scale {
            for (mut light, parent, &RelativeLightIntensivity(relative_intensity)) in &mut lights {
                light.intensity = (relative_intensity * scale * scale) as f32;

                let Ok(BodyRef(body_ref)) = bodies.get(parent.get()) else { continue };

                let index = simulation.bodies.get_index(body_ref);

                light.radius = (simulation.bodies.radiuses()[index] * scale) as f32;
            }
        }

        *previous_scale = scale;
    }
}
