use super::*;

fn empty_bake() -> SdfAtlasBake {
    SdfAtlasBake {
        pages: Arc::from([]),
        dirty_pages: Arc::from([]),
        glyphs: Arc::from([]),
        generation_failures: Arc::from([]),
        report: SdfAtlasBakeReport::default(),
    }
}

#[test]
fn retained_slot_product_generation_is_the_o1_reuse_authority() {
    let mut cache = SdfPreparedAtlasCache::default();
    let bake = empty_bake();
    cache.replace(UVec2::new(1, 1), &[], Some(7), &bake);

    let reused = cache.reuse(
        UVec2::new(1, 1),
        &[],
        Some(7),
        Default::default(),
        Default::default(),
    );

    assert!(reused.is_some());
    assert_eq!(reused.unwrap().report.compiled_atlas_reuse_count, 1);
}

#[test]
fn foreign_slot_product_generation_still_checks_exact_slots() {
    let mut cache = SdfPreparedAtlasCache::default();
    let bake = empty_bake();
    cache.replace(UVec2::new(1, 1), &[], Some(7), &bake);

    assert!(cache
        .reuse(
            UVec2::new(1, 1),
            &[],
            Some(8),
            Default::default(),
            Default::default(),
        )
        .is_some());
}
