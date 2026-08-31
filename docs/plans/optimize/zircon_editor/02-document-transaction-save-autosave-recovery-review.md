---
related_code:
  - zircon_editor/src/core/editing
  - zircon_editor/src/core/document
  - zircon_editor/src/core/asset/dirty
  - zircon_editor/src/core/extension/toolkit
  - zircon_editor/src/core/recovery
  - zircon_editor/src/ui/host/layout_commands.rs
  - zircon_editor/src/ui/host/editor_ui_host.rs
  - zircon_editor/src/ui/host/editor_document_autosave.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/save.rs
  - zircon_editor/src/ui/host/animation_editor_sessions/save.rs
  - zircon_editor/src/ui/asset_editor/undo_stack.rs
  - zircon_editor/src/ui/retained_host/app/native_window_close.rs
  - zircon_editor/src/ui/retained_host/app/native_window_close/prompt_actions/actions.rs
  - zircon_editor/src/ui/retained_host/app/welcome_session/actions/project.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
  - docs/plans/zircon_editor/editor/03/2026-07-29-transaction-journal-contract-r2.md
  - docs/plans/zircon_editor/editor/03/2026-07-18-scene-command-transaction-hardcut.md
  - docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
  - docs/plans/zircon_editor/editor/17/2026-07-23-autosave-recovery-core-foundation.md
  - docs/plans/zircon_editor/editor/17/2026-08-05-current-source-implementation-records.md
  - docs/plans/zircon_editor/editor/01/failure-2026-07-29-document-message-producer-missing.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Public/FileHelpers.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/PackageAutoSaver.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Classes/Editor/TransBuffer.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/ITransaction.h
  - dev/godot/editor/editor_undo_redo_manager.cpp
  - dev/godot/editor/editor_node.cpp
  - dev/Fyrox/editor/src/lib.rs
  - dev/Fyrox/editor/src/menu/file.rs
  - dev/Fyrox/editor/src/command/mod.rs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: in_progress
source_recheck_required: true
---

# 02 · Document、Transaction、Save、Autosave 与 Crash Recovery 工程化差距

## 1. 结论

当前 Editor 的底层并非临时玩具实现。`EditorTransactionEngine` 已有 RAII transaction scope、嵌套/跨 context 拒绝、apply/revert 失败恢复、faulted 状态、per-document history、save token、generation-bound dirty、事件 sink 与分页 history；`DirtyRegistry` 能把 transaction dirty 和外部副作用合并；`SaveDirtyViewsRequest`/`SaveDirtyViewsJobAdapter` 已有批量 preflight、原子 admission、资源 mutex、非阻塞 budgeted polling、逐文档终态与 stale-generation 保护；autosave 已有单飞、公平窗口、同源保存互斥、临时文件 flush/rename、三代轮转、project-relative BLAKE3 identity 和 OS-backed session lease。这些基础应保留并成为唯一产品路径。

问题在于产品纵向链没有闭合，且已存在可直接丢数据的入口。单个 dirty tab 的 `close_view()` 不查询 `DirtyRegistry` 就删除 session/toolkit；`File > Close Project` 直接调用 `EditorManager::close_project()`，同样绕过主窗口 close prompt；主窗口和 floating window 虽会枚举 dirty document，但 prompt 的 **Save** 分支明确没有实现，只让用户选择 Discard 或 Cancel。UI asset 和 animation 的权威保存又直接 `fs::write`，进程或掉电中断可把原文件截断。残留 session lock 虽有 takeover 和 restore planning 底层，产品打开工程时却只报错并永久保留锁，没有恢复、比较、丢弃或接管入口。

事务、保存和恢复还分裂成多套权威：scene/inspector 使用 core transaction engine，UI asset 保留私有无界 undo stack，animation 只有局部 dirty bool；全局 Undo/Redo 固定投递到 `HistoryContextId::Global`；core journal 只能生成 JSON，不能持久化、解码 command、回放或作为 autosave 失败后的恢复层；core Save All 类型没有产品 caller，project scene/workspace 保存又绕开 document batch。底层能力存在，但产品行为仍由各 editor session 的特殊分支决定。

本轮登记 5 项 P0、30 项 P1、8 项 P2。修复顺序必须是：先封住所有 close/overwrite 数据丢失路径；再把 Save/Save All/Save on Close/Save As 收敛到一个异步 `DocumentSaveCoordinator`；随后把所有 editor domain 接入统一 transaction/history/journal；最后完成 startup restore、session takeover、source-control 和大型工程性能门。不能继续在每个 asset editor 内复制 `save()`、`dirty: bool` 和私有 undo stack。

## 2. 审查边界与证据

### 2.1 本轮物理范围

| 集合 | 文件 / 物理行 | 证据等级与边界 |
|---|---:|---|
| core editing production | 29 / 4,740 | E3：command/context/operation、transaction/history/event/journal/routing；fingerprint `7a7f5663b289257a7e629691181b464120b227eee4b6e941bd5ee8659d9246e8` |
| document lifecycle production | 4 / 959 | E3：project/scene lifecycle、ticket、retention；fingerprint `da8cae1480b9e84923b50c9176be4398f26302878dcd65410858ce0f7d601cc7` |
| dirty/save clean production | 4 / 1,448 | E3：registry、save batch、job adapter、external effect id；fingerprint `d4d62e5b6a62924e8f8e28f900159d21f4aa32cb4ba22379f3fe8442975f4c60` |
| document toolkit production | 17 / 913 | E3：registry、save/close lease、autosave payload；fingerprint `8d5285ddce63c1a0db0240711d8875fd1110fbef832567ffbdacd85caaef8b40` |
| recovery production | 17 / 2,901 | E3：autosave scheduler/store/catalog、restore flow、session guard；fingerprint `971797f076eb266f025756f9bee32d941695c6f4a26e893e1cd2c9eeb8ce3586` |
| product integration focused set | 18 / 5,254 | E3：tab/project/window close、foreground/autosave、UI asset/animation save、undo routing；fingerprint `417aa42287427ae2ef71f3c869d1b0794a92f780439be01282dc3658265702ac` |
| focused clean tests | 51 / 20,047 | E2：366 个 test attributes，未运行；只证明已有意图，不替代产品/崩溃测试 |

指纹算法与同目录前序报告一致：相对路径排序，逐文件 SHA-256，再对 `path<TAB>hash<LF>` 清单取 SHA-256。成文时，`core/asset/dirty/error.rs`、`mod.rs`、`tests.rs`、`save_job_adapter/tests.rs`、`core/recovery/tests/autosave_adapter.rs` 和 `ui/asset_editor/session/replay_artifact.rs` 有其他 Session 修改，已从上述 clean fingerprint 排除。本文结论依赖的 close owner、save owner、transaction engine、save batch、recovery production 均为干净快照；实施前仍须重读被排除文件及调用方，因此保留 `source_recheck_required`。

### 2.2 纵向调用链

本轮逐项追踪了：

1. input/menu/layout command -> `close_view()`/`close_project_from_workbench()`/native window close -> toolkit lease -> dirty unregister；
2. scene/UI asset/animation edit -> transaction 或私有 undo -> DirtyRegistry/external effect -> workbench dirty projection；
3. Explicit/SaveAll/Close reason -> toolkit save -> foreground job -> source write -> import/refresh -> generation-bound mark saved；
4. dirty snapshot -> autosave admission -> document save mutex -> capture -> atomic snapshot/rotation -> completion projection；
5. project startup -> session lock admission -> residual/live decision -> recovery catalog -> restore planning；
6. core transaction -> history/save token/event/journal serialization -> UI Undo/Redo routing与project transition cleanup；
7. project scene/workspace save、document toolkit save、UI asset local-copy、animation save之间的 durability 与错误语义差异。

### 2.3 动态证据限制

本轮没有运行 Cargo、Editor、真实 window close、进程 kill、断电注入、大文件 autosave、source-control provider、文件 watcher race 或跨版本 journal replay。静态源码足以确认 prompt Save no-op、close bypass、direct write、residual admission hard error、无产品 Save All caller、blocking `ticket.wait()` 和 journal/restore consumer 缺失；并不足以宣称现有测试当前通过或量化性能。后续 gate 必须使用 child process、故障注入和真实 retained host，而不是继续增加 `include_str!` source assertion。

### 2.4 参考源码校准

- Unreal `FEditorFileUtils` 将 Save Dirty、Save Dirty Content、checkout/save 参数、failed packages 与 Success/Failure/Declined/Cancelled 终态分开；`PackageAutoSaver` 监听 dirty/saved/undo 并在恢复失败时保留恢复信息。可借鉴的是 batch authority、source-control admission 和失败后不丢恢复证据，不是复制 package 模型。
- Unreal `UTransBuffer` 同时以 transaction count 和 `MaxMemory`/`DataSize()` 驱逐 history，并提供 undo barrier。Zircon 当前只按 128 条计数，不能约束大 subtree/reflected payload。
- Godot `EditorUndoRedoManager` 为各 history 维护 saved version、unsaved state、clear/discard 生命周期，并按最近 history 路由；scene tab close 会同时检查 scene history 和 plugin unsaved state，再显示 Save & Close。
- Fyrox 的 `SaveSceneConfirmationDialog` 明确区分 Yes/No/Cancel，并把原动作延迟到决策后；File 菜单提供 Save、Save As、Save All，Save All 遍历全部 open scene。
- Bevy 参考树没有完整 editor document/save/recovery 产品，可借鉴的 asset saver 不能作为编辑器关闭协议基线。
- 仓内 Unity Graphics 是 package/rendering 源码，不含 Unity Editor 文档事务、撤销和恢复权威实现；本文不推断闭源行为。

## 3. 已有可保留基础

1. Transaction scope 能保证显式 commit/cancel，拒绝非法嵌套和跨 history 操作；apply/revert 中途失败会尝试恢复并进入 typed faulted state，而不是静默留下半修改 world。
2. History 已按 Global/Document 分域，save token 同 engine/history/generation 绑定，旧 token、undo 后 branch、history clear 和并发编辑不会错误地清除 dirty。
3. DirtyRegistry 把 transaction saved-top 与 typed external effects 合并，并以 generation snapshot 清理已保存副作用；这是统一 dirty authority 的正确方向。
4. `SaveDirtyViewsRequest` 会校验 toolkit generation、document/instance 对应、重复项、resource key、writable、reference policy 和 estimated-byte overflow。
5. `SaveDirtyViewsJobAdapter` 先原子预留整个 batch，再建立 per-resource mutex/job；completion polling 有固定 budget，并保留 Saved/Failed/Cancelled/StaleGeneration 的逐项结果。
6. Toolkit registry 对 save/close 使用 lease，能阻止同一 document 同时 save/close，并提供独立 autosave capture 与 source path contract。
7. Autosave scheduler 有 single-flight、windowed fairness、bounded admission、cooperative cancel；foreground save 与 autosave 通过同一 source-derived mutex 串行化。
8. Autosave snapshot 使用同目录 staged write、flush、rename 和最多三代 rotation；source identity 使用 project-relative path 与 BLAKE3，明显强于临时编号。
9. Session guard 使用 OS-backed ownership lease、staged lock publication、live/residual distinction 与 compare-and-replace takeover primitive，底层不是简单“有文件就拒绝”。
10. UI asset 已有 reload/keep-local/local-copy/diff 冲突交互；project scene/workspace 保存已使用 runtime `atomic_write` 并在 scene save 失败时回滚 workspace，证明共享 durability primitive 已存在。

## 4. P0：必须先封闭的数据丢失与恢复不可达路径

### P0-01 · 单个 dirty tab 关闭直接删除 document/session

`ui/host/layout_commands.rs:129-141` 的 `close_view()` 只取得 toolkit close lease、执行 layout close、删除 animation/UI asset session 和 view registry，最后 commit close 并从 DirtyRegistry unregister。它从未读取 dirty snapshot，也不返回 NeedsDecision。所有 Dock、命令和程序化 CloseView 最终都可走这条链。必须把 `close_view()` 改成只能消费 `PreparedDocumentClose`：clean 可立即关闭；dirty 必须持有 document/generation/toolkit generation 并进入 Save/Discard/Cancel；save 失败或 generation 改变必须保持 tab/session/toolkit 打开。

### P0-02 · `File > Close Project` 绕过统一 dirty gate

`ProjectCloseRequested` 被投影成 `close_active_project`，`dispatch_effects/side_effects.rs:15-17` 立即调用 `close_project_from_workbench()`，后者在 `welcome_session/actions/project.rs:35-38` 直接执行 `EditorManager::close_project()`。它不复用 native main-window dirty enumeration。项目切换、自动化收尾和 host shutdown也存在直接 close caller。所有 close project/open another project/app exit 必须先进入同一 `CloseCoordinator`；runtime/plugin/session guard teardown 只能在文档终态全部为 Saved/Discarded 后 commit。

### P0-03 · Window close prompt 的 Save 按钮明确未实现

native main/floating window close 已能发现 dirty documents，但 `native_window_close/prompt_actions/actions.rs:23-27` 的 `"save"` 分支只显示 “Documents could not be saved; use Discard or Cancel” 并重新展示 prompt。`SaveReason::Close` 没有产品 caller。用户无法执行工程编辑器应有的 Save & Close，只能丢弃或取消。必须接入 batch request/adapter，逐项显示进度和失败；失败项保持打开，可 retry/save-as/discard/cancel，成功项只在 generation 未变时关闭。

### P0-04 · Crash residual lock 让工程永久不可打开，恢复基础不可达

`editor_manager_project_session.rs:145-151` 遇到 `SessionGuardAdmission::Residual` 直接返回错误并保留锁。`SessionGuardResidual::take_over`、`replace_residual_at`、`AutosaveStore::recovery_candidates` 与 `RestoreFlow` 只有定义/测试，没有产品 executor 或 UI caller。一次 crash 可把工程锁死，用户也无法恢复 autosave。启动必须提供 Inspect/Restore Selected/Open Copy/Discard Recovery/Take Over/Cancel；takeover 要显示旧 instance、pid、heartbeat、lock age，执行 compare-and-replace 后才允许打开，失败不得删除恢复物。

### P0-05 · UI asset 与 animation 权威保存非原子覆盖原文件

`ui/host/asset_editor_sessions/save.rs:84` 和 `ui/animation_editor/session/lifecycle.rs:73` 直接 `fs::write`；local-copy 也先检查后直接写。进程中止、磁盘写满或掉电可能留下截断的权威源文件，而 scene/workspace 已有 `atomic_write`。所有 document save 必须经统一 staged write：同卷 temp、write/flush、可选 parse/validate、write-time source generation/digest CAS、atomic replace、parent durability、typed post-commit；禁止 asset editor 自己覆盖源文件。

## 5. P1：产品化前必须闭合的架构与工作流

### P1-01 · Core Save All 基础没有产品入口

`SaveDirtyViewsRequest`、adapter 与 `SaveReason::SaveAll` 已存在，但 SaveAll reason 只在 toolkit tests 使用，菜单 `SaveProject` 只保存 global scene/workspace。应增加产品 Save All command、快捷键、进度、cancel 与 summary，并复用同一 batch executor。

### P1-02 · Foreground save 提交后台 job 后在 caller 线程阻塞等待

`editor_ui_host.rs:236-246` submit 后立即 `ticket.wait()`；从 UI command 调用时会冻结 input/present，且失去 adapter 已有的 budgeted polling。改为异步 save session，由 frame pump 推进 completion，UI 显示 per-document 状态且允许取消。

### P1-03 · Close prompt 没有不可变 save plan 与 generation revalidation

Prompt 虽保存 dirty generation，但未构造 `SaveDirtyViewsRequest`，也没有 toolkit generation/source key/estimated bytes/writable/reference preflight。关闭决策必须先冻结可审计 plan，执行前和 completion 时都验证 generation；prompt 打开后继续编辑不能被旧结果误关。

### P1-04 · Scene/core 与 UI asset 使用两套 undo authority

只有 `EditorCommand` 实现 core `EditCommand`；UI asset 仍持有 `UiAssetEditorUndoStack` 和独立 journal/replay artifact。应让 UI tree/style/theme/promotion command 实现统一 command contract，私有栈只可作为迁移 adapter，最终删除双轨。

### P1-05 · Undo/Redo 固定路由 Global history

`EditorState::apply_intent` 对 Undo/Redo 直接调用 `HistoryContextId::Global`；`resolve_history_context()` 仅在测试使用。路由必须基于 focused document、tool mode、active transaction participant 和 modal scope，命令 palette/menu label也应显示目标 history 的 next action。

### P1-06 · Animation editor 没有 undo/redo

Animation session 只有 document bytes、dirty bool 与 save，没有 transaction/history。clip、track、key、curve 和 metadata 编辑必须使用 typed reversible commands，并支持 selection restoration、merge/coalesce 与 per-document history。

### P1-07 · UI asset 私有 undo stack 无界增长

`UiAssetEditorUndoStack` 使用两个 `Vec<SourceEditEntry>`，push 没有 count/byte budget；entry 可克隆 source/document/selection/external effects，`replay_records()` 又复制全部 history。大 UI 文档长时编辑会无界占用内存。迁移前至少增加 byte accounting、count cap、eviction 与 telemetry。

### P1-08 · Core history 只有 128 条计数上限，没有内存预算

一条 delete command 可持有整个 subtree，一条 reflected command 可携带大值；相同 128 条可能从 KB 到 GB。仿照 Unreal `UTransBuffer::MaxMemory/DataSize()`，每 command/transaction 必须报告 retained bytes，history 同时受 per-document/global byte budget、count 和 pinned barrier 约束。

### P1-09 · 关闭 document 不清理对应 history/context

Toolkit close 只 unregister DirtyRegistry；transaction engine 的 `histories`/`history_generations` BTreeMap 不因 document close 清除，产品只在 project transition 清 Global。必须由 committed document close 执行 exclusive history/context teardown，或明确保留可恢复 session 并设总预算/生命周期。

### P1-10 · Journal 只序列化，不持久化也不能恢复执行

Core 可生成 versioned transaction JSON，scene command 也有 payload，但没有 durable journal sink、sequence/checksum、command codec registry、decoder、replay executor 或 truncation/compaction。恢复系统也不读取 journal。应建立 append-only per-document journal，checkpoint 后裁剪，并在 autosave 无法覆盖最后 edits 时 replay。

### P1-11 · Journal decode 缺大小预算与 schema 迁移

当前 raw JSON 可直接 `serde_json::from_slice`，只接受 exact schema v1；command schema version 被记录但没有兼容表。需要 framed record、长度/深度/command 数预算、checksum、engine build/schema fingerprint、N-1/N compatibility 与 quarantine，不可对不受信恢复文件无界解析。

### P1-12 · DocumentLifecycleAuthority 仍按单一 active document 建模

Workbench/toolkit 可同时打开多个 document，lifecycle authority 却围绕一个 active project scene，在 scene switch 时发 Closed。应拆分 ProjectSessionAuthority 与多 document registry，每个 document 独立 Open/Dirty/Saving/Conflict/Closing/Closed generation，active/focused 只是一种 UI projection。

### P1-13 · Project save 与 document batch 是两套提交协议

`MenuAction::SaveProject` 直接保存 scene/workspace 并标记 Global history；UI asset/animation 走 toolkit。Save All/Close 无法给出一个 terminal aggregate。统一 coordinator 应把 global scene、assets、animations、workspace metadata 作为 typed participants，明确 all-or-partial policy 与失败后状态。

### P1-14 · UI asset 外部冲突检查没有 write-time CAS

Watcher 能显示 reload/keep-local/copy/diff，但保存前后没有用 baseline digest/generation 对源文件做 compare-and-swap；`DefaultHasher` 也不是稳定内容标识。外部编辑可能在 prompt 后、write 前发生并被覆盖。保存必须在 replace 前复核 file identity+digest+mtime/size，冲突则保留 local buffer并重新决策。

### P1-15 · 磁盘 commit 与 import/refresh/hydrate 没有复合终态

UI asset 先写磁盘，再 import、refresh、hydrate；后续失败会向用户报告 save failure，但源文件已永久改变。应区分 `PersistedButProjectionFailed`，保留 committed digest/path，自动重试 projection；绝不能把它当“没有保存”让用户再次覆盖。

### P1-16 · Autosave admission budget 不覆盖实际序列化 payload

Pending request 明确不携带 bytes，estimated bytes 主要是 request/job size；worker capture 后才分配完整 document `Vec<u8>`。队列有界不等于内存有界。capture contract 应先给出上限/stream writer，或使用 immutable snapshot/COW，并对单文档、全局在途字节与序列化时间设 budget。

### P1-17 · Autosave 在 worker 内捕获完整可变 document

Worker 取得 toolkit save lease后调用 editor session 序列化，可能长时间与前台编辑/显式保存竞争，也没有清晰的 immutable generation snapshot。UI thread 应快速冻结 generation-bound snapshot/command checkpoint，后台只编码和写盘。

### P1-18 · Autosave 间隔硬编码且没有 settings hot projection

Context builder 使用固定 300 秒默认，用户/项目不能配置 interval、idle delay、retention、最大磁盘/内存、battery/play-mode policy 或禁用。设置变化也不能热投影到 scheduler。

### P1-19 · Autosave failure 被压缩为计数

`AutosaveCompletion`/retained status 主要暴露失败数量，不保留 document、source、stage、path、error chain、retryability 和可用 snapshot。需要 durable diagnostics、activity entry、per-document retry/open-folder 与 health telemetry。

### P1-20 · 正常 shutdown 先取消 autosave，再直接 close project

Retained host 退出时调用 autosave shutdown deadline，然后直接 `EditorManager::close_project()`；没有 final dirty snapshot、pending save drain、journal checkpoint 或 user decision convergence。shutdown coordinator 必须先封闭新编辑，再完成/取消并解释在途 save，最后只在所有文档有终态后释放 project/session owner。

### P1-21 · Project switch/drop 会丢失旧 autosave completion 诊断

切换 project 时旧 adapter 被 begin_shutdown 后很快丢弃，未把 terminal/cancel/failure 投影到新 session 或全局 recovery log。旧工程的 recovery evidence 必须在 owner teardown 前持久化并可从 Welcome/Hub 查询。

### P1-22 · Recovery freshness 仅依赖 mtime

Restore candidate 主要比较 source/autosave modified time；metadata 只有 version/source path，没有 authoritative source digest、dirty generation、base checkpoint、engine/schema id。时钟回拨、复制文件和 coarse timestamp 会误判。snapshot metadata 应携带 source identity/digest、base generation、journal range 与 committed checksum。

### P1-23 · RestoreFlow 只生成 decision plan，没有 executor

`RestoreFlow::detect/plan` 能校验候选和 resolution，但没有实际 materialize/open-copy/replace-source/import/hydrate/mark-dirty executor，也没有 rollback。应以不覆盖源文件为默认：先打开 recovered copy，验证成功后由用户显式 Save/Save As。

### P1-24 · 一个坏 recovery entry 会使整个 catalog 失败

Catalog scan 对 malformed metadata、duplicate sequence 等使用全局 `Result` 早退。一个损坏 document 可阻止其他全部文档恢复。改为 per-entry quarantine/report，catalog 返回 valid candidates + diagnostics，只有 catalog root 不可访问才整体失败。

### P1-25 · Session heartbeat API 没有产品调用

`refresh_heartbeat()` 只存在于 guard/test；正常 editor session 的 persisted heartbeat 会停在启动时间。live/residual 判断与 takeover UI 因而缺少可靠新鲜度。由 lifecycle timer 定期 refresh，失败进入可见 degraded state并停止悄悄延长所有权假象。

### P1-26 · 早期 Drop 释放 session lock 的 I/O 失败被忽略

显式 close 可传播错误，但 `EditorManager` Drop 的 guard release 失败只能被丢弃，可能留下 residual；而当前产品又不能处理 residual。Drop 只能作为最后保险，正常 shutdown 必须显式 release + terminal log；失败锁要携带可诊断 reason，不得变成永久阻塞。

### P1-27 · DocumentToolkit 没有 Save As 合同

Toolkit 只有 save、source path 与 autosave capture，没有目标选择、extension policy、overwrite confirmation、new identity commit、reference rewrite 和 history saved-top 迁移。工程编辑器必须为 scene/UI/animation 统一提供 Save As，并区分 Save Copy。

### P1-28 · Writable flag 没有 source-control/checkout owner

Batch candidate 有 `writable`，但产品没有构建 batch candidate，也没有 provider、checkout、make-writable、failed/declined/cancelled package 终态。应建立可选 source-control service；provider 不可用时仍要有 filesystem read-only policy和 Save As fallback。

### P1-29 · Animation `dirty: bool` 与 DirtyRegistry 双重真值

Animation session 自己在 edit/save 中设置/清除 bool，host 再通过 external effect 投影 DirtyRegistry。任一后处理失败或异步 completion 都可能只更新一边。session dirty 字段应删除或降为从统一 history/external-effect generation 派生的只读 projection。

### P1-30 · Transaction fault/drop error 没有产品恢复 owner

RAII drop rollback error 存入 `take_drop_error()`，engine faulted 后会拒绝后续操作；但没有生产 consumer 清晰展示受影响 history、导出诊断、保存副本、reload document 或安全 reset。必须把 fault 变成 document-level blocking incident，禁止只在日志中留下不可操作状态。

## 6. P2：一致性、诊断与长期可维护性

### P2-01 · Transaction event sink rejection 只记录 warning

事件投递背压时没有 per-history resync marker；UI 可能保留旧 undo label/dirty projection。拒绝后应合并为 generation-based snapshot invalidation，由下一帧强制重取。

### P2-02 · Document path identity 使用弱 FNV/collision probing

Lifecycle 的 path-derived id 与 autosave BLAKE3 identity 不一致，collision probing还可能受打开顺序影响。建立统一 canonical project-relative `DocumentKey`，持久化边界使用稳定 digest，运行期整数 id 只作 session-local handle。

### P2-03 · Recovery metadata/JSON 缺输入大小与深度限制

扫描用户可修改的 recovery 文件时应在读取前限制单文件 bytes、JSON nesting、string/path 长度和 candidates 数量，超限进入 quarantine，避免启动期内存/CPU 放大。

### P2-04 · Windows parent-directory durability 语义未证明

Snapshot file 自身 flush/rename 已实现，但 Windows parent directory sync 为 no-op。需要按平台记录 durability level，并用 kill/power-loss harness 验证；不能把“rename 返回成功”等同于断电后必然可见。

### P2-05 · Rotation-after-write failure 没有把可用 snapshot 告诉 UI

轮转失败可能发生在新 snapshot 已成功写入之后，底层有 typed stage，产品却只显示失败计数。终态应携带 `snapshot_committed=true/path/sequence`，避免用户误以为完全没有恢复点。

### P2-06 · Save Local Copy 存在 exists-then-write TOCTOU

先 `exists()` 再 `fs::write()`，另一进程可在两步之间创建同名文件并被覆盖。使用 create-new/atomic no-replace 语义，冲突后重新生成目标。

### P2-07 · Save report 缺内容与 durability metadata

`DocumentSaveReport` 主要只有 written bytes/reason；应至少返回 document/source identity、captured/committed generation、content digest、atomic replace/durability level、post-commit projection和 warning，便于审计、恢复及 batch summary。

### P2-08 · 多条保存路径把 typed error 压成 String

UI asset/animation/host 多处 `error.to_string()`，丢失 source path、operation stage、I/O kind、committed/not-committed 和 retryability。统一 `DocumentSaveFailure` 并保留 source chain，UI 再负责本地化展示。

## 7. 目标架构

### 7.1 单一 Document Authority

每个打开文档由 `DocumentSessionRegistry` 拥有：稳定 `DocumentKey`、session-local id、toolkit、source identity、history context、dirty generation、conflict generation、autosave checkpoint、journal range 与 lifecycle state。Editor session 不再自行拥有第二份 dirty/undo 真值。

状态至少为：

`Opening -> Clean/Dirty -> SavePlanning -> Saving -> Clean | Dirty | Conflict | PersistedProjectionFailed -> ClosePlanning -> Closed`

Crash startup 另走 `RecoveryDetected -> Inspecting -> RestoredCopy/Discarded/Deferred`。所有转换产生 typed event 和 generation，UI 只投影状态。

### 7.2 Transaction/History/Journal Authority

统一 `EditCommandCodec` 注册 command id、schema version、apply/revert、retained bytes、journal encode/decode与兼容范围。History 同时受 count/byte/global budget，支持 document close teardown、barrier/coalescing、focused routing和 participant validation。Journal 使用 length-delimited record、sequence、checksum、base checkpoint 与 engine/schema fingerprint；autosave成功后可裁剪已覆盖 range。

### 7.3 Save Coordinator

`DocumentSaveCoordinator` 是 Save、Save All、Save on Close、Save As、autosave/manual checkpoint 的唯一调度 owner：

1. collect dirty snapshot并冻结 toolkit/source/generation；
2. source-control/writable/reference/conflict/budget preflight；
3. 原子 batch admission与per-resource mutex；
4. immutable snapshot/stream serialization；
5. write-time digest CAS与atomic replace；
6. typed post-commit import/refresh/hydrate；
7. generation-bound mark saved；
8. 逐文档 terminal report与batch summary。

显式 Save/Close 由 frame pump 非阻塞推进；autosave使用相同 durability writer，但永不覆盖 authoritative source。

### 7.4 Close Coordinator

Tab、floating window、main window、Close Project、Open Another Project、automation 和 process shutdown 只能提交 `CloseIntent`。Coordinator 收集 documents，生成 immutable plan，等待 Save/Discard/Cancel/Retry/Save As 决策；只有 terminal documents 才 commit toolkit/layout/runtime/plugin/session teardown。任何新编辑或 source conflict 都使旧 plan stale并回到决策态。

### 7.5 Recovery Coordinator

Project admission 先 inspect live/residual lease，再容错扫描 recovery catalog 与 journal。UI 对每个 document 显示 source/recovery time、digest relation、snapshot sequence和原因；默认恢复到临时 copy，不覆盖源文件。Takeover 使用 compare-and-replace ownership lease；所有 invalid entries quarantine，所有恢复选择与执行结果写 durable audit。

## 8. 分阶段重构计划

### M0 · 封闭数据丢失入口

1. 为所有 tab/project/window/process close 建立统一 dirty gate，删除直接 close caller。
2. 实现 prompt Save/Save All/Retry，失败 document 保持打开。
3. UI asset/animation 改用共享 atomic writer与write-time CAS。
4. 增加 residual inspect/takeover/restore-copy 最小产品入口。

### M1 · 收敛 Save Authority

1. 将现有 `SaveDirtyViewsRequest`/adapter 接入菜单、快捷键、close与progress UI。
2. 去除 UI-thread `ticket.wait()`，以 frame pump 处理终态。
3. 加入 Save As/Save Copy、source-control provider、typed compound commit和完整 save report。
4. 让 project scene/workspace成为 batch participant，不再使用平行协议。

### M2 · 收敛 Transaction 与 History

1. UI asset/animation commands迁移到 core history；实现 focused-document undo routing。
2. 引入 retained-byte accounting、global/per-document budget、close teardown与barrier/coalescing。
3. 完成 command codec registry、durable journal、checkpoint/compaction和受限 decoder。
4. 将 engine fault/drop error接入 document incident UI与recovery action。

### M3 · 完成 Autosave/Recovery 产品链

1. immutable snapshot/stream capture、真实 byte/time budget与settings hot projection。
2. 容错 catalog、digest/generation metadata、per-document diagnostics与retry。
3. startup restore executor、open-copy默认策略、journal fallback与session heartbeat。
4. shutdown quiesce -> final checkpoint -> decision -> teardown 的确定性闭环。

### M4 · 大型工程与长期门禁

1. 10k open documents、million-command editing、GB级asset、slow/network disk与source-control latency基准。
2. process kill、write failure、disk full、rename failure、watcher race、clock skew、corrupt catalog/journal故障注入。
3. N/N-1 journal与autosave兼容、Windows/Linux/macOS durability矩阵。
4. 与 Unreal/Fyrox/Godot 相同文档规模和保存场景做延迟、峰值内存、恢复时间与数据完整性对比；没有同负载证据前不得宣称性能优于 Unreal。

## 9. 验收门

1. Dirty tab 从任意 UI、command、plugin 或 automation close 路径都不能无决策消失。
2. Close Project/Open Another Project/main window exit共享同一 close plan；没有直接 teardown旁路。
3. Save & Close 能保存全部、部分失败可逐项 retry，失败/新编辑 document 始终保持打开。
4. Kill 发生在 write任意字节、flush、rename、import、refresh阶段时，原文件或新完整文件至少有一个可解析版本，不出现截断源文件。
5. External writer 在prompt后/write前修改源文件时，Zircon拒绝覆盖并进入 Conflict。
6. Save All 在1/100/10k documents下有有界admission、可取消、非阻塞UI和逐项终态。
7. Undo/Redo根据focused document路由；UI asset、animation、scene共享同一history authority。
8. History在单条巨大command与长session下同时满足count/byte budget，eviction不破坏dirty saved-top语义。
9. Closed document释放history/context/dirty/toolkit资源；长期打开/关闭不线性泄漏。
10. Journal损坏、截断、超限、旧schema不会导致启动崩溃或无界分配；有效前缀可恢复，坏记录被quarantine。
11. Autosave序列化与写入有明确byte/time/concurrency budget，前台input/present延迟满足产品预算。
12. 单个坏recovery entry不阻止其他documents恢复；恢复默认不覆盖source。
13. Residual lock可inspect、takeover、defer/cancel，live owner不会被误抢，takeover竞争只有一个winner。
14. Heartbeat持续更新；refresh/release失败形成可见diagnostic和可恢复residual。
15. Save report能区分未写入、已持久化但projection失败、stale generation、conflict、cancelled和source-control declined。
16. 产品测试使用真实 retained host、child process和故障注入；source-string测试不能作为上述行为gate。

## 10. 与既有计划的关系

- [Editor03](../../zircon_editor/editor/03-command-transaction-and-undo.md) 已拥有 transaction engine 基础、UI asset私有栈迁移与journal方向；本文不重复宣称其完成，而是把当前产品旁路、memory budget、document close lifecycle和恢复消费端列为必须补齐的验收条件。
- [Editor17](../../zircon_editor/editor/17-editor-services-and-recovery.md) 已拥有 autosave/session guard 基础；本文将“底层已实现但startup/UI/executor不可达”提升为产品级P0/P1，并补充统一close/save authority。
- [Editor01 failure](../../zircon_editor/editor/01/failure-2026-07-29-document-message-producer-missing.md) 记录 Close Project producer曾缺失；当前 producer 已可达，但实现从“不可达”演化为“可达却绕过dirty gate”，不能把旧 failure关闭等同于工作流完成。
- 本文只关闭 document/transaction/save/autosave/recovery 这一审查切片；selection、scene/prefab authoring、viewport/runtime bridge、inspector/tool mode、content import/reimport和editor plugin UX仍需独立报告。

## 产出记录与时间

- 2026-08-24 16:25 +08:00 | P0-02/P0-03 | `进行中，未验收`：新增 retained-host `project_close` owner，将 File > Close Project 先收敛到 `request_project_close()`，仅在 close plan 的 active scene（`HistoryContextId::Global` generation）与全部 dirty toolkit documents 均已保存或由用户明确 Discard 后，才调用 `commit_project_close()`；主窗口退出复用相同 scene/document plan。Save 会先走权威 Save Project，再复用既有 `SaveReason::Close` 异步 document batch；Discard 会重新校验每个 participant generation，发现新编辑时重新展示决策。旧的 `EditorManager::close_project` 产品 API 已硬切为 crate 内 `commit_project_close`。新增 scene-generation 和 dirty-project-close regression 覆盖；只完成 Rustfmt/`git diff --check` 静态检查，未运行 Cargo、真实窗口、进程退出或故障注入。自动化/进程收尾仍为受控 commit caller，尚未完成统一 CloseCoordinator、Save As/Retry、source-control/CAS、恢复和 P0 验收，故不得提交或发送企微。
- 2026-08-24 16:51 +08:00 | P0-05 | `进行中，未验收`：运行时 `resource::io` 新增 staged conditional replace 和 no-replace publication；UI asset canonical save 在暂存完成后复核打开时的完整 BLAKE3 源基线，检测到未被 watcher 投递的外部变更时保留本地缓冲、建立既有 conflict projection，且只在持久化成功后标记 source buffer 已保存。工作区、刷新计划和 conflict snapshot 已从 `DefaultHasher` 的 `u64` 改为完整 BLAKE3 digest；Save Local Copy、组件/主题提升使用 no-replace 发布并对名称竞争重试，提升发布失败会回滚未发布的内存命令；undo/redo 外部 source effect 已转为原子替换。动画文档保存先前已使用共享 atomic writer。新增 conditional/no-replace、未观测外部改写、相邻副本碰撞与提升目标已存在覆盖；只完成 Rustfmt/`git diff --check`/定向静态扫描，未运行 Cargo 或故障注入。conditional compare 是 staged 后、replace 前的同进程写入前复核，不构成跨进程或 source-control 的线性化 CAS；SaveCoordinator、typed persisted-projection terminal、OS 级身份锁和 P0 完整验收仍未完成，故不得提交或发送企微。
- 2026-08-24 16:59 +08:00 | P1-23 | `进行中，未验收`：新增 `core/recovery/restore_executor.rs`，仅消费既有 `RestoreFlow` 的完整决策计划；`RestoreAutosave` 与 `OpenComparison` 均以 no-replace atomic write 将 snapshot 物化到 `<project>/.zircon/recovered/{restore|comparison}/`，绝不覆盖 authoritative source，`DiscardAutosave` 只删除已验证归属的单个 autosave document 目录。执行器验证 candidate 文档映射和 autosave-path containment，返回 typed per-document report，并新增 source 不变与 discard 范围覆盖。项目 admission 仍把 residual 压成错误文本，retained host 仍没有 Inspect/Take Over/Restore/Open Copy/Discard/Cancel 决策 owner，且 report 尚未驱动 recovered-copy 打开、import/hydrate 或错误重试；只完成单文件 Rustfmt/`git diff --check` 静态检查，未运行 Cargo、真实进程或 kill/recovery 故障注入，故不得提交或发送企微。
