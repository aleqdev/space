use bevy::math::DVec3;

pub const BODY: fn() -> super::SolarSystemBodyBuilder = || super::SolarSystemBodyBuilder {
    radius: 3389.92e3,
    mass: 6.4171e23,
    position: 1e3
        * DVec3::new(
            -1.784469356035438E+08,
            1.127798260336012E+06,
            -1.542321107589649E+08,
        ),
    velocity: 1e3
        * DVec3::new(
            1.683941650091127E+01,
            -7.519462098882466E-01,
            -1.618915624854859E+01,
        ),
    material: super::SolarSystemBodyBuilderMaterial::TexturePath("textures/mars_base_color.jpg"),
    rotation: Default::default(),
    rotation_rate: 0.0000708822,
};
