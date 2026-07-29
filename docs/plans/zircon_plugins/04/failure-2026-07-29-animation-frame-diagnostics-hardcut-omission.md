---
handoff_kind: failure
status: open
created_at: 2026-07-29
summary_slug: animation-frame-diagnostics-hardcut-omission
origin_plan: docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
fixing_plan: docs/plans/zircon_plugins/04-animation.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/06
fixing_child_dir: docs/plans/zircon_plugins/04
plan_link_mode: child_record_only
related_code:
  - zircon_plugins/animation/runtime/src/runtime_system.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/events.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/pose_apply.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/requests.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/tick.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/parameter_apply.rs
  - docs/zircon_runtime/performance/hotspot_inventory.md
tests:
  - animation scene frame diagnostics record populated and empty animation.evaluate frames
  - python tools/check_conventions.py --only docs --json
  - cargo +1.94.1 check -p zircon_plugin_animation_runtime --lib --locked --jobs 1 --color never
---

# Plugins04: animation frame diagnostics hard-cut omission

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md`
- 来源执行切片：G7 retired machine-owner path convergence after the Animation
  Plugin `animation.evaluate` hard cut.
- 修复责任计划：`docs/plans/zircon_plugins/04-animation.md`
- 交接原因：Plugins04 owns the replacement evaluation pipeline and must preserve
  frame-observability behavior while physically removing Runtime `scene_hook`.
- 生命周期键：`animation-frame-diagnostics-hardcut-omission`

## 失败现象与复现证据

Fresh G7 reports twelve missing-path violations in
`docs/zircon_runtime/performance/hotspot_inventory.md` because both machine
fields still list the six deleted Runtime owners under
`zircon_runtime/src/animation/scene_hook/{diagnostics,events,node_pose,pending,scan,tick}.rs`.

Five responsibilities have current Plugin owners: event publication is in
`evaluation/pipeline/events.rs`, pose writeback is in `pose_apply.rs`, request
records are in `requests.rs`, scene scanning is in `parameter_apply.rs`, and
frame orchestration is in `tick.rs`. The sixth responsibility was not migrated:
repository-wide production search finds no current
`AnimationSceneFrameDiagnostics` definition and no producer for
`animation.scene.scanned_entities` or the other nine `animation.scene.*` frame
counters. Only stale documentation, structure-audit inventories, and guards
still mention them.

The retired implementation recorded scan/sample/output/writeback/event/state
transition counts on every active frame and explicit zeroes when animation was
unavailable or disabled. The current Plugin `tick_animation_world` performs the
same evaluation work but neither constructs nor records that frame diagnostic.

## 最低共享层根因

The `scene_hook` to `animation.evaluate` hard cut moved evaluation behavior into
the Animation Plugin but treated the Runtime performance diagnostic owner as a
path-only artifact. Removing the old module therefore also removed observable
behavior required by Runtime07 hotspot evidence. Updating the Markdown to a
nearby Plugin file would hide that loss instead of completing the migration.

## 架构修复验收

- Plugins04 gives the frame-count contract one current owner inside the
  evaluation pipeline and records the ten established `animation.scene.*`
  counters from the current scan/evaluation results.
- Missing-manager and disabled-playback paths publish explicit zeroes, preserving
  the distinction between no work and missing instrumentation.
- Event, pose, request, scan, and tick owners remain Plugin-local; Runtime
  `animation/scene_hook` stays physically absent with no alias, shim, facade, or
  compatibility module.
- Runtime07 performance inventories, guards, and
  `docs/zircon_runtime/performance/hotspot_inventory.md` hard-cut to the real
  Plugin owners only after the production behavior exists.
- Focused diagnostics tests, G7 docs validation, Runtime07 structure guards, and
  canonical Rust 1.94.1 Plugin compilation pass before fixed return.

## 禁止临时方案

- 不得恢复 Runtime `animation/scene_hook`，不得新增 alias、shim、转发 owner 或
  consumer-local duplicate counters。
- 不得仅修改 Markdown/静态守卫来声称当前不存在的 production counters 已迁移。
- 不得删除 counters、空帧语义或 Runtime07 验收项来消除 stale-path 失败。

## 修复结果与回传

Open state: `待修复`; Plugins04 production diagnostics and the upward Runtime07
and Frameworks06 gates have not passed. Independent Frameworks06 work may
continue on unrelated G7 documents, but this twelve-violation slice cannot be
counted as converged until the behavior-preserving hard cut is returned fixed.
