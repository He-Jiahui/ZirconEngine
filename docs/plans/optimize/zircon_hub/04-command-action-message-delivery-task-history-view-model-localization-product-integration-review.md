---
related_code:
  - zircon_hub/build.rs
  - zircon_hub/src/error.rs
  - zircon_hub/src/lib.rs
  - zircon_hub/src/main.rs
  - zircon_hub/src/settings/hub_config.rs
  - zircon_hub/src/state
  - zircon_hub/src/tauri_app/action_id.rs
  - zircon_hub/src/tauri_app/action_request.rs
  - zircon_hub/src/tauri_app/commands.rs
  - zircon_hub/src/tauri_app/mod.rs
  - zircon_hub/src/tauri_app/runtime_state.rs
  - zircon_hub/src/tauri_app/runtime_state/action_targets.rs
  - zircon_hub/src/tauri_app/runtime_state/action_tasks.rs
  - zircon_hub/src/tauri_app/runtime_state/learn_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/output_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/quick_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/tests.rs
  - zircon_hub/src/tauri_app/runtime_state/project_actions/tests.rs
  - zircon_hub/src/tauri_app/view_model.rs
  - zircon_hub/src/tauri_app/view_model/action_history.rs
  - zircon_hub/src/tauri_app/view_model/display.rs
  - zircon_hub/src/tauri_app/view_model/localized.rs
  - zircon_hub/src/tauri_app/view_model/new_project.rs
  - zircon_hub/src/tauri_app/view_model/project_templates.rs
  - zircon_hub/src/tauri_app/view_model/quick_actions.rs
  - zircon_hub/src/tauri_app/view_model/source_engines.rs
  - zircon_hub/src/tauri_app/view_model/tests.rs
  - zircon_hub/src/tauri_app/view_model/ui_text.rs
tests:
  - zircon_hub/tests
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_hub/01-project-engine-build-editor-launch-process-persistence-delivery-review.md
  - docs/plans/optimize/zircon_hub/02-web-shell-catalog-settings-team-cloud-accessibility-performance-review.md
  - docs/plans/optimize/zircon_hub/03-marketplace-account-auth-organization-cloud-repository-provider-review.md
  - docs/plans/optimize/zircon_runtime/02-core-runtime-events-tasks-review.md
  - docs/plans/optimize/zircon_runtime/44-process-diagnostic-log-router-filter-record-queue-sink-durability-rotation-crash-multi-session-product-integration-review.md
  - docs/plans/optimize/zircon_editor/48-editor-message-bus-topic-subscription-inbox-retention-admission-dispatch-request-dirty-projection-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_tooling/10-test-architecture-partition-selection-isolation-fixture-flake-results-review.md
  - docs/plans/optimize/zircon_tooling/23-failure-contract-panic-unwind-error-propagation-poison-recovery-result-observability-review.md
  - docs/plans/optimize/zircon_tooling/24-concurrency-locking-atomic-ordering-blocking-thread-lifecycle-backpressure-deadlock-review.md
  - docs/plans/optimize/zircon_tooling/26-security-principal-credential-trust-capability-cryptography-supply-chain-audit-review.md
  - docs/plans/optimize/zircon_tooling/37-transaction-atomicity-prepare-commit-publish-rollback-compensation-idempotency-crash-recovery-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/AsyncTaskNotification.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/SlowTask.h
  - dev/UnrealEngine/Engine/Source/Developer/MessageLog/Public/IMessageLogListing.h
  - dev/UnrealEngine/Engine/Source/Editor/GameProjectGeneration/Private/SProjectBrowser.cpp
  - dev/godot/editor/project_manager/project_manager.cpp
  - dev/godot/editor/project_manager/project_list.cpp
  - dev/godot/editor/gui/progress_dialog.h
  - dev/godot/editor/gui/progress_dialog.cpp
  - dev/godot/editor/gui/editor_toaster.h
  - dev/Fyrox/project-manager/src/manager.rs
  - dev/Fyrox/project-manager/src/project.rs
  - dev/Fyrox/project-manager/src/settings.rs
  - dev/bevy/crates/bevy_ecs/src/message/messages.rs
  - dev/bevy/crates/bevy_ecs/src/message/message_registry.rs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 04 · Command / Action / Message Delivery / Task / History / ViewModel / Localization 产品集成工程化差距

## 1. 结论

Zircon Hub已经不是纯演示壳。当前代码有严格action enum、按action拆分的typed payload、`deny_unknown_fields`、绝对路径基础校验、结构化双语`HubMessageId`、后台工作线程、panic containment、原子配置替换、最近项目三方合并、action history和Rust-owned ViewModel。这些基础应保留，尤其不能退回前端直接拼命令、直接改配置或以英文字符串判断业务状态。

但它仍不是工程级Launcher/Hub控制面。Tauri只暴露`hub_state`和`hub_action`两个命令，所有状态、同步I/O、catalog refresh、项目选择、队列和完成回写共享一个`Mutex<HubRuntimeSession>`。后台请求进入进程内无界`VecDeque<HubActionRequest>`时既未绑定immutable project/engine/artifact，也未获得稳定OperationId、principal、capability、expected generation、deadline、idempotency key或confirmation token；出队后重新读取当前全局选择，并把动作目标反写为新的UI选中项目。队列等待期间一次普通选择变化就可能改变真正被build/package/install/open的对象。

任务系统只有一个全局`TaskStatus`和一个重启归零的`u64`计数器，进度固定为0/10/35/100。队列、running task、取消意图、阶段、真实工作量、进程owner和完成回执都不持久化。进程崩溃会丢掉全部排队动作；外部副作用成功后若TOML持久化失败，调用方收到错误，但目录已经打开、进程已经启动或文件已经复制，系统没有reconcile/compensation状态。prepare、dispatch和panic错误也可以只落TaskStatus而不进入history。

安全边界同样不合格。`open-output-folder`接受WebView传入的任意绝对`outputDir/path`，历史匹配失败后直接把字符串转成`PathBuf`并调用系统shell；它不要求路径属于project/build/package/install root或可信receipt。action history又把命令参数和日志摘录明文写入Hub TOML并直接投影，未定义secret/token/path redaction。Hub03未来一旦接入credential和remote URL，这会从本地正确性缺口升级为credential exposure和native shell capability问题。

消息与本地化已有145个唯一稳定ID和双语placeholder tests，但构造/反序列化不校验`param_count`，模板以循环`replace`渲染，参数中的`{1}`可被后续替换再次解释。未知message ID会不可逆降级成RawText，44个production/test调用仍直接制造raw English/path/error。`TaskStatus.label`、operation target和巨型`ui_text.rs`仍靠自由字符串映射，plugin template和未来provider不能注册locale bundle、fallback或版本化message schema。

因此本轮给出5个P0、60个P1、15个P2。目标不是复制Unreal类层次，而是建立`admission -> immutable OperationSpec -> durable scheduler/journal -> executor -> effect ledger -> terminal receipt -> localized read model`。只有正确性、权限、恢复和可观测性门先通过，才有资格比较Hub冷启动、action dispatch、queue latency、projection cost和工作流完成时间，更不能通过删除语义或证据来宣称性能优于Unreal。

## 2. 审查范围、证据与边界

本轮冻结84个Hub物理文件，共31,885行、1,189,411 bytes、404个选定`#[test]`、0 ignored。路径按小写forward-slash排序，对每个文件取SHA-256，再以`path|hash`和LF连接形成manifest；当前工作树fingerprint为`fd1e66a7f681c3b24634b211d621c8cbb8e56670ea87e2bdd1c6087f7f01dd30`。参考侧冻结14个文件、8,695行、309,066 bytes。

| 子域 | 文件 / 行 / bytes | 测试证据 | 本轮判定 |
|---|---:|---:|---|
| Hub state/message/action/task/history/ViewModel | 45 / 12,624 / 480,641 | 134个selected unit tests | E3逐文件静态审查，覆盖构造、路由、queue、persist、projection、localization和failure path |
| Hub integration contracts | 39 / 19,261 / 708,770 | 270 tests，0 ignored | E3逐文件分类；只有1个文件链接`zircon_hub`业务类型，38个文件主要做source/doc/CSS/TSX字符串合同 |
| Unreal参考 | 4 / 1,820 / 60,040 | reference only | E3核对async notification、real work progress、message log和project open/upgrade/repair门 |
| Godot参考 | 5 / 4,311 / 159,706 | reference only | E2/E3核对Project Manager、多task progress、cancel、toast和version/migration确认 |
| Fyrox参考 | 3 / 2,021 / 69,311 | reference only | E2/E3核对Rust Project Manager command queue、Child polling和log surface；仅作较低基线 |
| Bevy参考 | 2 / 543 / 20,009 | 2 reference tests | E2核对typed `MessageId`、cursor/count和bounded lifecycle；不外推为durable Hub operation |
| 合计 | 98 / 40,580 / 1,498,477 | 406个test attributes | production/tests未修改，动态测试未重跑 |

39个integration contract文件共有270个tests、81处文件读取和194处`.contains()`调用。只有`project_management_contract.rs`直接`use zircon_hub::...`，其中9个tests真正执行metadata/config/recent/template/recycle/filter业务类型，其余3个仍检查源码或文档字符串。剩余38个integration文件不链接Hub业务crate，适合作为source-shape guard，但不能证明Tauri权限、跨线程顺序、重启恢复、OS副作用、消息兼容或端到端行为。

本报告拥有Hub内部command/action envelope、admission、target binding、task registry、action history、message delivery和localized read model。Hub01继续拥有具体Project/Engine/Build/Editor launch/process backend；Hub02继续拥有web shell、页面、通用accessibility和catalog/settings/team页面；Hub03继续拥有Marketplace/Auth/Organization/Cloud provider；Tooling09继续拥有engine update/release repository；Tooling23/24/26/37只拥有跨仓治理规则，不重复计算本报告的Hub product findings。

本轮是review-only。没有运行Cargo、Tauri窗口、OS shell、真实build/package/install、故障注入、并发soak或性能benchmark。静态事实不能证明运行性能、没有死锁或真实恢复成功；这些都明确保留为验收门。

## 3. 当前值得保留的基础

1. `HubActionId`和`HubAction`建立了后端action allowlist；legacy alias集中在Rust侧，前端不应重新拥有路由真相。
2. payload类型普遍使用`deny_unknown_fields`，create/import/project target/settings/output/resource已从无结构target string向typed DTO演进。
3. `HubMessageId`目前有145个唯一string ID，`all()`、双语template、placeholder存在性和string roundtrip已有测试。
4. Hub config通过同目录temporary file和Windows `ReplaceFileW(...WRITE_THROUGH)`替换，失败尽量保留previous file。
5. shared recent project有三方合并和focus refresh retry，避免Hub与Editor简单last-writer-wins。
6. 后台worker把prepare、external run、complete分开，长时外部命令没有全程持有session mutex。
7. worker使用`catch_unwind`把单次panic转换为可见状态，队列可继续前进。
8. learn resource至少要求目标存在于当前catalog，project delete至少有pending confirmation与Recycle Bin实现。
9. ViewModel在Rust侧完成多数本地化和DTO投影，React不应重新解析英文status或业务路径。
10. remote Marketplace/Auth/Cloud capability仍disabled，未用伪造成功冒充远程provider。

这些基础只能作为重构输入。严格字段、原子单文件替换、panic捕获或source-shape tests均不能单独证明operation有identity、权限、durability、idempotency和recovery。

## 4. 当前实现事实

### 4.1 Action request不是工程级command envelope

`HubActionRequest`只有`action_id: String`、`target_id: Option<String>`和`payload: Option<Value>`。Tauri反序列化后，后台路径先仅按action string判断是否异步；worker active时直接clone原始request入队，不提前parse payload、解析project/engine、冻结source revision或检查state generation。`serde_json::from_value(payload.clone())`没有bytes/depth/string/item budget。unknown field rejection是正向基础，但绝对路径只证明语法，不证明canonical path、root containment、symlink/reparse-point策略或调用者权限。

Tauri command没有principal/origin/capability，返回值是完整`HubViewModel`或`String`。错误的stable code、retry class、operation ID、partial-effect状态和repair action全部丢失。前端的sequence/generation只能避免部分UI旧response覆盖，不能成为后端CAS、admission或幂等合同。

### 4.2 Background queue保存意图，不保存已解析操作

首个background action在提交时调用`apply_request_project_target`，随后worker prepare前又调用一次；排队action则只保存raw request，出队时`set_background_action_status`先从当前session计算label，prepare时才解析并激活目标。`activate_recent_project_target`会修改`selected_project_path`、active engine并刷新scoped catalogs。执行特定项目动作因此会改变作者正在浏览的项目，且queue waiting期间recent registry、engine binding、project path和selection变化都可能改变真实目标。

无显式target时，scope还会使用latest recent project；active engine缺失时可fallback到first engine。UI没有选择不等于用户授权“最近项目”。对package/install/build/open这种有外部副作用的命令，implicit latest/first fallback必须在admission前被替换为显式、可展示、可确认的immutable target lease。

### 4.3 TaskStatus是单一提示条，不是任务系统

`HubRuntimeSession`只有一个`task_status`、一个`background_task_counter`、一个`background_worker_active`和一个无界`VecDeque`。counter每次启动从0开始，普通`+= 1`没有checked overflow，也无process/session epoch。snapshot只暴露queue length，不暴露queued operation identity、target、owner、priority、deadline或cancel state。

进度只有idle 0、started 10、prepared 35和success 100。没有total work、completed work、phase tree、indeterminate、ETA、throughput、start/finish monotonic time或stalled detection。只有一条action可运行，build/package/install/open editor互相head-of-line block；用户不能cancel、pause、resume、retry或reorder。

worker panic记录失败后忽略persist错误并继续队列。`commands::session()`把poison视为错误，而background `lock_session()`却取`poisoned.into_inner()`继续，两个入口对invariant破坏采取相反策略，且都没有validation/repair generation。

### 4.4 持久化不是operation commit

`persist()`会先三方合并shared recents，再保存整个Hub TOML。许多普通UI操作也同步调用它；focus refresh、catalog refresh和部分filesystem discovery都在全局session lock内执行。一个慢盘、network-mounted project、杀毒扫描或坏目录可阻塞所有Tauri command和background completion publication。

open folder/resource在`spawn()`成功后先把history/status写入内存，再调用persist。persist失败时外部shell已经启动，调用者收到失败，内存又已突变；没有`EffectCommittedButReceiptPending`、repair journal或retry token。build/package/install/open editor的具体partial-effect由Hub01拥有，本报告要求它们统一接入shared operation commit protocol。

`record_background_action_error`只写TaskStatus，不写ActionHistory。invalid target、prepare error、dispatch failure和worker panic可能在下一次status覆盖后完全消失。`take_next_background_action`又更新running status但不持久化。任务提示、历史、外部effect和配置不是一个可恢复transaction。

### 4.5 History不是durable audit/evidence log

`HubActionRecord`只有finished wall-clock、10种action kind、3种status、target、detail/log/recovery、PID、command argv和output dir。它没有OperationId、request digest、principal、admission decision、resolved target identity/revision、start time、duration、attempt、phase、artifact digest、parent/child、idempotency key、effect/receipt state、cancel reason或repair outcome。

history存入同一个Hub TOML，固定只保留16条。ID由`finished_ms:action:target`拼接，同毫秒同动作同target会碰撞，target中的冒号也无编码。output lookup还允许按target或英文render后的detail匹配，找不到就把target当路径。没有append journal、pagination、retention bytes/age、corruption isolation、crash recovery、archive或audit chain。

command line以`Vec<String>`保存但显示时`join(" ")`，丢失quoting/argument boundary。日志与argv没有secret/path/token redaction，TOML和ViewModel访问也没有privilege separation。未来remote token、signed URL、credential helper或environment-derived argument不得进入现有record。

### 4.6 Message ID存在，但message schema仍不稳定

145个ID当前无重复，English/Chinese template也覆盖声明的placeholder，这是明确正向事实。但`HubMessage::new/with_params`和deserialize不检查实参个数；缺参会把`{n}`裸留给用户，多参会被静默忽略。循环`replace`会把参数内容重新当模板，例如第0个参数包含`{1}`时会被第1轮替换污染。

未知structured ID在load时降级为RawText并丢失原始structured shape，旧二进制保存后无法由新版本恢复。wire/persisted record没有message schema version、namespace/provider、locale domain、argument type或sensitivity。`HubError::into_status_messages`和44个`raw_text`调用又让I/O/parser/process error直接保留English字符串。

### 4.7 Localized ViewModel仍由自由字符串驱动

`TaskStatus.label`和target是String，`localized.rs`用大match把英文label映射成中文；unknown label原样返回。operation target也按英文常量映射。一个backend改标点或plugin新增label即可静默回退English，不会产生missing-key diagnostic。

`ui_text.rs`接近千行，以一个静态双语DTO承载shell/pages/buttons/status。没有locale catalog版本、fallback chain、translation ownership、plugin bundle、completeness report、pseudo-locale、RTL、hot reload或ICU-style plural/select。project template只硬编码四个ID；unknown provider template被投影成通用“Project Template”，丢失可区分身份。

ViewModel每次clone recent projects、catalogs、history、settings和draft。`HubSnapshot::filtered_recent_projects`还在projection时逐项`Path::exists()`。状态读取因此可触发同步filesystem I/O，且没有snapshot generation、delta、cache validity或projection budget。

### 4.8 测试数量大，但behavioral coverage很窄

selected Hub代码有134个unit tests，action parser、queue FIFO、task ID preservation、message templates、config replace和局部failure已有覆盖。39个integration contract又有270个tests，但38个文件不链接Hub业务crate，大量断言只是检查某个Rust/TSX/doc片段仍存在。

当前没有真实Tauri permission/origin测试、bounded admission压力、并发submit/select/focus-refresh模型、kill/restart queue recovery、persist fail after external effect、secret redaction、message fuzz/property、old/new binary roundtrip、symlink/reparse-point containment、real OS folder open acknowledgement或真实Hub window end-to-end task lifecycle。source-shape tests可以继续保留为辅助层，但不得在测试矩阵中计作这些behavior gates。

## 5. 参考源码约束

### 5.1 Unreal

`FAsyncTaskNotificationConfig`显式描述title/progress、headless、can-cancel、success/failure retention、icon和log category；状态有Pending/Success/Failure/Prompt，Prompt action有Continue/Cancel/Unattended，并支持hyperlink和动态state update。`FSlowTask`记录total/completed/current-frame work、start time、visibility和cancel，而不是用固定百分比模拟进度。`IMessageLogListing`把listing identity、filter、selection、token action和data/page events分开，说明可操作diagnostic不是一段拼接字符串。

`SProjectBrowser::OpenProject`在真正启动前检查project file、engine version、project status、compiler、migration、copy/in-place/skip、project file generation和build failure；危险migration默认推荐copy并允许cancel。Zircon不需要复制Slate或阻塞dialog实现，但必须保留typed preflight、显式选择、可取消、repair detail和terminal outcome。

### 5.2 Godot

Godot Project Manager对missing project、main scene、imported assets、config version、unsupported feature、多项目open/run和migration逐项preflight；转换操作展示项目路径、兼容风险和backup选项，cancel优先获得focus。`ProgressDialog`和`BackgroundProgress`按task string在`HashMap`中维护多个task、steps/state/last tick，并可取消；`EditorToaster`把severity、message、count、progress和生命周期分开。

Godot仍不是durable distributed operation系统，task string也不是Zircon应采用的跨进程ID。可借鉴的是多任务、真实steps、取消和preflight语义，不是具体singleton/UI结构。

### 5.3 Fyrox

Fyrox Project Manager以`Mode::CommandExecution`保存Child和command queue，pipe stdout/stderr，逐帧`try_wait`并在命令之间reset build window；退出时若game/editor仍运行会确认。它比Zircon“spawn即opened成功、drop Child”更完整，但queue仍是进程内、无durable receipt/operation identity，不能作为最终上限。

### 5.4 Bevy

Bevy `Messages<M>`给每条typed message分配单调`MessageId<M>`，cursor和start count定义reader可见窗口，registry统一更新与retire双buffer。它证明typed identity和显式retention lifecycle可以低成本实现；但frame-local message不是Hub durable command，也不提供principal、idempotency或crash recovery。

### 5.5 Unity Graphics适用边界

仓内`dev/Graphics`是render package源码，不包含Unity Hub/Launcher/Package Manager账号与项目操作控制面。本轮不把SRP package manifest或render command当作Hub operation precedent，也不因缺少Unity Launcher源码推断Unity的行为。

## 6. 目标架构与Owner

```text
WebView/Tauri caller
  -> HubCommandGateway
     -> envelope budget + protocol/version
     -> principal/origin/capability/confirmation
     -> expected state generation + idempotency
  -> OperationAdmission
     -> resolve ProjectId/EngineId/ArtifactId/PathCapability
     -> immutable OperationSpec + request digest
  -> DurableOperationJournal
     -> bounded scheduler / priority / deadline / cancellation
     -> worker supervisor / process-tree owner
  -> Domain Executor
     -> preflight -> effect -> verify -> compensate/reconcile
  -> EffectLedger + TerminalReceipt
  -> TaskRegistry + MessageCatalog
  -> versioned HubReadModel(delta/snapshot)
  -> React projection
```

| Owner | 唯一职责 | 禁止承担 |
|---|---|---|
| `HubCommandGateway` | Tauri protocol、payload budget、principal/origin、capability、request/response schema | 不解析业务project目录，不执行shell |
| `OperationAdmission` | 绑定immutable target、state generation、confirmation、idempotency和deadline | 不修改UI selection，不执行外部effect |
| `HubOperationScheduler` | durable journal、bounded queue、priority/fairness、cancel、worker lease、restart recovery | 不拥有build/package业务语义 |
| Domain executor | Project/Build/Package/Install/Editor/Open Resource具体preflight与effect | 不自建第二套queue/history/message schema |
| `EffectLedger` | effect intent/commit/verify/compensate/receipt、artifact/process/path identity | 不存presentation copy |
| `TaskRegistry` | 多operation phase/work/progress/timing/stall/cancel projection | 不以单一global status覆盖历史 |
| `MessageCatalog` | versioned key、typed args、sensitivity、fallback、provider locale bundle | 不接收任意backend English作为业务ID |
| `HubReadModel` | generation-consistent snapshot/delta、localized display、pagination | 不在projection做同步filesystem/process I/O |

UI selection必须是authoring/navigation state，OperationSpec target必须是execution state。两者可以在admission时从同一个显式用户动作产生，但此后不得互相覆盖。Path不是权限；打开目录、读catalog、package和install必须消费root-scoped `PathCapability`或可信receipt中的artifact/path identity。

## 7. P0阻断项

### ZHUB-CTL-P0-01 · WebView action边界可把任意绝对目录交给系统shell

`OpenOutputFolderPayload`只验证absolute path，`resolve_output_folder`优先接受`output_dir/path`，历史匹配失败后也把target直接构造为路径；存在目录即`open_folder`。没有principal、capability、root containment、receipt provenance、confirmation或audit。必须改为`OpenRecordedOutput { receipt_id }`或受限`OpenPathCapability`，只允许Hub自身生成且仍有效的project/build/package/install/document root。测试必须覆盖`..`、symlink/junction/reparse point、UNC/device path、case alias、stale receipt和compromised WebView caller。

### ZHUB-CTL-P0-02 · 排队动作没有immutable target，执行时可随全局选择漂移

queue保存raw request，出队/prepare时才解析；无显式target又会fallback latest project/first engine。显式target执行还会反写`selected_project_path`和active engine。必须在admission时解析为含ProjectId/path identity/engine generation/source revision的OperationSpec，验证expected Hub state generation并持久化；执行器不得修改navigation selection。并发select/import/remove/engine-rebind与queued build/package/install/open必须有确定性模型测试。

### ZHUB-CTL-P0-03 · 无界进程内队列和单状态槽无法取消、恢复或证明终态

`VecDeque`无items/bytes/time budget，counter重启归零，queue/running task不持久化，worker没有deadline/cancel/process-tree lease。崩溃或关闭后所有意图丢失，外部Child可能继续。必须建立durable bounded journal、stable OperationId、worker lease/heartbeat、cancel/deadline、restart classification和terminal receipt；shutdown要drain/cancel/transfer owner并禁止silent abandonment。

### ZHUB-CTL-P0-04 · 外部effect与history/config持久化不是同一可恢复commit

open folder/resource和domain completion可以先产生外部effect，再因persist失败向调用方报错；prepare/panic错误又可能不进history。系统无法区分“未执行”“已执行未记账”“已记账未验证”“补偿失败”。必须以intent journal先持久化，effect后写commit/verification，失败进入typed reconciliation；idempotency key阻止重复副作用，compensation/repair receipt保留真实状态。具体backend补偿仍由Hub01领域owner实现。

### ZHUB-CTL-P0-05 · Action history会持久化并投影未分级的命令与日志敏感数据

完整argv、log excerpt、绝对路径和未来可能出现的token/signed URL进入Hub TOML和ViewModel，没有field sensitivity、redaction、access policy、retention或secure storage boundary。必须在接Auth/Cloud/Marketplace前建立typed diagnostic fields、secret classification、redaction-at-source、privileged detail fetch和audit export policy；禁止credential进入command line，旧历史需迁移/清理。security tests必须用canary secret证明disk、IPC、UI、log和crash artifact均不泄露。

## 8. P1工程化差距

### 8.1 Action protocol与admission

- ZHUB-CTL-P1-01：`HubActionId`是31项closed enum和若干alias，但没有protocol version、descriptor version、introduced/deprecated range或capability negotiation。
- ZHUB-CTL-P1-02：request没有RequestId/OperationId、caller principal、origin window、trace/span、expected state generation、deadline、idempotency key和confirmation token。
- ZHUB-CTL-P1-03：`serde_json::Value`在Tauri边界先完整分配再clone/deserialize，没有payload bytes、nesting depth、string length、array/map item和parse-time budget。
- ZHUB-CTL-P1-04：absolute path校验没有canonicalization policy、root identity、symlink/reparse resolution、filesystem capability、case/Unicode normalization和TOCTOU reopen规则。
- ZHUB-CTL-P1-05：`target_id`、`projectId`、`projectPath`、resource title/path、history ID和raw target存在多套precedence，错误或stale alias可命中不同对象。
- ZHUB-CTL-P1-06：后台判断先按action string，真正parse在另一阶段；admission与execution没有共享compiled descriptor，新增action容易漏掉background/security/persistence规则。
- ZHUB-CTL-P1-07：worker active时request未经parse/validate/resolution就入队，invalid payload占用queue并延迟失败，无法计算资源预算和权限。
- ZHUB-CTL-P1-08：Tauri返回`Result<HubViewModel, String>`，stable error code、retryability、field violation、partial-effect、operation receipt和support correlation全部丢失。
- ZHUB-CTL-P1-09：所有action都回传完整ViewModel，没有server generation、base generation、delta或snapshot hash，后端无法证明response与effect同代。
- ZHUB-CTL-P1-10：action descriptor没有集中声明scope、risk、confirmation、foreground/background、queue class、required capability、idempotency和expected receipt；规则散落在match和前端map。

### 8.2 Scheduler、并发与生命周期

- ZHUB-CTL-P1-11：一个全局`Mutex<HubRuntimeSession>`同时保护config、draft、navigation、catalog、queue和task completion，锁域远大于单一invariant。
- ZHUB-CTL-P1-12：focus refresh和多条persist/catalog/project refresh路径在session lock内做filesystem I/O，慢盘可冻结所有command和completion publication。
- ZHUB-CTL-P1-13：build/package/install/open editor共用一个串行worker，长任务造成无分类head-of-line blocking，读类动作也没有独立lane。
- ZHUB-CTL-P1-14：queue无items/bytes/tenant/target budget、priority、fairness、dedupe、coalescing和admission rejection receipt。
- ZHUB-CTL-P1-15：`background_task_counter += 1`无checked overflow、process/session epoch或持久化，ID重启复用且不能作为全局关联键。
- ZHUB-CTL-P1-16：`take_next_background_action`设置新running status但不persist，崩溃后既无queued也无running证据。
- ZHUB-CTL-P1-17：进度0/10/35/100是workflow装饰常量，不来自work units；package文件数、build phase、copy bytes和process preflight都未接入。
- ZHUB-CTL-P1-18：snapshot只暴露queued count，用户看不到每项target、phase、waiting reason、priority、elapsed、cancelability和失败依赖。
- ZHUB-CTL-P1-19：没有cancel/pause/resume/retry/reorder/deadline API，BackgroundTask trait也不接收CancellationToken、progress sink或deadline。
- ZHUB-CTL-P1-20：foreground command遇poison直接失败，background取poisoned inner继续；panic后没有invariant validation、generation bump、queue quarantine或repair receipt。

### 8.3 History、receipt与诊断

- ZHUB-CTL-P1-21：31个action只映射到10个history kind，navigation/settings/filter/browse等operation无统一audit分类，新增action容易没有history policy。
- ZHUB-CTL-P1-22：history status只有success/failed/cancelled，缺admitted/queued/running/partial/compensating/reconciled/abandoned/timed-out/superseded。
- ZHUB-CTL-P1-23：record只有finish wall-clock，没有admitted/started/phase/finished monotonic time、duration、queue latency和attempt。
- ZHUB-CTL-P1-24：固定16条且与config同文件，没有按bytes/age/severity/operation policy、pagination、archive、corruption isolation或user-visible retention。
- ZHUB-CTL-P1-25：`finished_ms:action:target`不是稳定ID，同毫秒碰撞且target未编码；lookup可能打开错误record/output。
- ZHUB-CTL-P1-26：`push_action_record`、manual insert/truncate和多处record helper分散，不能保证一条operation只有一个canonical transition authority。
- ZHUB-CTL-P1-27：background invalid target、prepare error、dispatch error和worker panic只写TaskStatus，history/audit可能永久缺失。
- ZHUB-CTL-P1-28：command display以空格join argv，无法还原Windows/POSIX quoting、empty argument、embedded whitespace和secret placeholder。
- ZHUB-CTL-P1-29：relative time对future timestamp饱和为“just now”，不显示absolute timestamp/timezone，也没有clock-skew diagnostic。
- ZHUB-CTL-P1-30：path/target广泛`to_string_lossy`，不可表示OS路径会丢identity或碰撞；persisted identity必须与display string分开。

### 8.4 Message schema与localization

- ZHUB-CTL-P1-31：顺序`.replace("{n}", arg)`允许argument中的placeholder被二次解释，message内容可被后续参数篡改。
- ZHUB-CTL-P1-32：constructor和deserialize不强制`params.len() == id.param_count()`，缺参、多参和错误类型静默进入history/UI。
- ZHUB-CTL-P1-33：unknown structured ID降级RawText并丢失ID/params，旧binary读取新record再保存会破坏forward recovery。
- ZHUB-CTL-P1-34：persisted message没有schema version、namespace/provider、argument names/types、sensitivity或fallback key。
- ZHUB-CTL-P1-35：44处`raw_text`调用仍承载I/O、process、project、path和status，无法系统统计missing translation或执行redaction。
- ZHUB-CTL-P1-36：普通`HubError`通过`to_string()`变成English RawText，error source chain、code、field和retry class不可投影。
- ZHUB-CTL-P1-37：`TaskStatus.label`由英文自由字符串驱动本地化match，backend文案改动会静默变成未翻译状态。
- ZHUB-CTL-P1-38：operation target也用英文常量映射，identity、display name和localized category混在一个String。
- ZHUB-CTL-P1-39：`ui_text.rs`单文件静态双语DTO没有locale catalog、fallback、completeness、pseudo-locale、plural/select、RTL和provider bundle。
- ZHUB-CTL-P1-40：project template只本地化四个hardcoded ID，plugin/provider template会退化为通用标题，扩展生态无法拥有稳定copy。

### 8.5 Scope、projection与OS动作

- ZHUB-CTL-P1-41：无selected project时scope使用latest recent，package/install/open等高风险动作可在用户没有显式选择时被enabled。
- ZHUB-CTL-P1-42：configured active engine缺失时可fallback first engine，repair和execution target没有显式区分。
- ZHUB-CTL-P1-43：执行targeted background action会修改navigation selected project和active engine，作者视图被后台operation副作用污染。
- ZHUB-CTL-P1-44：`filtered_recent_projects`在read-model projection逐项`Path::exists()`，snapshot读取包含不可预算同步I/O和TOCTOU结果。
- ZHUB-CTL-P1-45：每次state/action clone catalogs、recent、history、settings/draft并重建完整ViewModel，没有dirty set、incremental projection或cache generation。
- ZHUB-CTL-P1-46：new-project stale template/engine会静默替换为first enabled/first engine，UI显示目标可与persisted draft和用户原意不同。
- ZHUB-CTL-P1-47：learn resource以localized title或lossy path识别，重复title和catalog drift可重定向；动作实际打开parent folder却记录“Resource opened”。
- ZHUB-CTL-P1-48：output lookup按history ID、target或English-rendered detail匹配，失败又退回raw path，identity、display和capability完全混合。
- ZHUB-CTL-P1-49：系统shell `spawn()`成功即记“opened”，Child立即drop；没有window/folder acknowledgement、exit status、focus outcome或process ownership。
- ZHUB-CTL-P1-50：action history detail把PID/command/output/log同级投影给所有页面，没有summary/detail privilege、lazy fetch和large-log artifact boundary。

### 8.6 Test architecture与qualification

- ZHUB-CTL-P1-51：39个integration文件中38个不链接Hub业务crate，source snippet存在被当作集成完成证据。
- ZHUB-CTL-P1-52：270个integration tests中只有9个直接执行production business types；queue/message/Tauri/ViewModel主链没有跨边界behavior test。
- ZHUB-CTL-P1-53：大量`.contains()`只证明词法片段，重命名会误红，死代码/错误调用/永不到达分支仍可误绿。
- ZHUB-CTL-P1-54：没有Tauri capability/origin/command schema和compromised WebView测试，`open-output-folder`的native shell权限未被安全门覆盖。
- ZHUB-CTL-P1-55：没有submit/select/focus refresh/persist completion并发模型、lock-order test、race injection或bounded queue stress。
- ZHUB-CTL-P1-56：没有kill -9/restart、journal replay、stale worker lease、orphan Child和queued operation recovery测试。
- ZHUB-CTL-P1-57：没有在external effect后、receipt write前注入disk full/permission/rename failure，partial success合同未验证。
- ZHUB-CTL-P1-58：没有payload size/depth/path alias fuzz、message deserialize property test、history corruption corpus或parser timeout budget。
- ZHUB-CTL-P1-59：没有placeholder injection、wrong param count、unknown-ID lossless roundtrip、provider locale collision和canary-secret redaction测试。
- ZHUB-CTL-P1-60：没有真实Tauri窗口驱动build/package/install/open/cancel/restart，也没有OS explorer acknowledgement和source/build-bound machine-readable receipt。

## 9. P2完善项

- ZHUB-CTL-P2-01：legacy action alias缺deprecation telemetry、support window和removal manifest。
- ZHUB-CTL-P2-02：unknown action/payload error没有可机器消费的supported action/capability discovery和最近合法值提示。
- ZHUB-CTL-P2-03：history没有按project/action/status/time/owner筛选、搜索、导出和bookmark。
- ZHUB-CTL-P2-04：relative time旁没有absolute local/UTC切换和timezone展示。
- ZHUB-CTL-P2-05：真实work model完成后仍需ETA、throughput和phase duration，但不得在无样本时伪造。
- ZHUB-CTL-P2-06：action icon、category、risk、availability reason和repair links没有由后端descriptor统一生成。
- ZHUB-CTL-P2-07：重复相同failure没有group/deduplicate/count策略，notification与audit record也未分层。
- ZHUB-CTL-P2-08：target display缺project/engine/artifact breadcrumb和copyable stable ID，长路径难以辨识。
- ZHUB-CTL-P2-09：开发构建没有pseudo-locale、missing-key overlay和message argument inspector。
- ZHUB-CTL-P2-10：locale fallback/missing/provider collision没有diagnostic dashboard和export report。
- ZHUB-CTL-P2-11：command detail没有OS-aware quoted preview、逐argument copy和redacted/raw privileged切换。
- ZHUB-CTL-P2-12：大量history/task row时尚无server pagination、virtualization和incremental append protocol。
- ZHUB-CTL-P2-13：没有queue latency、admission rejection、task stall、reconcile count和projection cost metrics。
- ZHUB-CTL-P2-14：没有ViewModel projection、message render、history paging和high-cardinality task registry microbenchmark。
- ZHUB-CTL-P2-15：action/message/task/receipt schema没有生成的protocol catalog、compatibility table和operator troubleshooting文档。

## 10. 分层重构计划

### M0 · Freeze、truth split与安全止血

1. 冻结现有action/message/history wire fixtures和84-file source fingerprint，建立source drift recheck。
2. 立即移除`open-output-folder`的raw path/target fallback；只接受现有history record中验证过的output receipt，直到PathCapability上线。
3. 在history producer引入最小secret redaction并禁止token/signed URL进入argv；标记现有TOML历史为untrusted legacy。
4. 将implicit latest/first target对高风险action改为Unavailable，UI必须显式选择并展示resolved target。
5. 测试账本把source-shape、unit behavior、component、Tauri integration、OS/e2e、fault、soak、performance分lane，不再汇总冒充。

### M1 · Versioned command envelope与immutable admission

1. 新建`HubCommandEnvelopeV1`：RequestId、protocol version、action descriptor ID、typed payload bytes、principal/origin、expected generation、deadline、idempotency、confirmation。
2. 以generated/compiled descriptor统一payload schema、risk、capability、queue class、target resolver和receipt type。
3. admission先执行budget、parse、authorization、preflight和target binding，产出immutable `OperationSpec`及request digest。
4. ProjectId/EngineId/ArtifactId/PathCapability与display string分离；navigation selection不再由executor写入。
5. Tauri response改为typed `CommandAccepted/Rejected/Completed`，带stable error、retry、field issue、OperationId和server generation。

### M2 · Durable scheduler、TaskRegistry与worker supervision

1. 建立append-only operation journal，先写admitted/queued再响应；以checksum/version和atomic segment rotation恢复。
2. queue按items/bytes/target/class/age有界，支持priority/fairness/dedupe/coalescing和明确rejection receipt。
3. stable OperationId包含随机/monotonic安全身份与session epoch，不从process-local counter推断。
4. `TaskRegistry`支持多个operation、phase tree、real work units、indeterminate、timing、stall、cancel/deadline/retry。
5. worker supervisor拥有thread/process tree、lease/heartbeat和shutdown；restart分类resume/retry/reconcile/abandon，不能silent drop。
6. 拆分navigation/config/catalog/scheduler/task read model锁域，任何filesystem/process/network I/O均在锁外且以generation提交。

### M3 · Effect ledger、history与诊断安全

1. 每个executor遵循`preflight -> intent -> effect -> verify -> receipt`，失败进入compensate/reconcile而非覆盖一个status。
2. EffectLedger记录resolved identities、attempt、artifact/process/path、effect state和repair outcome；domain receipt引用它而不复制自由文本。
3. history成为journal的paged read model，按age/bytes/severity policy保留，corrupt record隔离，旧16-row TOML迁移。
4. diagnostic field声明public/path/PII/credential/secret级别，在producer端redact；privileged raw artifact单独授权和加密。
5. output/resource open只消费receipt/capability，记录request accepted、OS spawn、acknowledged/unknown结果，不把spawn等同opened。

### M4 · MessageCatalog与generation-consistent read model

1. `MessageEnvelopeV1`保留namespace/key/version/typed named args/sensitivity；unknown key lossless roundtrip。
2. renderer单遍解析模板，不重新解释argument；constructor、deserialize、catalog load都严格验证arg schema。
3. builtin与provider locale bundle有owner/generation、fallback、collision、completeness、pseudo-locale、plural/select和RTL资格。
4. backend error先映射stable ErrorCode/source chain/retry，display最后本地化；禁止用English label驱动业务。
5. HubReadModel把identity与display分开，以generation/delta投影；filesystem existence、catalog discovery和relative time在独立producer/cache完成。

### M5 · Behavioral qualification与竞争性性能门

1. 建立Tauri command harness、deterministic scheduler、fake filesystem/process/clock和crash-restart fixture。
2. 执行payload/path/message fuzz、queue/load/poison/fault matrix、effect-after-persist-failure和secret canary tests。
3. 真窗口验证多task、cancel、restart、history/detail、locale switch、stale generation和OS folder/process outcome。
4. 对冷启动、first state、action admission、queue latency、progress publication、ViewModel delta、RSS/I/O和workflow completion建立versioned workload。
5. 与Unreal/Godot/Fyrox比较时固定机器、project规模、操作语义、cache state、并发和统计协议；先通过correctness/recovery，再报告中位数、p95/p99和置信信息。

## 11. 验收矩阵

| Gate | 验收内容 |
|---|---|
| HCTL-G01 | Tauri只接受versioned typed envelope，oversize/deep payload在业务分配前被拒并有stable receipt |
| HCTL-G02 | 每个accepted command有全局唯一OperationId、request digest、principal/origin和server generation |
| HCTL-G03 | 高风险action声明capability、confirmation、idempotency、deadline、target和receipt type |
| HCTL-G04 | `open-output-folder/resource`不能接受raw arbitrary path，只能消费仍有效的receipt/PathCapability |
| HCTL-G05 | path matrix覆盖relative、`..`、case/Unicode alias、symlink/junction/reparse、UNC/device和TOCTOU |
| HCTL-G06 | queued operation的project/engine/artifact identity在admission后不受UI selection和registry reorder影响 |
| HCTL-G07 | executor运行targeted action不改变navigation selected project或active authoring context |
| HCTL-G08 | 无显式target时build/package/install/open fail-close，不fallback latest project/first engine |
| HCTL-G09 | queue有items/bytes/class/target/age预算，overload返回typed rejection且RSS受控 |
| HCTL-G10 | 多class scheduler无无限head-of-line blocking，并有fairness/priority/dedupe证据 |
| HCTL-G11 | queue/running/cancel/deadline/attempt在kill/restart后可恢复或明确终止 |
| HCTL-G12 | stable OperationId跨restart不复用，worker lease过期不会产生双执行 |
| HCTL-G13 | build/package/install/open editor支持cancel并拥有process-tree/filesystem cleanup合同 |
| HCTL-G14 | progress来自真实work units/phase或明确indeterminate，不使用固定装饰百分比 |
| HCTL-G15 | TaskRegistry可同时展示多个queued/running/terminal task及target、elapsed、reason和action |
| HCTL-G16 | session lock不跨filesystem/process/network I/O，锁域和order有静态/动态证据 |
| HCTL-G17 | focus refresh、action submit、completion和selection并发不死锁、不丢更新、不跨代发布 |
| HCTL-G18 | worker panic/poison后执行invariant validation、quarantine/repair并持久化terminal record |
| HCTL-G19 | 每个accepted operation先持久intent，再执行effect；crash point可重放而不重复副作用 |
| HCTL-G20 | effect成功、receipt写失败时状态为typed reconcile pending，不向用户谎报未执行 |
| HCTL-G21 | idempotency retry不会重复open/process/copy/delete/install，或返回同一terminal receipt |
| HCTL-G22 | domain compensation/reconcile fault matrix覆盖disk full、permission、rename、process exit和shutdown |
| HCTL-G23 | history ID无碰撞且不依赖wall-clock/target拼接，record可追溯OperationId和attempt |
| HCTL-G24 | history支持paged read、bytes/age retention、corrupt isolation、migration和archive policy |
| HCTL-G25 | prepare/dispatch/panic/cancel/timeout/partial/reconcile全部进入canonical history |
| HCTL-G26 | argv/log/path/error按sensitivity redaction，canary secret不出现在disk、IPC、UI、log、crash artifact |
| HCTL-G27 | command arguments保留边界，display使用OS-aware quoting且raw detail需单独权限 |
| HCTL-G28 | non-Unicode OS path identity无lossy collision，display conversion不参与lookup |
| HCTL-G29 | MessageEnvelope unknown key可lossless roundtrip，旧binary不破坏新record |
| HCTL-G30 | message constructor/deserializer严格验证named arg type/count/sensitivity和schema version |
| HCTL-G31 | template renderer单遍替换，argument中的`{n}`、markup和control text永不被二次解释 |
| HCTL-G32 | production business status无未登记RawText；OS/raw diagnostic与localized summary分层 |
| HCTL-G33 | locale catalog有fallback、completeness、pseudo-locale、plural/select、RTL和collision tests |
| HCTL-G34 | provider/plugin template与message bundle按owner generation install/revoke，不退化成通用身份 |
| HCTL-G35 | ViewModel带generation/delta，projection不做同步filesystem/process/network I/O |
| HCTL-G36 | 10k recent/history/task/catalog workload下projection allocation、latency、RSS有预算和回归门 |
| HCTL-G37 | source-shape tests单独标记，不能替代unit/component/Tauri/OS/fault/restart/soak lane |
| HCTL-G38 | 真实Tauri窗口完成submit/queue/progress/cancel/restart/history/locale/repair端到端验证 |
| HCTL-G39 | performance报告绑定source/build/workload/hardware/cache/statistics并先证明correctness parity |
| HCTL-G40 | `git diff --check`、frontmatter path、finding ID、severity count、index/coverage/link与source fingerprint验证通过 |

## 12. 与现有报告的依赖和非目标

| 依赖 | 本报告消费/提供 | 不重复拥有 |
|---|---|---|
| Hub01 | 提供shared OperationSpec/Task/Effect/Receipt合同；消费build/package/install/editor backend preflight和compensation | 不重写具体build script、copy algorithm、Child launch或delete实现finding |
| Hub02 | 提供generation-consistent task/history/message DTO；消费React shell和accessibility surface | 不重复页面布局、drag region、catalog scan和通用a11y finding |
| Hub03 | 提供principal/capability/redaction/idempotency入口 | 不实现Auth/RBAC/Marketplace/Cloud provider |
| Runtime02/44 | 对齐task cancellation、diagnostic record和process log identity | 不把Runtime executor/log router搬进Hub |
| Editor48 | 对齐message identity、subscription/retention和request/reply概念 | 不共享Editor topic bus作为Hub durable scheduler |
| Tooling10/23/24/26/37 | 消费测试、failure、concurrency、security、transaction全局门 | Hub04的product finding仍在本报告唯一计数 |

本报告不要求一次性把Hub做成分布式服务，也不要求照搬Unreal Slate、Godot singleton或Bevy ECS message buffer。单机产品仍可采用嵌入式journal和进程内scheduler，但accepted operation必须有稳定身份、明确权限、bounded admission、可恢复状态和terminal receipt。

## 13. 状态与产出记录

| 项目 | 状态 | 日期 | 证据 |
|---|---|---|---|
| 84个Hub selected files逐文件静态审查 | review_complete | 2026-08-19 | 31,969行、1,189,411 bytes、fingerprint `fd1e66a7f681c3b24634b211d621c8cbb8e56670ea87e2bdd1c6087f7f01dd30` |
| 39个integration contracts分类 | review_complete | 2026-08-19 | 270 tests、0 ignored；38/39不链接业务crate，9个直接production behavior tests |
| 14个参考文件对照 | review_complete | 2026-08-19 | Unreal/Godot/Fyrox/Bevy共8,709行；Unity Graphics明确不外推 |
| Finding与refactor/gate | review_complete | 2026-08-19 | 5 P0 / 60 P1 / 15 P2；M0-M5；HCTL-G01-G40 |
| Cargo/Tauri/OS/fault/restart/performance验证 | not_run | - | review-only，不把静态审查报告为动态通过 |
| Production重构 | pending | - | 本轮未修改`zircon_hub`产品源码与tests |
