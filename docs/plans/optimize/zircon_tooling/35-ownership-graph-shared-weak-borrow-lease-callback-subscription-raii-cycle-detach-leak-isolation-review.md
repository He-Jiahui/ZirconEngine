---
related_code:
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_editor/src/core/context/builder.rs
  - zircon_editor/src/core/editor_event/listener/registry.rs
  - zircon_editor/src/core/editor_event/listener/route.rs
  - zircon_editor/src/core/editor_message/bus.rs
  - zircon_editor/src/core/editor_message/shared.rs
  - zircon_editor/src/core/plugin/lifecycle_message_bridge.rs
  - zircon_editor/src/core/settings/authority.rs
  - zircon_editor/src/ui/host/editor_asset_manager/change_stream.rs
  - zircon_editor/src/ui/host/editor_host_event_controller.rs
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring.rs
  - zircon_plugins/net/features/rpc/runtime/src/manager/registry.rs
  - zircon_plugins/net/features/rpc/runtime/src/manager/state.rs
  - zircon_plugins/net/runtime/src/service_types/http_routes.rs
  - zircon_plugins/net/runtime/src/service_types/listeners.rs
  - zircon_plugins/net/runtime/src/worker/net_worker.rs
  - zircon_plugins/sound/runtime/src/dynamic_event_abi/callback.rs
  - zircon_plugins/sound/runtime/src/service_types/dynamic_event_executors/registration.rs
  - zircon_plugins/sound/runtime/src/service_types/dynamic_event_executors/unregistration.rs
  - zircon_runtime/src/core/framework/state/hook_index.rs
  - zircon_runtime/src/core/resource/lease.rs
  - zircon_runtime/src/core/resource/manager/lease_ops.rs
  - zircon_runtime/src/core/resource/manager/resource_manager.rs
  - zircon_runtime/src/core/runtime/descriptors/service_factory.rs
  - zircon_runtime/src/core/runtime/events/subscriber.rs
  - zircon_runtime/src/core/runtime/events/topic.rs
  - zircon_runtime/src/core/runtime/handle/states.rs
  - zircon_runtime/src/plugin/bridge/weak.rs
  - zircon_runtime/src/plugin/extension_registry/runtime_extension_registry.rs
  - zircon_runtime/src/plugin/extension_registry/runtime_extension_registry/owner_revocation.rs
  - zircon_runtime/src/plugin/native_plugin_loader/loaded_native_plugin.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_host_handle.rs
  - zircon_runtime/src/scene/ecs/events/store.rs
  - zircon_runtime/src/scene/ecs/observer/store.rs
  - zircon_runtime/src/scene/event_mirror/subscription.rs
  - zircon_runtime/src/scene/world/event_mirror.rs
  - zircon_runtime/src/scene/world/observers.rs
  - zircon_runtime/src/script/vm/gc_bridge/vm_object_ref.rs
  - zircon_runtime/src/script/vm/runtime/vm_plugin_manager.rs
  - zircon_runtime/src/ui/binding/router.rs
  - zircon_runtime/src/ui/dispatch/navigation/dispatcher.rs
  - zircon_runtime/src/ui/dispatch/pointer/dispatcher.rs
  - zircon_runtime/src/ui/event_ui/manager/registration.rs
  - zircon_runtime/src/ui/event_ui/manager/subscription.rs
  - zircon_runtime/src/ui/event_ui/manager/ui_event_manager.rs
tests:
  - zircon_editor/src/tests/editor_message/refresh.rs
  - zircon_runtime/src/plugin/native_plugin_loader/behavior_validation/tests.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests.rs
plan_sources:
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/coverage.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_runtime/01-core-runtime-lifecycle-registry-review.md
  - docs/plans/optimize/zircon_runtime/02-core-runtime-events-tasks-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/07-script-plugin-runtime-review.md
  - docs/plans/optimize/zircon_runtime/08e-network-runtime-review.md
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime_interface/01-runtime-dll-abi-ffi-version-handle-foreign-ownership-review.md
  - docs/plans/optimize/zircon_runtime_interface/05-runtime-host-foreign-output-safe-api-ownership-admission-budget-fuse-observability-review.md
  - docs/plans/optimize/zircon_tooling/21-unsafe-rust-ffi-native-memory-thread-affinity-panic-unload-safety-governance-review.md
  - docs/plans/optimize/zircon_tooling/23-failure-contract-panic-unwind-error-propagation-poison-recovery-result-observability-review.md
  - docs/plans/optimize/zircon_tooling/24-concurrency-locking-atomic-ordering-blocking-thread-lifecycle-backpressure-deadlock-review.md
  - docs/plans/optimize/zircon_tooling/25-memory-allocation-domain-budget-oom-pressure-fragmentation-pooling-cache-residency-observability-review.md
  - docs/plans/optimize/zircon_tooling/33-reference-engine-source-corpus-snapshot-provenance-citation-applicability-comparison-currentness-review.md
  - docs/plans/optimize/zircon_tooling/34-global-state-scope-singleton-service-locator-static-registry-cache-initialization-reset-multi-instance-isolation-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Delegates/MulticastDelegateBase.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Delegates/DelegateSignatureImpl.inl
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/UObject/WeakObjectPtr.h
  - dev/bevy/crates/bevy_ecs/src/observer/mod.rs
  - dev/Fyrox/fyrox-resource/src/event.rs
  - dev/Fyrox/fyrox-core/src/pool/handle.rs
  - dev/godot/core/object/object.cpp
  - dev/godot/core/object/object.h
  - dev/godot/core/object/ref_counted.h
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Textures/RTHandleSystem.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 35 · Ownership Graph、Shared/Weak/Borrow/Lease、Callback/Subscription、RAII、Cycle、Detach 与 Leak Isolation 审查

## 1. 结论

Zircon 并非只有临时性的 `Arc` 堆叠。Core EventBus 的 subscription 只弱持有 bus state，`Drop` 会注销或在 bus 已销毁时主动断开并清空；订阅建立期间还有 `PendingSubscription` reservation。Editor asset change stream 让 hub 只保存 mailbox 的 `Weak`，发布时自动剔除死亡项。保留式 Editor host 的回调 wiring 普遍捕获 `Rc::Weak`。原生插件每一加载代有稳定 library owner，执行 callback 前取得 activity lease，卸载 transition 拒绝仍有在途 callback 的代。`WeakBridge` 按 generation 刷新 weak provider，VM manager 的 `Arc::new_cyclic` 只保存 self weak，`VmObjectRef` 又以最后一个 root lease 的 `Drop` 注销 GC root。这些不是应该推倒的实现，而是全仓所有权合同的可复用基线。

问题在于这些规则没有成为统一准入条件。同一仓库里，`UiEventManager::subscribe` 返回裸 ID 和 unbounded receiver，receiver 被丢弃后 sender 永久留在 map，广播持续向死亡端发送且忽略失败；公开 state hook API 只把 `Arc<dyn Fn>` 追加进 Vec，没有 token、owner、unregister 或 revoke；World observer 和 EventStore observer 依赖调用者保存裸句柄并手动删除；UI pointer/navigation dispatcher 与通用 UI router 甚至没有删除 API。Editor message bus 本身有有界 inbox 和显式 unregister，但大量 caller 只保存 ID，插件 lifecycle bridge 既不保存 bus 也没有 `Drop`，形成同一产品内互相矛盾的生命周期合同。Net HTTP route 可注销、worker 可在 `Drop` shutdown；RPC handler/schema validator 却只有注册，没有 owner 或 revoke。

更严重的结构性风险是 lease 反向强持有整个 authority。`ResourceLease` 的 release closure 捕获完整 `ResourceManager` clone，而 manager 又拥有任意 type-erased resource payload。代码允许形成 `manager -> payload -> lease -> release closure -> manager` 的强环；静态审查没有证明当前 shipping payload 已构造此环，但类型合同明确允许它，且没有 cycle conformance test。一个逃逸 lease 还会固定 registry、event publisher 与 commit gate，而它实际只需要一个窄 release authority。此问题不能靠“Rust 没有 use-after-free”来否认：内存安全不等于 owner 可终止、插件可卸载、world 可回收或 project 可隔离。

本篇登记 **0 项 P0、48 项 P1、12 项 P2 和 40 个验收门**。没有新增 P0，因为当前证据能证明 API 允许无界保留、旧 callback 存活和强环，但未独立证明 shipping BuildSet 已发生不可恢复泄漏、错误 DLL 调用或跨 owner 数据破坏。Tooling21 继续拥有 unsafe/FFI memory soundness，Tooling24 拥有线程、lock、channel/backpressure，Tooling25 拥有 resident bytes/pressure，Runtime24 拥有 handle identity/exhaustion，各 domain 报告拥有具体事件、UI、network、plugin、scene 行为。本篇只拥有：

`OwnershipEdgeInventory -> Owner/Owned/Borrowed/Observed -> Strong/Weak/LeasePolicy -> SubscriptionToken -> CallbackCapturePolicy -> CycleAnalysis -> Close/Drop/Unload -> LeakCensus -> OwnershipReceipt -> Qualification`

## 2. 审查边界、口径与限制

### 2.1 当前物理账本

| Evidence | 本轮结果 | 解释 |
|---|---:|---|
| production-like Rust physical candidate | 11,877 files / 1,199,398 physical lines / 45,771,850 bytes | 覆盖 App、Editor、Hub、Plugins、Runtime、Runtime Host、Runtime Interface；排除常见 tests/benches/fixtures/native fixture，不等于 Cargo-resolved BuildSet |
| strong shared-owner lexical signal | 2,819 matches / 823 files | `Arc<T>`/`Rc<T>` 只表示候选，immutable sharing、snapshot 和 library pin 可完全合理 |
| weak-owner lexical signal | 212 matches / 64 files | `Weak<T>`、downgrade、upgrade 的保守下界；不能证明 upgrade 后调用与 generation 安全 |
| callback-owner lexical signal | 113 matches / 52 files | `dyn Fn` 与 Callback/Handler/Hook alias；宏展开和 C ABI function pointer 不在此数内 |
| subscription/observer API signal | 108 matches / 55 files | subscribe/unsubscribe/listener/observer 词法下界，包含正例与手动合同 |
| `Drop` implementation | 141 matches / 123 files | `Drop` 存在不等于完成 quiescence，也不等于 cleanup 结果可观测 |
| detach/forget/ManuallyDrop signal | 19 matches / 14 files | 大部分是 ABI ownership transfer、erased move 或明确 detach；逐项路由而非机械禁止 |
| source evidence currentness | 59 个唯一 related/test/reference 输入、886,906 bytes；SHA-256 `61c567645f559d99f9c2e0d5ba33e2d12c7237d70fdfdfaf05f658c6e54b0adb` | HEAD `25e09a23178000f2e783ce2143cf70a8b118d404`；plan/index 不进入自引用指纹 |

这些数字只用于固定审查面。一个 `Arc` 可能是正确的 immutable snapshot，一个没有 `Arc` 的裸 C function pointer 反而可能越过 DLL generation；风险排序必须读取 owner、termination 与 caller，而不是按关键词计数。

### 2.2 OwnershipEdge 必须描述方向与终止

后续 inventory 的每条边至少包含：`EdgeId`、source owner、target owner、strength、purpose、creation site、revocation trigger、in-flight policy、generation、thread domain、close/drop behavior、cycle class、census metric 和 evidence owner。建议的边类型如下：

| Edge kind | 允许语义 | 禁止的隐含行为 |
|---|---|---|
| `OwnedStrong` | owner 决定 target 生命周期，reverse teardown 可达 | child 反向强持有 owner 而无 cycle break |
| `SharedImmutable` | content/build-bound immutable artifact共享 | 用 immutable 名义隐藏 project/world mutable state |
| `BorrowedScoped` | 编译期或 guard 证明调用期有效 | 裸指针/引用逃逸到异步、callback 或 DLL 外 |
| `ObservedWeak` | target 不因 observer 存活，upgrade 失败是正常状态 | upgrade 后跳过 generation/closing 检查 |
| `LeaseStrong` | 在途操作显式固定 target generation | lease 固定整个 service graph而非窄 authority |
| `Subscription` | publisher 持路由，subscriber token决定撤销 | 丢 receiver/owner 后仍永久保留 sender/callback |
| `SnapshotInFlight` | revoke 后已取得快照可按声明完成或取消 | 未声明地向 detached inbox/旧 DLL继续投递 |
| `DetachedTerminal` | supervisor 记录 forced detach且产品资格失败 | 把失去 join/close authority 当作成功清理 |

### 2.3 Evidence 边界

1. 本轮读取所有权声明、关键 creator/caller、`Drop`/close/revoke 路径与代表性测试，未运行 heap profiler、DLL reload、双 Project、PIE、observer churn 或 leak sanitizer。
2. 当前工作树有其他会话的大量 Editor 源码改动；本篇记录审查时物理内容和 HEAD，但不把 dirty 内容误称为提交基线，后续实施必须 source recheck。
3. 已知 Editor、Hub、WOC 与 plugin metadata 动态验证阻断未改变，本篇不重复运行；报告只修改文档。
4. `Arc` strong count、registry length 或进程退出后的 OS 回收不能单独证明泄漏；必须以 owner terminal 后的 typed census、heap/GPU/foreign allocation和线程/callback归零共同证明。
5. 本篇不要求所有 callback 都变 weak。执行中必须 pin DLL、资源或 provider 的边应保留强 lease；要求的是边被声明、可撤销、代际化且能终止。

## 3. 必须保留的工程基础

### 3.1 Core EventBus 已有完整 subscription owner

`EventSubscription` 强持有 topic/subscriber但只弱持有 `EventBusState`；`Drop` 成功 upgrade 时从 bus 删除，失败时仍 deactivate 并 drain subscriber。topic 的 `PendingSubscription` 用 RAII 保护创建窗口，bus state teardown又会断开所有 subscriber。它应成为普通进程内事件订阅的默认语义。

### 3.2 Editor asset change stream 自动清理死亡 receiver

subscription 强持有 mailbox，hub 只保存 `Weak<Mutex<Mailbox>>`；subscribe 与 publish 都会 prune 无法 upgrade 的项。clone subscription 的语义是最后一个 clone drop 后自然失效。需要补诊断与显式 close，但所有权方向正确。

### 3.3 RetainedEditorHost 系统性使用弱回调

host 建立 self weak，callback wiring 和 native presenter callback 捕获 `Rc::Weak<RefCell<RetainedEditorHost>>`，避免 widget/callback table 反向固定整棵 host。Host `Drop` 还启动 autosave shutdown并撤销 hierarchy watch。应把这套 capture helper 上升为 UI 组件规范。

### 3.4 Runtime extension 已有 owner-scoped revoke

`RuntimeExtensionRegistry` 的 typed extension points记录 `PluginModuleId`，owner revoke 时先 deactivate bridge/unbind import、通知 revocation listener，再删除 owner 的 executable contribution并重建 bridge。listener 自己也在其 owner 被撤销时删除。这是插件动态生命周期的正确顺序基础。

### 3.5 Native callback lease 固定加载代

每个 `LoadedNativePlugin` generation 持有 `Arc<NativePluginStableLibrary>`。callback admission 用原子 activity count，lifecycle transition 只能从零 callback进入，callback lease 的 `Drop` 递减并固定 `Library`。该协议比裸函数指针可靠，必须覆盖所有绕行 ABI callback。

### 3.6 WeakBridge 和 VM 根租约已有代际意识

`WeakBridge` cache包含 provider generation和 `Weak<T>`，`pin()`显式产生短期 `BridgeGuard`。`VmPluginManager` 的 cyclic construction只保存 weak self；`VmObjectRef` 最后一个 clone释放时注销 GC root，并让 registry 活到最后引用结束。这些正例说明 weak observation与强 operation lease可以并存。

### 3.7 Runtime event mirror 已处理跨 owner Drop

subscription handle包含 slot/generation；subscription `Drop` 不直接借用 World，而是向 owner-side reclaim queue提交意图，World 后续 disconnect、更新 reader count并可重试，World `Drop` 还回收所有 live record。该模式适合无法在 token `Drop` 时同步取得 owner mutability 的场景。

### 3.8 Network worker、HTTP route 和 sound executor 有显式撤销

Net worker shutdown有 reply、join与 `Drop` fallback；HTTP route能 unregister，listener能 close。Sound dynamic executor也有 unregister，并在 handler删除时同步移除 executor。这些局部合同应保留；缺口是与 plugin owner/generation的统一绑定，而不是删除显式 close。

## 4. 已确认的结构断点

### 4.1 UiEventManager 保留死亡 sender

`subscribe()` 创建 unbounded channel，把 sender 强存进 `subscriptions` 并返回裸 ID/receiver。receiver drop不会触发 unregister，`broadcast(&self)`又不能 prune，发送失败被忽略。结果是死亡条目和每次无效发送持续到 manager drop或调用者恰好手动 unsubscribe；unbounded queue预算归 Tooling24/25，本篇拥有 token/死亡 owner清理。

### 4.2 State hook 是只增不减的公开 callback 图

`CoreRuntime/CoreHandle::register_on_enter/exit/transition` 返回 `()`，`StateHookIndex` 只把 `Arc<dyn Fn>` push进嵌套 Vec/HashMap。没有 unregister、owner、generation或 revoke；动态 module注册的 closure可活到整个 runtime结束，并能强捕获 service/CoreHandle形成环。当前 production caller稀少不构成豁免，公开 API 已允许这一状态。

### 4.3 Scene observer 依赖裸 ID 与调用者记忆

World lifecycle/event observer返回 `ObserverId`，只有 `remove_observer`。entity-specific observer会随 entity despawn清除，这是正向边；global lifecycle/event observer没有 owner entity或 RAII token。EventStore observer同样返回 handle并要求 `unobserve`。Runtime event mirror做了包装，但普通直接 caller仍可忘记撤销。

### 4.4 UI handler table 没有 node/route 终止合同

pointer/navigation dispatcher按 `(UiNodeId, kind[, phase])`保存 callback Vec，通用 `UiEventRouter`按 path保存 `Box<dyn Fn>`，均只有 register。node从 tree移除、document替换、plugin卸载或 route重编译时没有 remove owner API。`UiEventManager::register_route`另有 routes_by_binding覆盖旧 ID但旧 routes_by_id仍可保留的专项缺口；具体 UI 数据流继续由 Runtime11A拥有。

### 4.5 Editor message/事件 listener 的合同不一致

message bus有 checked ID、bounded inbox和手动 unregister；`EditorHostEventController::Drop`正确注销。但 `EditorPluginLifecycleMessageBridge`只存 subscriber ID，不保存 bus或 lease，不能自行 close。Editor event listener unregister后，已取得的 `Arc` route snapshot和 detached handle仍可继续向旧 inbox排队，这是测试明确允许的 in-flight语义，却没有 closing epoch、quiescence receipt或“最多保留多久”的合同。

### 4.6 ResourceLease 可反向固定完整 ResourceManager

lease 的 release closure捕获 `ResourceManager` clone，而 manager authority持有任意 `Arc<dyn ResourceData>`。payload若保存指向自身或同 manager资源的 lease，就可形成强环，最后 lease永远不 drop，payload也永远不卸载。release实际只需要 authority weak/lease table，不需要 commit gate和event publisher。必须先收窄边，再写构造性 cycle test。

### 4.7 RPC callback 缺 plugin owner/revoke

`NetRpcRuntimeManager`由 Core service factory按 runtime实例创建，这是正向作用域；但 schema validator和 RPC handler只是按字符串插入 HashMap，没有 token、unregister、owner module或 generation。动态脚本/插件注册后重载，旧 closure可能被服务继续调用。HTTP route具备 unregister，说明此差异不是技术限制。

### 4.8 ABI callback 仍有绕过 generation lease 的路径

Sound dynamic event ABI把 `ZrPluginEventCallbackFnV1`直接包装进 Rust closure；executor unregister存在，但 callback值本身不携带 `NativePluginLibraryGenerationOwner`。若注册来源是可卸载 DLL，必须证明调用都经过 native callback lease，或把 generation owner纳入 registration。具体 unsafe可调用性由 Tooling21/Plugins01/Interface01修复，本篇要求所有权边闭合。

### 4.9 单 subscriber 和 owner listener 缺局部撤销

`SettingsAuthority`只能 replace一个强 `Arc<dyn SettingsChangeSubscriber>`，没有 clear/token；当前 i18n subscriber未发现反向环，但复用 authority或局部 host重建时缺少显式终止。owner revocation listener会随 owner revoke删除，是正例；但不能在 owner仍存活时单独撤销，captured state会至少保留到 owner结束。

### 4.10 Drop 不能替代可观测 close

许多 owner在 `Drop` 中忽略 cleanup错误，这是 Rust 析构的合理限制，但系统目前没有统一显式 close receipt来区分 clean、retry pending、forced detach和 owner already gone。Network worker和runtime mirror已展示可用模型；需要推广而不是要求 Drop 返回 `Result`。

## 5. 目标架构

### 5.1 OwnershipGraph

由 Cargo/feature/cfg resolved BuildSet生成静态边，再以 runtime census补充动态实例。图节点是 Process/Library/Runtime/Host/Project/World/Entity/Window/Document/Plugin/Service/Resource/Task；边不是简单的 strong/weak二分，而要声明创建、pin、revoke、in-flight和termination。

### 5.2 SubscriptionToken 与 CloseState

统一 token至少包含 owner identity、slot/generation、state weak或reclaim sink、close state和诊断 ID。`close()`提供 typed结果并可等待quiescence；`Drop`只执行有界幂等fallback。若 owner需要 `&mut`才能撤销，使用 runtime mirror式 reclaim queue而非泄漏。

### 5.3 CallbackCapturePolicy

callback registration必须机器声明 capture mode：`BorrowedCall`、`WeakObserved`、`StrongOperationLease`、`ImmutableSnapshot`或 `ForeignGenerationLease`。UI/widget默认 weak；在途 native/资源操作默认短期强 lease；禁止不经声明捕获 Core/World/Manager根。

### 5.4 CycleAnalysis 与 narrow authority

对 service/resource/plugin/UI graph运行类型级和构造性 cycle检查。release callback只持窄 `Weak<LeaseAuthority>`；payload不得拥有能反向固定其 authority的强 lease，或必须通过显式 cycle breaker/arena epoch。cycle分析失败不得靠 `Arc::strong_count`猜测修复。

### 5.5 LeakCensus 与 OwnershipReceipt

每个 terminal scenario记录 active subscription、callback、lease、observer、task、thread、resource residency、native allocation和detached owner。receipt绑定 source/build/product/scenario；等待超时、forced detach、未知 owner或非零 census只能标 partial/failed，不能自报 clean unload。

## 6. P1 重构项

### OL-P1-001 · 建立 OwnershipEdgeInventory 单一真源

用 Rust AST、Cargo feature/cfg与外部 ABI schema登记 owner graph；词法数字只作 bootstrap，不能成为最终 finding。

### OL-P1-002 · 定义所有权边 taxonomy

统一 OwnedStrong、SharedImmutable、BorrowedScoped、ObservedWeak、LeaseStrong、Subscription、SnapshotInFlight和DetachedTerminal，禁止自由文本 `shared`。

### OL-P1-003 · 为 owner、target 与 edge绑定身份和 generation

每条动态边记录 Runtime/Host/Project/World/Entity/Window/Document/Plugin/Library身份；旧 token不能命中新代 owner。

### OL-P1-004 · 绑定 Cargo BuildSet 与产品可达性

inventory区分shipping、editor、tool、test、optional feature和 native dynamic路径，不能用未启用代码稀释风险。

### OL-P1-005 · 建立 CallbackCapturePolicy manifest

每个长期 callback声明允许捕获的 owner层级与 strength；未登记 strong root capture在required gate失败。

### OL-P1-006 · 增加 registration API 结构准入

新增 `register/subscribe/observe/add_listener`若不返回 token/owner receipt或声明 startup-immutable freeze，结构审计失败。

### OL-P1-007 · 建立 DomainOwner 与跨报告路由

每条边指定行为owner、内存owner、并发owner、ABI owner和资格owner，避免五份报告各修一半。

### OL-P1-008 · 定义 OwnershipReceipt schema

记录创建、active count、revoke cause、close result、quiescence、forced path、generation与证据位置。

### OL-P1-009 · 统一 SubscriptionToken 接口

token提供 identity、`is_active`、typed `close`和幂等 Drop；不同 domain可有专用类型但不得退化为裸整数。

### OL-P1-010 · 区分显式 close 与 Drop fallback

close可返回错误/等待在途；Drop必须有限、无 panic、无阻塞失控，并把未完成清理交给 owner reclaim queue。

### OL-P1-011 · publisher registry默认弱持 subscriber owner

registry只强持必要路由或短期投递快照；业务对象生命周期不得由 publisher无意决定。

### OL-P1-012 · 广播时自动剔除死亡接收端

推广 asset stream/Fyrox 模式；send disconnected要删除或进入bounded prune queue，不能永久忽略。

### OL-P1-013 · 定义 quiescent unsubscribe

明确 revoke后新投递停止、在途 callback完成/取消、队列 drain/drop与返回时点；普通 remove和quiescent close不可混用。

### OL-P1-014 · 类型化 SnapshotInFlight 语义

route/subscriber快照必须声明 unregister后是否继续、最多保留多少、能否触达 detached inbox/旧 DLL以及如何计入 census。

### OL-P1-015 · 把队列预算链接到订阅owner

每个 subscription具有条目/bytes/age/drop/backpressure政策；具体算法引用 Tooling24/25，不在本篇复制。

### OL-P1-016 · 世代化 token slot并拒绝 stale close

token复用必须有 generation，close错误区分 already closed、wrong owner、stale generation和owner gone；exhaustion由Runtime24统一。

### OL-P1-017 · 固化 Core EventBus 正向模型

抽取 conformance tests覆盖 pending reservation、bus先drop、subscriber先drop、并发publish/unsubscribe和最终 topic prune。

### OL-P1-018 · 重构 UiEventManager subscription

返回 RAII token，使用 weak/reclaim owner，drop receiver自动注销，broadcast可剔除 disconnected sender，并接入有界队列政策。

### OL-P1-019 · 给 state hooks增加 owner与撤销

`register_on_*`返回 token；支持 module/runtime owner revoke、generation和quiescent transition snapshot，禁止只增不减 Vec。

### OL-P1-020 · 将 World observer绑定 entity/system/plugin owner

支持 observer entity或 owner scope；entity/system/plugin despawn/revoke自动清理global observer，保留现有 entity-specific正例。

### OL-P1-021 · 为 EventStore observer提供 RAII包装

普通 caller不再持裸 `EventObserverHandle`；无 `&mut World` 的 Drop走 owner reclaim queue，显式 close仍可同步失败。

### OL-P1-022 · 固化 runtime event mirror reclaim协议

把generation handle、live budget、deduplicated reclaim、retry与World Drop shutdown做成跨域参考实现并增加故障注入。

### OL-P1-023 · 为 Editor message subscriber引入 lease

lease持有 bus weak/reclaim authority与subscriber generation；context/host/plugin局部销毁时自动 unregister并清 inbox。

### OL-P1-024 · 让 plugin lifecycle bridge显式 close

bridge保存 subscription lease而非裸 ID；EditorManager shutdown先停 pump、等待在途 callback，再释放 subscriber。

### OL-P1-025 · 给 Editor event listener增加 closing epoch

区分普通 unregister与quiescent unregister；旧 route snapshot不得无限向 detached store追加，receipt报告仍在途快照。

### OL-P1-026 · 让 SettingsChangeSubscriber 可清理

配置返回 token或host-owned guard，支持 clear/replace generation；当前单 subscriber优化可保留但生命周期必须显式。

### OL-P1-027 · 将 pointer handler绑定 UiNode/Document lifecycle

注册返回 route token；node移除、tree replacement、document close和plugin revoke自动清 handler与capture状态。

### OL-P1-028 · 将 navigation handler绑定 UiNode/Surface lifecycle

与 pointer使用共同 owner schema但保留事件类型差异；focus route不得调用 retired node callback。

### OL-P1-029 · 为 UiEventRouter增加 freeze或remove二选一合同

startup immutable router必须 finalize后拒绝 mutation；动态 router必须提供 owner-scoped remove，不允许无限 append。

### OL-P1-030 · 收敛 UiEventManager route identity

binding replacement必须原子 retire旧 route，route ID/generation可诊断；具体调用语义与Runtime11A共同验收。

### OL-P1-031 · 为 RPC validator/handler增加 PluginModuleId owner

register返回token，plugin/script reload先 revoke旧 callback并等待在途 RPC；descriptor、validator和handler同一事务更新。

### OL-P1-032 · 让 sound ABI callback携带 library generation lease

注册裸函数指针时必须关联 `NativePluginLibraryGenerationOwner`或受同等调用gate保护；unregister和handler删除产生receipt。

### OL-P1-033 · 提供类型化 strong/weak capture helper

UI、service、plugin callback使用明确 helper，review可直接识别 weak upgrade、operation pin和owner-gone行为。

### OL-P1-034 · 固化 CoreWeak factory边界

保留 service/plugin factory接收 weak Core的设计；增加结构测试禁止factory产物无声明地反捕获 strong CoreHandle。

### OL-P1-035 · 实现强环静态与构造性检查

对 owner graph找 SCC，再为 type-erased payload、callback和plugin bridge构造最小环测试；允许环必须声明 breaker与termination。

### OL-P1-036 · 收窄 ResourceLease release authority

release closure改持 `Weak<ResidencyAuthority>`或generation token，不克隆整个 ResourceManager、event publisher和commit gate。

### OL-P1-037 · 禁止 payload反向拥有 authority lease

ResourceData contract声明 nested lease政策；检测 self/same-authority cycle，必要时使用weak handle、arena epoch或显式 owner graph。

### OL-P1-038 · 固化 RetainedEditorHost weak callback模式

将 callback wiring helper、owner-gone no-op/diagnostic和native presenter弱捕获纳入UI conformance gate。

### OL-P1-039 · 完整化 owner revocation listener token

保留 owner自动删除；补单 listener提前close、generation与在途通知语义，避免captured state只能等整个owner结束。

### OL-P1-040 · 统一 ForeignGenerationLease

所有跨DLL function pointer、callback table、host handle和allocation都绑定同一 library generation admission/quiescence协议。

### OL-P1-041 · 固化 WeakBridge 与 VM root lease正例

增加 provider reload、stale generation、last root drop、registry先drop和并发pin测试；不得退回永久strong cache。

### OL-P1-042 · 为 lease提供显式 release结果

资源、GPU、native、host lease支持typed close/receipt；Drop fallback幂等且不吞掉产品shutdown必须看到的失败。

### OL-P1-043 · 给网络 socket/listener/connection提供 owner guard

保留 manager close和worker Drop；上层句柄可选 RAII guard，service shutdown全量close并报告未关闭资源，不依赖用户逐个记忆。

### OL-P1-044 · 禁止无监督 detach escape

detach必须转移到有身份的 supervisor并生成 terminal receipt；`mem::forget`/ManuallyDrop按Tooling21逐项证明ownership transfer。

### OL-P1-045 · 建立统一 LeakCensus

按owner/generation采集subscription、callback、observer、lease、resource、task/thread、native allocation和detached count，不以进程RSS单指标判定。

### OL-P1-046 · 建立生命周期 fault injection

覆盖owner先drop、callback重入、close失败、receiver死亡、plugin reload、World销毁、DLL transition和cycle payload。

### OL-P1-047 · 量化所有权策略性能

在同workload测量 weak upgrade、token lookup、snapshot fanout、reclaim queue和callback lease开销；不能以未经测量的性能理由保留永久strong边。

### OL-P1-048 · 以 OwnershipReceipt作为产品资格门

BuildSet-bound场景必须在terminal后达到声明census并完成quiescence；partial/forced/unknown owner不得标clean/reload-safe/leak-free。

## 7. P2 完善项

### OL-P2-001 · 生成所有权图可视化

按owner层级展示strong/weak/lease/subscription边和SCC，支持从finding回到source，图本身不替代验证。

### OL-P2-002 · 增加 active token调试面板

显示owner、generation、创建点、age、queue/callback count和closing state，敏感payload不得暴露。

### OL-P2-003 · 增加 lease provenance采样

开发构建按预算记录长寿lease创建栈和最后使用点；release构建保持低成本聚合指标。

### OL-P2-004 · 提供 weak-upgrade失败诊断政策

区分正常owner gone、stale generation和异常premature drop，避免所有失败都静默no-op或刷屏。

### OL-P2-005 · 增加 token misuse property test

随机注册/clone/drop/close/reuse验证幂等、stale reject、slot generation与最终census。

### OL-P2-006 · 增加 cycle regression corpus

保存ResourceData、callback、plugin/interface、UI和VM代表性环形fixture，验证breaker而非只看strong count。

### OL-P2-007 · 生成 callback capture lint建议

标记长期 closure捕获Arc root、CoreHandle、World facade或DLL function pointer候选；人工/AST语义确认后才升级finding。

### OL-P2-008 · 增加 owner teardown timeline

记录close开始、admission关闭、last callback、registry revoke、resource release和final receipt，定位timeout根因。

### OL-P2-009 · 发布所有权API工程手册

说明何时用borrow/weak/lease/snapshot、如何实现reclaim queue与close/Drop，绑定Tooling28 currentness。

### OL-P2-010 · 建立 reference currentness复核

通过Tooling33追踪Unreal delegate、Bevy observer、Fyrox broadcaster、Godot signal和Unity RTHandle版本漂移。

### OL-P2-011 · 增加 debug owner labels

token/lease可附业务label和owner path，禁止用字符串作为权威identity或安全判断。

### OL-P2-012 · 建立 ownership debt趋势

统计unknown edge、naked registration、unowned callback、allowed cycle和forced detach；不得把Arc/Weak数量作为质量KPI。

## 8. 参考引擎差异与适用性

### 8.1 Unreal

Unreal multicast delegate返回 `FDelegateHandle`，支持按handle或owner移除，并在广播/维护中压缩不可执行实例；delegate binding还区分 UObject、shared pointer与weak lambda语义。`FWeakObjectPtr`以对象索引/serial验证对象是否仍是同一代而不延长其生命。Zircon不需要复制 UObject/GC，但需要达到同等的“绑定模式可见、可撤销、旧代失效、死亡callback可压缩”。

### 8.2 Bevy

Bevy observer本身是ECS entity；observer despawn会从cache和被观察实体的 `ObservedBy`关系中注销，测试明确保证despawn后不再触发。Zircon的entity-specific observer清理方向相似，但global observer仍是裸ID。适用结论是让observer拥有可被World/system/plugin生命周期管理的实体或token，不是把全部事件改成Bevy API。

### 8.3 Fyrox

Fyrox resource broadcaster用generational Pool handle支持显式remove，broadcast又 `retain` 发送成功者，receiver死亡时自动清sender。Zircon Core EventBus和asset change stream已达到这一方向，`UiEventManager`则没有。可直接借鉴的是generational token加死亡端prune，不是照搬其同步mpsc或Pool实现。

### 8.4 Godot

Godot signal connection记录 callable、flags与reference count；`CONNECT_REFERENCE_COUNTED`允许重复连接计数，disconnect递减并在归零时删除。Object/RefCounted/WeakRef又区分对象身份、引用所有权和弱查询。Zircon应吸收连接可撤销、引用次数明确和对象代际验证，不应引入Godot式全局ObjectDB作为新的service locator。

### 8.5 Unity Graphics

Unity `RTHandleSystem`显式跟踪 auto-sized与resize-on-demand handles，重复Initialize时会报告未释放资源，`Release`从owner集合删除，`Dispose`遍历释放并清集合。它只证明render resource owner需要diagnostic census与全量terminal cleanup，不是通用应用callback/observer或GC方案。Zircon的GPU/资源 lease应采用同等owner诊断，同时保持Rust RAII和generation优势。

## 9. 实施顺序

### M0 · Inventory 与不变量

- 生成OwnershipEdgeInventory与BuildSet source graph；
- 冻结新增naked registration、unowned callback和unknown detach；
- 给现有edge分类，unknown先显式入账而非机械改成Weak。

### M1 · Token、capture 与 receipt基础

- 定义SubscriptionToken、CloseState、CallbackCapturePolicy和OwnershipReceipt；
- 抽取EventBus、runtime mirror、native callback lease conformance；
- 接入owner/generation/stale reject。

### M2 · 高风险裸订阅硬切

- 先切UiEventManager、state hooks、World/EventStore observers；
- 再切Editor message lifecycle bridge、event listener closing和UI handler tables；
- 删除旧裸API，不保留长期双合同。

### M3 · 强环与窄authority

- 收窄ResourceLease release edge并加入cycle fixture；
- 审计service callback/root capture与settings subscriber；
- 所有允许SCC声明breaker和terminal order。

### M4 · Plugin、ABI 与 network convergence

- 给RPC/sound callback和owner listener接PluginModuleId/LibraryGeneration；
- 为network resource补owner guard与shutdown census；
- DLL unload统一admission-close-quiesce-revoke-release顺序。

### M5 · 动态验证与资格

- 运行receiver death、observer churn、World/Host close、plugin/DLL reload和cycle fault；
- 采集heap/GPU/foreign/thread/callback/lease census；
- 性能回归与leak qualification绑定同一BuildSet/workload。

### M6 · Required gate 与文档

- registration/capture/cycle/terminal receipt进入required CI；
- waiver包含owner、expiry、reason和source fingerprint；
- 只有G01-G40动态证据完成后才能升级implementation状态。

## 10. 验收门

| Gate | 验收内容 |
|---|---|
| G01 | Cargo-resolved shipping/editor/tool BuildSet中的所有长期ownership edge进入AST/ABI inventory |
| G02 | 每条edge有source/target owner、strength、generation、revoke、in-flight和terminal policy |
| G03 | 新增register/subscribe/observe API必须返回token或证明startup immutable freeze |
| G04 | unknown strong root capture、unknown detach和allowed cycle无waiver时required gate失败 |
| G05 | token包含owner identity与generation，跨owner/旧代close返回typed failure |
| G06 | close幂等且可等待quiescence；Drop有限、无panic并不伪报clean |
| G07 | publisher不因死亡subscriber永久保留业务owner或无效sender |
| G08 | receiver/owner drop后自动prune或reclaim，最终registry count归零 |
| G09 | unregister后新callback停止；在途snapshot完成/取消政策可验证 |
| G10 | bounded queue、bytes、age与overflow policy绑定具体subscription owner |
| G11 | Core EventBus在bus-first/subscriber-first/concurrent drop场景最终清空topic/subscriber |
| G12 | UiEventManager receiver drop无需手动ID即可注销且无unbounded retention |
| G13 | state hook可按token/module/runtime owner撤销并拒绝stale generation |
| G14 | World global observer随system/plugin/owner termination自动删除 |
| G15 | entity-specific observer在entity despawn后不触发并清反向索引 |
| G16 | EventStore和runtime mirror token drop在无`&mut World`时可靠提交reclaim |
| G17 | runtime mirror reclaim失败可重试且World Drop后live record为0 |
| G18 | Editor message context/host/plugin subscriber均由lease管理，bridge不遗留naked ID |
| G19 | Editor event listener quiescent close后旧route snapshot不能无限写detached inbox |
| G20 | Settings subscriber支持clear/replace generation且authority复用不保留旧host |
| G21 | UiNode/tree/document/plugin终止会清pointer/navigation/router/event route handler |
| G22 | route replacement原子retire旧ID，旧generation invocation返回typed stale |
| G23 | RPC descriptor/validator/handler同owner事务注册和撤销，reload无旧closure |
| G24 | sound/native/foreign callback调用期间持正确LibraryGeneration lease |
| G25 | owner revocation先阻止新调用、等待在途，再移除 executable contribution和卸载DLL |
| G26 | weak callback upgrade失败区分正常owner gone与stale/premature failure |
| G27 | service/plugin factory默认只获得weak root，strong recapture必须显式声明 |
| G28 | OwnershipGraph SCC检查覆盖type-erased payload、callback、bridge和UI route |
| G29 | ResourceLease不强持完整ResourceManager，release只触达窄generation authority |
| G30 | ResourceData无法构造manager-payload-lease-manager永久强环，或有经验证breaker |
| G31 | RetainedEditorHost所有长期UI callback保持weak capture且host close后不可调用 |
| G32 | WeakBridge reload后不命中旧provider，operation pin只活到调用结束 |
| G33 | VM最后root lease释放会注销GC root，manager/registry销毁顺序有测试 |
| G34 | network service shutdown全量关闭socket/listener/connection并报告未释放项 |
| G35 | detach只转移给有identity supervisor，forced/timeout进入失败receipt |
| G36 | owner terminal后的subscription/callback/observer/lease/task/thread/native census为声明值 |
| G37 | cycle、receiver death、callback重入、owner-first drop、World/DLL reload fault injection通过 |
| G38 | weak/token/reclaim/lease策略在相同workload下无不可接受性能回归 |
| G39 | OwnershipReceipt绑定source/build/product/scenario，partial/forced不得标leak-free/reload-safe |
| G40 | `git diff --check`、frontmatter路径、finding ID、severity、fingerprint与索引/coverage计数通过 |

## 11. 状态与产出记录

| 里程碑 | 状态 | 日期 | 证据 |
|---|---|---|---|
| production-like ownership lexical inventory | review_complete | 2026-08-16 | HEAD `25e09a23...d404`；11,877 files / 1,199,398 lines / 2,819 shared-owner / 212 weak-owner / 113 callback-owner signals |
| representative owner/termination review | review_complete | 2026-08-16 | EventBus、asset stream、UI、state hook、scene observer、resource lease、plugin/native/VM、Editor bus、network/sound/RPC |
| source/reference evidence fingerprint | review_complete | 2026-08-16 | 59 unique paths / 886,906 bytes / SHA-256 `61c56764...b0adb` |
| reference ownership comparison | review_complete | 2026-08-16 | Unreal delegate/weak object、Bevy observer entity、Fyrox broadcaster、Godot signal/refcount/weak、Unity RTHandle owner |
| OwnershipGraph/Token/Capture/Census/Receipt architecture | design_complete | 2026-08-16 | 本篇第5节；未实现schema、token、cycle validator或receipt |
| production refactor与动态leak/unload tests | pending | - | 本篇只review，不修改production/tests |

当前结论仍是 `review_complete / implementation_pending`。在M0-M6和G01-G40完成前，Zircon不能把“Rust内存安全”“进程退出会回收”“Arc clone能工作”或“存在手动unsubscribe”当成owner可终止、World可销毁、plugin/DLL可卸载和产品无泄漏的工程证明。
