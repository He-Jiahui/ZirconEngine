---
related_code:
  - zircon_editor/src/ui/host/resource_access.rs
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/assets.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback/gameplay_state.rs
  - zircon_editor/src/ui/workbench/state/editor_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/capture_frame/capture_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/create_viewport/create.rs
  - zircon_runtime/src/graphics/runtime/render_framework/destroy_viewport/destroy_viewport.rs
  - zircon_runtime/src/graphics/runtime/render_framework/query_stats/query_stats.rs
  - zircon_runtime/src/graphics/runtime/render_framework/query_virtual_geometry_debug_snapshot/query_virtual_geometry_debug_snapshot.rs
  - zircon_runtime/src/graphics/runtime/render_framework/register_pipeline_asset/register_pipeline_asset.rs
  - zircon_runtime/src/graphics/runtime/render_framework/reload_pipeline/reload_pipeline.rs
  - zircon_runtime/src/graphics/runtime/render_framework/set_pipeline_asset/set_pipeline_asset.rs
  - zircon_runtime/src/graphics/runtime/render_framework/set_quality_profile/set_quality_profile.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/resolve_viewport_record_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs
  - docs/engine-architecture/runtime-reference-engine-evidence.md
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/non_network_server_naming.py
implementation_files:
  - zircon_editor/src/ui/host/resource_access.rs
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/assets.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle.rs
  - zircon_editor/src/tests/host/resource_access/mod.rs
  - zircon_editor/src/ui/workbench/state/editor_state.rs
  - docs/engine-architecture/non-network-server-naming-m1.md
  - docs/engine-architecture/runtime-architecture-review-m0.md
  - docs/engine-architecture/runtime-interface-convergence.md
  - docs/editor-and-tooling/crate-boundary-audit-round-2.md
  - docs/engine-architecture/index.md
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/non_network_server_naming.py
  - .codex/sessions/20260604-1232-runtime-architecture-review.md
plan_sources:
  - user: 2026-06-04 optimize Zircon Engine runtime architecture with breaking changes allowed
  - .codex/plans/Zircon Runtime 架构渐进式 Review 与优化计划.md
  - docs/engine-architecture/runtime-reference-engine-evidence.md
tests:
  - python -m py_compile .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/non_network_server_naming.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py --json
  - python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - non_network_server_references M1 gate status, explicit count fields, classification count, migration debt, and unclassified reference checks
doc_type: milestone-detail
---

# Non-Network Server Naming M1 Gate

## Purpose

This document turns the non-network `server` naming rule into an auditable M1 gate. `server` stays valid for real network, target-runtime, service-host, dev-server, and external API contexts. It is not valid as a generic name for render-framework receivers or editor/workbench authority labels.

The reference-engine evidence is in `docs/engine-architecture/runtime-reference-engine-evidence.md`:

- Bevy uses app, schedule, world, system, query, and resource terms for non-network runtime coordination.
- Fyrox uses engine, plugin, scene, resource, editor, and viewer terms for non-network ownership.
- Unreal uses Runtime, Editor, Programs, modules, plugins, and build targets rather than arbitrary non-network server owners.

## Current Gate Output

The structural audit now reports `non_network_server_references.m1_gate_status`. Current status is:

`migration-debt-present`

Current evidence:

- `count = 58` suspect non-network references;
- `sample_location_count = 20`;
- `reference_decision_count = 58`;
- `reference_decision_group_count = 2`;
- `classification_count = 2`;
- `observer_false_positive_count = 72`, because `observer` contains the letters `server` but is not server vocabulary;
- `allowed_context_count = 93` for real network, target-runtime, dev-server, UNC fixture, and external UI API contexts;
- `non_network_server_migration_debt_count = 2`;
- `unclassified_location_count = 0`;
- `unclassified_locations = []`.

Current classification:

- `graphics-render-framework-debt = 57`
- `editor-workbench-authority-label-debt = 1`

The classification means every current suspect reference has an explicit migration owner. It does not mean the naming is converged.

## M1 Decision Rules

`observer` and `Observer*` are not server naming. The audit ignores them so ECS observer work is not blocked by a substring false positive.

Real target or network vocabulary remains allowed: `ServerRuntime`, `RuntimeProfileId::Server`, dedicated/listen server modes, network feature target rows, export target profiles, browser dev-server config, UNC path fixtures, and third-party UI API values that literally use `"server"`.

`graphics-render-framework-debt` belongs to the M6 graphics/RHI public-surface slice. The target names should describe the actual owner, for example render framework, render context, state owner, or submit context.

`editor-workbench-authority-label-debt` belongs to the editor workbench extension owner. The current hit is fixture/output text that says `server authority`; the target wording should describe editor/runtime authority without using non-network server terminology.

The previous stale editor scene comment debt is resolved. `EditorState` now describes the runtime scene inspection boundary directly.

The editor asset/resource owner debt is resolved. `resolve_ready_handle`, retained-host app fields, retained-host startup locals, and the focused resource-access test fixture now use asset/editor-asset/resource manager terminology instead of `asset_server`, `editor_asset_server`, `resource_server`, or `FakeResourceServer`.

Any future `unclassified-non-network-server` location is a review blocker. Classify it with an owner reason or remove the non-network `server` naming before accepting the boundary.

## Required Follow-Up

Before renaming production symbols, run the structural audit and inspect:

- `non_network_server_references.count`
- `non_network_server_references.sample_location_count`
- `non_network_server_references.reference_decision_count`
- `non_network_server_references.reference_decision_group_count`
- `non_network_server_references.classification_counts`
- `non_network_server_references.classification_count`
- `non_network_server_references.non_network_server_migration_debt_count`
- `non_network_server_references.unclassified_locations`
- `non_network_server_references.unclassified_location_count`
- `non_network_server_references.allowed_context_count`
- `non_network_server_references.observer_false_positive_count`

The first production cuts should be bounded by active owner areas:

- M6 graphics/RHI: rename render-framework `server` parameters while keeping WGPU/RHI behavior stable.
- Editor workbench extensions: rename authority fixture/output labels when the extension-module workbench owner is next touched.

Do not create compatibility aliases for old `*_server` names. Each rename should be a hard cut inside its owner slice, with call sites updated directly.
