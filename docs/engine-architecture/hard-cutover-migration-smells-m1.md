---
related_code:
  - zircon_runtime/src/ui/surface/input/dispatch.rs
  - zircon_runtime/src/ui/surface/input/navigation.rs
  - zircon_runtime/src/ui/surface/input/pointer.rs
  - zircon_runtime/src/ui/surface/input/pointer_reply.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/post_process_sources/encode_hybrid_gi_probes/runtime_parent_chain.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/gpu_resources/execute_prepare/execute/collect_inputs.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/gpu_resources/execute_prepare/probe_quantization.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/resolve_enabled_features.rs
  - zircon_runtime/src/graphics/scene/render_product_streamer_tests/material_runtime.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/runtime_features/runtime_features_from_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs
  - zircon_runtime/src/asset/assets/animation/mod.rs
  - zircon_runtime/src/asset/assets/animation/binary.rs
  - zircon_runtime/src/asset/assets/animation/clip.rs
  - zircon_runtime/src/asset/assets/animation/graph.rs
  - zircon_runtime/src/asset/assets/animation/sequence.rs
  - docs/zircon_runtime/asset/assets/animation.md
  - zircon_runtime/src/asset/importer/registry.rs
  - zircon_runtime/src/asset/importer/ingest/asset_importer.rs
  - zircon_plugins/texture_importer/runtime/src/container/dds.rs
  - zircon_editor/src/ui/asset_editor/session/command_entry.rs
  - zircon_editor/src/ui/asset_editor/session/lifecycle.rs
  - zircon_editor/src/ui/asset_editor/session/ui_asset_editor_session.rs
  - zircon_editor/src/ui/template_runtime/runtime/build_session.rs
  - zircon_runtime/src/ui/template/asset/schema/migrator.rs
  - zircon_runtime/src/ui/template/asset/schema/flat_nodes.rs
  - zircon_runtime/src/ui/template/asset/schema/source_template_fixture.rs
  - zircon_runtime/src/ui/template/asset/compiler/style_apply.rs
  - zircon_editor/src/ui/layouts/views/view_projection.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_table_rows.rs
  - zircon_runtime/src/ui/layout/pass/taffy_arrange.rs
  - zircon_runtime_interface/src/ui/pipeline/stage.rs
  - zircon_runtime_interface/src/tests/pipeline_contracts.rs
  - docs/zircon_runtime_interface/ui/pipeline.md
  - zircon_hub/src/state/hub_message/message.rs
  - zircon_hub/src/tauri_app/runtime_state/build_actions.rs
  - zircon_plugins/net/features/http/runtime/src/backend/client.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/hard_cutover_migration_smells.py
implementation_files:
  - zircon_editor/src/ui/template_runtime/runtime/build_session.rs
  - zircon_editor/src/ui/asset_editor/session/ui_asset_editor_session.rs
  - zircon_editor/src/ui/asset_editor/session/lifecycle.rs
  - zircon_editor/src/ui/asset_editor/session/command_entry.rs
  - zircon_runtime/src/ui/template/asset/schema/mod.rs
  - zircon_runtime/src/ui/template/asset/schema/source_template_fixture.rs
  - zircon_runtime/src/ui/template/asset/schema/migrator.rs
  - zircon_runtime/src/ui/template/asset/schema/flat_nodes.rs
  - zircon_runtime/src/ui/template/asset/compiler/style_apply.rs
  - zircon_runtime/src/ui/tests/asset_schema_migration.rs
  - zircon_runtime/src/ui/tests/asset.rs
  - zircon_runtime/src/ui/tests/asset_contract_spine.rs
  - zircon_runtime/src/ui/layout/pass/taffy_arrange.rs
  - zircon_runtime_interface/src/ui/pipeline/stage.rs
  - zircon_runtime_interface/src/tests/pipeline_contracts.rs
  - docs/zircon_runtime/ui/layout/pass.md
  - docs/ui-and-layout/ui-asset-documents-and-editor-protocol.md
  - docs/editor-and-tooling/ui-asset-editor-host-session.md
  - docs/zircon_editor/ui/template_runtime/runtime_host.md
  - docs/zircon_runtime/ui/v2.md
  - docs/ui-and-layout/shared-ui-template-runtime.md
  - docs/ui-and-layout/bevy-ui-text-widgets-focus-a11y-m0-gap-audit.md
  - docs/zircon_runtime_interface/ui/pipeline.md
  - docs/engine-architecture/hard-cutover-migration-smells-m1.md
  - docs/zircon_runtime/asset/assets/animation.md
  - docs/engine-architecture/runtime-architecture-review-m0.md
  - docs/engine-architecture/runtime-interface-convergence.md
  - docs/engine-architecture/index.md
  - docs/engine-architecture/runtime-reference-engine-evidence.md
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/hard_cutover_migration_smells.py
  - .codex/sessions/20260604-1232-runtime-architecture-review.md
plan_sources:
  - user: 2026-06-04 optimize Zircon Engine runtime architecture with breaking changes allowed
  - .codex/plans/Zircon Runtime 架构渐进式 Review 与优化计划.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/全系统重构方案.md
  - docs/engine-architecture/runtime-reference-engine-evidence.md
tests:
  - python -m py_compile .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/hard_cutover_migration_smells.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py --json
  - python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - hard_cutover_migration_smells gate status, explicit count fields, classification count, migration debt, allowed bridge count, and unclassified reference checks
  - rustfmt --edition 2021 --check zircon_runtime\src\ui\template\asset\schema\mod.rs zircon_runtime\src\ui\template\asset\schema\source_template_fixture.rs zircon_runtime\src\ui\template\asset\schema\migrator.rs zircon_runtime\src\ui\template\asset\schema\flat_nodes.rs zircon_runtime\src\ui\template\asset\compiler\style_apply.rs zircon_runtime\src\ui\tests\asset_schema_migration.rs zircon_runtime\src\ui\tests\asset.rs zircon_runtime\src\ui\tests\asset_contract_spine.rs
  - rustfmt --edition 2021 --check zircon_runtime\src\ui\layout\pass\taffy_arrange.rs
  - rustfmt --edition 2021 --check zircon_runtime_interface\src\ui\pipeline\stage.rs zircon_runtime_interface\src\tests\pipeline_contracts.rs
  - cargo test -p zircon_runtime_interface pipeline_contracts --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-interface-stage-policy-0604 --message-format short --color never -- --test-threads=1 --nocapture
doc_type: milestone-detail
---

# Hard-Cutover Migration Smells M1 Gate

## Purpose

This document turns hard-cutover migration vocabulary into an auditable M1 gate. When a migration is complete, production Rust should not keep compatibility modules, shim traits, legacy aliases, or migration-only bridge layers just to preserve old behavior.

The gate scans production Rust files under the root `zircon_*` packages and nested `zircon_plugins/**/src` packages. It intentionally treats `bridge` differently from `legacy`, `compat`, and `shim`: `bridge` is a valid business term in UI pointer routing, navigation off-mesh links, native/live-host surfaces, and resource fixtures. It becomes migration debt only when it appears with migration context such as `legacy`, `compat`, `shim`, `deprecated`, `compatibility`, or `forwarding`.

## Current Gate Output

The structural audit now reports `hard_cutover_migration_smells.hard_cutover_gate_status`. Current status is:

`migration-debt-present`

Current evidence:

- `source_file_count = 5901`
- `legacy_reference_count = 213`
- `compat_reference_count = 0`
- `shim_reference_count = 0`
- `bridge_reference_count = 300`
- `allowed_business_bridge_reference_count = 300`
- `migration_bridge_smell_count = 0`
- `smell_decision_count = 213`
- `smell_decision_group_count = 7`
- `classification_count = 7`
- `hard_cutover_migration_debt_count = 7`
- `unclassified_location_count = 0`
- `unclassified_locations = []`

Current classification:

- `legacy-runtime-ui-input-debt = 63`
- `legacy-hybrid-gi-render-debt = 56`
- `legacy-runtime-graphics-debt = 31`
- `legacy-hub-message-archived-text-debt = 58`
- `legacy-texture-importer-dds-debt = 1`
- `legacy-net-hyper-client-api-debt = 1`
- `legacy-editor-ui-fixture-debt = 3`

The classification means every current hard-cutover smell has an explicit owner group. It does not mean the runtime is converged.

## M1 Decision Rules

`compat` and `shim` are hard blockers in production Rust. If either appears, the owning migration slice must remove the compatibility layer or rename the code to its real current responsibility before accepting the boundary.

`legacy` is migration debt unless it is deliberately converted to an explicit versioned schema, archived diagnostic format, or test fixture policy. The word should not remain as a generic variable, helper, route, schema, or behavior label after the owner slice completes.

`bridge` is allowed when it names a real business owner: UI surface bridge, retained-host pointer bridge, native timeline bridge, navigation off-mesh bridge, or resource alias fixture. It is not allowed as a migration-only forwarding layer.

Any future `unclassified-hard-cutover-smell` is a review blocker. Classify it with an owner reason or remove the migration wording before accepting the boundary.

## Owner Decisions

`legacy-runtime-ui-input-debt` belongs to the runtime UI input dispatch owner. The current locations are split across the input dispatch, navigation, pointer, and pointer-reply files. The target language should describe current pointer routing, capture, navigation handoff, and component dispatch rather than preserving a `legacy` route or reply variable.

`legacy-hybrid-gi-render-debt` belongs to the hybrid GI render plugin owner. Current references describe old extract/trace schedules and scene-prepare filters; this must be resolved with the active render/plugin owner, not by adding a compatibility path.

`legacy-runtime-graphics-debt` belongs to the M6 graphics/RHI slice. It covers viewport packet wording, render feature fallback labels, runtime feature conversion, render graph execution records, and render-product test fixtures.

`legacy-hub-message-archived-text-debt` belongs to the Hub message archived-text compatibility owner. The target language should rename Hub message text constructors, enum variants, fixtures, and build-action helpers to explicit archived/raw text policy terminology during the Hub support slice instead of keeping generic legacy labels.

`legacy-texture-importer-dds-debt` belongs to the texture importer DDS container owner. Runtime asset importer source-template suffix guards have been renamed to current `.zui` policy language; the remaining asset-adjacent debt is plugin-owned DDS container wording that should become explicit DDS caps policy or be deleted.

`legacy-net-hyper-client-api-debt` belongs to the Net plugin HTTP backend dependency owner. The current hit is the third-party `hyper_util::client::legacy::Client` API path in the HTTP backend; wrap or rename that dependency edge as an explicit HTTP backend policy when the Net plugin backend is next touched.

`legacy-editor-ui-fixture-debt` belongs to editor retained-host fixture and projection owners. These are stale labels and view IDs, not runtime compatibility contracts.

## Resolved Owner Cuts

`legacy-runtime-interface-diagnostics-debt` was cleared on 2026-06-04. `UiPipelineStage` now names the stored-report policy directly with `ARCHIVED_DIAGNOSTIC_FORMAT_VERSION`, `ARCHIVED_DIAGNOSTIC_STAGES`, `is_runtime_schedule_stage()`, and `is_archived_diagnostic_stage()`. The archived names remain deserializable for stored diagnostics, but they are excluded from `UiPipelineStage::ORDER` and must not drive current runtime/editor scheduling.

`legacy-runtime-ui-layout-debt` was cleared on 2026-06-04. `WrapBox` now describes Flow slots as the current runtime contract for order, padding, and alignment only; Taffy-native wrap still ignores Flow slot `linear_sizing` instead of treating it as flex growth.

`legacy-runtime-ui-template-debt` was cleared from production Rust on 2026-06-05 through three direct sub-cuts. The runtime UI asset schema private conversion module moved from `legacy_template.rs` to `source_template_fixture.rs`, `UiAssetSchemaMigrator` now exposes `migrate_source_template_fixture_*` entry points, and flat/source-template helper variables describe the current migration responsibility directly. The public `UiAssetMigrationReport` labels remain unchanged for now because they are cross-crate report DTOs and need a separate interface-versioning cut if renamed.

The editor UI asset/session source-schema branch was hard-renamed from `UiAssetSourceSchema::Legacy` to `UiAssetSourceSchema::LayoutDocument`, and revalidate, canonical serialization, and undo replay now branch on the current layout document source schema rather than an old-version label. The editor host-template runtime cache assertions now call the non-v2 path the tree-template compile/document cache, so v2 bypass checks no longer preserve old recursive-cache wording.

The editor view-projection rejection path was narrowed on 2026-06-05. `ViewTemplateProjectionError::LegacyAssetPath` became `ViewTemplateProjectionError::NonV2AssetPath`, the corresponding source guards now look for the current non-v2 path contract, and the old `view.legacy.project_overview` fixture id was renamed to `view.archived.project_overview`. The remaining editor fixture debt is confined to retained-host/workbench fixture labels owned by active editor UI sessions.

The animation asset binary migration path was narrowed on 2026-06-05 and then split on 2026-06-14. `zircon_runtime/src/asset/assets/animation/binary.rs` now names the older clip, sequence, and graph payload conversion as `decode_binary_asset_with_v1_payload_fallback(...)`, while `clip.rs`, `sequence.rs`, and `graph.rs` own the `V1` payload DTOs and `v1 animation asset decode failed` diagnostics. The production animation asset module no longer uses generic `legacy` wording for this stored-payload migration; the remaining asset-adjacent hard-cutover debt is confined to plugin-owned DDS container parsing.

Runtime asset importer source-template suffix guards were narrowed on 2026-06-05. `AssetImporterRegistryError::{UiTomlSourceImporter,V2UiTomlSourceImporter}` now reject `.ui.toml` and `.v2.ui.toml` production registration with current source-template policy language, while exact test fixture allowances use `ui_toml_source_importer_allowed_for_tests(...)` and `v2_ui_toml_source_importer_allowed_for_tests(...)`. The production runtime asset importer files no longer use generic `legacy` wording; the remaining `legacy-texture-importer-dds-debt` count is confined to the plugin-owned DDS container path.

## Required Follow-Up

Before any hard-cutover migration slice, run the structural audit and inspect:

- `hard_cutover_migration_smells.source_file_count`
- `hard_cutover_migration_smells.smell_decision_count`
- `hard_cutover_migration_smells.smell_decision_group_count`
- `hard_cutover_migration_smells.classification_counts`
- `hard_cutover_migration_smells.classification_count`
- `hard_cutover_migration_smells.hard_cutover_migration_debt_count`
- `hard_cutover_migration_smells.unclassified_locations`
- `hard_cutover_migration_smells.unclassified_location_count`
- `hard_cutover_migration_smells.compat_reference_count`
- `hard_cutover_migration_smells.shim_reference_count`
- `hard_cutover_migration_smells.migration_bridge_smell_count`
- `hard_cutover_migration_smells.allowed_business_bridge_reference_count`

Production cuts should be bounded by owner areas. Do not add compatibility aliases, shim modules, or forwarding bridge modules while removing these references. Update call sites and tests directly to the converged owner path.
