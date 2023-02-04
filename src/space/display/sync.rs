pub mod systems {
    use super::super::BodyRef;
    use bevy::prelude::*;
    use bevy_mod_raycast::RaycastMesh;
    use bevy_polyline::prelude::Polyline;

    use crate::space::{
        display::{BodyTrailRef, CameraScale, RelativeWorldOffset, RelativeWorldScale},
        scene::SelectionRaycastSet,
        simulation::SpaceSimulation,
    };

    pub fn sync_with_simulation(
        mut bodies: Query<
            (&mut Transform, &BodyRef, &BodyTrailRef),
            (
                Without<RaycastMesh<SelectionRaycastSet>>,
                Without<Handle<Polyline>>,
            ),
        >,
        mut polyline_entities: Query<(&mut Transform, &Handle<Polyline>), Without<BodyTrailRef>>,
        mut polylines: ResMut<Assets<Polyline>>,
        simulation: Res<SpaceSimulation>,
        camera_scale: Res<CameraScale>,
        relative_world_scale: Res<RelativeWorldScale>,
        relative_world_offset: Res<RelativeWorldOffset>,
    ) {
        use ringbuffer::RingBufferExt;

        let scale = camera_scale.scale * relative_world_scale.scale;
        let offset = relative_world_offset.translation;

        info!("{offset:?}, {scale:?}");

        for (mut transform, &BodyRef(i), &BodyTrailRef(trail)) in &mut bodies {
            let (mut polyline_transform, polyline_handle) =
                polyline_entities.get_mut(trail).unwrap();

            let position = simulation.bodies.position[i];

            transform.translation = Vec3::new(
                ((position.x - offset.x) * scale) as f32,
                ((position.y - offset.y) * scale) as f32,
                ((position.z - offset.z) * scale) as f32,
            );

            polyline_transform.translation =
                Vec3::new((-offset.x * scale) as f32, (-offset.y * scale) as f32, (-offset.z * scale) as f32);
            polyline_transform.scale = Vec3::splat(scale as f32);

            polylines.get_mut(polyline_handle).unwrap().vertices = simulation.bodies.trail[i]
                .iter()
                .map(|v| Vec3::new(v.x as f32, v.y as f32, v.z as f32))
                .collect();
        }
    }
}
