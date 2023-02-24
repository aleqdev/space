pub mod systems {
    use bevy::{pbr::NotShadowCaster, prelude::*, render::view::RenderLayers};

    use crate::space::{
        display::{
            CameraScale, PrimarySelectionRectMarker, RelativeWorldOffset, RelativeWorldScale,
            SecondarySelectionRectMarker,
        },
        scene::markers::CubemapCamera3d,
        simulation::SpaceSimulation,
    };

    pub fn insert_resources(world: &mut World) {
        use crate::space::{
            controls::camera::CameraControlSensitivity, simulation::SpaceSimulationParams,
        };

        world.insert_resource(SpaceSimulation {
            G: 6.67e-11,
            ..default()
        });

        world.insert_resource(SpaceSimulationParams {
            speed: 86400.0 * 1.0 * 2.0,
        });

        world.insert_resource(CameraScale {
            scale: 1.0 / (147.1 * 1_000_000.0 * 1000.0),
        });

        world.insert_resource(RelativeWorldScale { scale: 1.0 });

        world.insert_resource(RelativeWorldOffset::default());

        world.insert_resource(CameraControlSensitivity {
            zoom: 1.359,
            orbit: Vec2::splat(2.0),
        });
    }

    pub fn spawn_entities(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
        mut test: EventWriter<crate::space::nasa_horizons::SpawnNasaBodyRequest>,
        camera_scale: Res<CameraScale>,
        asset_server: Res<AssetServer>,
        mut simulation: ResMut<SpaceSimulation>,
        mut ev: EventWriter<crate::space::nasa_horizons::NasaBodyAddition>,
    ) {
        simulation.bodies.insert(
            "a12".into(),
            crate::space::simulation::SpaceBody {
                position: bevy::math::DVec3::X * 1e10,
                velocity: bevy::math::DVec3::Y * 1e3,
                mass: 1e28,
                radius: 1e1,
            },
        );

        ev.send(crate::space::nasa_horizons::NasaBodyAddition {
            date: chrono::Utc::now(),
            name: "a12".into(),
            position: bevy::math::DVec3::X * 1e10,
            velocity: bevy::math::DVec3::Y * 1e3,
            radius: 1e1,
            mass: 1e28,
            material: crate::space::nasa_horizons::SpaceBodyKnownDetailsMaterial::TexturePath(
                "textures/asteroid.jpg",
            ),
            rotation: Default::default(),
            rotation_rate: Default::default(),
        });
        simulation.bodies.insert(
            "a13".into(),
            crate::space::simulation::SpaceBody {
                position: bevy::math::DVec3::ZERO,
                velocity: bevy::math::DVec3::Y * 1e2,
                mass: 1e17,
                radius: 1e1,
            },
        );

        ev.send(crate::space::nasa_horizons::NasaBodyAddition {
            date: chrono::Utc::now(),
            name: "a13".into(),
            position: bevy::math::DVec3::ZERO,
            velocity: bevy::math::DVec3::Y * 1e2,
            radius: 1e1,
            mass: 1e17,
            material: crate::space::nasa_horizons::SpaceBodyKnownDetailsMaterial::TexturePath(
                "textures/asteroid.jpg",
            ),
            rotation: Default::default(),
            rotation_rate: Default::default(),
        });
        /*test.send_batch(include!("./MB-2.txt").split(";").map(|name| {
            crate::space::nasa_horizons::SpawnNasaBodyRequest {
                date: chrono::Utc::now(),
                name: name.into(),
            }
        }));*/

        /*test.send_batch("10;199;299;399;499;599;699;799;899;301".split(";").map(|name| {
            crate::space::nasa_horizons::SpawnNasaBodyRequest {
                date: chrono::Utc::now(),
                name: name.into(),
            }
        }));*/

        use crate::space::{
            controls::camera::UnconstrainedOrbit,
            scene::{markers::MainCamera3d, SelectionRaycastSet},
        };
        use bevy::core_pipeline::{bloom::BloomSettings, clear_color::ClearColorConfig};
        use bevy_dolly::prelude::*;
        use bevy_mod_raycast::RaycastSource;

        commands.insert_resource(AmbientLight {
            brightness: 0.015,
            ..default()
        });

        commands.spawn((
            MainCamera3d,
            Rig::builder()
                .with(
                    UnconstrainedOrbit::new()
                        .yaw_degrees(45.0)
                        .pitch_degrees(-30.0),
                )
                .with(Arm::new(Vec3::Z * 8.0))
                .build(),
        ));

        commands.spawn((
            Camera3dBundle {
                camera: Camera {
                    hdr: true,
                    priority: 1,
                    ..default()
                },
                camera_3d: Camera3d {
                    clear_color: ClearColorConfig::None,
                    ..default()
                },
                projection: PerspectiveProjection {
                    near: 0.0000001,
                    far: 1e30,
                    ..default()
                }
                .into(),
                ..default()
            },
            RaycastSource::<SelectionRaycastSet>::new(),
            MainCamera3d,
        ));

        commands.spawn((
            Camera3dBundle {
                camera: Camera {
                    hdr: true,
                    priority: 0,
                    ..default()
                },
                camera_3d: Camera3d {
                    clear_color: ClearColorConfig::Custom(Color::BLACK),
                    ..default()
                },
                projection: PerspectiveProjection {
                    near: 0.0000001,
                    far: 1e30,
                    ..default()
                }
                .into(),
                ..default()
            },
            CubemapCamera3d,
            RenderLayers::layer(1),
        ));

        let cubemap_radius = 1e15 * camera_scale.scale as f32;

        let mut make_cubemap_material = |filename: &str| {
            materials.add(StandardMaterial {
                base_color_texture: Some(asset_server.load(filename)),
                metallic: 0.0,
                perceptual_roughness: 1.0,
                reflectance: 0.0,
                unlit: true,
                base_color: Color::rgb(0.7, 0.7, 0.7),
                ..default()
            })
        };

        commands.spawn((
            PbrBundle {
                mesh: meshes.add(
                    shape::Quad {
                        size: Vec2::splat(cubemap_radius * 2.0),
                        ..default()
                    }
                    .into(),
                ),
                material: make_cubemap_material("textures/cubemap/nx.jpg"),
                transform: Transform::from_translation(Vec3::NEG_X * cubemap_radius)
                    .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
                ..default()
            },
            NotShadowCaster,
            RenderLayers::layer(1),
        ));

        commands.spawn((
            PbrBundle {
                mesh: meshes.add(
                    shape::Quad {
                        size: Vec2::splat(cubemap_radius * 2.0),
                        ..default()
                    }
                    .into(),
                ),
                material: make_cubemap_material("textures/cubemap/px.jpg"),
                transform: Transform::from_translation(Vec3::X * cubemap_radius)
                    .with_rotation(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2)),
                ..default()
            },
            NotShadowCaster,
            RenderLayers::layer(1),
        ));

        commands.spawn((
            PbrBundle {
                mesh: meshes.add(
                    shape::Quad {
                        size: Vec2::splat(cubemap_radius * 2.0),
                        ..default()
                    }
                    .into(),
                ),
                material: make_cubemap_material("textures/cubemap/ny.jpg"),
                transform: Transform::from_translation(Vec3::NEG_Y * cubemap_radius)
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                ..default()
            },
            NotShadowCaster,
            RenderLayers::layer(1),
        ));

        commands.spawn((
            PbrBundle {
                mesh: meshes.add(
                    shape::Quad {
                        size: Vec2::splat(cubemap_radius * 2.0),
                        ..default()
                    }
                    .into(),
                ),
                material: make_cubemap_material("textures/cubemap/py.jpg"),
                transform: Transform::from_translation(Vec3::Y * cubemap_radius)
                    .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
                ..default()
            },
            NotShadowCaster,
            RenderLayers::layer(1),
        ));

        commands.spawn((
            PbrBundle {
                mesh: meshes.add(
                    shape::Quad {
                        size: Vec2::splat(cubemap_radius * 2.0),
                        ..default()
                    }
                    .into(),
                ),
                material: make_cubemap_material("textures/cubemap/nz.jpg"),
                transform: Transform::from_translation(Vec3::Z * cubemap_radius).with_rotation(
                    Quat::from_euler(
                        EulerRot::XYZ,
                        std::f32::consts::PI,
                        0.0,
                        std::f32::consts::PI,
                    ),
                ),
                ..default()
            },
            NotShadowCaster,
            RenderLayers::layer(1),
        ));

        commands.spawn((
            PbrBundle {
                mesh: meshes.add(
                    shape::Quad {
                        size: Vec2::splat(cubemap_radius * 2.0),
                        ..default()
                    }
                    .into(),
                ),
                material: make_cubemap_material("textures/cubemap/pz.jpg"),
                transform: Transform::from_translation(Vec3::NEG_Z * cubemap_radius),
                ..default()
            },
            NotShadowCaster,
            RenderLayers::layer(1),
        ));

        commands
            .spawn((
                Camera3dBundle {
                    camera: Camera {
                        priority: 2,
                        hdr: true,
                        ..default()
                    },
                    camera_3d: Camera3d {
                        clear_color: ClearColorConfig::None,
                        ..default()
                    },
                    projection: OrthographicProjection::default().into(),
                    ..default()
                },
                RenderLayers::layer(2),
                BloomSettings {
                    intensity: 0.002,
                    scale: 0.5,
                    ..default()
                },
            ))
            .insert(SpatialBundle::default())
            .with_children(|commands| {
                commands.spawn((
                    PrimarySelectionRectMarker,
                    PbrBundle {
                        mesh: meshes.add(
                            shape::Plane {
                                size: 32.0,
                                ..default()
                            }
                            .into(),
                        ),
                        material: materials.add(StandardMaterial {
                            base_color_texture: Some(
                                asset_server.load("textures/selection_texture.png"),
                            ),
                            base_color: Color::AZURE,
                            unlit: true,
                            alpha_mode: AlphaMode::Mask(0.5),
                            ..default()
                        }),
                        transform: Transform::from_rotation(Quat::from_euler(
                            EulerRot::XYZ,
                            std::f32::consts::FRAC_PI_2,
                            0.0,
                            0.0,
                        )),

                        ..default()
                    },
                    NotShadowCaster,
                    RenderLayers::layer(2),
                ));
                commands.spawn((
                    SecondarySelectionRectMarker,
                    PbrBundle {
                        mesh: meshes.add(
                            shape::Plane {
                                size: 32.0,
                                ..default()
                            }
                            .into(),
                        ),
                        material: materials.add(StandardMaterial {
                            base_color_texture: Some(
                                asset_server.load("textures/selection_texture.png"),
                            ),
                            base_color: Color::ORANGE,
                            unlit: true,
                            alpha_mode: AlphaMode::Mask(0.5),
                            ..default()
                        }),
                        transform: Transform::from_xyz(0.0, 0.0, -1.0).with_rotation(
                            Quat::from_euler(EulerRot::XYZ, std::f32::consts::FRAC_PI_2, 0.0, 0.0),
                        ),

                        ..default()
                    },
                    NotShadowCaster,
                    RenderLayers::layer(2),
                ));
            });
    }
}
