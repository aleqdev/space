use bevy::{math::DVec3, prelude::*};

#[derive(Resource, Default)]
pub struct RelativeWorldOffset {
    pub translation: DVec3,
}

pub mod systems {
    use crate::space::{
        display::BodyRef, scene::markers::FocusedBody, simulation::SpaceSimulation,
    };
    use bevy::prelude::*;
    use bevy_ecs_markers::params::Marker;

    use super::RelativeWorldOffset;

    pub fn extract_relative_world_offset(
        mut relative_world_offset: ResMut<RelativeWorldOffset>,
        simulation: Res<SpaceSimulation>,
        bodies: Query<&BodyRef>,
        focused_body: Marker<FocusedBody>,
    ) {
        let Ok(&BodyRef(body)) = bodies.get(**focused_body) else { return };

        relative_world_offset.translation = simulation.bodies.position[body];
    }
}
