pub mod systems {
    use bevy::prelude::*;
    use bevy_mod_raycast::RaycastMesh;
    use bevy_polyline::prelude::{Polyline, PolylineBundle, PolylineMaterial};

    use crate::space::{
        display::{
            BodyRef, BodyTrail, CameraScale, RealisticView, RelativeLightIntensivity,
            SchematicView, StarMaterial,
        },
        nasa_horizons::{NasaBodyAddition, SpaceBodyKnownDetailsMaterial},
        scene::{markers::BodySystemRoot, SelectionRaycastSet, SelectionTargetRedirect},
    };

    pub fn spawn_nasa_body(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
        mut star_materials: ResMut<Assets<StarMaterial>>,
        mut polylines: ResMut<Assets<Polyline>>,
        mut polyline_materials: ResMut<Assets<PolylineMaterial>>,
        camera_scale: Res<CameraScale>,
        asset_server: Res<AssetServer>,
        mut ev: EventReader<NasaBodyAddition>,
        body_system_root: Query<Entity, With<BodySystemRoot>>,
    ) {
        let body_system_root = body_system_root.single();

        commands.entity(body_system_root).with_children(|commands| {
            for response in ev.iter() {
                let mut add_polyline = || {
                    polylines.add(Polyline {
                        vertices: Vec::with_capacity(1024),
                    })
                };

                let mut add_polyline_entity =
                    |commands: &mut ChildBuilder, polyline, mut color: Color, name: String| {
                        color.set_a(0.1);
                        color.set_r((color.r() * 4.5).min(1.0));
                        color.set_g((color.g() * 4.5).min(1.0));
                        color.set_b((color.b() * 4.5).min(1.0));
                        commands
                            .spawn((
                                PolylineBundle {
                                    polyline,
                                    material: polyline_materials.add(PolylineMaterial {
                                        width: 2.0,
                                        color,
                                        ..Default::default()
                                    }),
                                    ..Default::default()
                                },
                                BodyTrail {
                                    body_name: name,
                                    ..default()
                                },
                            ))
                            .id()
                    };

                let uv_sphere =
                    || asset_server.load::<Mesh, _>("glb/sphereUV.glb#Mesh0/Primitive0");

                let polyline = add_polyline();

                add_polyline_entity(commands, polyline, Color::GRAY, response.name.clone());

                commands
                    .spawn((
                        BodyRef(response.name.clone()),
                        SpatialBundle {
                            transform: Transform::from_translation(Vec3::new(
                                (response.body.position.x * camera_scale.scale) as f32,
                                (response.body.position.y * camera_scale.scale) as f32,
                                (response.body.position.z * camera_scale.scale) as f32,
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
                                    (response.body.radius * camera_scale.scale) as f32,
                                )),
                                ..default()
                            },
                            RaycastMesh::<SelectionRaycastSet>::default(),
                            SelectionTargetRedirect(anchor.parent_entity()),
                            SchematicView,
                        ));
                        if let SpaceBodyKnownDetailsMaterial::Star(star_material) =
                            &response.material
                        {
                            anchor.spawn((
                                MaterialMeshBundle {
                                    mesh: uv_sphere(),
                                    material: make_star_material(star_material.clone()),
                                    transform: Transform::from_scale(Vec3::splat(
                                        (response.body.radius * camera_scale.scale) as f32,
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
                                        radius: response.body.radius as f32,
                                        shadows_enabled: true,
                                        ..default()
                                    },
                                    ..default()
                                },
                                RelativeLightIntensivity(
                                    200.0 / camera_scale.scale / camera_scale.scale,
                                ),
                            ));
                        } else if let SpaceBodyKnownDetailsMaterial::TexturePath(path) =
                            &response.material
                        {
                            anchor.spawn((
                                MaterialMeshBundle {
                                    mesh: uv_sphere(),
                                    material: make_solid_material(&mut materials, path),
                                    transform: Transform::from_scale(Vec3::splat(
                                        (response.body.radius * camera_scale.scale) as f32,
                                    ))
                                    .with_rotation(response.body.rotation.initial),
                                    ..default()
                                },
                                RealisticView,
                            ));
                        }
                    });
            }
        });
    }
}
