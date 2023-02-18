use bevy::{math::DVec3, prelude::*};
use ringbuffer::AllocRingBuffer;

#[derive(Default, Component)]
pub struct BodyTrail {
    pub body_index: usize,
    pub anchor: Option<usize>,
    pub last_anchor_position: DVec3,
    pub trail: AllocRingBuffer<DVec3>,
}

pub mod systems {
    use bevy::{math::DVec3, prelude::*};
    use bevy_ecs_markers::params::Marker;
    use bevy_polyline::prelude::Polyline;

    use crate::space::{
        display::{custom_params::ComputedScale, BodyRef, RelativeWorldOffset},
        ext::EntityOpsExt,
        scene::markers::FocusedBody,
        simulation::SpaceSimulation,
    };

    use super::BodyTrail;

    pub fn extract_positions_from_simulations(
        mut body_trails: Query<&mut BodyTrail>,
        simulation: Res<SpaceSimulation>,
    ) {
        for mut body_trail in &mut body_trails {
            use ringbuffer::RingBufferWrite;

            let position = simulation.bodies.position[body_trail.body_index];
            body_trail.last_anchor_position = body_trail
                .anchor
                .map(|i| simulation.bodies.position[i])
                .unwrap_or(DVec3::ZERO);

            let new_pos = position - body_trail.last_anchor_position;
            body_trail.trail.push(new_pos);
        }
    }

    pub fn sync_polylines_to_trails(
        mut polylines: ResMut<Assets<Polyline>>,
        mut trails: Query<(&mut Transform, &BodyTrail, &Handle<Polyline>)>,
        relative_world_offset: Res<RelativeWorldOffset>,
        scale: ComputedScale,
    ) {
        let scale = scale.get_scale();
        let offset = relative_world_offset.translation;

        for (mut transform, body_trail, polyline_handle) in &mut trails {
            use ringbuffer::RingBufferExt;

            let mut polyline = polylines.get_mut(polyline_handle).unwrap();
            polyline.vertices = body_trail
                .trail
                .iter()
                .map(|v| {
                    Vec3::new(
                        ((v.x - offset.x) * scale) as f32,
                        ((v.y - offset.y) * scale) as f32,
                        ((v.z - offset.z) * scale) as f32,
                    )
                })
                .collect();

            transform.translation = Vec3::new(
                ((body_trail.last_anchor_position.x) * scale) as f32,
                ((body_trail.last_anchor_position.y) * scale) as f32,
                ((body_trail.last_anchor_position.z) * scale) as f32,
            );
        }
    }

    pub fn change_trail_anchor(
        mut body_trails: Query<&mut BodyTrail>,
        focused: Marker<FocusedBody>,
        keyboard: Res<Input<ScanCode>>,
        bodies: Query<&BodyRef>,
    ) {
        use FocusedBody::*;

        if !keyboard.just_pressed(ScanCode(30))
            || !focused[Primary].is_valid()
            || !focused[Secondary].is_valid()
        {
            return;
        };

        let &BodyRef(body) = bodies.get(focused[Primary]).unwrap();
        let anchor = bodies.get(focused[Secondary]).ok().map(|&BodyRef(b)| b);

        for mut body_trail in &mut body_trails {
            if body_trail.body_index == body {
                use ringbuffer::RingBufferExt;

                body_trail.anchor = anchor;
                body_trail.trail.clear();
                return;
            }
        }
    }

    /*    pub fn dissolve_extreme_trails(
            polyline_entities: Query<
                (&Handle<Polyline>, &Handle<PolylineMaterial>),
            >,
            polylines: ResMut<Assets<Polyline>>,
            mut polyline_materials: ResMut<Assets<PolylineMaterial>>,
            focused_body: Marker<FocusedBody>,
            time: Res<Time>,
        ) {
            use FocusedBody::*;

            let Ok(&BodyTrailRef(trail)) = bodies.get(focused_body[Primary]) else {return};

            let (polyline_handle, polyline_material_handle) = polyline_entities.get(trail).unwrap();
            let polyline = polylines.get(polyline_handle).unwrap();

            let mut iter = polyline.vertices.iter().rev();

            let (Some(v1), Some(v2)) = (iter.next(), iter.next()) else { return };

            let distance = v1.distance(*v2);

            let polyline_material = polyline_materials
                .get_mut(polyline_material_handle)
                .unwrap();
            let opacity_change = if distance > 1000.0 { -2.0 } else { 1.0 };
            let a = polyline_material.color.a();
            polyline_material
                .color
                .set_a((a + opacity_change * time.delta_seconds() * 0.5).clamp(0.0, 1.0));
        }
    */
}
