use bevy::math::DVec3;

pub const BODY: fn() -> super::SolarSystemBodyBuilder = || super::SolarSystemBodyBuilder {
    radius: 6051.84e3,
    mass: 48.685e23,
    position: 1e3
        * DVec3::new(
            7.004305232813431E+07,
            -2.975190887366954E+06,
            8.174535532481459E+07,
        ),
    velocity: 1e3
        * DVec3::new(
            -2.642892528261364E+01,
            1.840059141150036E+00,
            2.294240404862722E+01,
        ),
    material: super::SolarSystemBodyBuilderMaterial::TexturePath("textures/venus_base_color.jpg"),
    rotation: Default::default(),
    rotation_rate: -0.00000029924,
};
