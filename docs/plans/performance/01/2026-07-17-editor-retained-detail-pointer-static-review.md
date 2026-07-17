---
related_code:
  - zircon_editor/src/ui/retained_host/detail_pointer
  - zircon_editor/src/tests/host/retained_detail_pointer
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
reference_sources:
  - zircon_runtime/src/ui/surface/surface/event_routing.rs
  - zircon_runtime/src/ui/tree/node/scroll.rs
tests:
  - detail scroll source boundary RED then GREEN
  - existing retained detail pointer behavior tests and current-source Windows focused Cargo pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor Retained Detail Pointer 逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`zircon_editor/src/ui/retained_host/detail_pointer` 当前共 **27** 个 Rust 文件，已逐文件阅读 **27/27**，覆盖 console、inspector、asset-details extent/layout、共享 scroll state、surface 构建与分发。动态 Cargo 与 1k-scroll interaction trace 尚未完成，因此继续留在 `pending.md`。

## 主要结论与直接修复

- console/inspector/asset-details 共用只有 root+viewport 的 pointer surface。runtime default scroll fallback 已通过 `UiRuntimeTreeScrollExt::scroll_by` 原地提交并 clamp viewport offset，但 bridge 随后仍调用 `rebuild_surface()`，重复创建 tree id/path、route map、arranged tree、hit grid 与 render extract。
- 已先加入源码边界回归并确认旧源码为 RED，再删除 scroll hot path 的 `rebuild_surface()`；bridge state 从已更新 viewport 读取 offset，route/state/clamp 语义不变。layout 或外部 state generation 变化仍由 `sync()` 重建。
- asset-details 的五个固定 section 高度曾每次构造临时 `Vec`，现改为编译期常量和可选 diagnostics 高度求和，保留共享 section-count 语义且无临时堆分配。
- console text extent 对正文逐字符估算仍与文本长度线性，但只应在 console generation 或宽度变化时调用；后续动态 trace 需确认上层 generation gate。

## 待验收

运行 `retained_detail_pointer` focused suite，覆盖连续 scroll、overscroll clamp、route intent、console/inspector/asset-details；1k scroll 记录 `rebuild_surface`、tree/path allocation 与 p95。通过前不进入 `review.md`。
