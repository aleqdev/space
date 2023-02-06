use bevy::{math::DVec3, prelude::Color};

use crate::space::display::StarMaterial;

pub const BODY: fn() -> super::SolarSystemBodyBuilder = || super::SolarSystemBodyBuilder {
    radius: 695700e3,
    mass: 1988500e24,
    position: 1e3
        * DVec3::new(
            -1.268391475744417E+06,
            2.524786977603121E+04,
            5.371680130843105E+05,
        ),
    velocity: 1e3
        * DVec3::new(
            -6.397134144538324E-03,
            2.646427706603236E-04,
            -1.447281778805041E-02,
        ),
    material: super::SolarSystemBodyBuilderMaterial::Star(StarMaterial {
        primary_color: Color::rgb(8.0 * 4.0, 8.0 * 4.0, 0.0),
        secondary_color: Color::rgb(8.0 * 4.0, 5.2 * 4.0, 0.0),
        ..Default::default()
    }),
    rotation: Default::default(),
    rotation_rate: Default::default(),
};
