use zircon_runtime::core::math::Vec3;

use super::*;

#[test]
fn trace_page_table_uses_clipmap_origin_for_sampleable_pages() {
    let mut scene = HybridGiGlobalSdfSceneState::default();
    scene.synchronize(Vec3::ZERO, &[], 4);
    let requests = scene.dirty_page_build_requests();
    scene.commit_pages(&requests);

    let table = build_trace_page_table(&scene);

    assert_eq!(table.page_count, requests.len() as u32);
    for request in requests {
        let index = trace_page_table_index(request.key(), &table.clipmaps)
            .expect("resident page must map into its clipmap table");
        assert_eq!(table.slots[index], request.atlas_slot());
    }
}

#[test]
fn trace_page_table_does_not_publish_uninitialized_pages() {
    let mut scene = HybridGiGlobalSdfSceneState::default();
    scene.synchronize(Vec3::ZERO, &[], 1);
    let request = scene.dirty_page_build_requests()[0];

    let table = build_trace_page_table(&scene);
    let index = trace_page_table_index(request.key(), &table.clipmaps)
        .expect("resident page must map into its clipmap table");

    assert_eq!(table.page_count, 0);
    assert_eq!(table.slots[index], GLOBAL_SDF_TRACE_PAGE_UNAVAILABLE_SLOT);
}

#[test]
fn trace_page_table_keeps_typed_fallback_pages_unavailable() {
    let mut scene = HybridGiGlobalSdfSceneState::default();
    scene.synchronize(Vec3::ZERO, &[], 1);
    let requests = scene.dirty_page_build_requests();
    let fallback = requests[0];
    scene.resolve_pages_to_fallback(&[fallback]);

    let table = build_trace_page_table(&scene);
    let index = trace_page_table_index(fallback.key(), &table.clipmaps)
        .expect("resident fallback page remains in the clipmap domain");

    assert_eq!(table.page_count, 0);
    assert_eq!(table.slots[index], GLOBAL_SDF_TRACE_PAGE_UNAVAILABLE_SLOT);
}

#[test]
fn trace_clipmap_abi_matches_wgsl_storage_layout() {
    assert_eq!(std::mem::size_of::<GlobalSdfGpuTraceClipmap>(), 32);
    assert_eq!(
        std::mem::offset_of!(GlobalSdfGpuTraceClipmap, page_coordinate_origin_and_padding),
        0
    );
    assert_eq!(
        std::mem::offset_of!(GlobalSdfGpuTraceClipmap, page_world_size_and_padding),
        16
    );
}
