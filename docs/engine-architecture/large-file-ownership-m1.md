---
related_code:
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/camera.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/visibility.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/hzb.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/light_grid.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/effect_stack.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/material.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/light.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/mesh_queue.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/gpu_scene.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/sprite.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/ui.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/tests.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/tests/camera_targets.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/tests/visibility_hzb_light.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/tests/mesh_gpu_scene.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot/bvh_visualization.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot/cpu_reference.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot/cull_input.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot/execution.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot/node_and_cluster_cull.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot/snapshot.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot/sources.rs
  - zircon_runtime/src/core/framework/render/post_process/stack.rs
  - zircon_runtime/src/core/framework/render/post_process/volume_component.rs
  - zircon_runtime/src/asset/assets/scene/mod.rs
  - zircon_runtime/src/asset/artifact/cache_payload.rs
  - zircon_runtime/src/asset/artifact/cache_payload/json_value.rs
  - zircon_runtime/src/asset/artifact/cache_payload/mesh.rs
  - zircon_runtime/src/asset/artifact/cache_payload/toml_value.rs
  - zircon_runtime/src/navigation/runtime.rs
  - zircon_runtime/src/navigation/runtime/baked_mesh.rs
  - zircon_runtime/src/navigation/runtime/world_scan.rs
  - zircon_runtime/src/navigation/runtime/avoidance.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/graph.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
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
  - zircon_editor/src/ui/retained_host/host_contract/template_component_family/workbench.rs
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
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/large_file_ownership_markdown.py
implementation_files:
  - docs/engine-architecture/large-file-ownership-m1.md
  - docs/engine-architecture/runtime-architecture-review-m0.md
  - docs/engine-architecture/runtime-interface-convergence.md
  - docs/engine-architecture/index.md
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/large_file_ownership.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/large_file_ownership_markdown.py
plan_sources:
  - user: 2026-06-04 optimize Zircon Engine runtime architecture with breaking changes allowed
  - .codex/plans/Zircon Runtime 架构渐进式 Review 与优化计划.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/全系统重构方案.md
tests:
  - python -m py_compile .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/large_file_ownership.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/large_file_ownership_markdown.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py --json
  - python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - large_file_ownership_gate M1 gate status, explicit count fields, classification count, migration debt, and unclassified hotspot checks
doc_type: milestone-detail
---

# Large File Ownership M1 Gate

> 规范权威：跨域通用规则已统一收敛至 [Zircon 开发规范总纲](../plans/zircon_runtime/frameworks/development-conventions.md)；本文保留大文件 owner gate 的细节论证与执行上下文，不再作为并列规则源。

## Purpose

This document turns large production files into an auditable architecture gate. A large file is not automatically wrong, but in this runtime review it is evidence that behavior families, DTO conversions, validation paths, diagnostics, or lifecycle steps may be sharing one owner.

The gate is deliberately ownership-first. It does not prescribe arbitrary line-count chopping. Each split must move coherent behavior into folder-backed owner modules and keep the public facade structural.

## Current Gate Output

The structural audit now reports `large_file_ownership_gate.m1_gate_status`. Current status is:

`classified-and-clear`

Current evidence:

- `threshold = 1000`
- `hotspot_count = 0`
- `classification_count = 0`
- `decision_group_count = 0`
- `large_file_migration_debt_count = 0`
- `unclassified_hotspot_count = 0`
- `unclassified_hotspots = []`

Each `large_file_ownership_gate.hotspots` entry now carries both the raw line count and the owner bucket through `lines`, `line_count`, and `owner_class`. The duplicate count field is intentional so downstream checks and ad-hoc PowerShell inspection do not need to remember the older `lines` key before choosing a split owner.

Current classification:

- no current owner-class buckets above the large-file threshold

The classification means the current production Rust workspace has no files at or above the 1000-line large-file threshold. It does not close the historical owner-budget work; it only records the current audit value that Runtime 07 must mirror before optimization work can use the gate.

The 2026-06-21 renderer split keeps the ownership gate as data first: `large_file_ownership.py` owns owner classification, hotspot summary, migration-debt, and risk data at 223 lines, while `large_file_ownership_markdown.py` owns hotspot, ownership-class, and ownership-gate Markdown rendering at 73 lines. The current direct probe reports no production Rust files at or above the 1000-line threshold and therefore a `classified-and-clear` gate.

The 2026-07-01 owner-budget sync records 0 current hotspots while preserving the historical migration trail below. The former single-file animation asset module was cut into the folder-backed `zircon_runtime/src/core/framework/animation/asset/{mod,binary,channel,clip,graph,reference,sequence,skeleton,state_machine}.rs` owner set, the former single-file scene asset module was cut into `zircon_runtime/src/asset/assets/scene/{mod,animation,asset,camera,defaults,entity,extensions,lighting,management,mesh,physics,post_process,transform}.rs`, scene project I/O was split into `zircon_runtime/src/scene/world/project_io/{camera,physics,post_process,references,script,transform}.rs`, dynamic-session event routing was split into `zircon_runtime/src/dynamic_api/session/events.rs`, the script gameplay host was split into `zircon_runtime/src/script/vm/gameplay_host/{combat,components,input,lifecycle,navigation,script_bindings,transform,values}.rs`, artifact cache payload JSON/Mesh/TOML wire owners were split into `zircon_runtime/src/asset/artifact/cache_payload/{json_value,mesh,toml_value}.rs`, render product diagnostics were split into `zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/{camera,visibility,hzb,light_grid,effect_stack,material,light,mesh_queue,gpu_scene,sprite,ui}.rs`, virtual geometry debug snapshot DTOs were split into `zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot/{bvh_visualization,cpu_reference,cull_input,execution,node_and_cluster_cull,snapshot,sources}.rs`, and the navigation fallback runtime was split into `zircon_runtime/src/navigation/runtime/{baked_mesh,world_scan,avoidance,state,math,tests}.rs`. The current gate fact is that `hotspot_count = 0`, `large_file_migration_debt_count = 0`, `classification_count = 0`, and `unclassified_hotspot_count = 0`; Runtime 07 M2 optimization work still needs its separate extract, ECS query, profiling, and FPS evidence before promotion.

The 2026-06-24 Runtime 15 M4 render-stats product diagnostics test owner split keeps the production recorder owner set above intact and moves product diagnostics tests into `zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/tests.rs` plus `zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/tests/{camera_targets,visibility_hzb_light,mesh_gpu_scene}.rs`. Status: `runtime_15_render_stats_product_diagnostics_tests_owner_split_static_passed_cargo_deferred_active_editor_lane`; guard: `runtime_15_render_stats_product_diagnostics_tests_are_child_owners`. This removes inline-test pressure from `product.rs` without changing product diagnostic paths, `RenderStats` projection, or the large-file ownership gate shape.

## M1 Decision Rules

Large files must be split by ownership, not by equal line ranges. A valid split moves behavior to named owner modules such as lifecycle, route policy, renderer extract, DTO conversion, diagnostics, manifest parsing, validation, or fixture support.

Root files may remain as structural facades. They should not keep behavior just to avoid updating call sites.

Do not preserve old compatibility wrappers while splitting. If a moved helper exposes a new owner, consumers should call the new owner path directly or the intended curated facade.

Any future `unclassified_hotspots` entry is a review blocker. Classify it with an owner class before accepting more behavior in that area.

## Owner Decisions

There are no current owner-decision groups because no production Rust file is at or above the 1000-line threshold. If a future audit reintroduces a hotspot, classify it into a concrete owner bucket before adding behavior there and before using the file as a Runtime 07 optimization target.

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
