---
related_code:
  - zircon_editor/src/ui/retained_host/route_intent
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/01/failure-2026-07-17-retained-route-intent-owned-payload-cloning.md
reference_sources:
  - dev/bevy/crates/bevy_ui/src/layout/ui_surface.rs
  - dev/bevy/crates/bevy_ui/src/layout/debug.rs
tests:
  - hash-index source contract RED then GREEN
  - existing retained pointer route suites and Windows focused Cargo pending
  - 1k move/click route clone-count profile pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor Retained Route Intent 逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`zircon_editor/src/ui/retained_host/route_intent` 当前共 **2** 个 Rust 文件，已逐文件阅读 **2/2**，覆盖 node→route、route→typed intent 绑定和 pointer dispatch target fallback。动态 Cargo 与 clone-count profile 未完成，因此继续留在 `pending.md`。

## 主要结论与直接修复

- 两个索引原为 `BTreeMap`，每次 pointer event 先查 node，再查 route；代码不暴露有序迭代，键均为实现 `Hash` 的 `u64` ID。已用 `HashMap` 替换，源码契约 RED→GREEN，路由 fallback 与 typed intent 语义不变。
- document/drawer/menu/activity/hierarchy/host-page/toolbar/welcome typed accessor 仍克隆 route；payload 可含多个 `String` 或 `Vec<usize>`，move-only 路径也可能深复制。本轮不把 route 类型局部改成另一套 owner，已交 EditorUI01 建立 stable handle/immutable payload。
- Bevy UI surface 用 `EntityHashMap` 和 `HashMap<NodeId, Entity>` 维护 UI identity 映射；Zircon 保留 typed route，但热查找不需要排序树，行/路由确定性由生成顺序与稳定 ID 维护。

## 待验收

运行所有 retained pointer route focused suites，覆盖 `handled_by`/target fallback、missing route、各 typed variant；1k move/click 记录 lookup、String/Vec clone 与 allocation。payload clone 归零且 route parity 通过前不进入 `review.md`。
