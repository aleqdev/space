use bevy::math::DVec3;

pub const BODY: fn() -> super::SolarSystemBodyBuilder = || super::SolarSystemBodyBuilder {
    radius: 58232e3,
    mass: 5.6834e26,
    position: 1e3
        * DVec3::new(
            1.023901704845962E+09,
            -2.210094444566405E+07,
            -1.073440945321824E+09,
        ),
    velocity: 1e3
        * DVec3::new(
            6.449172166058710E+00,
            -3.729601512076588E-01,
            6.648045617832159E+00,
        ),
    material: super::SolarSystemBodyBuilderMaterial::TexturePath("textures/saturn_base_color.jpg"),
    rotation: Default::default(),
    rotation_rate: 0.0334979 / (24.0 * 60.0 * 60.0),
};
