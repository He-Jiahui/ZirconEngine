---
owner_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
milestone: M2
slice: ui-asset-watcher-bounded-refresh
status: in_progress
related_code:
  - zircon_editor/src/ui/host/asset_editor_sessions/watcher.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/watcher
  - zircon_editor/src/ui/host/asset_editor_sessions/refresh/reconcile.rs
  - zircon_editor/src/ui/asset_editor/session/import_reference_access.rs
tests:
  - tools/tests/test_editor09_ui_asset_watcher_bounded_refresh_contract.py
  - zircon_editor/src/ui/host/asset_editor_sessions/watcher/tests.rs
  - zircon_editor/src/tests/host/manager/ui_asset_workspace_watcher.rs
---

# Editor09 UI asset watcher bounded refresh

## 目标与边界

- 硬删除 `crossbeam_channel::unbounded` 与 `while try_recv` drain-all 路径。
- notify callback 只写入容量受限、按物理路径去重的 latest-set ingress；重复事件不重复占用队列。
- 每次 retained poll 的 ingress/reconcile 枚举阶段同时受条目数和 wall-time 预算约束，剩余工作跨 tick 保留；同步 parse/hydrate 仍是后续 worker 切片。
- ingress 溢出不能静默丢失最终状态：转为当前打开 UI asset 文档及其导入根的 reconcile cursor，继续按预算处理。
- poll 返回 typed report，公开 pending、reconcile-cursor-active、coalesced、overflow、oldest-age 与 budget-exhausted 诊断；不保留旧 `Vec<String>` 返回兼容入口。
- 本切片不创建第二资产 inventory、第二文件 watcher 或 UI 私有导入器；文件解析、冲突和 stale import 仍由现有 refresh owner 执行。

## 架构裁决

- 采用 mutex 保护的 bounded latest-set，而不是 bounded channel：channel 满时无法保留“该路径仍需处理”的最终状态。
- 不采用完整事件 ring：rename/write burst 的逐事件历史不是当前 UI asset editor 的权威，保留它会让容量随重复事件增长。
- latest-set 达到容量后只记录一次 overflow generation，并丢弃该 generation 的不完整 path 集；host 以 `{session id, next import index}` cursor 按同一预算 borrowed 枚举打开文档，不先物化完整 ID 集。
- `watcher.rs` 仅作 module façade；budget、diagnostics、ingress、path identity、service、host integration 与 tests 分属 folder-backed owner。

## 实施切片

### W1 契约与 RED

- [x] 新增静态合同，拒绝 `unbounded`、`while try_recv` 与旧 `Vec<String>` poll surface，并锁定 folder-backed owner。
- [x] 新增真实状态测试：10,000 同路径事件只保留 1 项；容量溢出后 pending 不超过上限且 reconcile 标志可见。
- [x] 新增 poll 预算测试：每 tick 最多处理配置项数，剩余路径跨 tick 保留，重复路径只返回一次。
- [x] 新增 oldest-age、累计 coalesced/overflow 与 budget-exhausted 诊断测试。

### W2 Bounded ingress 与预算化 poll

- [x] 实现 typed `UiAssetWatchBudget`，拒绝零容量、零条目预算和零时长预算。
- [x] 实现 bounded latest-set ingress；callback 临界区只做路径去重与计数，不做 I/O、解析或 host 调用。
- [x] 实现路径到唯一 `res://` ID 的 borrowed projection，保留多 root 歧义拒绝与 `.zui` 过滤。
- [x] 实现 count/time 双预算、跨 tick cursor 与 typed poll report。

### W3 Overflow reconcile 与 public hard cut

- [x] overflow 时以增量 cursor borrowed 枚举当前打开 session 的 route + direct import root；单 session 不 clone 完整 imports，transitive import 由现有 traversal 重新展开。
- [x] reconcile 枚举与新文件事件共享同一 count/time allowance，session 锁只覆盖有界批次；优先完成 overflow generation，之后恢复正常 ingress。
- [x] `EditorManager::poll_ui_asset_workspace_watcher` 硬切返回 typed report，并迁移全部仓库内调用方。
- [x] 更新 watcher 模块文档，明确容量、最终一致、锁域与尚未落地的 generation reverse dependency/worker commit 边界。

### W.T 测试阶段

- [x] 静态合同、exact-scope rustfmt、结构预算与 `git diff --check` 通过。
- [ ] Coordinator01 validation-copy 闭包修复后，运行 watcher focused Rust tests 与 `zircon_editor --lib` broad gate。
- [x] 独立 review 从 `0/3/0` 收敛到 Critical/Important/Minor=`0/0/0`：容量恢复、增量 reconcile 与二次 overflow 诊断均已复核闭合。
- [ ] 后续反向依赖 generation、异步 parse/commit 与 1k/10k 产品 p95 仍在原 Failure 中保持 open，不用本基础设施切片提前关闭整个 Failure。

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 证据/剩余项 |
| --- | --- | --- | --- |
| 2026-07-23 07:39 +08:00 | in_progress（source/static/review ready） | 完成 bounded latest-set、共享 count/time allowance、跨 tick reconcile cursor、typed diagnostics/public hard cut；将 borrowed import accessor 隔离到独立 session owner，未吸收 `lifecycle.rs` 的 foreign palette 改动 | Python 合同 5/5、import-flow 邻接守卫 8/8、exact rustfmt/`git diff --check`/文件预算通过，独立 review=`0/0/0`；Coordinator01 validation-copy 外部 sibling path 闭包 Failure 仍阻断 current-source Cargo，故不写 accepted/fixed/commit，原 watcher Failure 保持 open |
