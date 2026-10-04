#[test]
fn published_draw_proxy_is_the_only_non_test_bundle_projection() {
    let accessors = include_str!("../resource_streamer_accessors.rs");
    let proxy = include_str!("../published_material_draw_proxy.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("published material draw proxy test boundary");

    assert!(proxy.contains("prepared.published.as_ref()"));
    assert!(proxy.contains("prepared.previous_published.as_ref()"));
    assert!(proxy.contains("bundle.runtime"));
    assert!(proxy.contains("bundle.uniform"));
    assert!(proxy.contains("bundle.standard_uniform"));
    assert!(proxy.contains("bundle.textures"));
    assert!(proxy.contains("same_texture_revision"));
    assert!(!accessors.contains("pub(crate) fn published_material_uniform("));
    assert!(!accessors.contains("pub(crate) fn published_standard_material_uniform("));
}

#[test]
fn same_revision_accepts_mip_streaming_but_rejects_another_asset_generation() {
    assert!(super::same_texture_revision(Some(7), Some(7)));
    assert!(!super::same_texture_revision(Some(7), Some(8)));
    assert!(!super::same_texture_revision(None, Some(7)));
}

// BUG: [CR-GRAPHICS-SCENERES-0001] 此结构测试在成员访问跨行时仍查找连续的 prepared.published 等文本，三个断言恒为假；证据：上方函数使用 prepared 换行后再访问字段。
#[test]
fn resource_streamer_publishes_only_the_three_live_material_generation_slots() {
    let source = include_str!("../published_material_draw_proxy.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("published material draw proxy test boundary");
    let generations = source
        .split("fn material_draw_generations(")
        .nth(1)
        .expect("live material generation projection");

    assert!(generations.contains("prepared.published"));
    assert!(generations.contains("prepared.previous_published"));
    assert!(generations.contains("prepared.staged_candidate"));
    assert!(source.contains("fn staged_material_draw_generation("));
}
