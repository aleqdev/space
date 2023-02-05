use bevy::{app::PluginGroupBuilder, prelude::*};

pub mod controls;
pub mod display;
pub mod scene;
pub mod simulation;

pub struct SpacePlugins;

impl PluginGroup for SpacePlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<SpacePlugins>()
            .add(scene::ScenePlugin)
            .add(simulation::SpaceSimulationPlugin)
            .add(display::DisplayPlugin)
            .add(controls::ControlsPlugin)
            .add(bevy_prototype_lyon::prelude::ShapePlugin)
            .add(bevy_polyline::PolylinePlugin)
            .add(noisy_bevy::NoisyShaderPlugin)
            .build()
    }
}
