---
handoff_kind: failure
status: open
created_at: 2026-08-01
summary_slug: interactive-save-batch-admission-lane
origin_plan: docs/plans/zircon_editor/editor/06-ui-extension-framework.md
fixing_plan: docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
origin_child_dir: docs/plans/zircon_editor/editor/06
fixing_child_dir: docs/plans/zircon_editor/editor/14
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/asset/dirty/save_batch.rs
  - zircon_editor/src/core/asset/dirty/save_job_adapter.rs
  - zircon_editor/src/core/jobs/system/mod.rs
  - zircon_editor/src/core/jobs/system/pending.rs
tests:
  - python -B -m unittest tools.tests.test_editor14_interactive_save_job_adapter_contract -v
  - cargo test -p zircon_editor --lib interactive_save_ --locked --jobs 1 -- --test-threads=1
---

# Editor14：缺少交互式文档保存批次的有界 admission lane

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/06-ui-extension-framework.md`
- 来源执行切片：Editor06 DocumentToolkit failure 的 save-all / close-prompt 上行验收
- 修复责任计划：`docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md`
- 交接原因：Editor06 只拥有 toolkit 写入 hook，Editor09 拥有 dirty generation 与批次语义；实际 I/O
  admission、资源互斥、取消、进度、完成回流和 shutdown 收口属于 Editor14 唯一 `EditorJobSystem`。

## 失败现象与复现证据

`docs/plans/zircon_editor/editor/09-editor-asset-management.md` 的 2026-07-31 PERF-MVP-602 约束要求
`SaveDirtyViewsRequest/Result` 先完成全批 typed preflight，再提交一次 canonical save batch，且 UI 线程的
serialize/fs/import 必须为零。Editor14 同日计划记录要求显式有界 interactive lane，包含 entry、estimated
bytes、age reservation、per-resource mutex group、cooperative cancel 与有界 completion apply。

原始交接时缺少 `InteractiveSave` 类别与有限默认值；该类别和配额已经由 Editor14 后续 quota 切片补齐。
当前剩余最低层缺口是 canonical `SaveDirtyViewsRequest` 没有生产 adapter 将整批轻量 intent 原子提交到唯一
`EditorJobSystem`，也没有在构造 save mutex、job payload 与 result channel 前检查共享 entry/estimated-byte/
oldest-age admission。若 close prompt 直接逐 view 调 toolkit，仍会在 retained callback 同步执行 serialize、文件
写入、import 与 workspace refresh；若自行建队列，又会形成第二 job owner。两者都不能满足既定架构。

## 最低共享层根因

Editor14 的统一 job 门面已具备交互式保存专用类别和有限 category quota，但尚未提供共享 pending admission
window 与将轻量 `document + dirty generation + estimated bytes` intent 转为执行期 toolkit payload 的 adapter。
Editor06/09 不能在 toolkit、retained host 或领域 editor 中自行建立队列、worker 或 parallel save-all owner。

## 架构修复验收

- Editor14 在唯一 `EditorJobSystem` 中建立 typed interactive save 类别/adapter，并提供 entries、estimated
  bytes、oldest age 的硬上限；不得落入默认无限 `Misc`。
- admission 队列只持有轻量 document/generation intent；serialize payload 只在 ticket 获准执行后，从唯一
  DocumentToolkit/transaction owner 取得，取消或 supersede 必须发生在 payload 构建前。
- 每个文档使用稳定 resource mutex group，与 autosave/source save 共用互斥 owner；同文档写入不得重叠，
  不同文档可在显式预算内并行。
- worker 不捕获 `UiHostWindow`、retained host borrow、session mutex 或可变 UI 状态；完成结果通过有界回流，
  Editor09 仅在 dirty generation 匹配时 compare-and-mark。
- 覆盖 submit 拒绝、partial failure、cancel、stale generation、1/100/10k 文档、1KiB/1GiB payload、
  1/16 writers、stall 0/10ms/2s 与 shutdown deadline；记录 queue entries/bytes/age、payload owners/RSS、
  mutex wait 和 terminal latency。
- lower-layer gate 通过后，Editor06/09 重跑 DocumentToolkit save-all 与 close-prompt 矩阵：retry 只重提失败
  或新 generation 项，全部成功且 generation 匹配才允许 close commit。

## 禁止临时方案

- 禁止用 `Misc + Interactive` 的默认无限配额冒充有界保存 lane。
- 禁止在 UI callback、toolkit、asset/animation editor 中逐 view 同步写盘或创建第二个 save-all 循环。
- 禁止 admission 前序列化全部 payload、缓存第二份 dirty 状态、无条件 mark clean 或跨 generation 提交 close。
- 禁止用 test-only executor、全局禁止编辑或缩小规模矩阵掩盖 backpressure 和 partial failure。

## 修复结果与回传

Open state: `Editor14 最低层 adapter、atomic batch admission reservation、resource mutex 复用、bounded
completion 与 shutdown 合同已实现。M4 前 source iteration 的首轮独立复审 1/3/2 已前向修复，历史静态
复验记录仍保留；原 47-path hard-cut input inventory 因需要外部 dirty autosave fixture 与 Settings API 而
不具备 source-bound Cargo 闭包，不能继续作为 successor。受管 current-source Cargo、规模/WPR、Editor06/09
上行验收以及基于 owner-complete source 的独立二次复审均仍 pending。完成这些 gate 和受管提交后按 lifecycle
key 回传 fixed，不能以静态结果提前关闭。`

2026-08-10 current-source correction: `SaveDirtyViewsJobAdapter::schedule` first reads the optimistic
`pending_admission_window`, then resolves every save mutex and calls `executor_factory`, and only after that
calls the authoritative `submit_batch`. A concurrent admission can consume the observed remaining capacity in
that gap, so a rejected final submission has already materialized request-specific execution resources. This
does not satisfy the pre-materialization contract and must not be counted as a fixed adapter. Editor14 must
provide an atomic batch admission reservation or deferred-materialization submission path: reserve the complete
entry/byte request under the pending-state transaction, materialize mutex/executor/job/channel resources only
after that reservation succeeds, then commit the batch or roll the reservation back on save-mutex materialization
or commit validation failure. The
adapter must not add its own queue, reservation truth, or fallback. Add a race regression that forces capacity
loss after the advisory window and proves the executor factory is not invoked unless the atomic reservation
succeeds.

2026-08-10 forward repair complete: `EditorJobSystem::reserve_batch_admission` now reserves the complete
entry/byte set under the pending-state transaction before the adapter resolves save mutexes, constructs the
executor, or creates request-specific job payloads. `EditorJobBatchAdmissionReservation::commit` creates its
transient channel/task wrappers only after that reservation, then verifies the exact request/spec contract inside
the state transaction before registering or enqueueing executable pending jobs. A mismatch drops those transient
wrappers, and reservation `Drop` releases every uncommitted claim. Adapter regressions cover concurrent capacity
loss, mutex resolution failure, and byte-full rejection release; the commit-mismatch release regression belongs
to the shared JobSystem reservation unit tests. A subsequent `PendingAdmissionLedger` extraction keeps reservation
and category accounting isolated from dependency/ready-queue promotion without changing those semantics.

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 证据 |
|---|---|---|---|
| 2026-08-01 | `open_handoff_recorded` | 完成当前源码与 Editor09/14 PERF-MVP-602 对账，识别 interactive save 的 bounded admission、resource mutex、cancel/completion/shutdown 归属 Editor14。 | 当时 `JobCategory` 尚无 interactive save 类别，非 Thumbnail/Export 默认无界；未增加同步 fallback、第二 job owner 或兼容 API。 |
| 2026-08-10 | `implementation_complete_static_validation_pending` | 新增 shared admission window 和 canonical `SaveDirtyViewsJobAdapter`，以轻量 intent、共享 executor、foreground/autosave mutex 和有界 completion pump 接入统一 job owner。 | Python static contract、scoped rustfmt/diff-check 与独立复审证据在里程碑记录维护；failure 保持 `open`，未将 Editor06/09 上行 wiring 计为最低层完成。 |
| 2026-08-10 | `second_review_findings_forward_fixed_revalidation_pending` | 前向处理精确候选闭包、terminal completion O(1) ownership handoff、`Failed/Cancelled` 生命周期映射、byte-full current/requested 回归和 observer 构造入口文档。 | 独立复审 `Critical/Important/Minor = 1/3/2`；旧 snapshot/ticket 不作为修复后验收，等待 successor snapshot、静态复验和独立 re-review。 |
| 2026-08-10 | `second_review_forward_fix_source_bound_validation_queued` | post-review source 已封存并提交 focused `interactive_save_` 受管验证。 | snapshot `1587`，receipt `9824a91b2b6b498294263d1eb7359784`；仅为 queued receipt，不代表 terminal Cargo/WPR/Editor06/09 上行证据，failure 继续 `open`。 |
| 2026-08-10 | `open / atomic-admission-materialization-gap-recorded` | 发现 `pending_admission_window` 只是 advisory preflight：`save_mutex_for` 与 `executor_factory` 在最终 `submit_batch` 原子 admission 前执行，竞争提交可使拒绝发生在执行资源已物化之后。 | 修复限定为 Editor14 JobSystem 的 atomic batch reservation/deferred materialization contract，并要求 race regression；未以 adapter 私有队列、同步 fallback 或未验证的 code patch 冒充修复。 |
| 2026-08-10 | `implementation_complete_static_validation_pending` | 完成原子 batch admission reservation：预留成功后才物化 save mutex/executor/job payload，commit 精确匹配 spec，save-mutex materialization、commit mismatch 与 shutdown 均释放 claim；将准入账本从 executable ready/dependency 队列中提取为 `PendingAdmissionLedger`。 | snapshot `1607`（InteractiveSave exact source manifest）与 snapshot `1608`（ledger extraction）；Python static contract `3/3`、scoped `rustfmt --check`/`git diff --check` 通过。M4 前两轮历史独立复审为 `Critical/Important/Minor = 0/0/0`；受管 Cargo、规模/WPR、Editor06/09 上行验收及 47-path successor 独立复审仍未完成，failure 保持 open。 |
| 2026-08-11 | `open / m4-hardcut-static-contract-forward-fixed` | M4 将 admission-window 定义与 submission 行为从 `system/mod.rs` 硬切至叶模块后，旧静态合同仍要求根模块定义符号而失败。已前向更新合同：根只验证精选 re-export，`construction.rs`/`submission.rs` 分别验证真实定义与行为，并拒绝根模块重新承载实现；另将 composite 内两个 asset-save 格式漂移机械收敛。 | `python -B -m unittest tools.tests.test_editor14_interactive_save_job_adapter_contract -v` 从 2/3 + 1 failed 转为 3/3；47-path Rust `rustfmt --check` 通过，diff-check 只有 LF/CRLF 提示。原 16-path snapshot 不再作为 current-source acceptance；后续仅使用 47-path M1-M4 composite successor，failure 仍 `open`，不产生 fixed return。 |
| 2026-08-12 | `open / review-record-consistency-forward-fixed` | 当前 47-path 独立复审发现 failure 把 M4 前历史复审误写为当前 successor 的已完成二审，且把不可失败的 `executor_factory` 误列为可失败释放路径。已收紧为历史证据，并把回滚条件限定为 save-mutex materialization 或 commit validation；没有改动生产实现或验收门。 | 47-path manifest 明确 `independent second review` 仍 pending；`SaveDirtyViewsJobAdapter::schedule` 的 `executor_factory` 返回 `Arc<dyn SaveDirtyViewExecutor>` 而非 `Result`。本记录变更后需重新复核 manifest 哈希和当前独立审查，受管 Cargo、规模/WPR、Editor06/09 上行验收及 fixed return 继续保持未完成。 |
| 2026-08-12 | `open / source-manifest-foreign-overlay-recorded` | 独立清单审计确认 47-path 表未纳入两个非本切片 dirty 前置：`save_batch/tests.rs` 为 autosave trait fixture，`quota_settings_contract.rs` 所消费的 Settings startup/defaults API 也在另一 owner 的未提交范围。该表不再声明为可受管验证的 exact successor，避免把外部修复混入 InteractiveSave。 | `2026-08-11-interactive-save-m1-m4-current-source-manifest.md` 状态改为 `invalidated_for_source_bound_validation`；仅保留已列 46 个输入的 hash inventory。后续必须等待这些 owner 的提交祖先或通过 coordinator 创建已租约的组合 successor，再进行 Cargo、规模/WPR、上行验收与独立复审；failure 保持 `open`。 |
| 2026-08-13 | `second-review-findings-forward-fixed / re-review-pending` | 当前源码独立二审 `Critical/Important/Minor = 0/2/2`。本轮前向修复本地 I1/M1/M2：用批次预分配 completion slots 与 ticket slot index 取代 `BTreeMap` 热路径，使每个 terminal accumulator write 为 O(1) 且整批 ownership handoff 为 O(1)；Rust 回归不再假定并发 terminal 顺序，并补 `Cancelled` job-bus 证据；同步校正 commit 物化顺序与 mismatch 测试归属记录。 | I2 是 Autosave fixture 与 Settings API 两个外部 owner 未提交造成的 source-bound closure，不由本切片吸收；旧 47-path snapshot 继续失效。等待当前五路径静态复验与独立 re-review；受管 Cargo、规模/WPR、Editor06/09 上行验收及 fixed return 仍未完成，failure 保持 `open`。 |
| 2026-08-13 | `re-review-important-forward-fixed / final-re-review-pending` | 独立 re-review `Critical/Important/Minor = 0/1/0` 发现 shutdown 回归在释放同 mutex blocker 前等待取消 ticket，无法到达新增 bus 断言。本轮把 blocker release/join 移至 `await_batch` 前，使已调度 save job 能运行到取消检查。 | 修复后的用例同时保留 executor 调用数为 0、domain completion 为 `Cancelled` 与对应 save label 的 job-bus `Cancelled` 三层断言；等待静态复验与最终独立 re-review，外部 source-bound closure 和受管产品 gate 状态不变。 |
| 2026-08-13 | `implementation-complete / final-second-review-clean / managed-acceptance-pending` | 最终独立复审 `Critical/Important/Minor = 0/0/0`：固定 completion slots、O(1) accumulator write/ownership handoff、无序 terminal label 断言与可达的 shutdown `Cancelled` 三层证据均闭合。 | Python 静态合同 `3/3`、五路径 scoped `rustfmt --check`/`git diff --check` 通过。Autosave/Settings 外部 source-bound closure、受管 Cargo、规模/WPR 与 Editor06/09 上行验收仍只延迟 accepted closeout；failure 保持 `open`，未提前 return fixed。 |
