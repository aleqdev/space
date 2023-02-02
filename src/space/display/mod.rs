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

pub struct DisplayPlugin;

impl Plugin for DisplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_state_to_stage(CoreStage::PostUpdate, ViewMode::Realistic);

        {
            use bevy::transform::{TransformSystem, transform_propagate_system};
            use schematic::systems::*;

            app.add_system_set_to_stage(
                CoreStage::PostUpdate,
                SystemSet::on_enter(ViewMode::Schematic).with_system(update_bodies_on_enter),
            );
            app.add_system_set_to_stage(
                CoreStage::PostUpdate,
                SystemSet::on_update(ViewMode::Schematic).after(TransformSystem::TransformPropagate)
                    .with_system(update_bodies)
                    .with_system(transform_propagate_system.after(update_bodies)),
            );
        }

        {
            use realistic::systems::*;

            app.add_system_set_to_stage(
                CoreStage::PostUpdate,
                SystemSet::on_enter(ViewMode::Realistic).with_system(update_bodies_on_enter),
            );
        }

        {
            use crate::space::simulation::space_simulation::systems::simulation_take_step;
            use sync::systems::*;

            app.add_system(sync_with_simulation.after(simulation_take_step));
        }

        {
            use view_mode::systems::*;

            app.add_system(toggle_mode);
        }
    }
}
