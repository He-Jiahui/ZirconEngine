---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: world-inspection-generation-projection
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor/05
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/scene/inspection/snapshot.rs
  - zircon_runtime/src/scene/world/query.rs
  - zircon_editor/src/scene/viewport/edit_mode_projection/build.rs
  - zircon_editor/src/scene/viewport/mod.rs
tests:
  - cargo test -p zircon_runtime --lib inspection --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib editing::editor_projection --locked --jobs 1 -- --nocapture --test-threads=1
---

# Editor05：World inspection generation projection交接

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：Runtime scene inspection 5/5逐Rust文件性能审查，PERF-MVP-456
- 修复责任计划：`docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md`
- 交接原因：Editor05拥有Hierarchy/Inspector/viewport edit-mode projection与F4启用边界；Editor02共同拥有generation/delta消息，Runtime07提供World change projection。
- 生命周期键：`world-inspection-generation-projection`

## 失败现象与复现证据

Runtime每次inspection全量`node_records()` project/clone/sort，重建多份hierarchy/hash容器；selected fields全扫TypeRegistry并复制metadata/value。editor test-only consumer再构造第二套owned hierarchy/inspector DTO，`build_stats`第二次`node_records()`全场扫描。`generation/subtree_hash`当前只是输出字段，没有阻止stable generation重建。

本轮只删除focus第二遍、parent BTreeMap、field-name临时clone并预分配基础容器；这些局部止损不允许直接解除editor consumer的`cfg(test)`门。

## 最低共享层根因

World没有发布按hierarchy/name/active/reflection generation封存的inspection artifact/delta；Editor05各consumer也没有共享同一projection owner。Runtime与Editor两套owned DTO让cache放在哪一侧都会形成第二份truth。

## 架构修复验收

- Runtime07按world hierarchy/name/active/type/component generation发布immutable inspection row/field artifact与added/changed/removed delta；subtree hash只重算changed row到ancestor chain。
- Editor05 Hierarchy、Inspector、viewport stats共享同一artifact；stable generation零producer build/scan/clone，selection-only只切field projection，不重建hierarchy。
- Editor02传递generation/delta与backpressure，不按idle frame重复请求全量snapshot；consumer落后时按generation合并并可显式请求一次resync。
- 删除editor第二套完整runtime DTO复制或把它收敛为borrowed/Arc view；解除`cfg(test)`必须由F4产品trace与Cargo门共同批准。
- 1/1k/10k/100k nodes、depth 1/64/5k、types/components 1/100/10k及stable/rename/reparent/selection/field edit记录node/type/subtree visits、clone bytes、build/delta/resync count、queue age与p95；工作随dirty范围而非total scene或frame count增长。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止仅在Editor缓存完整WorldInspection而没有精确Runtime generation/delta；禁止Runtime/Editor各持一份独立authoritative hierarchy。
- 禁止用固定帧节流掩盖全量重建；变更延迟/合并必须有generation与最大age语义。
- 禁止在未完成规模/Cargo/F4验收前直接移除`cfg(test)`。

## 修复结果与回传

Open state: `待修复`; no pass is claimed.
