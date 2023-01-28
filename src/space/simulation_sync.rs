use crate::space::SpaceSimulation;

use bevy::prelude::*;
use bevy_polyline::prelude::Polyline;

use super::{BodyRef, CameraScale};

pub fn sync_with_simulation(
    mut bodies: Query<(&mut Transform, &BodyRef, &Handle<Polyline>)>,
    mut polylines: ResMut<Assets<Polyline>>,
    simulation: Res<SpaceSimulation>,
    camera_scale: Res<CameraScale>,
) {
    use ringbuffer::RingBufferExt;

    for (mut transform, &BodyRef(i), polyline_handle) in &mut bodies {
        let p = simulation.positions[i];
        transform.translation = Vec3::new(
            (p.x * camera_scale.scale) as f32,
            (p.y * camera_scale.scale) as f32,
            (p.z * camera_scale.scale) as f32,
        );
        polylines.get_mut(polyline_handle).unwrap().vertices = simulation.trails[i]
            .iter()
            .map(|v| {
                Vec3::new(
                    (v.x * camera_scale.scale) as f32,
                    (v.y * camera_scale.scale) as f32,
                    (v.z * camera_scale.scale) as f32,
                )
            })
            .collect();
    }
}
