use bevy::prelude::Bundle;

pub mod body_mass;
pub use body_mass::*;

pub mod body_velocity;
pub use body_velocity::*;

pub mod body_trail;
pub use body_trail::*;

pub mod query;
pub use query::*;

#[derive(Debug, Default, Bundle)]
pub struct BodyBundle {
    pub mass: BodyMass,
    pub velocity: BodyVelocity,
    pub trail: BodyTrail,
}
