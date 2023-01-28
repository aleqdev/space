use bevy::prelude::*;

pub mod setup;
pub use setup::*;

pub mod simulation_sync;
pub use simulation_sync::*;

pub mod selection;
pub use selection::*;

pub mod camera;
pub use camera::*;

pub mod unconstrained_orbit_driver;
pub use unconstrained_orbit_driver::*;

pub mod simulation;
pub use simulation::*;

pub mod body_ref;
pub use body_ref::*;

pub struct SpacePlugin;

impl Plugin for SpacePlugin {
    fn build(&self, app: &mut App) {
        use bevy_dolly::prelude::DollyComponent;
        use bevy_ecs_markers::InitMarkerData;
        use bevy_mod_raycast::RaycastSystem;

        app.add_plugin(bevy_mod_raycast::DefaultRaycastingPlugin::<
            SelectionRaycastSet,
        >::default());
        app.add_plugin(bevy_prototype_lyon::prelude::ShapePlugin);
        app.add_plugin(bevy_polyline::PolylinePlugin);

        app.add_dolly_component(MainCamera3d);

        app.world.init_markerdata::<SelectedBody>();
        app.world.init_markerdata::<FocusedBody>();

        app.insert_resource(SpaceSimulation {
            positions: Default::default(),
            velocities: Default::default(),
            masses: Default::default(),
            time: Default::default(),
            trails: Default::default(),
            percision_table: Default::default(),
            G: 6.67e-11,
        });
        app.insert_resource(CameraScale {
            scale: 1.0 / (147.1 * 1_000_000.0 * 1000.0 / 2.0),
        });
        app.insert_resource(SpaceSimulationParams { speed: 86400.0 * 20.0 });

        app.add_startup_system(setup::setup);
        app.add_system(simulation_take_step);
        app.add_system(sync_with_simulation.after(simulation_take_step));

        app.add_system_to_stage(
            CoreStage::First,
            update_raycast_with_cursor.before(RaycastSystem::BuildRays::<SelectionRaycastSet>),
        );
        app.add_event::<DeselectionEvent>();
        app.add_event::<SelectionEvent>();
        app.add_system(selection_raycast_update); //.add_system(intersection_remove);
        app.add_system(deselect_previous_body.after(selection_raycast_update));
        app.add_system(select_current_body.after(deselect_previous_body));
        app.add_system(focus_camera_on_click.after(select_current_body));

        app.insert_resource(CameraControlSensitivity {
            zoom: 1.359,
            orbit: Vec2::splat(2.0),
        });

        app.add_system(camera_zoom).add_system(camera_orbit);

        app.add_system(print_simulation_energy);
    }
}
