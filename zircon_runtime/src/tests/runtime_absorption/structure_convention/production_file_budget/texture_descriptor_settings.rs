use super::super::support::assert_contains_all_exact;
use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_texture_descriptor_settings_parser_is_child_owner() {
    let parent = read_runtime_src("asset/assets/texture/descriptor.rs");
    let settings = read_runtime_src("asset/assets/texture/descriptor/settings.rs");
    let current_anchor_owner = read_repo(
        "docs/plans/zircon_runtime/runtime/15/2026-07-17-descriptor-filter-plan-anchor-current-owner.md",
    );
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let importer_doc = read_repo("docs/zircon_runtime/asset/importer.md");
    let render_assets_doc = read_repo("docs/zircon_runtime/asset/render-assets.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m4.rs",
    );
    let status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m4_surface_cleanup.rs",
    );
    let date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m4_surface_cleanup.rs",
    );

    assert_contains_all(
        "texture descriptor parent keeps public descriptor behavior and delegates settings parsing",
        &parent,
        &[
            "mod settings;",
            "use self::settings::{",
            "pub struct TextureAssetDescriptor",
            "pub fn apply_import_settings(",
            "pub fn to_render_image_descriptor(",
            "fn normalize_extent_fields(",
            "fn normalize_import_extent_fields(",
        ],
    );
    for moved_owner in [
        "struct ExtentSettingKeys",
        "fn parse_usage_list(",
        "fn parse_asset_usage_list(",
        "fn parse_sampler(",
        "fn parse_array_layout(",
        "fn parse_color_space(",
        "fn parse_dimension(",
        "fn parse_address_mode(",
        "fn normalized_token(",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "asset/assets/texture/descriptor.rs should delegate {moved_owner} to descriptor/settings.rs"
        );
    }
    assert_contains_all(
        "texture descriptor settings child owns TOML parser helpers and sampler token normalization",
        &settings,
        &[
            "pub(super) struct ExtentSettingKeys",
            "pub(super) fn parse_usage_list(",
            "pub(super) fn parse_asset_usage_list(",
            "pub(super) fn parse_sampler(",
            "pub(super) fn parse_array_layout(",
            "pub(super) fn parse_color_space(",
            "pub(super) fn parse_dimension(",
            "fn parse_address_mode(",
            "fn normalized_token(",
        ],
    );

    for (path, source) in [
        ("asset/assets/texture/descriptor.rs", parent.as_str()),
        (
            "asset/assets/texture/descriptor/settings.rs",
            settings.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 production-file soft budget; got {line_count} lines"
        );
    }

    assert_contains_all_exact(
        "Runtime 15 descriptor-filter current child owner",
        &current_anchor_owner,
        &[
            "Runtime 15 M4 texture descriptor settings parser owner split",
            "runtime_15_texture_descriptor_settings_parser_owner_split_static_passed_cargo_deferred",
            "asset/assets/texture/descriptor.rs",
            "asset/assets/texture/descriptor/settings.rs",
            "runtime_15_texture_descriptor_settings_parser_is_child_owner",
            "2026-06-24",
        ],
    );
    for (label, source) in [
        ("module convention doc", module_doc.as_str()),
        ("importer doc", importer_doc.as_str()),
        ("render assets doc", render_assets_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all_exact(
            label,
            source,
            &[
                "Runtime 15 M4 texture descriptor settings parser owner split",
                "runtime_15_texture_descriptor_settings_parser_owner_split_static_passed_cargo_deferred",
                "asset/assets/texture/descriptor.rs",
                "asset/assets/texture/descriptor/settings.rs",
                "runtime_15_texture_descriptor_settings_parser_is_child_owner",
            ],
        );
    }
    assert_contains_all_exact(
        "status-output status map",
        &status_map,
        &[
            "Runtime 15 M4 texture descriptor settings parser owner split",
            "runtime_15_texture_descriptor_settings_parser_owner_split_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all_exact(
        "status-output date map",
        &date_map,
        &[
            "Runtime 15 M4 texture descriptor settings parser owner split",
            "2026-06-24",
        ],
    );
}
