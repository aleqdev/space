use bevy::prelude::*;

pub mod camera;

pub struct ControlsPlugin;

impl Plugin for ControlsPlugin {
    fn build(&self, app: &mut App) {
        use bevy_dolly::prelude::*;

        app.add_dolly_component(crate::space::scene::markers::MainCamera3d);

        {
            use crate::space::scene::selection::systems::select_current_body;
            use camera::systems::*;

            app.add_system(zoom).add_system(orbit);
            app.add_system(focus.after(select_current_body));
        }
    }
}
