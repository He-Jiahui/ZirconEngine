---
related_code:
  - zircon_runtime_interface/src/hub_protocol
  - zircon_runtime_interface/src/project/manifest_summary
  - zircon_runtime_interface/src/project/session_lock
  - zircon_runtime_interface/src/project/project_name
  - zircon_runtime_interface/src/project/rel_path
  - zircon_runtime_interface/src/project/mod.rs
  - zircon_editor/src/core/hub_link
  - zircon_editor/src/core/recovery/session_guard.rs
  - zircon_editor/src/core/recovery/session_guard
  - zircon_editor/src/core/project
  - zircon_editor/src/ui/host/editor_manager_project_session.rs
  - zircon_editor/src/ui/host/startup
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/run_config.rs
  - zircon_hub/src/process/editor_focus
  - zircon_hub/src/process/editor_handshake
  - zircon_hub/src/process/editor_launch.rs
  - zircon_hub/src/projects
  - zircon_hub/src/tauri_app/runtime_state/editor_launch_actions.rs
  - zircon_app/src/entry/cli/launch_args.rs
  - zircon_runtime/src/asset/project/manifest
tests:
  - zircon_runtime_interface/src/hub_protocol/tests.rs
  - zircon_runtime_interface/src/project/session_lock/tests.rs
  - zircon_runtime_interface/src/project/tests/manifest_summary.rs
  - zircon_hub/src/process/editor_handshake/tests.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime_interface/02-serialization-reflection-resource-project-world-sync-public-dto-contract-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/51-editor-project-startup-open-create-authority-hub-handshake-session-guard-focus-recent-recovery-product-integration-review.md
  - docs/plans/optimize/zircon_hub/01-project-engine-build-editor-launch-process-persistence-delivery-review.md
  - docs/plans/optimize/zircon_tooling/27-version-domain-schema-compatibility-support-window-migration-deprecation-upgrade-downgrade-review.md
  - docs/plans/optimize/zircon_tooling/37-transaction-atomicity-prepare-commit-publish-rollback-compensation-idempotency-crash-recovery-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Public/ProjectDescriptor.h
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/ProjectDescriptor.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Public/Interfaces/IProjectManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/ProjectManager.cpp
  - dev/UnrealEngine/Engine/Source/Editor/GameProjectGeneration/Private/SProjectBrowser.cpp
  - dev/UnrealEngine/Engine/Source/Programs/Shared/EpicGames.Core/SingleInstanceMutex.cs
  - dev/godot/editor/project_manager/project_list.h
  - dev/godot/editor/project_manager/project_list.cpp
  - dev/godot/editor/project_manager/project_manager.cpp
  - dev/godot/editor/project_manager/project_dialog.cpp
  - dev/Fyrox/project-manager/src/project.rs
  - dev/Fyrox/project-manager/src/manager.rs
  - dev/Fyrox/project-manager/src/settings.rs
  - dev/bevy/crates/bevy_app/src/app.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Core/Migration/IVersionable.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/package.json
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Editor/AssetProcessors/AssetVersion.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Editor/AssetVersion.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 06 · Project Manifest / Session Lock / Hub Protocol / Recent Project 跨进程合同与产品集成审查

## 1. 结论

`zircon_runtime_interface` 已经提供可保留的跨进程底座：manifest summary 能识别旧格式并拒绝未来格式，session token 使用 canonical UUID v4，Hub mailbox/focus/recent DTO 使用严格协议标记，Editor 与 Hub 采用同目录临时文件加 rename 发布，SessionGuard 把 OS 独占 lease 与恢复 record 分开，Editor 又在 host、focus watcher 建立后才发送 startup `Ready`。这些实现比“直接 spawn 后假定成功”前进了一步。

但当前公共接口仍把五种不同承诺混在少量字符串、文件和枚举中：summary 解析成功只证明一组显示字段可读，却被 Hub 投影成项目 `Valid`；lock 存在只证明某进程取得 lease，却被消费方解释成 Editor `Ready`；mailbox `Ready` 没有绑定 request、BuildSet、project identity、instance generation 或首帧；focus 文件 rename 成功没有证明目标进程消费；recent JSON 写入失败反而能让已经可打开的项目 activation 整体失败。协议局部均可能返回 `Ok`，端到端产品结论仍可能错误。

这不是靠再给 `HubEditorOutcomeV1` 加两个可选字段就能闭合的问题。目标必须拆成 data-only `ProjectCompatibilityProbe`、带 epoch 的 `ProjectSessionAdmissionRecord`、自描述 `HubLaunchRequest/EditorStartupReceipt`、有 sequence/ack 的 `FocusRequest`，以及不参与项目 open commit 的 `RecentProjectOperation` 投影。Interface 只拥有有界、版本化、平台中立的 schema/codec；Runtime 拥有完整 manifest load/migration；Editor 拥有 admission/activation/focus receiver；Hub 拥有 build resolver、child supervision 与 recent store；App 只组合 CLI intent 和 host wiring。

本轮没有发现一个尚未由其他报告登记、且应由 Interface06 单独拥有的新 P0。Editor51 的五项 project lifecycle P0、Interface02 的 project identity/persistence/schema问题、Hub01 的 child/process owner和Editor02的heartbeat/recovery继续由原报告 canonical 持有。本文新增 **0 项 P0、56 项 P1、14 项 P2与36个资格门**，专门拥有这些类型如何组成可验证跨进程合同的差距；所有实现均为 `pending`。

## 2. 审查边界、currentness 与证据

### 2.1 冻结源码语料

| 集合 | 文件 / 物理行 / bytes | selected tests | 指纹 / 证据等级 |
|---|---:|---:|---|
| Interface DTO / codec | 43 / 1,806 / 63,065 | 19 | `6648e6dde01b4696bfd40396ae68db1ea738d64f86a1e4f61026999f5fbd02d8`，E3逐文件 |
| Editor direct consumers | 29 / 3,725 / 135,662 | 22 | `242cd88e8dcbe71a350b8d8f93468006f7b05cb43bb674ba68a2de9738050e59`，E3纵向调用链 |
| Hub / App direct consumers | 16 / 3,558 / 122,423 | 40 | `5e09ca4527b3805097a734039bb35feb8947909f18c1b68c787175ac9323a603`，E3纵向调用链 |
| Runtime full manifest | 6 / 309 / 10,799 | 0 | `d797621d7e35c26611ce569eb57d740b1f7a07ac770e1c01de7dd6022e32bc46`，E3 authority对照 |
| 去重合计 | 94 / 9,398 / 331,949 | 81 | `88fd24544ca3fc9029ed258d0e14166c9a9f668f08057914960b239068a806cc` |

`selected tests` 是上述94个Rust文件中的 `#[test]` 属性数，0个 `#[ignore]`。指纹按selected path去重排序，对每个文件取lowercase SHA-256，再以 `forward/slash/path<TAB>hash` 和LF连接、无末尾LF后取总SHA-256。冻结日期为2026-08-19，基线提交为 `25e09a23178000f2e783ce2143cf70a8b118d404`。工作树存在其他Session的未提交修改，因此实施前必须重取语料和调用链，不能把本指纹当作编译通过凭据。

### 2.2 参考语料与适用性

| 参考 | 冻结文件 | 本轮可采用事实 | 明确不外推 |
|---|---:|---|---|
| Unreal | 6 | descriptor version、engine association、project status、Open Copy/Convert In-place/Skip/Cancel、可取消mutex wait | 不复制其大型类层次，也不由UI分支证明Zircon兼容性 |
| Godot | 4 | config/feature compatibility、missing/recovery状态、backup/conversion、Recovery Mode禁用项目扩展 | 不把Godot单进程细节直接当Zircon跨DLL ABI |
| Fyrox | 3 | recent清理、Cargo metadata engine version、child queue/output与Upgrade入口 | 其cwd fallback、缺少原子写和直接`cargo update`不是工程上限 |
| Bevy | 1 | plugin `Adding -> Ready -> Finished/Cleaned` 的组合式Ready语义 | 不把App lifecycle当Project Manager或跨进程协议 |
| Unity Graphics | 4 | package显式版本/依赖与资产migration version map | 本地镜像没有Unity Hub/Editor Project Manager，launch/session/recent为N/A |
| 去重合计 | 18 / 12,678行 / 447,122 bytes | fingerprint `10e80eb62d797b0090c412edb50992c443c2bf2bf9d108e6c6b02bc3198e2923` | E2/E3结构对照，不是性能证据 |

Unreal/Godot共同说明“项目可解析”“可转换”“当前引擎可打开”“恢复模式可打开”和“可正常运行”必须是不同状态。Fyrox只证明较小引擎也会持有child与版本信息；Bevy只提供Ready组合语义；Unity Graphics仅支持schema/package migration论证。任何参考都不能证明Zircon当前性能或表现达到、超过Unreal。

### 2.3 实际协议链

```text
Hub / CLI
  -> --project + --hub-session + --hub-protocol=1
  -> spawn Editor
  -> poll <token>.json for Ready { pid, project } / Failed { reason }

Editor
  -> parse summary / full manifest at different phases
  -> claim OS project lease + write pid/instance/heartbeat record
  -> activate host/plugins/recent/document/UI
  -> create focus watcher
  -> publish Ready

Second Hub / Editor
  -> probe lease
  -> if active, write focus request named by target instance
  -> report FocusedExisting immediately

Recent projection
  -> Hub and Editor each lock/read/merge/truncate/write same global JSON
```

协议中没有共同的 `ProjectId`、`BuildSetId`、`LaunchOperationId`、`AdmissionEpoch` 或 `ProjectSessionGeneration`。session token只存在于mailbox路径，recent又嵌套可直接serde构造的summary，导致每一层都能局部验证自己的文件，却无法证明它与同一个项目、构建、进程创建代次和用户意图相连。

### 2.4 动态证据边界

本轮是review-only，没有修改production/tests，也没有运行Cargo、真实Hub+Editor双进程、进程kill/PID复用、网络文件系统、Windows reparse point、Unix权限、GUI first-present或性能测试。静态调用链足以确认DTO字段、parser边界、阻塞锁、发布/消费顺序和重复owner；崩溃一致性、安全权限、时序与性能仍必须由第9节资格门动态证明。

## 3. 已有可保留基础

1. 保留 `HubSessionToken` canonical UUID v4格式及私有构造/解析，不退回自由字符串。
2. 保留协议marker、`deny_unknown_fields`和原子rename发布的最低防线，但升级为各消息家族独立版本。
3. 保留manifest summary的future-version拒绝与显式migration入口，补兼容probe和完整receipt。
4. 保留Runtime `ProjectManifest`作为完整项目描述符authority；summary只做有声明的partial read model。
5. 保留OS lease与恢复record分离；record不能冒充互斥本身，lease也不能冒充Ready。
6. 保留Editor启动失败也能向Hub发布terminal outcome的思路，扩展为request-bound terminal receipt。
7. 保留focus文件先rename到private claim再消费的基本claim模式，补sequence、ack、expiry和清理。
8. 保留recent registry排序、去重和容量上限，但将其降级为可重建投影，不得阻塞项目打开。
9. 保留Hub在启动前重新读取真实manifest的防线，明确summary `Valid`只表示partial probe可显示。
10. 保留Editor在host/focus watcher建立后才发ready的顺序，进一步要求window和first-present receipt。
11. 保留跨平台lock实现，但所有blocking wait必须受deadline/cancel/telemetry约束。
12. 保留typed error枚举，删除依靠message字符串恢复状态或决定重试的做法。

## 4. 继承P0与canonical owner路由

Interface06不新增P0，也不把以下事实重新计数：

| 既有阻断 | Canonical owner | Interface06责任 |
|---|---|---|
| 项目派生代码在exclusive admission与兼容批准前加载 | Editor51 `E-PROJ-P0-01` | 定义data-only preflight和request/admission DTO，不拥有加载顺序 |
| activation rollback关闭失败仍释放guard | Editor51 `E-PROJ-P0-02` | 定义typed lifecycle/terminal disposition，不拥有rollback执行器 |
| Claimed/Activating被Hub误判Ready | Editor51 `E-PROJ-P0-03` | 定义phase/generation schema，不拥有Editor状态机 |
| focus watcher不重绑且publisher无ack仍成功 | Editor51 `E-PROJ-P0-04` | 定义request/ack schema，不拥有window attention行为 |
| 无Engine/BuildSet/migration/Safe Mode决策 | Editor51 `E-PROJ-P0-05` | 定义compatibility receipt，不拥有产品决策UI |
| project summary局部验证、`library_version`丢失、弱session record、mailbox不绑定token/build/project/nonce、recent无稳定identity/revision/tombstone | Interface02 P1-17/19/21/26/27/51/52 | 本文只定义组合和消费语义；底层schema根因仍回原报告 |
| heartbeat/residual restore与dirty recovery | Editor02 | 只规定record字段和读取合同 |
| engine resolver、child owner、process creation identity、spawn supervision | Hub01 | 只承载typed reference/receipt |

## 5. P1：发布前必须完成的跨进程合同

### 5.1 Manifest summary 与partial probe

#### RI-PROJ-P1-001 · 公共summary可直接serde反序列化，绕过parser invariants

`ProjectManifestSummary`派生`Deserialize`且字段公开，recent registry可嵌套构造没有经过trim、semver、asset-root和migration检查的summary。应让validated summary使用私有字段/validated constructor；wire DTO与domain value分离。

#### RI-PROJ-P1-002 · summary parse成功被命名成项目Valid

Hub validation把summary可读投影为`Valid`，而Runtime完整manifest中的plugins、scripts、export profiles、settings、asset manifest仍可能失败。应返回`PartialProbe { display, deferred_sections, diagnostics }`，只有完整preflight才能发布`Openable`。

#### RI-PROJ-P1-003 · partial reader没有声明被忽略字段和unknown extension策略

summary文档读取后丢弃`asset_roots/settings/library_version`，其他完整字段也未进入结果。应在receipt中列出validated、deferred、ignored和unsupported section，避免调用方把“未检查”解释为“合法”。

#### RI-PROJ-P1-004 · validation结果没有source identity与parser policy

当前返回value或error，没有descriptor canonical path、content hash、size、reader version、policy ID和读取时间。文件在probe与spawn之间变化时无法证明两端基于同一份descriptor。

#### RI-PROJ-P1-005 · migration report只有`migrated_from`

缺少逐步migration ID、输入/输出schema、lossy flag、warning、backup要求和write-back policy。必须让产品层能区分内存兼容读取、可逆升级和需要用户批准的破坏性转换。

#### RI-PROJ-P1-006 · manifest JSON读取没有bytes/depth/string/item预算

summary与full manifest都直接读取/解析完整JSON；项目文件是外部输入。应在分配和serde前执行统一budget，并把budget exceeded作为typed compatibility disposition。

#### RI-PROJ-P1-007 · summary没有canonical descriptor identity

返回值只有显示字段，没有稳定ProjectId、descriptor hash、canonical root或filesystem binding。该根因的稳定身份由Interface02拥有；本文要求所有后续request/receipt引用同一qualified identity。

### 5.2 版本、能力与schema家族

#### RI-PROJ-P1-008 · mailbox、focus、recent共享一个Hub协议版本常量

三个演进速度和兼容风险不同的schema被同一整数耦合；任何单域变化都迫使全协议跳版或诱导宽松兼容。应建立独立SchemaId和envelope version。

#### RI-PROJ-P1-009 · 版本合同只有exact equality

没有minimum reader/writer、compatible range、feature bit或deprecated field窗口。旧Hub/新Editor只能全拒绝，无法给出升级方向或安全降级。

#### RI-PROJ-P1-010 · 启动前没有capability negotiation

Hub不知道Editor支持哪些receipt milestone、focus ack、safe mode或manifest版本。LaunchRequest应携带required/optional capabilities，Editor以accepted/rejected capability set回复。

#### RI-PROJ-P1-011 · 没有schema fingerprint与跨版本golden corpus

当前单元测试只覆盖本crate roundtrip，无法检测字段重命名、默认值或另一版本binary的漂移。每个wire family需要canonical fixtures和reader/writer matrix。

#### RI-PROJ-P1-012 · 多个public DTO允许无效值先进入内存

`Ready`可带PID 0或空/超长project，`Failed` reason无界，session record heartbeat/PID/instance也可构造后再等codec检查。domain constructor必须在创建时完成bounded validation。

#### RI-PROJ-P1-013 · `PathBuf`直接进入JSON wire

PathBuf的Unicode、分隔符、drive letter、UNC、case和非UTF-8语义跨平台不稳定。wire应使用明确platform/encoding的qualified locator；本机PathBuf只在owner adapter中存在。

#### RI-PROJ-P1-014 · 不兼容只有字符串错误，没有typed upgrade direction

缺少`ReaderTooOld`、`WriterTooOld`、`UnsupportedFeature`、`MigrationRequired`、`SafeModeAvailable`等disposition。产品层无法像Unreal/Godot那样提供可审计选择。

### 5.3 Session lease、record 与liveness composition

#### RI-PROJ-P1-015 · OS lease与record不是同一观测receipt

Hub先探测lease、释放探测handle，再读取record；两次观察之间owner可退出或替换。API应返回带observed epoch的combined snapshot，或在持有共享观测lease期间读取record。

#### RI-PROJ-P1-016 · active probe没有lifecycle qualification

`Active`只说明互斥存在，不说明Claimed、Activating、Ready、Closing或RecoveryRequired。应让record phase与lease epoch共同决定typed probe result。

#### RI-PROJ-P1-017 · record没有admission epoch，存在ABA歧义

同一路径的新Editor可复用PID形状和相似instance字符串；旧observer无法区分A退出、B取得lease后的新代。每次claim需要不可复用epoch和process creation identity reference。

#### RI-PROJ-P1-018 · heartbeat refresh没有生产lifecycle owner

接口提供refresh，但聚焦语料中只有tests调用；Editor51/Editor02拥有服务实现。Interface需规定refresh monotonicity、allowed phase、deadline、late refresh和terminal stop语义。

#### RI-PROJ-P1-019 · liveness evidence没有clock domain与observation time

单个Unix milliseconds无法表达producer clock、observer time、allowed skew或suspend/resume。record应携带monotonic sequence，wall time只用于诊断。

#### RI-PROJ-P1-020 · record codec无输入预算和完整性envelope

`read_to_string`无界，ad-hoc `key=value`没有length/checksum或partial-write classification。即使原子写也需bounded decode、corrupt/truncated/unsupported typed状态。

#### RI-PROJ-P1-021 · session terminal与record清理没有receipt

删除record无法证明owner已关闭project writers，保留record也无法区分正常Closing与崩溃Residual。需要terminal disposition、cleanup sequence和recovery handoff ID。

### 5.4 Hub launch mailbox

#### RI-PROJ-P1-022 · request与response没有独立envelope

CLI参数隐式构成request，JSON文件只含response outcome；协议无法审计accepted request snapshot。应序列化LaunchRequest并由StartupReceipt引用其OperationId与hash。

#### RI-PROJ-P1-023 · `Ready`是单一终态而不是milestone receipt

无法区分ProcessStarted、PreflightApproved、AdmissionClaimed、HostActivated、WindowCreated、FirstPresented和Interactive。Hub应只对明确required milestone作成功结论。

#### RI-PROJ-P1-024 · 没有Pending、Progress、RetryAfter与取消结果

固定poll期间Hub只能等待或超时，用户看不到迁移、shader warmup、插件加载或恢复状态，也不能协商取消后terminal cleanup。

#### RI-PROJ-P1-025 · 10秒timeout与250毫秒poll是未命名产品常量

没有按启动类型、机器profile、recovery/migration工作量或deadline预算配置；也没有jitter/backoff。应由request deadline和policy ID驱动。

#### RI-PROJ-P1-026 · valid mailbox没有claim/ack/retention语义

Hub读取后不删除或确认，旧文件可被重读；Editor不知道Hub是否消费。需要single-consumer claim、ack或有TTL的immutable receipt log。

#### RI-PROJ-P1-027 · mailbox reader无bytes/depth/string/path预算

token路径虽canonical，文件内容仍是外部跨进程输入。读取必须在分配前限制大小，并限制reason/project等字段。

#### RI-PROJ-P1-028 · Editor与Hub各自维护mailbox path/atomic write细节

相同schema的path construction、temp naming、write/rename/error映射分散在consumer crates。Interface应提供中立codec/path policy，filesystem owner adapter只处理权限和平台I/O。

### 5.5 Focus request / acknowledgement

#### RI-PROJ-P1-029 · 每个target只有一个固定focus mailbox

并发publisher会互相覆盖或争用同一路径，无法保留每个用户意图。应以target generation + request sequence/OperationId形成append/queue identity。

#### RI-PROJ-P1-030 · FocusSignal没有ack schema

Hub只能证明publish，不能证明claim、generation match、window activation或用户可见attention。需要`Accepted/Rejected/Stale/Unavailable/Completed` typed ack。

#### RI-PROJ-P1-031 · watcher返回session token但调用方丢弃

Editor消费后只触发attention，不把request identity传播到window result或ack，无法关联Hub action history。

#### RI-PROJ-P1-032 · malformed/mismatch claim文件没有回收策略

rename到private claim后若parse或target校验失败会留下孤儿。需要quarantine、bounded diagnostics、TTL和安全清理owner。

#### RI-PROJ-P1-033 · request没有expiry/deadline

旧focus文件可能在窗口重新创建或session代次变化后被消费。receiver必须在claim后验证deadline、project/session generation与operation status。

#### RI-PROJ-P1-034 · publisher可自行创建mailbox目录

目录存在并不证明receiver已bind；应由Ready session发布receiver lease/capability，publisher只向已声明inbox投递。

#### RI-PROJ-P1-035 · focus没有idempotency与duplicate disposition

重试可能重复闪窗/抢前台，丢包又无法安全重试。receiver需按OperationId记忆terminal disposition，并对duplicate返回同一ack。

### 5.6 Recent project projection

#### RI-PROJ-P1-036 · recent嵌套summary绕过validated parser

registry JSON可恢复出格式正确但领域无效的summary；加载后必须验证或只存ProjectId/descriptor locator，再从authority重建display projection。

#### RI-PROJ-P1-037 · 同毫秒冲突用manifest name字典序静默择胜

这不是revision或因果顺序，会丢掉同时间内其他字段更新。需要writer ID、monotonic revision和明确conflict resolution。

#### RI-PROJ-P1-038 · 截断为8条发生在缺少tombstone/operation语义的merge上

并发remove/update可被旧writer复活，容量淘汰又不可区分用户删除。Interface02拥有稳定identity/revision/tombstone根因；本文要求投影先合并operations再materialize limit。

#### RI-PROJ-P1-039 · Windows与Unix recent write lock都可无限阻塞

`WaitForSingleObject(INFINITE)`和blocking flock没有deadline/cancel/owner diagnostics。UI或activation线程可永久挂起。

#### RI-PROJ-P1-040 · Hub与Editor实现两套registry load/lock/write

两端temp naming、atomic writer、错误处理和恢复策略已经分叉。必须有一个shared storage service或同一transaction adapter contract。

#### RI-PROJ-P1-041 · registry损坏直接硬失败

没有last-known-good、quarantine、repair或从project descriptors重建。recent是衍生数据，损坏不应阻止Hub或Editor启动。

#### RI-PROJ-P1-042 · recent写失败会回滚有效project activation

`complete_project_open`把全局历史投影作为项目open commit步骤。应在project session commit后异步/可重试更新，失败只产生非阻断diagnostic。

### 5.7 Owner、storage 与transaction边界

#### RI-PROJ-P1-043 · wire locator与本机filesystem identity没有adapter边界

raw path同时承担显示、序列化、锁key和项目identity。应由owner把`ProjectLocator`解析为canonical `ProjectIdentityBinding`，wire不直接承诺OS等价。

#### RI-PROJ-P1-044 · shared storage没有principal/profile/build namespace

固定用户目录下的recent/mailbox可能混合不同安装、channel、用户profile或测试实例。storage root必须由qualified Product/Profile/BuildSet namespace派生。

#### RI-PROJ-P1-045 · Interface类型开始吸收业务存储算法

Interface应拥有schema、validation和codec，不应成为Hub/Editor业务store或lifecycle owner。lock acquisition、repair、retention和writeback属于明确service owner。

#### RI-PROJ-P1-046 · atomic rename没有durability level

成功只说明namespace替换，未声明file flush、directory flush、power-loss保证或network filesystem支持。每个receipt必须标注`Published`、`DurableLocal`等可验证等级。

#### RI-PROJ-P1-047 · handshake/focus/recent没有统一crash-point模型

temp write、flush、rename、read、claim、ack、cleanup各阶段失败的可见状态未定义。Tooling37拥有通用taxonomy，本文需要三种协议的具体状态映射。

#### RI-PROJ-P1-048 · protocol文件没有retention/quota owner

过期handshake、focus claim、quarantine和temp文件可无限累积；清理又可能误删活跃operation。需要namespace manifest、lease-aware TTL和quota。

#### RI-PROJ-P1-049 · recent、session与handshake没有共同operation lineage

Hub action history、Editor session、record与recent entry无法串成一条审计链。必须贯穿LaunchOperationId，但不能让Interface拥有业务history。

### 5.8 安全、观测与验证

#### RI-PROJ-P1-050 · mailbox目录缺少明确访问控制合同

协议依赖本地文件系统，却未声明owner-only权限、symlink/reparse-point拒绝、父目录验证或跨用户攻击模型。token不可预测不能替代ACL。

#### RI-PROJ-P1-051 · token与项目路径没有sensitivity/redaction策略

错误和diagnostic可能把完整session token、用户路径或project name进入日志/history。schema字段需要分类和统一redaction。

#### RI-PROJ-P1-052 · 没有跨进程correlation context

日志、timeout、focus和recent conflict无法按operation/session/build聚合。receipt应携带有界correlation IDs与producer identity。

#### RI-PROJ-P1-053 · 协议没有指标与失败census

缺少startup phase latency、timeout原因、stale mailbox、focus ack latency、recent lock wait/corruption/repair和session ABA检测指标。

#### RI-PROJ-P1-054 · 测试没有crash-point与真实并发矩阵

当前以shape/roundtrip/happy path为主；需覆盖双Hub、双Editor、publisher race、reader crash、writer crash、kill-after-rename、PID复用和suspend/resume。

#### RI-PROJ-P1-055 · parser/codec没有fuzz与资源预算测试

manifest JSON、session record、mailbox、focus和recent都需要arbitrary bytes、deep JSON、oversize field、duplicate key、Unicode/path边界与OOM隔离。

#### RI-PROJ-P1-056 · 没有跨版本binary compatibility harness

同一source tree单元测试不能证明N/N-1 Hub/Editor或升级/降级行为。需要保留旧reader/writer artifacts、golden corpus和明确support window。

## 6. P2：主重构中一并收敛

### RI-PROJ-P2-001 · `V1`同时出现在类型名、常量和serde helper中

改为SchemaId/catalog生成，避免手工三处同步。

### RI-PROJ-P2-002 · path helper分散在`focus_signal_path`与Hub mailbox模块

统一命名规则和validated component helper，但不把filesystem I/O塞入DTO crate。

### RI-PROJ-P2-003 · session instance validator只接受digits/hyphen却没有domain type名称

与process/session/operation ID容易混淆；改为qualified newtype。

### RI-PROJ-P2-004 · error display包含可变自由文本

保留typed code、bounded detail和source chain，UI本地化不得解析英文句子。

### RI-PROJ-P2-005 · `ProjectManifestSummary`字段顺序被当作隐式wire稳定性

canonical serialization应由schema/fixture保证，不依赖Rust声明顺序偶然稳定。

### RI-PROJ-P2-006 · recent常量容量8没有policy ID

把UI显示上限、存储retention和merge window分开。

### RI-PROJ-P2-007 · temp文件命名策略在Hub/Editor不一致

统一collision、owner、cleanup和diagnostic格式。

### RI-PROJ-P2-008 · platform cfg lock代码缺少同形状测试接口

抽出最小OS adapter contract，使Windows/Unix运行相同的行为测试。

### RI-PROJ-P2-009 · public re-export面没有区分wire DTO与validated value

按`wire`、`validated`、`codec`组织，减少调用方误用。

### RI-PROJ-P2-010 · protocol field没有长度/单位文档

PID、时间、reason、path和name需明确范围、单位与canonical form。

### RI-PROJ-P2-011 · tests重复手写JSON和filesystem fixture

建立versioned fixture builder与corpus manifest，避免测试本身漂移。

### RI-PROJ-P2-012 · protocol diagnostics没有稳定code catalog

为operator、telemetry和本地化提供稳定code，message仅作detail。

### RI-PROJ-P2-013 · storage root API把环境变量fallback细节暴露给调用方

通过qualified profile storage provider返回root或typed unavailable；Interface02继续拥有相对cwd fallback根因。

### RI-PROJ-P2-014 · crate/module文档没有陈述Interface非业务owner原则

在public docs和architecture tests中固定“schema/codec only”边界。

## 7. 目标架构

### 7.1 中立Interface类型

```text
ProjectDescriptorEnvelopeVn
  -> ProjectCompatibilityProbe
       ProjectIdentityRef
       descriptor_hash / reader_policy / validated_sections
       compatibility disposition / migration receipt

HubLaunchRequestVn
  -> LaunchOperationId + ProjectIdentityRef + BuildSetId
  -> required capabilities + deadline + safe/recovery policy

ProjectSessionAdmissionRecordVn
  -> AdmissionEpoch + ProcessCreationIdentityRef
  -> Claimed | Activating | Ready | Closing | RecoveryRequired
  -> heartbeat sequence + ProjectSessionGeneration

EditorStartupReceiptVn
  -> request hash + operation/admission/session/build identity
  -> milestone + terminal disposition + diagnostics refs

FocusRequestVn -> FocusAckVn
  -> target generation + sequence + deadline + idempotency disposition

RecentProjectOperationVn
  -> ProjectId + writer/revision + upsert/tombstone
  -> materialized RecentProjectProjection (derived, rebuildable)
```

所有wire payload必须有独立schema family、bounded decoder、canonical serializer、unknown-field policy、reader/writer window、fixture corpus和sensitivity metadata。Validated value不可由裸serde直接构造；filesystem path、process handle、window attention、child supervision、migration execution和storage transaction都留在owner crate。

### 7.2 Owner矩阵

| Owner | 唯一职责 | 不得拥有 |
|---|---|---|
| `zircon_runtime_interface` | schema、validated value、bounded codec、compatibility/protocol disposition | project open、spawn、lock service、window、recent store |
| `zircon_runtime` | full manifest authority、migration execution、project content validation | Hub policy、Editor UX、cross-process child owner |
| `zircon_editor` | preflight消费、session admission/activation、heartbeat、focus receiver、ready milestone | engine installation resolver、global Hub history |
| `zircon_hub` | engine/BuildSet resolver、child supervision、request issuer、ack consumer、recent projection store | Runtime manifest truth、Editor activation内部步骤 |
| `zircon_app` | CLI/entry intent解析、host composition、typed request wiring | 第二套schema、session store或liveness policy |

### 7.3 端到端状态

```text
IntentAccepted
  -> DescriptorProbed(partial, data-only)
  -> CompatibilityDecided(open / migrate / safe / reject)
  -> AdmissionClaimed(epoch)
  -> Activating(generation)
  -> WindowCreated
  -> FirstPresented
  -> ReadyCommitted
  -> FocusRequest(sequence) -> FocusAck(completed/rejected)
  -> Closing -> Closed | RecoveryRequired

RecentProjectOperation is emitted after ReadyCommitted and never gates it.
```

## 8. 重构里程碑

### M0 · Schema 与owner freeze

- 固定六个schema family、ID、support window、budget和sensitivity；
- 把Interface02/Editor51/Hub01/Editor02 existing findings映射为单一owner；
- 建立94文件currentness清单与cross-version fixture corpus。

### M1 · Manifest preflight

- 分离wire document、validated summary与full compatibility receipt；
- 引入descriptor hash、ProjectIdentityRef、validated/deferred sections；
- 对Unreal/Godot式open/migrate/copy/safe/reject disposition建立数据合同。

### M2 · Admission record 与startup receipt

- SessionGuard record升级phase/epoch/process creation/build/project/session generation；
- LaunchRequest/StartupReceipt自绑定，建立milestone/progress/cancel/terminal语义；
- Editor51完成真正的preflight->admission->activation->first-present commit。

### M3 · Focus request/ack

- generation-qualified inbox、sequence、deadline、claim、ack、dedupe、cleanup；
- watcher随session commit/rebind/unbind；
- Hub只在Completed ack后报告FocusedExisting。

### M4 · Recent operation projection

- 稳定ProjectId、revision/writer/tombstone、conflict与bounded lock；
- 单一store owner、last-known-good/quarantine/rebuild；
- recent失败从project activation commit中移出。

### M5 · 安全、故障与兼容资格

- ACL/reparse/symlink、quota/retention/redaction；
- crash-point、PID reuse、双进程race、N/N-1 binary matrix；
- telemetry与operator repair receipt进入BuildSet-bound evidence。

## 9. 验收门

| Gate | 验收内容 |
|---|---|
| G01 | 94个selected source path与fingerprint可重建，source drift自动要求recheck |
| G02 | 18个reference file的snapshot/applicability可重建，不引用不存在的Unity Hub源码 |
| G03 | summary wire DTO无法直接变成validated value，所有constructor执行bounded invariants |
| G04 | partial probe明确列出validated/deferred/ignored/unsupported sections |
| G05 | probe和full open绑定同一descriptor hash与ProjectIdentityRef |
| G06 | manifest bytes/depth/items/string/path预算在分配与serde前执行 |
| G07 | migration receipt含step、input/output schema、lossy、backup与writeback policy |
| G08 | Open/Copy/Migrate/Safe/Reject disposition通过旧、新、未来和unsupported feature fixtures |
| G09 | mailbox/focus/recent各有独立SchemaId、reader/writer window和golden corpus |
| G10 | N/N-1及允许的N+1 reader/writer matrix由真实旧binary artifact执行 |
| G11 | LaunchRequest与StartupReceipt绑定operation、project、BuildSet、deadline和nonce/hash |
| G12 | startup milestones至少区分preflight、admission、activation、window、first-present与interactive |
| G13 | timeout/cancel产生terminal receipt，Hub不把超时进程遗留为成功 |
| G14 | mailbox read/claim/ack/cleanup在duplicate、stale、truncated与oversize输入下确定 |
| G15 | OS lease与record观测返回同一AdmissionEpoch，不存在probe/read TOCTOU成功结论 |
| G16 | process identity包含creation epoch，PID复用不能复活旧session |
| G17 | heartbeat sequence单调，suspend/skew/late refresh有typed disposition |
| G18 | Claimed/Activating/Closing不会被Hub解释为Ready |
| G19 | Ready只在Editor51定义的commit与first-present证据后发布 |
| G20 | focus request按target generation和sequence排队，不发生last-writer overwrite |
| G21 | Hub只有收到Completed focus ack才报告FocusedExisting |
| G22 | stale/duplicate/expired/mismatch focus均返回稳定ack且不抢前台 |
| G23 | malformed claim进入bounded quarantine并由明确owner清理 |
| G24 | recent registry使用ProjectId、writer/revision和tombstone，双writer不静默丢更新 |
| G25 | recent锁等待有deadline/cancel/owner diagnostics，UI与activation线程不无限阻塞 |
| G26 | recent损坏可quarantine/last-good/rebuild，Hub与Editor仍可启动 |
| G27 | recent write失败不回滚已成功的project activation |
| G28 | Hub与Editor使用同一store transaction contract，temp/durability/repair语义一致 |
| G29 | storage namespace隔离product/profile/BuildSet/test instance |
| G30 | protocol目录owner-only，symlink/reparse/parent replacement攻击fail-closed |
| G31 | token/path/project字段按sensitivity策略redact，日志无凭据或完整私有路径泄漏 |
| G32 | crash-point覆盖write/flush/rename/read/claim/ack/cleanup的每个边界 |
| G33 | 双Hub、双Editor、PID reuse、kill、suspend、publisher race和replay动态测试通过 |
| G34 | fuzz覆盖arbitrary bytes、deep JSON、duplicate key、Unicode/path和oversize字段 |
| G35 | telemetry可按LaunchOperationId/AdmissionEpoch/SessionGeneration关联所有跨进程阶段 |
| G36 | `git diff --check`、Markdown链接/frontmatter、计数与索引总账验证通过 |

## 10. 依赖与状态

实施顺序必须先接Interface02的ProjectId/schema、Tooling27的version window与Hub01的BuildSet/process identity，再由Editor51落preflight/admission/activation/first-present状态机；focus和recent不能反向定义session truth。Editor02继续拥有heartbeat service和residual recovery，Tooling37提供crash transaction taxonomy，Tooling26提供本地filesystem principal/ACL基线。

本报告为 `review_complete / implementation pending / source_recheck_required`。它不宣称任何production修复、测试通过、性能提升或Unreal等价；只有第9节所有适用门绑定同一source/BuildSet后，才能更新实现状态。
