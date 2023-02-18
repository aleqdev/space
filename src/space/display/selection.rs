use bevy::prelude::*;

#[derive(Component)]
pub struct PrimarySelectionRectMarker;

#[derive(Component)]
pub struct SecondarySelectionRectMarker;

pub mod systems {
    use bevy::prelude::*;
    use bevy_ecs_markers::params::Marker;

    use crate::space::{
        display::{custom_params::ComputedScale, BodyRef},
        ext::EntityOpsExt,
        scene::markers::{FocusedBody, MainCamera3d},
        simulation::SpaceSimulation,
    };

    use super::{PrimarySelectionRectMarker, SecondarySelectionRectMarker};

    pub fn display_selection_rects(
        camera: Query<(&GlobalTransform, &Camera), With<MainCamera3d>>,
        selected: Marker<FocusedBody>,
        bodies: Query<(&GlobalTransform, &BodyRef)>,
        mut primary_rect: Query<
            (&mut Transform, &mut Visibility),
            (
                With<PrimarySelectionRectMarker>,
                Without<SecondarySelectionRectMarker>,
            ),
        >,
        mut secondary_rect: Query<
            (&mut Transform, &mut Visibility),
            With<SecondarySelectionRectMarker>,
        >,
        simulation: Res<SpaceSimulation>,
        scale: ComputedScale,
    ) {
        use FocusedBody::*;

        let scale = scale.get_scale();

        const MIN_RADIUS: f32 = 1.25;
        const MAX_RADIUS: f32 = 200.55;

        for (selected, (mut transform, mut visibility)) in [
            (selected[Primary], primary_rect.single_mut()),
            (selected[Secondary], secondary_rect.single_mut()),
        ] {
            if selected.is_valid() {
                let (camera_transform, camera) = camera.single();

                let Some(viewport) = camera.logical_viewport_size() else { return };

                let (body_transform, &BodyRef(body_ref)) = bodies.get(selected).unwrap();

                let radius = simulation.bodies.radius[body_ref];

                let Some(body_projected) = camera.world_to_viewport(camera_transform, body_transform
                    .translation()).map(|x| x - viewport / 2.0) else { return };

                transform.translation.x = body_projected.x;
                transform.translation.y = body_projected.y;

                let Some(body_edge_projected) = camera.world_to_viewport(camera_transform, body_transform
                    .translation() + camera_transform.right() * (radius * scale) as f32).map(|x| x - viewport / 2.0) else { return };

                let projected_radius = body_projected.distance(body_edge_projected) / 10.0;

                transform.scale = Vec3::splat(projected_radius.clamp(MIN_RADIUS, MAX_RADIUS));

                visibility.is_visible = true;
            } else {
                visibility.is_visible = false;
            }
        }
    }
}
