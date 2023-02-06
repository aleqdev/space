use bevy::math::DVec3;

pub const BODY: fn() -> super::SolarSystemBodyBuilder = || super::SolarSystemBodyBuilder {
    radius: 25362e3,
    mass: 86.813e24,
    position: 1e3
        * DVec3::new(
            2.164327314689041E+09,
            -2.059187029480827E+07,
            2.005198925386528E+09,
        ),
    velocity: 1e3
        * DVec3::new(
            -4.678235375068675E+00,
            7.820819317412120E-02,
            4.678325969429742E+00,
        ),
    material: super::SolarSystemBodyBuilderMaterial::TexturePath("textures/uranus_base_color.jpg"),
    rotation: Default::default(),
    rotation_rate: -0.000101237,
};
