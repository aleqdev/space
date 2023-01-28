use bevy::{
    core_pipeline::{bloom::BloomSettings, clear_color::ClearColorConfig},
    math::DVec3,
    prelude::*,
};
use bevy_polyline::prelude::*;

use super::{BodyRef, CameraScale, SpaceSimulation, UnconstrainedOrbit};

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut polylines: ResMut<Assets<Polyline>>,
    mut polyline_materials: ResMut<Assets<PolylineMaterial>>,
    mut simulation: ResMut<SpaceSimulation>,
    camera_scale: Res<CameraScale>,
) {
    use super::MainCamera3d;
    use super::SelectionRaycastSet;
    use bevy_dolly::prelude::*;
    use bevy_mod_raycast::{RaycastMesh, RaycastSource};

    for (i, (color, mass, position, velocity)) in [
        (
            Color::GREEN,
            5.97217 * 10e24,
            DVec3::X * 147.095 * 10e6 * 1000.0,
            -DVec3::Z * 30.29 * 1000.0,
        ),
        (Color::ORANGE, 1_988_500.0 * 10e24, DVec3::ZERO, DVec3::ZERO),
    ]
    .into_iter()
    .enumerate()
    {
        simulation.positions.push(position);
        simulation.velocities.push(velocity);
        simulation.masses.push(mass);
        simulation.trails.push(Default::default());

        let polyline = polylines.add(Polyline {
            vertices: Vec::with_capacity(1024),
        });

        commands.spawn((
            BodyRef(i),
            PbrBundle {
                mesh: meshes.add(
                    shape::Icosphere {
                        radius: 0.1,
                        subdivisions: 2,
                    }
                    .into(),
                ),
                material: materials.add(StandardMaterial {
                    base_color: color,
                    unlit: true,
                    ..default()
                }),
                transform: Transform::from_translation(Vec3::new(
                    (position.x * camera_scale.scale) as f32,
                    (position.y * camera_scale.scale) as f32,
                    (position.z * camera_scale.scale) as f32,
                )),
                ..default()
            },
            polyline.clone(),
            RaycastMesh::<SelectionRaycastSet>::default(),
        ));

        commands.spawn(PolylineBundle {
            polyline,
            material: polyline_materials.add(PolylineMaterial {
                width: 16.0,
                color,
                perspective: true,
                ..Default::default()
            }),
            ..Default::default()
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
            .with(Smooth::new_position(0.1))
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
