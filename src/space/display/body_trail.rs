pub mod systems {
    use bevy::prelude::*;
    use bevy_ecs_markers::params::Marker;
    use bevy_polyline::prelude::{Polyline, PolylineMaterial};

    use crate::space::{display::BodyTrailRef, scene::markers::FocusedBody};

    pub fn dissolve_extreme_trails(
        bodies: Query<&BodyTrailRef>,
        polyline_entities: Query<
            (&Handle<Polyline>, &Handle<PolylineMaterial>),
            Without<BodyTrailRef>,
        >,
        polylines: ResMut<Assets<Polyline>>,
        mut polyline_materials: ResMut<Assets<PolylineMaterial>>,
        focused_body: Marker<FocusedBody>,
        time: Res<Time>,
    ) {
        let Ok(&BodyTrailRef(trail)) = bodies.get(**focused_body) else {return};

        let (polyline_handle, polyline_material_handle) = polyline_entities.get(trail).unwrap();
        let polyline = polylines.get(polyline_handle).unwrap();

        let mut iter = polyline.vertices.iter().rev();

        let (Some(v1), Some(v2)) = (iter.next(), iter.next()) else { return };

        let distance = v1.distance(*v2);

        let polyline_material = polyline_materials
            .get_mut(polyline_material_handle)
            .unwrap();
        let opacity_change = if distance > 1000.0 { -2.0 } else { 1.0 };
        let a = polyline_material.color.a();
        polyline_material
            .color
            .set_a((a + opacity_change * time.delta_seconds() * 0.5).clamp(0.0, 1.0));
    }
}
