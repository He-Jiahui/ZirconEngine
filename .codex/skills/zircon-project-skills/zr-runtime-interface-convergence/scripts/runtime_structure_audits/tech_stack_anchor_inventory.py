from __future__ import annotations


EXPECTED_TECH_STACK_GUARD_COUNT = 12

TEXT_STACK_DOC_ANCHORS = (
    "## Backend Responsibility Matrix",
    "Shaping, segmentation, layout, and measurement",
    "Font registry, raster, and SDF policy",
    "GPU/native text submission",
    "SharedTextService",
    "shared_text_shaper_matches_public_layout_entrypoint",
    "text_shaper_stack_uses_shared_text_service_for_font_backends",
)
TECH_STACK_DOC_ANCHORS = (
    "## Dependency Matrix",
    "## Corrected Non-Dependencies",
    "## Prerelease Version Governance",
    "## External ZrVM Path Dependency",
    "## Export Archive Decision",
    "## Text Stack Boundary",
    "runtime_01_tech_stack_cargo_gate_stays_visible_until_dependency_validation",
    "winit 0.31.0-beta.2",
    "notify 9.0.0-rc.3",
    "../../zr_vm",
    "non-null pointer with length `0`",
    "zip 9.0.0-pre2",
    "export archive materializer",
    "ZIP archive materialization is implemented",
    "replacement implementation of `UiTextShaper`",
    "Runtime Editor-Only Dependency Backlog",
)
TECH_STACK_GUARDS = (
    "runtime_tech_stack_doc_exists_and_is_linked_from_architecture_index",
    "runtime_manifest_keeps_pinned_prerelease_versions_until_upgrade_gate",
    "zr_vm_path_dependency_gate_is_documented_with_version_pairing",
    "interface_and_editor_dependency_boundaries_stay_documented_and_guarded",
    "removed_or_editor_only_dependencies_do_not_silently_enter_runtime_stack",
    "export_archive_policy_allows_zip_only_for_archive_materializer",
    "physics_backend_option_decision_keeps_jolt_feature_gated_and_plugin_owned",
    "editor_only_dependency_candidates_have_editor_backlog_owner",
    "fontdue_editor_retained_host_dependency_has_migration_owner",
    "complex_text_backends_can_only_enter_through_ui_text_shaper",
    "runtime_text_doc_records_three_layer_stack_and_cross_reference",
    "runtime_01_tech_stack_mirror_docs_match_structure_audit_counts",
)
TECH_STACK_BEHAVIOR_TEST_ANCHORS = (
    "shared_text_shaper_matches_public_layout_entrypoint",
    "text_shaper_stack_uses_shared_text_service_for_font_backends",
    "empty_jolt_feature_slot_reports_unavailable_not_ready",
    "unavailable_jolt_backend_does_not_fallback_to_builtin_scene_tick",
    "linked_jolt_backend_reports_ready",
    "linked_jolt_backend_ticks_scene_without_builtin_fallback",
)
MIRROR_DOCS_GUARD = "runtime_01_tech_stack_mirror_docs_match_structure_audit_counts"
PHYSICS_DECISION_ANCHORS = (
    "selected native backend",
    "optional `joltc-sys`",
    "Neither path silently downgrades to builtin stepping",
    "No Rapier dependency is introduced",
    "no concrete physics library is added to `zircon_runtime`",
)
EDITOR_BACKLOG_ANCHORS = (
    "fontdue",
    "rfd",
    "arboard",
    "zircon_editor/src/ui/host",
    "Do not add to `zircon_runtime`",
    "zircon_runtime_interface",
)
CARGO_GATE_ANCHORS = (
    "cargo test -p zircon_runtime --lib tech_stack --locked -- --nocapture",
    "cargo test -p zircon_runtime --lib extensions --locked",
    "cargo test -p zircon_runtime --lib text_shaper --locked -- --nocapture",
    "cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_physics_runtime --locked",
    "runtime_01_tech_stack_cargo_gate_stays_visible_until_dependency_validation",
)
