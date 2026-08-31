---
related_code:
  - zircon_editor/src/core/gui_startup_request.rs
  - zircon_editor/src/core/project
  - zircon_editor/src/core/hub_link
  - zircon_editor/src/core/recovery
  - zircon_editor/src/ui/host/editor_host_startup.rs
  - zircon_editor/src/ui/host/editor_manager.rs
  - zircon_editor/src/ui/host/editor_manager_project.rs
  - zircon_editor/src/ui/host/editor_manager_project_session.rs
  - zircon_editor/src/ui/host/editor_manager_startup.rs
  - zircon_editor/src/ui/host/project_access.rs
  - zircon_editor/src/ui/host/startup
  - zircon_editor/src/ui/host/editor_manager_plugins_export/native_registration
  - zircon_editor/src/ui/workbench/project
  - zircon_editor/src/ui/workbench/startup
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup
  - zircon_editor/src/ui/retained_host/app/welcome_session/actions
  - zircon_editor/src/ui/retained_host/host_contract/window/attention.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/capture.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/lifecycle.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/redraw/present.rs
  - zircon_editor/src/ui/retained_host/run_config.rs
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_app/src/entry/entry_runner/editor
  - zircon_app/src/entry/entry_runner/bootstrap.rs
  - zircon_app/src/entry/first_party_editor_plugins.rs
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_hub/src/process/editor_focus
  - zircon_hub/src/process/editor_handshake
  - zircon_hub/src/process/editor_launch.rs
  - zircon_hub/src/tauri_app/runtime_state/editor_launch_actions.rs
  - zircon_runtime_interface/src/hub_protocol
  - zircon_runtime_interface/src/project/session_lock
  - zircon_runtime_interface/src/project/manifest_summary
  - zircon_runtime/src/asset/project/manifest
  - zircon_runtime/src/asset/project/manager/open.rs
  - zircon_runtime/src/asset/project/manager/durable_transaction.rs
  - zircon_runtime/src/asset/project/paths.rs
tests:
  - zircon_editor/src/core/project/tests
  - zircon_editor/src/core/recovery/tests.rs
  - zircon_editor/src/tests/host/manager/bootstrap_and_startup/session_startup.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/welcome/open_recent.rs
  - zircon_app/src/entry/entry_runner/editor/tests
  - zircon_hub/src/process/editor_handshake/tests.rs
  - zircon_runtime_interface/src/hub_protocol/tests.rs
  - zircon_runtime_interface/src/project/session_lock/tests.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/06-plugin-manager-discovery-enablement-live-reload-settings-diagnostics-review.md
  - docs/plans/optimize/zircon_editor/07-play-session-process-pie-game-view-live-edit-recovery-review.md
  - docs/plans/optimize/zircon_editor/50-editor-extension-contribution-store-registry-toolkit-provider-snapshot-reload-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_app/01-product-host-bootstrap-loop-dynamic-runtime-shutdown-review.md
  - docs/plans/optimize/zircon_app/07-renderable-empty-project-template-create-import-render-export-evidence-product-integration-review.md
  - docs/plans/optimize/zircon_hub/01-project-engine-build-editor-launch-process-persistence-delivery-review.md
  - docs/plans/optimize/zircon_hub/04-command-action-message-delivery-task-history-view-model-localization-product-integration-review.md
  - docs/plans/zircon_editor/editor/16/failure-2026-07-18-runtime-preview-play-scene-report-args.md
  - docs/plans/zircon_editor/editor/16/failure-2026-07-23-project-session-lock-reuse-for-recovery.md
  - docs/plans/zircon_editor/editor/16/failure-2026-08-16-editor-host-hub-handshake-config-visibility.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/GameProjectGeneration/Private/GameProjectUtils.cpp
  - dev/UnrealEngine/Engine/Source/Editor/GameProjectGeneration/Private/SProjectBrowser.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/UnrealEdMisc.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/ProjectDescriptor.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Launch/Private/LaunchEngineLoop.cpp
  - dev/godot/editor/project_manager/project_manager.cpp
  - dev/godot/editor/project_manager/project_list.cpp
  - dev/godot/main/main.cpp
  - dev/Fyrox/project-manager/src/manager.rs
  - dev/Fyrox/project-manager/src/settings.rs
  - dev/Fyrox/project-manager/src/project.rs
  - dev/bevy/crates/bevy_app/src/app.rs
  - dev/bevy/crates/bevy_app/src/schedule_runner.rs
refreshes:
  - docs/plans/optimize/zircon_editor/51-editor-project-startup-open-create-authority-hub-handshake-session-guard-focus-recent-recovery-product-integration-review.md
doc_type: review-and-refactor-plan
review_status: current_source_refresh_complete
implementation_status: pending
source_recheck_required: true
---

# 124 · Editor 项目启动、打开/创建、Authority、Hub 握手、Session Guard、Focus、Recent、Recovery 当前源码刷新审查

## 1. 结论

这次复核以当前工作树为准，旧编号 51 中关于“没有 intent、没有生命周期、Ready 早于首帧、focus 无 generation、rollback 无 quarantine”的大部分结论已经被实现修正：`ProjectLaunchIntent` 已版本化并带 operation id，`ProjectAuthority` 有 data-only preflight 和 manifest digest，`ProjectSessionAdmissionRecordV1` 有 `Claimed -> PreflightApproved -> Activating -> Ready -> Closing/RecoveryRequired`，`SessionGuard` 区分 OS lease 与持久记录，Hub ready 已延迟到 first-presented，focus watcher 使用 generation 校验，activation ledger 也会在补偿失败时保留独占 guard。

当前仍不能宣称工程级项目会话闭环。核心问题不再是“有没有模块”，而是多个已存在的模块没有共同的 durable commit 语义：`editor_manager_project_session.rs` 先把 session lock 提交为 `Ready`，再提交 activation ledger 的 `Session` effect；guard、activation ledger、runtime project、plugin catalog、document journal、Hub ready 和 first-presented 仍然是互相独立的存储/回调。崩溃或并发 focus 落在这些窗口时，系统可以看到一个 `Ready` 会话，却无法证明所有 activation effect 已落盘。

普通 profile 的 composition plan 仍然把 manifest 中的 project scripts、native extensions 和 scene restore 全部批准，代码中没有签名、信任主体、授权版本或用户批准 receipt。`Safe`/`Recovery` 是一个很好的减权起点，但 `Normal` 目前等价于“打开项目即执行其派生代码”，这与 Unreal/Godot 的项目转换/不可信项目决策边界仍有明显差距。

`operation_id` 只是被写入 session/ledger，没有在 `execute_project_launch_intent` 入口做 durable dedup 或 replay lookup；Hub 重试、Welcome 双击和崩溃恢复无法得到同一个 terminal receipt。更直接的异常路径是 `RetainedEditorHost::new` 成功后，插件注册、模板同步、startup scene、layout、focus binding 任一步失败都会从 `run_editor_with_config` 提前返回，而 `RetainedEditorHost::Drop` 只停止 autosave、丢弃 hierarchy watcher，没有调用 `commit_project_close` 或显式释放 project session guard。

本报告登记 **5 项 P0、60 项 P1、15 项 P2 和 40 个资格门**。它是旧编号 51 的 current-source refresh，不重复声称旧问题仍存在；Editor02 继续拥有 document/autosave/heartbeat，Editor07 继续拥有 Play，Editor50/06 继续拥有 extension/plugin lifecycle，App07 继续拥有 project template transaction，Hub01 继续拥有 child supervision。本文只收敛项目启动到首帧、聚焦、关闭和恢复之间的跨模块契约。

## 2. 语料冻结与检查方法

### 2.1 当前证据规模

本轮从旧 51 的 75 个根中排除不属于本域的外部计划根，保留 74 个 evidence roots，并加入旧 51 作为 refresh link。按物理文件去重后得到：

| 类别 | 文件 | 总行 | 非空行 | bytes | `#[test]` | `#[ignore]` | fingerprint |
|---|---:|---:|---:|---:|---:|---:|---|
| current source/test | 250 | 34,833 | 31,700 | 1,255,806 | 451 | 0 | `479f92ad0ab3f187e4ed29e0a13c88c7999dc83ec6872501be2c0a3a88763db` |
| reference engines | 13 | 29,487 | 25,337 | 1,065,419 | 22 | 0 | `9d9ad77dcfc1754778ad76c7b68a38c70bab14a193e0a581c6af647fda7018b5` |
| plan/docs | 14 | 4,890 | 3,657 | 577,818 | 2 | 0 | `25827e52745d658c8ce75a1a80e5a7e9623ff5ccd049ffd86b531b16bf14aff8` |
| deduplicated union | 277 | 69,210 | 60,694 | 2,899,043 | 475 | 0 | `9891587d096e5b680c7d6ac9def32e1e38818fa5df63ff934c96a950790e05d2` |

fingerprint 采用规范化 forward-slash path、每个文件 lowercase SHA-256、LF 连接后再取 SHA-256；实施前必须重新计算。451 个测试属性不等同于 451 个产品 E2E；当前大量测试仍是 `include_str!`、`.contains` 和源码形状断言。

### 2.2 检查顺序与参考边界

按 `Hub/CLI/Welcome/Recent -> ProjectLaunchIntent -> canonical identity -> data-only preflight -> compatibility/composition -> session admission -> activation effects -> Ready/session generation -> focus binding -> native window -> first present -> Hub acknowledgement -> close/recovery` 正向追踪，再从每个 terminal receipt 反查 owner、generation、写入顺序和失败补偿。Unreal 用于 project browser、descriptor、migration、restart/转换语义；Godot 用于 project manager、backup、recovery mode 和 lock；Fyrox 用于 manager/recent 的反例；Bevy 只用于 runner/plugin phase，不把通用 App 生命周期当成项目管理实现。当前 `dev/Graphics` 没有与此产品会话等价的 Unity Editor/Hub 源码，本报告不以闭源行为作猜测。

## 3. 当前已达到的基线（不要回退）

1. `ProjectLaunchIntent` 已有 schema、source、profile、target 和 `ProjectActivationOperationId`；UI 不应绕过该入口直接操作 materialized project。
2. `ProjectAuthority` 的 canonical root、manifest digest、migration assessment、engine compatibility 和 composition profile 是正确的 data-only 边界。
3. Safe/Recovery profile 已移除 project scripts、native extensions、scene restore；该策略应扩展为显式 trust policy，而不是删除。
4. `SessionGuard` 的 OS lease、PID/instance/heartbeat、residual detection、atomic record mutation 和 recovery takeover 是可复用底座。
5. `ProjectActivationLedgerStore` 对每个 effect 记录 prepare/commit/rollback/recovery，recent 是 post-commit projection；不要把 recent 再变成 activation gate。
6. `HubEditorReadyReceiptV1::after_first_present`、first-presented callback、generation-aware focus target 和 owner-thread attention 已修正早报中的竞态方向。
7. activation failure 现在会保留 guard/quarantine，不再无条件 release；后续重构应围绕 typed reconciliation，而不是恢复旧的字符串 rollback。

## 4. 当前仍存在的主链断路

```text
ProjectLaunchIntent
  -> data-only preflight + SessionGuard(Claimed/Activating)
  -> runtime/plugin/document activation effects + activation ledger
  -> guard.commit_ready(generation)
  -> ledger.commit(Session)
  -> first-presented Hub ready / focus ack

问题：Ready 持久提交先于 Session effect；各存储没有共同 receipt/digest。
问题：Normal composition 直接批准 manifest-derived executable capabilities。
问题：operation_id 可寻址但没有 dedup/replay terminal store。
问题：host construction 后的 early-return 只走 Drop，Drop 不执行 project close。
```

因此当前状态机比旧报告完整，但仍可能出现“lock record=Ready、activation ledger=Prepared、Hub=未 ack、native window=已创建或已失败”的分裂组合。恢复流程能识别部分 residual，却没有一张可以证明整个 activation transaction 的不可变 receipt。

## 5. P0：必须先收敛的安全与一致性问题

### **P0-01** Ready record 在 activation ledger Session effect 之前提交

`activate_prepared_project_after_admission` 在 activation closure 成功后先调用 `guard.commit_ready()`，随后才调用 `ledger.commit(ProjectActivationEffect::Session)`。如果第二次 atomic write 失败或进程在两次写之间崩溃，`.zircon` session lock 已经对 Hub/其他 Editor 宣布 `Ready`，但 activation ledger 仍是 `Prepared`。focus 只要求 Ready generation，因此可以把请求发送给尚未拥有完整 session effect 的会话。

重构为单一 commit fence：先完成 ledger 的 session commit、runtime/plugin/document 的 terminal receipts，再以包含 ledger digest 的 `Ready` record 发布；或者把两者放进同一恢复可重放的 journal，并要求 Hub 只接受 receipt digest 匹配的 Ready。必须添加 crash injection 覆盖每个写入间隙。

### **P0-02** Guard、ledger、runtime、plugin、document、Hub ready 之间没有共同 durable activation receipt

当前每个子系统有自己的 generation、revision 或 ledger effect，但 `ProjectSessionAdmissionRecordV1` 不携带 activation-ledger digest、manifest digest、plugin catalog generation、document session generation、window/first-present evidence。`HubEditorReadyReceiptV1` 也只证明 process、instance、session generation 和 milestone，不证明这些存储属于同一 operation 的同一 commit。

这会把恢复从“按一份 receipt 重放”降级为“逐个猜测哪个模块已完成”。应建立不可变 `ProjectActivationReceiptV2`，绑定 operation id、ProjectIdentity、BuildSet、manifest/preflight digest、ledger digest、all effect dispositions、session generation、first-present proof 和 close disposition；所有 projection 只能引用该 receipt。

### **P0-03** Normal profile 没有显式的项目代码信任/授权边界

`ProjectPreflightCompositionPlan::compile(Normal)` 原样复制 project plugin/script manifest，并把 `allows_project_scripts`、`allows_native_extensions`、`allows_scene_restore` 全部设为 true。`EditorManager::complete_project_open` 随后按该 plan 应用 project plugins。代码没有签名验证、来源 principal、授权版本、用户批准记录、沙箱能力集或首次打开确认；只要选择 Normal，打开项目就可能加载 DLL/脚本。

保留 Safe/Recovery 的减权 profile，新增显式 `TrustDecision` 和 signed manifest/extension catalog 校验。Normal 也必须产生可审计 approval receipt，并在项目内容、BuildSet 或签名发生变化时失效；未信任项目只能进入数据预览或 Safe 模式。

### **P0-04** operation id 可写入但没有 durable dedup/replay authority

`ProjectLaunchIntent` 文档称其为 idempotency-addressable，但 `EditorManager::execute_project_launch_intent` 每次都重新执行 preflight、admission 和 activation，没有按 operation id 查询已完成、进行中或失败的 terminal receipt。activation ledger 文件名虽包含 nonce，重复请求在 ledger 尚存时只会得到 create error；ledger 清理后重试又可能再次执行实际副作用，调用者拿不到原始结果。

引入按 operation id 和 canonical ProjectIdentity 索引的 bounded launch journal：第一次请求创建 `Pending`，后续相同 payload 返回同一 receipt，不同 payload 返回 conflict；重启时从 journal/ledger 恢复到 `InProgress/Committed/RecoveryRequired`，绝不把重试当作新操作。

### **P0-05** Retained host 的 early-return 没有显式 session close/release 兜底

`run_editor_with_config` 在 `RetainedEditorHost::new` 成功后还会执行 editor plugin registration、template sync、startup scene、layout、focus binding 等可能失败的步骤。这些步骤直接 `return Err(...)`。`impl Drop for RetainedEditorHost` 当前只调用 autosave `begin_shutdown()` 并丢弃 hierarchy watcher；真正的 `commit_project_close()` 只在 event loop、autosave、settings flush 和 capture 全部成功后的正常路径调用。

这让构造后异常路径依赖进程退出和字段析构顺序，不能保证 runtime project、document journal、plugin registrations、session lock 和 Hub focus binding 已经按逆序关闭。实现不可失败的 shutdown coordinator：所有 early-return 进入 `Closing`，尝试 bounded drain/close/release，失败则保留 RecoveryRequired quarantine receipt，并为 Drop 仅保留最后一次 best-effort 诊断而不是承担业务清理。

## 6. P1：Intent、Identity、Preflight、Trust、Admission（01-15）

| ID | 当前差距 | 必须重构 |
|---|---|---|
| P1-01 | intent 只有 operation id，没有 caller request id 与 retry attempt。 | 分离业务 operation、传输 request、attempt；receipt 按 operation 去重。 |
| P1-02 | source 枚举没有 principal、权限和审计上下文。 | 加入 authenticated principal、origin process、policy profile 和 audit correlation。 |
| P1-03 | `retarget_open_existing_project` 保留原 operation id 却可改变路径。 | retarget 必须生成新 operation，或把 target digest 纳入 idempotency key。 |
| P1-04 | canonical path、project GUID、manifest digest 尚未组成单一 `ProjectIdentity` 类型。 | 所有 session、focus、recent、document API 使用 qualified identity。 |
| P1-05 | create 成功 receipt 与随后 open activation 没有父子 operation 关系。 | 建立 create transaction id、activation child id 和可追溯父 receipt。 |
| P1-06 | engine compatibility 只覆盖当前解析到的版本，不列出 provider/feature 缺口。 | 输出 Compatible/Upgrade/Downgrade/Reject 与 required provider matrix。 |
| P1-07 | migration decision 进入 preflight，但用户选择、backup digest、转换工具版本未进 receipt。 | 保存 decision history、backup/copy artifact 和 converter identity。 |
| P1-08 | Normal/Safe/Recovery 是粗粒度 profile，不能表达脚本、DLL、网络、写入等独立能力。 | 使用 capability lattice 与逐项授权，不靠三个 bool。 |
| P1-09 | Trust decision 没有签名密钥轮换、撤销、未知签名和本地覆盖策略。 | 设计 trust store、revocation、offline policy 和 operator override。 |
| P1-10 | preflight revalidation 只比较 manifest digest，未锁定所有会影响 composition 的输入。 | 绑定 engine BuildSet、plugin catalog、toolchain 和 policy digest。 |
| P1-11 | preflight receipt 没有明确 TTL 和 stale reason。 | 加 monotonic expiry、invalidated-by 字段和重新 preflight 入口。 |
| P1-12 | session admission principal 目前只能表达有限来源，角色与 access mode 不完整。 | 编码 writer/read-only/recovery/headless/migration 组合规则。 |
| P1-13 | `checked_epoch` 是记录内部计数，不是跨存储 fencing token。 | 让所有 effect commit 携带并校验同一 fencing epoch。 |
| P1-14 | local OS lease 对 network/removable filesystem 的语义没有产品级拒绝矩阵。 | 检测锁可靠性，unsupported storage fail-close 并显示迁移建议。 |
| P1-15 | process instance id 仍是可读格式，PID reuse/boot identity 的校验不足。 | 使用不可预测 boot token、OS creation token 和 instance generation。 |

## 7. P1：Activation、Commit、Rollback、Close、Recovery（16-30）

| ID | 当前差距 | 必须重构 |
|---|---|---|
| P1-16 | effect ledger 与 session record 各自 atomic write，没有跨文件 commit protocol。 | 引入 coordinator journal、prepare set、commit fence 和 recovery replay。 |
| P1-17 | ledger effect 集合固定，但未来窗口、shader cache、asset index 等 effect 没有扩展契约。 | 使用版本化 effect registry，unknown effect 进入 quarantine。 |
| P1-18 | `run_project_activation_effect` 的 closure 不能报告 partial side effects。 | 返回 typed effect receipt、resource handles 和 compensation requirement。 |
| P1-19 | plugin load 可能成功而 catalog projection 更新失败，缺同代标识。 | plugin mount receipt 绑定 session generation/catalog generation。 |
| P1-20 | document journal begin 与 UI document message publish 不是同一提交点。 | document session 先 durable commit，再发布 generation-stamped events。 |
| P1-21 | runtime project close 失败时仍需判定哪些 settings/log/plugin 已经清理。 | 每个 effect 保留 terminal disposition，reconciler 按逆依赖重试。 |
| P1-22 | guard release 只由显式 close 触发，缺少 coordinator-owned lease object。 | `ProjectSessionLease` 持有 close state、fence 和 release receipt。 |
| P1-23 | `Drop` 不能异步等待 close，也没有“shutdown in progress”诊断通道。 | Drop 只记录 unresolved lease，并由 owner-thread shutdown service 收敛。 |
| P1-24 | RecoveryRequired 只有项目级结论，缺 effect-level operator action。 | 输出每个 effect 的 retry/rollback/restore/manual action。 |
| P1-25 | residual takeover 要求 terminal ledger，但缺完整 ready receipt 的交叉校验。 | takeover 同时验证 session record、ledger、manifest 和 BuildSet digest。 |
| P1-26 | heartbeat 是 wall-clock millis，没有 monotonic sequence。 | 加 sequence、monotonic elapsed 和 clock-skew policy。 |
| P1-27 | liveness residual 与 operator takeover 的超时/权限没有统一 policy。 | centralize lease policy，记录谁、何时、为什么接管。 |
| P1-28 | close 期间新 open/focus 请求的拒绝原因不统一。 | `Closing` 返回 typed Busy/RetryAfter/TargetClosing ack。 |
| P1-29 | session record 没有明确 Closed terminal receipt，release 后证据被删除。 | 保留 bounded terminal close index，锁文件删除只删除 lease，不删除历史。 |
| P1-30 | ledger cleanup 失败会把已成功 close 变成后续启动噪声。 | cleanup 变成独立 maintenance projection，不能改变已提交 close 语义。 |

## 8. P1：Hub、Focus、Ready、Recent（31-45）

| ID | 当前差距 | 必须重构 |
|---|---|---|
| P1-31 | ready receipt 没有 activation ledger digest 和 manifest digest。 | receipt 绑定所有安全关键输入与 commit digest。 |
| P1-32 | first-presented callback 只报告一次，窗口之后的 presenter/GPU 丢失无 revocation。 | 增加 Ready、Interactive、Degraded、Closed milestone 与撤销事件。 |
| P1-33 | focus watcher 回调错误只 `eprintln!`，Hub 可能一直等待 ack。 | attention/consume 错误写入 bounded owner health 与 typed ack。 |
| P1-34 | focus request 过期、旧 generation、target mismatch 已能拒绝，但没有可查询的 audit record。 | 保存 request/ack/disposition 的短期审计记录。 |
| P1-35 | focus mailbox 仍依赖单机文件系统，跨用户/权限拒绝没有 capability handshake。 | 启动时交换 mailbox capability、owner principal 和 directory ACL。 |
| P1-36 | Hub poll 的 phase progress、deadline negotiation、cancel 仍缺统一 schema。 | 增加 phase stream、retry-after、cancel token 和 terminal reason。 |
| P1-37 | mailbox failed detail 可能经过 `error.to_string()`，公共错误与诊断边界不稳定。 | public code/parameters 与受控详细 log 分离并脱敏路径。 |
| P1-38 | ready/failed mailbox 的 cleanup、retention、replay protection 未形成统一 owner。 | handshake store 管理 nonce、ack、expiry、quarantine 和回收。 |
| P1-39 | focus publish 成功只代表文件入队，调用方仍容易把 Queued 当 Focused。 | API 类型上区分 Queued、Delivered、Focused、Denied、Unavailable。 |
| P1-40 | native foreground 被 OS 拒绝时只保留本地 attention 结果，协议没有 degraded UX。 | 返回 ForegroundDenied 并提供 taskbar/notification fallback。 |
| P1-41 | project switch 虽会 sync focus binding，但旧 request 的清理与新代绑定非同一 receipt。 | switch 先终止旧 generation，再原子绑定新 watcher。 |
| P1-42 | recent registry 是 best-effort，但 validity probe、排序、写入 revision 仍缺统一 CAS。 | monotonic sequence、bounded journal、merge/CAS 和 corrupt rebuild。 |
| P1-43 | recent 条目保留 canonical/display 混合字符串，无法安全迁移路径别名。 | identity digest 与 display alias 分开，支持 redaction 和 relocation. |
| P1-44 | recent auto-open 策略未把上次 crash、trust change、BuildSet change 纳入决策。 | policy engine 按 risk reason 选择 chooser、Safe 或 Normal。 |
| P1-45 | Hub 与 Welcome 的“已有 Editor”选择没有正式 single-project/multi-window 支持矩阵。 | 明确 topology、focus routing 与 admission policy，拒绝偶然行为。 |

## 9. P1：产品入口、测试与性能资格（46-60）

| ID | 当前差距 | 必须重构 |
|---|---|---|
| P1-46 | Welcome open/create/recent 虽都能形成 intent，仍缺统一 command receipt/UI pending model。 | 所有入口观察同一 operation timeline，禁止直接改 manager 状态。 |
| P1-47 | CLI/Hub 与本地 Welcome 的错误码、cancel、retry 语义不一致。 | 共享 launch protocol 和 localized public error catalog。 |
| P1-48 | create/open 的 progress 没有 manifest、compat、lease、plugin、document、present 分阶段计时。 | 暴露 phase timeline 和 per-phase diagnostics。 |
| P1-49 | project switch 对 dirty document、Play session、asset jobs 的 veto 由其他报告分别拥有。 | Editor51 只编排 typed veto/drain receipts，再进入 Closing。 |
| P1-50 | startup failure 测试多为源码字符串断言，不能证明写入顺序。 | 用 fake filesystem/ledger/barrier 运行真实 coordinator。 |
| P1-51 | 没有 Ready-before-ledger-commit 的 crash-point regression。 | 在两次 atomic write 间 kill，断言 Hub 不得看到可聚焦 Ready。 |
| P1-52 | 没有 host post-construction early-return 的 guard leak regression。 | 注入 plugin/template/scene/layout/focus 错误，检查 close receipt 和 lease。 |
| P1-53 | 没有同 operation 的 concurrent duplicate request 测试。 | 断言单次副作用、同一 terminal receipt、不同 payload conflict。 |
| P1-54 | 没有 Normal/Safe/Recovery trust/capability matrix 的执行隔离测试。 | 证明 DLL/script/scene restore 在未批准 profile 中绝不 materialize。 |
| P1-55 | 没有 activation ledger、session record、manifest digest 不一致的恢复测试。 | 每一种 split-brain 都进入确定性 quarantine/action。 |
| P1-56 | 没有真实 Hub+Editor 双进程 first-present/focus/close E2E。 | Windows process harness 覆盖 ack、foreground denied 和 stale generation。 |
| P1-57 | 没有 residual heartbeat pause、PID reuse、clock rollback 和 takeover 权限测试。 | 控制时钟与 OS lease，覆盖 Active/Residual/RecoveryRequired。 |
| P1-58 | 没有 corrupt/oversized mailbox、ledger、recent、lock record 的 bounded parser 测试。 | 统一 bytes/depth budget、quarantine、retention 和 operator diagnostics。 |
| P1-59 | 没有 network/removable filesystem 的 lock atomicity qualification。 | 明确支持矩阵，unsupported storage fail-close。 |
| P1-60 | 没有 p50/p95/p99 startup、first-present、focus、close 的 CPU/RSS/I/O 基线。 | 固定大型项目、BuildSet、插件数量与硬件，记录失败成本和抖动。 |

## 10. P2：长期工程能力

| ID | 能力 | 目标 |
|---|---|---|
| P2-01 | Project Session Inspector | 展示 operation、identity、preflight、trust、admission、activation、ready、focus、close receipt。 |
| P2-02 | Launch decision history | 记录 compatibility、migration、backup/copy、trust 与 recovery 选择。 |
| P2-03 | Activation timeline | 对 ledger effect、plugin mount、document、window、first-present 给出分布式 trace。 |
| P2-04 | Signed project catalog | 管理项目/插件签名、密钥轮换、撤销与离线策略。 |
| P2-05 | Per-project trust policy | 保存脚本、native、network、scene restore 的独立授权和过期规则。 |
| P2-06 | Launch provenance | UI 显示 Hub/CLI/Welcome/Recent、principal、operation 和 retry lineage。 |
| P2-07 | Recovery assistant | 按 effect 提供 retry、rollback、restore、open copy、safe-mode 和 quarantine 操作。 |
| P2-08 | Terminal receipt index | 保留 bounded close/failed receipts，支持 support bundle 与审计查询。 |
| P2-09 | Focus topology inspector | 展示 project/session/window/generation 与 foreground capability。 |
| P2-10 | Recent workspace model | recent 支持 workspace/tag/pin/alias，identity 不被 display path 污染。 |
| P2-11 | Multi-window project policy | 明确一个进程多窗口、一个项目多进程和 read-only viewer 的长期模型。 |
| P2-12 | Privacy-aware diagnostics | support bundle 自动脱敏绝对路径、principal 和签名细节。 |
| P2-13 | Crash-point simulator | 可重复模拟每个 file replace、plugin mount、present、focus 和 close 断点。 |
| P2-14 | Startup performance budget | 对大项目建立文件数、插件数、首次 shader/asset warmup 和交互首帧预算。 |
| P2-15 | Compatibility lab | 用多个 engine BuildSet、旧 manifest、坏 migration 和撤销签名进行持续资格验证。 |

## 11. 资格门（实现前必须逐项通过）

| Gate | 通过条件 |
|---|---|
| G-01 | 同一 operation id 的重复请求只产生一个 activation ledger 和一个 terminal receipt。 |
| G-02 | 不同 payload 复用 operation id 返回 deterministic conflict，不执行副作用。 |
| G-03 | `Ready` 记录永远携带已提交且 digest 匹配的 Session effect。 |
| G-04 | 任意 crash point 都能从 journal/ledger 恢复到 Ready、Closed 或 RecoveryRequired。 |
| G-05 | Hub 只对 committed Ready + first-present receipt 返回成功。 |
| G-06 | first-present 之后 presenter/GPU 失败能发布 degraded/revoked milestone。 |
| G-07 | focus Queued、Delivered、Focused、Denied、Stale 在类型和 UI 上可区分。 |
| G-08 | project switch 会终止旧 watcher、旧 generation 和旧 focus inbox。 |
| G-09 | close 期间新 focus/open 请求得到 Closing/RetryAfter，不会触碰旧 runtime。 |
| G-10 | host 构造后每个 early-return 都有 shutdown receipt 或 RecoveryRequired evidence。 |
| G-11 | Drop 不承担无限等待；bounded shutdown 超时会保留 quarantine 诊断。 |
| G-12 | runtime project、plugin、document、settings、logs 按逆依赖顺序关闭。 |
| G-13 | Normal profile 执行 project-derived code 前必须有签名/trust/approval receipt。 |
| G-14 | Safe/Recovery profile 在 materialize 前阻断脚本、DLL 和 scene restore。 |
| G-15 | trust、BuildSet、manifest、plugin catalog 任一 digest 变化都会使 approval 失效。 |
| G-16 | preflight 不创建 writer、加载 DLL、运行脚本或写入最终项目目录。 |
| G-17 | migration/copy/backup/convert/cancel 的决定可审计且可重放。 |
| G-18 | network/removable filesystem 不满足锁语义时 fail-close。 |
| G-19 | PID reuse 与 heartbeat clock rollback 不会误判为可接管。 |
| G-20 | residual takeover 需要 operator policy、terminal ledger 和 matching identity。 |
| G-21 | corrupt/oversized lock、ledger、mailbox、recent 输入都 bounded、quarantine、可诊断。 |
| G-22 | public Hub failure message 不泄漏绝对路径、内部 stack 或 secret。 |
| G-23 | ready/failed/focus mailbox 有 nonce、expiry、ack、retention 和 replay protection。 |
| G-24 | focus directory ACL/owner 与 Hub principal 在启动时完成校验。 |
| G-25 | recent corruption 不阻断项目 activation，能异步重建 projection。 |
| G-26 | recent 排序基于 monotonic revision，不依赖 wall-clock 顺序。 |
| G-27 | create/open/recent/CLI/Hub 都只提交同一个 launch intent。 |
| G-28 | Welcome 双击、Hub retry 和 process restart 均有真实 integration coverage。 |
| G-29 | activation ledger/session record/manifest/plugin/document 的 split-brain 都有 deterministic action。 |
| G-30 | dirty documents、Play、asset jobs 的 typed veto/drain 在 project switch 前收敛。 |
| G-31 | first-present、focus、close 有 Windows 双进程 E2E，而非源码 shape test。 |
| G-32 | crash-point harness 覆盖每个 atomic replace 与跨线程 callback 边界。 |
| G-33 | P0 fault injection 证明 guard 在未知状态不会被释放。 |
| G-34 | 真实大型项目 p50/p95/p99 启动/首帧/聚焦/关闭预算记录入 CI artifact。 |
| G-35 | plugin 数量、manifest 大小、文件数增长不会造成无限 UI/Hub wait。 |
| G-36 | 所有 phase 都有 cancellation、deadline、retry-after 和 terminal reason。 |
| G-37 | session generation 跨 document、plugin、focus、window、Hub receipt 一致。 |
| G-38 | shutdown 失败能生成脱敏 support bundle，且不会静默删除 recovery evidence。 |
| G-39 | 重启后旧 terminal receipt 不会被误认为新 operation 的成功。 |
| G-40 | 重新计算本报告 277-file fingerprint，并由独立 review 确认调用顺序和 owner 边界。 |

## 12. 建议实施顺序

1. 先实现 `ProjectActivationReceiptV2`、operation journal 和 Ready/ledger 单一 commit fence；这是 P0-01、P0-02、P0-04 的共同根。
2. 再实现 trust/capability admission，把 Normal 也纳入 signed/approved composition；不得先加载 project-derived code 再补 UI 提示。
3. 将 `RetainedEditorHost` 的所有构造后错误接入 shutdown coordinator，验证 P0-05 的 lease、runtime、plugin、document 和 focus 清理。
4. 以 crash-point、duplicate-request、trust matrix、双进程 Hub/Editor E2E 作为门禁；源码字符串测试只能保留为 schema guard，不能作为产品通过条件。
5. 最后补 recent、diagnostics、performance 和 P2 工具；它们是 projection/observability，不能反过来决定 activation correctness。

## 13. 复核结论

当前 Editor 项目启动链已经从临时拼装进入“有明确模块、有局部状态机、有局部 durable recovery”的中间阶段，但还没有 Unreal/Godot 级别的项目会话事务。最危险的不是缺少更多功能，而是 `Ready`、activation ledger、trust approval、Hub ack 和 host shutdown 之间仍有可观测但不可证明的一致性空洞。后续实现应先填上五项 P0，并在每个跨文件写入点保留可重放证据；在此之前，不应把“能打开项目、能显示首帧、能把窗口置前”描述为工程级完成。
