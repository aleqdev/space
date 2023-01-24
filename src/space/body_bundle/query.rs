use bevy::{ecs::query::WorldQuery, prelude::*};
use bevy_polyline::prelude::Polyline;
use ringbuffer::RingBufferExt;

use super::{BodyMass, BodyTrail, BodyVelocity};

#[derive(WorldQuery)]
#[world_query(mutable, derive(Debug))]
pub struct BodiesQueryParam {
    mass: &'static mut BodyMass,
    trail: &'static mut BodyTrail,
    polyline: &'static Handle<Polyline>,
    material: &'static Handle<StandardMaterial>,
    velocity: &'static mut BodyVelocity,
    transform: &'static mut Transform,
}

impl<'w> BodiesQueryParamItem<'w> {
    #[allow(dead_code)]
    pub fn mass(&self) -> &f32 {
        &self.mass.mass
    }

    #[allow(dead_code)]
    pub fn mass_mut(&mut self) -> &mut f32 {
        &mut self.mass.mass
    }

    #[allow(dead_code)]
    pub fn color<'a, 'assets: 'a>(&'a self, materials: &'assets Assets<StandardMaterial>) -> &'a Color {
        &materials.get(self.material).unwrap().base_color
    }

    #[allow(dead_code)]
    pub fn color_mut<'a, 'assets: 'a>(&'a mut self, materials: &'assets mut Assets<StandardMaterial>) -> &'a mut Color {
        &mut materials.get_mut(self.material).unwrap().base_color
    }

    #[allow(dead_code)]
    pub fn polyline<'a, 'assets: 'a>(&'a self, polylines: &'assets Assets<Polyline>) -> &'a Polyline {
        polylines.get(self.polyline).unwrap()
    }

    #[allow(dead_code)]
    pub fn polyline_mut<'a, 'assets: 'a>(&'a mut self, polylines: &'assets mut Assets<Polyline>) -> &'a mut Polyline {
        polylines.get_mut(self.polyline).unwrap()
    }

    #[allow(dead_code)]
    pub fn trail(&self) -> &impl RingBufferExt<Vec3> {
        &self.trail.buffer
    }

    #[allow(dead_code)]
    pub fn trail_mut(&mut self) -> &mut impl RingBufferExt<Vec3> {
        &mut self.trail.buffer
    }

    #[allow(dead_code)]
    pub fn velocity(&self) -> &Vec3 {
        &self.velocity.velocity
    }

    #[allow(dead_code)]
    pub fn velocity_mut(&mut self) -> &mut Vec3 {
        &mut self.velocity.velocity
    }

    #[allow(dead_code)]
    pub fn position(&self) -> &Vec3 {
        &self.transform.translation
    }

    #[allow(dead_code)]
    pub fn position_mut(&mut self) -> &mut Vec3 {
        &mut self.transform.translation
    }
}

pub type BodiesQuery<'world, 'state> = Query<'world, 'state, BodiesQueryParam>;
