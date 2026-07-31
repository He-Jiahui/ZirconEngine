---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: runtime-ui-template-authoring-linear-index-and-style-copy
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/05
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/template/asset/document.rs
  - zircon_runtime/src/ui/template/asset/document/validation.rs
tests:
  - 100k-node lookup and parent-index counter
  - 10k-rule single-edit clone-byte and selector-parse counter
  - duplicate id, invalid selector and undo-redo regression matrix
---

# Runtime UI template authoring线性查树与整表复制

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_runtime/src/ui/template` 83/83逐文件审查
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md`
- 交接原因：source document generation、undo/redo transaction与authoring index由EditorUI05统一拥有。

## 失败现象与复现证据

PERF-MVP-305：`node/node_mut/parent_of/child_mount`及结构编辑为每次操作全树DFS；style rule/sheet insert、replace、move复制全部stylesheets并重新校验id、重新parse全部selectors。authority validation的subtree clone已在本轮止损为borrowed map，但authoring热路径仍为O(document)。

## 最低共享层根因

`UiAssetDocument`是唯一source authority，却没有随generation维护node→parent/child、stylesheet/rule id和parsed selector索引；transactional validation只能靠clone整份候选文档回滚。

## 架构修复验收

- generation-owned node/parent/rule indexes随局部transaction增量更新，常规lookup近O(1)。
- style edit只验证changed rule及受id/selector约束影响的局部集合，rollback使用delta/undo record，不clone无关stylesheets。
- 1/100/10k/100k nodes和1/100/10k rules记录visits、selector parses、clone bytes、RSS与edit p95；单rule编辑无无关rule parse/clone。

## 禁止临时方案

- 不得只把BTreeMap换HashMap而继续每操作重建全索引。
- 不得跳过duplicate id/invalid selector校验换取速度。

## 修复结果与回传

Open state: `等待EditorUI05回传generation authoring index、delta validation与undo-redo规模证据`。
