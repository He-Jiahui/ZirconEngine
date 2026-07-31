---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: runtime-ui-surface-frame-full-copy-and-ecs-reprojection
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/08
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/tests/surface_frame_authority.rs
  - zircon_runtime/src/ui/tests/surface_frame_authority
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/surface/ecs_projection.rs
  - zircon_runtime/src/ui/surface/surface/rebuild.rs
  - zircon_runtime_interface/src/ui/surface/frame.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/viewport_toolbar/bridge.rs
tests:
  - stable-generation 1000-frame zero-payload-clone test
  - ECS dirty-delta changed-node-only test
  - independent window-layout-render-input generation test
---

# Runtime UI SurfaceFrame全量复制与ECS重投影

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：surface frame/ECS/rebuild与editor consumer追踪
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`
- 关联failure：`failure-2026-07-17-viewport-toolbar-surface-rebuild-storm.md`
- 交接原因：surface frame publish、workbench presentation generation和toolbar consumers由EditorUI08收束。

## 失败现象与复现证据

PERF-MVP-278：每次frame access深clone arranged/render/hit/focus并重建pipeline Strings和全树ECS snapshot；delta helper也先full project。toolbar/temporary hit surface会在产品recompute中实际调用，放大既有storm。

## 最低共享层根因

UiSurface只存mutable stage artifacts，跨owner读取靠重复owned snapshot；没有published immutable frame generation，也没有从dirty transaction直接产ECS delta的changed set。

## 架构修复验收

- 每stage变化时发布一次immutable Arc frame data；stable consumer只clone handle，payload clone/projection/report build=0。
- window/rebuild stats提供轻量view，不强制携带tree/render/hit/ECS。
- ECS projection随changed nodes增量更新，delta不构造两份full snapshots。
- 1/1k/10k nodes及1k frame accesses记录published/clone bytes、projection visits、Arc count和CPU p95；ABI/serde/toolbar hit通过。

## 禁止临时方案

- 不得在每个toolbar/pane consumer各缓存一份可陈旧的owned frame。
- 不得用Arc包住每次重新物化的新全图；publish generation必须阻止构建本身。

## 修复结果与回传

Open state: `等待EditorUI08回传immutable frame publish、ECS delta与toolbar整合证据`。
