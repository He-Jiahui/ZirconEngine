#[test]
fn heatmap_source_projection_reserves_array_capacity_and_keeps_filtering() {
    let source = include_str!("../weight_heatmap.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("weight heatmap implementation");

    assert!(implementation.contains("let mut sources = Vec::with_capacity(values.len())"));
    assert!(implementation.contains("let Some(source) = value.as_table() else"));
    assert!(implementation.contains("let (Some(x), Some(y), Some(weight))"));
    assert!(implementation.contains("sources.push(WeightHeatmapSource::new("));
}
