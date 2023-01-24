use bevy::prelude::*;

#[derive(Debug, Default, Component, derive_more::From, derive_more::Into)]
pub struct BodyVelocity {
    pub velocity: Vec3,
}
