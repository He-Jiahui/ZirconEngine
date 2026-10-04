use super::*;

#[test]
fn all_lod_revision_changes_for_geometry_or_bounds_changes() {
    let original = ResolvedSourceLevel {
        local_bounds: RenderMeshBounds::from_min_max([-1.0; 3], [1.0; 3]),
        resource_revisions: vec![7, 11],
    };
    let changed_geometry = ResolvedSourceLevel {
        local_bounds: original.local_bounds,
        resource_revisions: vec![7, 12],
    };
    let changed_bounds = ResolvedSourceLevel {
        local_bounds: RenderMeshBounds::from_min_max([-2.0; 3], [1.0; 3]),
        resource_revisions: original.resource_revisions.clone(),
    };

    let original_revisions = resolved_revision_set([&original]);
    let geometry_revisions = resolved_revision_set([&changed_geometry]);
    let bounds_revisions = resolved_revision_set([&changed_bounds]);

    assert_ne!(original_revisions.geometry, geometry_revisions.geometry);
    assert_ne!(original_revisions.bounds, geometry_revisions.bounds);
    assert_eq!(original_revisions.geometry, bounds_revisions.geometry);
    assert_ne!(original_revisions.bounds, bounds_revisions.bounds);
}

#[test]
fn source_level_bounds_include_every_primitive_binding() {
    let merged = union_bounds(
        RenderMeshBounds::from_min_max([-1.0; 3], [1.0; 3]),
        RenderMeshBounds::from_min_max([-4.0, -2.0, 0.0], [2.0, 3.0, 5.0]),
    );

    assert_eq!(merged.min, [-4.0, -2.0, -1.0]);
    assert_eq!(merged.max, [2.0, 3.0, 5.0]);
}
