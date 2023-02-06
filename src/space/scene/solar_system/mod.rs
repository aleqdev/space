use bevy::{math::DVec3, prelude::Quat};

use crate::space::display::StarMaterial;

pub enum SolarSystemBodyBuilderMaterial {
    TexturePath(&'static str),
    Star(StarMaterial),
}

pub struct SolarSystemBodyBuilder {
    pub radius: f64,
    pub mass: f64,
    pub position: DVec3,
    pub velocity: DVec3,
    pub material: SolarSystemBodyBuilderMaterial,
    pub rotation: Quat,
    pub rotation_rate: f64,
}

pub mod earth;
pub mod jupiter;
pub mod mars;
pub mod mercury;
pub mod moon;
pub mod neptune;
pub mod saturn;
pub mod sun;
pub mod uranus;
pub mod venus;
