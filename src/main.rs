mod space;

fn main() {
    use bevy::prelude::*;

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(space::SpacePlugins)
        .add_plugin(bevy_editor_pls::EditorPlugin)
        .run();
}
