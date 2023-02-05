pub mod systems {
    use bevy::prelude::*;
    use bevy_ecs_markers::params::Marker;
    use bevy_mod_raycast::RaycastMesh;
    use bevy_polyline::prelude::{Polyline, PolylineMaterial};

    use crate::space::{
        display::BodyTrailRef,
        scene::{markers::FocusedBody, SelectionRaycastSet},
    };

    pub fn dissolve_extreme_trails(
        bodies: Query<&BodyTrailRef, With<RaycastMesh<SelectionRaycastSet>>>,
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

        info!("{distance}");

        let polyline_material = polyline_materials
            .get_mut(polyline_material_handle)
            .unwrap();
        let opacity = if distance > 1000.0 { 0.0 } else { 1.0 };
        let a = polyline_material.color.a();
        polyline_material
            .color
            .set_a(a + (opacity - a) * time.delta_seconds() * 5.0);
    }
}
