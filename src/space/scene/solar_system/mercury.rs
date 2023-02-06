use bevy::math::DVec3;

pub const BODY: fn() -> super::SolarSystemBodyBuilder = || super::SolarSystemBodyBuilder {
    radius: 2440e3,
    mass: 3.302e23,
    position: 1e3
        * DVec3::new(
            -1.652217971561605E+07,
            -4.132618246765319E+06,
            -6.746578653994957E+07,
        ),
    velocity: 1e3
        * DVec3::new(
            3.774999560708947E+01,
            -4.133773678579869E+00,
            -8.222465491717815E+00,
        ),
    material: super::SolarSystemBodyBuilderMaterial::TexturePath("textures/mercury_base_color.jpg"),
    rotation: Default::default(),
    rotation_rate: 0.00000124001,
};
