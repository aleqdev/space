use bevy::prelude::*;

pub mod body_ref;
pub use body_ref::*;

pub mod camera;
pub use camera::*;

pub mod sync;
pub use sync::*;

pub mod realistic;
pub use realistic::*;

pub mod schematic;
pub use schematic::*;

pub mod view_mode;
pub use view_mode::*;

pub mod relative_world_scale;
pub use relative_world_scale::*;

pub mod relative_world_offset;
pub use relative_world_offset::*;

pub mod body_trail;
pub use body_trail::*;

pub mod star_material;
pub use star_material::*;

pub mod selection;
pub use selection::*;

pub mod custom_params;

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
pub enum DisplayStage {
    Sync,
}

pub struct DisplayPlugin;

#[derive(SystemLabel)]
pub enum DisplayStageSyncSystems {
    TransformPropagate,
    PreDisplayLogic,
}

#[allow(unused_labels)]
impl Plugin for DisplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<StarMaterial>::default());

        app.add_stage_after(
            CoreStage::Update,
            DisplayStage::Sync,
            SystemStage::parallel(),
        );

        app.add_state_to_stage(DisplayStage::Sync, ViewMode::Realistic);

        'add_toggle_mode: {
            use view_mode::systems::*;

            app.add_system(toggle_mode);
        }

        'add_cubemap_camera_sync: {
            use crate::space::controls::camera::systems::orbit;
            use camera::systems::*;

            app.add_system(sync_cubemap_camera.after(orbit));
        }

        'add_pre_display_logic: {
            use body_trail::systems::*;
            use relative_world_offset::systems::*;
            use relative_world_scale::systems::*;

            app.add_system_set_to_stage(
                DisplayStage::Sync,
                SystemSet::new()
                    .with_system(update_world_scale)
                    .with_system(extract_relative_world_offset)
                    .with_system(change_trail_anchor)
                    .label(DisplayStageSyncSystems::PreDisplayLogic),
            );
        }

        'add_early_transform_propagation: {
            use bevy::transform::transform_propagate_system;

            app.add_system_to_stage(
                DisplayStage::Sync,
                transform_propagate_system
                    .label(DisplayStageSyncSystems::TransformPropagate)
                    .after(DisplayStageSyncSystems::PreDisplayLogic),
            );
        }

        'add_bodies_sync_with_simulation: {
            use sync::systems::*;

            app.add_system_to_stage(
                DisplayStage::Sync,
                sync_with_simulation.after(DisplayStageSyncSystems::PreDisplayLogic),
            );
        }

        /*{
            use body_trail::systems::*;

            app.add_system(dissolve_extreme_trails);
        }*/

        'add_simulation_polylines_logic: {
            use body_trail::systems::*;

            app.add_system_to_stage(DisplayStage::Sync, extract_positions_from_simulations);

            app.add_system_to_stage(
                DisplayStage::Sync,
                sync_polylines_to_trails
                    .after(extract_positions_from_simulations)
                    .after(DisplayStageSyncSystems::PreDisplayLogic),
            );
        }

        'add_schematic_display: {
            use schematic::systems::*;

            app.add_system_set_to_stage(
                DisplayStage::Sync,
                SystemSet::on_enter(ViewMode::Schematic)
                    .after(DisplayStageSyncSystems::PreDisplayLogic)
                    .with_system(update_bodies_on_enter),
            );
            app.add_system_set_to_stage(
                DisplayStage::Sync,
                SystemSet::on_update(ViewMode::Schematic)
                    .after(DisplayStageSyncSystems::TransformPropagate)
                    .with_system(update_bodies),
            );
        }

        'add_realistic_display: {
            use realistic::systems::*;

            app.add_system_set_to_stage(
                DisplayStage::Sync,
                SystemSet::on_enter(ViewMode::Realistic)
                    .after(DisplayStageSyncSystems::PreDisplayLogic)
                    .with_system(update_bodies_on_enter),
            );
            app.add_system_set_to_stage(
                DisplayStage::Sync,
                SystemSet::on_update(ViewMode::Realistic)
                    .after(DisplayStageSyncSystems::PreDisplayLogic)
                    .with_system(update_bodies),
            );
        }

        'add_selection_rects_sync: {
            use selection::systems::*;

            app.add_system_to_stage(
                DisplayStage::Sync,
                display_selection_rects.after(DisplayStageSyncSystems::TransformPropagate),
            );
        }
    }
}
