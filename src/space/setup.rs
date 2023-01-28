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

    for (i, (color, radius, mass, position, velocity)) in [
        (
            Color::GREEN,
            6378e3,
            5.9724e24,
            DVec3::X * 149.596e9,
            // -DVec3::Z * 29.78e3,
            -DVec3::Z * 29.78e2,
        ),
        (Color::rgb(1.0, 0.6549, 0.0), 695700e3, 1.989e30, DVec3::ZERO, DVec3::ZERO),
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
                        radius: (radius * camera_scale.scale) as f32,
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
                width: 4.0,
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
            projection: PerspectiveProjection { near: 0.0000001, ..default() }.into(),
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
