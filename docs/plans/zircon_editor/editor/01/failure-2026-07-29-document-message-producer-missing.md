---
handoff_kind: failure
status: open
created_at: 2026-07-29
summary_slug: document-message-producer-missing
origin_plan: docs/plans/zircon_editor/editor/12-plugin-management.md
fixing_plan: docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
origin_child_dir: docs/plans/zircon_editor/editor/12
fixing_child_dir: docs/plans/zircon_editor/editor/01
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/editor_message/message/document.rs
  - zircon_editor/src/core/editor_message/topics.rs
  - zircon_editor/src/core/context/editor_context.rs
  - zircon_editor/src/core/project
  - zircon_editor/src/core/document/mod.rs
  - zircon_editor/src/core/document/lifecycle.rs
  - zircon_editor/src/core/mod.rs
  - zircon_editor/src/ui/host/editor_manager.rs
  - zircon_editor/src/ui/host/editor_manager_project.rs
  - zircon_editor/src/ui/host/editor_manager_startup.rs
  - zircon_editor/src/ui/host/project_access.rs
tests:
  - cargo test -p zircon_editor --lib --locked editor_message
  - cargo test -p zircon_editor --lib --locked document
---

# Editor01: document authority does not publish typed document facts

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/12-plugin-management.md`
- 来源执行切片：M1.2 plugin lifecycle message bridge
- 修复责任计划：`docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md`
- 交接原因：Editor12 将 document authority 的事实订阅为 `SceneChanged` plugin lifecycle event；文档创建、打开、保存与关闭的单一事实来源和总线出口属于 Editor01 kernel/message boundary。

## 失败现象与复现证据

`DocumentMessage::{Opened, Closed, Saved, DirtyChanged, FocusRequested}` 与 `TOPIC_DOCUMENT` 已定义，Editor12 bridge 已明确只消费结构性 Opened/Closed/Saved。当前生产代码搜索没有任何 `DocumentMessage::{Opened, Closed, Saved}` 或 `TOPIC_DOCUMENT` publish 调用，命中仅为消息定义与 bus 测试夹具。

因此没有真实 document lifecycle 进入 `editor.document`，也不能由 plugin bridge 诚实地发出 `SceneChanged`。这不是 Editor12 缺少更多 fallback，而是 Editor01 事实生产端尚未接线。

## 最低共享层根因

Editor01 尚未在 document/project authority 提交成功后向 `SharedEditorMessageBus` 发布 typed document fact。最低修复层是唯一 document lifecycle owner，不是 UI tab、retained host 或 plugin callback 层。

## 架构修复验收

- document authority 成功完成 open、save、close 时，各发布一次对应 `EditorMessagePayload::Document` 到 `TOPIC_DOCUMENT`；失败、取消和 UI focus/dirty 变更不得伪装为结构性事件。
- 事件携带稳定 `DocumentId`，在 authority lock 之外发布，且不会由 observer 重入改变已提交 document state。
- 单测覆盖 success/failure/no-op 的精确消息计数与顺序；回跑 Editor12 bridge，验证每个结构性消息产生带相同 document id subject 的 `SceneChanged`。
- 统一的 document bus producer 供 Editor04/08/12 等消费者订阅，不建立 UI 私有旁路。

## 禁止临时方案

- 不得由 document tab、retained-host tick 或反射刷新猜测 open/save/close。
- 不得让 Editor12 从 dirty/focus 消息推断 `SceneChanged`。
- 不得在 plugin manager 中维护第二份 document state、兼容事件名或测试专用直接回调。

## 修复结果与回传

Open state: `部分修复，未回传`。2026-07-29 current source 已新增 `core::document::DocumentLifecycleAuthority`，由 `EditorManager` 在成功打开、启动恢复、预制项目打开和创建项目打开后发布 `Opened`，在保存的持久化提交后发布 `Saved`；同项目重复打开与非活动保存保持 no-op，项目切换保持 `Closed -> Opened` 顺序，发布发生在 authority lock 与 host commit 之外。显式 project close 仍缺少 runtime `AssetManager` 的原子 retire contract，已移交 Frameworks05：[`project-asset-manager-close-contract-missing`](../../../zircon_runtime/frameworks/05/failure-2026-07-29-project-asset-manager-close-contract-missing.md)。在 success/failure/no-op 消息矩阵、受管 Cargo、独立复审和 Editor12 真实 subscriber 回归完成前，不得生成 fixed return。

2026-07-29 current-source reachability review: `EditorManager::close_project -> EditorUiHost::close_project -> AssetManager::close_project` 只有定义，无 `MenuAction::CloseProject`、workbench action-id/control-id 映射、command binding 或 host event executor 调用。因此既有 `Closed` producer 不能从项目管理 UI 到达；也不能直接复用 `resolve_startup_session` 收尾，因为它会恢复并再次打开 last project，违反 projectless 单次投影边界。Editor01 必须补齐“committed runtime close -> non-restoring welcome session -> EditorState::clear_project -> welcome presentation”的单向 action，并覆盖 success/no-op/failure；在该链路、success/failure/no-op 消息矩阵、受管 Cargo、独立复审和 Editor12 上行回归完成前，不得生成 fixed return。

2026-07-29 managed validation-copy handoff: 跨 Editor01/Editor09/Frameworks05 的 31-path 联合验证会话已建立，副本 `f37a00eb32d8489eb408178722ee3e15` 在 `closure_planning` 阶段以 `validation_copy_external_source_missing` 终态失败，未物化源码副本、未启动 Cargo。该失败属于 Cargo 输入闭包缺失 external source descriptor，已归入 Coordinator01 [`validation-copy-zr-vm-external-source-pin`](../../../zircon_tooling/session_coordinator/01/failure-2026-07-27-validation-copy-zr-vm-external-source-pin.md)，不得作为生产代码编译结论；联合验证会话必须补全受管 external source 声明后重建副本，再执行 current-source Cargo、独立复审和 Editor12 subscriber 回归。

2026-07-29 managed validation-copy attribution handoff: external source 已固定到 `E:/Git/zr_vm@1326c9fc40500444ee524ce86c7d459c9636ec5a` 后，副本 `9e62796283784043b2956ecfb5ab6610` 在 `overlay_ownership` 以 `validation_copy_overlay_not_owned` 终态失败，错误路径为 `zircon_editor/src/core/commands/defaults.rs`。31-path 联合会话持有有效 lease，但该路径的 baseline attribution 仍属于旧的 Editor01 修复会话；当前 validation-copy 仅允许单一 session attribution，不能诚实地合并多个已受管 owner 的 overlay。该能力缺口归入 Coordinator01 [`live-lease-attribution-validation-copy-divergence`](../../../zircon_tooling/session_coordinator/01/failure-2026-07-26-live-lease-attribution-validation-copy-divergence.md)，必须由 attribution handoff/union-validation 路由解决，禁止通过 whole-file baseline reattribute 覆盖其中已有 foreign dirty hunk；无 Cargo 进程、无测试结果，failure 保持 open。

2026-07-29 consumer bridge independent re-review: `EditorPluginLifecycleMessageBridge` 当前源码在 bus drain 后将 delivery 先入 FIFO pending 队列，manager transition 返回错误时回推当前 delivery；`lifecycle_event_for` 同时匹配 topic 与 typed payload，且 `callback_failures` 累加实际 diagnostics 数。独立复审 Critical/Important/Minor = 0/0/0。该结论只证明 Editor12 消费端可以无丢失地消费将来的 `editor.document` 事实，不替代 Editor01 producer、immutable Cargo、或真实上行 subscriber 回归。

## 产出记录与时间

| 时间 | 状态 | 完成项目与证据 |
|---|---|---|
| 2026-07-29 CST | `OPEN / 待修复` | Editor12 M1 事件桥接审计确认 `DocumentMessage` 和 `TOPIC_DOCUMENT` 在生产端没有 publish 调用，仅有消息模型与测试夹具；按 Editor12 M1 接线清单回传 Editor01。尚未修改 Editor01 生产代码或运行 Cargo。 |
| 2026-07-29 CST | `OPEN / open-save source ready` | `DocumentLifecycleAuthority` 已提供稳定 `DocumentId`、打开切换顺序、活动保存和 no-op 语义；Manager 的 direct/startup/prepared/create 打开路径及成功保存路径已接到 `editor.document`。rustfmt 与 scoped diff-check 通过；Cargo、独立复审及 Editor12 上行回归未运行。显式 close 的 runtime retire contract 已按 Frameworks05 failure handoff 移交。 |
| 2026-07-29 CST | `OPEN / close reachability source ready` | File 命令注册表已投影 `file.project.close`，并经 workbench action/control 映射、host effect 到达 `EditorManager::close_project`；仅在该调用成功后进入不恢复 last-project 的 Welcome session，`EditorState::clear_project` 清除残留项目世界与历史。默认命令、菜单绑定和 effect 路由已有源码断言，rustfmt 与 scoped diff-check 通过；受管 Cargo、success/failure/no-op 的完整行为验证、独立复审、Frameworks05 runtime retire 验收及 Editor12 subscriber 回归尚未完成，failure 保持 open。 |
| 2026-07-29 CST | `OPEN / managed validation input failure` | 联合会话 `editor01-project-close-integration-validation-r1-20260729` 的 31-path immutable validation-copy `f37a00eb32d8489eb408178722ee3e15` 在 Cargo closure planning 以 `validation_copy_external_source_missing` 失败；无 source root、无 Cargo 进程、无测试结果。后续必须提供受管 external source descriptor 后重新物化，不能复用此失败副本。 |
| 2026-07-29 CST | `OPEN / managed validation attribution failure` | 固定 ZrVM external source 后，联合验证副本 `9e62796283784043b2956ecfb5ab6610` 在 `overlay_ownership` 以 `validation_copy_overlay_not_owned` 拒绝 `zircon_editor/src/core/commands/defaults.rs`；联合会话的 lease 不等于旧会话 baseline attribution，当前副本不能合并多 session overlay。无 Cargo 进程、无测试结果；须由 coordinator 提供 attribution handoff 或 union-validation，再重建不可变副本。 |
| 2026-07-29 CST | `OPEN / consumer bridge review accepted` | 独立复审 `EditorPluginLifecycleMessageBridge` 为 Critical/Important/Minor = 0/0/0：pending FIFO 在 transition error 时保留 delivery，topic 与 payload 双重过滤，失败计数按 callback diagnostics 累加。只接受 Editor12 consumer 静态正确性；Editor01 producer 的完整消息矩阵、immutable Cargo 与真实 subscriber 上行回归仍未运行，failure 保持 open。 |

## 2026-07-30 Performance01 性能验收补充

当前 producer 已进入 EditorManager 的open/save/committed-close链，但`DocumentLifecycleAuthority`仍在每次activate/close/save查询前复制完整root路径；首次root又分别由active及双向identity map拥有三份PathBuf，close后两张map永久保留历史root。该问题不改变本failure的消息正确性状态，却会让长会话root count/path bytes/RSS单调增长，已登记`PERF-MVP-593`。

Editor01修复producer验收时必须同时保持单一canonical path owner：active只保留handle/id，no-op/save/close先借用查询，closed identity/collision side table有明确硬预算或会话清理策略，同时保持reopen复用DocumentId、`Closed -> Opened`顺序与锁外publish。规模门为roots `1/1k/100k`、path `16B/4KiB`、operations `1/1M`、threads `1/16`；记录path alloc/clone bytes、PathBuf owner、map/RSS、hash/probe及mutex wait/hold，要求no-op/save/close path alloc为0、每known root正文owner不超过1、历史root状态有界。当前只完成静态审查，未运行managed Cargo或F0/F4产品trace，不得据此关闭failure。

2026-07-30 union-input recheck: R3 snapshot `1330` 仅覆盖 `lifecycle.rs` 与本记录，但 `HEAD` 不含 `zircon_editor/src/core/document/mod.rs`，该 lifecycle mount 和上层 producer/host 仍是历史未提交工作树状态。单文件 overlay 不能物化可编译的 `zircon_editor` source tree，故不得对 R3 单独预约或解释 Cargo 结果；后续必须先以完整 union source manifest 重新归属 document mount、core/host producer、Editor09 deactivation 与 Frameworks05 close contract，再执行受管联合验证。此为已有 validation-copy attribution/input failure 的补充证据，不是 Cargo 结果或 fixed return。

2026-07-30 Gateway union-validation dependency: Gateway R3 immutable copy `89f7c98785a444a5b92a48053f547fbb` pinned input manifest `7d8727f92cb11ea9ca60c00ff7ca4efc863078c4c2abf028cb028b918777fe55`; run `34e2ab62170a4c8999d0f1550873dbf5` ended `exit 101`. Coordinator persistence recorded empty `stdout_text` and `stderr_text` before the copy was removed, so this cannot honestly be attributed to Rust compilation, a test assertion, or child-process termination. The copy also attributed only Gateway overlay and excludes current uncommitted document lifecycle/producer/startup overlays, so it cannot represent full Editor01 current source. Snapshot `1346` then froze lifecycle hash `1c5a57a21169692f736fb6202a98bb6e24dd91ef1c7abb94fb203e4765cb3659` after accepted lifecycle `lease claim` request `df1a79b9f6224576a6999fc2865a5751`, but `baseline attribute` still returned `baseline_lease_missing`; this independently reproduces the attribution side of the same union gate. Rebuild only after Coordinator01's open [terminal-output-loss](../../../zircon_tooling/session_coordinator/01/failure-2026-07-30-validation-copy-run-terminal-output-loss-regression.md) and [live-lease-attribution-union](../../../zircon_tooling/session_coordinator/01/failure-2026-07-26-live-lease-attribution-validation-copy-divergence.md) repairs provide an auditable union manifest. This is not Cargo GREEN, code attribution, or a fixed return.

## 2026-08-14 PERF-MVP-593 调研与采样计划

### 结论边界

本节是开始优化前的调研与测量协议，不是完成记录，也不改变本 failure 的 `OPEN` 状态。2026-08-14 对 `DocumentLifecycleAuthority`、其 scene route、计划结构约束及参考引擎源码的复审确认：问题在 identity/retention 的数据模型，而不是 UI callback 的局部实现。动态基线尚未产生；当前共享验证窗口为 Tooling atomic closeout 与 UI12 M3 保留，禁止启动新的 Cargo 或产品进程，因此不得把静态审计误报为性能数据。

### 已确认的当前源码瓶颈

- `DocumentLifecycleState` 同时以 `ids_by_root: BTreeMap<PathBuf, DocumentId>`、`ActiveProjectSession::root: PathBuf` 与 `SceneDocumentKey::project_root: PathBuf` 保存 project root；scene key 还每次构造 `String` URI。已知 root 的查询虽可借用 `&Path`，首次/场景激活仍会形成多份路径正文 owner。
- root 与 scene identity 在 1,024 条预算达到后分别由 `trim_closed_roots`、`trim_closed_scene_documents` 在有序映射中查找一个非活动项、clone key 再删除。该淘汰是每次进入上限后的 O(N) 扫描与额外路径/键分配，不能作为 100k churn 的稳态算法。
- `document_id_is_occupied` 遍历两个 identity map；特意制造哈希碰撞时 `document_id_for` 和 `scene_document_id_for` 会退化为 O(N) probe。当前测试只证明 ID 语义，没有暴露实际 root/scene 的重复 body owner 或 lock hold 时间。
- `activate`、`begin_project_session`、scene route 验证和 close/save 都经过 `scene_route_gate` 加 state `Mutex`。计划要求保持 authority lock 外发布；重构不得把 path hash、淘汰或 bus publish 扩展到锁外的错误时序，也不得在生产路径引入 `.lock().unwrap()`。

### 参考约束与目标结构

- Unreal `UAssetEditorSubsystem` 将 asset editor 打开/关闭归入一个 subsystem，并在 `CullRecentAssetEditorsMap` 检测其 recent map 超过 MRU 预算的两倍后收敛，而不是让历史 identity 无界增长。Zircon 采用其“单 owner + 明确 budget/eviction”的原则，不复制其 UObject/toolkit 多实例模型。
- Fyrox `GameScene::from_native_scene` 在构造时借用 `Option<&Path>`，而场景运行时 state 以 engine handle/resource owner 组织；Zircon 保持 `DocumentLifecycleAuthority` 为唯一 document/scene identity owner，不让 host、picker 或 scene installer 建立第二份路径状态。
- `engine-code-structure-convention.md` 的 E9 要求生产共享锁通过集中 helper 处理 poison；`engine-code-review-findings-2026-06.md` 的 cache 复核要求 source identity 单一 owner、显式预算、O(1) 反向索引和可观测的 reuse/eviction 计数。该重构必须拆出 owner/test child，不能把 map、淘汰、指标和 UI route 混入一个大文件。

候选设计将在动态基线确认后落实为：每个 root/scene identity 的正文只由 lifecycle-owned slot 保存一次；active project/session/scene 状态只保留 typed slot 或 `DocumentId`；lookup 通过带随机化 hash 的 bucket 加精确 path/URI 比较完成；关闭 identity 以 epoch ticket FIFO 回收，使陈旧 ticket 可 O(1) 跳过，活跃 identity 不扫描；live `DocumentId` 使用专用 occupancy index 完成碰撞 probe。scene identity 必须引用 project identity slot，不能再次持有 project root。所有预算将命名为 policy/预算类型并附带指标，不能散落为新的裸常量。

### 受管测量协议

在 coordinator 允许新的 Windows validation/product job 后，先执行 WPR CPU+heap/allocation+context-switch capture（`wpr.exe` 10.0.26100.8972，2026-08-14 状态为 `not recording`）并将 ETL 与汇总仅写入受管的 `E:`/`D:`/`F:` 目标目录。采样按同一机器、release product path、预热后 31 次重复记录以下矩阵：roots `1/1k/100k`，path bytes `16B/4KiB`，operations `1/1M`，threads `1/16`；分别覆盖首次 activate、known-root no-op activate、save、close、closed-root reopen、project session 切换和 scene route switch。

每个样本必须同时记录 path allocation/clone bytes、root/scene body owner 数、live/history map 条目与 logical bytes、hash/collision probe 数、eviction/ticket stale-skip 数、`scene_route_gate` 和 state mutex 的 wait/hold、CPU p50/p95、RSS/private bytes、context switch 与功耗。验收下限是 known-root no-op/save/close path allocation 为零、每个 live identity 正文 owner 至多一个、历史状态有界、正常命中不扫描 closed history、event 顺序仍为 `Closed -> Opened` 且 publish 仍发生在 authority lock 外。ETL 火焰图必须验证热点不再落在 map 全扫描、key clone 或重复路径正文上；功耗只与同机空闲/基线对比，不伪造跨引擎绝对结论。

### 当前 source 状态（未验收）

2026-08-15 current source 已将原 test-only `retention_metrics` 硬切为 `DocumentLifecycleRetentionSnapshot`：它在既有 state lock 内按需读取 root/scene/session 的 identity 数、逻辑路径字节、活动 owner 与累计 probe/eviction 数据，不 clone path，也不重置计数。`DocumentLifecycleState` 对现有 document-id occupancy 查询、root eviction scan 和 scene eviction scan 记录饱和累计值；Rust source regression 覆盖 scene/session 路径观测、collision 两次 probe 与 root eviction scan。此处没有替换双 `BTreeMap<PathBuf, ...>` 算法、没有开始 Cargo/WPR/product capture，未产生动态基线或性能结论；在共享 validation window 解除、同矩阵 ETL 完成及受管复审前，本 failure 继续保持 `OPEN`，本节不是 `产出记录`。

### 实施与验证顺序

1. 先增加只读的 lifecycle retention/probe instrumentation 与 focused source regressions，锁定现状中重复 owner、淘汰扫描及 ID 语义；不改变 producer message contract。
2. 用上述 capture 建立重构前基线，复核数据是否支持 static hotspot 假设；若实际热点属于 project I/O 或 scene installation，则停止本次索引重构并按最低 owner 重新交接。
3. 仅在数据确认后硬切旧的双 `BTreeMap<PathBuf, ...>` retention 模型，实施 single-owner slots、O(1) ticket eviction 和 occupancy index；不保留旧 map/fallback/兼容层。
4. 重跑同一矩阵并作 p50/p95、allocation、RSS、lock 和功耗差分；随后才进入现有 document、editor_message、Editor12 bridge 的受管 Cargo/上行验证与独立复审。
