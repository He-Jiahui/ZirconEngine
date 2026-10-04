---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: ui-surface-dirty-full-tree-scans
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/02-layout-taffy-and-containers.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/02
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/surface/surface/rebuild.rs
  - zircon_runtime/src/ui/surface/surface/rebuild/incremental.rs
  - zircon_runtime_interface/src/ui/tree/node/ui_tree.rs
  - zircon_runtime/src/ui/tests/surface_dirty_domains.rs
tests:
  - 10k-node one-dirty-leaf visit-count test
  - dirty aggregate domain union test
  - failed rebuild preserves dirty-set retry test
---

# Editor UI 02：UiSurface dirty rebuild 前后全树扫描

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：F4 runtime UI surface rebuild 与 editor retained-host call chain 静态审查
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/02-layout-taffy-and-containers.md`
- 交接原因：dirty aggregate/set 必须与 layout mutation、state flags 和错误重试合同一起设计，不能由 editor host 上层缓存绕过。

## 失败现象与复现证据

`UiSurface::rebuild_dirty` 先用 `dirty_flags()` fold 全部 nodes，再用 `dirty_node_count()` 再扫全部
nodes。即使 incremental layout 只访问一个 dirty subtree，成功后 `clear_dirty_flags()` 仍第三次
遍历全部 nodes。任意 hover/input/render/layout 失效因而有至少 3N 的 metadata scan。

## 最低共享层根因

dirty authority 只分散存储在 node flags；surface 没有 aggregate domain mask、dirty node id set/count
或 rebuild generation。调用端只能每次重新发现全树状态。

## 架构修复验收

- 所有 mutation/state-flag 路径增量维护 surface dirty mask 与 dirty node ids/count；重复标脏不重复计数。
- rebuild 成功后只清登记/访问节点；失败时保留完整 dirty state 供重试，显式 clear 仍有确定语义。
- 10k nodes/1 dirty leaf 的 dirty discovery/clear visit count 接近 dirty set 大小，idle no-dirty 为 O(1)。
- layout/hit/render/input/style/text/visible-range union 与既有 incremental tests 全部保持。

## 禁止临时方案

- 不得删除 dirty diagnostics 或只缓存 `dirty.any` 而让 count/domain 失真。
- 不得把三次扫描移到 worker 后继续与主线程 mutation 竞争同一 tree。

## 修复结果与回传

Open state: `待 Editor UI 02 建立 surface dirty aggregate 与 dirty-node authority`。

### 2026-09-11 滚动修复记录

- 当前源码已在最低共享层建立 `UiTreeNodes` dirty-domain 聚合索引。节点的单点 mutable entry、批量 mutable entry、插入/删除和 state-flag 变更均登记候选 ID；surface rebuild/compute/incremental rebuild 共用候选集合，成功清理只触及登记节点，失败保留 dirty authority 供重试。
- 新增回归覆盖：10,000 节点单叶 discovery/clear 访问计数、跨 domain 合并且同一节点不重复计数、缺失子节点导致 rebuild 失败后保留 dirty 集并成功重试，以及 interface 层 idle 查询不再重扫。
- 源码归属快照：pre-edit `3388`；post-edit `3389`（当前五个实际改动源文件的哈希已冻结）。本记录本身尚未作为新的 return 文件提交。
- 受管静态票据 `d18457f72bef4833a39ae0cbeaca9bed`（job `78c122ec7a3b4ab2a1863049e630b07f`）已实际执行 `rustfmt --check`，终态 `failed`/exit `1`；输出仅包含现有 import 排序、换行和测试格式差异，不能作为 Cargo 或行为通过证据。此前 `rustfmt --emit stdout` 解析检查返回 `0`。
- 受管 Cargo 接口回归请求 `editorui02-dirty-authority-interface-cargo-3389-r2` 与 runtime 回归请求 `editorui02-dirty-authority-runtime-cargo-3389-r1` 均在 admission 阶段被拒绝：`validation_ticket_external_worktree_dirty`，外部仓库为 `E:\Git\zr_vm`。因此原始复现、interface 索引测试和 surface dirty-domain 测试尚无可复用的动态结果；不得回传或关闭 failure。
- 当前状态：`source_repair_recorded / managed_validation_pending / external_dependency_blocked`。待外部仓库提交干净固定 revision 后，需以新鲜源码快照重新提交上述精确 Cargo 测试，并完成独立 Critical/Important/Moderate 全零审查后再生成唯一 canonical return。

### 2026-09-19 rolling source-contract snapshot (EditorUI02 owner retry)

- 本轮协调器 Session `failure-roll-01a084c8-editorui02-dirty-authority-r2` 已从上一份已归档 Session 转移六个精确路径；`tree_node.rs` 及 `surface_dirty_domains/` 下由其他会话归属的拆分测试文件保持排除。转移前后保留了当前工作树哈希，未覆盖其他会话的改动。
- 当前源码仍显示 `UiTreeNodes` dirty-domain index、按节点候选集合、surface rebuild/compute/incremental 共用候选摘要，以及失败重试时保留 dirty authority；根测试文件保留 10,000 节点 discovery/clear、domain union、失败重试和 idle 查询回归断言。
- 本轮静态源契约检查实际执行：关键符号与回归断言均存在；`rustfmt +1.94.1 --edition 2024 --config skip_children=true --check` 退出码为 `1`，仅报告现有 import 排序、单行条件和跨模块格式差异，未将该结果伪报为通过。`rustfmt --emit stdout` 解析检查及精确源码断言可作为静态证据，但不替代受管 Cargo 行为验收。
- 本轮快照哈希（failure doc 哈希随本记录写入后由受管 manifest 冻结）：`surface.rs` `26fa9866478b63be830181ed3f126b946935dbbe86a550c25385adc804650cc7`；`rebuild.rs` `943759cbe4262e31644f99a8254ea872ecfa3eb28d0ff8e5b68221764aeb3c62`；`incremental.rs` `0c9a039ca5506ca5d609de5f58ec7583648ba453a773c588282166889744a0e7`；`ui_tree.rs` `1b612c04f8e2383a431ffdbbe59d1f5720a24a35632124f78c96b97ec83e7330`；`surface_dirty_domains.rs` `246793d4eaf366c92c52b2dcafda57c242ba4293931508f627bb11f6df17c5a6`。
- 状态仍为 `source_repair_recorded / static_parse_evidence / managed_validation_pending`。由于外部 `E:\Git\zr_vm` dirty admission blocker 以及本轮 rustfmt check 非零，不生成 fixed/return，也不关闭 failure；后续必须以新鲜快照执行 managed interface/runtime Cargo 原始复现、10k 性能门槛和向上验收，再进入独立审查。

### 2026-09-19 successor static source-contract receipt

Fixing Session `failure-roll-01a084c8-editorui02-dirty-authority-r2` sealed
ticket `58206656afd94c73b7f1f34d19cf221f`. Coordinator copy job
`a4c59956d7eb466b822270c2d183329b` and run
`58206656afd94c73b7f1f34d19cf221f` exited 0 with
`EDITORUI02_DIRTY_AUTHORITY_SOURCE_PARSE_PASS`. The six-path scope retained
the shared surface, incremental rebuild, tree-index, regression-test and
failure-record owners; the pre-receipt source manifest was
`af403a40112293ba38bdb0104da08ffbaeab8ea5e4092595833900166c0e3eeb`.

This is source-parse evidence only. The non-green rustfmt result remains
recorded, and the managed interface/runtime Cargo behavior tests, 10k-node
performance threshold, external `E:/Git/zr_vm` admission, independent
Critical/Important/Moderate zero-finding review, canonical `failure return`,
and closeout remain pending. The receipt append changes the document hash
after the static snapshot; any dynamic successor must reseal the current file.

## 2026-09-21 independent source review receipt

- Reviewer Session `review-editorui02-dirty-authority-r2` inspected the five
  current source/test paths in the successor manifest without editing them;
  their hashes remain identical to the sealed manifest
  `af403a40112293ba38bdb0104da08ffbaeab8ea5e4092595833900166c0e3eeb`.
- `rustfmt +1.94.1 --edition 2024 --config skip_children=true --emit stdout`
  parsed every path successfully, and the independent source probe passed as
  `EDITORUI02_DIRTY_AUTHORITY_INDEPENDENT_SOURCE_REVIEW_PASS`. It verified the
  shared `UiTreeNodes` dirty index and mutation candidate tracking, surface
  candidate-set aggregation, one-pass dirty summary/clear paths, incremental
  pending-node handling, duplicate-domain union, failed-rebuild retention, and
  the 10,000-node visit-count regression anchors.
- The stricter `rustfmt --check` acceptance command remains **non-zero** on four
  paths (`ui_tree.rs`, `surface.rs`, `rebuild.rs`, and `rebuild/incremental.rs`)
  for import ordering and equivalent formatting differences; scoped
  `git diff --check` passed. This is preserved as an explicit validation repair
  blocker, not promoted to a formatting pass.
- Independent source review result: **Critical=0 / Important=0 / Moderate=0**;
  no foreign tree/surface source was absorbed. Fresh formatting repair,
  managed interface/runtime Cargo behavior, 10k performance, and upward gates
  remain pending (external `E:\Git\zr_vm` is dirty). Canonical `fixed-*` return,
  closeout, and WeCom notification remain pending.

### 2026-09-25 successor current-source reconciliation r3

Fixing Session `failure-roll-01a084c8-editorui02-dirty-authority-r3` sealed
coordinator snapshot `3822` after the related-code manifest correction. The
current five-path source manifest is:

```text
zircon_runtime/src/ui/surface/surface.rs
  26fa9866478b63be830181ed3f126b946935dbbe86a550c25385adc804650cc7
zircon_runtime/src/ui/surface/surface/rebuild.rs
  943759cbe4262e31644f99a8254ea872ecfa3eb28d0ff8e5b68221764aeb3c62
zircon_runtime/src/ui/surface/surface/rebuild/incremental.rs
  0c9a039ca5506ca5d609de5f58ec7583648ba453a773c588282166889744a0e7
zircon_runtime_interface/src/ui/tree/node/ui_tree.rs
  1b612c04f8e2383a431ffdbbe59d1f5720a24a35632124f78c96b97ec83e7330
zircon_runtime/src/ui/tests/surface_dirty_domains.rs
  246793d4eaf366c92c52b2dcafda57c242ba4293931508f627bb11f6df17c5a6
```

The exact source probe passed:
`EDITORUI02_DIRTY_AUTHORITY_CURRENT_SOURCE_PASS 5 paths`. It verifies the
shared `UiTreeDirtyIndex`/candidate authority, single dirty-summary collection,
indexed clear and failed-rebuild retention, incremental dirty-node routing, and
the 10,000-node visit-count/union/retry regression anchors. Scoped
`git diff --check` passed. The Rust 1.94.1 `rustfmt --check` command remains
non-zero only for pre-existing import-order and equivalent formatting drift in
the four production owners (`surface.rs`, `rebuild.rs`, `rebuild/incremental.rs`,
`ui_tree.rs`); the test file is not implicated, and no formatter-only rewrite
was made.

This is current-source/static evidence only. Managed interface/runtime Cargo,
10k-node performance, external `E:/Git/zr_vm` admission, independent review,
canonical `failure return`, closeout, and WeCom remain pending.

### 2026-09-25 independent current-source review r3

Reviewer Session `review-editorui02-dirty-authority-r3` rechecked the five
source paths against coordinator snapshot `3822` without editing or absorbing
foreign changes. All five hashes match the manifest. The review located the
`UiTreeDirtyIndex`/candidate authority, one-summary/indexed-clear paths, failed
rebuild dirty-state retention, and the exact 10,000-node discovery/clear/union
and retry anchors. Scoped `git diff --check` is clean; the four production
owners retain only the documented pre-existing rustfmt import/equivalent
formatting drift, while the regression test is not implicated.

Independent result: **Critical=0 / Important=0 / Moderate=0**. Snapshot `3822`
sealed the source manifest before this receipt; the current failure document
hash `4e24abbc84646eee8b269d83b04cf0fcd1de33068eeaf3c0989467b733d750b2` is a
post-snapshot doc-only change. This remains static evidence; managed Cargo,
10k performance, external `E:/Git/zr_vm` admission, upward gates, canonical
return and closeout remain pending.

### 2026-09-27 Rustfmt 验收阻断修复

- 沿用现行 fixing Session `failure-roll-01a084c8-editorui02-dirty-authority-r3`，保留 baseline 611 / `6b4bc86089cb4464f850136c8079e90cb6513ebc` 和原票据归属。该 Session 没有已接受或待执行 Cargo 请求；r1 的失败格式票据与 r2 的静态通过票据继续作为原始证据。
- 审计转移 fingerprint `6c3da59075ca6471d1280047123d7d5ea0a23508847db806af7ad560bf34bf5e` 接管上述六个精确路径。四份生产文件仅通过 `rustup run 1.94.1 rustfmt --edition 2024 --config skip_children=true` 修正已记录的格式差异；没有新增属性 journal 或其他行为。现有 `surface_dirty_domains.rs` 字节保持不变。
- 实际严格检查 `rustup run 1.94.1 rustfmt --check --edition 2024 --config skip_children=true` 覆盖五个源/测试路径，退出码为 **0**。此前四路径非零 gate 现已修正；这不是 Cargo 或 10k 行为验收通过。
- 独立 preimage→当前源码审查为 **Critical=0 / Important=0 / Moderate=0**，确认仅 import 顺序与等价 `if` 格式变化，profiling `cfg` 附着关系保留；五个源/测试哈希匹配记录。该格式切片审查不替代指定任务的正式 closeout 审查。
- 当前五个源/测试哈希：

```text
zircon_runtime/src/ui/surface/surface.rs
  f9c55c514b4947eb4bed2bcf076c31fbd06d18c5e409f82320f6e2cf6ca6990d
zircon_runtime/src/ui/surface/surface/rebuild.rs
  3404cc90524d490744fa86d91d2865d214f5be6aea6b7d8180d54e9747eb23c4
zircon_runtime/src/ui/surface/surface/rebuild/incremental.rs
  3b99a31074bd7ae7de0d851bdf56cc0d0de36016d02a45bec86cb5594e92a823
zircon_runtime_interface/src/ui/tree/node/ui_tree.rs
  5421b9834c2a62009e48ab6b37a2327b79aa6223a202fc5528b4d2ce0bf5e071
zircon_runtime/src/ui/tests/surface_dirty_domains.rs
  246793d4eaf366c92c52b2dcafda57c242ba4293931508f627bb11f6df17c5a6
```

- 新鲜 interface/runtime 动态票据仍待正常受管外部归档准入。当前共享阻断已交接为 [Coordinator01 归档错误诊断](../../../zircon_tooling/session_coordinator/01/failure-2026-09-27-external-worktree-archive-error-diagnostics.md)，保留其原 owner 与单次任务投递；不绕过冻结、不重复盲提 Cargo 请求。
- 10k 单叶 discovery/clear、domain union、失败 rebuild 重试和 upward acceptance 保持 open；不生成 canonical fixed/return，也不关闭 failure。

### 2026-09-27 related stage input continuation

The same stable Session now carries the related
[arranged index and stage invalidation repair](failure-2026-07-18-runtime-ui-arranged-index-and-stage-invalidation.md).
Its current-source inspection exposed local patch flags that were not forwarded
to frame publication or stage profiling. That lifecycle retains its own original
acceptance requirements and return record. The stage repair preserves indexed
dirty collection/clear, and mounts a separate small regression module.

Snapshot 4183 remains evidence for the preceding format-only slice. Subsequent
changes to `rebuild/incremental.rs` and `surface_dirty_domains.rs` mean its exact
hashes must not be used as passing evidence for the new candidate. Runtime
validation needs a new frozen manifest including the existing invalidation
helper, stage report, interface diagnostics and render-domain test compile fix.
The unchanged interface-only `ui_tree.rs` hash can still support a separately
declared lower gate. No fresh dynamic acceptance or canonical return is claimed.
