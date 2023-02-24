use bevy::prelude::*;

pub mod params;
pub use params::*;

pub mod space_simulation;
pub use space_simulation::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SpaceSimulationState {
    Running,
    Stopped,
}

pub struct SpaceSimulationPlugin;

impl Plugin for SpaceSimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_state(SpaceSimulationState::Stopped);
        app.add_system_set(
            SystemSet::on_update(SpaceSimulationState::Running)
                .with_system(space_simulation::systems::simulation_take_step),
        );
    }
}
