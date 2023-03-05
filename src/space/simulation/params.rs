use bevy::prelude::*;

#[derive(Resource)]
pub struct SpaceSimulationParams {
    pub speed: f64,
    pub percision: usize,
}
