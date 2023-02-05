pub mod systems {
    use super::super::BodyRef;
    use bevy::prelude::*;
    use bevy_polyline::prelude::Polyline;

    use crate::space::{
        display::{BodyTrailRef, CameraScale, RelativeWorldOffset, RelativeWorldScale},
        simulation::SpaceSimulation,
    };

    pub fn sync_with_simulation(
        mut bodies: Query<(&mut Transform, &BodyRef, &BodyTrailRef)>,
        mut polyline_entities: Query<&Handle<Polyline>>,
        mut polylines: ResMut<Assets<Polyline>>,
        simulation: Res<SpaceSimulation>,
        camera_scale: Res<CameraScale>,
        relative_world_scale: Res<RelativeWorldScale>,
        relative_world_offset: Res<RelativeWorldOffset>,
    ) {
        use ringbuffer::RingBufferExt;

        let scale = camera_scale.scale * relative_world_scale.scale;
        let offset = relative_world_offset.translation;

        for (mut transform, &BodyRef(i), &BodyTrailRef(trail)) in &mut bodies {
            let polyline_handle = polyline_entities.get_mut(trail).unwrap();

            let position = simulation.bodies.position[i];

            transform.translation = Vec3::new(
                ((position.x - offset.x) * scale) as f32,
                ((position.y - offset.y) * scale) as f32,
                ((position.z - offset.z) * scale) as f32,
            );

            polylines.get_mut(polyline_handle).unwrap().vertices = simulation.bodies.trail[i]
                .iter()
                .map(|v| {
                    Vec3::new(
                        ((v.x - offset.x) * scale) as f32,
                        ((v.y - offset.y) * scale) as f32,
                        ((v.z - offset.z) * scale) as f32,
                    )
                })
                .collect();
        }
    }
}
