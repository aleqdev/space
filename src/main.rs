mod space;

fn main() {
    use bevy::prelude::*;

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(space::SpacePlugin)
        .add_plugin(bevy_editor_pls::EditorPlugin)
        .run();
}
