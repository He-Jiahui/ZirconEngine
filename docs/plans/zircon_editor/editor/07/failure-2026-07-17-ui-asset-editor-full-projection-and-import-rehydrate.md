---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: ui-asset-editor-full-projection-and-import-rehydrate
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor/07
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/ui/host/asset_editor_sessions
  - zircon_editor/src/ui/asset_editor
reference_sources:
  - dev/slint/internal/core/model/repeater.rs
tests:
  - source typing and palette drag projection build-count stress
  - diamond/cyclic import read/parse-count regression
  - UI asset editor presentation byte/order parity
---

# Editor07：UI asset editor 每 mutation 全投影与 import graph 重水化

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`ui/host/asset_editor_sessions` 19/19 Rust 文件逐文件审查
- 修复责任计划：`docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md`
- 交接原因：domain document generation、import dependency graph、presentation projection 与 editor gesture cadence 必须由 Editor07 UI asset editor session 统一拥有。

## 失败现象与复现证据

多数 binding/inspector/style/node/palette mutation 在 session 锁内更新后立即 `sync_ui_asset_editor_instance`，后者构造完整 reflection model并把 title/dirty/route 写回 view。source typing 还在每次 edit 后递归 `hydrate_ui_asset_editor_imports`，逐引用 resolve path、读盘、parse、project v2，再替换全部 resolved imports。palette pointer drag 原先即使状态未变也重建 projection，本轮已只在 changed 时同步；其余真实变更仍是全量。

import traversal 只在读盘/parse/insert之后检查 normalized visited id，diamond/cycle 会按重复边重新读取/解析同一物理文档后才停止递归；不同 fragment alias 又要求共享物理 parse但保留逻辑 reference rows，不能靠简单提前 return 修复。Slint repeater 对 row change 定点更新对应 instance或标 dirty，model 未 dirty 时直接返回；Zircon 需要相同的 document/property generation，而非每个 UI gesture 重建整份 presentation/import graph。

## 最低共享层根因

session mutation API 只返回 bool，不返回 dirty domains/changed nodes；reflection、preview、route、import graph 没有 generation cache或 physical-document parse cache，因此 host 只能统一 full sync/full hydrate。

## 架构修复验收

- mutation 返回 typed dirty domains/ids；source/property/palette/selection 只重建受影响 presentation rows，单帧相同 domain 合并一次。
- import graph 按 canonical physical asset id 缓存 parse/project结果，并保留 reference+fragment aliases；diamond/cycle 每 physical document 每 generation read/parse≤1。
- source typing 采用 parse/dependency debounce或增量 parser；磁盘 I/O 不在每 key event 主线程执行，最后 revision与 diagnostics 可观测。
- 1k typing/drag/selection stress 记录 build/read/parse/clone count与p95；save/undo/redo/conflict/preview/serialized route行为等价。

## 禁止临时方案

- 不得只延迟 UI paint却继续每输入读盘/parse全图。
- 不得按 normalized path 简单跳过而丢失不同 fragment alias或 expected-kind诊断。
- 不得让后台 parse 旧 revision 覆盖新 source generation。

## 修复结果与回传

Open state: `待 Editor07 实现 domain-delta projection、generation import graph/cache 与 typing debounce，并回传规模/交互/alias parity`。
