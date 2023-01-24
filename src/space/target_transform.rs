use bevy::prelude::*;

#[derive(Component)]
pub struct TargetTransform {
    pub transform: Transform,
    pub smooth: f32
}

pub fn approach_target_transform_system(
    mut bodies: Query<(&mut Transform, &TargetTransform)>,
    time: Res<Time>
) {
    for (mut transform, target) in &mut bodies {
        transform.translation = transform.translation + (target.transform.translation - transform.translation) * target.smooth * time.delta_seconds();
        transform.rotation = transform.rotation + (target.transform.rotation - transform.rotation) * target.smooth * time.delta_seconds();
        transform.scale = transform.scale + (target.transform.scale - transform.scale) * target.smooth * time.delta_seconds();
    }
}