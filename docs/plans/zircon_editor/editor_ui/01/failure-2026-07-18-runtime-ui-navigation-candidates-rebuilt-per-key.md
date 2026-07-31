---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: runtime-ui-navigation-candidates-rebuilt-per-key
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/tree/node/focus.rs
  - zircon_runtime/src/ui/surface/input/effect/navigation.rs
  - zircon_runtime/src/ui/surface/surface/event_routing.rs
---

# Runtime UI每导航键全量重建候选

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_runtime/src/ui/tree` 9/9及surface产品导航调用图
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md`
- 联动责任：EditorUI02提供layout/tree generation和bounds变化信号；UE Slate `FNavigationMetaData`作为explicit/custom/wrap/stop/escape语义参考。
- 交接原因：统一输入路由、focus lifecycle和navigation index属于EditorUI01。

## 失败现象与复现证据

PERF-MVP-253：每个Tab、Shift-Tab或方向键都递归遍历整棵tree重建candidate Vec；每个focus node沿parent链求modal root并clone group id，之后retain+sort全部候选。manual group target再次独立重建/排序。最坏O(N*depth + N log N)/key，发生在基础editor键盘路径。

## 最低共享层根因

focusability、tab/group/modal metadata和layout bounds只存于通用tree nodes，没有随tree/layout generation发布可复用的navigation index；输入事件因此承担了本应在结构变化时完成的派生数据构建。

## 架构修复验收

- tree/layout generation提交时构建tab order、group首项、node→position/group/modal ancestry和paint order index；mutation按受影响scope失效。
- stable generation按键candidate rebuild、sort、ancestor walk与group-id String clone=0；Tab/Previous近O(1)。
- directional query限定在active modal/group scope并使用generation-owned候选/bounds；先以规模counter证明，再选择grid/BVH等空间结构。
- manual Node/Group、Auto/Blocked、wrap、nested modal、focus deletion/reparent、Home/End/Activate/Cancel语义不变。
- 100/1k/10k nodes、1/10/100 groups、depth 1/16/64连续10k key记录visited/sorted/ancestor steps/alloc bytes、index rebuild scope与CPU p50/p95。
- current-source Cargo、editor workbench键盘/IME/popup/focus行为矩阵与产品trace通过。

## 禁止临时方案

- 不得只reserve candidate Vec而保留每key全树遍历和排序。
- 不得用不受tree/layout generation失效约束的全局cache。
- 不得为了O(1)删除manual、modal或UE式navigation boundary语义。

## 修复结果与回传

Open state: `等待EditorUI01联动EditorUI02回传generation-owned navigation index、规模counter、current-source Cargo与产品键盘trace`。
