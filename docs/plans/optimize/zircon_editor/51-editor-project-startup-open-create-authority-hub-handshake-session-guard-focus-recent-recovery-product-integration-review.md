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
  - docs/plans/optimize/zircon_tooling/37-transaction-atomicity-prepare-commit-publish-rollback-compensation-idempotency-crash-recovery-review.md
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
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 51 · Editor Project Startup / Open / Create Authority / Hub Handshake / Session Guard / Focus / Recent / Recovery 产品集成工程化差距

## 1. 结论

Zircon Editor已经具备一批可保留的项目生命周期基础：项目创建会先渲染模板到同级staging目录、校验并保存manifest、备份空目标再rename；项目路径进入`ProjectManager`前会canonicalize；`SessionGuard`能用跨进程锁和PID/heartbeat记录区分Vacant、Active与Residual；Hub launch token、ready/failed mailbox和focus request已经形成最小跨进程协议；Editor activation也有host、log、native plugin、recent、document与UI的顺序和rollback入口。当前代码不是空壳。

但是这些组件没有组成一条从“用户启动意图”到“首帧已呈现且可被Hub确认”的工程级事务。App在`SessionGuard::claim`和兼容性判定之前，已经从项目manifest选择runtime/editor plugin并可加载项目native DLL；独占准入因此不是项目代码执行的前置安全门。准入后若activation失败且`host.close_project()` rollback也失败，外层仍释放guard，当前进程可继续持有已打开的runtime project，而第二个Editor重新取得同一项目锁，形成真实双writer窗口。

跨进程状态又把“锁已取得”误写成“Editor已可交互”。`SessionGuard`没有Claimed/Activating/Ready/Closing状态和committed generation；Hub看到live lease即发送focus并直接记录`FocusedExisting`。RetainedHost只在进程初始已经打开项目时创建一次focus watcher，Welcome启动后再Open、项目切换或关闭都不会bind/rebind/unbind；publisher还能自行创建mailbox目录，所以没有任何接收者时也会返回成功。Hub ready更早于native window创建、presenter建立和第一帧present，后续窗口/GPU失败无法撤回已经发布的Ready。

直接Editor路径只校验`engine_version_req`语法，不与当前engine/BuildSet比较；普通`load()`丢弃manifest migration report，也没有newer/older/unsupported feature、backup、Open Copy、Convert In-place、Cancel或Safe/Recovery Mode决策。用户最近打开的第一个Valid项目还能在启动时自动进入上述链并执行项目派生代码。与Unreal的版本/转换决策和进程切换、Godot的兼容/backup/recovery-mode以及显式启动锁相比，Zircon尚缺项目级preflight、admission、activation commit、ready receipt与recovery policy。

本报告登记 **5项P0、60项P1、15项P2和40个资格门**。Editor02继续拥有dirty/save/autosave与Residual恢复，Editor07拥有Play和跨项目运行态，Editor50/06拥有extension/plugin挂载与卸载，App07拥有create模板及持久事务，Hub01拥有child supervision和Hub launch，Tooling37拥有全局事务taxonomy；Editor51唯一拥有这些能力如何组合成`ProjectLaunchIntent -> Preflight -> Admission -> Activation -> Ready/FirstPresent -> Focus -> Close/Recovery`的产品会话状态机。

## 2. 审查边界、currentness 与证据等级

### 2.1 冻结语料

| 子域 | 文件 / 行 / bytes | selected tests | 证据等级 | 本轮检查重点 |
|---|---:|---:|---|---|
| Editor authority/session | 62 / 11,677 / 406,451 | 126 | E3 | ProjectAuthority、SessionGuard、Hub link、recovery、EditorManager activation/rollback/close |
| Editor product tests | 63 / 6,661 / 252,116 | 74 | E3 | startup/session/Welcome/recent/retained-host contracts与source-shape assertions |
| App composition | 12 / 2,942 / 111,503 | 55 | E3 | manifest到runtime/editor plugin选择、bootstrap顺序与handshake传递 |
| Hub counterpart | 10 / 1,511 / 53,396 | 17 | E3 | editor launch、focus publish、ready polling、action projection |
| Interface/runtime contract | 42 / 3,171 / 107,304 | 37 | E3 | hub protocol、session lock、manifest summary、project open与durable transaction底座 |
| 去重冻结合计 | 189 / 25,962 / 930,770 | 309 | E3 | 当前工作树fingerprint `722174e750c1db79e7b728e6ba923e72167557f800fac4de1a3d432f818ddae7` |

`selected tests`是聚焦Rust源码中的`#[test]`属性数，0个`#[ignore]`。其中37处`include_str!`、177处`.contains(`和51处`read_to_string`只说明测试大量检查源码/文本形状；`.contains`并不都属于source guard，本文没有把309个属性等同309个真实产品E2E。

指纹按189个selected path去重排序，对每个文件取lowercase SHA-256，再以`forward/slash/path<TAB>hash`和LF连接、无末尾LF后取总SHA-256。冻结日期为2026-08-19，基线提交为`25e09a23178000f2e783ce2143cf70a8b118d404`。

### 2.2 参考语料与适用性

| 参考 | 文件 / 行 / bytes | 证据等级 | 适用范围 |
|---|---:|---:|---|
| Unreal Project Browser / GameProjectUtils / UnrealEd / ProjectDescriptor / Launch | 5 / - / - | E2/E3 | 版本preflight、Open Copy/Convert/Skip/Cancel、restart式项目切换、startup phase |
| Godot Project Manager / ProjectList / Main | 3 / - / - | E2/E3 | config compatibility、backup/conversion、Recovery Mode、lock/PID/focus、spawn Editor |
| Fyrox Project Manager | 3 / - / - | E2 | child ownership、settings/recent清理及较低成熟度反例 |
| Bevy App / ScheduleRunner | 2 / - / - | E2 | runner与plugin Adding/Finished/Cleaned生命周期；不作为Project Manager对照 |
| Unity Graphics | 0 applicable | E0 | 本地`dev/Graphics/Packages`没有Unity Editor/Hub/Project Manager源码，不猜测闭源产品行为 |
| 去重参考合计 | 13 / 29,487 / 1,065,419 | E2/E3 | fingerprint `b8e41fcf56e80abde2ff74c644b7016a141a139ba76c8cd55c9b0b3c25465bf8` |

单项行数没有在表内伪造拆分，合计是本轮实际冻结结果。参考源码用于提取生命周期选择、失败边界与产品语义，不用“参考目录存在”替代Zircon动态资格，也不证明性能或表现超过Unreal。

### 2.3 在途边界与开放handoff

1. 当前MVP session持有若干App/Editor source与tests lease，包含Editor启动、RunConfig和session tests。本轮只读，不修改、不回退；实施前必须重算189文件fingerprint并重做调用链审计。
2. `failure-2026-08-16-editor-host-hub-handshake-config-visibility.md`的`with_hub_handshake`可见性修复已在source，动态回归仍待原owner验证；本报告不把可编译字段视为handshake产品闭环。
3. `failure-2026-07-23-project-session-lock-reuse-for-recovery.md`已经落下SessionGuard、focus/mailbox与测试底座，但Residual takeover/restore产品流程仍未完成，由Editor02及原编号计划继续拥有。
4. `failure-2026-07-18-runtime-preview-play-scene-report-args.md`继续由原owner处理Play/preview参数。本报告只记录project session identity必须能传入该链，不复制其failure。
5. 本轮为review-only，没有运行Cargo、真实Hub+Editor双进程、Windows锁kill-point、GUI第一帧、GPU presenter、网络文件系统或性能基准。确定性调用顺序足以证明五项P0，但所有动态资格仍为pending。

### 2.4 检查方法

按`Hub/CLI/Welcome intent -> canonical project identity -> manifest read/migration -> engine/BuildSet/feature compatibility -> safe/recovery policy -> cross-process admission -> runtime/plugin/log/document activation -> commit ProjectSessionGeneration -> bind focus inbox -> create native window -> first successful present -> Hub ready acknowledgement -> close/switch/recovery`正向阅读，再从Hub success、Welcome status、recent registry、SessionGuard record、runtime ProjectManager和plugin registration逐项反查唯一owner、generation、commit point与failure compensation。

## 3. 必须保留的工程基础

1. 保留`ProjectAuthority`的输入验证、同级staging、manifest roundtrip、目标备份与rename骨架；由App07/Tooling37补durable journal，而不是退回直接写最终目录。
2. 保留canonical descriptor path作为项目物理身份起点，升级为不可混淆的`ProjectIdentity`，不要把lossy display string当identity。
3. 保留`ProjectManager`作为runtime project open/close authority，但它必须参与activation transaction和rollback disposition。
4. 保留`SessionGuard`的OS互斥与PID/heartbeat检测；扩展为带lifecycle/generation/BuildSet的admission lease。
5. 保留Hub launch token一次性消费思路，补OperationId、nonce、expiry、principal和terminal receipt。
6. 保留focus request的原子文件publish/claim基础，升级为有sequence、ack、deadline和retention的inbox。
7. 保留ready/failed mailbox的bounded polling边界，扩展为milestone receipt和ack，不用固定字符串代替状态机。
8. 保留EditorManager显式activation/rollback函数，重构为可恢复staged transaction而不是堆叠更多布尔值。
9. 保留document/log/plugin/UI各自明确的启动步骤，但每步必须返回effect lease和rollback/compensation receipt。
10. 保留recent registry作为衍生投影；它不能成为项目open成功的commit gate，也不能取代canonical project catalog。
11. 保留Welcome中的create/open/recent入口，统一投递同一种`ProjectLaunchIntent`，删除直接变更Manager的第二路径。
12. 保留manifest的version requirement和migration report数据，提升为用户可见、可审计的preflight决策。
13. 保留RetainedHost延迟创建native window的架构，但Ready必须晚于窗口与第一帧证据。
14. 保留Unreal/Godot的决策语义而非其大型类层次：兼容、复制、转换、取消、recovery mode和restart策略必须显式。
15. 保留Bevy runner/plugin phase仅作为生命周期对照，不把通用App runner冒充项目管理产品。

## 4. 当前产品断路

```text
Hub / CLI / Welcome
  -> EditorGuiStartupRequest(PathBuf/String; no operation/session generation)
  -> ProjectAuthority.open/create -> ProjectManager is already open
  -> App reads manifest and chooses runtime/editor/native registrations
  -> bootstrap runtime modules and load discovered project native plugins
  -X-> no engine/BuildSet/migration/recovery-mode decision
  -X-> no SessionGuard yet

EditorManager.activate_prepared_project
  -> consume Hub launch token
  -> SessionGuard.claim (record means only lease exists)
  -> host.open_project
  -> logging -> native plugins again -> recent write -> document -> finish UI
  -> install guard in active slot

Failure
  -> rollback host.close_project may fail
  -> outer guard.release always runs
  -> runtime project can remain open without OS exclusion

Hub focus / ready
  -> any live guard => publish focus file => immediately report FocusedExisting
  -X-> no receiver acknowledgement
  -> watcher exists only if project was active at initial RetainedHost construction
  -> Ready published before event loop creates native window / first present
```

这条链没有共同的`ProjectActivationOperationId`、`AdmissionEpoch`、`ProjectSessionGeneration`或`BuildSetId`。因此每个局部API都可能返回`Ok`，用户仍然无法知道当前究竟是“仅解析了manifest”“已执行项目插件”“已拿锁但正在激活”“Editor已显示首帧”“focus已被消费”，也无法在崩溃后用一份receipt恢复或仲裁。

## 5. P0：当前安全、正确性与产品状态断路

### E-PROJ-P0-01 · 项目派生代码在exclusive admission与兼容性批准之前加载

`prepare_editor_startup`先创建或打开`ProjectManager`；App随后从manifest构造runtime/editor plugin registrations并bootstrap。`selected_native_editor_plugin_registration_reports`还能在`EditorManager::admit_project_session`之前调用`load_discovered_native_editor_plugins`。两个Editor可在任何一方取得`SessionGuard`前同时解析并执行同一项目的native/plugin入口；不兼容或不受信任项目也能在版本/feature/BuildSet决策前进入进程。

目标：先以数据专用、不可执行reader建立`ProjectPreflightReceipt`，完成canonical identity、signature/trust、engine/BuildSet、schema/migration、feature/provider、safe/recovery policy；再取得`AdmissionLease(Claimed)`。任何项目派生runtime/editor/native code只能在lease与批准receipt同代后进入prepare，并由Editor50/Plugins01的extension admission继续约束。

### E-PROJ-P0-02 · Activation rollback失败后仍释放guard，可留下无锁的已打开runtime project

guard取得后，`activate_prepared_project`依次打开host与其他子系统。任一步失败会调用rollback；但若`host.close_project()`本身失败，rollback只聚合错误，外层仍无条件执行`guard.release()`。此时当前进程可能继续持有ProjectManager/project资源与可写状态，跨进程锁却已经释放，第二个Editor能取得同一项目并写入。

目标：rollback返回typed effect inventory。只有runtime project、plugin/log/document和writer lease都确认关闭时才允许释放AdmissionLease；失败进入`QuarantinedOpen`/`RecoveryRequired`并保持exclusive fence，或终止隔离进程。不得用错误字符串掩盖“compensation失败且项目仍open”的unknown state。

### E-PROJ-P0-03 · SessionGuard把Claimed/Activating误当Ready，Hub可聚焦一个尚未成功的会话

lock record只有pid、instance ID与heartbeat；`probe`看到live owner即返回Active。lease在activation开始前已写入，Hub/第二Editor因此可在logs/plugins/document/UI尚未完成时把focus request投递给owner并宣布`FocusedExisting`。原activation随后可能失败并退出，已经给调用方的成功结论无法撤销或关联失败。

目标：AdmissionRecord至少有`Claimed -> PreflightApproved -> Activating -> Ready -> Closing -> Closed/RecoveryRequired`、checked epoch、ProjectSessionGeneration、BuildSet与owner instance。只有committed Ready generation可以接受focus并向Hub返回成功；Activating返回typed Pending/RetryAfter，失败发布terminal receipt。

### E-PROJ-P0-04 · Focus watcher是一次性startup局部对象，且协议没有ack

Retained app只在初始`refresh_ui`后按当时active target创建`_hub_focus_watch`。Welcome启动再Open时不会安装；切换项目不会rebind；close后旧watcher仍存活。publisher会创建父目录并把原子rename当成功，Hub和第二Editor均不等待owner读取、窗口前置或generation匹配。结果是“Focus successful”可在无人监听、监听旧项目或窗口不可用时出现。

目标：focus inbox由committed `ProjectSessionLease`拥有，bind/rebind/unbind与session generation同一事务；每个request含RequestId、target generation、deadline与reply endpoint，owner消费后返回Focused/Deferred/Denied/Stale/Unavailable ack。Hub只能按ack形成用户可见终态，目录或文件写成功仅是Queued。

### E-PROJ-P0-05 · 直接Editor启动跳过engine/BuildSet兼容与migration决策，也没有Safe/Recovery Mode

manifest的`engine_version_req`只做语法验证，未与运行Editor的engine/BuildSet比较；普通`load()`抛弃migration report。启动链没有newer/older/unsupported feature判断、Open Copy/Convert In-place/Cancel选择，也没有在崩溃或不可信项目时禁用tool scripts、Editor plugins、native extensions和自动scene restore的Safe/Recovery Mode。recent auto-open还会无交互进入这条路径。

目标：建立无副作用`ProjectPreflight`，输出兼容矩阵、migration plan、backup/copy策略、required providers和风险；任何mutating migration或项目代码执行都需显式决策与OperationId。Recovery Mode必须在composition前生效，默认禁止项目派生代码与自动恢复，直到用户升级信任级别。

## 6. P1：Intent、Identity、Preflight 与 Admission

| ID | 当前差距 | 目标重构 |
|---|---|---|
| E-PROJ-P1-01 | `EditorGuiStartupRequest`只是PathBuf/String enum，无OperationId、schema version或idempotency key。 | 引入versioned `ProjectLaunchIntent`和checked `ProjectActivationOperationId`。 |
| E-PROJ-P1-02 | startup request不携来源、principal、权限、deadline、cancel或recovery intent。 | 记录Hub/CLI/Welcome/recent来源、PrincipalId、policy、deadline与SafeMode。 |
| E-PROJ-P1-03 | create在session admission前已完成目录、manifest与rename提交。 | App07持有创建事务；Editor51要求create receipt先进入preflight/admission再激活，不把创建成功写成Editor ready。 |
| E-PROJ-P1-04 | open在准入前已canonicalize并打开ProjectManager，边界含语义副作用。 | 拆data-only inspect与runtime attach；前者不能注册writer或执行项目代码。 |
| E-PROJ-P1-05 | startup recent默认取第一个Valid并自动打开，缺显式项目风险与recovery policy。 | 用户策略明确允许时才auto-open；崩溃/版本变化默认进入chooser或Safe Mode。 |
| E-PROJ-P1-06 | builtin startup view可在无project时拼出`Opened <descriptor>`英语状态。 | 状态只消费typed activation receipt，无receipt显示未打开/失败。 |
| E-PROJ-P1-07 | `EditorStartupSessionDocument`是UI投影，没有ProjectSessionId或generation。 | document绑定ProjectIdentity、SessionGeneration、BuildSet和activation disposition。 |
| E-PROJ-P1-08 | canonical path跨App/Hub/UI后退化为PathBuf或lossy String。 | 定义qualified `ProjectIdentity`，物理path与display path分离。 |
| E-PROJ-P1-09 | `engine_version_req`只验证表达式语法。 | preflight用当前EngineVersion/BuildSet计算Compatible/Upgrade/Downgrade/Reject。 |
| E-PROJ-P1-10 | `ProjectManifest::load`丢弃migration report。 | 所有产品open保留MigrationAssessment并要求用户/政策决策。 |
| E-PROJ-P1-11 | 没有required feature/provider/plugin与当前composition的兼容receipt。 | 编译`ProjectCompatibilityReceipt`，列出满足、缺失、替代与禁止项。 |
| E-PROJ-P1-12 | Hub launch token消费与SessionGuard claim没有共同事务/receipt。 | token与admission绑定同一OperationId，重复/过期/失败可仲裁。 |
| E-PROJ-P1-13 | Active命中只关联PID，未验证instance generation、user、BuildSet或project manifest generation。 | owner identity加入boot/process instance、principal、BuildSet和manifest digest。 |
| E-PROJ-P1-14 | SessionGuard record未区分read-only、writer、recovery、migration或headless角色。 | admission声明Role/AccessMode并由policy拒绝不兼容组合。 |
| E-PROJ-P1-15 | 本地文件锁被隐式当成所有filesystem部署都可靠。 | 明确local/network/removable policy；不支持的锁语义fail-close并给迁移建议。 |

## 7. P1：Activation、Rollback、Close 与 Session Liveness

| ID | 当前差距 | 目标重构 |
|---|---|---|
| E-PROJ-P1-16 | host/log/plugin/recent/document/UI顺序执行，没有共同prepare/commit receipt。 | `ProjectActivationPlan`为每步返回effect lease、rollback与commit digest。 |
| E-PROJ-P1-17 | recent registry写入是非关键投影，却可使整个activation失败。 | session先commit，recent异步更新并报告独立projection fault。 |
| E-PROJ-P1-18 | plugin rollback与runtime host rollback不是一个原子/可恢复边界。 | 由session coordinator按逆依赖序quiesce/revoke/close并保留partial disposition。 |
| E-PROJ-P1-19 | settings/log cleanup只在部分host close成功路径推进，失败后状态含糊。 | 每个effect有独立terminal state，reconciler重复执行到Closed或RecoveryRequired。 |
| E-PROJ-P1-20 | document session发布不携ProjectSessionGeneration。 | 所有document/dirty/tab identity绑定project generation，旧代事件被拒绝。 |
| E-PROJ-P1-21 | finish UI失败时recent等较早步骤已持久提交。 | UI publish前只写staging/provisional effects；commit后再发布非关键投影。 |
| E-PROJ-P1-22 | project native插件在App pre-admission和EditorManager post-admission可能经历两套load/materialize路径。 | 一个项目composition plan、一个owner generation和一次materialize/mount。 |
| E-PROJ-P1-23 | guard直到全部finish后才进入active slot，期间缺可查询activation owner。 | slot从Claimed开始保存lease/state，外部只读snapshot能看到真实phase。 |
| E-PROJ-P1-24 | activation/rollback错误主要flatten为字符串，丢失effect与可恢复性。 | typed `ProjectActivationFailure`列phase、cause、effects、compensation和operator action。 |
| E-PROJ-P1-25 | 崩溃后没有activation intent/effect ledger重放或仲裁。 | durable journal记录prepare/commit点；restart只做幂等reconcile，不盲重试副作用。 |
| E-PROJ-P1-26 | SessionGuard record缺start time、state、BuildSet、project generation与ready epoch。 | versioned admission record包含完整owner/session/lifecycle字段。 |
| E-PROJ-P1-27 | heartbeat已有API但产品没有持续刷新owner；该实现缺口由Editor02拥有。 | Editor51消费Editor02 heartbeat service，并以missed deadline转Suspect而非立即抢锁。 |
| E-PROJ-P1-28 | lock record读取无明确byte/version/depth预算和unknown-field policy。 | bounded schema reader，oversize/corrupt/quarantine均typed。 |
| E-PROJ-P1-29 | instance ID由PID、wall clock和process-local counter组成，缺boot/persistent random identity。 | 使用不可预测ProcessInstanceId并记录OS creation token防PID复用。 |
| E-PROJ-P1-30 | close先关host/释放guard，再清log/settings/plugin/document，可能暴露半关闭会话。 | `Closing`拒绝新writer/focus，逆序drain全部consumer，最终才释放exclusive lease。 |

## 8. P1：Focus、Handshake、Ready 与 Recent Registry

| ID | 当前差距 | 目标重构 |
|---|---|---|
| E-PROJ-P1-31 | focus是单文件mailbox，新请求覆盖/合并旧请求。 | append/slot-based bounded inbox，RequestId+sequence+dedupe+terminal ack。 |
| E-PROJ-P1-32 | request没有ack、disposition、deadline或caller reply identity。 | owner返回Focused/Deferred/Denied/Stale/Unavailable及可审计时间。 |
| E-PROJ-P1-33 | 项目关闭后startup watcher仍存活并可能处理旧target。 | watcher lease随ProjectSessionGeneration关闭，旧request明确Stale。 |
| E-PROJ-P1-34 | native attention通知错误被丢弃，协议没有健康状态。 | attention callback返回receipt；失败使ack为Deferred/Unavailable而非Focused。 |
| E-PROJ-P1-35 | malformed claimed focus文件可隐藏保留且无quarantine/retention。 | bounded parser、quarantine原因、age/bytes清理与operator diagnostics。 |
| E-PROJ-P1-36 | focus payload读取没有明确最大bytes。 | producer/consumer同一schema budget，超限在分配前拒绝。 |
| E-PROJ-P1-37 | RetainedHost在`ui.run()`前发布Ready，早于native window和presenter。 | Ready至少等待event loop window created；交互ready与first-present分里程碑。 |
| E-PROJ-P1-38 | handshake只含Ready(pid,path)/Failed(reason)，缺instance/session/build/generation。 | versioned `EditorReadyReceipt`携ProjectSessionGeneration、BuildSet、window和milestones。 |
| E-PROJ-P1-39 | failed reason可直接写`error.to_string()`，暴露绝对路径或内部细节。 | typed public error code与脱敏message；详细诊断只进受控日志。 |
| E-PROJ-P1-40 | mailbox无ack、cleanup、retention、nonce或replay protection。 | caller ack后回收；过期/重复token不可被新launch误读。 |
| E-PROJ-P1-41 | Hub固定250ms/10秒poll没有phase progress、deadline negotiation或cancel。 | Hub01持有supervision；协议提供phase stream/heartbeat与caller deadline。 |
| E-PROJ-P1-42 | recent writer在Windows使用INFINITE mutex wait，UI/启动可永久阻塞。 | deadline/cancel和owner diagnostics；hung writer不阻塞project activation。 |
| E-PROJ-P1-43 | recent registry整文件读取，无bytes上限；corruption直接阻断open。 | bounded journal/index，corrupt隔离重建，recent永远只是best-effort projection。 |
| E-PROJ-P1-44 | recent排序用wall-clock timestamp，时钟回拨/并发写无revision/CAS。 | monotonic sequence+registry revision，跨进程compare-and-swap与merge。 |
| E-PROJ-P1-45 | recent固定保留8项且identity/display都可经lossy path字符串。 | typed identity、分页/age/bytes政策；display本地化且不参与去重。 |

## 9. P1：Product Transition、Recovery、Tests 与 Qualification

| ID | 当前差距 | 目标重构 |
|---|---|---|
| E-PROJ-P1-46 | Welcome create/open/recent直接mutate manager后调用`apply_startup_session`，没有统一transition coordinator。 | 所有入口只提交ProjectLaunchIntent并观察同一operation receipt。 |
| E-PROJ-P1-47 | dirty document/Play/project切换缺共同关闭门；父语义由Editor02/07拥有。 | Editor51编排其typed veto/drain receipt后才进入Closing。 |
| E-PROJ-P1-48 | close的runtime、guard、plugin、document和UI成功不是同一终态。 | `ProjectCloseReceipt`列出每个effect，未知状态保持隔离且可reconcile。 |
| E-PROJ-P1-49 | project switch不重建focus watcher、handshake target和session identity。 | switch等价old generation terminal + new generation activation，无就地偷换target。 |
| E-PROJ-P1-50 | recent validity probe与路径I/O可同步发生在startup/UI路径。 | background bounded preflight，UI消费immutable result pages。 |
| E-PROJ-P1-51 | startup/status消息是自由英语并含原始路径。 | typed message key+arguments、locale projection与path redaction policy。 |
| E-PROJ-P1-52 | 没有Safe/Recovery Mode关闭project scripts、Editor plugins、native extensions和scene restore。 | composition前应用最小权限profile，逐项显式升级。 |
| E-PROJ-P1-53 | 没有单进程多项目/多窗口与单项目多进程的正式支持矩阵。 | 明确支持模型并在admission policy中编码，不靠偶然mutex行为。 |
| E-PROJ-P1-54 | 部分测试只检查源码文本、函数名或`.contains`，不能证明调用顺序。 | 产品测试实例化真实App/EditorManager并观察typed phase/effect receipts。 |
| E-PROJ-P1-55 | 缺activation rollback中`host.close_project`失败的fault injection。 | RED测试证明guard不释放且第二Editor无法取得writer lease。 |
| E-PROJ-P1-56 | 缺Welcome-start -> later Open -> Hub focus -> foreground的双进程E2E。 | 真实Windows窗口/进程测试验证bind、ack、前置与generation。 |
| E-PROJ-P1-57 | 缺claim后activation失败时Hub Pending/Failed而非FocusedExisting的竞态测试。 | controllable barrier构造每个phase，断言协议disposition。 |
| E-PROJ-P1-58 | 缺newer/older/unsupported/migration/copy/backup/cancel/SafeMode矩阵。 | 对每个compatibility decision做无项目代码执行的preflight测试。 |
| E-PROJ-P1-59 | 缺kill-point、PID复用、heartbeat暂停、corrupt/oversize mailbox和recent registry测试。 | deterministic process harness覆盖crash/restart/recovery与bounded parsing。 |
| E-PROJ-P1-60 | 没有startup/open/close/focus的CPU、wall、RSS、I/O和first-present分布。 | 固定大型项目和同硬件采样p50/p95/p99、alloc/RSS/I/O及失败成本。 |

## 10. P2：长期工程能力

| ID | 能力 | 目标 |
|---|---|---|
| E-PROJ-P2-01 | Project Session Inspector | 展示preflight、admission、activation、first-present、focus与close receipt。 |
| E-PROJ-P2-02 | Compatibility decision history | 记录copy/convert/skip/recovery选择、BuildSet与migration artifact。 |
| E-PROJ-P2-03 | Pinned/Tagged recent projects | recent支持pin、workspace/tag与stable identity，不改写启动权威。 |
| E-PROJ-P2-04 | Startup phase timeline | 分解manifest、compat、lock、plugin、document、window、first-frame时延。 |
| E-PROJ-P2-05 | Orphan maintenance UI | 检查staging、backup、mailbox、claim、residual lock并提供受控修复。 |
| E-PROJ-P2-06 | Privacy-aware display paths | 支持相对化、别名与support bundle脱敏，identity仍保留canonical digest。 |
| E-PROJ-P2-07 | Project launch provenance | UI可见启动来源、principal、operation、Hub request与SafeMode。 |
| E-PROJ-P2-08 | Session support bundle | 导出脱敏records、receipts、phase timings、failure chain与owner generations。 |
| E-PROJ-P2-09 | Per-project startup policy | 保存可信项目的scene restore、plugin、安全与窗口偏好，带schema migration。 |
| E-PROJ-P2-10 | Multi-window focus routing | 以WindowId/ProjectSessionGeneration选择正确窗口并返回foreground限制。 |
| E-PROJ-P2-11 | Foreground-denied UX | OS拒绝抢前台时给taskbar attention与明确ack，不谎报Focused。 |
| E-PROJ-P2-12 | Accessible transition announcements | screen reader按typed phase/terminal result播报，不朗读原始内部错误。 |
| E-PROJ-P2-13 | Batch project operations | project validation/upgrade批处理使用data-only worker，不在管理器进程加载项目代码。 |
| E-PROJ-P2-14 | Session policy conformance SDK | plugin/provider声明startup依赖、safe-mode行为、rollback与quiescence测试。 |
| E-PROJ-P2-15 | Competitive startup lab | 同项目、同插件、同缓存/冷启动条件比较Unreal/Godot/Fyrox的分阶段成本。 |

## 11. 参考引擎对照与适用边界

| 参考 | 当前源码证据 | Zircon应吸收 | 不应误抄 |
|---|---|---|---|
| Unreal `SProjectBrowser` | Open前读取engine identifier与project status；遇到版本/代码差异给Open Copy、Convert In-place、Skip或Cancel。 | 项目代码执行前完成版本/编译/转换决策并保留用户可恢复选择。 | 不复制Slate和大型继承体系；吸收preflight语义与receipt。 |
| Unreal `GameProjectUtils` / `UnrealEdMisc` | 验证project path/name/file后调用SwitchProject，核心路径通过restart/switch进程切项目。 | 将process restart作为复杂项目切换的正式策略，避免旧project owner残留。 | 不要求所有切换都重启；只有证明热切换完整退休时才允许复用进程。 |
| Unreal `ProjectDescriptor` / Launch | descriptor版本/模块/plugin信息与Launch初始化阶段分离。 | data-only descriptor inspection、compatibility和可观察startup phase。 | Unreal自身复杂度不等于Zircon性能答案。 |
| Godot Project Manager | 检查config版本和unsupported features，提供backup/conversion；恢复模式关闭tool scripts、editor plugins、GDExtensions和scene restore。 | Safe/Recovery Mode必须在项目派生代码composition前生效。 | 不把一个布尔参数当完整信任/权限模型。 |
| Godot ProjectList / Main | 记录recovery lock、process/editor PID与focus；Project Manager spawn Editor。 | 明确manager与Editor进程所有权、startup lock、PID instance和focus协议。 | 单文件锁仍需lifecycle generation与ack，不能原样照搬。 |
| Fyrox Project Manager | 保存`Child`并`try_wait`，关闭时警告子进程；命令队列串行驱动UI。 | Hub必须拥有child/process终态并将launch结果与进程寿命关联。 | settings非原子写、create忽略结果等实现是反例，不是目标质量。 |
| Bevy App / ScheduleRunner | App有明确runner，plugin状态经历Adding、Finished、Cleaned并以AppExit终止。 | composition/runner phase应显式并可观察，cleanup是生命周期的一部分。 | Bevy没有Project Manager/Editor Hub，不用其App API替代项目会话状态机。 |
| Unity Graphics | 本地包只含graphics package/editor tooling，没有Unity Editor/Hub启动源码。 | 记录0 applicable，等待可验证source再对照。 | 不根据闭源产品表象猜内部session/lock/handshake实现。 |

## 12. 目标架构

```text
ProjectLaunchIntent(OperationId, Origin, Principal, RequestedMode, Deadline)
  -> ProjectPreflight (data-only, no project-derived code)
       -> ProjectIdentity + ManifestDigest
       -> Engine/BuildSet/Feature/Provider Compatibility
       -> Migration/Backup/Copy/Cancel Decision
       -> Trust + Safe/Recovery Composition Profile
       -> ProjectPreflightReceipt
  -> AdmissionService.claim(ProjectIdentity, OperationId)
       -> AdmissionLease(state=Claimed, epoch, owner instance)
  -> ProjectActivationCoordinator.prepare
       -> runtime attach -> plugin plan -> log/settings -> document -> UI/window
       -> reversible effect leases + durable intent
  -> atomic session commit
       -> ProjectSessionGeneration(state=Ready)
       -> bind generation-qualified focus inbox
       -> publish native-window-created / first-present milestones
       -> EditorReadyReceipt -> Hub acknowledgement

Close / switch / crash recovery
  -> state=Closing, reject new writer/focus admission
  -> dirty/play decision -> document/UI -> plugin -> log/settings -> runtime detach
  -> terminal effect reconciliation
  -> release AdmissionLease last
  -> Closed or RecoveryRequired receipt
```

核心identity与receipt：

```text
ProjectIdentity              = CanonicalDescriptorIdentity + ProjectGuid + ManifestDigest
ProjectActivationOperationId = OriginInstance + MonotonicOperationSequence + Nonce
AdmissionOwner               = PrincipalId + ProcessInstanceId + BuildSetId
ProjectSessionRef            = ProjectIdentity + AdmissionEpoch + ProjectSessionGeneration
ProjectReadyReceipt          = ProjectSessionRef + ActivationDigest + WindowMilestone
                               + FirstPresentMilestone + FocusInboxBinding + Disposition
```

`RecentProjectRegistry`、Welcome文本和Hub history都只消费这些receipt的投影，不参与session commit。project native/runtime/editor plugins只消费批准后的composition plan；Safe Mode在plan生成时移除项目派生代码，而不是加载后再尝试disable。

## 13. 依赖顺序与重构里程碑

### M0 · Truth Freeze 与 RED Contract

- 冻结五项P0的真实调用顺序、189文件manifest与source fingerprint。
- 加入pre-admission plugin execution、rollback close失败、Activating focus、Welcome late-open无watcher和compat bypass的RED测试。
- 明确Editor02/07/50、App07、Hub01与Tooling37父owner，不以本报告复制其实现。

### M1 · Versioned Intent 与 Data-only Preflight

- 引入ProjectLaunchIntent、ProjectIdentity、OperationId与bounded manifest inspector。
- 建立Engine/BuildSet/feature/provider compatibility和migration decision receipt。
- 在composition前实现Safe/Recovery profile，禁止项目派生代码提前执行。

### M2 · Admission Lifecycle Hard Cut

- SessionGuard升级为Claimed/Activating/Ready/Closing/RecoveryRequired状态机和checked epoch。
- token、owner instance、principal、BuildSet与project generation进入同一admission record。
- 删除“live PID等于ready Editor”的Hub/Editor判断。

### M3 · Transactional Activation 与 Reconciliation

- 每个host/plugin/log/recent/document/UI effect返回lease和terminal disposition。
- rollback失败保持exclusive fence并进入可恢复状态；recent等非关键投影移出commit gate。
- durable intent/effect ledger支持kill-point restart reconciliation。

### M4 · Ready、First Present 与 Focus Ack

- focus watcher随ProjectSessionGeneration bind/rebind/unbind。
- mailbox升级为bounded sequenced inbox和typed ack；Hub只按owner ack报告Focused。
- Ready拆为session committed、window created、first present/interactive等可验证milestone。

### M5 · Recent、Close、Switch 与 Recovery Product Flow

- recent registry变成revisioned、bounded、corruption-tolerant projection。
- close/switch编排Editor02 dirty/recovery、Editor07 Play与Editor50 plugin quiescence。
- RecoveryRequired提供Safe Mode、diagnostics、takeover/restore和operator选择。

### M6 · Multi-process Fault 与 Scale Qualification

- 双Editor、Hub+Editor、PID复用、heartbeat pause、mailbox corruption、kill-point和filesystem策略矩阵。
- 1/100/1k recent与大型manifest/plugin project测bytes、I/O、latency、RSS和contention。
- Windows真实窗口验证foreground、first present、GPU/window failure与ack。

### M7 · Competitive Qualification

- 固定相同项目、插件、缓存、硬件和启动条件，对Unreal/Godot/Fyrox采样cold/warm startup与switch/recovery。
- 报告preflight、plugin、document、window/first-frame、CPU/RSS/I/O的p50/p95/p99与失败成本。
- correctness、security、crash recovery和统计证据通过前，不得宣称性能或表现优于Unreal。

## 14. 验收门

| Gate | 验收内容 |
|---|---|
| E-PROJ-G01 | 所有Hub/CLI/Welcome/recent入口只生成同一种versioned ProjectLaunchIntent |
| E-PROJ-G02 | ProjectIdentity在App/Hub/Editor/Runtime全链typed且display path不参与identity |
| E-PROJ-G03 | preflight阶段执行hook证明0项目派生runtime/editor/native代码 |
| E-PROJ-G04 | engine newer/older/incompatible、BuildSet和unsupported feature产生typed decision |
| E-PROJ-G05 | migration必须有copy/backup/in-place/cancel receipt，失败不改变原项目 |
| E-PROJ-G06 | Safe/Recovery Mode在composition前禁用项目scripts/plugins/native extensions/scene restore |
| E-PROJ-G07 | Hub token与AdmissionLease共享OperationId，duplicate/expired/replay均fail-close |
| E-PROJ-G08 | admission record含owner instance、principal、BuildSet、epoch、session generation与lifecycle state |
| E-PROJ-G09 | Activating会话不能返回FocusedExisting，只返回Pending/RetryAfter或terminal failure |
| E-PROJ-G10 | PID复用、heartbeat pause与Residual都有确定政策且不会自动双writer |
| E-PROJ-G11 | project-native代码只能在PreflightApproved+AdmissionLease后materialize |
| E-PROJ-G12 | 同一project generation只执行一次runtime/editor/native composition plan |
| E-PROJ-G13 | activation每个effect有prepare/commit/rollback/terminal receipt |
| E-PROJ-G14 | recent写失败不回滚已成功的project session |
| E-PROJ-G15 | `host.close_project` rollback失败时guard保持且第二Editor无法取得writer |
| E-PROJ-G16 | crash/restart根据durable intent/effect ledger恢复，不盲重试外部副作用 |
| E-PROJ-G17 | document/log/plugin/UI只接受当前ProjectSessionGeneration |
| E-PROJ-G18 | finish UI失败不会留下可被当作Ready的session或半挂载project plugin |
| E-PROJ-G19 | close先进入Closing、drain全部consumer，最后才释放exclusive lease |
| E-PROJ-G20 | close partial failure形成RecoveryRequired并保留可操作effect inventory |
| E-PROJ-G21 | Welcome启动后later Open会创建当前generation focus watcher |
| E-PROJ-G22 | switch原子退休旧watcher并bind新watcher；旧request返回Stale |
| E-PROJ-G23 | focus request有RequestId、sequence、deadline、target generation和terminal ack |
| E-PROJ-G24 | 无listener、前置失败或OS拒绝foreground时Hub不得报告Focused |
| E-PROJ-G25 | focus/handshake mailbox受bytes/count/age约束并有quarantine/cleanup |
| E-PROJ-G26 | Editor Ready不早于session commit和native window creation |
| E-PROJ-G27 | FirstPresent receipt来自真实present success，GPU/window失败可形成terminal failure |
| E-PROJ-G28 | Ready/Failed公共payload脱敏且详细路径只进入受控诊断 |
| E-PROJ-G29 | Hub拥有child/process terminal state并把ready receipt与同一instance绑定 |
| E-PROJ-G30 | recent registry写有deadline/cancel，不可永久阻塞UI或activation |
| E-PROJ-G31 | recent corrupt/oversize可隔离重建，不阻断canonical project open |
| E-PROJ-G32 | recent merge使用revision/CAS，时钟回拨不丢记录或改变identity |
| E-PROJ-G33 | dirty/Play veto或drain receipt未通过时project close/switch不提交 |
| E-PROJ-G34 | startup/status/diagnostics使用typed message key、本地化和path redaction |
| E-PROJ-G35 | 双Editor与Hub+Editor真实进程测试覆盖claim/activate/focus/close/recovery竞态 |
| E-PROJ-G36 | kill-point覆盖preflight、claim、runtime attach、plugin、document、UI、first-present、close |
| E-PROJ-G37 | local/network/removable filesystem支持矩阵明确且unsupported配置fail-close |
| E-PROJ-G38 | 大型项目报告startup/open/close/focus的CPU、wall、RSS、alloc与I/O分布 |
| E-PROJ-G39 | 与Unreal比较使用同项目、插件、缓存、硬件、质量和统计方法，失败样本不剔除 |
| E-PROJ-G40 | source/reference fingerprints、5/60/15计数、frontmatter、links、LF/BOM/trailing-space与`git diff --check`通过 |

## 15. 与其他报告的唯一 Owner 边界

| 报告 | 继续拥有 | Editor51只拥有 |
|---|---|---|
| Editor02 | dirty/save/autosave/restore、heartbeat调用与Residual takeover语义 | 把其receipt编入project admission/close/recovery状态机 |
| Editor03 | scene/prefab/document项目切换语义 | project generation如何约束scene owner与切换commit |
| Editor06 / Editor50 | plugin discovery/enablement/reload及extension mount/revoke/quiescence | 项目准入前禁止执行、activation/close如何消费其typed receipt |
| Editor07 | Play/PIE/Game View进程与跨项目checkpoint | close/switch前消费Play drain/veto，不复制运行态实现 |
| App01 | executable host、bootstrap、event loop和shutdown owner | 项目preflight/admission/ready milestones与host phase的组合合同 |
| App07 | Renderable Empty模板、create/import/render/export与durable create transaction | create receipt如何进入项目session，不复制staging/backup P0 |
| Hub01 | project/build选择、Child supervision、launch timeout、active delete/focus gate | Editor侧ready/focus ack与session generation合同 |
| Hub04 | command/task/history/message delivery通用控制面 | project-specific launch/focus/ready disposition语义 |
| Runtime manifest/ProjectManager | manifest schema/migration和runtime attach/detach authority | Editor产品preflight、activation transaction与ready state machine |
| Tooling37 | 全局OperationId、prepare/commit/compensation/recovery taxonomy | 当前project lifecycle的具体effect ledger、fault matrix与产品gate |

## 16. 状态与产出记录

| 里程碑 | 状态 | 日期 | 证据 |
|---|---|---|---|
| 189文件静态审查与产品调用链反查 | review_complete | 2026-08-19 | 25,962行、930,770 bytes、309 selected tests、0 ignored；fingerprint `722174e750c1db79e7b728e6ba923e72167557f800fac4de1a3d432f818ddae7` |
| 四家适用参考与Unity Graphics不适用性对照 | review_complete | 2026-08-19 | 13文件、29,487行、1,065,419 bytes；fingerprint `b8e41fcf56e80abde2ff74c644b7016a141a139ba76c8cd55c9b0b3c25465bf8` |
| P0/P1/P2与owner去重 | review_complete | 2026-08-19 | 5 P0 / 60 P1 / 15 P2 / 40 gates |
| Production重构 | pending | - | 本篇不修改production或tests；M0-M7均未实施 |
