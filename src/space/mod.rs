use bevy::prelude::*;

pub mod body_bundle;
pub use body_bundle::*;

pub mod setup;
pub use setup::*;

pub mod simulation;
pub use simulation::*;

pub mod selection;
pub use selection::*;

pub mod camera;
pub use camera::*;

pub mod target_transform;
pub use target_transform::*;

pub struct SpacePlugin;

impl Plugin for SpacePlugin {
    fn build(&self, app: &mut App) {
        use bevy_mod_raycast::RaycastSystem;
        use bevy_ecs_markers::InitMarkerData;

        app.add_plugin(bevy_mod_raycast::DefaultRaycastingPlugin::<SelectionRaycastSet>::default());
        app.add_plugin(bevy_prototype_lyon::prelude::ShapePlugin);
        app.add_plugin(bevy_polyline::PolylinePlugin);

        app.world.init_markerdata::<SelectedBody>();
        app.world.init_markerdata::<FocusedBody>();

        app.add_startup_system(setup::setup);
        app.add_system(update);

        app.add_system_to_stage(
            CoreStage::First,
            update_raycast_with_cursor.before(RaycastSystem::BuildRays::<SelectionRaycastSet>),
        );
        app.add_event::<DeselectionEvent>();
        app.add_event::<SelectionEvent>();
        app.add_system(selection_raycast_update);//.add_system(intersection_remove);
        app.add_system(deselect_previous_body.after(selection_raycast_update));
        app.add_system(select_current_body.after(deselect_previous_body));
        app.add_system(focus_camera_on_click.after(select_current_body));
        app.add_system(align_camera_with_focus.after(focus_camera_on_click));

        app.insert_resource(CameraControlSensitivity {
            zoom: 0.359,
            orbit: Vec2::splat(2.0)
        });

        app.add_system(approach_target_transform_system);

        app.add_system(camera_zoom);//.add_system(camera_orbit);
    }
}
