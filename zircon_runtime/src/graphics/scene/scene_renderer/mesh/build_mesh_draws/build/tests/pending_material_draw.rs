#[test]
fn pending_mesh_draw_owns_material_state_as_one_replaceable_field() {
    let pending_mesh_draw = include_str!("../pending_mesh_draw.rs");
    let body = pending_mesh_draw
        .split_once("pub(super) struct PendingMeshDraw {")
        .and_then(|(_, body)| body.split_once("\n}"))
        .map(|(body, _)| body)
        .expect("pending mesh draw declaration");

    assert!(body.contains("material: PendingMaterialDraw"));
    assert!(!body.contains("material_uniform:"));
    assert!(!body.contains("pipeline_key:"));
    assert!(!body.contains("disabled_passes:"));
}

#[test]
fn pending_material_keeps_the_unhashed_draw_generation() {
    let source = include_str!("../pending_material_draw.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("pending material draw test boundary");

    assert!(source.contains("draw_generation: Option<u64>"));
}

#[test]
fn pending_material_preserves_renderer_authored_shadow_mode_before_material_merge() {
    let pending_material = include_str!("../pending_material_draw.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("pending material draw test boundary");
    let builder = include_str!("../extend_pending_draws_for_mesh_instance.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("pending material builder test boundary");

    assert!(pending_material.contains("renderer_cast_shadows: CastShadowsMode"));
    assert!(builder.contains("renderer_cast_shadows: mesh_instance.common.cast_shadows"));
}
