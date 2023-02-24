pub mod systems {
    use bevy::prelude::*;
    use bevy_debug_text_overlay::screen_print;

    use crate::space::{display::ViewMode, simulation::SpaceSimulationState};

    pub fn toggle_view_mode(keyboard: Res<Input<ScanCode>>, mut state: ResMut<State<ViewMode>>) {
        if keyboard.just_pressed(ScanCode(16)) {
            let before = state.current().clone();

            let next = match before {
                ViewMode::Realistic => ViewMode::Schematic,
                ViewMode::Schematic => ViewMode::Realistic,
            };

            state.overwrite_set(next.clone()).unwrap();

            screen_print!(sec: 2.0, col: Color::CYAN, "view mode changed: [{before:?}] -> [{next:?}]");
        }
    }

    pub fn toggle_simulation_state(
        keyboard: Res<Input<ScanCode>>,
        mut state: ResMut<State<SpaceSimulationState>>,
    ) {
        if keyboard.just_pressed(ScanCode(57)) {
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
