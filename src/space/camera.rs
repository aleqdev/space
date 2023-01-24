use bevy::{prelude::*, input::mouse::{MouseWheel, MouseMotion}};
use bevy_ecs_markers::params::{Marker, MarkerMut};

use super::{SelectedBody, FocusedBody, TargetTransform};

#[derive(Resource)]
pub struct CameraControlSensitivity {
    pub zoom: f32,
    pub orbit: Vec2
}

#[derive(Component)]
pub struct MainCamera3d;

#[derive(Component)]
pub struct CameraOrbitParams {
    pub distance: f32
}

pub fn focus_camera_on_click(
    mouse: Res<Input<MouseButton>>,
    selected_body: Marker<SelectedBody>,
    mut focused_body: MarkerMut<FocusedBody>
) {
    use SelectedBody::*;
    // use bevy_prototype_lyon::prelude::*;

    if mouse.just_pressed(MouseButton::Left) {
        if selected_body[Current].index() != u32::MAX {
            **focused_body = selected_body[Current];

            /*let shape = shapes::RegularPolygon {
                sides: 6,
                feature: shapes::RegularPolygonFeature::Radius(200.0),
                ..default()
            };

            commands.spawn(Camera2dBundle {
                camera: Camera {
                    priority: 1,
                    ..default()
                },
                camera_2d: Camera2d {
                    clear_color: ClearColorConfig::None,
                    ..default()
                },
                ..default()
            });

            commands.spawn(GeometryBuilder::build_as(
                &shape,
                DrawMode::Outlined {
                    fill_mode: FillMode::color(Color::CYAN),
                    outline_mode: StrokeMode::new(Color::BLACK, 10.0),
                },
                Transform::default(),
            ));*/
        } else {
            **focused_body = Entity::from_raw(u32::MAX);
        }
    }
}

pub fn align_camera_with_focus(
    mut camera: Query<(&mut TargetTransform, &CameraOrbitParams), (With<Camera3d>, With<MainCamera3d>)>,
    focused_body: Marker<FocusedBody>,
    bodies: Query<&Transform>
) {
    let (mut target_transform, params) = camera.single_mut();

    if focused_body.index() != u32::MAX {
        let body = bodies.get(**focused_body).unwrap().translation;
        target_transform.transform.translation = body - target_transform.transform.forward() * params.distance;
        let up = target_transform.transform.up();
        target_transform.transform.look_at(body, up);
    }
}


pub fn camera_zoom(
    mut camera: Query<(&mut TargetTransform, &mut CameraOrbitParams), (With<Camera3d>, With<MainCamera3d>)>,
    mut mouse: EventReader<MouseWheel>,
    sensitivity: Res<CameraControlSensitivity>
) {
    let offset = mouse.iter().fold(0.0, |offset, ev| offset + ev.y * sensitivity.zoom);
    if offset == 0.0 {
        return
    }

    let (mut camera, mut params) = camera.single_mut();

    let translation = camera.transform.forward() * offset;

    camera.transform.translation += translation;

    params.distance = camera.transform.translation.length();
}


pub fn camera_orbit(
    mut rig: Query<(&Transform, &mut Rig)>,
    mut mouse: EventReader<MouseMotion>,
    sensitivity: Res<CameraControlSensitivity>,
    mouse_button: Res<Input<MouseButton>>
) {
    if !mouse_button.pressed(MouseButton::Right) {
        return
    }

    let (transform, mut rig) = rig.single_mut();

    let signum = rig.final_transform.up().y.signum();

    let driver = rig.driver_mut::<Rotation>();

    let mut delta = Vec2::ZERO;
    for ev in mouse.iter() {
        delta += ev.delta;
    }

    driver.rotation += transform.rotate_local_axis(axis, angle)
    driver.rotate_yaw_pitch(
        -0.1 * delta.x * sensitivity.orbit.x * signum,
        -0.1 * delta.y * sensitivity.orbit.y,
    );
}
