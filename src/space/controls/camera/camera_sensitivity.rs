use bevy::prelude::*;

#[derive(Resource)]
pub struct CameraControlSensitivity {
    pub zoom: f32,
    pub orbit: Vec2,
}
