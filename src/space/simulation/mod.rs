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

pub mod systems {
    use bevy::prelude::*;
    use bevy_debug_text_overlay::screen_print;

    use super::SpaceSimulationState;

    pub struct ToggleSpaceSimulationStateEvent;

    pub fn toggle_simulation_state(
        mut ev: EventReader<ToggleSpaceSimulationStateEvent>,
        mut state: ResMut<State<SpaceSimulationState>>,
    ) {
        for _ in ev.iter() {
            let before = state.current().clone();

            let next = match before {
                SpaceSimulationState::Stopped => SpaceSimulationState::Running,
                SpaceSimulationState::Running => SpaceSimulationState::Stopped,
            };
            state.overwrite_set(next.clone()).unwrap();

            screen_print!(sec: 2.0, col: Color::CYAN, "simulation state changed: [{before:?}] -> [{next:?}]");
        }
    }
}

pub struct SpaceSimulationPlugin;

impl Plugin for SpaceSimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<systems::ToggleSpaceSimulationStateEvent>();
        app.add_state(SpaceSimulationState::Stopped);

        app.add_system(systems::toggle_simulation_state);

        app.add_system_set(
            SystemSet::on_update(SpaceSimulationState::Running)
                .with_system(space_simulation::systems::simulation_take_step),
        );
    }
}
