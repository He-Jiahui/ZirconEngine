---
title: Runtime Script 与 Plugin Runtime 当前源码复核
category: zircon_runtime
report_id: Runtime164
review_date: 2026-08-30
baseline_head: 189f72219eaf16a6d0db880b53f3f68b4f5ee15a
verification_head: 189f72219eaf16a6d0db880b53f3f68b4f5ee15a
canonical_owner: Runtime07
refreshes:
  - docs/plans/optimize/zircon_runtime/07-script-plugin-runtime-review.md
related_reports:
  - docs/plans/optimize/zircon_runtime/07/2026-08-26-vm-slot-invocation-lease-architecture-and-profile-plan.md
  - docs/plans/optimize/zircon_runtime/07/2026-08-28-typed-catalog-generation-identity.md
  - docs/plans/optimize/zircon_runtime/46-engine-module-service-contract-context-factory-descriptor-snapshot-composition-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/99zj-runtime-engine-module-service-contract-context-factory-descriptor-snapshot-composition-lifecycle-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99p-runtime-plugin-interface-bridge-slot-generation-strong-weak-native-vm-lifecycle-diagnostics-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_plugins/16-first-party-zr-vm-language-source-runtime-dist-catalog-reflection-callsite-host-interface-gc-hot-reload-product-integration-review.md
  - docs/plans/optimize/zircon_plugins/21-plugin-artifact-marketplace-third-party-package-install-update-trust-non-cargo-product-integration-review.md
related_code:
  - zircon_runtime/src/plugin/runtime_plugin
  - zircon_runtime/src/plugin/native
  - zircon_runtime/src/plugin/bridge
  - zircon_runtime/src/script
  - zircon_runtime/src/dynamic_api/session
  - zircon_plugins/plugin_sdk/src/native.rs
  - zircon_plugins/zr_vm_language/runtime
  - zircon_plugins/first_party_runtime_catalog
  - zircon_app/src/entry
  - examples/vampire
tests:
  - zircon_runtime/src/script
  - zircon_runtime/src/plugin
  - zircon_runtime/src/dynamic_api/session/tests
  - zircon_plugins/zr_vm_language/runtime/src/tests
  - zircon_plugins/plugin_sdk/src/native/tests.rs
reference_engines:
  - dev/bevy/crates/bevy_app/src/plugin.rs
  - dev/bevy/crates/bevy_app/src/plugin_group.rs
  - dev/Fyrox/fyrox-impl/src/plugin/mod.rs
  - dev/godot/core/object/script_language.h
  - dev/godot/core/object/script_language.cpp
  - dev/godot/core/object/script_language_extension.h
  - dev/godot/core/extension/gdextension_manager.h
  - dev/godot/core/extension/gdextension_manager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Modules/ModuleManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Modules/ModuleManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Public/Interfaces/IPluginManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/PluginManager.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/package.json
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Unity.RenderPipelines.Core.Runtime.asmdef
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Unity.RenderPipelines.Core.Editor.asmdef
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Tests/Editor/Unity.RenderPipelines.Core.Editor.Tests.asmdef
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: partial_foundations_product_atomicity_isolation_execution_and_tooling_closure_incomplete
source_recheck_required: true
working_tree_drift_observed_after_snapshot: true
---

# Runtime164 · Script 与 Plugin Runtime

## 1. 结论

当前工作树已经具备一批应保留的工程化基础。Runtime plugin catalog 不再只是可复制整数版本：`PluginCatalogGeneration`、以精确 base snapshot 为根的 candidate、prepared generation 和 `ArcSwap` compare-and-swap publication 已形成不可变目录发布骨架；`CompiledProjectPluginPlan` 固定 catalog generation、fingerprint、target、完成后的 manifest/feature/extension projection，App composition 与 dynamic session 也开始 pin 同一 plan identity。Native live host 已有按 generation 固定动态库寿命的 callback lease、关闭 admission 的 transition bit、锁外回调、host-owned bounded command output 和 per-plugin hot-reload rollback。VM hot reload 已有 state save/restore、schema migration、reflection staging/commit/rollback、panic-safe instance take/restore、cooperative GC 与 callback handle generation。Scene script 已实际接入 `onStart/onUpdate/onFixedUpdate`，借用式 host value、typed host descriptor 和反射表也比全 JSON/string dispatch 更接近正确方向。

这些局部实现仍没有汇聚为一个可发布、可隔离、可预算、可调试的产品系统。catalog、compiled plan、native live host、VM slot、bridge/extension、App bootstrap 与 dynamic session 仍各自维护 generation 或生命周期事实；native batch activation 和跨 catalog/world publication 仍可逐插件、逐 owner 可见，不能保证消费者只看到完整旧代或完整新代。依赖描述没有形成 version/source/digest/signer/trust/target 的确定性求解；原生 DLL 在校验来源和信任之前直接进入主进程，`catch_unwind` 与 capability table 不能隔离 access violation、abort、hang、内存破坏或插件直接访问 OS。ABI 虽在向 V3 hard cut 收敛，但仍缺单源生成的 C/Rust schema、跨编译器布局语料、allocator provenance 和可执行兼容矩阵。

脚本执行面仍有两个结构性瓶颈。真实 ZrVM backend 用进程级 `OnceLock<Mutex<()>>` 串行 compile/load/call/GC/drop，并以该锁作为 raw-pointer owner `unsafe Send/Sync` 的成立条件；Runtime slot 又用全局 lifecycle mutex和从 map 暂时取走 `Option<Instance>` 来拒绝碰撞，没有 per-slot admission、drain、deadline、reentrancy policy 或 retiring generation lease。`VmPluginMemoryPolicy` 仍只是可解析声明，普通 export 没有 fuel/instruction、wall/CPU deadline、allocation、host-call、nested-call 或 cancel budget。Scene lifecycle、stable component identity、failure isolation、typed aggregate marshalling、debugger/source map/profiler 和可重现已签名脚本 artifact 均未闭合。

产品链也尚不能作为验收证据。App feature graph能选择 first-party ZrVM plugin，但插件 crate 默认 feature 为空，真实 backend 依赖 workspace 外的 `zr_vm` path 和环境动态库；dynamic session 的启动脚本接线是真实的，但本轮产品选择集仍有 10 个 real-ZrVM 上层测试被 ignore，plugin owner 的 feature-gated fixture不能替代 Editor -> Play、App -> Dynamic Session、standalone/export/cook、并发 world、活跃调用 reload、crash/hang/OOM 和长时稳定性矩阵。

本文刷新 Runtime07 的 16 项 canonical finding，**不新增唯一 finding**。14 项 P1 当前重判为 **7 Open、7 Partial、0 Closed**；2 项 P2 为 **1 Open、1 Partial、0 Closed**；16 项资格门为 **10 Fail、6 Partial、0 Pass**。Plugins01/16/21、Runtime99p/99zj 已拥有的 P0 或共享边界继续由原 owner 关闭，本文只登记 current-source blocker candidate，不重复累计。现有静态证据不能证明脚本/插件功能完整，更不能证明同硬件性能、崩溃隔离或创作体验优于 Unreal。

## 2. 审查边界与冻结证据

### 2.1 统计口径

统计口径为当前工作树 UTF-8 physical lines、non-empty lines、bytes、精确 `#[test]` / `#[ignore]`；fingerprint 为每个选择文件的 lowercase `relative-path<TAB>SHA-256` 按路径排序，以 LF 连接且末尾不加 LF，再做外层 SHA-256。各分组有意重叠，不能相加。`selected production dedup` 是本轮选定 production/path 集的去重并集，不代表整个 `zircon_plugins` 功能库已逐算法完成审查。

| 选择集 | files | lines | nonempty | bytes | tests | ignored | fingerprint |
|---|---:|---:|---:|---:|---:|---:|---|
| Catalog / composition / project plan | **450** | **21,498** | **19,592** | **770,632** | **157** | **21** | `5aa0fa3ffbe07080daaddb96468d56c98215cea41c82b9d00042b9cdf4cb442d` |
| Native ABI / discovery / live host | **114** | **31,190** | **28,461** | **1,121,685** | **346** | **35** | `b13ac5b23f3495807b50273d4406b23934dd1d9889388534ab70c90c011acc05` |
| Runtime VM / script host / scene | **118** | **22,177** | **20,256** | **769,544** | **201** | **9** | `aa40d535b0debb14daee6ddbaf079384e7bbe00e2885feba2b046de82612bdc6` |
| First-party ZrVM plugin | **38** | **5,367** | **4,911** | **190,779** | **46** | **1** | `4f35d10e042364ca1df13bd1e16c40827abf9136ab24c0f1f686421381bda798` |
| Product tests / examples | **61** | **8,903** | **8,281** | **344,654** | **143** | **10** | `5e311955cbb99437af49aba8d74fb5a0f44ae5cbb39837ddc6c8eccd1e2f29ca` |
| Five-engine reference corpus | **16** | **12,526** | **10,786** | **457,780** | **19** | **0** | `ec2e7c383525b97e4619541f42b05d20a227ee7696f1e40619649daf711be29c` |
| Selected production dedup | **273** | **60,151** | **54,946** | **2,136,902** | **600** | **45** | `5a9dc232c34f2b8a070ea3318c1770779c5a8401df593f5e3b63bdf2ae776312` |

本轮只做静态 review 与文档更新，没有修改 production/test/Cargo/ABI，也没有运行 Cargo、App、Editor、Runtime DLL、真实 ZrVM、真实第三方 DLL、crash worker、stress、fault、security、soak 或动态 benchmark。Tooling 按用户要求排除；本报告不查询、轮询、等待或实时跟踪协调器状态，也不把会话状态当作源码证据。

成文期间相关 source 与 optimize 文档持续存在本地修改，`zircon_plugins/plugin_sdk/src/native.rs` 也处于未提交改写状态。本文不循环追逐持续变化的工作树；`source_recheck_required` 与 `working_tree_drift_observed_after_snapshot` 保持 `true`。实施任何 milestone、关闭 finding 或发布 ABI 前，必须重新取完整选择集和产品 build identity。

### 2.2 固定架构边界

本报告沿用仓库 C3 边界，不建立第四个根 package，也不把跨 DLL 共享 Rust object 当作稳定插件架构：

- `zircon_app` 拥有进程、Runtime DLL host、native library/worker 启动、操作系统 trust policy落地和产品 session bootstrap。
- `zircon_runtime` 拥有 package resolution、唯一 compiled generation、slot lifecycle、script execution policy、scene scheduling与backend-neutral language/debug contract。
- `zircon_plugins` 提供 SDK、first-party package、backend adapter 与 distribution artifact；它不自行成为第二个产品 catalog authority。
- `zircon_editor` 只拥有安装/启用/权限/诊断/debug UX 和 Play workflow projection；它消费 Runtime/App operation contract，不私建 loader、catalog、bridge 或 VM owner。
- dynamic ABI/IPC 只传 versioned POD、stable handle、bounded pages与structured receipt；所有 identity 都带 owner/generation，所有 variable payload 都有长度、来源、预算与释放协议。

## 3. 当前产品链与断点

### 3.1 Catalog、project plan 与 dynamic session

```text
package discovery
  -> RuntimePluginCatalogCandidate(base snapshot)
  -> prepare + ArcSwap CAS publication
  -> CompiledProjectPluginPlan(catalog generation/fingerprint/target)
  -> RuntimeModuleCompositionCompiler / App bootstrap identity
  -> LinkedRuntimePluginPlan + Dynamic Session pin
  -> startup scripts + extension world plan

native live host / VM slot / bridge lifecycle / world publication
  -X- still own independent activation and retirement state
  -X- no one transaction publishes every contribution as one generation
```

`generation.rs`、`candidate.rs`、`publication.rs` 和 `project.rs` 已把 catalog 从裸版本号提升为 typed immutable publication，这应作为重构起点。App builtin module 创建 catalog 与 compiled project plan，engine entry 保存 composition identity；dynamic session 在构造期 pin catalog snapshot、compiled plan字段与 linked extension world plan，并执行 `load_startup_scripts`。这些不是 mock-only stub。

但一个 plan identity 还没有成为所有 backend 与 consumer 的唯一事实。native live host 有自己的 loaded generation/callback transition，VM coordinator有 slots与global lifecycle，bridge/extension和world application仍有独立 publication；batch native load按插件推进，session/catalog/world也按阶段提交。当前系统能证明某些局部 publication 不变，却不能证明跨 package、backend、extension、system、script和world的失败原子性。

### 3.2 Native SDK、loader 与 live host

```text
manifest/discovery budgets
  -> path + engine compatibility checks
  -> Library::new(in editor/runtime process)
  -> descriptor/entry/behavior registration probe
  -> per-plugin callback admission + Arc<Library> pin
  -> synchronous invoke/save/restore/unload
```

native discovery已有路径、数量、字节、deadline、cancellation与 last-good publication等保护；V4 command output使用 host-owned bounded sink，callback lease也能在transition bit关闭后拒绝新调用并固定旧 library lifetime。这关闭了“锁内 foreign callback”和“command output完全无界”两个局部后果。

边界仍不足。加载前没有 signer/trust/revocation/quarantine verdict，兼容性检查后即 `Library::new`，descriptor验证发生在代码已经映射进主进程之后。transition在 active callback存在时立即报 busy，不提供等待到 deadline 的 drain ticket；batch load和跨 registry publication非原子。save/restore/unload、command input、foreign diagnostics、resident handles、wall/CPU time没有统一 policy。SDK中的 owned buffer仍把 foreign pointer/len/cap/token/free callback交给host，token构造不能替代不可伪造owner identity，任意 lifetime 的 borrowed slice helper也没有建立来源和最大长度证明。

当前 dirty SDK 还出现了 `NativePluginByteSliceV3`、`NativePluginOwnedByteBufferV3`、`NativePluginCallbackStatusV3` 及 save/restore/unload function type 对自身再次定义的别名。这在 Rust type namespace 中是静态重复定义候选，属于 Plugins01 ABI owner 下必须先清除的 build blocker；本轮未运行 Cargo，故不伪装成动态编译认证，也不重复新增 P0。即使删除这组明显别名，V3 descriptor、V4 behavior、entry layout epoch 5仍需要单一 schema 和完整 conformance matrix才能视为 ABI hard cut完成。

### 3.3 VM slot、真实 ZrVM 与执行预算

```text
Runtime HotReloadCoordinator
  global lifecycle mutex
  slots: HashMap<Slot, Option<Instance>>
  call/GC: take instance -> catch unwind -> restore instance
  reload: save -> migrate -> activate -> reflection commit/rollback

ZrVM real backend
  process-wide OnceLock<Mutex<()>>
  compile/load/call/GC/drop all serialized
  unsafe Send/Sync justified by that global serialization
```

panic-safe take/restore修复避免了普通 panic 后永久丢失 slot；reflection staging、rollback和GC telemetry也是真实基础。但这仍不是 call lease。相同 slot 的嵌套或并发调用在实例被取走时只能收到 busy/unavailable；reload没有先close admission、等待 active call/task/host handle、原子切换generation、再延迟retire旧generation的协议。不相关slot也被global lifecycle mutex串行。

真实 ZrVM更进一步把所有domain交给一个进程级mutex。默认 feature为空，`backend-zr-vm`依赖 workspace 外路径和环境动态库；当feature未启用时只是backend unavailable。即便产品feature显式启用，global lock仍会让一个package的长call、GC、compile或drop阻塞所有package/world。`VmPluginMemoryPolicy`只验证soft/hard数字，`VmPluginInstance::call_export`没有 execution context，`VmError`也没有 deadline/cancel/fuel/OOM/poison/quiesce等机器可判定终态。

### 3.4 Scene scripting、typed data 与工具链

Scene system已缓存 `onStart/onUpdate/onFixedUpdate` callback handle并按binding generation重建；但生命周期缺少init/enable/disable/destroy、scene enter/exit、pause/resume和reload转换。binding identity由package/module/index字符串和JSON数组位置组成，reorder或局部编辑会改变身份；thread-local cache一次只投影一个world；首个错误以 `?` 终止剩余bindings，没有component/package/world级failure policy和聚合receipt。fixed/update/start排序也未形成可验证状态机。

`ScriptHostValue` 已支持 scalar、string、bytes、host handle和borrowed argument view，typed descriptor/reflection registration也在扩展；但typed array/map/struct/enum、nullable/result、entity/component/resource lease、out/ref、async continuation和共同 marshalling plan仍缺失。复杂scene binding/state继续使用JSON，`HostHandle(u64)`没有把owner/type/generation编码进公共合同。运行时也没有 backend-neutral breakpoint/step/stack/locals/watch/evaluate、source map、per-export inclusive/exclusive time、allocation/host-call/GC/reload trace。

## 4. Runtime07 canonical finding 重判

状态规则：`Closed` 必须同时具备唯一 owner、产品调用链、失败/退役合同与对应动态资格证据；`Partial` 表示 current source 已有可保留实现，并确实消除了旧finding的至少一个后果。类型存在、source guard或fixture通过不能单独关闭finding。

| Canonical finding | 当前状态 | 当前源码判据 | 硬切重构要求 |
|---|---|---|---|
| P1-1 多个平行 plugin authority，没有单一 compiled generation | **Partial** | typed catalog generation、prepared CAS、compiled project plan、composition/session pin已存在；native/VM/bridge/world仍各自持代际与状态 | `PluginResolutionPlan -> PluginCatalogGeneration -> Backend/WorldGeneration`一次编译、一次事务发布；删除重复builder与产品可见registry真相 |
| P1-2 batch load/activate/reload/publication不失败原子 | **Partial** | VM与native有局部staging/rollback；native batch和catalog/backend/world仍逐插件或逐owner提交 | prepare/quiesce/activate/validate/publish/retire统一transaction；消费者只能观察完整old或new generation |
| P1-3 dependency不是版本/来源/信任求解器 | **Open** | manifest依赖仍不足以表达version range、source、digest、signer、conflict/provide、target/profile与lock identity | 单一deterministic resolver生成可重放lock/artifact和完整拒绝链；Editor/Runtime/Export/Cook共用 |
| P1-4 native DLL主进程直载，无信任与故障隔离 | **Open** | load前无签名/信任/quarantine；capability/catch_unwind不能限制直接OS访问、crash、abort、hang或内存破坏 | trusted in-process与isolated worker分级；签名、来源、revocation、CPU/memory/time/IPC预算和crash quarantine成为admission gate |
| P1-5 ABI epoch与兼容责任不清 | **Partial** | public surface正向V3 hard cut，static callback set改为closed unsafe trait；但当前SDK有自引用重复别名候选，V3/V4/layout epoch 5仍分离且无生成schema/conformance corpus | 一张schema生成Rust/C header/decoder；明确read/execute/upgrade matrix，跨MSVC/GNU/Linux/macOS验证布局、调用约定与allocator |
| P1-6 native callback预算不完整 | **Partial** | command output已host-owned且有hard cap，callback有generation lease；input/state/diagnostic/handles/time/cancel与hung containment仍缺 | per-slot `NativeExecutionPolicy`覆盖所有bytes/handles/time/thread；大state走host-owned page/artifact，timeout生成structured terminal status |
| P1-7 ZrVM进程级mutex串行所有domain | **Open** | real backend compile/load/call/GC/drop仍在同一global lock，unsafe Send/Sync依赖该事实 | 先证明VM线程模型；可隔离则per-domain owner，不可重入则worker actor/process并公开限制，不用全局锁伪装并发 |
| P1-8 VM slot无call lease、quiescence与reentrancy合同 | **Partial** | panic-safe take/restore和generation callback已实现；per-slot admission/drain/deadline/retire仍在M1计划，global lifecycle和immediate busy仍在 | `VmCallLease`固定generation；close admission、bounded drain、atomic switch、lease-zero retire；reentrancy明确allow/deny和call chain |
| P1-9 memory policy未执行，调用无fuel/deadline/cancel | **Open** | soft/hard只解析校验，call trait无execution context，GC deadline不能约束普通export | allocator/VM/host统一 `ScriptExecutionPolicy`，预算继承nested/async/host call并产生typed trap |
| P1-10 scene lifecycle、identity、调度与失败隔离不完整 | **Partial** | start/update/fixed、callback cache和binding generation存在；生命周期窄、identity依赖数组index、fail-first、无access schedule | stable component id + 完整lifecycle状态机；按access/backend能力编译batch；错误按component/package/world policy聚合 |
| P1-11 script value/reflection/gameplay边界过窄 | **Partial** | borrowed values、bytes、typed descriptors/reflection是进展；复杂对象仍JSON，handle无type/owner/generation，缺aggregate/lease/async | reflection生成 `TypeId + SchemaGeneration + MarshallingPlan`；热路径typed views/leases/commands，JSON只留文档与诊断边界 |
| P1-12 无debugger/source map/profiler/execution trace | **Open** | 只有build/log location和局部GC/hotpath counters，没有运行中debug adapter和per-export归因 | backend-neutral debug/profiler合同，pause与reload/world lock协调；shipping保留可解析crash/source artifact与低开销trace |
| P1-13 VM package不可重现、不可验证、依赖合同不足 | **Open** | manifest仍偏开发输入；project path/runtime compile、raw bytecode、缺compiler/options/dependency hashes/signature/pages/source map | source/compiled/installed分层；runtime只加载resolved、verified、paged artifact，并纳入同一package solver |
| P1-14 缺真实产品、并发、故障与性能证据 | **Open** | dynamic session会加载startup script，但real backend上层仍有10项ignore；feature/env/fixture不能证明产品链 | Editor/Play/App/export/cook全链 + real backend/native DLL + stress/fault/security/soak + 同机参考trace |
| P2-1 control-plane DTO/string index/full rebuild重复 | **Partial** | compiled plan、dense/cached index、immutable projection和borrowed路径已有多项改进；重复authority和调用期字符串/JSON仍在 | generation build一次intern/compile dense spans；增量affected-slot rebuild；记录build/copy/lookup/call分位数 |
| P2-2 test/source guard不能替代ABI/并发/故障/产品证据 | **Open** | 单元与结构测试很多，但foreign ABI、real crash/hang、active-call reload、签名链和shipping session未覆盖 | 给测试标unit/property/integration/product/stress/fault/security/performance；关键harness保存manifest/trace/crash/artifact |

汇总：P1 **7 Open / 7 Partial / 0 Closed**；P2 **1 Open / 1 Partial / 0 Closed**。

## 5. 专项 P0 与 current-source blocker 去重

Runtime99p、Plugins01、Plugins16和Plugins21已经覆盖 plugin interface bridge、native ABI、ZrVM backend与安装/信任边界中的P0。本文不把同一根因换名重复累计，也不依据dirty working tree替其他owner关闭P0。

| Owner | 本轮可见状态 | Runtime164处理 |
|---|---|---|
| Plugins01 native ABI / SDK | 当前 SDK V3改写中出现同名类型和函数指针自引用别名，静态上会与已声明item重复；foreign buffer、entry panic与跨编译器conformance仍未闭合 | 记录为原owner的build blocker candidate；实施前先恢复可编译单一surface，再跑ABI matrix，不新增唯一P0 |
| Plugins16 ZrVM backend | real backend仍default-off、依赖外部path/env library并由global mutex串行；owner已有真实backend fixture但不是产品session | 保留owner；Runtime164只定义跨Runtime slot/domain/execution-policy的验收依赖 |
| Plugins21 install/trust/distribution | 未形成可签名、可撤销、可隔离的installed package authority | 保留owner；P1-3/P1-4消费其package store与trust verdict，不另造marketplace/install owner |
| Runtime99p bridge/slot generation | stable handle与部分generation/lease基础存在；App/Editor产品bridge retention、native replay、VM product lifecycle仍不完整 | 保留owner；统一纳入M1/M2 generation transaction和产品验收 |
| Runtime99zj/module composition | composition compiler/identity已进入App，但backend/world contribution尚未与catalog同事务 | 作为P1-1/P1-2依赖，不复制module/service owner |

任何 blocker 只有在重新读取current source、取得可执行build/test artifact并通过原owner资格门后才能关闭。单独删除SDK重复别名只能修复一处编译问题，不能证明ABI、allocator、panic、trust或hot reload完整。

## 6. 参考引擎差距

### 6.1 Bevy 与 Fyrox

Bevy `Plugin` 明确build、ready、finish、cleanup阶段，PluginGroup负责有序组合、启用/禁用与重复检测。Zircon typed catalog/composition方向与其“先组合、后分阶段进入App”的边界一致，但Zircon还需要动态package、ABI、generation lease、失败原子publication和trust/isolation，不能把Bevy静态Rust plugin模型当最终上限。

Fyrox plugin面提供dynamic plugin prepare/reload和更宽的生命周期边界，可用于校验Zircon当前三回调scene lifecycle明显不足。其in-process dynamic model和局部reload同样不能替代签名、worker isolation、fuel与跨plugin原子generation；Zircon应保留自身typed generation优势，而不是复制unload-first行为。

### 6.2 Godot

Godot `ScriptLanguage` / `ScriptInstance` 把语言注册、脚本实例、reload、debug stack/local/global/expression、profile与thread hooks放在显式语言合同中；`GDExtensionManager`区分load status、initialization/deinitialization level和reload边界。Zircon当前host export/reflection/GC分别进展，却没有统一language adapter与debug surface。目标应达到同等可诊断性，同时使用强generation、typed handle和budget避免global singleton与裸identity限制。

### 6.3 Unreal

Unreal `ModuleManager` 显式表达load/unload/abandon、failure reason、module change与dynamic library lifetime，并对线程/卸载边界有明确限制；`PluginManager`负责descriptor、enabled graph、版本选择、dependency/reference chain、compatibility与content/module mount。Zircon最关键差距是尚无“同一个resolved plugin graph控制全部backend和贡献”的产品事实，也没有一套可解释的prepare/quiesce/publish/retire报告。

Zircon的目标应超过传统in-process module manager：确定性digest/signature、失败原子batch、generation lease、host-owned预算和可选worker isolation都应成为默认工程边界。只有这些合同与同机产品trace同时通过，才有资格讨论优于Unreal，而不是依据类型数量或微基准宣称领先。

### 6.4 Unity Graphics

本地 Unity Graphics 镜像不是通用脚本VM或native plugin host参考。本轮只使用其versioned package dependency和Runtime/Editor/Test asmdef分区，校验“发布artifact、运行时程序集、编辑器程序集和测试程序集必须可独立声明依赖与构建边界”。它不能证明Zircon已经对齐Unity Engine的脚本、domain reload、package manager或安全模型；相关能力仍须由Zircon自身产品资格证明。

## 7. 目标架构

```text
PluginPackageStore
  verified source/digest/signer/revocation/artifacts
        |
        v
PluginResolver -> immutable PluginResolutionPlan / lock artifact
        |
        v
PluginGenerationBuilder (staging)
  static | trusted native | isolated native | ZrVM | future language
  interfaces + systems + extensions + content + script/debug metadata
        |
        v
PluginTransitionCoordinator
  prepare -> close admission -> bounded drain -> activate/validate
          -> single publish -> lease-zero retire / rollback
        |
        v
PluginCatalogGeneration (only product-visible authority)
  App / Runtime / Editor / Export / Cook pin the same identity

Script domain per generation
  execution policy + call/task/state/handle leases
  scene lifecycle scheduler + typed marshalling plan
  debugger/source map/profiler + structured terminal receipts
```

### 7.1 必须统一的公共合同

1. `PluginPackageIdentity`：name/version/source/content digest/signer/target/profile/build，不能只靠路径或字符串id。
2. `PluginSlotHandle(index,generation)`：所有backend callable、interface、system、extension、content和debug symbol都从同一slot generation派生。
3. `PluginTransitionTicket`：受影响slots、旧/新generation、admission close、active lease census、deadline、rollback/retire report完整可观测。
4. `NativeExecutionPolicy` 与 `ScriptExecutionPolicy`：bytes、alloc、handles、threads/tasks、fuel、wall/CPU、host calls、cancel和shutdown预算从外层调用继承。
5. `ScriptTypeId + SchemaGeneration + MarshallingPlan`：标量、aggregate、borrowed view、host lease、command buffer与state migration由同一reflection schema编译。
6. `ScriptComponentId(index,generation)` 与 lifecycle state：编辑、reorder、scene transition、world clone、reload和destroy都保持exactly-once或显式迁移结果。
7. `ScriptDebugAdapter` / `ScriptProfiler`：source/document/module/export/entity/generation贯穿breakpoint、stack、trace、crash artifact与shipping counters。

旧catalog builder、registry双写、裸u64 handle、产品JSON gameplay ABI、业务层直接 `Library::new` 和global-VM-lock假并发必须进入删除清单。迁移期adapter只能位于明确兼容owner，不能从root facade长期暴露第二authority。

## 8. 重构序列

### M0：恢复可验证基线并冻结manifest

- 重新读取SDK/runtime/plugin当前diff，先解决Plugins01下的同名V3 item blocker candidate，取得Windows默认profile和目标feature build证据。
- 生成所有catalog/generation/registry/cache、load/reload/unload、V3/V4/layout epoch、native buffer/callback和VM slot入口清单。
- 冻结Schema/Owner/ABI/Operation/Artifact manifest；未知来源native package默认禁止产品in-process自动加载。

### M1：package resolver与唯一generation

- PluginPackageStore交付source/digest/signature/revocation/quarantine和artifact lifetime。
- deterministic resolver处理version/optional/conflict/provide/target/profile/trust并生成lock artifact和稳定错误链。
- bootstrap、dynamic session、editor、export/cook硬切共用同一resolution plan；backend/bridge/extension/world projection都由一个generation builder生成。

### M2：原子transition与per-slot lease

- native/VM/static backend统一prepare、close admission、bounded drain、activate、validate、single publish、retire/rollback。
- VM call取得per-slot generation lease；reload等待call/task/state/host-handle census到deadline，不相关slot可并发。
- 删除global lifecycle serialization和“take Option instance作为主要并发协议”；reentrancy按backend capability给出专用结果。

### M3：执行预算、ZrVM domain与native isolation

- memory policy接入allocator/load/call/GC，fuel/deadline/cancel/host-call/nested/async预算贯穿整个调用树。
- 根据ZrVM真实线程模型选择per-domain owner或受监管worker actor；移除以process-global mutex支撑的公开并发承诺。
- trusted/isolated native分级；worker拥有CPU/memory/time/output/IPC/handle预算、crash restart和quarantine，in-process只接受显式trust policy。

### M4：scene lifecycle、typed ABI和调试性能工具

- stable script component identity与init/enable/start/update/fixed/disable/destroy/reload状态机进入ECS schedule。
- reflection生成typed marshalling/state migration plans；frame gameplay热路径退出JSON/string lookup。
- 实现source maps、breakpoint/step/stack/locals/watch/evaluate、per-export profile和shipping crash trace，并与reload/pause/world lock协调。

### M5：产品、故障、安全与性能资格

- Vampire/WoC覆盖project open、Editor Play、hot reload、stop、App dynamic session、standalone、export/cook、restart与artifact重放。
- 执行ABI、signature、malformed package、foreign allocator、access violation、abort、hang、OOM、active-call reload、1/8/24h稳定性和1/10/100 package/world矩阵。
- 与参考引擎在同硬件、同项目规模、同build/profile下比较startup/reload/frame script CPU/RSS/p50/p95/p99/GC/hitch/tool latency；结果绑定source/binary/package/trace identity。

## 9. 资格门重判

| Gate | 当前状态 | 缺失证据 |
|---|---|---|
| G1 单一resolved/compiled product generation | **Partial** | typed catalog/plan/session pin存在；native/VM/bridge/world仍有平行authority |
| G2 跨package/backend/contribution失败原子transition | **Partial** | 局部rollback存在；无整批close/drain/single-publish/retire事务 |
| G3 version/source/digest/signer/trust deterministic resolver与lock | **Fail** | manifest和solver合同不足，Editor/Runtime/Export/Cook无法重放同一图 |
| G4 native pre-load trust admission、隔离与revocation | **Fail** | 主进程直载，校验与capability不能建立OS/crash/hang边界 |
| G5 单源ABI schema、header与跨工具链/外语conformance | **Fail** | 当前SDK仍有静态重复定义候选，epoch/allocator/布局/调用约定矩阵缺失 |
| G6 native callback全输入/输出/state/time/handle故障控制 | **Partial** | bounded command output与lease存在；其他payload、deadline/cancel/hang/isolation缺失 |
| G7 ZrVM多domain/多world并发与故障隔离 | **Fail** | process-global mutex串行所有真实backend动作 |
| G8 VM slot call lease、reentrancy、bounded quiesce与retire | **Partial** | panic-safe take/restore存在；per-slot lease和drain仍未实现 |
| G9 script fuel/deadline/memory/host-call/cancel执行预算 | **Fail** | policy只声明，普通call无execution context和typed budget trap |
| G10 scene完整lifecycle、stable identity、schedule与failure policy | **Partial** | start/update/fixed与cache存在；identity、destroy/enable/disable、聚合错误与access schedule缺失 |
| G11 typed value/reflection/marshalling/state migration热路径 | **Partial** | borrowed scalar/bytes与reflection存在；aggregate/lease/schema plan和JSON退出未完成 |
| G12 debugger/source map/profiler/shipping trace | **Fail** | 无backend-neutral runtime debugging和per-export/entity归因 |
| G13 verified/reproducible/paged script package artifact | **Fail** | runtime仍可依赖project path/feature/env，缺compiler/options/dependency hash/signature/source map |
| G14 Editor/Play/App/standalone/export/cook真实产品session | **Fail** | 启动接线存在但上层real-ZrVM仍有10项ignore，fixture不覆盖完整产品 |
| G15 stress/fault/security/soak资格 | **Fail** | 无crash/hang/OOM/signature/active-reload/long-run受管artifact |
| G16 同硬件参考引擎性能与稳定性对照 | **Fail** | 无同场景source/binary/trace绑定的CPU/RSS/latency/GC/hitch数据 |

汇总：**10 Fail / 6 Partial / 0 Pass**。

## 10. 当前状态

`review_complete；implementation_partial；product_atomicity_isolation_execution_tooling_and_qualification_pending；recheck_required`

本文只完成Runtime07 current-source重判、专项owner去重、参考引擎对照和依赖顺序，不修改生产实现。typed catalog generation、compiled project plan、native callback lease/output sink、VM rollback/GC与scene三回调是应保留基础，但没有任何canonical finding达到Closed。进入M0前先恢复SDK可验证基线并重取fingerprint；没有真实Cargo/profile、foreign ABI、产品session、crash/fault、安全、长时和同硬件benchmark证据时，任何“工程级完成”或“性能优于Unreal”的声明均不成立。
