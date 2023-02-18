use std::marker::PhantomData;

use bevy::{ecs::system::SystemParam, prelude::*};

use super::{CameraScale, RelativeWorldScale};

#[derive(SystemParam)]
pub struct ComputedScale<'w, 's> {
    pub camera_scale: Res<'w, CameraScale>,
    pub relative_world_scale: Res<'w, RelativeWorldScale>,
    #[system_param(ignore)]
    marker: PhantomData<&'s ()>,
}

impl ComputedScale<'_, '_> {
    pub fn get_scale(&self) -> f64 {
        self.get_camera_scale() * self.get_relative_world_scale()
    }

    pub fn get_camera_scale(&self) -> f64 {
        self.camera_scale.scale
    }

    pub fn get_relative_world_scale(&self) -> f64 {
        self.relative_world_scale.scale
    }
}
