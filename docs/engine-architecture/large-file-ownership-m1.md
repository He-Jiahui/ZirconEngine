---
related_code:
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot.rs
  - zircon_runtime/src/core/framework/render/post_process/stack.rs
  - zircon_runtime/src/core/framework/render/post_process/volume_component.rs
  - zircon_runtime/src/asset/assets/scene/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs
  - zircon_runtime/src/script/vm/gameplay_host.rs
  - zircon_runtime/src/script/vm/gameplay_host/combat.rs
  - zircon_runtime/src/script/vm/gameplay_host/components.rs
  - zircon_runtime/src/script/vm/gameplay_host/input.rs
  - zircon_runtime/src/script/vm/gameplay_host/lifecycle.rs
  - zircon_runtime/src/script/vm/gameplay_host/navigation.rs
  - zircon_runtime/src/script/vm/gameplay_host/script_bindings.rs
  - zircon_runtime/src/script/vm/gameplay_host/transform.rs
  - zircon_runtime/src/script/vm/gameplay_host/values.rs
  - zircon_runtime/src/scene/world/project_io.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/workbench.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/mod.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection.rs
  - zircon_editor/src/ui/asset_editor/style/theme_authoring.rs
  - zircon_hub/src/tauri_app/runtime_state/project_actions.rs
  - zircon_hub/src/tauri_app/view_model.rs
  - zircon_hub/src/tauri_app/runtime_state.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/large_file_ownership.py
implementation_files:
  - docs/engine-architecture/large-file-ownership-m1.md
  - docs/engine-architecture/runtime-architecture-review-m0.md
  - docs/engine-architecture/runtime-interface-convergence.md
  - docs/engine-architecture/index.md
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/large_file_ownership.py
  - .codex/sessions/20260604-1232-runtime-architecture-review.md
plan_sources:
  - user: 2026-06-04 optimize Zircon Engine runtime architecture with breaking changes allowed
  - .codex/plans/Zircon Runtime 架构渐进式 Review 与优化计划.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/全系统重构方案.md
tests:
  - python -m py_compile .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/large_file_ownership.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py --json
  - python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - large_file_ownership_gate M1 gate status, explicit count fields, classification count, migration debt, and unclassified hotspot checks
doc_type: milestone-detail
---

# Large File Ownership M1 Gate

## Purpose

This document turns large production files into an auditable architecture gate. A large file is not automatically wrong, but in this runtime review it is evidence that behavior families, DTO conversions, validation paths, diagnostics, or lifecycle steps may be sharing one owner.

The gate is deliberately ownership-first. It does not prescribe arbitrary line-count chopping. Each split must move coherent behavior into folder-backed owner modules and keep the public facade structural.

## Current Gate Output

The structural audit now reports `large_file_ownership_gate.m1_gate_status`. Current status is:

`migration-debt-present`

Current evidence:

- `threshold = 1000`
- `hotspot_count = 39`
- `classification_count = 5`
- `decision_group_count = 5`
- `large_file_migration_debt_count = 5`
- `unclassified_hotspot_count = 0`
- `unclassified_hotspots = []`

Each `large_file_ownership_gate.hotspots` entry now carries both the raw line count and the owner bucket through `lines`, `line_count`, and `owner_class`. The duplicate count field is intentional so downstream checks and ad-hoc PowerShell inspection do not need to remember the older `lines` key before choosing a split owner.

Current classification:

- `editor-retained-host = 12`
- `editor-ui = 8`
- `runtime-framework-render = 4`
- `runtime-other = 12`
- `support-hub = 3`

The classification means every current hotspot has an owner bucket. It does not mean the file is converged.

The 2026-06-15 Runtime 07 split sequence now records 39 hotspots while active render, UI, plugin, and Hub work continues. The former single-file animation asset module was cut into the folder-backed `zircon_runtime/src/asset/assets/animation/{mod,binary,channel,clip,graph,reference,sequence,skeleton,state_machine}.rs` owner set, the former single-file scene asset module was cut into `zircon_runtime/src/asset/assets/scene/{mod,animation,asset,camera,defaults,entity,extensions,lighting,management,mesh,physics,post_process,transform}.rs`, scene project I/O was split into `zircon_runtime/src/scene/world/project_io/{camera,physics,post_process,references,script,transform}.rs`, dynamic-session event routing was split into `zircon_runtime/src/dynamic_api/session/events.rs`, and the script gameplay host was split into `zircon_runtime/src/script/vm/gameplay_host/{combat,components,input,lifecycle,navigation,script_bindings,transform,values}.rs` with the registration owner at 371 lines. Those payload, conversion, event-routing, and gameplay-host surfaces no longer contribute large-file hotspots. The current total drift comes from active render/post-process owner work now placing `zircon_runtime/src/core/framework/render/post_process/stack.rs` and `zircon_runtime/src/core/framework/render/post_process/volume_component.rs` above the threshold, which moves `runtime-framework-render` to 4 without changing the owner-budget gate shape. Current runtime hotspots still include `zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product.rs` and `zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs`, so the important gate fact is that `unclassified_hotspot_count = 0` and all current hotspots still resolve to an owner bucket before Runtime 07 M2 optimization work can use them.

## M1 Decision Rules

Large files must be split by ownership, not by equal line ranges. A valid split moves behavior to named owner modules such as lifecycle, route policy, renderer extract, DTO conversion, diagnostics, manifest parsing, validation, or fixture support.

Root files may remain as structural facades. They should not keep behavior just to avoid updating call sites.

Do not preserve old compatibility wrappers while splitting. If a moved helper exposes a new owner, consumers should call the new owner path directly or the intended curated facade.

Any future `unclassified_hotspots` entry is a review blocker. Classify it with an owner class before accepting more behavior in that area.

## Owner Decisions

`runtime-framework-render` currently contains `zircon_runtime/src/core/framework/render/backend_types.rs`, `zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot.rs`, `zircon_runtime/src/core/framework/render/post_process/stack.rs`, and `zircon_runtime/src/core/framework/render/post_process/volume_component.rs`. This belongs to the M6 render/framework slice; split backend DTOs, debug snapshot DTOs, post-process stack/volume owners, projections, and diagnostic reports only after the active WGPU/RHI session settles.

`runtime-other` currently includes render stats product diagnostics, render graph execution record/resources, render pipeline asset compile, runtime UI surface/style/catalog/accessibility extract, RHI/WGPU UI surface, graphics UI render, and frame-extract virtual-geometry snapshot/update-stats helpers. The animation asset, scene asset, scene project I/O conversion, dynamic-session event-routing, and script gameplay-host surfaces have already been split below the hotspot threshold; remaining runtime-other hotspots should be split by runtime module owner before any M5/M7 performance work claims improvements in allocation, clone behavior, or dispatch cost.

`editor-retained-host` currently includes painter workbench, host lifecycle, native pointer, pane conversion, apply presentation, asset editor host, window contract, template node/status-control, profiling artifact, and retained-host helper hotspots. These belong to M7 editor/UI and should wait for the active host-editor UI session to quiet down.

`editor-ui` currently includes workbench host window projection, asset editor theme/binding/session/preview, animation editor session, editor manager asset editor, and template showcase state. Split by authoring workflow and template-runtime owner.

`support-hub` currently includes Hub `tauri_app` runtime-state project actions, view-model, and runtime-state root files. Coordinate with active Hub sessions before touching those files.

## Required Follow-Up

Before splitting any large file, run the structural audit and inspect:

- `large_file_ownership_gate.hotspot_count`
- `large_file_ownership_gate.decision_groups`
- `large_file_ownership_gate.decision_group_count`
- `large_file_ownership_gate.classification_counts`
- `large_file_ownership_gate.classification_count`
- `large_file_ownership_gate.large_file_migration_debt_count`
- `large_file_ownership_gate.unclassified_hotspots`
- `large_file_ownership_gate.unclassified_hotspot_count`
- `large_file_ownership_gate.m1_gate_status`

The next safe production split should be chosen only after checking active session notes. If the owner area is active, add or refine audit evidence instead of editing that production file.
