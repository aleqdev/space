use bevy::prelude::*;

pub mod body_ref;
pub use body_ref::*;

pub mod body_trail_ref;
pub use body_trail_ref::*;

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

pub struct DisplayPlugin;

impl Plugin for DisplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<StarMaterial>::default());

        app.add_state_to_stage(CoreStage::PostUpdate, ViewMode::Realistic);

        {
            use crate::space::simulation::systems::simulation_take_step;
            use relative_world_offset::systems::*;

            app.add_system(extract_relative_world_offset.after(simulation_take_step));
        }

        {
            use bevy::transform::{transform_propagate_system, TransformSystem};
            use schematic::systems::*;

            app.add_system_set_to_stage(
                CoreStage::PostUpdate,
                SystemSet::on_enter(ViewMode::Schematic).with_system(update_bodies_on_enter),
            );
            app.add_system_set_to_stage(
                CoreStage::PostUpdate,
                SystemSet::on_update(ViewMode::Schematic)
                    .after(TransformSystem::TransformPropagate)
                    .with_system(update_bodies)
                    .with_system(transform_propagate_system.after(update_bodies)),
            );
        }

        {
            use bevy::transform::{transform_propagate_system, TransformSystem};
            use realistic::systems::*;

            app.add_system_set_to_stage(
                CoreStage::PostUpdate,
                SystemSet::on_enter(ViewMode::Realistic).with_system(update_bodies_on_enter),
            );
            app.add_system_set_to_stage(
                CoreStage::PostUpdate,
                SystemSet::on_update(ViewMode::Realistic)
                    .after(TransformSystem::TransformPropagate)
                    .with_system(update_bodies)
                    .with_system(transform_propagate_system.after(update_bodies)),
            );
        }

        {
            use crate::space::simulation::space_simulation::systems::simulation_take_step;
            use relative_world_scale::systems::*;
            use sync::systems::*;

            app.add_system(
                sync_with_simulation
                    .after(simulation_take_step)
                    .after(update_world_scale),
            );
        }

        {
            use view_mode::systems::*;

            app.add_system(toggle_mode);
        }

        {
            use relative_world_scale::systems::*;

            app.add_system(update_world_scale);
        }

        {
            use body_trail::systems::*;

            app.add_system(dissolve_extreme_trails);
        }

        {
            use crate::space::controls::camera::systems::orbit;
            use camera::systems::*;

            app.add_system(sync_cubemap_camera.after(orbit));
        }
    }
}
