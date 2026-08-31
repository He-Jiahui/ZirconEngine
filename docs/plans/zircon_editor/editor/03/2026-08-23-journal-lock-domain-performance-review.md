# Editor03 M4 Journal Lock-Domain Performance Review

## Scope

This review covers only the explicit transaction-journal export path.  It is not
an accepted optimization: the current checkout has no production caller of
`EditorTransactionEngine::journal_transaction`, and no managed benchmark has
run.  It records the implementation preconditions required before Plan 11 wires
journal persistence.

## Current Source Evidence

| Boundary | Current behavior | Consequence |
| --- | --- | --- |
| `engine/transaction/replay.rs::journal_transaction` | Acquires the exclusive operation owner, then the `EngineState` mutex, then calls `HistoryStore::journal`. | The operation is isolated, but the state mutex remains held for the whole projection. |
| `engine/history.rs::HistoryStore::journal` | Finds the completed `TransactionRecord` in the bounded `VecDeque` and delegates to `TransactionRecord::journal`. | The record is borrowed from mutable engine-owned history; it cannot outlive the mutex without an ownership change. |
| `engine/journal.rs::TransactionJournal::from_record` | Calls each dynamic `EditCommand::journal_payload`, builds command `serde_json::Value`s, clones label/participants, and projects both selections to owned journal values. | Payload traversal, allocation, and command-defined serialization execute in the state-lock critical section. |
| `engine/transaction/operation_gate.rs` | A single `operation` owner rejects concurrent engine operations with `EngineBusy`. | It can protect an explicit journal-materialization lease, but does not by itself make a borrowed history record valid after unlock. |
| `engine/command.rs::EditCommand` | Commands are `Box<dyn EditCommand + Send>` and journal projection is a dynamic `&self` method. | A naive `Arc` handle is not available: the trait is not `Sync`, and apply/revert still require exclusive mutable access. |

The current performance review reaches the same result: the path is outside
commit/undo/redo and has no production caller, but must not be wired to
persistence with journal projection under the engine mutex.  The format contract
is already covered by `transaction_journal_round_trips_typed_metadata_and_command_payload`,
unknown-schema rejection, unsupported-command context, and scene-command payload
tests.  No duplicate format test is needed for this review.

## Reference Alignment

Fyrox keeps commands as reversible, stack-owned objects with explicit
`execute`, reverse-order `revert`, and terminal `finalize` lifecycle methods in
`dev/Fyrox/editor/src/command/mod.rs`.  Zircon must retain the equivalent
single history owner and command lifecycle.  Journal persistence is a projection
of a completed record, not a replacement undo stack and not a UI-local cache.

This also agrees with the existing `Performance01` review: preserve immutable
selection handles in the editing hot path; do not restore JSON or a second
selection cache merely to make journaling convenient.

## Design Decision Before Implementation

No lock-domain optimization is implemented in this change.  The following
shortcuts are invalid:

- Unlocking while retaining `&TransactionRecord` would make the borrow invalid
  if the history changes or is evicted.
- Cloning the transaction record to serialize outside the lock is unavailable
  for move-only command state and would copy the exact wide reflected payloads
  the `64 B / 1 MiB / 256 MiB` gate is intended to measure.
- Moving command JSON generation into commit would merely move unbounded
  serialization onto the normal edit path.

The candidate design for a later, separately reviewed implementation is an
explicit `JournalMaterializationLease`:

1. Under the operation owner and state mutex, locate one completed transaction
   and move it into a stable in-flight slot together with its history id,
   generation, and original position.  The history must retain an explicit
   in-flight marker so undo/redo/save/eviction cannot observe a shortened or
   reordered history.
2. Release the state mutex while retaining the operation owner.  Build the
   journal from the moved record, including dynamic command payload projection.
3. Reacquire the mutex and restore the exact record before clearing the
   operation owner, on both success and typed projection failure.  A recovery
   failure must fault the engine rather than lose a history record.

This needs a focused transaction/history design review before code: it adds an
in-flight record state, restore-on-error behavior, generation/dirty/save-token
rules, and eviction behavior.  It must be tested against undo/redo/save
ordering; it must not be smuggled in as a simple `Arc` or deep-copy refactor.

## Measurement Gate

The hypothesis is not currently a measured bottleneck.  Before any such code is
accepted, use the managed Windows Cargo lane to capture a baseline and candidate
at values `64 B`, `1 MiB`, and `256 MiB`, commands `1` and `128`, and selection
sizes `1` and `10,000`.  Record at minimum:

- engine state-mutex acquire wait and hold p50/p95;
- journal projection wall time, command count, selection items, and output bytes;
- JSON traversal/copy bytes and peak RSS;
- operation-gate contention and preservation of undo/redo/rollback/save order.

The candidate is acceptable only if journal payload serialization is outside the
state mutex, no normal commit/undo/redo path gains serialization work, and all
existing lifecycle semantics remain unchanged.  The current coordinator CPU
reservation prevents managed Cargo from starting; no timing, power, or
post-optimization claim is made here.

## Durable Capture Architecture Decision (2026-08-24)

The current source has two independent costs inside `JournalWriter::append`:
the conversion of a `TransactionJournal` into JSON bytes, and durable framing
(`sequence`, length, BLAKE3 digest, append, and `sync_data`).  Treating these as
one writer operation prevents a later bounded job from owning its input bytes
without also inheriting dynamic command projection or re-encoding work.

The first implementation cut therefore introduces one immutable,
size-validated `PreparedJournalRecord` boundary:

```text
completed TransactionJournal
  -> prepare JSON bytes + BLAKE3 digest + record-size validation
  -> PreparedJournalRecord (owned bytes, exact digest, transaction identity)
  -> per-document ordered writer
  -> sequence/framing/file-size validation/write/sync
```

This is deliberately narrower than the final commit capture path.  The current
scene `CreateNodeCommand` only learns its record during `apply`; generic
`EditCommand` is `Send` but not `Sync`; and `UpdateNodeCommand` can mutate its
retained `after` state through merge.  A job must therefore never borrow a
live `TransactionRecord`, and the new record type is not exposed as a way to
send dynamic commands across threads.  It is a transport for bytes already
made immutable by a future materialization owner.

The selected writer hard cut has these rules:

1. `JournalWriter` accepts only `PreparedJournalRecord`; it no longer accepts a
   mutable journal object and performs JSON conversion itself.
2. Preparation validates the same `MAX_RECORD_BYTES` ceiling before work is
   admitted.  The writer still validates file-size and sequence limits under
   its per-document lock.
3. The digest is computed once during preparation and written verbatim.  The
   reader remains the sole checksum verifier, so no separate checksum protocol
   is introduced.
4. This change does not wire `append_committed` to the UI, transaction event
   bus, autosave worker, or job system.  Those paths still lack an
   `EditCommand`-safe immutable snapshot and save/close drain contract.

Unreal's `UTransBuffer` establishes the relevant ownership precedent: it keeps
one transaction buffer, tracks memory limits, and removes redo/old records
under that owner.  Its `FTransaction` lifecycle is not an invitation to detach
mutable command state into a worker.  Godot likewise assigns a history before
`commit_action` and preserves per-history saved-version state.  Zircon follows
those constraints by retaining a single history owner and making persistence a
post-materialization byte stream, not a second undo store.

## Commit-to-Durable Append Current-Topology Re-review (2026-08-29)

The current transaction lifecycle, journal coordinator, and job system were reread as one
module boundary before attempting the next implementation step. The current production flow is:

```text
TransactionScope::commit
  -> commit_root_transaction
  -> publish completed TransactionRecord into HistoryStore
  -> emit transaction event

explicit later query
  -> journal_transaction(history, transaction)
  -> find the completed record again
  -> materialize TransactionJournal under the engine operation/state owner
```

There is no production call from the commit linearization point to
`DocumentJournalCoordinator`. Its direct append helper remains test-only. Consequently the
writer hard cut has produced the correct immutable byte boundary, but no durability owner yet
guarantees that every accepted commit reaches it exactly once.

`EditorJobSystem` is not that owner in its current form. A mutex group serializes jobs that are
actually dispatched for one key, but ordinary shutdown can cancel pending jobs and does not
provide a journal-specific acceptance receipt, per-document durable sequence, or save/close drain
barrier. Reusing the generic job lane would therefore allow an accepted transaction event to
outlive or lose its pending append during close/shutdown. It would also conflate progress/cancel
semantics with a write-ahead durability obligation.

The next implementation remains deferred. A valid design must first prove all three boundaries:

1. The transaction engine creates or leases exactly one immutable `PreparedJournalRecord` for the
   committed record without serializing an unbounded payload on the normal commit hot path and
   without borrowing move-only command state across the engine lock.
2. A journal-owned, byte-bounded per-document FIFO accepts that record with an explicit durable
   receipt and typed rejection/incident semantics. Generic `EditorJobSystem` cancellation cannot
   be the only retention contract.
3. document save, project close, runtime shutdown, and crash recovery share one drain/checkpoint
   protocol, so a commit cannot be reported durable before append+sync and pending accepted records
   cannot be silently dropped.

No code or performance claim follows from this review. The existing `64 B / 1 MiB / 256 MiB`,
`1 / 128` command, and `1 / 10,000` selection matrix remains mandatory before lock-domain or
materialization optimization. The current Windows product check is also stopped before
`zircon_editor` by the registered RuntimeHost `WorldQueryResult::TransformSnapshot` exhaustiveness
failure. Other MVP infrastructure work continues while those measurement and product gates are
unavailable.

## 产出记录与时间

| 时间 | 完成项目 | 状态 | 证据与后续 |
| --- | --- | --- | --- |
| 2026-08-23 | M4 journal 锁域与所有权复核 | `research_complete / implementation_deferred / measurement_pending` | 已完成 `replay.rs`、`history.rs`、`journal.rs`、operation gate、command trait 与 Fyrox command lifecycle 的源码对照。确认当前风险是未接线的持久化前置条件，而非已测得的编辑器帧瓶颈。 |
| 2026-08-23 | 优化方案准入 | `design_gate_defined` | 定义了需先审查的 in-flight `JournalMaterializationLease`，以及 64 B/1 MiB/256 MiB、1/128 command、1/10k selection 的受管度量门。禁止借用越锁、深拷贝 record 或把序列化迁入 commit。 |
| 2026-08-23 | 动态验证与性能数据 | `blocked_by_coordinator_reservation` | 受管 Cargo CPU lane 仍由 `plugins09-particles-neutral-identity-runtime-r3-20260823` 的 reservation `3bba2c9f4999490aacce9ef80df1f9fd` 占用。未启动 Cargo，未产生性能、功耗或优化后对比数据。 |
| 2026-08-24 21:55 +08:00 | 强制 flush 微基线 | `measured / not_engine_acceptance` | E 盘工作区的 .NET `FileStream.Flush(true)` 对 1 KiB append 采集 200 次：p50 `0.5926 ms`、p95 `0.8938 ms`、p99 `2.1235 ms`、最大 `21.4729 ms`、总计 `150.1861 ms`。该采样不含 Zircon 的 JSON 编码、`metadata()` 或 engine 锁，不能外推为完整 editor 或 Unreal 对比；但最大值已超过 60 Hz 帧预算，确认每事务 `sync_data` 不得直接放入 UI 帧。受管 Cargo/WPR 的 64 B/1 MiB/256 MiB、锁等待、RSS、功耗与候选设计对比仍为准入门，尚未开始 writer/engine 优化。 |
| 2026-08-24 22:07 +08:00 | 提交捕获/后台写入结构复审 | `research_complete / implementation_deferred` | 审读 `AutosaveJobAdapter`、`AutosaveWriteJob`、`EditorJobSystem` admission/mutex group 与 transaction operation gate 后，排除两条错误路径：autosave 的 worker-time mutable capture 会在 history eviction 后失去 exact transaction；`TransactionMessage` bus 可背压/拒绝，不能作 durability owner。后续唯一候选是 engine 侧产生有界 immutable `PreparedJournalRecord { document, transaction, encoded_bytes, checksum }`，按实际 bytes 通过 job admission 入队，以 stable document key mutex group 串行写入；writer 接受预编码帧，避免二次 JSON encode。该设计仍需解决 `EditCommand` 非 `Sync` 下的安全 materialization、history pin/eviction、save/close drain、typed incident 和 64 B/1 MiB/256 MiB WPR gate；在这些 owner/语义评审完成前，不把 `append_committed` 接到 UI、message bus 或 autosave worker。 |
| 2026-08-24 22:39 +08:00 | durable capture/writer 边界裁决 | `design_complete / implementation_in_progress` | 复核 current source 的 scene command `apply` 后状态、merge 可变性、`EditCommand: Send` 边界、`JournalWriter` 的 JSON+framing 复合职责，以及 Unreal `ITransaction`/`UTransBuffer` 和 Godot history routing。决定先硬切 writer 至 `PreparedJournalRecord`，保持动态 command snapshot、job admission、close drain、WPR 与能耗资格为后续独立 owner；未把未测模型或微基线表述为端到端性能结论。 |
| 2026-08-24 23:06 +08:00 | `PreparedJournalRecord` writer hard cut 与独立审查修复 | `implementation_complete / review_repair_complete / validation_blocked_by_foreign_runtime90` | `PreparedJournalRecord` 现唯一拥有已验证 JSON bytes、BLAKE3 digest 和 transaction identity；`JournalWriter` 仅接收该不可变记录，compaction/coordinator 与 durable regressions 同步迁移，静态 guard 证明 writer 不再引用 `TransactionJournal` 或 `serde_json`。独立审查发现的两项基础设施缺口已硬切：compaction 通过 Runtime `atomic_write` 取得 Windows `ReplaceFileW` 语义，避免 `std::fs::rename` 不能覆盖既有目标；同文档 append gate 串行测试 bridge 的 append/read/compact/unbind，旧 writer 关闭后才允许同 `DocumentId` 重绑。复审进一步确认 engine commit operation 尚未释放时不能调用 `journal_transaction`，因此不保留虚假的 production callback API：`append_from_commit_callback` 已删除，coordinator 的 append 仅在 lib test 编译；真正的 production publication 仍等待 engine 在 commit 线性化点生成 immutable capture，再由 pinned snapshot + save/close drain owner 接入。该 compaction 复制仅发生于有 `MAX_JOURNAL_BYTES` 上限的维护路径，正常 append 仍是 streaming。`rustfmt`、scoped diff check 和针对性 static guard 通过。D 盘受管 target 的 `cargo test -p zircon_editor --lib document_journal_coordinator --locked` 仍在编译 editor 前停止于 `zr_rhi_wgpu` 的 14 个外部错误；[`Runtime90 handoff`](../../../optimize/zircon_runtime/90/failure-2026-08-24-rhi-wgpu-diagnostics-current-source-compile-blocker.md) 已拥有 diagnostics 子集，剩余 RHI WGPU closure 需由 Runtime90 扩展并回传。未获得 Editor Cargo/test、WPR、功耗或跨引擎性能资格，不提交、不发送企微。 |
| 2026-08-29 | commit -> durable append current-topology 复审 | `architecture_re_review_complete / implementation_deferred_by_measurement_gate / alternative_mvp_work_continues` | 逐段复读 commit lifecycle/scope/replay、history state、`DocumentJournalCoordinator` 与 `EditorJobSystem` admission/mutex-group/pending/shutdown。确认 commit 目前只发布 memory history + event，journal 是后续二次查询；coordinator 无 production append，普通 job shutdown 可取消 pending，不能承担 exactly-once durable FIFO/drain。后续必须先定 immutable capture lease、journal-owned byte-bounded admission/durable receipt、save/close/shutdown drain 三个边界，并完成 64 B/1 MiB/256 MiB 受管测量；本轮不写优化代码、不声称性能或功耗收益，继续其它可落地 MVP 基础设施。 |
