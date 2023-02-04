pub mod systems {
    use bevy::prelude::*;
    use bevy_polyline::prelude::{Polyline, PolylineMaterial};

    use crate::space::{
        display::{BodyTrailRef, CameraScale, RelativeWorldOffset, RelativeWorldScale},
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

        for (i, (color, radius, mass, position, velocity)) in [
            (
                Color::GREEN,
                0.0001,
                5.9724e24,
                DVec3::X * 149.596e9,
                //-DVec3::Z * 29.78e3,
                -DVec3::Z * 19.78e3,
            ),
            (
                Color::rgb(1.0, 0.6549, 0.0),
                695700e3,
                1.989e30,
                DVec3::ZERO,
                DVec3::ZERO,
            ),
        ]
        .into_iter()
        .enumerate()
        {
            simulation.bodies.push(SpaceBody {
                position,
                velocity,
                mass,
                radius,
                trail: AllocRingBuffer::with_capacity(512),
            });

            let polyline = polylines.add(Polyline {
                vertices: Vec::with_capacity(512),
            });

            let polyline_entity = commands
                .spawn(PolylineBundle {
                    polyline,
                    material: polyline_materials.add(PolylineMaterial {
                        width: 4.0,
                        color,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .id();

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
                        BodyRef(i),
                        BodyTrailRef(polyline_entity),
                        PbrBundle {
                            mesh: meshes.add(
                                shape::Icosphere {
                                    radius: 1.0,
                                    subdivisions: 2,
                                }
                                .into(),
                            ),
                            material: materials.add(StandardMaterial {
                                base_color: color,
                                unlit: true,
                                ..default()
                            }),
                            transform: Transform::from_scale(Vec3::splat(
                                (radius * camera_scale.scale) as f32,
                            )),
                            ..default()
                        },
                        RaycastMesh::<SelectionRaycastSet>::default(),
                    ));
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
                .with(Smooth::new_position_rotation(0.22, 0.22))
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
                    ..default()
                }
                .into(),
                ..default()
            },
            BloomSettings {
                intensity: 5.0,
                ..default()
            },
            RaycastSource::<SelectionRaycastSet>::new(),
            MainCamera3d,
        ));
    }
}
