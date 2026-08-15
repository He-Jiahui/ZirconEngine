fn section_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split(start)
        .nth(1)
        .and_then(|text| text.split(end).next())
        .unwrap_or_else(|| panic!("read section from {start} to {end}"))
}

#[test]
fn node_record_uses_direct_generic_storage_lookup_branches() {
    let source = include_str!("../world/records.rs");
    let node_record = section_between(source, "pub fn node_record", "pub fn insert_node_record");

    assert!(
        node_record.contains("let parent = match self.get::<Hierarchy>(entity)")
            && node_record.contains("Some(hierarchy) => hierarchy.parent")
            && node_record.contains("None => None")
            && !node_record.contains(".and_then(|hierarchy| hierarchy.parent)"),
        "node_record parent projection must branch directly on hierarchy presence"
    );
    assert!(
        node_record.contains("let transform = match self.get::<LocalTransform>(entity)")
            && node_record.contains("Some(local) => local.transform")
            && node_record.contains("None => LocalTransform::default().transform")
            && !node_record.contains(".copied().unwrap_or_default().transform"),
        "node_record transform projection must branch directly before the default transform fallback"
    );
    assert!(
        node_record.contains("let active = match self.get::<ActiveSelf>(entity)")
            && node_record.contains("Some(active) => active.0")
            && node_record.contains("None => ActiveSelf::default().0")
            && !node_record.contains(".copied().unwrap_or_default().0"),
        "node_record active projection must branch directly before the default-active fallback"
    );
    assert!(
        node_record.contains("let render_layer_mask = match self.get::<RenderLayerMask>(entity)")
            && node_record.contains("Some(mask) => mask.0")
            && node_record.contains("None => RenderLayerMask::default().0")
            && !node_record.contains(".copied().unwrap_or_default().0"),
        "node_record render-layer projection must branch directly before the default-mask fallback"
    );
    assert!(
        node_record.contains("let mobility = match self.get::<Mobility>(entity)")
            && node_record.contains("Some(mobility) => *mobility")
            && node_record.contains("None => Mobility::default()")
            && !node_record.contains(".copied().unwrap_or_default()"),
        "node_record mobility projection must branch directly before the default mobility fallback"
    );
    assert!(
        node_record.contains("parent,")
            && node_record.contains("transform,")
            && node_record.contains("active,")
            && node_record.contains("render_layer_mask,")
            && node_record.contains("mobility,"),
        "node_record must populate the snapshot from the precomputed direct-branch values"
    );
}
