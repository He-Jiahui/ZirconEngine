---
title: Runtime Manager Resolver、Named Service、Handle、Generation、Lifecycle 与 Product Integration Current Source Review
category: zircon_runtime
report_id: Runtime123
review_date: 2026-08-23
baseline_head: 01d3ebc247f8f6027f4eacc47567a7ceb2a11621
baseline_epoch: 369
supersedes:
  - docs/plans/optimize/zircon_runtime/50-runtime-manager-resolver-named-service-handle-generation-lifecycle-product-integration-review.md
related_code:
  - zircon_runtime/src/core/manager
  - zircon_runtime/src/core/runtime/descriptors/manager_descriptor.rs
  - zircon_runtime/src/core/runtime/descriptors/registry_name.rs
  - zircon_runtime/src/core/runtime/handle
  - zircon_runtime/src/core/runtime/state/service_entry.rs
  - zircon_runtime/src/core/runtime/state/core_runtime_state.rs
  - zircon_runtime/src/core/runtime/error.rs
  - zircon_runtime/src/foundation/module.rs
  - zircon_runtime/src/asset/module.rs
  - zircon_runtime/src/input/module/descriptor.rs
  - zircon_runtime/src/platform/module.rs
  - zircon_runtime/src/animation/module.rs
  - zircon_runtime/src/navigation/module.rs
  - zircon_runtime/src/scene/module
  - zircon_runtime/src/ui/module.rs
  - zircon_runtime/src/script/vm/module/module_descriptor.rs
  - zircon_runtime/src/graphics/runtime_builtin_graphics/host/module_host/module_registration
  - zircon_runtime/src/dynamic_api
  - zircon_runtime/src/runtime_diagnostics
  - zircon_editor/src/ui/host
  - zircon_editor/src/ui/retained_host
  - zircon_plugins
tests:
  - zircon_runtime/src/core/manager/tests.rs
  - zircon_runtime/src/core/runtime/tests/resolution
  - zircon_runtime/src/core/runtime/tests/activation
  - zircon_runtime/src/tests/extensions/manager_handles.rs
  - zircon_runtime/src/tests/runtime_absorption/service_registry_lifecycle.rs
  - zircon_editor/src/tests/host
plan_sources:
  - docs/plans/mvp/index.md
  - docs/plans/optimize/zircon_runtime/50-runtime-manager-resolver-named-service-handle-generation-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/01-core-runtime-lifecycle-registry-review.md
  - docs/plans/optimize/zircon_runtime/02-core-runtime-events-tasks-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/42-builtin-runtime-module-catalog-profile-target-feature-selection-extension-registration-capability-load-report-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/46-engine-module-service-contract-context-factory-descriptor-snapshot-composition-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_runtime_interface/01-runtime-dll-abi-ffi-version-handle-foreign-ownership-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_app/01-product-host-bootstrap-loop-dynamic-runtime-shutdown-review.md
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02/2026-08-15-runtime-module-service-lifecycle-current-architecture-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/Subsystems/Subsystem.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/Subsystems/SubsystemCollection.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Subsystems/SubsystemCollection.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/Subsystems/EngineSubsystem.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/Subsystems/WorldSubsystem.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/Subsystems/LocalPlayerSubsystem.h
  - dev/bevy/crates/bevy_ecs/src/system/system_param.rs
  - dev/bevy/crates/bevy_app/src/sub_app.rs
  - dev/godot/core/config/engine.h
  - dev/godot/core/config/engine.cpp
  - dev/Fyrox/fyrox-impl/src/plugin/mod.rs
  - dev/Fyrox/fyrox-impl/src/engine/mod.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Common/ContextContainer.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Volume/VolumeManager.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Tests/Runtime/ContextContainerTests.cs
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 99x · Runtime Manager Resolver Current Source Review

## 1. 结论

当前 manager 层不是空壳。`core::manager` 已提供只持 `CoreWeak` 的 `ManagerResolver`、canonical name、typed `ManagerServiceHandle<T>`、`RegisteredManagerService<T>` 和 generation 复核；统一 service kernel 内还存在 `ServiceHandle<T>::enter()` / `ServiceCallGuard<T>`、admission close、in-flight drain、lazy factory 并发互斥与 module deactivate/reactivate generation 失效测试。这些都是可保留底座。

但 production manager facade 仍然绕过最关键的调用期合同。`resolve_manager_service` 只在解析瞬间按 `RegistryName` 查表和校验 `index/generation`，随后从 `RegisteredManagerService<T>` clone 裸 `Arc<T>`；产品调用期间没有 `ServiceCallGuard`，不会计入 in-flight，也没有 provider code lease。模块关闭 admission 后，先前解析出的 manager 仍可继续被调用；对 native plugin trait object，这会让 vtable 执行和 DLL 卸载失去同一生命周期权威。全仓 production 搜索仍只有 core 定义使用 `ServiceCallGuard` / `resolve_manager_handle`，没有一个 manager 产品 caller 调用 `enter()`。

注册模型也没有收敛。AI、Animation、Net、Physics、Sound、Scene 等模块继续同时发布 concrete `Default*Manager` 和包装后的 trait manager；Editor、Script、Particles、Texture、UI 仍直接发布 raw concrete manager。六个 Net feature 又各自注册 manager，其中 HTTP/WebSocket 等 factory 构造独立 `DefaultNetManager`，虽然 descriptor 声明依赖 canonical `NetManager`，却不消费该 provider。这不是单一 subsystem graph，而是名称、依赖边和 factory 约定拼出的多轨 service locator。

性能路径仍由完整名称和全局可变表支配。`ManagerServiceHandle` 的 `index/generation/service` 仍为 public；解析先按完整 `RegistryName` 获取 `Mutex<HashMap<...>>`，取出 entry 后才检查 index。`RegistryName` 已缓存校验后的 module/kind/service offsets，避免重复切分，这是局部实现质量；但 equality/hash/borrow/serde 权威仍是完整 `String`，没有 dense slot 直达。Graphics 当前有 32 处 `.asset_manager()` 调用，dynamic runtime、Editor viewport、diagnostics 与插件热路径继续反复 hash、锁、downcast 和 clone `Arc`。

本轮裁决为 **0 项本地 P0；52 P1 Open、4 P1 Partial、0 Closed；14 P2 Open；26 Gate Fail、10 Gate Partial**。Partial 只承认四类局部证据：两个粗粒度 resolve profiling scope、诊断显式 `available=false + error`、内核 guard/identity/admission 行为测试，以及 Editor cancel-aware handle 获取；它们都没有让 production manager 调用获得 scope、call lease、provider lease 或 O(1) slot。

本轮只做 current-source 静态审查与文档记录，没有修改 production、tests、Cargo、ABI 或参考源码；没有运行 Cargo、真实 DLL unload、Editor/WGPU、feature powerset、multi-world/session、soak 或 benchmark。MVP `00` 仍为 `in_progress`，F0-F5 仍 blocked；本文不把共享工作树候选、ignored benchmark 或静态测试写成 accepted milestone，也不展开 tooling 优化。

## 2. 审查边界与物理冻结

### 2.1 Zircon focused 集合

| 范围 | 文件 / 行 / bytes / tests | fingerprint |
|---|---:|---|
| manager/service kernel owner | 13 / 3,071 / 108,920 / 1 | `fd659506cf641ee13a1ee1e8301ea53e9f3479ada0a758a2bcf3575cb8b5534f` |
| production registration | 26 / 2,463 / 89,827 / 1 | `1140f23f7d7bea0df479ca9b13e0fd32f5eb8c3f65d60addde7c652afd2da313` |
| production caller | 33 / 7,364 / 279,048 / 47 | `9ce3ae4ec2b9fb2dae37487e0a13c1763451b1e886e50ae81ac1376471c0a121` |
| focused tests / test support | 106 / 35,286 / 1,299,663 / 518 | `f03c568db8a429bbb7570d3275bb938dbca8347b5fa57bc4ff75337287adfef6` |
| 五引擎参考实现与直接参考测试 | 15 / 11,338 / 426,099 / 28 | `7a286bb751cdaa1d69abed7427b8785eb6886417cb19e6d15a7508eb88623172` |

Zircon 四组为不重叠分区，共 178 个 Rust 文件、48,184 行、1,777,458 bytes、567 个 `#[test]`，总 fingerprint 为 `a81dcb9b4d416c5acec556278c68da2601ddeb713665a42e2a4994b2e0a201f1`。fingerprint 算法：仓库相对路径转 `/` 后小写并 ordinal 排序去重；每项编码为 `lowercase-path + NUL + lowercase per-file SHA-256`，以 LF 连接且末尾无 LF，再计算 UTF-8 SHA-256。它只冻结本轮实际读取集合，不是 runtime identity、BuildSet、ABI、artifact 或 release identity。

### 2.2 currentness

从 Runtime50 基线 `bea1acf91b909525ab1759e2c800858b0eda6528` 到本会话注册基线 `01d3ebc247f8f6027f4eacc47567a7ceb2a11621`，focused 集合有 43 个文件变化；当前 working tree 又有 24 个 focused 文件处于其他会话/用户改动中。关键 `core::manager`、`ManagerDescriptor`、`RegistryName`、service identity 与 `ServiceEntry` 相对旧基线没有语义变更，`resolution.rs` 的差异仅为格式；变化主要集中于 activation/diagnostics、dynamic session、Asset/Editor caller 和测试扩充。

本报告按当前 working tree 逐项读取，不覆盖、不暂存、不回退共享改动。由于 `core/runtime/error.rs`、activation/runtime state、dynamic session、diagnostics 和产品 caller 并非 clean baseline，`source_recheck_required` 保持 true；任何实现切片开始前都必须重新冻结 owner/caller/deletion matrix。

### 2.3 当前双轨调用图

```text
ModuleDescriptor
  +-> raw concrete service: Arc<DefaultXManager>
  |     -> CoreHandle::resolve_manager::<DefaultXManager>() -> Arc<T>
  |
  +-> trait wrapper service: RegisteredManagerService<dyn XManager>
        -> ManagerServiceHandle<dyn XManager>
        -> RegistryName -> global Mutex<HashMap>
        -> validate index/generation
        -> clone Arc<dyn XManager>             // guard/admission在这里结束

正确底层但零 production manager caller：
ServiceHandle<T>::enter() -> ServiceCallGuard<T> -> in_flight++/--
```

## 3. 当前源码事实与状态变化

| 主题 | 当前证据 | 裁决 |
|---|---|---|
| descriptor contract | `ManagerDescriptor` 仍只有 name/startup/dependencies/factory；无 contract/version/scope/provider/ABI/affinity/access/unload | 相关 P1 保持 Open |
| handle identity | public `index/generation/service`；无 runtime/scope/provider/contract；按 name 查表后才校验 index | P1-006/007/036/038 Open |
| generation | unload 时 wrapping add，`u32::MAX` 后回到 1；旧 handle 最终可重新有效 | P2-014 Open；exhaustion 继续由 Runtime24 拥有 |
| call lease | `ServiceCallGuard`、admission 与 drain primitive 存在且有内核测试；manager facade 返回裸 `Arc` | P1-019..025 Open；G06/G08/G10/G30 Partial |
| production reachability | 26 个 production 注册文件、33 个 production caller；production `ServiceCallGuard::enter()` 为 0 | P1-024 Open |
| registration graph | concrete + trait wrapper 双注册继续存在；Net feature 继续扩张独立 raw manager | P1-001/002/011/012 Open |
| name/index hot path | `RegistryName` 缓存 offsets，但完整 String 仍为 hash/equality authority；全局 map mutex 仍在 steady-state | P1-036..038 Open |
| instrumentation | 只有 `resolve_named_service` 与 `resolve_registered_service` 两个 aggregate scope | P1-044 Partial；G33 Partial |
| diagnostics | Render/Animation/Physics resolve 失败会生成 `available=false` 和字符串 error，不再完全静默；没有 typed reason/provider/scope/retry | P1-046 Partial；G23 Partial |
| Editor viewport | 后台 job 只获取 generation handle，支持 cancellation；真正 resolve 仍返回裸 Arc，且无 deadline/scope/thread contract | P1-050 Partial；G27 Partial |
| tests | kernel 覆盖 held guard、admission drain、runtime identity 与 reactivate stale；manager surface 测试仍以名称/include_str 源码形状为主 | P1-055 Partial；G29/G30 Partial |
| product error/shutdown | caller 继续 `to_string()`；dynamic session cleanup 仍 `if let Ok(manager)` 后推进状态 | P1-047/048 Open；G22 Partial、G24 Fail |
| repeated hot access | Graphics 有 32 处 `.asset_manager()`；dynamic/session/viewport/diagnostics 重复 resolve | P1-040/041 Open；G17/G25 Fail |

## 4. 产品纵切面差距

### 4.1 Runtime 与首方插件

- Foundation、Input、Graphics 主要把 trait object 包成 `RegisteredManagerService`；UI、Script、Particles、Texture 又发布 raw concrete manager，公共 contract 策略不一致。
- Asset 同时发布 `ProjectAssetManager`、`AssetManager` 和 `ResourceManager`；后两者都持有同一个 concrete manager，却成为不同 service entry、generation 和 lifecycle 节点。
- Scene、AI、Animation、Net、Physics、Sound 先注册 `Default*Manager`，再由第二个 factory resolve concrete service 并包装 trait；一个产品能力因此有两套名称、依赖、启动和故障面。
- Net HTTP/WebSocket/Replication/RPC/Reliable UDP feature 的 descriptor 依赖 canonical `NetManager`，但多数 factory 直接新建 feature-local manager，dependency edge 不等于共享 provider/composition。
- Content Download 保存 `ManagerServiceHandle<dyn NetManager>` 并按调用解析，能发现 stale；但每次仍得到裸 `Arc`，下载操作不进入 provider drain，也没有下载 operation 与 provider generation 的一致 lease。

### 4.2 Graphics、dynamic runtime 与 Editor

- `ResourceStreamer` 分散的 32 处 asset-manager 获取没有 phase-local batch borrow；同帧调用会重复 name hash、global lock、downcast 和 Arc clone。
- dynamic render bridge 保存 RenderFramework handle，frame/present 路径反复解析；session 保存 Input handle，frame、event、host-request 和 shutdown 路径分别解析，没有统一 session/scope generation。
- runtime diagnostics 已把 manager resolve 失败投影成不可用，但只保存字符串，无法区分 Disabled、Blocked、Initializing、ProviderFailed、Stale 或 Retryable。
- Editor retained asset access 限制了可解析 manager 的集合，这是较好的 API 收口；返回值仍是可长期逃逸的裸 Arc。
- viewport 后台 job 只生成 handle，避免直接在 worker 执行 manager 方法，并有 cancellation；actual resolve/call 仍没有 thread assertion、deadline、call guard 或 render-device/view scope。

## 5. 五引擎参考差异

| 参考 | 已核对源码/测试能力 | Zircon 应吸收的合同 | 不照搬项 |
|---|---|---|---|
| Unreal | `FSubsystemCollectionBase` 绑定 owner，显式 Initialize/Deinitialize、InitializeDependency、typed single/array lookup、dynamic module add/remove 与 collection teardown；Engine/World/LocalPlayer 分层 owner | scope owner、依赖初始化、动态 provider 增删、集合查询、逆序销毁和调用有效期必须显式 | 不复制 UObject、反射宏、全局 engine singleton 或其线程/GC 成本 |
| Bevy | `Res`/`ResMut` 在 SystemParam 初始化时以 resource component id 登记 read/write access 并拒绝冲突；SubApp 拥有独立 World、Schedule 与 extract/update 边界；inline tests 覆盖 alias/missing resource | typed access declaration、phase conflict、dense typed slot 与 sub-world scope | 不把 Rust TypeId/monomorphization 当跨 DLL 稳定 ABI，也不强制所有 manager 变 ECS resource |
| Godot | Engine singleton 记录 name/pointer/class/user-created/editor-only，拒绝 duplicate，支持 has/get/list/remove，并按 editor policy 隐藏 | discovery/status catalog、duplicate provider fail-close、origin/policy 与对称 remove | 不复制裸 pointer singleton 或无 generation 的全局表 |
| Fyrox | `PluginRegistrationContext` 与 `PluginContext` 分离注册期/使用期能力；Plugin trait 区分 register/init/on_loaded/on_deinit 和 graphics-context create/destroy | registration/use/deinit 阶段能力分离，使用期传入受限 borrow/context | 不复制过宽 context，也不把 context 变成新的万能 locator |
| Unity Graphics | `ContextContainer` 用 typed slot array、active set、Create/Get/GetOrCreate/Dispose 做局部上下文复用，测试验证 duplicate、reset 和 reuse 后无分配；VolumeManager 显式拥有 stack 创建/销毁/更新 | render 局部 dense context、active set、批量 reset 与 allocation qualification | 只作为 Graphics 局部容器参考，不外推为完整 engine service architecture |

这些参考实现并不共同指向某个具体类层级，而是共同证明：工程级 service 必须有 owner/scope、typed contract、受控访问、显式生命周期和可验证失败。Zircon 可以用 immutable composition、dense slot 和更短调用路径争取性能优势，但不能靠删除这些语义来宣称优于 Unreal。

## 6. Canonical owner 边界

| 事实 | Canonical owner | Runtime123 纵切面 |
|---|---|---|
| module admission、in-flight drain、逆依赖 shutdown、失败恢复 | Runtime01、App01 | manager facade 接入 call lease 与产品 terminal receipt |
| scheduler phase、任务取消、线程亲和和 access conflict | Runtime02 | descriptor 暴露 manager access/affinity 声明 |
| qualified identity、generation、owner epoch、index exhaustion | Runtime24 | manager handle 的不可伪造 runtime/scope/contract 投影 |
| profile/capability/provider composition | Runtime42 | manager directory 消费最终 CompositionPlan/BuildSet |
| module/service descriptor、typed binding、context/factory/snapshot | Runtime46 | manager contract 成为统一 service descriptor 的一种投影 |
| native ABI、vtable、DLL unload、provider code lease | Interface01、Plugins01 | trait manager 每次调用持有 provider lease |
| process-global index、HashMap/global mutex/condvar 热路径 | PERF-MVP-628 | manager caller 规模、dense slot 和资格门 |

本篇不重复登记这些父 owner 的 P0。实现时必须更新父 owner 状态和共享 manifest，不能另造 manager-only registry、compat facade 或双写目录。

## 7. P1 差距（56 项）

### 7.1 Contract、identity 与 composition

| ID | 状态 | 当前差距与硬切目标 |
|---|---|---|
| MGR-P1-001 | Open | raw concrete manager 与 wrapped trait manager 两套解析模型并存；硬切为一个 compiled service kernel。 |
| MGR-P1-002 | Open | trait wrapper 不是 descriptor contract，而是额外普通 service；contract 必须成为 descriptor 字段。 |
| MGR-P1-003 | Open | 捕获 `ManagerServiceHandle<T>` 只确认名称存在，不验证 entry 提供 `T`；composition 阶段绑定 type/contract。 |
| MGR-P1-004 | Open | service object 以 `Any` 保存，错误 contract 到 use-time downcast 才失败；activation 前 fail-close。 |
| MGR-P1-005 | Open | descriptor 无跨 crate/DLL 稳定 contract key/version；建立版本化 `ServiceContractId`。 |
| MGR-P1-006 | Open | handle 三字段 public，可伪造矛盾 identity；字段私有且只能由 runtime directory 签发。 |
| MGR-P1-007 | Open | handle 无 runtime instance、BuildSet、module owner、provider generation provenance。 |
| MGR-P1-008 | Open | 只有 runtime-wide global scope；加入 engine/project/session/world/player/view/device scope。 |
| MGR-P1-009 | Open | generic name+T 可任意配对；typed accessor 必须由 compiled contract catalog 生成。 |
| MGR-P1-010 | Open | `ManagerServiceResolver` 泛型方法不可 object-safe，也不表达 availability/access policy。 |
| MGR-P1-011 | Open | Default manager 与 trait wrapper 重复 factory、依赖和 lifecycle 节点；实现对象仅为单一 public contract 的私有 provider。 |
| MGR-P1-012 | Open | 无唯一 provider、多实现集合、priority/query/selection；Net feature 还会新建独立 manager。 |
| MGR-P1-013 | Open | required/optional/disabled/blocked/degraded 无统一 typed availability。 |
| MGR-P1-014 | Open | provider artifact、ABI/schema、signature、source provenance 不进入 identity。 |
| MGR-P1-015 | Open | canonical name 散落在模块常量和 accessor 宏中，未由 composition 生成。 |
| MGR-P1-016 | Open | builtin Pascal 与 plugin 小写 namespace 并存，稳定/显示/日志/持久名称未分域。 |
| MGR-P1-017 | Open | feature gate 会删除 accessor/contract 编译表面，未证明 powerset schema 稳定。 |
| MGR-P1-018 | Open | 没有 immutable compiled manager directory，运行期仍依赖可变字符串 registry。 |

### 7.2 Call lease、reload 与 phase

| ID | 状态 | 当前差距与硬切目标 |
|---|---|---|
| MGR-P1-019 | Open | `ManagerServiceHandle::resolve` 绕过 `ServiceHandle::enter()`。 |
| MGR-P1-020 | Open | 裸 `Arc<T>` 可跨 generation 失效与 deactivate 长期存活。 |
| MGR-P1-021 | Open | manager 方法调用不计入 registered service in-flight drain。 |
| MGR-P1-022 | Open | native trait object 可在 library unload 后保留 vtable/code 引用。 |
| MGR-P1-023 | Open | public raw `CoreHandle::resolve_manager<T>` 同样返回无 guard 的裸 Arc。 |
| MGR-P1-024 | Open | production `ServiceHandle<T>`/`ServiceCallGuard<T>` manager caller 为 0。 |
| MGR-P1-025 | Open | generation 只保护 resolve 瞬间，不保护完整方法调用区间。 |
| MGR-P1-026 | Open | 多次调用/跨 await operation 无 phase lease、deadline 和一致 generation snapshot。 |
| MGR-P1-027 | Open | API 不区分短调用 borrow、长 operation lease 与 subscription。 |
| MGR-P1-028 | Open | 无 reload/rebind policy；旧 handle 应失败、重取或跟随 provider 未定义。 |
| MGR-P1-029 | Open | 无 provider availability/generation change subscription。 |
| MGR-P1-030 | Open | 多 manager 依赖不能按同一 composition/scope generation 原子解析。 |
| MGR-P1-031 | Open | handle 捕获不证明 contract/provider/factory readiness。 |
| MGR-P1-032 | Open | lazy factory 可由任意 resolve caller 线程触发，无初始化 executor/phase。 |
| MGR-P1-033 | Open | descriptor 无 main/render/task affinity 与 allowed phase。 |
| MGR-P1-034 | Open | manager call 无 deadline、cancel token、readiness fence 或 completion receipt。 |
| MGR-P1-035 | Open | error 无 typed availability reason、retryability、remediation、last-good。 |

### 7.3 Hot path、observability 与 product integration

| ID | 状态 | 当前差距与硬切目标 |
|---|---|---|
| MGR-P1-036 | Open | handle index 不直接寻址；RegistryName HashMap 仍是 authority。 |
| MGR-P1-037 | Open | steady-state resolve 获取全局 `Mutex<HashMap>`，无关 manager 互相竞争。 |
| MGR-P1-038 | Open | offsets 虽缓存，完整 RegistryName 仍参与 hash/compare/error clone；无 compact slot key。 |
| MGR-P1-039 | Open | wrapper 增加 Any downcast 和 Arc 层，却不增加 contract 语义。 |
| MGR-P1-040 | Open | Graphics、render loop、dynamic session、viewport/diagnostics 反复 resolve 同一 manager。 |
| MGR-P1-041 | Open | 无 phase-local generation-bound batch borrow 合并同帧重复访问。 |
| MGR-P1-042 | Open | process-global Atomic index 单调递增且不复用；隔离/耗尽由 Runtime24/PERF-MVP-628 关闭。 |
| MGR-P1-043 | Open | registry wait 共用状态锁/condvar，独立 service 可产生无关唤醒。 |
| MGR-P1-044 | Partial | 有两个 aggregate resolve profiling scope；仍无 hash、lock wait、factory wait、downcast、Arc clone、stale reject 和 call-lease 分解。 |
| MGR-P1-045 | Open | 无 manager 数量/线程/reload/tail-latency scale benchmark；active-ledger ignored benchmark 不覆盖此路径。 |
| MGR-P1-046 | Partial | diagnostics 现在显式投影 unavailable/error；仍把 CoreError 字符串化，无法区分 Disabled/Blocked/Failed/Retryable。 |
| MGR-P1-047 | Open | dynamic/session shutdown 忽略部分 resolve/call 错误后推进状态，无 terminal receipt。 |
| MGR-P1-048 | Open | Graphics/RenderFramework/Editor 等把 `CoreError` 字符串化，丢失 typed root cause。 |
| MGR-P1-049 | Open | render/drop cleanup 仍可能吞 backend failure，无法证明资源在 provider unload 前释放。 |
| MGR-P1-050 | Partial | viewport handle-acquisition job 有 cancellation 且不直接调用 manager；仍无 scope/deadline/thread assertion/call lease，实际 resolve 返回裸 Arc。 |
| MGR-P1-051 | Open | 无 typed discovery/list/status catalog 供 Editor/Hub/diagnostics 展示 provider truth。 |
| MGR-P1-052 | Open | resolve 无 principal、capability、sandbox、least-privilege 或 audit policy。 |
| MGR-P1-053 | Open | scheduler 不知道 manager read/write/exclusive access，无法做 conflict planning。 |
| MGR-P1-054 | Open | `core::manager` 测试仍主要验证名称、源码形状和 weak Core lifetime。 |
| MGR-P1-055 | Partial | core runtime 已测 guard/admission、per-runtime identity、deactivate/reactivate stale；仍缺 manager wrong-T、public mutation、cross-Core、resolved-Arc unload、near-wrap 与 native lease 测试。 |
| MGR-P1-056 | Open | 无 real native unload、feature powerset、多 scope/world/session、soak 和产品热路径性能资格。 |

## 8. P2 差距（14 项）

| ID | 状态 | 差距与清理目标 |
|---|---|---|
| MGR-P2-001 | Open | `ManagerResolver` 实际是 typed service locator，名称不表达 scope/lease/availability。 |
| MGR-P2-002 | Open | resolver trait、free function 与 Core method 重复，没有稳定扩展点。 |
| MGR-P2-003 | Open | accessor 命名如 `resource_handle`、`input_actions_handle` 与 `*_manager_handle` 不一致。 |
| MGR-P2-004 | Open | `PlatformManager` canonical name 实际映射 `PreferenceStorage`，领域名与 contract 不符。 |
| MGR-P2-005 | Open | builtin/plugin 名称大小写和分隔风格不一致，诊断不可稳定扫读。 |
| MGR-P2-006 | Open | accessor 由宏手工枚举，新增 manager 易漏 catalog/test/feature combination。 |
| MGR-P2-007 | Open | `PhantomData<fn() -> T>` 的 variance/send/sync 意图未写入 contract。 |
| MGR-P2-008 | Open | handle Debug 无 contract/provider/scope/runtime identity。 |
| MGR-P2-009 | Open | CoreWeak 升级失败映射 `ServiceUnavailable("CoreRuntime")`，混淆 runtime unavailable。 |
| MGR-P2-010 | Open | resolve 消费 handle 并促使调用方 clone，掩盖短借用和长期持有。 |
| MGR-P2-011 | Open | canonical name 是 `&str` 常量，不是 build-time validated/interned/generated key。 |
| MGR-P2-012 | Open | display error 无 contract/provider/scope/retry metadata。 |
| MGR-P2-013 | Open | `include_str!` 源码形状测试可被注释伪绿，也会被无行为重排击碎。 |
| MGR-P2-014 | Open | stale 测试只覆盖 generation+1；无 near-max/wrap/exhaustion/slot reuse fixture。 |

## 9. 目标架构

### 9.1 单一 compiled service directory

```text
CompositionPlan / BuildSet
  -> ServiceContractDescriptor
       ContractId + version
       scope + owner + provider artifact/ABI
       startup + dependency + affinity/access
       availability + reload/unload policy
  -> validated immutable ServiceDirectory
       dense ServiceSlot[]
       generation + admission + in_flight
       state + provider code lease + typed projection

ManagerHandle<T>
  private { runtime_id, scope_id, slot, generation, contract_id }
  -> enter(CallContext)
  -> ManagerCallGuard<'a, T>
  -> method call / operation lease
  -> guard drop / in_flight-- / provider lease release
```

Manager 不再拥有第二套 registry；它只是统一 service descriptor 的 typed/scoped 投影。concrete implementation 只能作为 provider factory 私有对象，一个 public contract 只发布一个 service slot。旧 raw resolve、wrapper service、兼容 re-export、双写 registry 和长期裸 Arc 必须硬删除，不能保留 shim。

### 9.2 Scope、access 与 replacement

- engine/runtime、project/session、world、local player、view/render device 使用不同 `ScopeId` 和 owner teardown graph；
- accessor 从 compiled catalog 生成，contract id 与 slot 在 activation transaction 冻结；
- scheduler/CallContext 声明 read/write/exclusive、affinity、phase、deadline、principal/capability；
- required/optional 返回 typed Ready/Disabled/Blocked/Degraded/Failed disposition；
- provider replacement 发布新 generation，旧 generation 只 drain，不自动跳转；multi-manager batch 绑定同一 composition/scope generation。

### 9.3 性能合同

- steady-state 只做 runtime/scope/contract/generation 检查、dense slot 读取和轻量 call lease；
- frame/tick/render phase 可创建不可逃逸的 batch borrow，消除重复 hash/lock/downcast/Arc clone；
- metadata、discovery、错误字符串走冷路径，per-slot admission/waiter 避免全局无关唤醒；
- benchmark 分别报告 1/64/1K/64K services、1/8/64 threads 的 median/p95/p99、contention、reload、memory 与失败路径。

## 10. 依赖顺序重构里程碑

### M0 · Contract/caller/deletion matrix 与 RED 证据

- 冻结全部 raw/wrapped manager、provider、scope、feature gate、product caller 和旧 API 删除清单；
- 增加 manager wrong-T/cross-Core/public mutation、resolved Arc unload、generation wrap、native vtable unload RED repro；
- 记录现有 hash/lock/downcast/Arc clone、1/64/1K/64K manager 与 contention 基线；
- 锁定 Runtime01/02/24/42/46、Interface01、Plugins01、App01 和 PERF-MVP-628 owner 边界。

### M1 · Unified descriptor 与 composition validation

- 扩展统一 service descriptor：contract/version/scope/owner/provider artifact/ABI/affinity/access/availability/unload；
- composition 阶段验证 duplicate/missing/wrong-version/scope-invalid provider 和 feature powerset；
- Net feature、Editor、Script、Particles、Texture、UI 与 builtins 全部进入同一 catalog，禁止依赖边与实际 provider 脱节。

### M2 · Dense scoped directory 与不可伪造 handle

- 建立 runtime-owned immutable dense slot directory 和 scope directory；
- handle 字段私有化并携带 runtime/scope/contract/generation；定义 exhaustion/reuse fail-close；
- steady-state 不再 hash RegistryName 或持有 global service-map mutex。

### M3 · Guarded typed access 硬切

- 所有 manager method/operation 通过 call guard 或 operation lease，完整区间计入 in-flight；
- provider code lease 与 call guard 同生共死，先 close admission/drain，再析构 trait object，最后卸载 DLL；
- 删除 `CoreHandle::resolve_manager<T> -> Arc<T>`、`RegisteredManagerService<T>` 双注册和所有 compat facade。

### M4 · Product caller、diagnostics 与 shutdown 收敛

- 迁移 Graphics、dynamic runtime/session、Editor viewport/asset access、Runtime diagnostics 和首方插件；
- 加入 phase-local batch borrow、typed availability/status catalog、structured error 和 terminal shutdown receipt；
- Editor job 明确 scope/generation/affinity/deadline/cancel，dynamic path 绑定 session/world/view scope。

### M5 · Native、powerset、scale 与性能资格

- 真实动态库 hot load/unload/fault recovery，multi-world/session/reload soak；
- feature powerset 与 provider replacement 资格；
- 同硬件、同场景、同语义、同失败条件对照 benchmark；正确性门未过前不得以更快的错误语义宣称领先 Unreal。

## 11. 验收门（36 项）

| Gate | 状态 | 验收内容与当前缺口 |
|---|---|---|
| G01 | Fail | 单一 compiled descriptor、无 raw/wrapped 双注册；当前双轨。 |
| G02 | Fail | descriptor 有 stable contract/version/scope/owner/provider/ABI/BuildSet；当前缺失。 |
| G03 | Fail | duplicate/missing/wrong-version/scope-invalid 在 activation 前 fail-close；当前只校验名称/依赖/kind 子集。 |
| G04 | Fail | feature powerset 保持 contract identity，禁用项 typed Disabled；当前 accessor 会被 cfg 删除。 |
| G05 | Fail | handle 私有且不可跨 runtime/scope/contract/generation 伪造；当前字段 public。 |
| G06 | Partial | module deactivate/reactivate stale 已测；slot reuse、world/session scope、near-wrap 未测。 |
| G07 | Fail | 每次 production manager call 都持 guard；当前 caller 为 0。 |
| G08 | Partial | kernel 支持 close admission/drain deadline；manager 裸 Arc 绕过。 |
| G09 | Fail | trait object 析构和 DLL unload 晚于全部 guard；无 code lease。 |
| G10 | Partial | RAII guard drop 底座存在；panic/cancel/timeout/provider lease 资格缺失。 |
| G11 | Fail | lazy factory 只在声明 executor/phase；当前任意 resolve 线程可触发。 |
| G12 | Fail | affinity/access conflict/exclusive 可检测；descriptor 无声明。 |
| G13 | Fail | required/optional/degraded/blocked 有 typed reason/retry/remediation；当前无。 |
| G14 | Fail | provider replacement 发布新 generation、旧调用仅 drain；当前无 replacement contract。 |
| G15 | Fail | multi-manager batch 绑定同一 scope/composition generation；当前无。 |
| G16 | Fail | engine/project/session/world/player/view/device scope 有 owner 和逆序 teardown；当前无。 |
| G17 | Fail | steady-state 不 hash RegistryName、不锁 global HashMap；当前仍发生。 |
| G18 | Fail | handle slot O(1) 直达且 exhaustion/reuse 可证明；当前 index 只做后验校验。 |
| G19 | Fail | phase borrow 不可逃逸为长期裸 Arc；当前返回 Arc。 |
| G20 | Fail | discovery/status catalog 报 contract/provider/scope/generation/availability；当前无。 |
| G21 | Fail | principal/capability 在返回 guard 前执行并留 audit；当前无。 |
| G22 | Partial | Core 内错误为 enum；Graphics/Editor/dynamic 边界仍字符串化。 |
| G23 | Partial | diagnostics 显示 unavailable/error；无 typed Disabled/Blocked/Failed 与 provider identity。 |
| G24 | Fail | shutdown 每个 manager step 有 terminal receipt；dynamic cleanup 仍 best-effort 吞错。 |
| G25 | Fail | Graphics 同 phase 重复 asset access 合并且保留 generation；当前 32 处调用。 |
| G26 | Fail | dynamic frame/event/host request 在正确 scope/thread phase 解析；当前无 scope/affinity。 |
| G27 | Partial | viewport acquisition 有 handle generation/cancel；无 scope/deadline/thread/call lease。 |
| G28 | Fail | AI/Net/Physics/Sound/Animation/Scene 只发布一个 public contract；当前双注册。 |
| G29 | Partial | per-runtime identity/stale generation 有测试；wrong-T/public mutation/cross-Core/near-max 缺失。 |
| G30 | Partial | kernel guard/admission race 有测试；manager resolved Arc 仍可越过 unload。 |
| G31 | Fail | real native hot unload/recovery；未验证。 |
| G32 | Fail | 1/64/1K/64K manager、1/8/64 threads 报 median/p95/p99/contention；无。 |
| G33 | Partial | 有两个 aggregate resolve scope；无 call lease/factory wait/stale/drain 分解。 |
| G34 | Fail | powerset/multi-world/multi-session/reload soak 无泄漏/死锁/旧代访问；未验证。 |
| G35 | Fail | 同硬件正确性/失败/CPU/memory/tail latency 比较；未验证。 |
| G36 | Partial | 本报告可完成 frontmatter/link/count/fingerprint/diff 静态检查；production coverage 与动态资格仍未形成 accepted evidence。 |

## 12. 状态与限制

| 项目 | 状态 | 证据 / 限制 |
|---|---|---|
| current-source 静态审查 | review_complete | 178 个 focused Rust 文件；48,184 行；1,777,458 bytes；567 个 test attribute |
| production reachability | review_complete | 26 个注册文件、33 个非注册 caller；Graphics 32 处 `.asset_manager()`；production call guard caller 0 |
| 参考引擎对照 | review_complete | Unreal 6、Bevy 2、Godot 2、Fyrox 2、Unity Graphics 3 个源码/测试文件 |
| P0/P1/P2 | review_complete | 本地 0；P1 52 Open + 4 Partial；P2 14 Open |
| Gate | review_complete | 26 Fail + 10 Partial；Partial 不等于 accepted |
| Production 重构 | pending | 本篇未修改源码、测试、Cargo、ABI 或产品行为 |
| 动态/性能验证 | not_run | 未运行 Cargo、DLL unload、Editor、WGPU、powerset、multi-scope、soak、benchmark；不能形成性能领先声明 |
