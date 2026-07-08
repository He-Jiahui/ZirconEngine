#[path = "foundation/asset_provider_cleanup.rs"]
mod asset_provider_cleanup;
#[path = "foundation/core_cleanup.rs"]
mod core_cleanup;
#[path = "foundation/graphics_diagnostics.rs"]
mod graphics_diagnostics;
#[path = "foundation/lock_poison.rs"]
mod lock_poison;
#[path = "foundation/map_rows.rs"]
mod map_rows;
#[path = "foundation/typed_error_core.rs"]
mod typed_error_core;
#[path = "foundation/typed_error_plugin.rs"]
mod typed_error_plugin;

// Route-level mirrors for runtime dead-code guard dates; concrete rows stay in core_cleanup.rs:
// Runtime 15 M5 production dead-code suppression global gate
// Runtime 15 F12 production dead-code current-state wording cleanup
// 2026-06-30
// Runtime 15 F12 UI text edit-state dead-code suppression cleanup
// 2026-06-27

pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    map_rows::expected_date_for_slice(slice)
        .or_else(|| core_cleanup::expected_date_for_slice(slice))
        .or_else(|| lock_poison::expected_date_for_slice(slice))
        .or_else(|| graphics_diagnostics::expected_date_for_slice(slice))
        .or_else(|| typed_error_core::expected_date_for_slice(slice))
        .or_else(|| typed_error_plugin::expected_date_for_slice(slice))
        .or_else(|| asset_provider_cleanup::expected_date_for_slice(slice))
}
