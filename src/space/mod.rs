use bevy::{app::PluginGroupBuilder, prelude::*};

pub mod controls;
pub mod display;
pub mod ext;
pub mod nasa_horizons;
pub mod scene;
pub mod simulation;
pub mod ui;

pub struct SpacePlugins;

impl PluginGroup for SpacePlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<SpacePlugins>()
            .add(scene::ScenePlugin)
            .add(simulation::SpaceSimulationPlugin)
            .add(display::DisplayPlugin)
            .add(controls::ControlsPlugin)
            .add(nasa_horizons::NasaHorizonsPlugin)
            .add(ui::SpaceUIPlugin)
            .add(bevy_prototype_lyon::prelude::ShapePlugin)
            .add(bevy_polyline::PolylinePlugin)
            .add(noisy_bevy::NoisyShaderPlugin)
            .add(bevy_debug_text_overlay::OverlayPlugin {
                font_size: 14.0,
                ..default()
            })
            .add(bevy_egui::EguiPlugin)
            .build()
    }
}
