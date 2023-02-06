use bevy::math::DVec3;

pub const BODY: fn() -> super::SolarSystemBodyBuilder = || super::SolarSystemBodyBuilder {
    radius: 6371.01e3,
    mass: 5.97219e24,
    position: 1e3
        * DVec3::new(
            4.740462482051019E+07,
            1.827485543281585E+04,
            1.397549779170149E+08,
        ),
    velocity: 1e3
        * DVec3::new(
            -2.862111509639989E+01,
            1.111602822769786E-03,
            9.711937011665990E+00,
        ),
    material: super::SolarSystemBodyBuilderMaterial::TexturePath("textures/earth_base_color.jpg"),
    rotation: Default::default(),
    rotation_rate: 0.00007292115,
};
