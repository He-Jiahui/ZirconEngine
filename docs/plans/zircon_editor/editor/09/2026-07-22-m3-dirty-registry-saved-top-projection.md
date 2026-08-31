---
owner_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
milestone: M3
slice: dirty-registry-saved-top-projection
status: implementation_complete_static_validation_complete_managed_validation_blocked
related_code:
  - zircon_editor/src/core/asset/mod.rs
  - zircon_editor/src/core/asset/dirty/mod.rs
  - zircon_editor/src/core/asset/dirty/error.rs
  - zircon_editor/src/core/asset/dirty/external_effect_id.rs
  - zircon_editor/src/core/asset/dirty/registry.rs
  - zircon_editor/src/ui/host/editor_save_batch.rs
  - zircon_editor/src/ui/retained_host/app/document_save.rs
tests:
  - zircon_editor/tests/editor_asset_facade.rs
  - zircon_editor/src/core/asset/dirty/tests.rs
  - tools/tests/test_editor09_dirty_registry_contract.py
  - zircon_editor/src/ui/retained_host/app/tests/document_save.rs
---

# Editor09 M3.1 DirtyRegistry Saved-Top Projection

Plan: `docs/plans/zircon_editor/editor/09-editor-asset-management.md`

Milestone: M3

Status: `source_complete_static_green_review_clean_cargo_blocked`

## 范围

本切片实现 M3.1 的安全子集：以 Editor03 `saved_top` 为唯一 transaction dirty 权威，补充 typed、
revisioned external-effect ledger，为关闭询问和后续 save/save_all 提供统一快照。它不缓存 transaction
dirty、history top 或保存基线，也不执行文件 I/O。

## 实施阶段

- [x] 注册并领取 8 文件精确 Session scope。
- [x] 以静态合同和 Rust 行为合同锁定 RED。
- [x] 实现 folder-backed `core/asset/dirty/` owner，按 façade/error/id/registry/test 拆分。
- [x] 单文档快照实时调用 `EditorTransactionEngine::is_dirty(Document)`；首次/reset delta 按 DocumentId 排序。
- [x] external effect 使用开放 typed ID 与单调 revision；未知文档不隐式注册，unregister 原子移除 ledger。
- [x] 旧实现独立初审 `0/2/0` 的 stale revision 误清与双来源 false-clean 已以 revision
  compare-and-clear、generation 复核和 typed unstable error 关闭。
- [x] 性能 failure 修复把旧 `snapshots/dirty_snapshots` 整批入口物理删除，改为 registry-bound cursor、
  `changes_since` delta 与 typed removal；无兼容别名。
- [x] 当前静态合同 8/8 GREEN，精确 Rust 文件已格式化。
- [x] 当前 cursor-delta 两路独立终审均为 `0/0/0`；旧整批实现的 review 不冒充本轮证据。
- [x] 由 exact18 successor 统一领取共享 asset façade，并以唯一 typed re-export 挂载本 owner。
- [ ] Editor03 原子 save token 回传后完成 save/save_all/关闭询问写回闭环与 Cargo 验收。

## TDD 证据

- RED：生产 owner 缺失时静态合同为 1 failure + 4 errors。
- GREEN：`python -m unittest tools.tests.test_editor09_dirty_registry_contract -v` 为 8/8；与 Editor03
  save-token/dirty-batch 合并静态门为 13/13。
- Rust 合同源码覆盖：edit→dirty、mark saved→clean、undo→dirty、redo 回 saved_top→clean；external
  effects typed 排序、revision compare-and-clear；未知文档、typed unregister、cursor lineage、10,000
  文档 stable/single-delta，以及 external generation 切换时只重试 changed projection 而不返回 false-clean。
- owner 已挂载但尚未完成受管 Cargo，故本记录不宣称 Rust/Cargo GREEN。

## 外部依赖裁决

Editor03 已硬切旧 `mark_saved`，当前使用 engine-bound save token 在同一 operation/state lock 内
compare-and-mark；源码与独立复审已完成，受管 Cargo/fixed return/managed SHA 尚未完成。原 failure 保持：
`docs/plans/zircon_editor/editor/03/failure-2026-07-22-saved-top-compare-and-mark-save-token.md`，要求
Editor03 提供同锁 compare-and-mark typed token。Editor09 不以 UI 单线程假设、禁事务或平行 saved_top
规避。

## 架构与性能

- Registry 只持有 `BTreeSet<DocumentId>`、有界 document-change journal 与
  `DocumentId → typed effect/revision`；transaction dirty 永不驻留，避免第四套 dirty bool。
- 首次/journal Reset 才遍历全注册集；稳定 cursor 不访问 journal entry，live delta 根据连续 generation
  直接定位 retained suffix。external change 只点查 changed document，Editor03 transaction delta 再覆盖竞态。
- 不读取 sidecar、digest 或资产本体；保存触发 reimport 时也不会自动 mark external effect，避免热重载
  把刚保存文档再次置脏。

## 产出记录与时间

- 2026-07-22：状态 `source_complete_static_green_review_pending_mount_pending`。已完成 8 文件精确
  scope、TDD RED→GREEN（静态 6/6）、saved_top 实时投影、typed/revisioned external-effect ledger、
  deterministic multi-document snapshot 和生命周期测试源码。发现 save completion 缺少 Editor03
  原子 compare-and-mark token，已写入对应功能 failure；在其回传及 asset façade 前序提交完成前，
  不伪造保存成功、不宣称 Cargo GREEN，父 M3 保持 `pending`。
- 2026-07-22：独立初审 `0/2/0` 的 stale clear 与 false-clean finding 已修复。external clear 现在
  强制比较 expected revision；document/register/unregister/effect mutation 推进 generation，单文档与
  整批 snapshot 在 Editor03 查询后复核并最多重试 8 次，持续抖动返回 typed unstable error。新增两个
  确定并发合同，静态门保持 6/6 GREEN；等待复审。
- 2026-07-22：最终独立复审 `0/0/0`。确认 expected-revision compare-and-clear、单文档与整批
  generation 复核/有限重试、失败和无变化不推进 generation、typed unstable error 均闭环；状态更新为
  `source_complete_static_green_review_clean_mount_pending`。仍须等待前序 asset façade 受管提交与
  Editor03 save token 回传后再挂载/验证，父 M3 保持 `pending`。
- 2026-07-22 性能复核：`DirtyDocumentSnapshot`已从ID Vec+BTreeMap双owner改为单份sorted ID Vec
  与平行revision Vec，revision查询binary search，源码守卫、rustfmt/diff通过。整批snapshot仍复制
  全document effect maps、逐document访问Editor03 history并可能重试8轮；新增
  [open failure](failure-2026-07-22-dirty-registry-snapshot-retry-clone-budget.md)要求batch saved-top/dirty
  generation与delta projection。Cargo/规模/F4未完成，状态不变。
- 2026-07-22：状态 `source_mounted_static_green_review_clean_cargo_blocked`。exact18 successor 已在
  `core/asset/mod.rs` 私有挂载 `dirty` 并重导出 6 个 snapshot/effect/revision/registry/error typed API，
  新增 crate 外部 Rust consumer；挂载合同改为解析 `pub use` exact token set，同时核对 owner 三个
  re-export block，关闭独立初审 `0/1/0` 的 substring 假绿 finding。dirty `7/7`、import+dirty 合并
  静态门 `14/14` 与 exact Rust rustfmt 通过，终审 Critical/Important/Minor=`0/0/0`；baseline221 候选
  snapshot916 已冻结 exact18 全部路径。受管 Cargo 仍被 Coordinator01 validation-copy
  manifest graph 闭包故障阻断，Editor03 compare-and-mark token failure 继续 open，父 M3 保持 `pending`。
- 2026-07-22 Editor03 dependency source return：dirty consumer test 已硬切为
  `capture_save_token`→`mark_saved_if_unchanged`，不再调用已删除的 transaction `mark_saved`；Editor03
  exact12 静态合同与本计划合同合并 `11/11`。因此 snapshot918 中本记录与 `dirty/tests.rs` 已由
  Editor03 successor 接管并合法漂移，剩余 Editor09 16 路径不变；在 Editor03 独立复审、受管 Cargo、
  fixed return 与 SHA 完成前，save/save_all 仍不写为完成。
- 2026-07-22 Editor03 dependency review return：successor scope 扩展为 exact14，补齐 engine lineage、active
  scope 拒绝与 operation-group pre-begin capability/失败恢复/successor 隔离；合并静态合同保持 `11/11`，
  两路独立终审均为 Critical/Important/Minor=`0/0/0`，candidate snapshot936 preview 为 successor exact14
  `14/14` 无漂移。本计划继续只消费 typed save token；受管 Cargo、fixed return 与 Editor03 managed SHA
  未完成前，M3 save/save_all 仍保持 pending。
- 2026-07-22：状态 `source_complete_static_green_review_pending_cargo_blocked`。性能 failure 将旧整批
  snapshot API 硬切为 cursor delta；10,000 文档合同覆盖 initial reset、stable empty、single-change，当前
  静态门 8/8。独立初审 `0/1/2` 指出 live delta 仍扫描完整 retained journal、失败发布矩阵不足及文档陈旧；
  已改为 generation→suffix 起点并加入 stable=0/single=1 journal-visit counter，补 undo/redo/clear/失败
  no-delta 合同并更新本文。等待独立复审与受管 Cargo，旧 review-clean 证据不冒充当前实现验收。
- 2026-07-22：状态 `source_complete_static_green_review_clean_cargo_blocked`。后续复审发现 visit counter
  预填算术仍可假绿且 test helper 对 sibling 不可见；现已把两层 suffix 遍历统一为
  `VecDeque::range(start..)`，在真实 iterator yield 中计数，并修正 `pub(super)`。两路最终复审均为
  `0/0/0`，专项静态 8/8、相关 Editor03/09 静态 48/48、exact rustfmt/diff-check GREEN；Cargo/产品
  trace/fixed return/commit 继续 pending。
- 2026-08-23：状态 `implementation_complete_static_validation_complete_managed_validation_blocked`。新增
  `ui/host/editor_save_batch.rs` 作为唯一产品 batch coordinator：从同一份 toolkit snapshot 建立
  `SaveDirtyViewsRequest`，以 DirtyRegistry save token 和 dirty generation 驱动既有
  `SaveDirtyViewsJobAdapter`，worker 只调用 DocumentToolkit 的 write hook，完成后才统一
  compare-and-mark/清 external effect 并刷新 workbench dirty projection。native close prompt 的 Save
  改为异步提交和 tick 回收；保存中拒绝重复动作，完成后重新查询当前 target 的 dirty views，只有集合为空
  才提交关闭，failed/cancelled/stale 或新 generation 均回到可重试提示。close-prompt UI asset 回归已改为
  验证“点击后窗口仍打开，tick 后才关闭”。受影响 Rust 文件 `rustfmt --check` 与 scoped
  `git diff --check` 通过（仅 LF/CRLF 提示）。当前 coordinator 仍因外部未登记 D/E/F Cargo target
  被 managed validation 拒绝，未运行 Cargo 或声明运行时通过；全局 Save All 命令 consumer 尚未接线，
  因此 M3 和本 failure 不标记 accepted/completed。
- 2026-08-23：状态 `implementation_complete_static_validation_complete_managed_validation_blocked`。全局
  `Save All Documents` 已作为独立 `MenuAction::SaveAllDocuments` 与
  `EditorEventEffect::DocumentSaveAllRequested` 接入 command registry、workbench ID 映射、runtime
  reflection、retained-host effect owner 和 tick completion polling；它只调度上述唯一 document batch
  coordinator，不把旧 `Save Project` 的场景文件 I/O 伪装成 document save。关闭询问保存中的 batch 与
  Save All 双向互斥，避免两个 UI 路径争用 `SaveDirtyViewsJobAdapter` 的单一完成队列；主窗口与浮动窗口
  均会在 Save All 未完成时保持可见。新增 retained-host 回归覆盖异步 Save All 与 close-prompt ownership
  guard。受影响 Rust 文件已执行 `rustfmt --check` 与 scoped `git diff --check`；受管 Cargo 仍因外部未登记
  D/E/F target 被 coordinator 拒绝，未将本条写作运行时通过或 M3 accepted。场景尚未成为
  `DocumentToolkit`，故旧 `Save Project` 的场景保存 hard cut 仍是后续工作项。
