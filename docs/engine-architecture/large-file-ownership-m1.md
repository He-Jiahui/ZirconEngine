---
related_code:
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot.rs
  - zircon_runtime/src/ui/surface/input/dispatch.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface.rs
  - zircon_runtime/src/asset/assets/scene.rs
  - zircon_runtime/src/asset/assets/animation.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/workbench.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/mod.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection.rs
  - zircon_editor/src/ui/asset_editor/style/theme_authoring.rs
  - zircon_hub/src/app/runtime.rs
  - zircon_hub/src/app/view_model.rs
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
- `hotspot_count = 33`
- `classification_count = 5`
- `decision_group_count = 5`
- `large_file_migration_debt_count = 5`
- `unclassified_hotspot_count = 0`
- `unclassified_hotspots = []`

Each `large_file_ownership_gate.hotspots` entry now carries both the raw line count and the owner bucket through `lines`, `line_count`, and `owner_class`. The duplicate count field is intentional so downstream checks and ad-hoc PowerShell inspection do not need to remember the older `lines` key before choosing a split owner.

Current classification:

- `editor-retained-host = 11`
- `editor-ui = 8`
- `runtime-framework-render = 1`
- `runtime-other = 10`
- `support-hub = 3`

The classification means every current hotspot has an owner bucket. It does not mean the file is converged.

## M1 Decision Rules

Large files must be split by ownership, not by equal line ranges. A valid split moves behavior to named owner modules such as lifecycle, route policy, renderer extract, DTO conversion, diagnostics, manifest parsing, validation, or fixture support.

Root files may remain as structural facades. They should not keep behavior just to avoid updating call sites.

Do not preserve old compatibility wrappers while splitting. If a moved helper exposes a new owner, consumers should call the new owner path directly or the intended curated facade.

Any future `unclassified_hotspots` entry is a review blocker. Classify it with an owner class before accepting more behavior in that area.

## Owner Decisions

`runtime-framework-render` currently contains `zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot.rs`. This belongs to the M6 render/framework slice; split debug snapshot DTOs, projections, and diagnostic reports only after the active WGPU/RHI session settles.

`runtime-other` currently includes runtime UI input/surface, RHI/WGPU UI surface, UI catalog/accessibility extract, asset scene/animation, and graphics UI render hotspots. These should be split by runtime module owner before any M5 performance work claims improvements in allocation, clone behavior, or dispatch cost.

`editor-retained-host` currently includes painter workbench, host lifecycle, native pointer, pane conversion, apply presentation, asset editor host, window contract, and template node hotspots. These belong to M7 editor/UI and should wait for the active host-editor UI session to quiet down.

`editor-ui` currently includes workbench host window projection, asset editor theme/binding/session/preview, animation editor session, editor manager asset editor, and template showcase state. Split by authoring workflow and template-runtime owner.

`support-hub` currently includes Hub runtime, view-model, and project-workspace files. Coordinate with active Hub sessions before touching those files.

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
