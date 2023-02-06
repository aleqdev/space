pub use bevy::prelude::*;

pub mod selection;
pub use selection::*;

pub mod setup;
pub use setup::*;

pub mod markers {
    use bevy::prelude::*;
    use bevy_ecs_markers::EntityMarker;

    #[derive(Component)]
    pub struct MainCamera3d;

    #[derive(Component)]
    pub struct CubemapCamera3d;

    #[derive(EntityMarker)]
    #[entity_marker(data_name = "SelectedBodyMarker")]
    pub enum SelectedBody {
        CurrentRedirected,
        PreviousRedirected,
        Current,
        Previous,
    }

    #[derive(EntityMarker)]
    #[entity_marker(data_name = "FocusedBodyMarker")]
    pub struct FocusedBody;
}

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        use bevy_ecs_markers::InitMarkerData;
        use bevy_mod_raycast::{DefaultRaycastingPlugin, RaycastSystem};

        app.add_plugin(DefaultRaycastingPlugin::<SelectionRaycastSet>::default());

        app.world.init_markerdata::<markers::SelectedBody>();
        app.world.init_markerdata::<markers::FocusedBody>();

        app.add_event::<DeselectionEvent>();
        app.add_event::<SelectionEvent>();

        {
            use setup::systems::*;

            app.add_startup_system(insert_resources)
                .add_startup_system(spawn_entities.at_end());
        }

        {
            use selection::systems::*;

            app.add_system_to_stage(
                CoreStage::First,
                update_raycast_with_cursor.before(RaycastSystem::BuildRays::<SelectionRaycastSet>),
            );
            app.add_system(selection_raycast_update);
            app.add_system(deselect_previous_body.after(selection_raycast_update));
            app.add_system(select_current_body.after(deselect_previous_body));
        }
    }
}
