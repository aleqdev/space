use super::BodiesQuery;
use bevy::prelude::*;
use bevy_polyline::prelude::Polyline;

pub fn update(
    mut bodies: BodiesQuery,
    time: Res<Time>,
    mut polylines: ResMut<Assets<Polyline>>
) {
    use ringbuffer::{RingBufferWrite, RingBufferExt};

    let mut combinations = bodies.iter_combinations_mut();
    while let Some([mut b1, mut b2]) = combinations.fetch_next() {
        let distance = b1.position().distance(*b2.position());
        let distance2 = distance.powi(2);
        let direction = (*b2.position() - *b1.position()).normalize();
        *b1.velocity_mut() += direction / distance2 * *b2.mass() * time.delta_seconds();
        *b2.velocity_mut() -= direction / distance2 * *b1.mass() * time.delta_seconds();
    }
    for mut body in &mut bodies {
        let velocity = *body.velocity();
        *body.position_mut() += velocity * time.delta_seconds();
        let position = *body.position();
        body.trail_mut().push(position);
        body.polyline_mut(&mut *polylines).vertices = body.trail().iter().map(ToOwned::to_owned).collect();
    }
}
