#[path = "runtime_15/foundation.rs"]
mod foundation;
#[path = "runtime_15/m3_structure_support.rs"]
mod m3_structure_support;
#[path = "runtime_15/m4_surface_cleanup.rs"]
mod m4_surface_cleanup;
#[path = "runtime_15/naming_boundary.rs"]
mod naming_boundary;

pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    foundation::expected_date_for_slice(slice)
        .or_else(|| naming_boundary::expected_date_for_slice(slice))
        .or_else(|| m4_surface_cleanup::expected_date_for_slice(slice))
        .or_else(|| m3_structure_support::expected_date_for_slice(slice))
}

// Runtime 15 M2 UI editor showcase descriptor builders module naming hard cutover.
// Status: runtime_15_ui_editor_showcase_descriptor_builders_naming_hard_cutover_static_passed_cargo_deferred.
// File: ui/component/catalog/editor_showcase/descriptor_builders.rs.
// Guard: runtime_15_ui_editor_showcase_descriptor_builders_use_owner_name.
