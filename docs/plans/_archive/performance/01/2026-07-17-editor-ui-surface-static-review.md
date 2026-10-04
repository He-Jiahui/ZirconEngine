---
title: Editor UI surface and shell static performance review
date: 2026-07-17
status: static-reviewed-handoffs-open-dynamic-pending
related_code:
  - zircon_runtime/src/ui/surface/surface/rebuild.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/viewport_surfaces.rs
  - zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection/surface_frames.rs
  - zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection/surface_frames/pane_frame.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/viewport_toolbar/bridge.rs
plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
---

# Editor UI surface/shell 静态性能审查

## UiSurface 增量路径仍有全树元数据扫描

`rebuild_dirty` 在决定是否工作前分别调用 `dirty_flags()` 与 `dirty_node_count()`，对全部 node 做两次
扫描；成功后 `clear_dirty_flags()` 再扫描全部 node。增量 layout 虽然只访问 dirty subtree，但任意
单节点 hover/input/layout 失效仍至少支付三次 O(N) 元数据遍历。当前没有 surface-level dirty
aggregate、dirty node set/count 或失败时可保留的 generation。

修复必须由 Editor UI 02 在 mutation 边界维护 aggregate/set，并覆盖 `state_flags.dirty`、多 domain
union、rebuild error/retry 与显式 `clear_dirty_flags`；不能只删除诊断 count。详见
`docs/plans/zircon_editor/editor_ui/02/failure-2026-07-17-ui-surface-dirty-full-tree-scans.md`。

## Viewport toolbar surface/frame 重建风暴

`RetainedEditorHost::recompute_if_dirty` 的 slow path 每次调用
`sync_recompute_viewport_surfaces`。后者通过 `get_host_presentation` 深 clone 整份 presentation，随后
遍历四个 dock 和全部 floating windows。每个 Scene/Game pane 都在同一个 toolbar bridge 上：

1. 标脏所有 toolbar roots 并重跑 layout/hit-grid/render extract；
2. 重新投影 host nodes；
3. 新建临时 `UiSurface`、逐 control 构造带 String/metadata 的树并 `rebuild()`；
4. `surface_frame()` 再深 clone arranged tree/render extract/hit grid/layout report；
5. 重建 floating-window model，并把整份 presentation 写回。

该路径没有 `{projection generation, size, route/hit mapping}` cache，也不检查 pane 现有 frame 是否
仍有效。同一慢重算即使由无关 pane/status 变化触发，也会重复以上工作。已作为 `PERF-MVP-033`
移交 Editor UI 08；Editor UI 02/Editor01 共同负责 surface snapshot/cache 与 presentation patch
authority。必须用构建计数和 1/4/16 pane trace 验收，而不是只看单帧 wall time。

## 验收状态

这是 production call chain 的静态证据；尚未取得当前源码 editor WPR/Tracy trace，相关 runtime UI
与 editor retained-host 目录继续保留在 `pending.md`。现有 paint-only invalidation fast path是正确
基础，但不覆盖需要 presentation/hit/layout 的 slow path。
