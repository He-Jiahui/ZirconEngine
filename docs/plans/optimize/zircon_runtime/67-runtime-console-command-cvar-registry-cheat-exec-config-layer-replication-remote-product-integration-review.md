---
title: Runtime Console Command、CVar Registry、Cheat/Exec、Config Layer、Replication、Remote 与 Product Integration 工程化差距
category: zircon_runtime
report_id: Runtime67
review_date: 2026-08-20
baseline_head: bea1acf91b909525ab1759e2c800858b0eda6528
baseline_epoch: 335
related_code:
  - Cargo.toml
  - zircon_runtime/Cargo.toml
  - zircon_runtime/runtime-feature-presets.toml
  - zircon_runtime/src/core/runtime/modules
  - zircon_runtime/src/core/runtime/config_store.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/diagnostics/devtools.rs
  - zircon_runtime/src/core/framework/foundation/config_manager.rs
  - zircon_runtime/src/foundation/runtime
  - zircon_runtime/src/operation
  - zircon_runtime/src/dynamic_api
  - zircon_runtime/src/runtime_diagnostics
  - zircon_runtime/src/diagnostic_log
  - zircon_runtime_interface/src
  - zircon_runtime_host/src
  - zircon_app/src/entry/entry_runner/runtime_session_args.rs
  - zircon_editor/assets/ui/editor/console.zui
  - zircon_editor/assets/ui/editor/host/console_body.zui
  - zircon_editor/src/ui/layouts/views/console.rs
  - zircon_editor/src/ui/retained_host/console_output.rs
  - zircon_editor/src/core/commands
  - zircon_editor/src/core/commandlet
  - zircon_plugins
tests:
  - zircon_runtime/src/foundation/runtime/config_manager_tests.rs
  - zircon_runtime/src/operation/tests.rs
  - zircon_runtime/src/operation/tests/phase_indexes.rs
  - zircon_runtime/src/operation/tests/source_guards.rs
  - zircon_editor/src/tests/editor_event/runtime/console.rs
  - zircon_editor/src/tests/host/retained_console_template_body.rs
  - zircon_editor/src/tests/ui/boundary/zui_asset_governance/interaction_policy.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_app/01-product-host-bootstrap-loop-dynamic-runtime-shutdown-review.md
  - docs/plans/optimize/zircon_runtime/01-core-runtime-lifecycle-registry-review.md
  - docs/plans/optimize/zircon_runtime/02-core-runtime-events-tasks-review.md
  - docs/plans/optimize/zircon_runtime/03-core-runtime-diagnostics-profiling-config-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/41-operation-service-handler-registry-admission-prepare-apply-progress-cancel-deadline-harvest-retention-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/42-builtin-runtime-module-catalog-profile-target-feature-selection-extension-registration-capability-load-report-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/45-preference-settings-scope-storage-overlay-bounded-io-generation-fence-durability-migration-multi-process-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/46-engine-module-service-contract-context-factory-descriptor-snapshot-composition-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/50-runtime-manager-resolver-named-service-handle-generation-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/55-runtime-foundation-module-config-event-service-driver-manager-persistence-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/57-platform-host-window-registry-monitor-display-event-loop-application-lifecycle-surface-command-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/65-runtime-scalability-quality-profile-device-profile-capability-tier-dynamic-resolution-frame-budget-lod-feature-fallback-product-integration-review.md
  - docs/plans/optimize/zircon_runtime_interface/01-runtime-dll-abi-ffi-version-handle-foreign-ownership-review.md
  - docs/plans/optimize/zircon_runtime_interface/05-runtime-host-foreign-output-safe-api-ownership-admission-budget-fuse-observability-review.md
  - docs/plans/optimize/zircon_runtime_interface/07-contract-certification-abi-layout-version-skew-cross-language-fuzz-test-architecture-review.md
  - docs/plans/optimize/zircon_editor/08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md
  - docs/plans/optimize/zircon_editor/25-runtime-diagnostics-performance-timeline-console-telemetry-observability-authoring-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
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
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 67 · Runtime Console Command、CVar Registry、Cheat/Exec、Config Layer、Replication、Remote 与 Product Integration 工程化差距

## 1. 结论

Zircon当前没有工程级runtime console command或console variable产品能力。这里的“没有”不是指缺一个输入框，而是稳定命名与typed descriptor、owner generation、注册/撤销lease、变量类型与验证、source/layer优先级、requested/effective/pending状态、safe-point apply、command参数/上下文/副作用、principal/permission、cheat/shipping策略、结构化输出、审计、复制、远程网关以及App/Editor/headless产品入口整条链均未建立。对第一方production Rust范围精确搜索`cvar`、`console_command`、`console variable`、`IConsole`、`cheat manager`与`remote console`，唯一命中是Editor测试文本中的“console command actions”，不是runtime实现。

现有底座并非完全为零。`ConfigStore`与Foundation `ConfigManager`能够存取和异步持久化任意字符串键/JSON值；`RuntimeOperationService`具有有界task、prepare/apply、progress/cancel/deadline和terminal retention；diagnostic log与Editor Console能显示、过滤和清空日志；Editor另有自己的command registry；builtin catalog/profile、dynamic API和runtime interface也提供未来装配点。但这些是不同责任的局部基础：raw JSON配置不是CVar，operation handler不是命令目录，Editor command不是runtime command，日志输出面板也不是交互控制台。把它们用字符串粘接会制造第二权威、权限旁路和热路径锁/解析，而不会形成可维护控制面。

本轮登记 **0项新增P0、72项P1、16项P2和48项验收门禁**。当前Runtime capability/profile、App参数和产品文档都没有宣称交互式console/CVar/remote console可用，所以能力缺失本身不提升为P0；未来若任何profile把该能力标为Available/Enabled，而没有provider、权限、shipping gate或可验证receipt，必须由Runtime42 capability truth gate fail-close并重新评估严重级别。目标架构是`RuntimeControlRegistry + ConsoleCommandCatalog + ConsoleVariableCatalog + ConsoleVariableLayerStack + ConsoleMutationTransaction + ConsoleApplyScheduler + ConsolePrincipalPolicy + ConsoleSession + ConsoleOutputStream + ConsoleHistoryStore + RemoteControlGateway + ConsoleDiagnosticsReceipt`。

本轮只做静态review与计划记录，没有修改production、tests、Cargo、ABI或参考源码，也没有运行Cargo、产品进程、网络安全、复制、fault、soak或benchmark，因此不能宣称控制面已实现，更不能宣称性能、安全性或易用性超过Unreal。用户已要求暂停tooling优化，本篇不新增脚本、代码生成器或tooling迁移任务；未来工具迁Rust时只能消费这里定义的runtime schema，不能建立第二套控制真值。

## 2. 审查边界、规模与currentness

### 2.1 Zircon物理冻结

| 冻结组 | 文件 | 行 | bytes |
|---|---:|---:|---:|
| Manifest、catalog、profile与capability truth | 315 | 15,559 | 555,597 |
| Runtime config、operation、diagnostics、dynamic/interface控制底座 | 68 | 10,982 | 368,562 |
| App、Editor Console/command与产品consumer | 109 | 13,371 | 480,927 |
| 聚焦external/inline-test-bearing文件 | 31 | 5,194 | 176,294 |
| 去重合计 | **523** | **45,106** | **1,581,380** |

Zircon冻结集fingerprint为SHA-256 `e31d719d45fea5e679dd9bd99a7581f0f2787a0f314884fa2aa07ee57914ea34`。算法沿用Runtime66：相对路径转`/`、排序去重，以`path|lowercase per-file SHA-256`逐行编码，LF连接且末尾不追加LF，再对UTF-8 payload计算SHA-256。四个分组指纹依次为`2ed6fa37e5d419190dae45273609794f7f05a810eab272a25576f87a14e75933`、`cb5aff369789e68f25c4259c0da9a44510279ae21074e3facfe76fbf9c4c1a44`、`73d96f9fee3f8e75de2bab7d8a4ae28c6714959c04c8ac22c1cf11c18551b69b`与`64282cb8b102bb604bacd489e60e57fde02e10c72178ca8566533bddcfcd583f`。

冻结时有29个入选working-tree路径带修改标记，集中在App frame cadence、Editor commandlet/command registry/Console、Runtime diagnostics/log/dynamic API/operation及其测试。当前结论按共享working copy冻结；实施前必须重验这些路径、第一方精确零搜索、Cargo feature/profile/catalog和全部指纹。本报告证明成文时的物理事实，不是可绕过current-source复核的永久设计许可。

### 2.2 参考物理冻结

| 参考 | 文件 | 行 | bytes |
|---|---:|---:|---:|
| Unreal Console Manager、Console UI、Remote Console、Cheat | 12 | 14,555 | 494,267 |
| Godot ProjectSettings、Debugger、RemoteDebugger、CommandQueue | 9 | 4,494 | 172,710 |
| Bevy Remote与Diagnostic | 4 | 5,070 | 176,501 |
| Unity Graphics DebugManager/DebugUI | 4 | 1,531 | 56,572 |
| Fyrox renderer/editor typed settings | 3 | 876 | 30,645 |
| 合计 | **32** | **26,526** | **930,695** |

参考集fingerprint为SHA-256 `04dbdfa0efc60afefdd0f1f5d15636ac79bdbdb93950ac2c5f52438a55a0d91c`，采用同一算法。Unreal是typed CVar、source priority、command、sink、autocomplete、cheat和remote lifecycle主参考；Godot是带metadata/override的ProjectSettings、debugger注册生命周期与有界远程消息主参考；Bevy是方法注册、OpenRPC discovery、transport分离和bounded mailbox对照；Unity Graphics与Fyrox用于比较typed debug/settings UI如何连接getter/setter和runtime apply。Bevy HTTP和Unreal RemoteConsole在本轮范围内都不能充当Zircon安全模板：前者未提供可复用认证模型，后者源码还明确把访问控制委托给transport。

### 2.3 本轮拥有与明确不拥有

- Runtime67拥有统一runtime control registry、command/CVar schema、layer/transaction/apply、principal/permission、cheat/exec、output/history、replication/remote gateway及App/Editor/headless产品接入合同。
- Runtime03与Runtime55继续拥有通用diagnostics/config/event/persistence；Runtime45拥有用户/项目Preference与多进程持久化。Runtime67只定义哪些CVar可桥接未来`ConfigAuthority`、如何携带source/effective receipt，不能复制文件存储owner。
- Runtime41拥有通用长操作task、progress/cancel/deadline/backpressure。Runtime67将长命令投影到该服务，但拥有command descriptor、typed arguments、principal/context和console output projection。
- Runtime24拥有stable identity/generation；Runtime42拥有profile/catalog/capability truth；Runtime46/50拥有module/service/manager装配与解析；Runtime01/02拥有lifecycle/task。Runtime67消费这些父合同，不建立私有弱化版本。
- Runtime57拥有platform/host lifecycle；Runtime65拥有scalability/device/quality policy。Runtime67为这些owner提供受控命令/CVar入口，不反向接管window或quality真值。
- Interface01/05/07拥有ABI、remote transport与dynamic control-plane host安全边界；Plugins01拥有SDK/package贡献与卸载。Runtime67定义wire-visible control schema、owner lease和权限语义，并要求这些父owner承载。
- Editor08拥有Editor自身command/menu/shortcut/undo，Editor25拥有diagnostics/profiler/console/debugger UI。Runtime67定义runtime命令/CVar控制面，不能把Editor command registry或日志pane提升为runtime authority。
- App01拥有启动/session/host。Runtime67只定义`--exec`、command file、CVar override与headless operator接入顺序。用户暂停tooling优化，因此脚本生成、shell helper和迁移工具不在本篇范围。

## 3. 当前实现的真实能力与断裂

### 3.1 ConfigStore能持久化JSON，但不是CVar authority

`core/runtime/config_store.rs`的权威状态是`Arc<Mutex<HashMap<String, Arc<serde_json::Value>>>>`。`store_value`按任意字符串直接替换JSON，`snapshot`克隆全部键值；framework `ConfigManager`公开`set_value(&str, Value)`、`get_value`与`flush`。Foundation manager把整个map序列化到单一JSON文件，使用单worker、25 ms默认debounce和2秒shutdown flush，并通过`ZIRCON_CONFIG_PATH`或用户全局路径选择文件。

这条链可作为未来持久化primitive，却没有CVar registry、typed kind/default/range/enum、source/layer优先级、owner、revision、requested/effective/pending、apply stage、observer、history/unset或shipping policy。生产中还有多处直接读写raw config manager/store。若直接把每个CVar映射为任意key/value，会继承Runtime55已记录的全Store snapshot、双manager、启动层覆盖和持久化问题，并让临时console mutation意外变成永久配置。

### 3.2 RuntimeOperationService是长操作底座，不是命令目录

`operation`模块已经具有有界task、prepare/apply阶段、terminal retention、progress/cancel/deadline和维护路径；handler registry使用`BTreeMap<String, Arc<dyn RuntimeOperationHandler>>`，空ID和重复ID会拒绝。它值得复用来承载cook、reload、capture等长命令。

但operation envelope仍以字符串`operation_id`和`serde_json::Value` payload/result为核心；handler没有命令help、typed argument schema、completion、principal、world/player/session context、side-effect classification、surface allowlist或结构化console stream。registry也缺少owner generation、registration lease、unregister/batch revoke和reload drain。把任意console文本翻译成JSON operation只能把解析、权限和生命周期责任藏到handler内部。

### 3.3 Editor Console目前是日志输出面板

`console_body.zui`只有level/source过滤按钮、virtualized output和Clear；`console.zui`是header与text panel；Rust layout发布“Console ready”和状态输出。`console_output.rs`只处理日志行虚拟化。对应测试验证clear、filter、source和output，还明确断言模板没有`FocusConsole`。因此这个名称在当前代码里语义清楚：它是diagnostic log history pane，并不是可输入、解析、补全、执行runtime命令的交互控制台。

未来UI可以复用日志投影与虚拟化，但必须增加独立的`ConsoleSession`、输入/IME、catalog snapshot、typed completion、principal surface和output cursor。不能让UI直接访问`ConfigStore`、Editor command registry或operation handler map，也不能因为面板名叫Console就把Runtime42能力标为Available。

### 3.4 App、headless、diagnostics与plugin均未接线

`runtime_session_args.rs`只解析project、profile、play scene/report、log level/filter和help，没有`--exec`、command script、CVar override或remote console启动策略。Editor host runtime services只暴露config、asset、diagnostics和VM；`devtools` snapshot只有modules/services/plugin catalog/backend/diagnostics summary，没有control catalog、effective value/source、principal/session、transaction或audit receipt。builtin/runtime plugin catalog与feature presets也没有console/CVar/cheat/remote capability。

第一方plugin没有贡献command/CVar的SDK surface，scripting没有受控调用边界，server/headless没有operator interface，replication没有authoritative CVar snapshot/delta。当前唯一精确命中是Editor治理测试中的文本，不是产品caller。这些负证据支持“能力未实现”，也支持本轮0新增P0；它们不支持用未受控debug hook快速补入口。

## 4. 五套参考实现的语义差异

| 参考 | 已验证的工程语义 | Zircon当前差异 | 应吸收/拒绝 |
|---|---|---|---|
| Unreal | `IConsoleManager`区分typed variable/command，支持注册、查找、遍历、unregister、sink、callback、history、tag、autocomplete；flags含Cheat/ReadOnly/RenderThreadSafe/Scalability/Preview等；set source从Constructor到Console具有明确优先级，测试覆盖source、unregister和sink | 无registry、flags、priority、sink、typed handle、input或测试 | 吸收typed registry、layer priority、lease、sink/safe-point和测试矩阵；不复制全局裸指针或无owner legacy路径 |
| Unreal Console/Cheat/Remote | UConsole有输入、补全与有界50条history；CheatManager有build/authority边界；RemoteConsole shipping编译关闭、带exec/connection timeout | Editor只有log pane；无cheat/shipping/remote lifecycle | 吸收产品形态和fail-close；拒绝RemoteConsole“transport自行负责访问控制”的安全弱点，Zircon必须显式认证、角色、审计 |
| Godot | ProjectSettings有initial/basic/internal/restart metadata和override；EngineDebugger支持profiler/capture注册撤销；RemoteDebugger有mutex、消息/速率上限、dropped/overflow、poll/flush；CommandQueue测试并发队列 | raw JSON无metadata/restart/override；无debugger registry与bounded remote channel | 吸收metadata、override、register/unregister、bounded queue和overflow receipt；不把ProjectSettings直接当热路径CVar |
| Bevy | RemotePlugin注册instant/watching方法，OpenRPC discovery与HTTP transport分离；HTTP默认loopback，mailbox容量16并在固定schedule drain，watch disconnect会清理 | 无method schema/discovery/transport separation/watch lifecycle | 吸收control/transport分离、schema discovery、bounded mailbox与watch cleanup；未看到auth不能作为省略安全的依据 |
| Unity Graphics | `DebugUI` widget以typed getter/setter、runtime/editor flags和hidden callback连接状态；DebugManager负责panel注册、移除、dirty/reset | 无typed runtime debug widget/catalog/effective state | 吸收typed projection和panel lifecycle；UI不能成为CVar真值或私有持久化owner |
| Fyrox | renderer/editor settings使用Reflect+Serde typed结构，Inspector修改后显式apply quality | Zircon config consumer多为任意JSON/key | 吸收typed settings与显式apply；Fyrox本轮没有通用console/CVar证据，不能包装成同等成熟参考 |

## 5. P0审计

本轮 **0项新增P0**。当前profile/catalog/App/Editor没有宣称交互式runtime console、CVar、cheat或remote control可用；Editor Console的源码和测试都明确是log output语义，不构成假能力。Runtime55的config P0、Runtime41的operation父问题、Runtime42的capability truth、Interface05/07的remote/control-plane安全以及Editor08/25的产品面缺口继续由原报告唯一拥有，不在这里重复累计。

以下条件会触发重新分级：产品把control capability标为Available但没有provider；shipping构建可绕过policy执行cheat/destructive命令；远程入口无认证/授权而可变更权威状态；CVar layer把低优先级或未验证值静默覆盖权威值并导致持久数据损坏。未发生这些事实前，按P1工程缺口治理。

## 6. P1工程化差距

| ID | 差距 | 当前证据/风险 | 目标重构 |
|---|---|---|---|
| RT67-P1-01 | 无runtime control service | Core只装配config/diagnostics/tasks/services | 建立`RuntimeControlRegistry`并由module/service lifecycle唯一装配 |
| RT67-P1-02 | 无稳定命名与typed ID | 任意字符串key/operation ID | command/CVar使用namespace、stable ID、canonical name与typed handle |
| RT67-P1-03 | command与variable类型未分离 | 只能用JSON或operation模拟 | `ConsoleCommandCatalog`和`ConsoleVariableCatalog`共享基础descriptor但保持不同语义 |
| RT67-P1-04 | 无descriptor schema/version | 无kind、help、flags、compatibility | descriptor记录schema version、value/arg type、owner、policy和lifecycle |
| RT67-P1-05 | 无owner module/plugin/generation | 注册项无法归属或判断stale | 绑定module/plugin instance、owner generation和provider receipt |
| RT67-P1-06 | 无lease/unregister/batch revoke | operation handler不能安全撤销 | registration返回lease，owner unload原子撤销整批并等待in-flight drain |
| RT67-P1-07 | 无duplicate/conflict policy | 字符串重复只在局部map拒绝 | canonical/alias冲突、版本替换和批注册all-or-nothing |
| RT67-P1-08 | 无immutable generation catalog | discovery会读可变map/锁 | 发布immutable catalog snapshot与monotonic generation |
| RT67-P1-09 | 无discovery/filter/help/tag | 产品无法解释控制面 | 按surface/category/module/permission/tag过滤并生成typed help |
| RT67-P1-10 | 无依赖与apply ordering | CVar/command间先后只能隐含 | descriptor声明依赖、conflict和safe apply order，检测cycle |
| RT67-P1-11 | 无deprecation/redirect/migration | rename会破坏脚本和配置 | versioned redirect、warning、sunset和persisted key migration receipt |
| RT67-P1-12 | capability/profile没有控制面事实 | catalog零条目，未来易私接 | 定义Disabled-safe capability与provider requirement，缺provider fail-close |
| RT67-P1-13 | raw ConfigStore被误当CVar | 任意key/JSON可存取 | CVar authority只通过typed catalog；ConfigStore降为受控持久化backend |
| RT67-P1-14 | 无typed value domain | JSON可在运行时任意变形 | 支持Bool/Int/Float/String/Enum/Bitset等closed kind与明确转换规则 |
| RT67-P1-15 | 无default/requested/effective/pending | 读取值无法解释是否已应用 | 每项发布四态、revision、source和last apply receipt |
| RT67-P1-16 | 无validator/range/enum约束 | invalid值进入store后才由consumer失败 | registration绑定pure validator、range、step、enum domain和cross-field preflight |
| RT67-P1-17 | 无clamp/fallback receipt | 调整可能静默改变用户意图 | reject/clamp/fallback必须显式policy并返回requested/effective差异 |
| RT67-P1-18 | 无source/layer优先级 | 启动、磁盘、CLI、console互相覆盖 | `ConsoleVariableLayerStack`冻结优先级与可写性，禁止调用顺序决定结果 |
| RT67-P1-19 | 无history/unset | 无法回到下层值或追责 | 每layer保存bounded mutation history，unset揭示下一有效source |
| RT67-P1-20 | 无platform/device/project/CLI/console/hotfix scope | 任意key混在同一map | typed layer和scope owner，跨scope写入需显式promotion |
| RT67-P1-21 | 无live/restart/recreate策略 | 所有配置看似立即生效 | metadata标记Live/SafePoint/WorldRestart/RendererRecreate/AppRestart |
| RT67-P1-22 | 无apply stage/thread affinity | callback可能在任意线程改权威状态 | `ConsoleApplyScheduler`路由main/render/server/world safe point |
| RT67-P1-23 | 无batch transaction | 多项设置可能半应用 | `ConsoleMutationTransaction`先解析/鉴权/验证/排序，再原子commit或rollback |
| RT67-P1-24 | 无sink/observer safe-point合同 | consumer只能poll或私加callback | generation sink、bounded observer、coalescing和panic isolation |
| RT67-P1-25 | 无game/render/server/world snapshot | 一份global map污染多实例 | 按runtime/world/view/device scope发布immutable typed snapshots |
| RT67-P1-26 | 无typed handle/hot read | 每帧字符串hash、锁、JSON/downcast风险 | 注册时解析handle，hot path只读immutable/atomic typed effective value |
| RT67-P1-27 | 无revision/CAS/stale拒绝 | UI/remote并发会last-writer-wins | mutation携expected revision/catalog generation，冲突返回typed receipt |
| RT67-P1-28 | 无持久化allowlist与ConfigAuthority桥 | 临时console值可能写入全Store | 仅Persistable变量桥接Runtime55未来authority，secret/session/cheat默认不落盘 |
| RT67-P1-29 | 无replication authority/late join | 客户端与server可各自漂移 | server-owned replicated subset、snapshot/delta、revision和late-join收敛 |
| RT67-P1-30 | 无save/replay/determinism/privacy语义 | CVar可能污染回放或泄露秘密 | descriptor声明recordability、determinism impact、save scope和redaction |
| RT67-P1-31 | 无runtime command catalog | 只有Editor command与operation map | 建立typed command descriptor、registration lease与generation snapshot |
| RT67-P1-32 | 无typed parser/argument schema | 只能临时split字符串或塞JSON | token/quote/escape/parser与named/positional/optional/repeated typed args |
| RT67-P1-33 | 无overload/version ambiguity policy | 同名命令未来不可稳定解析 | 禁止歧义或用schema version/signature显式选择，completion展示差异 |
| RT67-P1-34 | 无world/player/session/viewport context | handler无法知道作用域和authority | `ConsoleExecutionContext`持稳定lease、principal、surface与目标scope |
| RT67-P1-35 | 无side-effect/idempotence/transaction class | read/query与破坏操作等价 | descriptor声明ReadOnly/Mutation/Destructive/Operation及retry/idempotence |
| RT67-P1-36 | 无immediate/async contract | UI无法解释完成语义 | bounded immediate result或typed operation ticket，禁止假成功字符串 |
| RT67-P1-37 | OperationService raw JSON不是命令catalog | handler内部承担解析和权限 | control层验证后才桥接Runtime41 typed operation adapter |
| RT67-P1-38 | command handler无owner unload drain | plugin reload可留下旧代码调用 | owner lease撤销、停止admission、cancel/drain、terminal后卸载 |
| RT67-P1-39 | 无结构化输出流 | 日志与命令结果无法关联 | `ConsoleOutputStream`记录request、severity、channel、chunk cursor和terminal |
| RT67-P1-40 | 无progress/cancel/deadline投影 | 长命令只能阻塞或丢状态 | Runtime41 task状态映射为session-scoped progress/cancel/deadline |
| RT67-P1-41 | 无multiline/script/batch策略 | exec文件会产生注入和半执行 | versioned script grammar、include/sandbox/transaction/error policy与配额 |
| RT67-P1-42 | 无completion/history/MRU/exact help | Editor Console没有输入 | catalog-driven completion、bounded principal-local history和敏感参数剔除 |
| RT67-P1-43 | 无principal/auth/session identity | local/remote请求无法归责 | `ConsolePrincipal`、authenticated session、origin和expiry进入每次执行 |
| RT67-P1-44 | 无surface allow policy | Editor/CLI/remote/script可调用同一全集 | 每descriptor声明Local/Editor/CLI/Headless/Remote/Script allowlist |
| RT67-P1-45 | feature capability被误当permission | Available不等于可执行 | capability只描述provider事实，permission单独由principal policy判定 |
| RT67-P1-46 | 无role/object/world/project scope | 只有进程级开关 | role+capability+target scope共同授权并固定lease，避免TOCTOU |
| RT67-P1-47 | 无cheat/dev/shipping gate | debug命令可能进入发行版 | compile/profile/runtime三重gate；shipping缺省不注册或不可达 |
| RT67-P1-48 | 无sensitive/redaction classification | token/path/value可能进日志/history | 参数、输出、CVar和audit字段具分类与端到端redaction |
| RT67-P1-49 | 无每命令rate/concurrency/byte/depth quota | remote/script可放大CPU/内存 | principal/surface/command多层token bucket、并发和payload/output上限 |
| RT67-P1-50 | 无audit receipt | 谁改了什么无法追踪 | append-only bounded audit含principal、descriptor、target、before/after摘要和结果 |
| RT67-P1-51 | 无remote gateway/transport分离 | 未来易把socket写进registry | `RemoteControlGateway`消费中立control API，Interface05拥有transport |
| RT67-P1-52 | 无auth/encryption/bind/port lifecycle | 暴露入口会成为远程执行面 | 默认disabled/loopback，强认证、加密、bind policy、rotation与shutdown drain |
| RT67-P1-53 | 无request ID/replay/dedup/order | 重试可能重复破坏操作 | request nonce/idempotency key、sequence/window和terminal dedup ledger |
| RT67-P1-54 | 无disconnect cancel/watch backpressure/fail-close | 断线后operation/watch可能泄漏 | session owner批量取消/解绑，bounded watch queue与overflow/resync receipt |
| RT67-P1-55 | Editor Console仅输出 | ZUI无输入组件 | 保留log mode，新增清晰分离的interactive control session/view |
| RT67-P1-56 | 无input/focus/keyboard/IME | 测试明确无`FocusConsole` | text editing、IME、selection、paste policy、submit/cancel和focus lifecycle |
| RT67-P1-57 | 无autocomplete/help/value/source preview | 用户无法检查effective事实 | catalog snapshot驱动补全、descriptor help、requested/effective/layer preview |
| RT67-P1-58 | App无`--exec`/command file/CVar override/order | startup args只有项目/日志等 | 定义pre-project/post-config/post-world阶段和失败时exit receipt |
| RT67-P1-59 | 无headless/server operator interface | 无窗口产品不可受控 | authenticated stdin/IPC/remote adapter，共享同一policy/catalog/output |
| RT67-P1-60 | 无scripting invocation boundary | script可被迫绕到raw manager | 只暴露allowlisted typed command/CVar API、budget和principal delegation |
| RT67-P1-61 | diagnostics无catalog/value/session snapshot | devtools看不到control事实 | snapshot catalog generation、effective/source/pending、session和quota摘要 |
| RT67-P1-62 | mutation无telemetry/log receipt | config变化与故障不可关联 | structured event关联transaction/request/operation和apply duration/failure |
| RT67-P1-63 | UI无requested/effective/source/restart状态 | 值显示会误导 | 显示层、override provenance、pending/restart、validation和last receipt |
| RT67-P1-64 | plugin SDK无贡献surface | 插件只能私建控制入口 | SDK注册batch descriptor/handler/variable owner lease，unload原子撤销 |
| RT67-P1-65 | remote ABI/wire无schema/version | JSON形状会漂移 | Interface owner定义versioned envelope、closed errors、limits和compat matrix |
| RT67-P1-66 | 无registry lifecycle/duplicate/hot reload测试 | 只测局部operation/config | property/fault测试注册、冲突、撤销、stale、batch rollback和reload drain |
| RT67-P1-67 | 无layer/unset/transaction测试 | source优先级无法证明 | exhaustive precedence、unset、CAS、validation、rollback和persistence矩阵 |
| RT67-P1-68 | 无apply-stage/render hot-read测试 | callback/锁可能进入帧路径 | main/render safe-point、generation visibility、no-lock/no-allocation benchmark |
| RT67-P1-69 | 无permission/shipping/cheat/remote测试 | 安全策略不可证 | role/surface/profile/build/auth/redaction/quota/replay/fault matrix |
| RT67-P1-70 | 无CLI/Editor/headless E2E | 产品接线为零 | 同一命令跨三surface结果一致，权限和输出差异可解释 |
| RT67-P1-71 | 无scale benchmark | 目录、watch、history可能无界 | 10k变量/命令、并发读写、1k watch、长输出、reload和late join基准 |
| RT67-P1-72 | 无竞争性资格证据 | 无startup/frame/lookup/allocation/remote数据 | 固定场景对比Unreal同类控制面成本、安全门、恢复和操作闭环 |

## 7. P2治理与可维护性项

| ID | 差距 | 建议 |
|---|---|---|
| RT67-P2-01 | alias和模糊搜索治理不足 | alias只做versioned redirect；fuzzy ranking不改变canonical identity |
| RT67-P2-02 | help/label未纳入本地化 | stable ID与localized label/help分离，fallback可审计 |
| RT67-P2-03 | 无收藏、置顶与最近使用 | 仅保存非敏感principal-local UI偏好，不影响registry真值 |
| RT67-P2-04 | command palette可能重复控制面 | palette消费同一catalog/session，不复制parser或permission |
| RT67-P2-05 | typed值缺丰富编辑器 | enum、bitset、range、color等由schema投影适配控件 |
| RT67-P2-06 | 无layer/value diff视图 | 展示每层值、winner、requested/effective和last mutation |
| RT67-P2-07 | 无命名profile/preset | preset是versioned transaction artifact，不是隐藏批量脚本 |
| RT67-P2-08 | macro录制易泄露或重放危险命令 | 默认关闭，按descriptor recordability和redaction过滤 |
| RT67-P2-09 | 项目命令history迁移语义不清 | history按project/principal/schema分区并有retention |
| RT67-P2-10 | 无CVar趋势与变化可视化 | 只对allowlisted telemetry值采样，控制高基数和隐私 |
| RT67-P2-11 | help缺少文档深链 | descriptor可附稳定doc ID，断链在资格门检查 |
| RT67-P2-12 | shell completion未定义 | 未来由runtime schema导出；tooling迁Rust前不新增生成器 |
| RT67-P2-13 | 无remote client SDK | wire稳定后再生成/实现typed client，不暴露任意JSON escape hatch |
| RT67-P2-14 | 无常用诊断preset/quick action | quick action仍走完整permission/transaction/audit链 |
| RT67-P2-15 | legacy config/command导入未规划 | 显式迁移manifest、dry run、冲突报告与可回滚备份 |
| RT67-P2-16 | 交互控制台a11y细节未定义 | 屏幕阅读、键盘导航、live region节流、high contrast和reduced motion |

## 8. 目标架构与数据流

```text
Module / Plugin descriptors
          |
          v
 RuntimeControlRegistry ---- immutable catalog generation
      |            |
      v            v
CommandCatalog   VariableCatalog ---- ConfigAuthority bridge (persistable only)
      |            |
      |            v
      |      VariableLayerStack
      |    default/platform/device/project/CLI/console/remote
      |            |
      +------------+------------------+
                                       v
Surface -> ConsoleSession -> PrincipalPolicy -> MutationTransaction
 Editor      CLI/headless      auth/role/       parse/validate/CAS/
 local       remote/script     scope/quota       dependency/preflight
                                       |
                    +------------------+------------------+
                    v                                     v
              ApplyScheduler                    RuntimeOperationService
          main/render/world safe point            long-running command
                    |                                     |
                    +------------------+------------------+
                                       v
                      requested/effective/pending receipt
                                       |
                       OutputStream + History + Audit
                                       |
                       diagnostics / replication / remote
```

关键不变量：

1. command、CVar、Editor command、operation、diagnostic log和persistent config是六种不同事实；它们可以桥接，但任何一层不得冒充另一层authority。
2. 每个注册项必须有stable ID、schema version、owner generation和registration lease；owner撤销后旧handle、session、completion与watch都必须拒绝或终止。
3. CVar requested、effective、pending、default和各source layer必须同时可解释；低优先级set不能因调用较晚覆盖高优先级值，unset只揭示下一层。
4. mutation在解析、鉴权、验证、revision检查、依赖排序全部成功前不得改变权威状态；batch失败必须all-or-nothing并产生receipt。
5. frame/render/server热路径只消费typed immutable snapshot或atomic handle，不做字符串查找、JSON解析、锁内callback、分配或远程I/O。
6. principal、surface、target lease和permission必须固定到一次执行；capability availability不能替代authorization，cheat/shipping默认fail-close。
7. remote gateway只是受限consumer，必须认证、加密、限流、审计、去重和断线清理；transport不得直接调用handler或ConfigStore。
8. 长命令复用Runtime41 operation lifecycle；短命令也必须返回typed terminal result。日志输出、命令输出和audit可关联但保留独立retention/redaction策略。

## 9. 依赖顺序与实施里程碑

| 里程碑 | 目标 | 依赖 | 完成证据 |
|---|---|---|---|
| CTRL-M0 · Truth freeze | 固化零能力、owner route、catalog disabled状态与冻结集 | 本篇、Runtime42 | exact zero search、fingerprint、capability fail-close |
| CTRL-M1 · Registry/schema | typed IDs、descriptor、catalog generation、owner lease与batch registration | Runtime24/42/46/50、Plugins01 | duplicate/conflict/stale/unload/property tests |
| CTRL-M2 · CVar authority | typed value、validation、layer、requested/effective、revision与unset | M1、Runtime03/55/65 | exhaustive precedence、CAS、fallback、snapshot tests |
| CTRL-M3 · Transaction/apply | preflight/commit/rollback、dependency和main/render/world safe point | M2、Runtime01/02/41 | fault injection、no partial mutation、thread-affinity tests |
| CTRL-M4 · Command execution | parser、typed args、context、side effect和operation bridge | M1/M3、Runtime41 | parser fuzz、sync/async/target lease/cancel矩阵 |
| CTRL-M5 · Security/session | principal、surface、role、cheat/shipping、quota、redaction与audit | M1-M4、Interface05/07 | threat model、build/profile/auth/replay/fault matrix |
| CTRL-M6 · Persistence/replication | ConfigAuthority allowlist、server authority、snapshot/delta/late join | M2-M5、Runtime45/55 | restart/multiprocess/late join/replay/determinism evidence |
| CTRL-M7 · Product surfaces | App CLI、Editor interactive view、headless和script adapter | M4-M6、App01、Editor08/25 | CLI/Editor/headless E2E与同catalog结果 |
| CTRL-M8 · Remote gateway | versioned wire、transport、auth、watch/backpressure/disconnect | M5-M7、Interface01/05/07 | loopback/remote security、quota、soak、disconnect cleanup |
| CTRL-M9 · Qualification | scale、hot path、startup、reload、security和竞争性基准 | M0-M8、O11/O14 | 10k catalog、frame cost、fault/soak、Unreal对照报告 |

M1前不得在各module里增加私有字符串command map；M2前不得把任意ConfigStore key称为CVar；M5前不得开放shipping cheat或remote mutation；M7前不得用Editor command registry冒充runtime command；M9前不得宣称达到或超过Unreal。

## 10. 验收门禁

| Gate | 验收内容 |
|---|---|
| CTRL-G01 | physical exact search、capability/profile/catalog与产品入口事实保持一致，未实现能力为Disabled/Unavailable |
| CTRL-G02 | 无registry/provider/policy时所有surface fail-close并返回typed receipt |
| CTRL-G03 | command/CVar stable ID、canonical name、namespace、schema version和alias规则无歧义 |
| CTRL-G04 | descriptor完整表达kind/args/default/help/flags/policy/scope/lifecycle和compatibility |
| CTRL-G05 | 每项绑定module/plugin owner generation与registration lease，stale handle拒绝 |
| CTRL-G06 | duplicate/conflict/batch registration原子失败，不留下半目录 |
| CTRL-G07 | immutable catalog generation可按surface/permission过滤且reader无需持写锁 |
| CTRL-G08 | plugin reload先停admission、撤销lease、cancel/drain in-flight，再卸载旧代码 |
| CTRL-G09 | CVar closed value kinds与conversion不允许JSON形状静默漂移 |
| CTRL-G10 | range/enum/custom/cross-field validation在commit前完成并给出reject/clamp/fallback receipt |
| CTRL-G11 | default/platform/device/project/persisted/CLI/console/remote层有固定优先级和可写策略 |
| CTRL-G12 | history/unset揭示下一有效层，bounded retention不泄露敏感值 |
| CTRL-G13 | batch mutation通过parse/auth/validate/CAS/order后all-or-nothing commit/rollback |
| CTRL-G14 | expected revision/catalog generation使并发Editor/remote stale写入可检测 |
| CTRL-G15 | Live/MainSafePoint/RenderSafePoint/WorldRestart/Recreate/AppRestart按metadata执行 |
| CTRL-G16 | sink/observer锁外、bounded、coalesced、panic隔离且只在effective change触发 |
| CTRL-G17 | game/render/server hot read无字符串hash、JSON parse、全局mutex、allocation或callback |
| CTRL-G18 | 仅Persistable allowlist进入Runtime55 ConfigAuthority，session/secret/cheat默认不落盘 |
| CTRL-G19 | 每次set返回requested/effective/pending/source/revision/apply/rollback完整receipt |
| CTRL-G20 | replicated变量只由授权server source写入，client mutation明确拒绝或转request |
| CTRL-G21 | late join snapshot与delta按revision有序收敛，gap/overflow触发resync |
| CTRL-G22 | save/replay/determinism/privacy metadata决定记录、恢复、hash和redaction行为 |
| CTRL-G23 | parser对quote/escape/Unicode/size/depth/malformed输入有fuzz和bounded failure |
| CTRL-G24 | positional/named/optional/repeated参数按schema转换，错误保留argument位置和expected type |
| CTRL-G25 | world/player/session/viewport target以稳定lease固定，销毁或替换时执行拒绝 |
| CTRL-G26 | immediate命令有严格时限，超限转operation；两者均有terminal result |
| CTRL-G27 | 长命令桥接Runtime41 progress/cancel/deadline/retention，不复制task authority |
| CTRL-G28 | output stream按request/cursor有byte/chunk/backlog上限、redaction、overflow和terminal |
| CTRL-G29 | multiline/script/include/batch有版本、sandbox、配额、事务和失败策略 |
| CTRL-G30 | completion/help/history来自同代catalog；MRU不记录secret参数或被撤销命令 |
| CTRL-G31 | 每次执行有authenticated principal、origin、session expiry和不可伪造request identity |
| CTRL-G32 | Local/Editor/CLI/Headless/Remote/Script surface allowlist逐命令/CVar验证 |
| CTRL-G33 | role、capability、project/world/object scope同时授权，执行期间无TOCTOU |
| CTRL-G34 | cheat/dev命令在shipping compile/profile/runtime三层fail-close且无隐藏alias旁路 |
| CTRL-G35 | sensitive argument/value/output/audit字段端到端redact，错误与telemetry不回显秘密 |
| CTRL-G36 | audit记录principal/descriptor/target/before-after摘要/result并具bounded durable policy |
| CTRL-G37 | remote transport强认证、加密、证书/credential rotation与protocol downgrade拒绝 |
| CTRL-G38 | remote默认disabled/loopback，bind/port/profile变更有显式policy和lifecycle receipt |
| CTRL-G39 | principal/surface/command具rate、concurrency、payload/output/watch和operation quota |
| CTRL-G40 | request replay/dedup/order与disconnect cancel/watch cleanup在fault/soak中成立 |
| CTRL-G41 | Editor interactive view支持input/focus/IME/submit/cancel/completion并与log mode清晰分离 |
| CTRL-G42 | App `--exec`/script/CVar override阶段顺序固定，失败影响exit code且不越过项目/session admission |
| CTRL-G43 | headless/server与Editor/CLI共享catalog/policy/output，不出现私有后门 |
| CTRL-G44 | diagnostics可解释catalog generation、effective/source/pending、session/quota和transaction receipt |
| CTRL-G45 | plugin贡献与remote wire通过schema/ABI compatibility、skew、unknown-field和old-client测试 |
| CTRL-G46 | duplicate、panic、timeout、reload、disconnect、queue overflow和shutdown fault/soak无泄漏或半状态 |
| CTRL-G47 | 10k catalog/高并发读写/1k watch下startup、lookup、frame、allocation、latency优于既定Unreal基线 |
| CTRL-G48 | `git diff --check`、frontmatter path/link、0/72/16 finding计数、48 gates和五份账本一致 |

## 11. 状态与产出记录

| 项目 | 状态 | 日期 | 证据 |
|---|---|---|---|
| Zircon 523文件纵向冻结 | review_complete | 2026-08-20 | 45,106行、1,581,380 bytes；SHA-256 `e31d719d45fea5e679dd9bd99a7581f0f2787a0f314884fa2aa07ee57914ea34` |
| 五参考32文件语义对照 | review_complete | 2026-08-20 | 26,526行、930,695 bytes；SHA-256 `04dbdfa0efc60afefdd0f1f5d15636ac79bdbdb93950ac2c5f52438a55a0d91c` |
| Severity与owner路由 | review_complete | 2026-08-20 | 0 P0 / 72 P1 / 16 P2；48 gates；共享父owner不重复计数 |
| Production、tests与Cargo变更 | pending | - | 本篇只review；MVP gate下未运行Cargo或产品验证 |
