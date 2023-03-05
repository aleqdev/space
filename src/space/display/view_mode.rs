use bevy::prelude::*;
use bevy_debug_text_overlay::screen_print;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum ViewMode {
    Realistic,
    Schematic,
}

pub struct ToggleViewModeEvent;

pub fn toggle_view_mode(
    mut ev: EventReader<ToggleViewModeEvent>,
    mut state: ResMut<State<ViewMode>>,
) {
    for _ in ev.iter() {
        let before = state.current().clone();

        let next = match before {
            ViewMode::Realistic => ViewMode::Schematic,
            ViewMode::Schematic => ViewMode::Realistic,
        };

        state.overwrite_set(next.clone()).unwrap();

        screen_print!(sec: 2.0, col: Color::CYAN, "view mode changed: [{before:?}] -> [{next:?}]");
    }
}
