use bevy::math::DVec3;

pub const BODY: fn() -> super::SolarSystemBodyBuilder = || super::SolarSystemBodyBuilder {
    radius: 1737.53e3,
    mass: 7.349e22,
    position: 1e3
        * DVec3::new(
                4.718371657374899E+07,
            2.374383445678651E+04,
            1.394722949466201E+08,
        ),
    velocity: 1e3
        * DVec3::new(
                -2.774199319873983E+01,
            -9.910133319380021E-02,
            9.063751292606270E+00,
        ),
    material: super::SolarSystemBodyBuilderMaterial::TexturePath("textures/moon_base_color.jpg"),
    rotation: Default::default(),
    rotation_rate: 0.0000026617,
};
