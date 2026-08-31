---
title: Editor Document、Transaction、Save、Autosave、Recovery 与 Authoring I/O 当前源码复核
category: zircon_editor
report_id: Editor225
review_date: 2026-08-30
baseline_head: a6cc2b0ca091014944b1a0ce0b48e1a8c3101eb6
verification_head: 189f72219eaf16a6d0db880b53f3f68b4f5ee15a
canonical_owner: Editor02
refreshes:
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
related_reports:
  - docs/plans/optimize/zircon_runtime/160-runtime-filesystem-path-uri-vfs-mount-watch-sandbox-atomic-io-current-source-review.md
  - docs/plans/optimize/zircon_editor/223-editor-project-operations-source-control-provider-workspace-changelist-diff-automation-validation-submission-gate-health-dashboard-current-source-review.md
related_code:
  - zircon_editor/src/core/editing
  - zircon_editor/src/core/document
  - zircon_editor/src/core/asset/dirty
  - zircon_editor/src/core/extension/toolkit
  - zircon_editor/src/core/recovery
  - zircon_editor/src/core/project
  - zircon_editor/src/core/settings
  - zircon_editor/src/ui/host/asset_editor_sessions
  - zircon_editor/src/ui/host/animation_editor_sessions
  - zircon_editor/src/ui/host/editor_document_autosave.rs
  - zircon_editor/src/ui/host/editor_save_batch.rs
  - zircon_editor/src/ui/host/editor_ui_host.rs
  - zircon_editor/src/ui/host/layout_commands.rs
  - zircon_editor/src/ui/retained_host/app/native_window_close
  - zircon_editor/src/ui/retained_host/app/project_close.rs
tests:
  - zircon_editor/src/tests/editing/transaction_engine
  - zircon_editor/src/tests/host/animation_editor.rs
  - zircon_editor/src/tests/host/manager/document_toolkit_lifecycle.rs
  - zircon_editor/src/tests/host/manager/ui_asset_workspace_watcher.rs
  - zircon_editor/src/tests/ui/animation_editor
  - zircon_editor/src/ui/retained_host/app/tests/close_prompt.rs
  - zircon_editor/src/ui/retained_host/app/tests/document_save.rs
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Public/FileHelpers.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/PackageAutoSaver.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Classes/Editor/TransBuffer.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/ITransaction.h
  - dev/godot/editor/editor_undo_redo_manager.h
  - dev/godot/editor/editor_undo_redo_manager.cpp
  - dev/godot/editor/editor_node.cpp
  - dev/Fyrox/editor/src/lib.rs
  - dev/Fyrox/editor/src/menu/file.rs
  - dev/Fyrox/editor/src/command/mod.rs
  - dev/bevy/crates/bevy_asset/src/saver.rs
  - dev/bevy/crates/bevy_asset/src/io/mod.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/AssetDatabaseHelper.cs
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: partial_foundation_product_workflow_incomplete
source_recheck_required: true
---

# Editor225 · Document、Transaction、Save、Autosave、Recovery 与 Authoring I/O

## 1. 结论

当前 Editor 已经修复 Editor02 记录的多条直接数据丢失路径，不能再沿用“保存按钮为空实现、UI asset/animation 直接截断覆盖、工程切换丢失 autosave completion”等旧结论。`File > Close Project`、主窗口退出和 dirty scene/document 已进入 retained-host close plan；Save All 已有产品入口和异步批次；UI asset/animation 权威源保存已收敛到 `DocumentSourceWriteAuthority` 与 Runtime atomic writer；恢复入口能够评估 residual lock、展示 Restore/Discard/Compare 并把副本物化到 `.zircon/recovered`；autosave 能保留 retired project adapter、输出逐文档结构化结果并在正常退出前执行 final autosave。

但这仍不是 Unreal/Godot/Fyrox 级完整文档产品。单 tab 关闭和 layout reset 仍可绕过 dirty decision；显式单文档保存虽然提交 job，却立即在调用线程 `ticket.wait()`；project scene/workspace 与 document batch 仍是两套提交协议；UI asset 保留私有无界 undo 栈；journal 的 durable append 明确只开放给测试，尚未在 transaction commit 线性化点捕获；autosave 的 admission bytes 仍只是 request struct 大小，完整可变文档在 worker 内序列化；Save As、source-control checkout、统一 history byte budget、close 时 history/context 回收和“已持久化但 projection 失败”终态仍不存在。

磁盘写入的局部改进也不能被误称为跨进程 CAS 或已证明 durability。`commit_if_matches()` 在同进程 lease 下先读取并比较 expected bytes，再调用 atomic replace；外部进程仍可在 compare 与 replace 之间改写。authority 将成功标记为 `DurableBestEffort`，但 receipt 没有内容、源身份、hash、sync scope 或 publication generation，调用方又丢弃 receipt。UI asset 在源文件已提交后忽略 `import_asset` 结果，并可能在 refresh/hydrate 阶段失败，因此当前 report 无法准确区分“没有写入”和“已持久化但 Editor projection 失败”。底层 provider、capability root、secure open、TOCTOU 与 non-Windows scene path 是 [Runtime160](../zircon_runtime/160-runtime-filesystem-path-uri-vfs-mount-watch-sandbox-atomic-io-current-source-review.md) 的 owner，本报告只记录其对 Editor 工作流的后果，不重复登记 finding。

本报告刷新 Editor02，不新增唯一 finding。旧 5 项 P0 当前重判为 **1 Open / 1 Partial / 3 Closed**；30 项 P1 为 **13 Open / 9 Partial / 8 Closed**；8 项 P2 为 **4 Open / 2 Partial / 2 Closed**。16 项资格门为 **7 Fail / 8 Partial / 1 Pass**。现阶段没有动态 benchmark、kill/fault、真实窗口、跨进程 external-writer、source-control、10k documents 或长期内存证据，不能声称性能或可靠性优于 Unreal Engine。

## 2. 审查边界与证据

### 2.1 当前源码选择

| 选择集 | files / lines / nonempty / bytes / test attrs / ignored | fingerprint |
|---|---:|---|
| core editing/document/dirty/toolkit/recovery/project/settings owner | **242 / 40,792 / 37,001 / 1,393,204 / 331 / 14** | `8ed3f634537e31c18afd4dc47546e1b1c2ac59b47c276c63cd57cbecd23b0b82` |
| host/session/save/autosave/project integration | **237 / 50,784 / 46,915 / 1,827,845 / 276 / 60** | `ea90d7cfbe294e2a8a0f4ec98fdf82785227baf7e09b6a2aa9f5ef0cfdbaaf7c` |
| retained-host/workbench close/save/startup integration | **15 / 3,652 / 3,481 / 148,776 / 20 / 0** | `ff47d307bf023debbfff455ffed685ee6a20bcb4ebb378cfc80baceda62bf721` |
| focused transaction/host/animation/UI close-save tests | **24 / 5,401 / 4,845 / 183,734 / 116 / 0** | `e25ba4863a44e3eb53f58888470820d60773fb2e257fcc7b38d6e40b16605cac` |
| deduplicated selected union | **518 / 100,629 / 92,242 / 3,553,559 / 743 / 74** | `17c146852954b75a8e131ab89c2343762e5c3e6c4a654395f3e1b5f99f884664` |
| Unreal/Godot/Fyrox/Bevy/Unity Graphics reference selection | **13 / 19,435 / 16,845 / 763,886 / n/a / n/a** | `2c20db524f5215fdb6abf207be9bc81eea6d0e4437681e82ac973b74da1b1271` |

统计使用 UTF-8 物理行、非空行、文件 bytes、Rust 精确 `#[test]`/`#[ignore]` 属性。指纹按相对路径排序，对每个文件取 SHA-256，再对 `path<TAB>file_sha256<LF>` 清单取 SHA-256。工作树包含其他会话的大量 tracked/untracked 修改；本报告审查当前物理树，不回退或接管这些修改，实施前必须按 `source_recheck_required` 重建选择集。

### 2.2 纵向调用链

本轮逐项追踪以下产品链，而不是只扫描类型名：

1. menu/native close/layout close/reset -> close prompt -> dirty generation -> scene save/document batch -> revalidation -> teardown；
2. explicit Save/Save All/Save on Close -> toolkit capture -> job admission -> source lease/compare/atomic publish -> import/refresh/hydrate -> dirty generation commit；
3. scene/UI asset/animation mutation -> core/private history -> DirtyRegistry -> saved token -> workbench dirty projection；
4. transaction commit -> event -> journal encode/durable store/codec replay -> recovery consumer；
5. autosave policy -> document enumeration -> byte estimate -> worker capture -> rotation -> completion/diagnostic -> project switch/shutdown；
6. project admission -> session lease/heartbeat -> residual assessment -> restore decision -> no-overwrite recovered copy -> open/import projection。

### 2.3 动态证据限制

本轮是 review-only，未修改 production/test/Cargo/ABI，也未运行 Cargo、Editor、真实窗口、child process、source-control provider、故障注入、断电/kill、10k document、scale、soak 或 benchmark。743 个 test attribute 与 74 个 ignored 只能证明已有测试意图，不能替代运行结果；所有 Pass 只表示静态产品合同和测试意图均已出现，不等价于动态资格完成。Tooling 按用户要求排除；本轮也没有查询、轮询、等待或实时跟踪协调器。

## 3. 当前可保留基础

- `DirtyRegistry`、generation-bound save token、external effect 与 transaction dirty 已经形成共享脏状态底座，animation session 也改为从 registry 投影，不再维护第二个 `dirty: bool`。
- `SaveDirtyViewsRequest`/job adapter 具备批量 preflight、原子 admission、source mutex、budgeted polling、逐文档终态与 stale-generation 保护；Save All 和 close-save 已有产品调用点。
- `DocumentSourceWriteAuthority` 把 UI asset/animation 的 canonical write 收敛到同一 source identity lease、expected-byte compare 和 atomic publication；local copy 使用 no-replace publication。
- core transaction engine 已具备 context、RAII scope、apply/revert failure、faulted state、generation、saved token、分页 history；animation authoring 已接入 document-scoped history。
- durable journal 已有 framing、BLAKE3 checksum、1 MiB record/64 MiB file/65,536 record 上限、有效前缀读取、坏文件隔离、codec registry 与 replay 基础。
- autosave 已有单飞、公平调度、逐文档 mutex、snapshot rotation、retired adapter、fallback diagnostic、final autosave 和结构化 outcome。
- recovery 已有 residual/live assessment、activation ledger、Restore/Discard/Compare decision、no-overwrite recovered/comparison copy、坏 catalog entry 隔离和 source digest freshness。
- session lease 已有 heartbeat、显式 release 与 takeover admission；异常 Drop 保留 residual 文件，使后续 recovery 能观察 crash，而不是静默删除证据。

这些基础应收敛为唯一 authority，不应再为单个 editor domain 复制第二套 save/dirty/history/recovery 实现。

## 4. P0 当前重判

| ID | 当前状态 | 当前源码证据与差距 |
|---|---|---|
| P0-01 · 单个 dirty tab 关闭直接删除 document/session | **Open** | `layout_commands.rs:128-158` 的 `close_view()` 直接 begin/apply/commit close，未查询 `DirtyRegistry` 或启动 decision；`ResetToDefault` 在 `:53-64` 直接清 toolkit/session。plugin/automation 仍可经该旁路丢失未保存编辑。 |
| P0-02 · File > Close Project 绕过统一 dirty gate | **Closed** | `project_close.rs:160-198` 在任何 teardown 前检查 active save、queued Save All、dirty documents 与 dirty scene generation，建立统一 close prompt；只有无 dirty participant 时才 `commit_project_close()`。 |
| P0-03 · Window close prompt 的 Save 按钮未实现 | **Closed** | native close Save 会先保存 scene，再以 `SaveReason::Close` 调度 document batch，poll completion 并重验 dirty；旧 no-op 已不存在。动态真实窗口/kill 资格仍由 gate 覆盖。 |
| P0-04 · Crash residual lock 让工程永久不可打开，恢复基础不可达 | **Partial** | startup 已能区分 live/residual，Recovery profile 会验证 terminal activation ledger，并提供 Restore/Discard/Compare；`RestoreExecutor` 只写 `.zircon/recovered` 安全副本。仍缺 defer/cancel、自动打开/导入/hydrate、逐项 retry 与 richer inspection。 |
| P0-05 · UI asset/animation 权威保存非原子覆盖原文件 | **Closed** | 两者 canonical save 均使用 `DocumentSourceWriteAuthority::commit_if_matches()`；local copy 使用 atomic no-replace。旧 production `fs::write` 截断路径已移除。跨进程 CAS、provider/secure-open 与 projection terminal 属于后续 P1/Runtime160。 |

## 5. P1 当前重判

| ID | 状态 | 当前源码证据与必须重构内容 |
|---|---|---|
| P1-01 · Core Save All 基础没有产品入口 | **Closed** | menu/event/retained host 已能 queue 和执行 document Save All，并投影逐项结果。 |
| P1-02 · Foreground save 提交 job 后阻塞 caller | **Open** | `editor_ui_host.rs:245-275` submit 后立即 `ticket.wait()`；显式 UI asset/animation Save 仍可阻塞 UI owner。必须改为 operation handle + completion pump。 |
| P1-03 · Close prompt 无不可变 plan/generation revalidation | **Partial** | prompt 已冻结 dirty generations、token/source/preflight，completion 也重验当前 dirty；但捕获的 toolkit generation 未成为 terminal fence，programmatic toolkit mutation/close 仍可能改变 participant set。 |
| P1-04 · Scene/core 与 UI asset 两套 undo authority | **Open** | UI asset 仍使用私有 `UiAssetEditorUndoStack<Vec<SourceEditEntry>>`，与 core transaction/history/journal 分裂。 |
| P1-05 · Undo/Redo 固定路由 Global history | **Partial** | animation focused document 与 scene document 已按 document context 路由；UI asset 仍私有，跨 domain focus/history owner 未统一。 |
| P1-06 · Animation editor 没有 undo/redo | **Closed** | `AnimationAuthoringDocumentStore`、typed mutation command、document history context 和 focused undo/redo routing 已进入产品路径。 |
| P1-07 · UI asset 私有 undo stack 无界增长 | **Open** | entry 克隆 source/document/selection/effects，undo/redo Vec 无 count/byte limit。应硬切到统一 history budget。 |
| P1-08 · Core history 只有条数上限，无内存预算 | **Open** | `HistoryStore` 仅按 capacity eviction；`TransactionRecord` 没有 retained byte accounting/oversize admission。 |
| P1-09 · 关闭 document 不清理 history/context | **Open** | core 有 `clear_history_and_context`，但普通 document close 不调用；animation detach 也不清 document history。长期开关会保留 owner state。 |
| P1-10 · Journal 只序列化，不持久化/恢复执行 | **Partial** | durable store、discovery、framing、codec replay 已存在；production append 仍明确只提供 `append_for_test`，transaction commit 未发布 immutable record，recovery 产品不消费 journal。 |
| P1-11 · Journal decode 缺预算与 schema 迁移 | **Partial** | 已有 record/file/count 上限、checksum、valid-prefix 与 exact codec；仍无 N/N-1 schema compatibility/migration、payload depth budget 和 quarantine product workflow。 |
| P1-12 · DocumentLifecycleAuthority 单 active document | **Open** | lifecycle authority 仍保存一个 `active_document`，而 toolkit/session 支持多开；project/document/scene transition 语义不一致。 |
| P1-13 · Project save 与 document batch 两套协议 | **Open** | Save Project/scene/workspace 和 Save All/toolkits 使用不同 participant、receipt、retry、terminal；close 只是顺序调用而非一个原子/可恢复 plan。 |
| P1-14 · 外部冲突检查没有 write-time CAS | **Partial** | `source_write_authority.rs:152-188` 在同进程 lease 内比较 expected bytes 后 atomic publish；外部 writer 可在 read/replace 间竞争，缺 OS identity/replace precondition/source-control revision。 |
| P1-15 · disk commit 与 import/refresh/hydrate 无复合终态 | **Open** | UI asset 在 `save.rs:178-181` 忽略 import result，再执行 refresh/hydrate；source 已持久化后 projection 可失败，却没有 `PersistedButProjectionFailed` 与 recovery action。 |
| P1-16 · Autosave budget 不覆盖实际 payload | **Open** | `editor_document_autosave.rs:190` 只用 `size_of::<Self>()` 估算；没有序列化后 bytes、document complexity、compression/rotation I/O admission。 |
| P1-17 · Autosave 在 worker 捕获完整可变 document | **Open** | `editor_document_autosave.rs:131` 在 worker 升级 manager 并 capture；没有 UI commit 点的 immutable/COW snapshot 或 streaming serializer。 |
| P1-18 · Autosave interval 硬编码、无 hot settings | **Partial** | interval 已成为 validated user setting 并可 hot apply；idle、retention、disk/memory、battery、play mode、disable 和 per-domain policy 仍缺。 |
| P1-19 · Autosave failure 压成计数 | **Partial** | outcome 已含 document/path/stage/retryability/error chain/usable snapshot，且写 append-only diagnostic；产品仍缺逐项 retry/open folder/disable-document 行为。 |
| P1-20 · shutdown 先取消 autosave 再 close project | **Partial** | normal exit 会提交 final autosave 并以 5 秒 deadline 等待，失败可阻止成功退出；执行时 runtime session 已结束，capture 仍 worker-deferred，也没有 journal checkpoint/统一 close receipt。 |
| P1-21 · Project switch/drop 丢旧 autosave completion | **Closed** | service 保留 retired project adapters，持续 pump completion 并持久化 fallback diagnostics。 |
| P1-22 · Recovery freshness 仅依赖 mtime | **Closed** | catalog/decision 使用 captured source digest、committed snapshot digest 与当前 source 比较，不再只依赖 mtime。 |
| P1-23 · RestoreFlow 无 executor | **Partial** | `RestoreExecutor` 已执行 no-overwrite restore/comparison copy 和 scoped discard；仍未把 recovered copy 自动打开、import/hydrate，也无 retry/defer/cancel workflow。 |
| P1-24 · 一个坏 recovery entry 使 catalog 全失败 | **Closed** | discovery 隔离坏 entry 并继续返回其他 candidates/diagnostics。 |
| P1-25 · Session heartbeat 无产品调用 | **Closed** | retained tick 驱动 heartbeat，并投影 degradation/failure state。 |
| P1-26 · Drop 释放 session lock 的 I/O failure 被忽略 | **Closed** | normal close 使用显式 release 并传播错误；异常 Drop 只释放 OS lease、保留 residual 是 recovery 语义，不再做 fallible Drop I/O。 |
| P1-27 · DocumentToolkit 没有 Save As 合同 | **Open** | toolkit 只暴露 save/source/autosave capture；生产树没有 Save As command、path admission、format/extension、identity rebinding 或 recent/import transaction。 |
| P1-28 · Writable flag 无 source-control/checkout owner | **Open** | [Editor223](223-editor-project-operations-source-control-provider-workspace-changelist-diff-automation-validation-submission-gate-health-dashboard-current-source-review.md) 已确认 `SourceControlProvider` 等产品合同零命中；保存无法返回 checkout/declined/changelist terminal。 |
| P1-29 · Animation dirty bool 与 DirtyRegistry 双重真值 | **Closed** | animation session dirty 由 `DirtyRegistry::document_dirty_if_registered` 投影，私有 dirty truth 已移除。 |
| P1-30 · Transaction fault/drop error 无产品恢复 owner | **Open** | event rejection 只 warning，`take_drop_error` 仅测试消费；无 incident center、document quarantine、retry/reload/recovery transition。 |

## 6. P2 当前重判

| ID | 状态 | 当前源码证据与差距 |
|---|---|---|
| P2-01 · Transaction event sink rejection 只 warning | **Open** | Backpressured/Rejected 只记录日志，缺 bounded retry、resync marker、operation failure 与 health projection。 |
| P2-02 · Document path identity 使用弱 FNV/collision probing | **Open** | lifecycle document identity 仍含 FNV 常量与 collision stepping；未统一到 canonical persistent/document identity manifest。 |
| P2-03 · Recovery metadata/JSON 缺输入大小与深度限制 | **Partial** | activation ledger 有 4 KiB cap、journal 有 framing budget；autosave diagnostics/catalog/restore 仍有无界 `fs::read` 与 JSON depth/candidate-count 空白。 |
| P2-04 · Windows parent-directory durability 未证明 | **Open** | atomic writer 在 Windows 的 parent directory sync 语义与 crash matrix 未证明；Editor 却把 best effort receipt命名为 Durable。 |
| P2-05 · rotation failure 不向 UI 暴露可用 snapshot | **Closed** | `AutosaveDocumentOutcome` 明确携带 `usable_snapshot`，rotation-after-write failure 可被诊断层观察。 |
| P2-06 · Save Local Copy exists-then-write TOCTOU | **Closed** | local copy 已使用 no-replace atomic publication并在命名竞争时重试。 |
| P2-07 · Save report 缺内容与 durability metadata | **Open** | `DocumentSaveReport` 只有 document/toolkit/reason/bytes 与粗粒度 guarantee；empty receipt 被丢弃，缺 source identity/hash/generation/sync scope/publication/import state。 |
| P2-08 · 保存路径把 typed error 压成 String | **Partial** | source/recovery/autosave 有 typed error 基础；foreground job、save batch和产品 feedback 仍频繁 `to_string()`，调用方无法稳定分支 retry/conflict/declined。 |

## 7. 参考引擎校准

| 参考 | 可验证 owner/合同 | 对 Zircon 的约束 |
|---|---|---|
| Unreal | `FileHelpers.h` 同时提供 dirty package save、checkout-and-save、可拒绝结果和 failed package list；`PackageAutoSaver` 观察 dirty/saved/undo/end-frame，维护 restore data、source-control/uncontrolled changelist 与可配置 warning/time；`TransBuffer` 以 `MaxMemory`/`DataSize()` 驱逐。 | Zircon 不能把 Save All、checkout、autosave、restore、history byte budget 分给互不相干的 UI 分支，也不能只返回 String/boolean。 |
| Godot | `EditorUndoRedoManager` 有 per-history `saved_version`、unsaved 状态、object/history routing、global/remote history 和 clear/discard lifecycle。 | focused document routing、saved-top 与 document close cleanup 应由同一 history authority 管理。 |
| Fyrox | File menu 具有 Save、Save As、Save All；关闭场景使用 Yes/No/Cancel confirmation 与明确 action。command stack 是 scene mutation owner。 | Zircon 的单 tab close、Save As 和 multi-document action 必须成为产品合同，不能只依赖主窗口 close。 |
| Bevy | `AssetSaver` 把 typed serialization 与 `AssetWriter` storage boundary 分开，支持异步 writer/error。Bevy 本身不是完整 editor 文档/恢复产品。 | 可借鉴 serializer/writer/provider 分层，但不能用 Bevy 的 runtime asset API 证明 Zircon 已完成 editor close/recovery。 |
| Unity Graphics | 本地参考仅有 SRP `AssetDatabaseHelper` 的 find/load helper，不包含 proprietary Unity Editor document、undo、autosave 或 recovery owner。 | 该参考只能验证 AssetDatabase 调用边界，不能据此判定 Unity 级编辑器工作流已覆盖；相关结论必须保持“reference unavailable”。 |

## 8. 目标架构与 owner 边界

目标不是新增一个更大的 manager，而是冻结以下少数 owner，并消灭旁路：

| Owner | 唯一职责 | 禁止旁路 |
|---|---|---|
| `DocumentRegistry` | 多 document identity、source binding、toolkit instances、dirty generation、history context、lifecycle state | session 私有 dirty bool、单 active document authority |
| `DocumentTransactionAuthority` | command apply/revert、focused routing、count+byte budget、saved-top、journal capture、fault/quarantine | UI asset 私有 undo stack、warning-only event loss |
| `DocumentSaveCoordinator` | Save/Save As/Save All/Close/Project composite plan、async operation、CAS/revision、durability receipt、projection terminal、retry/cancel | caller `ticket.wait()`、domain-specific write/import chain、scene/document 双协议 |
| `EditorRecoveryCoordinator` | autosave immutable snapshot、journal checkpoint、session lease、startup decision、restore/open/import、retry/defer/cancel | worker 捕获 mutable owner、restore 只生成孤立副本 |
| Runtime filesystem provider | capability root、secure open、atomic/durable publish、no-replace、watch/reconcile、typed I/O telemetry | Editor direct `std::fs`、host-dependent path/URI；由 Runtime160 实施 |
| Source-control provider | checkout/status/revision/changelist/decline receipt | writable bool 或提示字符串代替 provider |

统一保存终态至少应包含：`Unchanged`、`Persisted`、`PersistedButProjectionFailed`、`Conflict`、`SourceControlDeclined`、`StaleGeneration`、`Cancelled`、`FailedBeforeWrite`。receipt 必须绑定 operation id、document/source identity、expected/committed digest、source-control revision、publication generation、durability scope、bytes 和 projection outcome。

## 9. 分阶段重构计划

### M0 · 封闭剩余数据丢失入口

1. 让 tab close、layout reset、plugin/automation close 全部提交 `ClosePlanRequest`，删除 `close_view()` 直接 teardown 权限。
2. 把 captured toolkit generation 变成 plan completion fence；participant set 变化必须重新决策。
3. 为当前 close/save/recovery 产品路径补 retained-host 与 child-process regression，证明 dirty participant 不会静默消失。

### M1 · 建立异步统一 Save Coordinator

1. 删除 foreground `ticket.wait()`，返回 operation handle；UI 只消费进度/逐项 terminal。
2. 将 scene/workspace/toolkit/Save As/Save All/close-save 纳入同一 immutable participant plan。
3. 扩展 source receipt 与 typed terminal；disk commit 后 import/refresh/hydrate 失败必须可重试且不得误报未保存。
4. 接入 source-control revision/checkout owner，并在 Runtime160 provider 完成后硬切 Editor direct filesystem writes。

### M2 · 收敛 Transaction、History 与 Journal

1. 将 UI asset 私有栈迁移到 document-scoped transaction authority；统一 scene/animation/UI asset focused routing。
2. 对 history 同时设置 count/bytes/oversize budget，关闭 document 时回收 history/context/dirty/toolkit。
3. 在 transaction commit 线性化点捕获 immutable journal record，开放 production durable append；定义 schema compatibility、quarantine 和 recovery replay。
4. 建立 transaction fault owner，把 event loss/drop error 转成可观察、可恢复的 document state。

### M3 · 完成 Autosave/Recovery 产品链

1. UI/transaction commit 点生成 immutable/COW/stream snapshot；admission 以实际 bytes、time、concurrency 和 I/O budget 计费。
2. 增加 idle/retention/disk/battery/play/per-domain policy，并保留 hot settings projection。
3. 将 final autosave、journal checkpoint、close receipt 与 session release 排入一个 shutdown transaction。
4. Restore/Compare 后自动打开或导入副本，支持 per-document retry、defer/cancel、open folder 与清晰 source-preservation 说明。

### M4 · 工程资格与性能

1. 建立 1/100/10k document Save All、巨大 command、长 session、large autosave 与 recovery catalog 的 p50/p95/p99、peak bytes 和 cancellation budget。
2. 覆盖 write/flush/replace/import/refresh 任一点 kill、external writer race、disk full、permission/source-control decline、journal corrupt/truncate/old schema。
3. Windows 与非 Windows 都验证 parent-directory durability、path admission 和 scene document workflow；当前 non-Windows `protect_scene_path` 拒绝 scene path 是 Runtime160 的产品后果，不能被标记为 Editor 支持。
4. 只有在相同 workload、hardware、build profile、warmup、capture 和 correctness gate 下与 Unreal 对比，才允许提出性能优越结论。

## 10. 资格门

| Gate | 状态 | 当前判断 |
|---:|---|---|
| G1 任意 UI/command/plugin/automation close 不丢 dirty tab | **Fail** | tab close/layout reset 仍有旁路。 |
| G2 Close Project/Open Another/main exit 共享 close plan | **Partial** | Close Project/main exit 已收敛；全部 programmatic/project switch caller 尚未证明。 |
| G3 Save & Close 全部/部分失败/retry/new edit 语义 | **Partial** | async batch和 generation recheck 已有，toolkit fence/per-item product retry不足。 |
| G4 任意 write/flush/rename/import/refresh kill 保留可解析版本 | **Partial** | atomic source publication 已有，projection/parent durability/kill matrix未证。 |
| G5 external writer prompt-to-write 冲突拒绝 | **Partial** | expected-byte compare存在，跨进程 read-replace race仍开。 |
| G6 Save All 1/100/10k 有界、取消、非阻塞、逐项终态 | **Partial** | batch/polling有基础，explicit save会wait且无10k动态证据。 |
| G7 全 domain focused document 统一 undo/redo | **Fail** | UI asset 私有栈。 |
| G8 history count+byte budget | **Fail** | 无 byte accounting。 |
| G9 close 回收 history/context/dirty/toolkit | **Fail** | 普通 document close不清history/context。 |
| G10 journal corrupt/limit/N-1 有效前缀恢复 | **Partial** | bounds/prefix/codec有基础，production append与migration缺失。 |
| G11 autosave byte/time/concurrency 与前台 latency budget | **Fail** | request struct size代替实际payload，无动态门。 |
| G12 坏 recovery entry 隔离且默认不覆盖 source | **Pass** | catalog isolation与no-overwrite recovered copy均存在；动态fault仍应补。 |
| G13 residual inspect/takeover/defer/cancel/live safety | **Partial** | restore/takeover基础存在，defer/cancel/rich inspect不足。 |
| G14 heartbeat/release failure 可见且可恢复 | **Partial** | heartbeat与显式release已有，完整产品故障矩阵未证。 |
| G15 save report区分完整 terminal | **Fail** | receipt/report无法表达 persisted-projection failure/source-control decline。 |
| G16 retained host/child/fault 产品测试 | **Fail** | 仅静态复核，未运行真实进程/故障资格。 |

汇总：**7 Fail / 8 Partial / 1 Pass**。

## 11. Canonical 与跨报告关系

- Editor02 继续拥有 5 项 P0、30 项 P1、8 项 P2 的 canonical finding id；Editor225 只刷新 currentness 和实施顺序，不重复增加唯一总数。
- Runtime160 拥有 filesystem provider、path/URI、secure open、TOCTOU、mount/watch、durability primitive 与 direct-fs hard-cut。Editor225 拥有 authoring participant、save/close terminal、transaction/history、autosave/recovery 产品工作流。
- Editor223 拥有 source-control/project operations provider、validation/submission owner；Editor225 只要求 Save Coordinator 消费该 provider，不复制其 workspace/changelist finding。
- Tooling 因未来迁移到 Rust 而按用户要求暂时排除。本报告没有借 Tooling 代码宣称 Editor 产品链已完成。
