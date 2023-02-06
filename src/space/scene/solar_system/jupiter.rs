use bevy::math::DVec3;

pub const BODY: fn() -> super::SolarSystemBodyBuilder = || super::SolarSystemBodyBuilder {
    radius: 69911e3,
    mass: 189818.722e22,
    position: 1e3
        * DVec3::new(
            6.834876127106260E+08,
            -1.404765975092945E+07,
            -2.997687199651589E+08,
        ),
    velocity: 1e3
        * DVec3::new(
            5.089151911582024E+00,
            -1.660871320893129E-01,
            1.257895162084005E+01,
        ),
    material: super::SolarSystemBodyBuilderMaterial::TexturePath("textures/jupiter_base_color.jpg"),
    rotation: Default::default(),
    rotation_rate: 0.00007292115, // GAS GIANT NEED SHADER
};
