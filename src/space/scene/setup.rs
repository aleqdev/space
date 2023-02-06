pub mod systems {
    use bevy::{pbr::NotShadowCaster, prelude::*, render::view::RenderLayers};
    use bevy_polyline::prelude::{Polyline, PolylineMaterial};

    use crate::space::{
        display::{
            BodyTrailRef, CameraScale, RealisticView, RelativeLightIntensivity,
            RelativeWorldOffset, RelativeWorldScale, SchematicView, StarMaterial,
        },
        scene::{markers::CubemapCamera3d, SelectionTargetRedirect},
        simulation::SpaceSimulation,
    };

    pub fn insert_resources(world: &mut World) {
        use crate::space::{
            controls::camera::CameraControlSensitivity,
            simulation::{SpaceBodyVec, SpaceSimulationParams},
        };

        world.insert_resource(SpaceSimulation {
            bodies: SpaceBodyVec::new(),
            time: Default::default(),
            percision_table: Default::default(),
            G: 6.67e-11,
        });

        world.insert_resource(SpaceSimulationParams {
            speed: 86400.0 * 1.0,
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
        mut star_materials: ResMut<Assets<StarMaterial>>,
        mut polylines: ResMut<Assets<Polyline>>,
        mut polyline_materials: ResMut<Assets<PolylineMaterial>>,
        mut simulation: ResMut<SpaceSimulation>,
        camera_scale: Res<CameraScale>,
        asset_server: Res<AssetServer>,
    ) {
        use crate::space::{
            controls::camera::UnconstrainedOrbit,
            display::BodyRef,
            scene::{markers::MainCamera3d, SelectionRaycastSet},
            simulation::SpaceBody,
        };
        use bevy::{
            core_pipeline::{bloom::BloomSettings, clear_color::ClearColorConfig},
            math::DVec3,
        };
        use bevy_dolly::prelude::*;
        use bevy_mod_raycast::{RaycastMesh, RaycastSource};
        use bevy_polyline::prelude::PolylineBundle;
        use ringbuffer::AllocRingBuffer;

        let mut push_body = |position, velocity, mass, radius| {
            simulation.bodies.push(SpaceBody {
                position,
                velocity,
                mass,
                radius,
                trail: AllocRingBuffer::with_capacity(512),
            });
        };

        let mut add_polyline = || {
            polylines.add(Polyline {
                vertices: Vec::with_capacity(1024),
            })
        };

        let mut add_polyline_entity = |commands: &mut Commands, polyline, mut color: Color| {
            color.set_a(0.1);
            color.set_r((color.r() * 4.5).min(1.0));
            color.set_g((color.g() * 4.5).min(1.0));
            color.set_b((color.b() * 4.5).min(1.0));
            commands
                .spawn(PolylineBundle {
                    polyline,
                    material: polyline_materials.add(PolylineMaterial {
                        width: 2.0,
                        color,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .id()
        };

        commands.insert_resource(AmbientLight {
            brightness: 0.015,
            ..default()
        });

        let uv_sphere_sectors = 128;

        // spawn sun
        let i = 0;
        let color = Color::YELLOW;
        let radius = 695700e3;
        let mass = 1.989e30;
        let position = DVec3::ZERO;
        let velocity = DVec3::ZERO;

        push_body(position, velocity, mass, radius);

        let polyline = add_polyline();

        let polyline_entity = add_polyline_entity(&mut commands, polyline, color);

        commands
            .spawn((
                BodyRef(i),
                BodyTrailRef(polyline_entity),
                SpatialBundle {
                    transform: Transform::from_translation(Vec3::new(
                        (position.x * camera_scale.scale) as f32,
                        (position.y * camera_scale.scale) as f32,
                        (position.z * camera_scale.scale) as f32,
                    )),
                    ..default()
                },
            ))
            .with_children(|anchor| {
                anchor.spawn((
                    MaterialMeshBundle {
                        mesh: meshes.add(
                            shape::Icosphere {
                                radius: 1.0,
                                subdivisions: 2,
                            }
                            .into(),
                        ),
                        material: materials.add(StandardMaterial {
                            unlit: true,
                            base_color: Color::GRAY,
                            ..default()
                        }),
                        transform: Transform::from_scale(Vec3::splat(
                            (radius * camera_scale.scale) as f32,
                        )),
                        ..default()
                    },
                    RaycastMesh::<SelectionRaycastSet>::default(),
                    SelectionTargetRedirect(anchor.parent_entity()),
                    SchematicView,
                ));
                anchor.spawn((
                    PointLightBundle {
                        point_light: PointLight {
                            color: Color::WHITE,
                            intensity: 200.0
                                / camera_scale.scale as f32
                                / camera_scale.scale as f32,
                            range: 1e8,
                            radius: radius as f32,
                            shadows_enabled: true,
                            ..default()
                        },
                        ..default()
                    },
                    RelativeLightIntensivity(200.0 / camera_scale.scale / camera_scale.scale),
                ));
                anchor.spawn((
                    MaterialMeshBundle {
                        mesh: meshes.add(
                            shape::UVSphere {
                                radius: 1.0,
                                sectors: uv_sphere_sectors,
                                stacks: uv_sphere_sectors / 2,
                            }
                            .into(),
                        ),
                        material: star_materials.add(StarMaterial {
                            primary_color: Color::rgb(8.0 * 4.0, 8.0 * 4.0, 0.0),
                            secondary_color: Color::rgb(8.0 * 4.0, 5.2 * 4.0, 0.0),
                            ..default()
                        }),
                        transform: Transform::from_scale(Vec3::splat(
                            (radius * camera_scale.scale) as f32,
                        )),
                        ..default()
                    },
                    RealisticView,
                ));
            });

        // spawn earth
        let i = 1;
        let color = Color::SEA_GREEN;
        let radius = 6378e3;
        let mass = 5.9724e24;
        let position = DVec3::X * 149.596e9;
        let velocity = -DVec3::Z * 29.78e3;

        push_body(position, velocity, mass, radius);

        let polyline = add_polyline();

        let polyline_entity = add_polyline_entity(&mut commands, polyline, color);

        commands
            .spawn((
                BodyRef(i),
                BodyTrailRef(polyline_entity),
                SpatialBundle {
                    transform: Transform::from_translation(Vec3::new(
                        (position.x * camera_scale.scale) as f32,
                        (position.y * camera_scale.scale) as f32,
                        (position.z * camera_scale.scale) as f32,
                    )),
                    ..default()
                },
            ))
            .with_children(|anchor| {
                anchor.spawn((
                    MaterialMeshBundle {
                        mesh: meshes.add(
                            shape::Icosphere {
                                radius: 1.0,
                                subdivisions: 2,
                            }
                            .into(),
                        ),
                        material: materials.add(StandardMaterial {
                            unlit: true,
                            base_color: Color::GRAY,
                            ..default()
                        }),
                        transform: Transform::from_scale(Vec3::splat(
                            (radius * camera_scale.scale) as f32,
                        )),
                        ..default()
                    },
                    RaycastMesh::<SelectionRaycastSet>::default(),
                    SelectionTargetRedirect(anchor.parent_entity()),
                    SchematicView,
                ));
                anchor.spawn((
                    MaterialMeshBundle {
                        mesh: asset_server.load("glb/sphereUV.glb#Mesh0/Primitive0"),
                        material: materials.add(StandardMaterial {
                            base_color_texture: Some(
                                asset_server.load("textures/earth_base_color.jpg"),
                            ),
                            perceptual_roughness: 1.0,
                            reflectance: 0.0,
                            metallic: 0.0,
                            ..default()
                        }),
                        transform: Transform::from_scale(Vec3::splat(
                            (radius * camera_scale.scale) as f32,
                        ))
                        .with_rotation(Quat::from_rotation_x(0.40840704497)),
                        ..default()
                    },
                    RealisticView,
                ));
            });

        // spawn moon
        let i = 2;
        let color = Color::ALICE_BLUE;
        let radius = 1737.4e3;
        let mass = 7.342e22;
        let position = DVec3::X * (149.596e9 - 385e6);
        let velocity = -DVec3::Z * (29.78e3 - 1.022e3);

        push_body(position, velocity, mass, radius);

        let polyline = add_polyline();

        let polyline_entity = add_polyline_entity(&mut commands, polyline, color);

        commands
            .spawn((
                BodyRef(i),
                BodyTrailRef(polyline_entity),
                SpatialBundle {
                    transform: Transform::from_translation(Vec3::new(
                        (position.x * camera_scale.scale) as f32,
                        (position.y * camera_scale.scale) as f32,
                        (position.z * camera_scale.scale) as f32,
                    )),
                    ..default()
                },
            ))
            .with_children(|anchor| {
                anchor.spawn((
                    MaterialMeshBundle {
                        mesh: meshes.add(
                            shape::Icosphere {
                                radius: 1.0,
                                subdivisions: 2,
                            }
                            .into(),
                        ),
                        material: materials.add(StandardMaterial {
                            unlit: true,
                            base_color: Color::GRAY,
                            ..default()
                        }),
                        transform: Transform::from_scale(Vec3::splat(
                            (radius * camera_scale.scale) as f32,
                        )),
                        ..default()
                    },
                    RaycastMesh::<SelectionRaycastSet>::default(),
                    SelectionTargetRedirect(anchor.parent_entity()),
                    SchematicView,
                ));
                anchor.spawn((
                    MaterialMeshBundle {
                        mesh: asset_server.load("glb/sphereUV.glb#Mesh0/Primitive0"),
                        material: materials.add(StandardMaterial {
                            base_color_texture: Some(
                                asset_server.load("textures/moon_base_color.jpg"),
                            ),
                            perceptual_roughness: 1.0,
                            reflectance: 0.0,
                            metallic: 0.0,
                            ..default()
                        }),
                        transform: Transform::from_scale(Vec3::splat(
                            (radius * camera_scale.scale) as f32,
                        ))
                        .with_rotation(Quat::from_rotation_x(0.40840704497)),
                        ..default()
                    },
                    RealisticView,
                ));
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
            BloomSettings {
                intensity: 0.002,
                scale: 0.5,
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
            BloomSettings {
                intensity: 0.002,
                scale: 0.5,
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
    }
}
