---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: runtime-ui-render-extract-cache-command-identity
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/08
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/surface/render/extract.rs
  - zircon_runtime/src/ui/surface/render/cache.rs
  - zircon_runtime/src/ui/surface/surface/rebuild.rs
  - dev/slint/internal/core/item_rendering.rs
  - dev/slint/internal/core/partial_renderer.rs
tests:
  - stable generation zero-command-build test
  - multi-command node cache identity and damage test
  - owner-specialized surface pixel parity test
---

# Runtime UI render全量extract与多command cache identity冲突

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`surface/render` 32/32
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`
- 联动责任：EditorUI02提供arranged changed ranges；EditorUI04提供compiled visual descriptor；Render17验证GPU产物。
- 交接原因：surface frame artifact、stage generation与render extract/cache发布属于EditorUI08。

## 失败现象与复现证据

PERF-MVP-288：每帧先完整extract再做cache equality，14类renderer逐node探测；cache只按node id存一条command，却允许同node多command相互覆盖。本轮已让invisible node提前退出、stable hit不再clone cached command，主要架构问题未变。

## 最低共享层根因

dirty changed set没有跨越arranged到render，command也缺node-local stable identity/range；所谓cache位于昂贵构建之后。

## 架构修复验收

- per-node generation-owned command range，command identity包含node+local role或稳定range slot。
- stable generation render build/equality/style/text probes=0；changed nodes只patch对应range和damage。
- 1/100/1k/10k nodes与multi-command controls记录build/clone bytes、hit/rebuild/damage和CPU p95。
- owner/specialized surface、z/clip/opacity/text/image、serde/Cargo与产品像素通过。

## 禁止临时方案

- 不得只把cache key改成`(node_id,index)`而每帧仍全量extract。
- 不得用错误的reused统计掩盖post-build equality成本。

## 修复结果与回传

Open state: `等待EditorUI08回传generation command ranges、pre-build invalidation cache与规模/像素证据`。
