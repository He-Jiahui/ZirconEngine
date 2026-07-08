use super::*;

#[rustfmt::skip]
const FOUNDATION_EXPECTED_SLICE_CHILD_STEMS: &[&str] = &["route_metadata_rows", "root_route_rows", "foundation_status_rows", "route_children_rows", "source_metadata_rows", "status_map_rows", "expected_slice_map_rows"];
#[rustfmt::skip]
const TYPED_ERROR_STATUS_DOC_CHILD_STEMS: &[&str] = &["base_status_doc_rows", "paths_inventory_rows", "delegation_rows", "status_maps_rows", "status_mirrors_rows", "expected_slice_map_rows"];

pub(super) fn extend_foundation_expected_slice_sources(sources: &mut Vec<String>, child: &str) {
    if child.ends_with("foundation_review_maps/expected_slice_rows.rs") {
        extend_child_stem_sources(sources, child, FOUNDATION_EXPECTED_SLICE_CHILD_STEMS);
    }
}

pub(super) fn extend_typed_error_status_doc_sources(sources: &mut Vec<String>, child: &str) {
    if child.ends_with("typed_error_maps/status_doc_rows.rs") {
        extend_child_stem_sources(sources, child, TYPED_ERROR_STATUS_DOC_CHILD_STEMS);
    }
}

fn extend_child_stem_sources(sources: &mut Vec<String>, child: &str, stems: &[&str]) {
    let child_dir = child.trim_end_matches(".rs");
    sources.extend(
        stems
            .iter()
            .map(|stem| read_runtime_src(&format!("{child_dir}/{stem}.rs"))),
    );
}
