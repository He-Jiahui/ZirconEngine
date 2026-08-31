# Editor03/11 Transaction Journal Unified Version Shell Hard Cut

状态：`implementation_complete / review_p1_repair_complete / typed_tail_read_reason_complete / independent_re_review_approved / isolated_actual_source_gate_7_of_7 / product_gate_blocked_by_runtime_interface01`

## 范围与当前拓扑复审

本切片只收口 durable journal frame 内的事务 record 协议。当前源码已将旧 `engine/journal.rs` 拆为 folder-backed `engine/journal/`；`DurableJournal`、codec registry/replayer、`DocumentJournalCoordinator` 和 UI asset session journal 是不同 owner，不能按旧计划清单再造一套 journal。

三层版本职责如下：

| 层 | owner | 合同 |
| --- | --- | --- |
| durable container | `engine/journal/durable/{format,reader,writer}.rs` | `ZRJNL001` + format v2、sequence/length/BLAKE3/framing/integrity |
| transaction record | `engine/journal/transaction.rs` | `$zircon` schema `zircon.editor.editing.transaction-journal` v1 |
| command payload | `CommandJournalPayload` + `EditCommandCodecRegistry` | `command_type + schema_version(u16)` 选择业务 codec |

事务 record 原先把私有 `schema_version: u16` 放在 raw JSON DTO 内，并由私有 reader 手工校验。该做法与 Plan11 的统一 `VersionedSchema` 冲突，也会把 record schema 与 command codec version 混成同一概念。本切片不改变 durable framing 版本，也不改 command payload 的业务版本。

## 架构裁决与实现

- `TransactionJournal` 删除私有 `schema_version`、`TRANSACTION_JOURNAL_SCHEMA_VERSION`、`TransactionJournalSchemaError` 和旧裸 `serde_json` codec。
- record 实现 `VersionedSchema`；`encode/decode` 统一走 runtime interface 的 `write_versioned/load_versioned` 文本格式。
- v0 migration step 明确拒绝所有无壳 payload；不保留旧 raw reader、alias、fallback 或静默升级。future schema 由共享 `LoadError::FutureVersion` 拒绝。
- payload 解码后仍校验 volatile-history 不变量，但该错误硬切为 `TransactionJournalValidationError`，不再伪装成 schema header 错误。
- `PreparedJournalRecord::prepare` 只调用 `TransactionJournal::encode`，随后执行既有 1 MiB 上限和 BLAKE3；writer 不二次编码，checksum 覆盖带统一壳的最终 bytes。
- `JournalReplayError` 同步改为 `JournalValidation`。旧错误名和常量无兼容重导出。
- `JournalTailFault::InvalidTransaction` 直接拥有 `TransactionJournalReadError`，并作为 `DurableJournalError::UnreadableTail` 的 source；future schema、payload decode 与 invariant failure 不再被压缩为无原因枚举。
- `LoadError` 不可无损 clone，因此 `JournalReadReport` 删除未被消费的 `Clone/PartialEq/Eq` 表面，新增 owner-private `take_tail_fault`。`DurableJournal::open` 与 compaction 只在拒绝坏尾时消费 fault；正常 valid-prefix 查询仍通过借用 accessor 读取，不引入 `Arc` 错误包装或字符串副本。

## 参考引擎结论

`dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Classes/Editor/TransBuffer.h` 以单个 `UndoBuffer` 持有 `FTransaction`、以 `MaxMemory` 约束历史，并让 transactor 自身实现 `Serialize(FArchive&)`。`Private/EditorTransaction.cpp` 的 `FObjectRecord` 由同一 transaction owner 捕获 serialized object/custom change。Zircon 采用相同所有权方向：事务引擎保留唯一 undo record owner，durable journal 是其显式投影；统一 record schema 不创建 UI-local undo store，也不允许 persistence 反向成为 document authority。

## 验证证据

- 测试先改为要求 `$zircon.header.schema_id/schema_version`、future fail-closed 和退役 raw fail-closed；产品 RED 命令因共享 workspace 编译超过 60/120 秒没有进入业务测试，故不把超时记作有效 RED。
- D 盘 `D:\zt\plan11-transaction-journal-shell` 直接包含当前工作区的 `transaction.rs` 与 `prepared.rs`，序列化内核使用 D 盘 current-source 精确副本以绕开无关 runtime UI 漂移。
- `zircon_runtime_interface/src/serialization` 与 `D:\zt\plan11-serialization-kernel\src\serialization` 逐文件 SHA-256 核对为 46/46 文件一致、missing/different/extra 均 0。
- 首轮独立审查为 `NOT APPROVED / 2 Important`：发现 header encode 仍引用已删除错误变体，以及 scene replay corruption test 仍构造 raw payload。整改后 header 改用带 path/source 的 `HeaderEncode`，删除 `error.rs` 对自身 record preparation error 的重复导入；scene replay 改为篡改 `$zircon.payload.commands`。
- 隔离门扩大为直接包含实际 `transaction.rs`、`codec/{error,replay}.rs` 与 `durable/{document_key,error,format,model,prepared,reader,writer,store}.rs`。read-reason TDD 在排除 harness façade 缺线后，RED 精确为 `InvalidTransaction` 缺少 `source`；最终 GREEN 命令 `cargo test --manifest-path D:\zt\plan11-transaction-journal-shell\Cargo.toml --offline` 为 7 passed / 0 failed / 0 ignored，tests 0.23 s。新增 future schema、payload corruption 的具体 `LoadError` 保留，以及 future tail 再次 open/compact 时 source-preserving refusal；标准 `Error::source()` 链非空。
- scoped `rustfmt` 与 `git diff --check` 已执行；旧 `TransactionJournalSchemaError`、`TRANSACTION_JOURNAL_SCHEMA_VERSION`、`JournalSchema`、`validate_schema()` 静态命中为 0。
- `cargo check -p zircon_editor --lib --offline --target-dir D:\zt\plan11-journal-product-target --message-format short` 最新复跑约 11.9 秒返回 exit 1；编译仍在进入 editor 前停止于 `zircon_runtime_host/src/foreign_output/item_count.rs:80` 的 E0004，`WorldQueryResult::TransformSnapshot` 未覆盖。该问题已有 [RuntimeInterface01 open failure](../../../optimize/zircon_runtime_interface/01/failure-2026-08-27-world-query-transform-snapshot-item-count.md)，本切片不吸收 foreign owner。

## 性能边界

本切片没有修改 `journal_transaction` 的 engine mutex 范围、append gate、`sync_data`、compaction、job admission 或恢复算法，因此不声明吞吐、功耗、锁等待或跨引擎性能改善。已存在的 [`Editor03 M4 journal lock-domain performance review`](../03/2026-08-23-journal-lock-domain-performance-review.md) 继续作为优化准入：必须先完成 64 B/1 MiB/256 MiB、command 1/128、selection 1/10k 的 Windows profiler 基线，再评审 immutable commit capture 和锁外物化。

## 未完成项

- transaction commit 线性化点尚未生成 immutable prepared bytes；`DocumentJournalCoordinator` 只有 `#[cfg(test)] append_for_test`，没有 production append API。
- save/close drain、autosave checkpoint、startup discovery 到 restore executor 的接线未完成。
- durable reader 的 typed reason/source 保留已完成；Plan17 仍需在启动恢复候选与 UI 接线时消费该 source，不得把不同原因重新折叠成一条字符串诊断。
- UI asset editor 的 replay records 是另一持久化面，不能用本切片冒充已收口。
- M4.2 完整 editor 行为门、production command payload 覆盖、性能/功耗资格仍开放。
- 两个 journal 切片的独立复审均已通过；产品门仍被外部 RuntimeInterface01 编译失败阻断，因此不提交 milestone commit、不同步协调器、不发送企微。

## 产出记录与时间

| 时间 | 完成项目 | 状态 | 证据与后续 |
| --- | --- | --- | --- |
| 2026-08-28 23:35 +08:00 | journal 当前拓扑、三层版本与 UE transaction owner 复审 | `research_complete` | 确认 durable container、transaction record、command codec 三层职责；排除重复 journal、UI-local undo owner 和未测锁域优化。 |
| 2026-08-28 23:35 +08:00 | TransactionJournal 统一版本壳硬切 | `implementation_complete` | 删除私有版本字段/常量/reader/error 名；新增 `zircon.editor.editing.transaction-journal` v1，raw/future 及 current shell 内退役字段均 fail closed；prepared digest 覆盖最终版本化 bytes。 |
| 2026-08-28 23:48 +08:00 | 独立审查 P1 整改 | `2_important_repaired / independent_re_review_approved` | 恢复 typed header encode source、删除重复自导入、scene replay corruption 改走 current envelope；扩大门覆盖 durable format/error 与 replay before-begin。复审结论 `APPROVED`、0 Critical / 0 Important；当时登记的 reader reason 压缩已由 2026-08-29 后续切片闭合。 |
| 2026-08-28 23:48 +08:00 | D 盘实际源码联合合同 | `5_passed / 0_failed / 0_ignored` | schema/legacy/prepared/header/replay 联合测试 0.18 s；无 C 盘产物。 |
| 2026-08-29 00:21 +08:00 | durable tail typed read reason | `implementation_complete / isolated_actual_source_gate_7_of_7 / independent_review_approved` | `InvalidTransaction` 直接保留 `TransactionJournalReadError` source；实际 reader/writer/store 联合门覆盖 future/payload 两类原因与坏尾 open/compact refusal。RED 精确命中 source 缺失，最终 GREEN 7/7、0.23 s；独立审查 `APPROVED`、0 Critical / 0 Important，审查提出的 compaction 动态覆盖缺口已补齐。产品门仍在 editor 前被同一 RuntimeHost E0004 阻断。 |
| 2026-08-28 23:35 +08:00 | 产品门、性能门与里程碑发布 | `product_gate_blocked_by_runtime_interface01 / performance_not_claimed / commit_wecom_deferred` | 产品 check 缓存复跑 14.5 s 后在 editor 前被 RuntimeHost `TransformSnapshot` E0004 阻断；独立复审已通过，但完整 editor、production append/recovery 与 profiler/功耗仍未完成，不提交、不发送企微。 |
