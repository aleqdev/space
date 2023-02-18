use bevy::prelude::*;

pub mod params;
pub use params::*;

pub mod space_simulation;
pub use space_simulation::*;

pub mod nasa_horizons;

pub struct SpaceSimulationPlugin;

impl Plugin for SpaceSimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(space_simulation::systems::simulation_take_step);
    }
}
