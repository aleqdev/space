use bevy::math::DVec3;

pub const BODY: fn() -> super::SolarSystemBodyBuilder = || super::SolarSystemBodyBuilder {
    radius: 24624e3,
    mass: 102.409e24,
    position: 1e3
        * DVec3::new(
            4.429998289679761E+09,
            -8.922274898704058E+07,
            -6.250174401654940E+08,
        ),
    velocity: 1e3
        * DVec3::new(
            7.234074820972649E-01,
            -1.275857370399349E-01,
            5.414933493668496E+00,
        ),
    material: super::SolarSystemBodyBuilderMaterial::TexturePath("textures/neptune_base_color.jpg"),
    rotation: Default::default(),
    rotation_rate: 0.000108338,
};
