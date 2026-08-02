---
related_code:
  - zircon_runtime/src/ui/surface/input/dispatch.rs
  - zircon_runtime/src/ui/surface/input/navigation.rs
  - zircon_runtime/src/ui/surface/input/pointer.rs
  - zircon_runtime/src/ui/surface/input/pointer_reply.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/gpu_resources/execute_prepare/execute/collect_inputs.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/gpu_resources/execute_prepare/probe_quantization.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/resolve_enabled_features.rs
  - zircon_runtime/src/graphics/scene/render_product_streamer_tests/material_runtime.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/runtime_features/runtime_features_from_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs
  - zircon_runtime/src/core/framework/animation/asset/mod.rs
  - zircon_runtime/src/core/framework/animation/asset/binary.rs
  - zircon_runtime/src/core/framework/animation/asset/clip.rs
  - zircon_runtime/src/core/framework/animation/asset/graph.rs
  - zircon_runtime/src/core/framework/animation/asset/sequence.rs
  - docs/zircon_runtime/core/framework/animation-assets.md
  - zircon_runtime/src/asset/importer/registry.rs
  - zircon_runtime/src/asset/importer/ingest/asset_importer.rs
  - zircon_plugins/texture_importer/runtime/src/container/dds.rs
  - zircon_editor/src/ui/asset_editor/session/command_entry.rs
  - zircon_editor/src/ui/asset_editor/session/lifecycle.rs
  - zircon_editor/src/ui/asset_editor/session/ui_asset_editor_session.rs
  - zircon_editor/src/ui/template_runtime/runtime/build_session.rs
  - zircon_runtime/src/ui/template/asset/schema/migrator.rs
  - zircon_runtime/src/ui/template/asset/schema/flat_nodes.rs
  - zircon_runtime/src/ui/template/asset/compiler/style_apply.rs
  - zircon_editor/src/ui/layouts/views/view_projection.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows.rs
  - zircon_runtime/src/ui/layout/pass/taffy_arrange.rs
  - zircon_runtime_interface/src/ui/pipeline/stage.rs
  - zircon_runtime_interface/src/tests/pipeline_contracts.rs
  - docs/zircon_runtime_interface/ui/pipeline.md
  - zircon_hub/src/state/hub_message/message.rs
  - zircon_hub/src/tauri_app/runtime_state/build_actions.rs
  - zircon_plugins/net/features/http/runtime/src/backend/client.rs
  - zircon_plugins/net/features/http/runtime/src/backend/http1_client_policy.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/net.rs
  - zircon_runtime/src/scene/dynamic_scene/document/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/document/read.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/hard_cutover_migration_smells.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/hard_cutover_migration_smells_markdown.py
implementation_files:
  - zircon_editor/src/ui/template_runtime/runtime/build_session.rs
  - zircon_editor/src/ui/asset_editor/session/ui_asset_editor_session.rs
  - zircon_editor/src/ui/asset_editor/session/lifecycle.rs
  - zircon_editor/src/ui/asset_editor/session/command_entry.rs
  - zircon_runtime/src/ui/template/asset/schema/mod.rs
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
  - docs/zircon_runtime/core/framework/animation-assets.md
  - docs/engine-architecture/runtime-architecture-review-m0.md
  - docs/engine-architecture/runtime-interface-convergence.md
  - docs/engine-architecture/index.md
  - docs/engine-architecture/runtime-reference-engine-evidence.md
  - zircon_plugins/net/features/http/runtime/src/backend/http1_client_policy.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/net.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/hard_cutover_migration_smells.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/hard_cutover_migration_smells_markdown.py
plan_sources:
  - user: 2026-06-04 optimize Zircon Engine runtime architecture with breaking changes allowed
  - .codex/plans/Zircon Runtime 架构渐进式 Review 与优化计划.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/全系统重构方案.md
  - docs/engine-architecture/runtime-reference-engine-evidence.md
tests:
  - python -m py_compile .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/hard_cutover_migration_smells.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/hard_cutover_migration_smells_markdown.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py --json
  - python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - hard_cutover_migration_smells gate status, explicit count fields, classification count, migration debt, allowed bridge count, and unclassified reference checks
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/net.rs::runtime_15_net_http_hyper_http1_client_policy_is_isolated
  - rustfmt --edition 2021 --check zircon_plugins/net/features/http/runtime/src/backend.rs zircon_plugins/net/features/http/runtime/src/backend/client.rs zircon_plugins/net/features/http/runtime/src/backend/http1_client_policy.rs zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/net.rs
  - rustfmt --edition 2021 --check zircon_runtime\src\ui\template\asset\schema\mod.rs zircon_runtime\src\ui\template\asset\schema\migrator.rs zircon_runtime\src\ui\template\asset\schema\flat_nodes.rs zircon_runtime\src\ui\template\asset\compiler\style_apply.rs zircon_runtime\src\ui\tests\asset_schema_migration.rs zircon_runtime\src\ui\tests\asset.rs zircon_runtime\src\ui\tests\asset_contract_spine.rs
  - rustfmt --edition 2021 --check zircon_runtime\src\ui\layout\pass\taffy_arrange.rs
  - rustfmt --edition 2021 --check zircon_runtime_interface\src\ui\pipeline\stage.rs zircon_runtime_interface\src\tests\pipeline_contracts.rs
  - rustfmt --edition 2021 --check zircon_plugins\texture_importer\runtime\src\container\dds.rs
  - cargo test -p zircon_runtime_interface pipeline_contracts --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-interface-stage-policy-0604 --message-format short --color never -- --test-threads=1 --nocapture
doc_type: milestone-detail
---

# Hard-Cutover Migration Smells M1 Gate

> 规范权威：跨域通用规则已统一收敛至 [Zircon 开发规范总纲](../plans/zircon_runtime/frameworks/development-conventions.md)；本文保留硬切迁移异味 gate 的细节论证与执行上下文，不再作为并列规则源。

## Purpose

This document turns hard-cutover migration vocabulary into an auditable M1 gate. When a migration is complete, production Rust should not keep compatibility modules, shim traits, legacy aliases, or migration-only bridge layers just to preserve old behavior.

The gate scans production Rust files under the root `zircon_*` packages and nested `zircon_plugins/**/src` packages. It intentionally treats `bridge` differently from `legacy`, `compat`, and `shim`: `bridge` is a valid business term in UI pointer routing, navigation off-mesh links, native/live-host surfaces, and resource fixtures. It becomes migration debt only when it appears with migration context such as `legacy`, `compat`, `shim`, `deprecated`, `compatibility`, or `forwarding`.

## Current Gate Output

The structural audit now reports `hard_cutover_migration_smells.hard_cutover_gate_status`. Current status is:

`migration-debt-present`

Current evidence:

- `source_file_count = 9593`
- `legacy_reference_count = 85`
- `compat_reference_count = 0`
- `shim_reference_count = 0`
- `bridge_reference_count = 326`
- `allowed_business_bridge_reference_count = 326`
- `migration_bridge_smell_count = 0`
- `smell_decision_count = 85`
- `smell_decision_group_count = 3`
- `classification_count = 3`
- `hard_cutover_migration_debt_count = 2`
- `unclassified_location_count = 0`
- `unclassified_locations = []`

Current classification:

- `legacy-hybrid-gi-render-debt = 56`
- `legacy-runtime-graphics-debt = 28`
- `external-hyper-http1-client-policy = 1`

The classification means every current hard-cutover reference has an explicit owner group. `external-hyper-http1-client-policy` is an allowed third-party API policy owner, not migration debt. The remaining legacy classifications still mean the runtime is not converged.

## M1 Decision Rules

`compat` and `shim` are hard blockers in production Rust. If either appears, the owning migration slice must remove the compatibility layer or rename the code to its real current responsibility before accepting the boundary.

`legacy` is migration debt unless it is deliberately converted to an explicit versioned schema, archived diagnostic format, or test fixture policy. The word should not remain as a generic variable, helper, route, schema, or behavior label after the owner slice completes.

`bridge` is allowed when it names a real business owner: UI surface bridge, retained-host pointer bridge, native timeline bridge, navigation off-mesh bridge, or resource alias fixture. It is not allowed as a migration-only forwarding layer.

Any future `unclassified-hard-cutover-smell` is a review blocker. Classify it with an owner reason or remove the migration wording before accepting the boundary.

## Owner Decisions

`legacy-hybrid-gi-render-debt` belongs to the hybrid GI render plugin owner. Current references describe old extract/trace schedules and scene-prepare filters; this must be resolved with the active render/plugin owner, not by adding a compatibility path.

`legacy-runtime-graphics-debt` belongs to the M6 graphics/RHI slice. It covers viewport packet wording, render feature fallback labels, runtime feature conversion, render graph execution records, and render-product test fixtures.

`external-hyper-http1-client-policy` belongs to the Net plugin HTTP backend dependency owner and is allowed. `zircon_plugins/net/features/http/runtime/src/backend/http1_client_policy.rs` is the only production owner allowed to spell the third-party `hyper_util::client::legacy` API path; `zircon_plugins/net/features/http/runtime/src/backend/client.rs` must consume it through `http1_client_policy::plain_http_client()`.

## Resolved Owner Cuts

`legacy-editor-ui-fixture-debt` was cleared on 2026-06-27 by Runtime 15 M2 editor Workbench archived fixture naming hard cutover with status `runtime_15_editor_workbench_archived_fixture_naming_hard_cutover_static_passed_cargo_deferred`. The old `zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/legacy.rs` private renderer module became `zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/host_window.rs`, the entry points became `draw_host_workbench_window(...)` and `draw_host_workbench_window_profiled(...)`, table row text fallback became `split_archived_table_text(...)`, and Workbench extension fixture rows now use `ArchivedRow` / `archived_table_row` / `icon-archive` wording across ZUI assets, builtin bindings, navigation specs, preview action lists, and feedback dispatch. `runtime_15_editor_workbench_archived_fixtures_use_current_names` guards these retired fixture names from returning.

`legacy-hub-message-archived-text-debt` was cleared on 2026-06-27 by Runtime 15 M2 Hub message raw text policy hard cutover with status `runtime_15_hub_message_raw_text_policy_hard_cutover_static_passed_cargo_deferred`. `zircon_hub/src/state/hub_message/message.rs` now names the unstructured message branch as `RawText(String)`, exposes `HubMessage::raw_text(...)`, and names old string wire payload deserialization as `ArchivedRawText(String)`. `zircon_hub/src/tauri_app/runtime_state/build_actions.rs` and other Hub callers consume the raw-text policy through `HubMessage::raw_text`, and `runtime_15_hub_message_raw_text_policy_uses_current_names` guards that `zircon_hub/src` does not reintroduce generic legacy wording.

`legacy-net-hyper-client-api-debt` was cleared on 2026-06-27 by Runtime 15 M2 Net HTTP backend Hyper HTTP/1 client policy hard cutover with status `runtime_15_net_http_hyper_http1_client_policy_hard_cutover_static_passed_cargo_deferred`. `zircon_plugins/net/features/http/runtime/src/backend/client.rs` no longer imports `hyper_util::client::legacy::Client`; the third-party Hyper HTTP/1 policy is isolated in `zircon_plugins/net/features/http/runtime/src/backend/http1_client_policy.rs`, classified as allowed `external-hyper-http1-client-policy`, and guarded by `runtime_15_net_http_hyper_http1_client_policy_is_isolated`.

`legacy-runtime-scene-document-debt` was cleared on 2026-06-27 by Runtime 15 M2 scene dynamic document v1 owner naming hard cutover. The old `scene/dynamic_scene/document/legacy.rs` owner became `scene/dynamic_scene/document/v1_project_document.rs`, `LegacyProjectDocument` became `V1ProjectDocument`, and the audit now reads the explicit v1 project document policy owner.

`legacy-runtime-interface-diagnostics-debt` was cleared on 2026-06-04. `UiPipelineStage` now names the stored-report policy directly with `ARCHIVED_DIAGNOSTIC_FORMAT_VERSION`, `ARCHIVED_DIAGNOSTIC_STAGES`, `is_runtime_schedule_stage()`, and `is_archived_diagnostic_stage()`. The archived names remain deserializable for stored diagnostics, but they are excluded from `UiPipelineStage::ORDER` and must not drive current runtime/editor scheduling.

`legacy-runtime-ui-layout-debt` was cleared on 2026-06-04. `WrapBox` now describes Flow slots as the current runtime contract for order, padding, and alignment only; Taffy-native wrap still ignores Flow slot `linear_sizing` instead of treating it as flex growth.

`legacy-runtime-ui-template-debt` was cleared from production Rust on 2026-06-05 through three direct sub-cuts. At that milestone the runtime UI asset schema private conversion module moved from `legacy_template.rs` to `source_template_fixture.rs`, `UiAssetSchemaMigrator` exposed `migrate_source_template_fixture_*` entry points, and flat/source-template helper variables described that intermediate migration responsibility. This evidence is historical: the 2026-08-01 Runtime09 cut removed the source-template converter, helpers and report variants entirely; the live loader now requires `[asset]` and keeps only tree/flat migration authority.

The editor UI asset/session source-schema branch was hard-renamed from `UiAssetSourceSchema::Legacy` to `UiAssetSourceSchema::LayoutDocument`, and revalidate, canonical serialization, and undo replay now branch on the current layout document source schema rather than an old-version label. The editor host-template runtime cache assertions now call the non-v2 path the tree-template compile/document cache, so v2 bypass checks no longer preserve old recursive-cache wording.

The editor view-projection rejection path was narrowed on 2026-06-05. `ViewTemplateProjectionError::LegacyAssetPath` became `ViewTemplateProjectionError::NonV2AssetPath`, the corresponding source guards now look for the current non-v2 path contract, and the old `view.legacy.project_overview` fixture id was renamed to `view.archived.project_overview`. The remaining editor fixture debt is confined to retained-host/workbench fixture labels owned by active editor UI sessions.

The animation asset binary migration path was narrowed on 2026-06-05 and then split on 2026-06-14. `zircon_runtime/src/core/framework/animation/asset/binary.rs` now names the older clip, sequence, and graph payload conversion as `decode_binary_asset_with_v1_payload_fallback(...)`, while `clip.rs`, `sequence.rs`, and `graph.rs` own the `V1` payload DTOs and `v1 animation asset decode failed` diagnostics. The production animation asset module no longer uses generic `legacy` wording for this stored-payload migration; the remaining asset-adjacent hard-cutover debt is confined to plugin-owned DDS container parsing.

Runtime asset importer source-template suffix guards were cut over again on 2026-06-28. `AssetImporterRegistryError::DeprecatedUiDocumentSuffixImporter` now rejects both `.ui.toml` and `.v2.ui.toml` registration with a single `.zui`-only policy, and the old test fixture allowance helpers were removed with `import_ui_asset.rs` / `import_ui_v2_asset.rs`. The production runtime asset importer files no longer keep legacy UI document suffix importers alive.

The runtime UI input dispatch debt was cleared by the Runtime 09 input owner rename slices. Pointer routing, navigation replies, capture fallback, table-row label fallback, template component-name fallback, property visibility, responsive visibility, accessibility open state, layout backend, and default interaction fallback wording now use current-route/current-fallback policy names instead of the old migration label, so `legacy-runtime-ui-input-debt` is absent from the current hard-cutover gate output.

The texture importer DDS container debt was cleared on 2026-06-17. The DX10 dual-cubemap diagnostic in `zircon_plugins/texture_importer/runtime/src/container/dds.rs` now names `DDSCAPS2_CUBEMAP caps2 policy` and the DX10 texturecube flag directly, so `legacy-texture-importer-dds-debt` is absent from the current hard-cutover gate output.

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
