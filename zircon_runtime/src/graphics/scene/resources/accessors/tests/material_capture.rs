#[test]
fn capture_reads_only_published_runtime_state_before_cold_asset_fallback() {
    let production = include_str!("../material_capture.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("material capture test boundary");
    let published = production
        .find("published_material_draw_proxy(id)")
        .expect("published draw proxy lookup");
    let cold_fallback = production
        .find("load_effective_material_asset")
        .expect("canonical cold material fallback");

    assert!(published < cold_fallback);
    assert!(production.contains("self.materials.contains_key(id)"));
    assert!(!production.contains("self.material(id)"));
    assert!(production.contains("material_capture_published_proxy"));
    assert!(production.contains("material_capture_generation_bound_texture_samples"));
}

#[test]
fn cold_texture_capture_uses_one_generation_bound_snapshot() {
    let production = include_str!("../material_capture.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("material capture test boundary");

    assert!(production.contains("load_texture_asset_snapshot(id)"));
    assert!(production.contains("Some(texture.revision())"));
    assert!(!production.contains("let Ok(revision) = self.resource_revision(id)"));
}
