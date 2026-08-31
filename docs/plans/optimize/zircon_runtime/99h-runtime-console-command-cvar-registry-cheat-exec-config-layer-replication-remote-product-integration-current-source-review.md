---
title: Runtime Console Command、CVar Registry、Cheat/Exec、Config Layer、Replication、Remote 与 Product Integration 当前源码工程化差距复核
category: zircon_runtime
report_id: Runtime107
review_date: 2026-08-22
baseline_head: bee4c707b714738346b49bba15c59468b8bd9b39
baseline_epoch: 339
related_code:
  - Cargo.toml
  - zircon_runtime/runtime-feature-presets.toml
  - zircon_runtime/src/core/runtime/config_store.rs
  - zircon_runtime/src/core/runtime/diagnostics/devtools.rs
  - zircon_runtime/src/core/framework/foundation/config_manager.rs
  - zircon_runtime/src/foundation/runtime
  - zircon_runtime/src/platform/preferences
  - zircon_runtime/src/operation
  - zircon_runtime/src/navigation/operation
  - zircon_runtime/src/dynamic_api/session/operation.rs
  - zircon_runtime/src/dynamic_api/session/diagnostics.rs
  - zircon_runtime/src/runtime_diagnostics
  - zircon_runtime/src/diagnostic_log
  - zircon_runtime/src/builtin/runtime_modules
  - zircon_runtime/src/plugin/runtime_profile
  - zircon_runtime_interface/src/runtime_api/session/operation.rs
  - zircon_app/src/entry/entry_runner/runtime_session_args.rs
  - zircon_app/src/entry/cli/launch_args.rs
  - zircon_app/src/entry/platform_preferences.rs
  - zircon_editor/src/core/commands
  - zircon_editor/src/core/commandlet
  - zircon_editor/src/ui/host/editor_operation_dispatch.rs
  - zircon_editor/src/ui/host/editor_event_control_requests.rs
  - zircon_editor/src/ui/layouts/views/console.rs
  - zircon_editor/src/ui/retained_host/console_output.rs
  - zircon_editor/assets/ui/editor/console.zui
  - zircon_editor/assets/ui/editor/host/console_body.zui
tests:
  - zircon_runtime/src/core/runtime/config_store.rs
  - zircon_runtime/src/foundation/runtime/config_manager_tests.rs
  - zircon_runtime/src/platform/preferences
  - zircon_runtime/src/operation/tests.rs
  - zircon_runtime/src/operation/tests
  - zircon_editor/src/core/commands/registry.rs
  - zircon_editor/src/core/commandlet/tests.rs
  - zircon_editor/src/tests/editor_event/runtime
  - zircon_editor/src/tests/host/retained_console_template_body.rs
  - zircon_editor/src/tests/ui/boundary/zui_asset_governance/interaction_policy.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/03-core-runtime-diagnostics-profiling-config-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/41-operation-service-handler-registry-admission-prepare-apply-progress-cancel-deadline-harvest-retention-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/42-builtin-runtime-module-catalog-profile-target-feature-selection-extension-registration-capability-load-report-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/45-preference-settings-scope-storage-overlay-bounded-io-generation-fence-durability-migration-multi-process-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/55-runtime-foundation-module-config-event-service-driver-manager-persistence-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/67-runtime-console-command-cvar-registry-cheat-exec-config-layer-replication-remote-product-integration-review.md
  - docs/plans/optimize/zircon_runtime_interface/05-runtime-host-foreign-output-safe-api-ownership-admission-budget-fuse-observability-review.md
  - docs/plans/optimize/zircon_editor/08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md
  - docs/plans/optimize/zircon_editor/25-runtime-diagnostics-performance-timeline-console-telemetry-observability-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/HAL/IConsoleManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/HAL/ConsoleManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Tests/HAL/ConsoleManagerTest.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Engine/EngineConsoleCommandExecutor.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Engine/EngineConsoleCommandExecutor.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/EngineSettings/Classes/ConsoleSettings.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/Console.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/UserInterface/Console.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/RemoteConsoleServer.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/RemoteConsoleServer.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/GameFramework/CheatManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/CheatManager.cpp
  - dev/godot/core/config/project_settings.h
  - dev/godot/core/config/project_settings.cpp
  - dev/godot/core/debugger/engine_debugger.h
  - dev/godot/core/debugger/engine_debugger.cpp
  - dev/godot/core/debugger/remote_debugger.h
  - dev/godot/core/debugger/remote_debugger.cpp
  - dev/godot/tests/core/config/test_project_settings.cpp
  - dev/godot/core/templates/command_queue_mt.h
  - dev/godot/tests/core/templates/test_command_queue.cpp
  - dev/bevy/crates/bevy_remote/src/lib.rs
  - dev/bevy/crates/bevy_remote/src/http.rs
  - dev/bevy/crates/bevy_remote/src/builtin_methods.rs
  - dev/bevy/crates/bevy_diagnostic/src/diagnostic.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/DebugManager.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/DebugUI.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/DebugDisplaySettings.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Tests/Runtime/DebugManagerTests.cs
  - dev/Fyrox/fyrox-impl/src/renderer/settings.rs
  - dev/Fyrox/editor/src/settings/debugging.rs
  - dev/Fyrox/editor/src/plugins/settings.rs
doc_type: review-and-refactor-plan
review_status: current_source_refresh_complete
implementation_status: pending
source_recheck_required: true
---

# Runtime107：Runtime Console Command/CVar 当前源码工程化差距复核

## 1. 结论

截至本轮冻结，Zircon 仍然**没有工程级 runtime console command 或 CVar 产品能力**。对 root manifest 与全部第一方产品源码精确搜索 `cvar`、`console variable`、`console command`、`remote console` 和 `cheat manager`，唯一命中仍是 Editor 测试文本中的“console command actions”；production 为 0。runtime feature preset、builtin module catalog、runtime profile、App 启动参数和 runtime diagnostics 对 whole-word `console/cvar/exec` 也均为 0。现有 Editor Console 仍是日志过滤、虚拟化输出和 Clear 面板，没有输入、解析、补全、history 或 submit。

Runtime67 的判断因此没有被后续源码推翻：其 **72 个 P1、16 个 P2、CTRL-G01 至 CTRL-G48 仍是唯一 canonical owner**。本轮登记 **0 个新增 P0、0 个新增 umbrella P1、0 个新增 P2**，也没有任何 RT67 问题可以标记为 closed。缺失能力尚未被 profile 宣称为 Available，所以当前不是 capability truth P0；一旦 App/profile/remote surface 宣称可用而 provider、principal 或 shipping gate 不成立，Runtime42 必须 fail-close，并重新评估为 P0。

当前源码有三类实质进展。第一，platform preferences 已形成有界 overlay、generation ticket、durability、deadline/cancel、retained-byte quota 和 atomic backend；第二，`RuntimeOperationService` 已有 raw admission budget、owner-thread snapshot/apply、worker prepare、cancel/deadline/retention/harvest 与 panic isolation；第三，Editor command registry 已有 stable operation path、descriptor、generation-cached palette catalog、capability、headless commandlet route 和结构化 exit report。这些都是应保留的 foundation，但仍分别属于 persistence、long operation 和 Editor authoring authority，不能被命名成 CVar/console 完成度。

本轮发现一个必须在远程产品接入前收紧的 fail-open 语义：`EditorCommandDescriptor::new` 和 serde default 都把 `callable_from_remote` 设为 `true`，而 `handle_operation_control_request` 的无 source 入口默认把来源解释为 Remote。当前 production 中没有外部 transport 调用这些入口，所以本轮不提升为新的 P0；但它直接证明 Runtime67 CTRL-G31 至 CTRL-G40 不能跳过，也应由 Editor08/25 在真正接线网络、IPC 或自动化 gateway 前改成显式 opt-in、authenticated principal、scope policy 和 audit receipt。

本轮只做 review、证据冻结和计划文档维护，没有修改 production、tests、Cargo、ABI 或参考引擎，也没有运行 Cargo、产品进程、网络攻击、replication、fault、soak 或竞争性 benchmark。因此不能宣称控制面已实现，更不能宣称性能、安全性或产品体验达到、优于 Unreal。用户已明确暂停 tooling 优化，本篇不新增 tooling、生成器或脚本迁移任务。

## 2. 审查边界、冻结与 ownership

### 2.1 唯一 owner 与去重规则

| 领域 | Canonical owner | Runtime107 的作用 | 本轮不重复登记 |
|---|---|---|---|
| Runtime command/CVar combined contract | Runtime67 | 用当前源码重验 registry、layer、apply、security、remote、product surface | RT67-P1-01..72、RT67-P2-01..16、CTRL-G01..48 |
| Config/persistence | Runtime03/55 | 说明 raw JSON config 的当前能力和边界 | 通用 config authority、flush、diagnostics 父问题 |
| Preference storage | Runtime45 | 记录 bounded persistence substrate 的真实进展 | scope/storage/migration/multi-process 父问题 |
| Long operation | Runtime41 | 记录 operation task primitive 的真实进展 | task lifecycle、prepare/apply、cancel/retention 父问题 |
| Module/profile truth | Runtime42 | future control provider 的 Disabled-safe availability | 通用 module catalog/profile 父问题 |
| Dynamic/foreign boundary | Interface05 | versioned bounded envelope 与 transport host | 通用 FFI/output/admission 父问题 |
| Editor command/console | Editor08/25 | Editor authoring adapter、interactive view、diagnostics consumer | Editor command registry、palette、undo、console UI 父问题 |

固定架构仍是 `zircon_app`、`zircon_runtime`、`zircon_editor` 三个 public root package，runtime 内部遵循 `core/{runtime,framework,manager,math,resource}` spine。runtime control authority 应位于 runtime；Editor 只消费同代 catalog/session/output adapter，App 只组合启动和产品策略。不得把 Editor command registry 提升为 runtime authority，不得让 App/transport 直接写 ConfigStore，也不得保留 raw-string compatibility console 作为长期旁路。

### 2.2 当前产品源码物理冻结

本轮冻结所有 tracked `.rs/.toml/.wgsl/.json/.ron/.zui/.zr` 产品文本文件，范围为 root `Cargo.toml`、`zircon_runtime*`、`zircon_app`、`zircon_editor`、`zircon_plugins`、`examples`、`templates` 与 `tests`。算法为 repo-relative path 小写排序，逐文件 lowercase SHA-256，以 `path<TAB>hash` 按 LF 连接且末尾无 LF，再计算 manifest SHA-256。

| 文件 | 行 | 非空行 | bytes | test attrs | ignored | Fingerprint |
|---:|---:|---:|---:|---:|---:|---|
| **18,753** | **3,254,294** | **3,061,482** | **113,858,905** | **20,944** | **248** | `ff30d40ef4e0334c88c47a9dd5176e9692c8a8560beeb29a1e3e0366ceb68b0e` |

冻结对应 HEAD `bee4c707b714738346b49bba15c59468b8bd9b39`、baseline epoch 339。共享 working copy 处于 degraded/dirty 状态，本轮没有接管、覆盖或回滚并行会话修改。报告绑定上述 working-copy 物理快照；implementation 前若这些 owner path 漂移，必须重跑 exact search、字段审查与 fingerprint。

### 2.3 聚焦逐层扫描范围

在全产品冻结之上，本轮逐文件阅读 147 个 control-related source/test 文件，共 27,149 行、943,528 bytes：

| 聚焦组 | 文件 | 行 | bytes | 阅读重点 |
|---|---:|---:|---:|---|
| Config + preferences | 24 | 5,045 | 172,494 | raw store、async flush、atomic backend、overlay、generation、quota、fault tests |
| Operation + dynamic/interface boundary | 17 | 3,738 | 131,882 | registry、admission、snapshot/prepare/apply、cancel、harvest、ABI bounds |
| Diagnostics + log | 37 | 4,923 | 164,131 | devtools/runtime/dynamic diagnostics、log projection、control observability absence |
| Module/profile + App args | 38 | 6,649 | 242,098 | feature truth、builtin assembly、profile、runtime/headless startup surface |
| Editor commands + Console | 31 | 6,794 | 232,923 | descriptor、registry、commandlet、remote flag、dispatch、ZUI/output/tests |

这 147 个文件不是全仓“抽样替代物”，而是从 18,753 文件 exact search 结果反向展开的全部相关 owner surface。render、physics、scene 等 consumer 没有 CVar/console descriptor 命中，因此不把相似的普通 config、debug label 或 operation 字符串误计为实现。

### 2.4 精确搜索与反例隔离

| 搜索 | 当前结果 | 判定 |
|---|---:|---|
| `cvar(s)` / `console variable` / `console command` / `remote console` / `cheat manager` | 1 行 Editor test text，production 0 | 无 runtime control product |
| runtime preset/builtin/profile whole-word `console/cvar/exec` | 0 | 无 capability、provider、module 或产品事实 |
| App production `--exec` / command file / CVar override | 0 | 无 startup/headless operator surface |
| runtime devtools/dynamic diagnostics whole-word `console/cvar/exec` | 0 | 无 catalog/value/session/quota receipt |
| Console ZUI input/history/completion/submit component | 0 | 当前仅 log output/filter/Clear |
| production caller of Editor operation remote-control dispatcher | 0 external caller | 当前只是 in-process API/test seam，不是远程 gateway |

必须隔离的反例是：`executor` 中的字母 `exec`、Editor 的 `callable_from_remote`、navigation operation JSON、日志 pane 名称 Console、ConfigStore 任意 key 和 commandlet `--run` 都不能证明 runtime console/CVar 已存在。

### 2.5 参考物理冻结

参考集保持 Runtime67 的 32 个文件，当前为 26,526 行、22,771 非空行、930,695 bytes。采用与产品冻结相同算法后 fingerprint 为 `77820d825e20adbc5f69d1ce7cb30d1d8f2630015d100eec6998ec09c3e5d085`。

| 参考 | 文件 | 主要用途 | 明确不用作模板的部分 |
|---|---:|---|---|
| Unreal | 12 | typed CVar、source priority、sink、console UI、Exec/Cheat、remote lifecycle | global singleton/raw Exec、RemoteConsole transport-owned access control |
| Godot | 9 | settings metadata/override、debugger registration、bounded remote messages、command queue | Variant/string global authority |
| Bevy | 4 | remote method discovery、transport separation、bounded mailbox/watch | 无 principal/auth policy、HTTP body 明确上限不足 |
| Unity Graphics | 4 | typed debug widget getter/setter/query path、runtime/editor projection、reset | debug UI manager 不是完整 CVar/command authority |
| Fyrox | 3 | reflected/serde quality settings、Editor property apply | typed settings 本身不是 console、layer 或 remote control |

## 3. 当前源码逐层事实

### 3.1 ConfigStore 持久化链增强，但仍不是 CVar authority

`ConfigStore` 的权威状态仍是 `Arc<Mutex<HashMap<String, Arc<serde_json::Value>>>>`，公开任意字符串 `store_value/load_value`、serde typed wrapper 和整表 `snapshot_values`。framework `ConfigManager` 仍以 `set_value(&str, Value)`、`get_value`、`flush` 和 persistence report 为合同。Foundation 实现已经具备 atomic writer、commit fence、dirty/persisted/attempted generation、25 ms debounce、2 s shutdown flush、failure report、bounded latency sample 与 worker panic recovery。

这证明 Runtime55 persistence 基础变强，但 RT67-P1-13..30 仍全部 open：没有 CVar descriptor、closed value kind、default/requested/effective/pending、validator/range/enum、source/layer priority、unset/history、scope、CAS、safe-point apply、observer、replication、recordability 或 privacy。若直接把 console mutation 写进 raw map，仍会让临时值意外持久化、由调用顺序决定优先级，并迫使 hot path 做字符串/JSON/锁访问。

### 3.2 Platform preferences 是可复用持久化底座，不是第二控制真值

`platform/preferences` 已明显超过旧报告中的简单偏好存储：存在 `PreferenceKey`、typed storage boundary、generation ticket、Pending/terminal durability、deadline/cancel authority、retry/eviction、bounded overlay/lane、4,096 entry 默认上限、128 MiB retained-byte 上限、64 MiB 单值硬上限、1,024-byte failure detail 和 atomic file path cache。App 桌面入口可以组合 project/user roots 与 host injection；headless/mobile/browser 若无 host provider 会明确 unavailable。

未来 CVar persistence 应通过 Runtime45 owner 的 narrow adapter 复用这些能力，只允许 descriptor 标记为 Persistable 的 layer 写入，并携带 source/revision/effective receipt。不得让 PreferenceKey 变成 CVar stable ID，也不得让 preference overlay 决定 runtime effective value；否则会产生第二 authority，并混淆 durability 与 apply completion。

### 3.3 RuntimeOperationService 是成熟长操作 primitive，不是命令目录

当前 `RuntimeOperationService` 默认上限为 1,024 tasks、32 in-flight prepares、4 MiB retained bytes、每 tick 8 次 owner apply、terminal 60 s TTL。它先对 foreign/raw JSON 做 bounded admission，再 decode ABI v1 envelope；owner thread 完成 world snapshot 和 apply，worker 完成 prepare，支持 progress/cancel/deadline/terminal/harvest、panic isolation、maintenance alarm 和 fixed-layout status ABI。当前唯一 production handlers 是四个 navigation bake/clear/restore operations。

这比 Runtime67 冻结时更接近可复用 long-command backend，但 registry 仍是 `BTreeMap<String, Arc<dyn RuntimeOperationHandler>>`；handler 输入/command/result 仍是 JSON value，context 只有 runtime core 与 mutable World。它没有 command descriptor/catalog/help/typed args/completion/principal/surface/side-effect/output stream，也没有 unregister、owner generation、registration lease、batch revoke 或 plugin unload drain。因此 RT67-P1-31..40 仍 open；正确方向是 control 层完成 parse/auth/validate 后，把 `Operation` class command 映射到 Runtime41 task，而不是反向把 operation ID 当 console command。

### 3.4 Editor command system 已工程化，但 owner 和语义不同

Editor command descriptor 当前具有 stable `EditorOperationPath`、display/description/category/menu、action/chord/when/keywords、payload schema ID、headless route/name、remote callable、asset write target 与 required capabilities。Registry 使用 `BTreeMap`、monotonic generation 和 generation-cached immutable palette catalog，拒绝 duplicate command/commandlet route/name，并绑定 operation factory；测试覆盖 10,000 条命令后的 catalog generation/cache 与大目录查询。

Commandlet runner 提供 `migrate-assets`、`plugin-list`、`authoring-automation` 三条 typed authoring route、结构化 JSON report、四类 exit code 与 capability evaluation。这是 Editor08 的真实进展，不能关闭 Runtime67 的 runtime catalog、CVar layer、world/player scope、cheat/shipping、replication 或 remote gateway 问题。未来可复用的是 descriptor projection、palette ranking 和 commandlet report 形态，不是复制一套 runtime registry 到 Editor。

### 3.5 Editor “remote”当前是 fail-open 标记，不是安全 gateway

`EditorCommandDescriptor::new` 将 `callable_from_remote` 默认设为 `true`，serde 缺字段时也通过 `default_callable_from_remote` 得到 true；只有部分 default command 显式关闭。commandlet runner 和 Editor operation dispatcher会检查该标记与 required capabilities，但没有 authenticated principal、role、session expiry、target scope、rate limit、replay/dedup、redaction 或 audit。无 source 的 `handle_operation_control_request` 还默认使用 `EditorOperationSource::Remote`。

精确 caller 扫描显示，dispatcher 的 production 符号只存在定义和内部 match，全部实际 invocation 位于 tests；当前没有 socket/HTTP/IPC consumer，所以不是已经暴露的远程执行漏洞。可是大量 reflection/workbench action 被显式标记 remote true，说明未来 transport 接线前必须把默认改成 false，并让 Runtime67 policy/gateway 与 Editor08 adapter共同决定可达性。单个 `bool` 不能承担 permission。

### 3.6 Editor Console 仍是日志 pane

`console_body.zui` 只导入 WorkbenchButton，构造 All/Error/Warning/Info 与 Editor/Runtime/Play/Plugin/Import/Script source filters、scrolling output prototype 和 Clear；footer 没有 text input。`console.rs` 与 retained output 负责“Console ready”状态和日志投影/虚拟化，测试关注 filter、clear、稳定 layout 和 output。没有 focus、IME、selection、paste policy、parser、submit、autocomplete、history、structured result 或 operation progress。

因此 RT67-P1-41/42、P1-55..57 仍 open。实现时应保留当前 log mode，并新增清晰分离的 interactive control session/view；二者可以共享 output projection，但不能让普通日志行冒充 command terminal result。

### 3.7 App、profile、headless 与 diagnostics 尚未接线

runtime session args 当前只有 project、runtime session profile、play scene/report pipe、help 与日志过滤/环境项；没有 `--exec`、command file、CVar override 或 apply phase。Editor `--run` 是三条 authoring commandlet，不是 runtime exec。Hub session UUID/protocol handshake 用于 Editor launch coordination，也不是 runtime remote-console authentication。

runtime feature presets、builtin modules/profile 没有 control capability/provider；`RuntimeDevtoolsDiagnostics` 只投影 modules、services、plugin catalog、native/VM backend 和 diagnostics summary，runtime/dynamic diagnostics 则覆盖 render/physics/animation/store/profile/device/input/scene 等，不含 control catalog generation、requested/effective/source/pending、principal/session/quota/transaction/audit。console variables 的 replicated subset、server authority、late-join snapshot/delta 和 revision gap/resync 也不存在。

所以 RT67-P1-43..65 全部 open。未来任何 App、Editor、headless、script 或 remote surface 都必须消费同一 runtime catalog/policy/output，不得各自开 raw JSON、stdin、socket 或 ConfigStore 后门。

## 4. 参考引擎交叉证据

### 4.1 Unreal：需要吸收完整语义，不照抄全局形态

`IConsoleManager` 把 command 与 variable 分开注册，variable 有 Bool/Int/Float/String 等 typed overload、flags、sink 和 typed read/write；SetBy source 从 Constructor、Scalability、Game/Project/System/DeviceProfile、ConsoleVariablesIni、Hotfix、Commandline、Code、Temp 到 Console 形成明确优先级。focused tests 验证低优先级不能覆盖更强 source、sink 只在有效变化时触发、unregister、typed set/get 和 duplicate/re-register 行为。

Console UI 有 autocomplete、help、运行时 rebuild 和 bounded 50-entry history；command executor 通过 modular feature 注册/撤销并路由 Exec。RemoteConsole 在 shipping build 编译关闭，具有 hello/version、line/cache/queue/timeout bounds 和 game-thread dispatch；但源码明确不承担 access control，而是委托 transport。CheatManager 也有 compile/build/authority gates。Zircon 应吸收 source ordering、lifecycle、UI 与 shipping fail-close，拒绝 global singleton、raw Exec 和 transport-owned permission 这些弱点。

### 4.2 Godot：metadata、override 和 bounded debugger lifecycle

Godot ProjectSettings 为 setting 提供 initial value、basic/internal/restart/order、custom PropertyInfo 与 feature override，并有 override config 加载和 focused tests。EngineDebugger 明确 register/unregister profiler/message capture，RemoteDebugger 用 mutex、每秒 error/warning/character limits、dropped/overflow receipt、poll/flush 管理远程流。`CommandQueueMT` 提供有界内存下的 push、sync、return、flush，并有 wrap/lap 和 worker-pool tests。

这些证据支持 Zircon 把 metadata、override provenance、registration lifecycle、bounded output/watch 和 owner-thread safe point 作为一等合同；Godot 的 global Variant/string access 不应成为 Zircon CVar hot-read 形态。

### 4.3 Bevy：方法发现和 transport 分离是优点，安全边界不是模板

Bevy `RemotePlugin` 与 `RemoteHttpPlugin` 分离，`RemoteMethods` 支持运行时插入 method，`rpc.discover` 输出 OpenRPC 1.3.2 document；instant 与 watching handler进入独立 schedule，main/render app各有 mailbox。全局 mailbox capacity 为 16，单请求 result channel 为 1、watch 为 8，closed watcher 有 cleanup。HTTP 默认绑定 `127.0.0.1:15702`，这比默认公网暴露更谨慎。

但 method insert 会替换同名 handler，没有 owner generation/lease；默认 methods可直接查询和修改 world/resource；focused HTTP 路径会先 collect 整个 request body，源码中没有 authenticated principal 或明确 body hard limit。Zircon 可借鉴 discovery、transport separation 和 bounded channel，不能把 loopback、header 或 JSON-RPC 本身当认证/授权。

### 4.4 Unity Graphics 与 Fyrox：typed settings/UI 连接只是下限

Unity DebugManager 能 register/unregister reset data、管理 panel/widget、生成 query path、区分 EditorOnly/RuntimeOnly，并由 widget getter/setter 投影 live debug value。DebugDisplaySettings 把设置聚合到 runtime/editor Rendering Debugger；focused test 只验证 window state callback，安全、layer、remote 和大规模生命周期证据很窄。

Fyrox renderer settings 是 serde/Reflect typed quality structure，包含 HDR、bloom、shadow precision、CSM 等 preset；Editor settings plugin 通过 property editor修改 model，并在值变化后调用 renderer `set_quality_settings`，错误有日志反馈。这证明 typed model到UI再到runtime apply比任意 JSON key更可靠，但它同样没有完整 CVar source stack、command catalog、principal、replication 或 remote gateway。

## 5. Runtime67 状态刷新

| Runtime67 owner | 当前状态 | 当前源码新增基础 | 为什么仍不能关闭 |
|---|---|---|---|
| P1-01..12 registry/product truth | **Open** | Editor 有 generation-cached command catalog | owner 是 Editor；runtime 仍 exact zero，无 control provider/capability |
| P1-13..30 CVar/config/layer/apply/replication | **Open** | config flush 和 preferences durability/quotas增强 | 无 typed descriptor、layer、effective/pending、safe-point、replication |
| P1-31..42 command/parser/operation/output | **Open** | Runtime41 task admission/lifecycle显著增强 | 仍是 raw JSON operation，无 command schema/principal/output/session |
| P1-43..54 principal/cheat/remote/security | **Open** | Editor 有 capability check、Bevy/Unreal提供反例 | `callable_from_remote` 默认 true，无 auth/scope/audit/quota/wire gateway |
| P1-55..65 Editor/App/headless/diagnostics/plugin ABI | **Open** | Editor commandlet report和palette成熟 | Console仍只读日志；App/profile/diagnostics/runtime ABI未接线 |
| P1-66..72 test/fault/scale/qualification | **Open** | 各底座已有局部 fault/perf tests | 没有 control product，无法证明48 gates或与Unreal竞争性表现 |
| P2-01..16 UX/governance | **Open** | palette/MRU和typed Editor controls可复用 | runtime schema、session和产品入口尚不存在 |

本轮不新建 issue ID。后续实现必须更新 Runtime67 对应行的 evidence/status；不得只在 Runtime107 写“完成”而让 canonical owner 保持旧状态。

## 6. 目标架构与当前 refactor 落点

Runtime67 的目标链保持有效：

```text
Module / Plugin / Product policy
              |
              v
RuntimeControlRegistry ---- immutable generation ----> discovery/help/completion
       |                                  |
       +--> ConsoleCommandCatalog         +--> surface-filtered catalog view
       |
       +--> ConsoleVariableCatalog --> ConsoleVariableLayerStack
                                           |
                                  requested/effective/pending
                                           v
                              ConsoleMutationTransaction
                         parse -> auth -> validate -> CAS -> order
                                           v
                               ConsoleApplyScheduler
                       main/render/world/recreate/restart safe point
                                           |
             +-----------------------------+------------------------+
             v                             v                        v
       hot typed snapshot         RuntimeOperation adapter      replication
                                     progress/cancel          snapshot/delta

ConsolePrincipalPolicy --> ConsoleSession --> ConsoleOutputStream / History / Audit
                                   |
                                   +--> Editor / CLI / Headless / Script adapters
                                   +--> RemoteControlGateway --> Interface transport
```

结合当前源码，重构必须遵循以下落点：

1. 在 runtime `core::framework` 定义 backend-neutral stable IDs、closed descriptors、value/argument kinds、layer/source/apply policy、principal/session/receipt DTO；不依赖 Editor、socket 或 serde_json hot value。
2. 在 runtime owner service 建立唯一 registry、generation snapshot、registration lease、batch revoke 和 unload drain；通过 Runtime42 module/profile truth装配，缺 provider 时 Disabled/Unavailable。
3. ConfigStore 降为 legacy/raw persistence primitive；Persistable CVar 通过 Runtime55/45 adapter落盘，durability completion与runtime apply completion分别出 receipt。
4. `RuntimeOperationService` 只承载 descriptor 标记为 Operation 的长命令；control layer先做 typed parse、principal authorization、target lease、quota和审计，再提交 operation ticket。
5. live variable mutation先计算 requested/effective/pending，执行 validation、expected revision、dependency ordering和transaction rollback；main/render/world/recreate/restart由明确 safe point消费。
6. hot consumer只读typed immutable/atomic snapshot，禁止每帧字符串hash、JSON parse、global mutex、callback或allocation。
7. replication只允许server-owned allowlisted subset，使用catalog generation、value revision、late-join snapshot/delta、gap detection/resync；client mutation必须拒绝或转授权request。
8. Editor command registry继续拥有authoring action。Editor runtime-control palette/view只消费runtime catalog adapter，不复制parser、permission、history或CVar authority。
9. Editor现有Console保留log mode，interactive mode提供input/IME/completion/history/output/progress，并通过明确视觉和focus状态区分。
10. `callable_from_remote` 改为显式 opt-in；remote gateway默认disabled/loopback，强认证、加密、principal、scope、rate/concurrency/payload/output/watch quota、replay/dedup、disconnect cleanup和audit全部在开放端口前成立。
11. App `--exec`、command file、CVar override与headless入口只有在统一provider完成后才接线，并固定 pre-project/post-config/post-world 等阶段与失败 exit receipt。
12. plugin SDK只提交versioned descriptor/handler/value batch并获得owner lease；不得注册raw socket、raw ConfigStore key或Editor-only shadow command。

## 7. 依赖顺序与门禁状态

继续沿用 Runtime67 `CTRL-M0` 至 `CTRL-M9` 的 dependency order，不创建平行 milestone：

| Milestone | 当前状态 | 下一步必须先完成 |
|---|---|---|
| CTRL-M0 Truth freeze | **Review evidence refreshed，implementation 未接受** | stable namespace/ID、Disabled-safe capability、owner边界、zero-provider tests |
| CTRL-M1 Registry/schema | **Not started** | descriptor、generation snapshot、lease/unregister、batch conflict/reload drain |
| CTRL-M2 CVar authority | **Not started** | closed kinds、validation、source priority、unset、requested/effective/pending/CAS |
| CTRL-M3 Transaction/apply | **Not started** | transaction、safe-point scheduler、typed snapshot、observer与no-lock benchmark |
| CTRL-M4 Command execution | **Not started** | parser/schema/context/principal、immediate/operation、bounded output/history |
| CTRL-M5 Security/cheat | **Not started** | surface/role/scope、shipping gates、redaction、quota、audit |
| CTRL-M6 Persistence/replication | **Not started** | narrow Runtime45/55 adapter、server authority、late join、replay/determinism |
| CTRL-M7 Product surfaces | **Not started** | Editor interactive、App/CLI/headless/script adapters，共享同一catalog/policy |
| CTRL-M8 Remote/plugin/ABI | **Not started** | authenticated gateway、versioned wire、plugin lease、disconnect/reload fault matrix |
| CTRL-M9 Qualification | **Not started** | 10k catalog、1k watch、security/fuzz/fault/soak、同机Unreal基线 |

Runtime67 的 48 个 gate 当前仍是 **0 个通过**。preferences/operation/Editor registry 的局部测试是 future gates 的 prerequisites，不具备 runtime control descriptor、principal、session、layer或产品入口证据，不能把任何 CTRL-G gate标记为 pass。

禁止的捷径保持不变：M1 前不得用 Editor registry冒充runtime registry；M2 前不得把raw ConfigStore key叫CVar；M4 前不得把operation ID或字符串split叫command parser；M5 前不得把`callable_from_remote`或capability bool叫permission；M8 前不得开放remote port；M9 前不得宣称达到或超过Unreal。

## 8. 性能、安全与表现超越基线

未来“优于当前 Unreal”至少需要同硬件、同build/profile、同命令/CVar规模和同安全策略比较：

- startup/register/reload：1k/10k/100k descriptor注册、冲突、catalog publish、plugin revoke/drain的时间与峰值内存；
- hot read/apply：game/render/server线程lookup与snapshot读取的p50/p95/p99、allocation、lock/contention、safe-point延迟和frame impact；
- mutation：parse/auth/validate/CAS/transaction/apply/rollback的吞吐、尾延迟、失败可解释性和requested/effective一致性；
- session/output：1k concurrent watch、长输出、slow consumer、overflow/resync、history/redaction与disconnect cleanup；
- remote/security：认证、加密、rotation、downgrade/replay/dedup、rate/quota、fuzz、malformed/deep JSON、body/output hard limit和shipping不可达；
- replication：late join、loss/reorder/duplicate、revision gap/resync、authority rejection和deterministic replay；
- product：Editor/CLI/headless/script同一命令结果、focus/IME/completion、restart/pending展示、structured exit/audit；
- stability：plugin unload、handler panic、worker/channel loss、device/world/session销毁、shutdown和24h soak无泄漏、半事务或stale handler调用。

必须报告均值、尾延迟、方差、allocation、working set、场景、硬件、OS/build、security configuration和fallback reason。没有这些 evidence 时，“比Unreal快”“更安全”“表现更好”均视为未证明。

## 9. 本轮验证边界与下一次复核

本轮验证覆盖 HEAD/epoch、18,753 文件全产品物理计数/指纹、147 文件聚焦逐层阅读、第一方 exact-zero、module/profile/App/diagnostics字段检查、Config/Preference/Operation/Editor command/Console生命周期与测试、32份五引擎参考、Runtime67 issue/gate去重、frontmatter路径、索引链接、Markdown和`git diff --check`。

本轮没有运行 Cargo、App/Editor、remote network、replication、security scanner、fuzzer、soak或benchmark，因为没有修改生产代码且当前不存在可执行 runtime console/CVar路径。实施前必须重新冻结 current source、coordinator leases、exact search、remote caller、descriptor default、operation registration、App args和canonical Runtime67状态；任一漂移都要求重审受影响结论。
