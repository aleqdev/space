#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum ViewMode {
    Realistic,
    Schematic,
}

pub mod systems {
    use bevy::prelude::*;

    use super::ViewMode;

    pub fn toggle_mode(keyboard: Res<Input<ScanCode>>, mut state: ResMut<State<ViewMode>>) {
        if keyboard.just_pressed(ScanCode(16)) {
            match state.current() {
                ViewMode::Realistic => state.overwrite_set(ViewMode::Schematic),
                ViewMode::Schematic => state.overwrite_set(ViewMode::Realistic),
            }
            .unwrap();
        }
    }
}
