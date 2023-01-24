use bevy::{core_pipeline::{bloom::BloomSettings, clear_color::ClearColorConfig}, prelude::*};
use bevy_polyline::prelude::*;

use crate::space::{TargetTransform, CameraOrbitParams};

use super::BodyBundle;

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut polylines: ResMut<Assets<Polyline>>,
    mut polyline_materials: ResMut<Assets<PolylineMaterial>>
) {
    use bevy_mod_raycast::{RaycastSource, RaycastMesh};
    use ringbuffer::AllocRingBuffer;
    use super::SelectionRaycastSet;
    use super::MainCamera3d;

    for (color, position, velocity) in [
        (Color::RED, Vec3::ZERO, Vec3::ZERO),
        (Color::CYAN, Vec3::Z * 4.0, Vec3::new(0.2, 0.7, 0.1)),
        (Color::GREEN, Vec3::X * 4.0, Vec3::new(0.2, 0.3, 0.1)),
        (Color::YELLOW, -Vec3::Z * 4.0, Vec3::ZERO),
        (Color::WHITE, -Vec3::Y * 4.0, Vec3::new(0.2, 0.7, 0.1)),
        (Color::ORANGE, -Vec3::X * 4.0, Vec3::new(0.2, 0.3, 0.1))
    ] {
        let polyline = polylines.add(Polyline {
            vertices: Vec::with_capacity(1024),
        });

        commands.spawn((
            BodyBundle {
                mass: 0.4.into(),
                velocity: velocity.into(),
                trail: AllocRingBuffer::with_capacity(1024).into(),
                ..default()
            },
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
                transform: Transform::from_translation(position),
                ..default()
            },
            polyline.clone(),
            RaycastMesh::<SelectionRaycastSet>::default()
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

    commands
        .spawn((
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
            TargetTransform {
                transform: Transform::from_xyz(0., 0., 10.,),
                smooth: 0.9,
            },
            CameraOrbitParams {
                distance: 10.0
            }
        ));
}
