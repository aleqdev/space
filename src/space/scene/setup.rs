pub mod systems {
    use bevy::prelude::*;
    use bevy_polyline::prelude::{Polyline, PolylineMaterial};

    use crate::space::{
        display::{
            BodyTrailRef, CameraScale, RealisticView, RelativeWorldOffset, RelativeWorldScale,
            SchematicView, StarMaterial,
        },
        scene::SelectionTargetRedirect,
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
            speed: 86400.0 * 20.0,
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

        let mut make_schematic_material = |color| materials.add(StandardMaterial {
            unlit: true,
            base_color: color,
            ..default()
        });

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
                        material: make_schematic_material(Color::GRAY),
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
                        mesh: meshes.add(
                            shape::Icosphere {
                                radius: 1.0,
                                subdivisions: 6,
                            }
                            .into(),
                        ),
                        material: star_materials.add(StarMaterial { primary_color: color, secondary_color: Color::ORANGE, ..default() }),
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
                        material: make_schematic_material(Color::GRAY),
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
                        mesh: meshes.add(
                            shape::Icosphere {
                                radius: 1.0,
                                subdivisions: 6,
                            }
                            .into(),
                        ),
                        material: make_schematic_material(Color::GREEN),
                        transform: Transform::from_scale(Vec3::splat(
                            (radius * camera_scale.scale) as f32,
                        )),
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
                    ..default()
                },
                camera_3d: Camera3d {
                    clear_color: ClearColorConfig::Custom(Color::BLACK),
                    ..default()
                },
                projection: PerspectiveProjection {
                    near: 0.0000001,
                    far: 1000000.0,
                    ..default()
                }
                .into(),
                ..default()
            },
            BloomSettings {
                intensity: 30.2,
                ..default()
            },
            RaycastSource::<SelectionRaycastSet>::new(),
            MainCamera3d,
        ));
    }
}
