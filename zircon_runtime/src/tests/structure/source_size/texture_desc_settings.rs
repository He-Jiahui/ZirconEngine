use super::super::rust_source_view::production_code_view;
use super::super::support::assert_contains_all_exact;
use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_texture_descriptor_settings_parser_is_child_owner() {
    let parent = read_runtime_src("asset/assets/texture/descriptor.rs");
    let settings = read_runtime_src("asset/assets/texture/descriptor/settings.rs");
    let current_anchor_owner = read_repo(
        "docs/plans/zircon_runtime/runtime/15/2026-07-17-descriptor-filter-plan-anchor-current-owner.md",
    );
    let parent_production = production_code_view(&parent);
    let settings_production = production_code_view(&settings);

    assert_contains_all(
        "texture descriptor parent keeps public descriptor behavior and delegates settings parsing",
        &parent_production,
        &[
            "mod settings;",
            "use self::settings::{",
            "pub struct TextureAssetDescriptor",
            "pub depth_or_array_layers: u32",
            "pub fn apply_import_settings(",
            "pub fn to_render_image_descriptor(",
            "fn normalize_extent_fields(",
            "fn apply_import_extent_settings(",
            "fn reject_retired_extent_settings(",
            "fn non_zero_extent_setting(",
            "reject_retired_extent_settings(settings)?;",
            "self.apply_import_extent_settings(depth, array_layers)?;",
        ],
    );
    for moved_owner in [
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
            !parent_production.contains(moved_owner),
            "asset/assets/texture/descriptor.rs should delegate {moved_owner} to descriptor/settings.rs"
        );
    }
    assert_contains_all(
        "texture descriptor settings child owns TOML parser helpers and sampler token normalization",
        &settings_production,
        &[
            "pub(super) fn u32_setting(",
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
    for (path, production) in [
        ("texture descriptor", parent_production.as_str()),
        ("texture settings parser", settings_production.as_str()),
    ] {
        for retired_owner in [
            "struct ExtentSettingKeys",
            "fn normalize_import_extent_fields(",
        ] {
            assert!(
                !production.contains(retired_owner),
                "{path} must not restore retired extent owner `{retired_owner}`"
            );
        }
    }

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
}
