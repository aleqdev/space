pub mod camera_orbit_driver;
pub use camera_orbit_driver::*;

pub mod camera_sensitivity;
pub use camera_sensitivity::*;

pub mod systems {
    use super::{CameraControlSensitivity, UnconstrainedOrbit};
    use crate::space::{scene::{
        markers::{FocusedBody, MainCamera3d, SelectedBody},
    }, display::BodyRef};
    use bevy::{
        input::mouse::{MouseMotion, MouseWheel},
        prelude::*,
    };
    use bevy_dolly::prelude::{Arm, Rig};
    use bevy_ecs_markers::params::{Marker, MarkerMut};

    pub fn zoom(
        mut camera: Query<&mut Rig, With<MainCamera3d>>,
        mut mouse: EventReader<MouseWheel>,
        sensitivity: Res<CameraControlSensitivity>,
    ) {
        let offset = mouse.iter().fold(1.0, |offset, ev| {
            offset * sensitivity.zoom.powi(ev.y.signum() as i32)
        });
        if offset == 0.0 {
            return;
        }

        let mut rig = camera.single_mut();
        let arm = rig.driver_mut::<Arm>();

        arm.offset.z /= offset;
    }

    pub fn orbit(
        mut rig: Query<&mut Rig, With<MainCamera3d>>,
        mut mouse: EventReader<MouseMotion>,
        sensitivity: Res<CameraControlSensitivity>,
        mouse_button: Res<Input<MouseButton>>,
    ) {
        if !mouse_button.pressed(MouseButton::Right) {
            return;
        }

        let mut rig = rig.single_mut();

        let driver = rig.driver_mut::<UnconstrainedOrbit>();

        let mut delta = Vec2::ZERO;
        for ev in mouse.iter() {
            delta += ev.delta;
        }

        driver.rotate_yaw_pitch(
            -0.1 * delta.x * sensitivity.orbit.x,
            -0.1 * delta.y * sensitivity.orbit.y,
        );
    }

    pub fn focus(
        mouse: Res<Input<MouseButton>>,
        selected_body: Marker<SelectedBody>,
        mut focused_body: MarkerMut<FocusedBody>,
        camera: Query<Entity, (With<MainCamera3d>, With<Camera3d>)>,
        mut commands: Commands,
        bodies: Query<Entity, With<BodyRef>>,
    ) {
        use SelectedBody::*;
        // use bevy_prototype_lyon::prelude::*;

        if mouse.just_pressed(MouseButton::Left) {
            if selected_body[CurrentRedirected].index() != u32::MAX {
                **focused_body = selected_body[CurrentRedirected];

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
                commands
                    .entity(camera.single())
                    .set_parent(bodies.get(**focused_body).unwrap());
            } else {
                **focused_body = Entity::from_raw(u32::MAX);
                commands.entity(camera.single()).remove_parent();
            }
        }
    }
}
