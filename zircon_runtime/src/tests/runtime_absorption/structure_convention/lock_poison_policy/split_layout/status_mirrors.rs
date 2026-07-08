use super::super::support::*;
use super::sources::LockPoisonSources;

pub(super) fn assert_lock_poison_status_mirrors(sources: &LockPoisonSources) {
    for (label, source) in [
        ("Runtime 15 plan", sources.runtime_15_plan.as_str()),
        ("Runtime index", sources.runtime_index.as_str()),
        ("review findings", sources.review_findings.as_str()),
        (
            "structure convention",
            sources.structure_convention.as_str(),
        ),
        ("module convention doc", sources.module_doc.as_str()),
        ("session note", sources.session_note.as_str()),
        (
            "status-output lock-poison row data",
            sources.lock_poison_status_rows.as_str(),
        ),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 lock poison policy guard folder split",
                "runtime_15_lock_poison_policy_guard_folder_split_static_passed_cargo_deferred",
                "structure_convention/lock_poison_policy.rs",
                "structure_convention/lock_poison_policy/core_runtime.rs",
                "structure_convention/lock_poison_policy/runtime_services.rs",
                "structure_convention/lock_poison_policy/asset_render_input.rs",
                "runtime_15_lock_poison_policy_guard_is_folder_backed",
            ],
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", sources.runtime_15_plan.as_str()),
        ("Runtime index", sources.runtime_index.as_str()),
        ("review findings", sources.review_findings.as_str()),
        (
            "structure convention",
            sources.structure_convention.as_str(),
        ),
        ("module convention doc", sources.module_doc.as_str()),
        ("Frameworks02 plan", sources.frameworks_plan.as_str()),
        ("session note", sources.session_note.as_str()),
        (
            "status-output lock-poison policy guard rows",
            sources.lock_poison_policy_guard_rows.as_str(),
        ),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 lock-poison split-layout guard folder-backed split",
                "runtime_15_lock_poison_split_layout_guard_folder_backed_static_passed_cargo_deferred",
                "structure_convention/lock_poison_policy/split_layout.rs",
                "structure_convention/lock_poison_policy/split_layout/folder_backing.rs",
                "structure_convention/lock_poison_policy/split_layout/status_mirrors.rs",
                "runtime_15_lock_poison_split_layout_guard_is_folder_backed",
            ],
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", sources.runtime_15_plan.as_str()),
        ("Runtime index", sources.runtime_index.as_str()),
        ("review findings", sources.review_findings.as_str()),
        (
            "structure convention",
            sources.structure_convention.as_str(),
        ),
        ("module convention doc", sources.module_doc.as_str()),
        ("Frameworks02 plan", sources.frameworks_plan.as_str()),
        ("session note", sources.session_note.as_str()),
        (
            "status-output lock-poison row data",
            sources.lock_poison_status_rows.as_str(),
        ),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 lock poison policy route-owner split",
                "runtime_15_lock_poison_policy_route_owner_split_static_passed_cargo_deferred",
                "structure_convention/lock_poison_policy.rs",
                "structure_convention/lock_poison_policy/split_layout.rs",
                "structure_convention/lock_poison_policy/support.rs",
                "runtime_15_lock_poison_policy_guard_is_folder_backed",
            ],
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", sources.runtime_15_plan.as_str()),
        ("Runtime index", sources.runtime_index.as_str()),
        ("review findings", sources.review_findings.as_str()),
        (
            "structure convention",
            sources.structure_convention.as_str(),
        ),
        ("module convention doc", sources.module_doc.as_str()),
        ("session note", sources.session_note.as_str()),
        (
            "status-output lock-poison row data",
            sources.lock_poison_status_rows.as_str(),
        ),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 asset/render/input lock-poison guard child-owner split",
                "runtime_15_asset_render_input_lock_poison_guard_child_owner_split_static_passed_cargo_deferred",
                "structure_convention/lock_poison_policy/asset_render_input.rs",
                "structure_convention/lock_poison_policy/asset_render_input/asset_pipeline.rs",
                "structure_convention/lock_poison_policy/asset_render_input/render_animation.rs",
                "structure_convention/lock_poison_policy/asset_render_input/input_script.rs",
                "runtime_15_asset_render_input_lock_poison_guard_child_owner_split",
            ],
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", sources.runtime_15_plan.as_str()),
        ("Runtime index", sources.runtime_index.as_str()),
        ("review findings", sources.review_findings.as_str()),
        (
            "structure convention",
            sources.structure_convention.as_str(),
        ),
        ("module convention doc", sources.module_doc.as_str()),
        ("session note", sources.session_note.as_str()),
        (
            "status-output lock-poison row data",
            sources.lock_poison_status_rows.as_str(),
        ),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 runtime services lock-poison guard child-owner split",
                "runtime_15_runtime_services_lock_poison_guard_child_owner_split_static_passed_cargo_deferred",
                "structure_convention/lock_poison_policy/runtime_services.rs",
                "structure_convention/lock_poison_policy/runtime_services/plugin_bridge.rs",
                "structure_convention/lock_poison_policy/runtime_services/dynamic_scene.rs",
                "structure_convention/lock_poison_policy/runtime_services/navigation_resource.rs",
                "runtime_15_runtime_services_lock_poison_guard_child_owner_split",
            ],
        );
    }
}
