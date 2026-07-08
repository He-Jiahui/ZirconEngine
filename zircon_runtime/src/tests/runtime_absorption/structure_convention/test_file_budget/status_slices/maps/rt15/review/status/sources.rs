use super::*;

#[path = "sources/m3_m4_maps.rs"]
mod m3_m4_maps;

pub(super) use m3_m4_maps::*;

pub(super) fn read_runtime_sources(paths: &[&str]) -> String {
    paths
        .iter()
        .map(|path| read_runtime_src(path))
        .collect::<Vec<_>>()
        .join("\n")
}

pub(super) fn read_status_support_expected_slice_rows() -> String {
    std::iter::once(STATUS_SUPPORT_EXPECTED_SLICE_ROWS)
        .chain(std::iter::once(STATUS_SUPPORT_EXPECTED_SLICE_ROWS_CHILD))
        .chain(STATUS_SUPPORT_EXPECTED_SLICE_ROW_CHILDREN.iter().copied())
        .map(read_runtime_src)
        .collect::<Vec<_>>()
        .join("\n")
}

pub(super) fn read_runtime_absorption_sources(paths: &[&str]) -> String {
    paths
        .iter()
        .map(|path| read_runtime_src(&format!("tests/runtime_absorption/{path}")))
        .collect::<Vec<_>>()
        .join("\n")
}
