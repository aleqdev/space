pub mod camera_orbit_driver;
pub use camera_orbit_driver::*;

pub mod camera_sensitivity;
pub use camera_sensitivity::*;

pub mod systems {
    use super::{CameraControlSensitivity, UnconstrainedOrbit};
    use crate::space::{
        display::BodyRef,
        ext::EntityOpsExt,
        scene::markers::{FocusedBody, MainCamera3d, SelectedBody},
    };
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
        keyboard: Res<Input<ScanCode>>,
        mut commands: Commands,
        bodies: Query<Entity, With<BodyRef>>,
    ) {
        use FocusedBody::*;
        use SelectedBody::*;

        if mouse.just_pressed(MouseButton::Left) {
            if selected_body[CurrentRedirected].is_valid() {
                if keyboard.pressed(ScanCode(29)) {
                    focused_body[Secondary] = selected_body[CurrentRedirected];
                } else {
                    focused_body[Primary] = selected_body[CurrentRedirected];

                    commands
                        .entity(camera.single())
                        .set_parent(bodies.get(focused_body[Primary]).unwrap());
                }
            } else {
                focused_body[Primary].invalidate();
                focused_body[Secondary].invalidate();
                commands.entity(camera.single()).remove_parent();
            }
        }
    }
}
