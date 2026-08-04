use super::*;

#[test]
fn runtime_15_rhi_device_contract_tests_are_folder_backed() {
    let parent = read_repo("zircon_runtime/crates/zr_rhi_wgpu/src/tests/device_contract.rs");
    let basic_resources =
        read_repo("zircon_runtime/crates/zr_rhi_wgpu/src/tests/device_contract/basic_resources.rs");
    let texture_sampler_descriptors = read_repo(
        "zircon_runtime/crates/zr_rhi_wgpu/src/tests/device_contract/texture_sampler_descriptors.rs",
    );
    let bind_groups =
        read_repo("zircon_runtime/crates/zr_rhi_wgpu/src/tests/device_contract/bind_groups.rs");
    let invalid_descriptors = read_repo(
        "zircon_runtime/crates/zr_rhi_wgpu/src/tests/device_contract/invalid_descriptors.rs",
    );
    let transfer_and_fences = read_repo(
        "zircon_runtime/crates/zr_rhi_wgpu/src/tests/device_contract/transfer_and_fences.rs",
    );
    let framework_boundary = read_repo(
        "zircon_runtime/crates/zr_rhi_wgpu/src/tests/device_contract/framework_boundary.rs",
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    assert_contains_all(
        "RHI device contract parent test module mounts",
        &parent,
        &[
            "mod basic_resources;",
            "mod bind_groups;",
            "mod framework_boundary;",
            "mod invalid_descriptors;",
            "mod texture_sampler_descriptors;",
            "mod transfer_and_fences;",
            "fn test_bind_group_layout_desc",
            "fn create_test_pipeline_layout",
        ],
    );

    for moved_guard in [
        "fn rhi_handles_are_stable_raw_identifiers",
        "fn wgpu_rhi_roundtrips_bind_group_layouts_and_bind_groups",
        "fn wgpu_rhi_rejects_invalid_resource_descriptors",
        "fn wgpu_rhi_write_copy_and_read_buffer_preserves_bytes",
        "fn app_editor_and_core_framework_sources_do_not_import_wgpu",
    ] {
        assert!(
            !parent.contains(moved_guard),
            "rhi/tests/device_contract.rs should mount child test owners instead of defining {moved_guard}"
        );
    }

    assert_contains_all(
        "RHI device contract basic child owns handle/resource contracts",
        &basic_resources,
        &[
            "fn rhi_handles_are_stable_raw_identifiers",
            "fn buffer_and_texture_usage_flags_are_composable",
            "fn wgpu_rhi_device_allocates_stable_resource_handles_and_fences",
        ],
    );
    assert_contains_all(
        "RHI device contract texture/sampler child owns descriptor roundtrips",
        &texture_sampler_descriptors,
        &[
            "fn wgpu_rhi_rejects_sparse_reserved_texture_without_backend_support",
            "fn wgpu_rhi_roundtrips_hdr_array_and_cube_texture_descriptors",
            "fn wgpu_rhi_device_roundtrips_resource_descriptors_by_handle",
            "fn wgpu_rhi_roundtrips_shadow_and_trilinear_sampler_descriptors",
        ],
    );
    assert_contains_all(
        "RHI device contract bind-group child owns layout/resource validation",
        &bind_groups,
        &[
            "fn wgpu_rhi_roundtrips_bind_group_layouts_and_bind_groups",
            "fn wgpu_rhi_rejects_invalid_bind_group_layout_descriptors",
            "fn wgpu_rhi_bind_group_validation_checks_layout_resource_types_and_usage",
        ],
    );
    assert_contains_all(
        "RHI device contract invalid descriptor child owns descriptor rejection matrix",
        &invalid_descriptors,
        &["fn wgpu_rhi_rejects_invalid_resource_descriptors"],
    );
    assert_contains_all(
        "RHI device contract transfer child owns fence/buffer IO contracts",
        &transfer_and_fences,
        &[
            "fn wgpu_rhi_fence_queries_reject_unissued_fence_values",
            "fn wgpu_rhi_write_copy_and_read_buffer_preserves_bytes",
            "fn wgpu_rhi_write_buffer_validates_usage_and_range",
            "fn wgpu_rhi_read_texture_validates_usage",
            "fn wgpu_rhi_read_buffer_validates_usage_and_range",
        ],
    );
    assert_contains_all(
        "RHI device contract framework child owns wgpu import boundary",
        &framework_boundary,
        &[
            "fn app_editor_and_core_framework_sources_do_not_import_wgpu",
            "fn collect_wgpu_imports",
        ],
    );

    for (path, source, expected_test_count) in [
        (
            "rhi/tests/device_contract/basic_resources.rs",
            basic_resources.as_str(),
            3,
        ),
        (
            "rhi/tests/device_contract/texture_sampler_descriptors.rs",
            texture_sampler_descriptors.as_str(),
            4,
        ),
        (
            "rhi/tests/device_contract/bind_groups.rs",
            bind_groups.as_str(),
            3,
        ),
        (
            "rhi/tests/device_contract/invalid_descriptors.rs",
            invalid_descriptors.as_str(),
            2,
        ),
        (
            "rhi/tests/device_contract/transfer_and_fences.rs",
            transfer_and_fences.as_str(),
            5,
        ),
        (
            "rhi/tests/device_contract/framework_boundary.rs",
            framework_boundary.as_str(),
            1,
        ),
    ] {
        assert_eq!(
            source.matches("#[test]").count(),
            expected_test_count,
            "{path} should keep all migrated test functions executable"
        );
    }

    for (path, source) in [
        ("rhi/tests/device_contract.rs", parent.as_str()),
        (
            "rhi/tests/device_contract/basic_resources.rs",
            basic_resources.as_str(),
        ),
        (
            "rhi/tests/device_contract/texture_sampler_descriptors.rs",
            texture_sampler_descriptors.as_str(),
        ),
        (
            "rhi/tests/device_contract/bind_groups.rs",
            bind_groups.as_str(),
        ),
        (
            "rhi/tests/device_contract/invalid_descriptors.rs",
            invalid_descriptors.as_str(),
        ),
        (
            "rhi/tests/device_contract/transfer_and_fences.rs",
            transfer_and_fences.as_str(),
        ),
        (
            "rhi/tests/device_contract/framework_boundary.rs",
            framework_boundary.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
