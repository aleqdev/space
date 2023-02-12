pub mod systems {
    use bevy::{pbr::NotShadowCaster, prelude::*, render::view::RenderLayers};
    use bevy_polyline::prelude::{Polyline, PolylineMaterial};

    use crate::space::{
        display::{
            BodyTrailRef, CameraScale, RealisticView, RelativeLightIntensivity,
            RelativeWorldOffset, RelativeWorldScale, SchematicView, SelectionRectMarker,
            StarMaterial,
        },
        scene::{
            markers::CubemapCamera3d, solar_system::SolarSystemBodyBuilderMaterial,
            SelectionTargetRedirect,
        },
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
        use bevy::core_pipeline::{bloom::BloomSettings, clear_color::ClearColorConfig};
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

        let uv_sphere = || asset_server.load::<Mesh, _>("glb/sphereUV.glb#Mesh0/Primitive0");

        for (i, builder) in [
            crate::space::scene::solar_system::sun::BODY(),
            crate::space::scene::solar_system::mercury::BODY(),
            crate::space::scene::solar_system::venus::BODY(),
            crate::space::scene::solar_system::earth::BODY(),
            crate::space::scene::solar_system::mars::BODY(),
            crate::space::scene::solar_system::jupiter::BODY(),
            crate::space::scene::solar_system::saturn::BODY(),
            crate::space::scene::solar_system::uranus::BODY(),
            crate::space::scene::solar_system::neptune::BODY(),
            crate::space::scene::solar_system::moon::BODY(),
        ]
        .into_iter()
        .enumerate()
        {
            let color = Color::GRAY;

            push_body(
                builder.position,
                builder.velocity,
                builder.mass,
                builder.radius,
            );

            let polyline = add_polyline();

            let polyline_entity = add_polyline_entity(&mut commands, polyline, color);

            commands
                .spawn((
                    BodyRef(i),
                    BodyTrailRef(polyline_entity),
                    SpatialBundle {
                        transform: Transform::from_translation(Vec3::new(
                            (builder.position.x * camera_scale.scale) as f32,
                            (builder.position.y * camera_scale.scale) as f32,
                            (builder.position.z * camera_scale.scale) as f32,
                        )),
                        ..default()
                    },
                ))
                .with_children(|anchor| {
                    let make_solid_material =
                        |materials: &mut ResMut<Assets<StandardMaterial>>, path: &str| {
                            materials.add(StandardMaterial {
                                base_color_texture: Some(asset_server.load(path)),
                                perceptual_roughness: 1.0,
                                reflectance: 0.0,
                                metallic: 0.0,
                                ..default()
                            })
                        };

                    let mut make_star_material = |material| star_materials.add(material);

                    let make_schematic_material =
                        |materials: &mut ResMut<Assets<StandardMaterial>>| {
                            materials.add(StandardMaterial {
                                unlit: true,
                                base_color: Color::GRAY,
                                ..default()
                            })
                        };

                    anchor.spawn((
                        MaterialMeshBundle {
                            mesh: meshes.add(
                                shape::Icosphere {
                                    radius: 1.0,
                                    subdivisions: 2,
                                }
                                .into(),
                            ),
                            material: make_schematic_material(&mut materials),
                            transform: Transform::from_scale(Vec3::splat(
                                (builder.radius * camera_scale.scale) as f32,
                            )),
                            ..default()
                        },
                        RaycastMesh::<SelectionRaycastSet>::default(),
                        SelectionTargetRedirect(anchor.parent_entity()),
                        SchematicView,
                    ));
                    if let SolarSystemBodyBuilderMaterial::Star(star_material) = builder.material {
                        anchor.spawn((
                            MaterialMeshBundle {
                                mesh: uv_sphere(),
                                material: make_star_material(star_material),
                                transform: Transform::from_scale(Vec3::splat(
                                    (builder.radius * camera_scale.scale) as f32,
                                )),
                                ..default()
                            },
                            RealisticView,
                        ));
                        anchor.spawn((
                            PointLightBundle {
                                point_light: PointLight {
                                    color: Color::WHITE,
                                    intensity: 200.0
                                        / camera_scale.scale as f32
                                        / camera_scale.scale as f32,
                                    range: 1e8,
                                    radius: builder.radius as f32,
                                    shadows_enabled: true,
                                    ..default()
                                },
                                ..default()
                            },
                            RelativeLightIntensivity(
                                200.0 / camera_scale.scale / camera_scale.scale,
                            ),
                        ));
                    } else if let SolarSystemBodyBuilderMaterial::TexturePath(path) =
                        builder.material
                    {
                        anchor.spawn((
                            MaterialMeshBundle {
                                mesh: uv_sphere(),
                                material: make_solid_material(&mut materials, path),
                                transform: Transform::from_scale(Vec3::splat(
                                    (builder.radius * camera_scale.scale) as f32,
                                )),
                                ..default()
                            },
                            RealisticView,
                        ));
                    }
                });
        }

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
            .with_children(|commands| {
                commands.spawn((
                    SelectionRectMarker,
                    PbrBundle {
                        mesh: meshes.add(
                            shape::Quad {
                                size: Vec2::splat(1.0),
                                ..default()
                            }
                            .into(),
                        ),
                        material: materials.add(Color::ALICE_BLUE.into()),
                        transform: Transform::from_translation(Vec3::Z),

                        ..default()
                    },
                    NotShadowCaster,
                    RenderLayers::layer(2),
                ));
            });
    }
}
