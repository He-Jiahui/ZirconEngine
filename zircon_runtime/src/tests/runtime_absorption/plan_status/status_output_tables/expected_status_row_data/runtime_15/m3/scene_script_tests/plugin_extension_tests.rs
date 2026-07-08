pub(super) type Slice = super::ExpectedStatusOutputSlice;

#[path = "plugin_extension_tests/export_build_rows.rs"]
mod export_build_rows;
#[path = "plugin_extension_tests/manifest_package_rows.rs"]
mod manifest_package_rows;
#[path = "plugin_extension_tests/native_loader_rows.rs"]
mod native_loader_rows;
#[path = "plugin_extension_tests/row_data_owner.rs"]
mod row_data_owner;
#[path = "plugin_extension_tests/runtime_catalog_rows.rs"]
mod runtime_catalog_rows;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    native_loader_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const MANIFEST_PACKAGE_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    manifest_package_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_CATALOG_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    runtime_catalog_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const EXPORT_BUILD_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    export_build_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    row_data_owner::EXPECTED_STATUS_OUTPUT_SLICES;
