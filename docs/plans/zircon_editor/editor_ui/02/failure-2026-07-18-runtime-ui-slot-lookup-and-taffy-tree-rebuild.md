---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: runtime-ui-slot-lookup-and-taffy-tree-rebuild
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/02-layout-taffy-and-containers.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/02
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/tests/layout_slots.rs
  - zircon_runtime/src/ui/tests/layout_slots
  - zircon_runtime/src/ui/tests/taffy_layout_pass.rs
  - zircon_runtime/src/ui/tests/taffy_layout_pass
  - zircon_runtime/src/ui/layout/pass/slot.rs
  - zircon_runtime/src/ui/layout/pass/measure.rs
  - zircon_runtime/src/ui/layout/pass/arrange.rs
  - zircon_runtime/src/ui/layout/pass/taffy_arrange.rs
  - zircon_runtime/src/ui/layout/taffy_bridge/compute.rs
  - dev/bevy/crates/bevy_ui/src/layout/ui_surface.rs
tests:
  - 10k-child slot lookup visit-count test
  - persistent Taffy stable-frame tree-operation test
  - reparent/remove/style/content-measure parity matrix
---

# Runtime UI slot线性查找与逐容器重建Taffy树

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：layout slot/measure/axis/arrange/Taffy bridge逐文件调用图
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/02-layout-taffy-and-containers.md`
- 参考：Bevy `UiSurface`持有persistent `TaffyTree<NodeMeasure>`、entity映射和children scratch。
- 交接原因：slot edge authority与persistent Taffy surface属于EditorUI02布局树所有权，不能在performance计划建立平行缓存。

## 失败现象与复现证据

PERF-MVP-260：每次slot lookup线性扫描全局slot Vec，同一child在排序、测量、引擎资格、Taffy input和arrange重复查找。PERF-MVP-261：每个Taffy容器每次arrange新建树、插入leaf和parent、compute后丢弃，nested UI放大短命分配并失去Taffy cache。`tests/layout_slots.rs`与目录的11个测试最多只有4个children，只锁定slot像素/render/hit语义，未记录slot probes或stable-frame Taffy操作；PERF-MVP-263已局部删除ordered desired O(N²) payload find，但未解决slot根因。

## 最低共享层根因

tree edge没有O(1) slot authority，布局stage也没有共享compiled child input；Taffy tree不是surface/layout generation拥有的持久求解结构。

## 架构修复验收

- tree generation维护`(parent,child)->slot`索引或edge-owned slot；重复/缺失/reparent有显式合同。
- 一次生成ordered child/layout input并供measure/engine/arrange共享，单pass slot entries访问总量O(edges)。
- surface长期持有Taffy tree与node映射，changed style/context/children精确upsert/remove；stable frame tree create/insert=0。
- 100/1k/10k nodes与1/100 nested containers记录slot probes、sort、tree create/insert/style/children/compute、alloc bytes和CPU p95；Taffy/Zircon fallback与current-source Cargo通过。

## 禁止临时方案

- 不得在每个consumer建立自己的slot map或无界Taffy subtree cache。
- 不得用全量persistent tree重建替代当前逐容器重建。

## 修复结果与回传

Open state: `等待EditorUI02回传edge slot authority、shared child input和persistent Taffy surface及规模counter`。
