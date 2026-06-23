use super::{assert_contains_all, repo_path, runtime_src_path};

#[test]
fn runtime_15_post_process_volume_component_is_folder_backed() {
    let parent = read_runtime_src("core/framework/render/post_process/volume_component.rs");
    let params = read_runtime_src("core/framework/render/post_process/volume_component/params.rs");
    let tests = read_runtime_src("core/framework/render/post_process/volume_component/tests.rs");
    let plan_07 = read_repo("docs/plans/zircon_runtime/render/07-postprocess-color-pipeline.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let post_process_doc =
        read_repo("docs/zircon_runtime/graphics/scene/scene_renderer/post_process/index.md");

    assert_contains_all(
        "volume component parent delegates params and tests",
        &parent,
        &[
            "mod params;",
            "pub use self::params::{",
            "pub struct VolumeComponentDescriptor",
            "pub const BUILTIN_POST_PROCESS_VOLUME_COMPONENTS",
            "#[cfg(test)]",
            "mod tests;",
        ],
    );
    for moved_owner in [
        "pub enum VolumeParamType",
        "pub enum VolumeParamValue",
        "pub struct VolumeParamSchema",
        "pub fn interp_float_lerp",
        "const fn lerp",
        "mod tests {",
        "const TEST_PARAMS",
        "fn render_volume_component_descriptor_applies_authored_values",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "post_process/volume_component.rs should delegate {moved_owner} to volume_component/params.rs or volume_component/tests.rs"
        );
    }

    assert_contains_all(
        "volume component params owns value and interpolation contracts",
        &params,
        &[
            "pub type VolumeParamInterpFn",
            "pub enum VolumeParamType",
            "pub enum VolumeParamValue",
            "pub struct VolumeParamSchema",
            "pub fn interp_float_lerp",
            "pub fn interp_vec3_lerp",
            "pub fn interp_discrete",
            "pub(super) const fn float_param",
            "const fn lerp",
        ],
    );
    assert_contains_all(
        "volume component tests preserve registry and descriptor behavior",
        &tests,
        &[
            "const TEST_PARAMS",
            "fn read_test_value",
            "fn apply_test_value",
            "fn render_volume_param_interp_blends_float_vec3_and_discrete_values",
            "fn render_volume_component_descriptor_applies_defaults_to_resolved_stack",
            "fn render_volume_component_descriptor_applies_authored_values",
            "fn render_volume_component_descriptor_applies_exposure_values",
            "fn render_volume_component_descriptor_rejects_bad_value_shape",
        ],
    );
    assert_eq!(
        tests.matches("#[test]").count(),
        5,
        "volume component child test owner should preserve the 5 moved tests"
    );

    for (path, source, budget) in [
        (
            "core/framework/render/post_process/volume_component.rs",
            parent.as_str(),
            700,
        ),
        (
            "core/framework/render/post_process/volume_component/params.rs",
            params.as_str(),
            250,
        ),
        (
            "core/framework/render/post_process/volume_component/tests.rs",
            tests.as_str(),
            300,
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < budget,
            "{path} should stay below the owner budget {budget}; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Plan 07", plan_07.as_str()),
        ("render index", render_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("post-process module doc", post_process_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Plan 07 volume component owner split",
                "render_plan07_volume_component_owner_split_static_passed",
                "core/framework/render/post_process/volume_component/params.rs",
                "core/framework/render/post_process/volume_component/tests.rs",
                "runtime_15_post_process_volume_component_is_folder_backed",
            ],
        );
    }
}

fn read_runtime_src(relative: &str) -> String {
    std::fs::read_to_string(runtime_src_path(relative))
        .unwrap_or_else(|error| panic!("failed to read runtime source `{relative}`: {error}"))
}

fn read_repo(relative: &str) -> String {
    std::fs::read_to_string(repo_path(relative))
        .unwrap_or_else(|error| panic!("failed to read repository file `{relative}`: {error}"))
}
