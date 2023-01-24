use bevy::prelude::*;

#[derive(Debug, Component, derive_more::From, derive_more::Into)]
pub struct BodyMass {
    pub mass: f32,
}

impl Default for BodyMass {
    fn default() -> Self {
        Self { mass: 1.0 }
    }
}
