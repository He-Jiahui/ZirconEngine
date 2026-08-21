---
related_code:
  - zircon_runtime/src/plugin/capability_status.rs
  - zircon_runtime/src/plugin/plugin_maturity.rs
  - zircon_runtime/src/plugin/runtime_plugin/capability_view.rs
  - zircon_runtime/src/plugin/runtime_profile/availability_projection.rs
  - zircon_runtime/src/core/runtime/lifecycle.rs
  - zircon_runtime/src/core/runtime/descriptors/module_descriptor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_editor/src/ui/host/editor_capabilities.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/enablement/capabilities.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/status/builtin.rs
  - zircon_editor/src/core/play/controller.rs
  - zircon_editor/src/core/play/backend/noop.rs
  - zircon_editor/src/core/play/plugin_activation/noop.rs
  - zircon_editor/src/ui/host/editor_host_startup.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/with_viewport/runtime_backend.rs
  - zircon_plugins/rendering/plugin.toml
  - zircon_plugins/rendering/features/baked_lighting/runtime/src/lib.rs
  - zircon_plugins/rendering/features/decals/runtime/src/lib.rs
  - zircon_plugins/rendering/features/post_process/runtime/src/lib.rs
  - zircon_plugins/rendering/features/shader_graph/runtime/src/lib.rs
  - zircon_plugins/rendering/features/vfx_graph/runtime/src/lib.rs
  - zircon_plugins/particles/runtime/src/interop/physics.rs
  - zircon_hub/src/state/task_status.rs
tests:
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests/registry_contracts.rs
  - zircon_plugins/particles/runtime/src/tests/optional_features.rs
  - zircon_editor/src/core/play/tests.rs
  - zircon_hub/tests/project_workflow_contract.rs
plan_sources:
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/01-core-runtime-lifecycle-registry-review.md
  - docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md
  - docs/plans/optimize/zircon_editor/07-play-session-process-pie-game-view-live-edit-recovery-review.md
  - docs/plans/optimize/zircon_plugins/04-rendering-umbrella-feature-bundles-solari-native-provider-product-integration-review.md
  - docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md
  - docs/plans/optimize/zircon_tooling/10-test-architecture-partition-selection-isolation-fixture-flake-results-review.md
  - docs/plans/optimize/zircon_tooling/15-mvp-build-staging-product-process-acceptance-evidence-resource-baseline-control-plane-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Public/PluginDescriptor.h
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/PluginDescriptor.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Modules/ModuleManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Features/IModularFeatures.h
  - dev/UnrealEngine/Engine/Source/Runtime/RHI/Public/RHIFeatureLevel.h
  - dev/UnrealEngine/Engine/Source/Runtime/RHI/Public/RHIShaderPlatform.h
  - dev/bevy/crates/bevy_app/src/plugin.rs
  - dev/bevy/crates/bevy_app/src/app.rs
  - dev/bevy/crates/bevy_render/src/settings.rs
  - dev/bevy/crates/bevy_render/src/lib.rs
  - dev/godot/servers/rendering/rendering_server.h
  - dev/godot/servers/rendering/rendering_device.h
  - dev/godot/servers/rendering/rendering_device_driver.h
  - dev/Fyrox/fyrox-impl/src/plugin/mod.rs
  - dev/Fyrox/fyrox-impl/src/engine/executor.rs
  - dev/Fyrox/editor/src/plugin.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraph.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/IRenderGraphBuilder.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Utilities/HDROutputUtils.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Tooling 16：能力真相、Placeholder、No-op、Fallback、Degraded 与资格控制面审查

## 1. 结论

本轮直接审查Runtime plugin/profile、module lifecycle、render executor、Editor capability/Play、首方rendering/particles plugin、Hub task status及其测试与产品接线，并对生产源码做了`todo!/unimplemented!`、TODO/FIXME/HACK、placeholder/stub、noop、fallback/degraded、unsupported、panic与状态词横向扫描。结论不是“仓库充满未写完的`todo!`”：生产owner没有可执行TODO，三个`unimplemented!`都位于Sound Editor live-output test double；真正危险的是已经能够编译、返回成功并被投影成`Complete`、`Available`、`Running`或`Success`的临时语义。

当前至少存在三套彼此不等价的真相。`CapabilityStatus`和`PluginMaturity`是package作者自声明；runtime profile把target、maturity、linked/native/provider membership通过后投影为`Available`，但不要求provider实例已经加载、初始化、健康或执行；Editor又把加载得到的capability字符串与启用配置合并成可用快照。与此同时，`ModuleLifecycle`默认`build/finish/cleanup = Ok(())`、`ready = true`，`ModuleDescriptor::new`默认安装`NoopModuleLifecycle`；render registry允许为若干pass注册no-op executor；`PlaySessionController::default/new`安装两个成功型no-op backend。单个机制可以有合理用途，问题在于仓库没有一条统一规则阻止它们成为“功能完成”的证明。

这不是要求删除所有空hook。Bevy和Fyrox同样允许plugin lifecycle默认不做工作，但Bevy的`PluginsState`只表达Adding/Ready/Finished/Cleaned，不把它命名为产品功能资格；Unreal把plugin descriptor、module load状态、modular feature registration和RHI feature level分开；Godot从当前RenderingDevice/driver查询feature、format support和limit；Unity Graphics在RenderGraph中显式绑定执行函数，并在fallback选择时检查资源有效性、格式与MSAA。本轮采用的共同约束是：声明、链接、加载、初始化、运行、执行观察和产品资格必须分层，不把“没有报错”外推成“产生了预期效果”。

仓库也有应保留的基础。runtime availability已经区分BlockedByTarget、BlockedByMaturity、Linked、NativeDynamic、ExternalizedMissing与Available；particles提供typed optional-feature status；Editor产品startup确实替换为`ProcessPlayBackend`，retained host也安装`NativePluginBridgeActivation`；render registry中的mesh no-op会被真实executor覆盖；不少fallback明确记录原因。这些机制说明无需推倒重来，需要补的是统一identity、state transition、observation、fallback policy和qualification receipt。

本篇登记 **3项P0、52项P1、12项P2**。它只拥有跨域状态语义、no-op/fallback政策和资格证据协议；render feature实际实现仍由Plugins04及Graphics报告拥有，Play产品闭环由Editor07拥有，module并发生命周期由Runtime01拥有，动画panic由Runtime08C拥有，Test/ValidationSet由Tooling10/15拥有。本轮没有修改production、test、manifest或CI，也没有把静态扫描候选批量定性为缺陷。

## 2. 扫描证据与边界

### 2.1 横向扫描结果

| 扫描面 | 原始命中 | 去除test/fixture/vendor/generated/docs后的候选 | 解释 |
|---|---:|---:|---|
| `todo!` / `unimplemented!` | 3 | 0 production | 三处均为Sound Editor测试替身，不能据此声称产品完成或未完成 |
| TODO/FIXME/XXX/HACK | 30 | 0 actionable production | 28处在vendored Recast，一处WOC预期字符串，一处版本错误消息 |
| placeholder/stub | 4,337行 / 831文件 | 924行 / 252文件 | 包含类型、诊断和显式状态，必须按consumer语义复核 |
| noop | 360行 / 125文件 | 108行 / 51文件 | 同时包含合理Null Object、test double和产品协议风险 |
| fallback/degraded | 7,182行 / 1,327文件 | 4,046行 / 761文件 | 数量大但高度异质，不能按关键词直接判错 |
| unsupported | 1,918行 / 620文件 | 1,231行 / 395文件 | 需要区分硬件事实、目标限制、临时缺失与错误恢复 |
| capability/status类声明 | 256声明 / 162文件 | 未发现统一qualification类型 | 多个局部状态机存在，但无跨域可组合证据链 |

关键词统计只用于定位，不作为finding数量。尤其`fallback`包含字体、资源、图形格式、平台、localization、网络、Editor预览等完全不同语义；任何实施批次都必须回到具体caller、输入域、last-good、用户可见性和失败政策。

### 2.2 当前状态投影链

| 层 | 当前输入 | 当前输出 | 尚未证明 |
|---|---|---|---|
| package declaration | manifest capability/status/maturity/targets | `Complete`、`Stable`等自述 | source owner、实现、测试、当前build |
| runtime catalog/profile | catalog membership、target、minimum maturity、provider ID | `Linked`、`NativeDynamic`、`Available` | instance load、init、health、执行效果 |
| runtime capability view | registration report + package/feature manifest | provided capability字符串集合 | 同generation、实际consumer、semantic effect |
| Editor enablement | loaded strings + enabled subsystem配置 | enabled capability snapshot/UI status | product backend、operation成功、reopen结果 |
| module lifecycle | 默认成功hook与`ready=true` | module可进入Running | service是否注册、线程/GPU/IO是否工作 |
| render registry | executor ID存在 | graph validation可继续 | executor是否为no-op、资源写入、GPU结果 |
| Play default | no-op start/stop成功，poll Running | controller可表现为running | 进程、world、window、bridge、退出码 |
| Hub action | caller选择`TaskStatus::success` | severity Success、progress 100 | receipt、BuildSet、qualification/currentness |

### 2.3 不是缺陷的空实现与必须禁止的假完成

允许的空实现必须同时满足：语义明确为optional hook；调用者不把返回成功当成功能完成；状态中可见`effect = none`；required profile不会依赖它；测试证明它不改变状态且不会吞掉必需工作。典型例子是一个没有cleanup资源的纯声明模块，或未启用可选物理集成时的particles降级。

必须禁止的模式包括：required capability由空executor满足；`Complete`只来自manifest；产品controller默认返回Running但不启动产品；fallback资源被使用却仍报告full-quality；捕获到错误后继续发布同一generation的success；Hub把action completion等同于artifact/product qualification。判断标准是协议效果，不是函数体行数。

## 3. P0：先切断“自声明即完成”的发布路径

### TOOL-CAPTRUTH-P0-001 · `Complete/Available`可以由声明与membership产生，没有实例、执行或资格证明

`CapabilityStatusManifest`允许package直接写`Complete`，`PluginMaturity`允许写`Stable`；availability projection在catalog、target、maturity和provider membership通过后生成`Available`。`capability_view`又可从字符串列表构造provided集合，Editor继续将其并入enabled snapshot。整条链没有强制携带ProviderInstanceId、load generation、health observation、execution receipt、ValidationSet或current BuildSet，因此“metadata存在”可以越级成为产品可用信号。

硬切要求：`DeclaredStatus`不得实现或隐式转换为`OperationalState`；required capability只有绑定当前产品实例的`QualificationReceipt`才可进入qualified profile。迁移前所有仅由manifest生成的`Complete/Available`降级为`Declared`或`Discoverable`，UI必须显示未验证而不是可用。

### TOOL-CAPTRUTH-P0-002 · 成功型no-op可满足生命周期、渲染与Play协议，缺少semantic-effect约束

核心公共API同时提供默认成功的`NoopModuleLifecycle`、builtin no-op render executors、`NoopPlayBackend`与`NoopPluginBridgeActivation`。产品Editor当前确实替换了Play backend，这一点应保留；但公共默认构造仍允许其他host、测试或后续接线在没有显式选择test/degraded policy时获得`Ok/Running`。render graph的验证主要检查executor ID存在，不能区分真实GPU工作与no-op；Plugins04已证明若干stable/default render feature仍注册空executor。

硬切要求：每个operation/provider声明`SemanticEffect`与required observations；no-op构造移入test-support或必须显式传入`IntentionalNoopPolicy`；no-op不能满足effectful required capability。任何保留的空hook都只表示“该hook无需动作”，不能证明所属feature qualified。

### TOOL-CAPTRUTH-P0-003 · fallback/degraded没有统一严重度、预算和发布门，current qualification可忽略降级事实

仓库已有大量局部fallback与少量typed degraded状态，却没有统一FallbackEvent identity、cause taxonomy、quality impact、lifetime、budget、last-good binding和promotion policy。局部系统可以正确选择fallback，外层仍只看到`Available/Running/Success`；Tooling15又证明当前产品资格本身未绑定可信BuildSet/Observation。结果是关键shader/texture/font/device/provider退化、空executor或optional module缺失都无法在同一产品receipt上聚合，发布门也没有“critical fallback为零、允许项不超预算”的可计算规则。

硬切要求：所有影响产品语义或质量的fallback发出typed event并绑定ProductInstance/Frame或Operation；profile定义critical/allowed/budgeted policy；Qualification Service聚合事件、观察与waiver。未知fallback、过期waiver和关键能力降级一律阻断promotion，不能仅写日志后继续标绿。

## 4. P1：状态、失败与证据控制面重构

### 4.1 统一身份、声明与状态机

1. **TOOL-CAPTRUTH-P1-001**：定义稳定`CapabilityId`，由namespace、owner、semantic version组成；禁止用任意展示字符串同时承担identity、selection和UI label。
2. **TOOL-CAPTRUTH-P1-002**：拆分`CapabilityDeclaration`、`CapabilityRequirement`、`CapabilityProviderDescriptor`和`CapabilityProviderInstance`；package metadata只拥有前两者。
3. **TOOL-CAPTRUTH-P1-003**：建立`Declared -> Discoverable -> Linked -> Loaded -> Initialized -> Ready -> Active -> Executed -> Qualified`正向状态，transition必须带generation和receipt。
4. **TOOL-CAPTRUTH-P1-004**：建立`Blocked/Unavailable/Degraded/Faulted/Retired`负向状态；不得用空数组、false或缺字符串表达不同原因。
5. **TOOL-CAPTRUTH-P1-005**：`Complete/Partial/Stub`改名或限定为implementation declaration，禁止与runtime operational state共用同一status字段。
6. **TOOL-CAPTRUTH-P1-006**：`PluginMaturity`只表达support policy；不能参与推导“实现完整”或“本机可运行”。
7. **TOOL-CAPTRUTH-P1-007**：所有状态携带`observed_at`、source generation、provider instance和currentness deadline，避免旧状态跨reload/install复用。
8. **TOOL-CAPTRUTH-P1-008**：transition使用compare-and-publish generation；旧provider、旧device或旧world的late completion必须被拒绝。
9. **TOOL-CAPTRUTH-P1-009**：required/optional/recommended从profile policy显式建模，禁止同一裸capability集合在不同产品中隐式改变严重度。
10. **TOOL-CAPTRUTH-P1-010**：建立dependency explanation graph，输出哪个requirement被哪个provider、哪个receipt满足，以及阻断链首因。
11. **TOOL-CAPTRUTH-P1-011**：capability alias、rename和version negotiation必须有迁移表；字符串同名不等于schema/ABI兼容。
12. **TOOL-CAPTRUTH-P1-012**：定义状态序列化schema和forward-compatible unknown handling；未知状态不得默认Available或Success。

### 4.2 Runtime lifecycle 与公共API

13. **TOOL-CAPTRUTH-P1-013**：`ModuleLifecycle::ready`默认值改为显式policy；descriptor必须选择declarative-only或提供ready observation。
14. **TOOL-CAPTRUTH-P1-014**：`ModuleDescriptor::new`不得静默安装成功型生命周期；提供`declarative_module`和`managed_module`两个语义清楚的构造入口。
15. **TOOL-CAPTRUTH-P1-015**：module build/finish/cleanup receipt记录注册的service、task、thread、resource和rollback owner，而非只有`CoreResult<()>`。
16. **TOOL-CAPTRUTH-P1-016**：ready必须验证声明的required postcondition；没有postcondition的module只能成为Loaded，不能自动Running。
17. **TOOL-CAPTRUTH-P1-017**：cleanup失败、超时或资源残留进入Faulted/Leaked，不能继续发布Unloaded并允许DLL卸载。
18. **TOOL-CAPTRUTH-P1-018**：capability view只消费同一registry generation的concrete registration report，package自述作为注释保留而非provided事实。
19. **TOOL-CAPTRUTH-P1-019**：provider membership与provider health分开缓存；catalog ID存在不能替代instance heartbeat或functional probe。
20. **TOOL-CAPTRUTH-P1-020**：public wait/resource/world API的panic policy分类为programmer invariant、recoverable product fault或process-fatal；跨plugin/foreign input边界禁止不受控panic。
21. **TOOL-CAPTRUTH-P1-021**：runtime asset root、device feature和format支持错误返回typed unavailable/fault并附输入来源，不用全局panic或隐式fallback抹平配置问题。
22. **TOOL-CAPTRUTH-P1-022**：operation cancellation、shutdown和reload必须撤销或retire相应capability receipt，避免ghost availability。

### 4.3 Graphics、asset 与fallback政策

23. **TOOL-CAPTRUTH-P1-023**：render executor registration增加`ExecutorKind::{Gpu, Cpu, Composite, IntentionalNoop}`和声明的read/write/effect集合。
24. **TOOL-CAPTRUTH-P1-024**：RenderGraph validation除ID存在外，验证required pass不能绑定IntentionalNoop，资源写入与后继read闭合。
25. **TOOL-CAPTRUTH-P1-025**：builtin no-op allowlist必须按profile、feature和测试用途生成；公开“一次装入全部no-op”构造不得用于产品renderer。
26. **TOOL-CAPTRUTH-P1-026**：plugin executor registration输出provider/version/generation，覆盖builtin placeholder时记录replacement receipt并禁止顺序依赖。
27. **TOOL-CAPTRUTH-P1-027**：frame observation记录required pass实际编码、提交和完成，不能以graph compile/validate成功代替执行。
28. **TOOL-CAPTRUTH-P1-028**：GPU device feature/limit/format来自活动adapter/device snapshot；manifest target mode只用于预筛选。
29. **TOOL-CAPTRUTH-P1-029**：fallback texture/shader/pipeline/font/material必须携带原请求identity、选择原因、质量等级与cache key，避免污染正常artifact cache。
30. **TOOL-CAPTRUTH-P1-030**：unsupported format、missing binding和output-transfer-only等调用错误在graph admission阶段返回typed diagnostic，避免执行期panic。
31. **TOOL-CAPTRUTH-P1-031**：每个quality profile定义critical render feature和允许fallback预算；stable/default标签不得绕过像素与GPU observation。
32. **TOOL-CAPTRUTH-P1-032**：device lost、shader compile失败和pipeline unavailable采用last-good generation或显式停帧政策，不能混合新旧资源后仍报告qualified。

### 4.4 Editor、App 与Hub消费面

33. **TOOL-CAPTRUTH-P1-033**：Editor capability snapshot保存状态对象与receipt reference，不再把loaded string和enabled config合并成同一种“已启用”。
34. **TOOL-CAPTRUTH-P1-034**：Plugin Manager分别显示Declared、Installed、Loaded、Ready、Degraded、Qualified和Blocked reason，禁止单一available徽标。
35. **TOOL-CAPTRUTH-P1-035**：Editor command/menu enablement依赖operation-level requirement；capability未知或过期时禁用并提供首因，不乐观开放。
36. **TOOL-CAPTRUTH-P1-036**：`PlaySessionController`公共构造要求显式backend；no-op只在test fixture或明确headless simulation policy中可用。
37. **TOOL-CAPTRUTH-P1-037**：Play进入Running必须绑定child process/world generation、bridge activation、first-frame或headless tick observation。
38. **TOOL-CAPTRUTH-P1-038**：App host startup生成ProductInstanceId并冻结所消费的capability snapshot，运行中变更通过新generation发布。
39. **TOOL-CAPTRUTH-P1-039**：Hub `TaskStatus::success`只表达UI action terminal；build/install/launch另携ArtifactReceipt、ProductReceipt或QualificationReceipt。
40. **TOOL-CAPTRUTH-P1-040**：Hub进度100与severity Success不得自动解锁Launch/Publish；下一操作按receipt policy admission。

### 4.5 Plugin SDK 与distribution

41. **TOOL-CAPTRUTH-P1-041**：plugin manifest的capability status必须由source registration、artifact export和runtime probe三方对账，漂移时拒绝package。
42. **TOOL-CAPTRUTH-P1-042**：stable/core package要求owner、support window、platform matrix、ABI range和qualification suite，不再只是rank较高的enum。
43. **TOOL-CAPTRUTH-P1-043**：native dynamic provider的load receipt绑定binary digest、signature、ABI negotiation、entry generation与unload policy。
44. **TOOL-CAPTRUTH-P1-044**：feature bundle若只注册空executor或静态descriptor，必须声明Stub/Externalized或从default profile移除。
45. **TOOL-CAPTRUTH-P1-045**：optional integration使用typed reason和capability requirement；日志中的“no-op because missing capability”同时发出可聚合degraded event。
46. **TOOL-CAPTRUTH-P1-046**：dist catalog不得重放部分registration后沿用source capability状态；导出过程生成closure diff并阻断丢失executor/service/provider。

### 4.6 Tooling、测试与资格证据

47. **TOOL-CAPTRUTH-P1-047**：新增Capability Truth validator，对manifest、source registration、profile和dist projection做同BuildSet闭包检查。
48. **TOOL-CAPTRUTH-P1-048**：validator禁止required+Complete/Stable capability仅由no-op effect满足，并输出最短provider/consumer路径。
49. **TOOL-CAPTRUTH-P1-049**：Test Service增加transition、fault injection、reload、device-loss、fallback-budget和stale-receipt conformance suite。
50. **TOOL-CAPTRUTH-P1-050**：每个qualified capability登记至少一个独立observation producer和oracle；producer自身状态不能作为oracle。
51. **TOOL-CAPTRUTH-P1-051**：QualificationReceipt绑定BuildSet、ProductReceipt、Profile、Platform、Device、Scenario、ObservationSet和waiver digest。
52. **TOOL-CAPTRUTH-P1-052**：required CI先阻止新增越级状态转换和成功型产品no-op，再按owner逐域收紧既有豁免；豁免必须有期限和owner。

## 5. P2：可观测性、治理与开发体验

1. **TOOL-CAPTRUTH-P2-001**：提供capability graph查询CLI，按产品输出provider、状态、receipt、首因和依赖路径。
2. **TOOL-CAPTRUTH-P2-002**：Editor/Hub共享状态词典、颜色与可访问文本，但不共享业务状态owner。
3. **TOOL-CAPTRUTH-P2-003**：为每次状态transition发结构化trace，支持generation、latency和failure-reason聚合。
4. **TOOL-CAPTRUTH-P2-004**：建立fallback dashboard，按产品版本、平台、设备和场景查看频率、持续时间与预算消耗。
5. **TOOL-CAPTRUTH-P2-005**：将no-op/fallback inventory生成机器可读基线，diff只提示新增/变更，不把关键词命中自动升级为缺陷。
6. **TOOL-CAPTRUTH-P2-006**：API文档自动展示构造器默认backend、semantic effect与产品可用限制。
7. **TOOL-CAPTRUTH-P2-007**：状态schema生成Rust/TypeScript/FFI投影，避免Hub、Editor、runtime各自手写近义enum。
8. **TOOL-CAPTRUTH-P2-008**：错误消息携带capability/provider/operation identity和修复入口，不只输出“unsupported”或“unavailable”。
9. **TOOL-CAPTRUTH-P2-009**：对频繁fallback建立采样与去重，保留首个完整诊断及计数，避免每帧日志风暴。
10. **TOOL-CAPTRUTH-P2-010**：qualification历史支持source drift/currentness查询，但历史通过不得自动成为当前通过。
11. **TOOL-CAPTRUTH-P2-011**：提供SDK conformance示例，分别演示真实provider、intentional no-op optional hook、degraded fallback和fault recovery。
12. **TOOL-CAPTRUTH-P2-012**：文档生成器把每个capability链接到canonical owner报告、实现owner和required gate，防止第二authority。

## 6. 目标架构

### 6.1 核心对象

| 对象 | 唯一owner | 必要绑定 | 不得替代 |
|---|---|---|---|
| `CapabilityDeclaration` | package/source owner | CapabilityId、version、targets、requirements | runtime availability |
| `ProviderInstance` | runtime host | instance、binary/build digest、generation、lifecycle | package maturity |
| `CapabilityLoadReceipt` | lifecycle manager | provider、dependency closure、registered effects、rollback | execution evidence |
| `ExecutionObservation` | domain observer | operation/frame/world、inputs、effects、result | self-reported success字符串 |
| `FallbackEvent` | actual selector | request、chosen fallback、cause、impact、lifetime | debug log |
| `QualificationReceipt` | qualification service | BuildSet、Product、Profile、Device、Scenario、observations、waivers | manifest `Complete` |

推荐状态序列为：

```text
Declared -> Discoverable -> Linked -> Loaded -> Initialized -> Ready
                                                      |
                                                      v
                                         Active -> Executed -> Qualified

任意阶段 -> Blocked | Unavailable | Degraded | Faulted | Retired
```

`Ready`只证明provider postcondition；`Executed`证明某次effectful operation；`Qualified`证明当前BuildSet和产品矩阵达到政策。它们可以在不同时间同时存在，例如provider Ready但某设备Degraded、某场景尚未Qualified。

### 6.2 Semantic effect 与no-op规则

operation至少声明`None`、`StateMutation`、`ArtifactPublication`、`ProcessLaunch`、`CpuWork`、`GpuWork`、`Persistence`或domain-specific effect。Intentional no-op只能满足`None`；一个hook可合法为空，不代表拥有该hook的feature没有required effect。测试替身通过单独feature/module和显式fixture注入，不让默认产品构造静默落入no-op。

### 6.3 Fallback政策

fallback按`Transparent`、`QualityReduced`、`FunctionReduced`、`SafetyCritical`分类，并记录temporary/permanent、retry/last-good策略和预算。Transparent仍要可观测；QualityReduced影响画质资格；FunctionReduced影响required capability；SafetyCritical立即阻断。waiver必须绑定精确BuildSet、platform/scenario、owner、理由和期限。

## 7. 与既有专项报告的非重复边界

| 事实 | Canonical实现owner | 本篇只拥有 |
|---|---|---|
| rendering stable/default feature注册空executor | Plugins04、Runtime Graphics系列 | no-op不能满足required/qualified的全局规则 |
| first-party catalog/profile/provider缺口 | Plugins06 | declaration、provider instance与qualification的状态schema |
| Play产品backend、PIE、process/recovery | Editor07 | 公共no-op构造与Running证据规则 |
| module lifecycle并发、cleanup、reload | Runtime01 | lifecycle状态如何进入capability truth |
| animation worker错误后channel panic | Runtime08C P1-10 | panic/fault分类政策，不重复该finding |
| TestPlan、ValidationSet、product evidence | Tooling10、Tooling15 | capability observation/receipt如何接入这些服务 |
| Hub build/install/launch流程 | Hub01、Tooling09 | UI success与artifact/product qualification分离 |

## 8. 分层实施路线

### M0 · 冻结越级声明

- 输出全部CapabilityId、声明状态、profile consumer、provider registration、no-op和fallback清单。
- required gate禁止新增“manifest Complete -> UI Available”直通路径；既有路径以有期限waiver登记。
- 将stable/default且已知空执行面的feature标为unqualified，不改变其canonical实现owner。

### M1 · Schema 与identity

- 引入Declaration/Requirement/ProviderInstance/OperationalState schema与稳定ID。
- 实现generation、currentness和unknown-state fail-closed。
- Rust、FFI、Editor、Hub投影由同一schema生成或验证。

### M2 · Lifecycle 与effect receipts

- 重构module/provider load、ready、cleanup receipt。
- 为render、Play和典型plugin operation接入SemanticEffect与ExecutionObservation。
- 移动test-only no-op构造，保留显式intentional no-op policy。

### M3 · Fallback/degraded聚合

- 先接图形、asset、font、device和plugin optional integration高风险路径。
- 建立FallbackEvent、last-good、budget和waiver。
- Editor/Hub呈现首因与qualification impact。

### M4 · Qualification Service

- 绑定Tooling10 Test Service和Tooling15 BuildSet/Product/Observation链。
- 按product/profile/platform/device/scenario生成不可变QualificationReceipt。
- promotion、Launch和Editor command admission消费receipt而非布尔位。

### M5 · Domain迁移

- Plugins04/06、Runtime Graphics、Editor07、Hub01先迁移已知高风险路径。
- Runtime/Editor其他报告随实施补operation effect、fallback policy和observation。
- 删除旧capability字符串集合与双写状态，禁止兼容层永久存活。

### M6 · 竞争性资格

- 同场景、同画质、同平台、同硬件对Unreal等参考建立correctness/performance evidence。
- fallback budget、fault、soak、device loss、artifact和currentness全部闭合后才允许比较结论。
- 性能更高但使用降质fallback、空pass或缺功能的结果不得进入对比。

## 9. 验收门

1. 所有required capability都能从Requirement追溯到ProviderInstance、LoadReceipt、ExecutionObservation和QualificationReceipt。
2. manifest `Complete/Stable`不能单独生成Operational Available或UI可用状态。
3. 产品profile中没有required effect由IntentionalNoop满足；test-only no-op不会链接进release产品。
4. render graph验证会拒绝required effect的空executor，并能证明实际encode/submit/complete。
5. Play Running绑定真实process/world/bridge及首帧或tick observation；headless policy显式区分。
6. 所有质量或功能相关fallback进入聚合账本，critical为零，budgeted项不超profile预算。
7. stale、旧generation、旧device和旧BuildSet receipt不能通过current qualification。
8. Hub Success/100%不会自动等同build/install/launch qualified，后续操作按typed receipt admission。
9. provider reload/unload/fault能撤销availability并拒绝late completion，且保留last-good/rollback证据。
10. required CI运行capability closure、no-op effect、fallback budget、fault/reload和qualification conformance suite。
11. 每项waiver都有owner、精确scope、期限和删除门；过期或未知状态fail closed。
12. 竞争性报告同时公开功能矩阵、画质/语义设置、fallback事件、硬件/driver和统计方法。

## 10. 本轮限制与下一步

本轮是静态控制面审查，没有重复运行当前已知被Editor 239个compile errors、Hub `persist_unchecked`签名漂移和WOC 6个compile errors阻断的动态lane。也没有运行会产生大量staging/capture artifact的MVP验收。既有动态失败由各canonical报告继续拥有。

下一步应从M0机器清单和P0-001的状态硬切开始，但在实际修改代码前，需按涉及的Runtime01、Plugins04/06、Editor07、Hub01与Tooling10/15重新读取当前source generation。任何局部系统实现真实功能后，也只能把状态推进到其有证据的阶段，不能直接写`Qualified`。
