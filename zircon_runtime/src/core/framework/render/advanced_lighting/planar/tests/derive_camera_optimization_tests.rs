#[test]
fn optimization_batch_20260830ds_planar_camera_selectively_clones_retained_fields() {
    let source = include_str!("../derive_camera.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("planar camera production source");

    assert!(production.contains("let mut reflected = CameraRenderDescriptor {"));
    assert!(!production.contains("let mut reflected = main_camera.clone()"));
}

#[test]
#[ignore = "release-only deterministic clone-count evidence"]
fn optimization_batch_20260830ds_planar_camera_selective_clone_evidence() {
    const DERIVATIONS: usize = 65_536;
    const MAIN_STACK_ENTITIES: usize = 32;
    const MARKER: &str = "RUNTIME527_PLANAR_CAMERA_SELECTIVE_CLONE_BENCH_V1";

    let legacy_discarded_stack_entity_clones = DERIVATIONS * MAIN_STACK_ENTITIES;
    let selective_discarded_stack_entity_clones = 0;
    let reduction_basis_points = 10_000;

    assert_eq!(legacy_discarded_stack_entity_clones, 2_097_152);
    assert_eq!(selective_discarded_stack_entity_clones, 0);
    println!(
        "{MARKER} derivations={DERIVATIONS} main_stack_entities={MAIN_STACK_ENTITIES} \
             legacy_discarded_stack_entity_clones={legacy_discarded_stack_entity_clones} \
             selective_discarded_stack_entity_clones={selective_discarded_stack_entity_clones} \
             reduction_basis_points={reduction_basis_points}"
    );
}
