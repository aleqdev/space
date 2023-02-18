pub mod systems {
    use super::super::BodyRef;
    use bevy::prelude::*;

    use crate::space::{
        display::{custom_params::ComputedScale, RelativeWorldOffset},
        simulation::SpaceSimulation,
    };

    pub fn sync_with_simulation(
        mut bodies: Query<(&mut Transform, &BodyRef)>,
        simulation: Res<SpaceSimulation>,
        relative_world_offset: Res<RelativeWorldOffset>,
        scale: ComputedScale,
    ) {
        let scale = scale.get_scale();
        let offset = relative_world_offset.translation;

        for (mut transform, &BodyRef(i)) in &mut bodies {
            let position = simulation.bodies.position[i];

            transform.translation = Vec3::new(
                ((position.x - offset.x) * scale) as f32,
                ((position.y - offset.y) * scale) as f32,
                ((position.z - offset.z) * scale) as f32,
            );
        }
    }
}
