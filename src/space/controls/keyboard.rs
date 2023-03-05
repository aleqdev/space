pub mod systems {
    use bevy::prelude::*;

    use crate::space::{
        display::ToggleViewModeEvent, simulation::systems::ToggleSpaceSimulationStateEvent,
    };

    pub fn toggle_view_mode(
        keyboard: Res<Input<ScanCode>>,
        mut ev: EventWriter<ToggleViewModeEvent>,
    ) {
        if keyboard.just_pressed(ScanCode(16)) {
            ev.send(ToggleViewModeEvent);
        }
    }

    pub fn toggle_simulation_state(
        keyboard: Res<Input<ScanCode>>,
        mut ev: EventWriter<ToggleSpaceSimulationStateEvent>,
    ) {
        if keyboard.just_pressed(ScanCode(57)) {
            ev.send(ToggleSpaceSimulationStateEvent);
        }
    }
}
