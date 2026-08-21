---
related_code:
  - zircon_app/src/entry/entry_runner/bootstrap.rs
  - zircon_app/src/entry/runtime_library/wake_registry.rs
  - zircon_editor/src/core/plugin/manager.rs
  - zircon_editor/src/core/plugin/manager/project_registration.rs
  - zircon_editor/src/ui/asset_editor/node_projection.rs
  - zircon_editor/src/ui/host/editor_manager.rs
  - zircon_editor/src/ui/layouts/views/view_projection/projection_cache.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_theme.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/template_runtime_projection.rs
  - zircon_editor/src/ui/retained_host/viewport/editor_viewport_render_defaults.rs
  - zircon_editor/src/ui/template_runtime/runtime/build_session.rs
  - zircon_editor/src/ui/v2_design_tokens.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/state/core_runtime_state.rs
  - zircon_runtime/src/core/runtime/tasks/pools.rs
  - zircon_runtime/src/core/runtime/tasks/timer.rs
  - zircon_runtime/src/diagnostic_log/sink.rs
  - zircon_runtime/src/dynamic_api/session/registry/allocation_registry.rs
  - zircon_runtime/src/dynamic_api/session/registry/session_store.rs
  - zircon_runtime/src/foundation/runtime/config_manager/commit_fence.rs
  - zircon_runtime/src/plugin/native_plugin_loader/discover/authority.rs
  - zircon_runtime/src/scene/dynamic_scene/session/io/atomic.rs
  - zircon_runtime/src/text/font/shared.rs
  - zircon_runtime/src/text/render_state.rs
tests:
  - zircon_editor/src/core/plugin/manager/tests/project_registration.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/workbench_projection_layout.rs
  - zircon_runtime/src/core/runtime/tests/activation/behavior/deactivation/veto_atomicity.rs
  - zircon_runtime/src/diagnostic_log/sink/tests/lifecycle.rs
  - zircon_runtime/src/dynamic_api/session/registry/tests.rs
  - zircon_runtime/src/text/font/shared/tests.rs
plan_sources:
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/coverage.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/12-settings-preferences-scope-persistence-locale-i18n-appearance-plugin-extensibility-review.md
  - docs/plans/optimize/zircon_runtime/01-core-runtime-lifecycle-registry-review.md
  - docs/plans/optimize/zircon_runtime/02-core-runtime-events-tasks-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/11b-runtime-text-font-shaping-layout-editing-ime-review.md
  - docs/plans/optimize/zircon_runtime_interface/05-runtime-host-foreign-output-safe-api-ownership-admission-budget-fuse-observability-review.md
  - docs/plans/optimize/zircon_tooling/10-test-architecture-partition-selection-isolation-fixture-flake-results-review.md
  - docs/plans/optimize/zircon_tooling/24-concurrency-locking-atomic-ordering-blocking-thread-lifecycle-backpressure-deadlock-review.md
  - docs/plans/optimize/zircon_tooling/25-memory-allocation-domain-budget-oom-pressure-fragmentation-pooling-cache-residency-observability-review.md
  - docs/plans/optimize/zircon_tooling/29-rust-module-boundary-root-entry-large-file-declaration-behavior-folder-topology-review.md
  - docs/plans/optimize/zircon_tooling/33-reference-engine-source-corpus-snapshot-provenance-citation-applicability-comparison-currentness-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/Subsystems/Subsystem.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/Subsystems/SubsystemCollection.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/Subsystems/EngineSubsystem.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/Subsystems/GameInstanceSubsystem.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/Subsystems/WorldSubsystem.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/Subsystems/LocalPlayerSubsystem.h
  - dev/bevy/crates/bevy_ecs/src/system/system_param.rs
  - dev/bevy/crates/bevy_ecs/src/world/mod.rs
  - dev/Fyrox/fyrox-impl/src/engine/executor.rs
  - dev/godot/core/config/engine.cpp
  - dev/godot/core/config/engine.h
  - dev/godot/scene/main/scene_tree.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraphObjectPool.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 34 · Global State Scope、Singleton、Service Locator、Static Registry/Cache、Initialization、Reset 与 Multi-instance Isolation 审查

## 1. 结论

Zircon 已经证明自己不必依赖一个真正的全局 Engine singleton。`CoreRuntime` 把 module、service、event bus、config、scheduler、clock、diagnostics 和 state registry 放在 `Arc<CoreRuntimeInner>` 中；每次 `CoreRuntime::new()` 都创建独立实例。冻结后的 module graph 有确定 activation order，显式 shutdown 会按逆序 deactivate。动态日志控制器还实现了 active generation、dynamic session count、先 unpublish 再 join 的 library-unload 协议；App wake callback 用 token registration 和 `Drop` 注销；native session/allocation registry 把 allocation 绑定到 session。这些是可保留并上升为全产品状态作用域合同的基础。

当前缺口是：实例级 Core 周围仍散布大量没有统一身份和生命周期的进程状态。本轮按明确口径扫描 `zircon_app`、`zircon_editor`、`zircon_hub`、`zircon_plugins`、`zircon_runtime`、`zircon_runtime_host`、`zircon_runtime_interface` 与 `zircon_reflect_derive` 的 tracked Rust 候选，排除 tests/benches/fixtures、典型测试文件和 native dynamic fixture，共覆盖 11,849 个文件、1,316,739 个物理行、45,742,170 bytes。检出 253 个 lexical static declaration，分布在 171 个文件；其中 94 个 `OnceLock`、3 个 `LazyLock`、38 个含 `Mutex` 的 static、2 个含 `RwLock`、1 个 `ArcSwap`、64 个 atomic static，以及 39 个 `thread_local!` 宏。未发现 production-like `static mut`，这是正向事实；但“Rust 不允许数据竞争”不等于“项目 A 不会读到项目 B 的主题、字体、插件、缓存或修订状态”。

已确认的代表性断点包括：默认 `TaskPools` 首次调用后固定进程线程政策，`PROCESS_TIMER` 永久持有最后一个 owner 而让其 Drop/join 在正常进程期不可达；Editor commandlet 使用进程级可变 builtin plugin manager，GUI `EditorManager` 却持有另一实例级 manager；UI Asset Editor 把唯一 `document/surface/size` 投影会话共享给所有窗口；进程字体数据库允许任意 renderer 修改 project composite font 和 default UI family；design token、paint theme 与 DPI scale 使用进程 authority 加 thread-local override；两个 `OnceLock<Option<EditorUiHostRuntime>>` 会把一次瞬态加载失败永久缓存为 `None`；dynamic scene path revision table 永久按路径保存 lineage；builtin template cache 只用 path/mtime/length，不含 project/build/content identity，也没有生产 reset/retire。

本篇登记 **0 项 P0、48 项 P1、12 项 P2和 40 个验收门**。没有新增 P0，因为这些结构已经构成跨项目污染、重启不可恢复、PIE/多世界串扰和 DLL unload 风险，但本轮静态证据没有独立证明 shipping 数据损坏或内存不安全已经发生。Tooling24 继续拥有 lock graph、atomic ordering、thread-local worker cleanup 和并发 interleaving；Tooling25 拥有 cache capacity、residency 与 pressure；Tooling10 拥有 test process/fixture lease；Runtime05、Runtime11B、Editor12 和 Interface05 分别拥有具体 world、font、settings/plugin 与 ABI 行为。本篇只拥有：

`GlobalStateInventory -> StateScopeDefinition -> Owner/Instance/Generation -> InitializationDAG -> StateLease -> Reset/Shutdown/Unload -> IsolationScenario -> StateScopeReceipt -> Qualification`

## 2. 审查边界、口径与限制

### 2.1 当前物理账本

| Evidence | 本轮结果 | 解释 |
|---|---:|---|
| tracked Rust candidate | 11,849 files / 1,316,739 physical lines / 45,742,170 bytes | 文件数不是产品可达 BuildSet；宏、cfg 与调用图需后续解析 |
| static declaration | 253 / 171 files | 包含不可变描述符、ID sequence、metrics、cache、registry 和 service，不能机械视为缺陷 |
| `OnceLock` | 94 / 81 files | 一次初始化语义广泛存在；需要区分 immutable process data 与 mutable scoped owner |
| `LazyLock` | 3 | 当前均接近不可变路由/绑定表；仍需进入 inventory |
| static `Mutex` / `RwLock` / `ArcSwap` | 38 / 2 / 1 | 可变进程 authority 的保守下界，不等于完整共享状态数量 |
| atomic static | 64 | 多数是唯一 ID 或 metrics；exhaustion 和 scope 由 identity/concurrency报告拥有 |
| `thread_local!` | 39 macros / 38 files | 包括 re-entry guard、scratch/cache、active scope 和 test metrics |
| production-like `static mut` | 0 | 必须保留为禁止回归的基线 |
| source evidence currentness | 43个唯一related/test/reference输入、808,312 bytes；SHA-256 `bb80393a...eb3b9` | HEAD `25e09a23178000f2e783ce2143cf70a8b118d404`；不把会自引用更新的plan/index纳入内容指纹 |

static declaration 按 crate root 的词法分布为 Runtime 116、Editor 85、Plugins 41、App 6、Runtime Interface 3、Hub 2。数量不能作为风险排序：Hub 的两个 static sequence 通常比 Editor 的可变 theme authority 风险低，Interface 的少量 ABI static 又可能处在动态库生命周期边界。

### 2.2 StateScope 必须是类型化合同

后续 inventory 至少区分以下作用域，禁止用“global”“shared”“default”替代：

| Scope | 典型 owner | 必需 identity / termination |
|---|---|---|
| `CompileTimeImmutable` | ABI table、migration chain、builtin descriptor | build/schema identity；不需要 runtime reset |
| `Process` | OS integration、真正共享的 worker budget | ProcessInstanceId；明确进程终止是否足够 |
| `DynamicLibraryGeneration` | exported callback table、native registry worker | LibraryInstanceId + generation；unload 前 quiesce |
| `EngineRuntime` | `CoreRuntimeInner`、module/service graph | RuntimeInstanceId；reverse shutdown |
| `EditorHost` | EditorManager、host theme、window registry | EditorHostId；host close/reopen |
| `ProjectSession` | plugin selection、project fonts、asset roots | ProjectSessionId + project generation；close/rollback |
| `World` / `PlaySession` | ECS resources、PIE systems、scene caches | WorldId/PlaySessionId + epoch；world unload |
| `Window` / `Viewport` / `Document` | DPI、projection surface、selection | concrete owner ID；close/resize/document switch |
| `Task` / `Thread` / `Frame` | scratch、re-entry guard、active scope | task/thread/frame epoch；scope guard或worker exit |
| `TestCase` | mutable fixture override、global serial guard | TestRunId/TestCaseId；always-reset terminal |

一个状态只可声明一个 authoritative scope；更窄作用域可以持有 immutable snapshot 或 validated lease，但不得把进程 current value 暗中当成所有 host/project/world 的唯一真相。

### 2.3 Evidence 边界

1. 本轮读取声明、关键 caller、生命周期实现和代表性测试，没有运行动态多项目、多窗口、PIE、DLL reload 或线程退出实验。
2. lexical inventory 会包含 production 文件中较深的 `#[cfg(test)]` static，也会漏掉宏展开、C/C++/C#/ZR script 生成的静态对象；数字只作 source recheck anchor。
3. 当前工作树存在其他会话/用户改动，且 Editor、Hub、WOC 和 plugin metadata 已有不相关动态阻断；本轮不重跑这些已知红线，也不修改 production/tests。
4. Tooling24 已登记 OnceLock/process singleton generation/reset 与 thread-local worker lifecycle。本篇不重复计算并发缺陷，只定义状态属于谁、可被谁看见、何时作废。
5. 路径级 cache 的容量、eviction 和 bytes 归 Tooling25；本篇只要求 key 包含正确 owner/generation，并在 owner termination 后不可命中。

## 3. 必须保留的工程基础

### 3.1 CoreRuntime 已经是实例 owner

`CoreRuntime::new()` 每次创建独立 `CoreRuntimeInner`，module/service registry、event bus、config、clock、diagnostics 和 state 都没有放进 process singleton。`CoreHandle` 通过 `Arc`/`Weak`传播 owner，而不是靠静态 service locator。冻结 module graph 后可按依赖 activation，shutdown 又反向 deactivate。这一骨架应成为 `EngineRuntime` scope 的唯一根，而不是被新的“全局 Engine”替换。

### 3.2 Module shutdown 已有逆序语义

`shutdown_registered_modules_with_drain_timeout`读取 frozen activation order 并逆序 deactivate，失败 activation 还会 reset started service state。正确方向是让所有 runtime-owned后台资源都挂到该 DAG，并生成 shutdown receipt；不是另建一批 `atexit` 或进程静态析构器。

### 3.3 Diagnostic log 已实现可复用的 generation lease

`ProcessLogController`记录 dynamic session count，最后一个 session 释放时先 unpublish active state，再执行 library-unload shutdown/join；稍后 session 得到新 generation。这个模型可以推广给 process-shared timer、native discovery worker、profiler sink 与动态 callback registry。

### 3.4 Wake registry 与 config fence 有明确 lease 雏形

App wake registration 产生非零 token，`Drop`删除 registry entry，FFI trampoline 捕获 panic；config commit fence 以绝对规范化路径和 `Weak` gate避免永久强持有每个 commit owner。这两处证明全局索引可以只存路由 lease，而不必拥有业务对象本身。

### 3.5 Native discovery 已有 root identity、generation 与上限

Discovery authority 将 lexical root解析为 `NativePluginDiscoveryRoot`，refresh snapshot有 generation，root identity cache上限为32并在并发 miss后重核。应保留这些属性；需要补的是 host/library owner、显式 retirement 和 worker shutdown，而不是删除所有缓存。

### 3.6 不可变 process static 是合理优化

ABI method table、migration chain、builtin component descriptor、compute/fullscreen plan、palette/visual constant和只读 route map适合 compile-time或lazy immutable process scope。治理目标不是把它们变成堆分配实例，而是机器声明 immutable、build-bound、无 project input，并禁止未来偷偷加入 mutation。

### 3.7 现有 cache 中已有 bounded/generation-aware 正例

rich text、preview image、host font set、visual asset、SVG/tree和部分 raster cache已经使用容量、bytes或resource/font generation。它们的算法和预算继续归 Tooling25/Runtime11B；StateScope层只补 owner scope、retirement和跨实例 isolation receipt。

## 4. 已确认的结构断点

### 4.1 Instance Core 与 process default 混合

`CoreRuntimeInner`拥有 `TaskPools`，表面上是实例资源；但 `CoreRuntime::new()`调用 `TaskPools::default()`，最终克隆首个 `PROCESS_TASK_POOLS`。因此两个 runtime拥有独立 service/state，却共享由第一次默认参数固定的线程池。显式 `TaskPoolOptions::create_pools()`存在但没有进入 `CoreRuntime`构造合同。类似地，asset/maintenance调用 `TaskTimer::process_default()`，进程槽永久持有一个 `TaskTimer` clone，最后 owner Drop中的 closing/join在正常进程生命周期无法到达。

### 4.2 Plugin manager 有两套 authority

GUI `EditorManager::new()`构造并持有实例级 `EditorPluginManager::builtin(...)`；commandlet runner却调用 `EditorPluginManager::builtin_shared()`，后者是进程级 mutable manager。manager支持替换 project registrations、clear project reports、apply project manifest和推进 lifecycle，但 snapshot没有 ProjectSessionId/EditorHostId。顺序执行两个不同项目必须靠调用者恰好 clear；并发 commandlet 或嵌入式多host则没有身份隔离。不能把“process-wide builtin catalog owner”同时当成 project selection authority。

### 4.3 UI Asset Editor 投影是单一进程会话

`NODE_PROJECTION_SESSION: OnceLock<Mutex<NodeProjectionSession>>`只保存一个 document、surface 和 size。Workbench每个 `UiAssetEditor` pane都调用同一函数，再按 pane数据改 designer tool mode。两个窗口、两个不同大小pane或并行 screenshot/export会串行复用同一 surface；key中没有 HostId、WindowId、DocumentId、template generation或theme/font generation。mutex只阻止数据竞争，不能证明结果属于当前窗口。

### 4.4 Theme、design token、DPI 和字体被提升为 process current value

V2 design token projection是进程 `RwLock`；paint theme是进程 `ArcSwap`，scale factor也是authority字段，只在绘制栈上用 thread-local snapshot临时覆盖；font database是进程 `RwLock`，任意 `TextRenderState`都能替换 owner font、project composite font和default UI family。多窗口不同DPI、多host不同settings、两个project不同字体或runtime/editor同进程时，generation只能通知“变了”，不能说明变化属于哪个owner。

### 4.5 OnceLock<Option<T>> 把瞬态失败变成进程永久状态

`BUILTIN_HOST_RUNTIME`与`EXPORT_WIZARD_PANEL_RUNTIME`在第一次 template加载或路径解析失败时缓存 `None`，此后即使插件安装、asset恢复或project切换也不再重试；caller再用 `Option` fallback吞掉初始化诊断。viewport Hybrid GI环境变量则被显式缓存一次，这对 ProcessStartupConfig 可以合理，但当前没有统一声明何时读取、如何在测试或嵌入式host覆盖、receipt中记录了哪个值。

### 4.6 Path/cache key 缺少 owner 与 content identity

dynamic scene `COMMITTED_PATH_REVISIONS`永久以字符串路径保存 commit/write generation/lineage/revision，没有 project/session scope和retire；同一路径被删除重建、项目关闭后复用或测试重复使用时会继承进程历史。builtin template cache使用 canonical path、mtime纳秒和length，不含content digest、build identity、template compiler version或project generation，也没有生产 clear；时间戳相同的内容替换可错误命中，旧key又会永久累积。

### 4.7 Thread-local cache只绑定线程，不绑定业务 owner

projection cache用字符串key放在线程 `BTreeMap`；active font/theme、script binding projection、font systems、profiling span/frame stack和多个UI composition cache也使用 thread-local。线程复用跨 host/project/world时，thread identity不是state scope。RAII theme scope是正例，但 fallback仍读取process authority；其他cache需显式 owner/generation key和scope exit，不应仅依赖线程最终退出。

### 4.8 初始化成功、ready、关闭和销毁没有共同 receipt

Core module DAG是局部正例，但 process static通常通过 getter在任意 caller第一次触发。仓库没有列出谁允许 first-use、依赖哪些resolved config/provider、失败是否可重试、何时 ready、谁触发 reset/shutdown、如何证明 callback/lease已清空。`CoreRuntime`本身没有 `Drop`自动执行 module shutdown；动态 API session显式调用它，普通 in-process owner必须自行记得。process timer、theme、font、projection、plugin shared manager和template runtime也没有共同终止协议。

## 5. 目标架构

### 5.1 GlobalStateInventory

为所有语言和生成物建立机器清单；Rust第一阶段由AST/HIR而非正则产生。每条至少包含：`StateId`、declaration/symbol、state kind、mutability、scope、owner type、identity fields、generation source、initialization mode、config dependencies、read/write callers、thread/latency domain、reset/shutdown/unload hooks、cache budget link、security sensitivity、test override policy和waiver。

### 5.2 Scope hierarchy 与 identity

建议最小身份链：

`ProcessInstanceId -> LibraryInstanceId -> RuntimeInstanceId / EditorHostId -> ProjectSessionId -> WorldId / PlaySessionId -> WindowId / ViewportId / DocumentId -> Task/FrameEpoch`

不是每条状态都携带全链；它必须携带从自身scope到所有可变输入owner所需的最短身份。跨scope共享只允许 immutable artifact、content-addressed cache或持有更窄owner lease的路由索引。

### 5.3 InitializationDAG

每个 state/service声明 dependencies、configuration phase、thread affinity、retry policy、ready probe、rollback和reverse teardown。Process startup先冻结 `ProcessStartupConfig`，再创建 library/runtime/host实例；禁止 getter第一次读取环境变量、工作目录或默认线程数后静默固定。cycle、missing dependency和double initialization必须 typed failure，不能靠调用顺序或 `expect`。

### 5.4 StateLease 与 termination

可变共享状态返回 `StateLease<OwnerId, Generation>`或领域等价物；lease close后旧handle fail-close。owner termination按 `Quiesce -> Stop admission -> Cancel/drain -> Unpublish -> Join/flush -> Retire generation -> Destroy`执行并产生 receipt。process-scope若只能随进程退出，必须证明不持有 DLL callback、project asset、window handle、GPU resource或任意需要更早释放的对象。

### 5.5 Cache/registry 约束

cache key为 `ContentIdentity + ProducerVersion + PolicyIdentity + ScopeIdentity/Generation`；registry key为 opaque handle + owner/generation，value不跨越owner生命周期。真正跨project的cache必须content-addressed且结果与绝对路径/current settings无关。eviction/bytes由Tooling25定义，scope retirement由本篇定义。

### 5.6 StateScopeReceipt

每次 product/test运行输出：创建了哪些scope实例、resolved config、DAG order、generation、共享/独占决策、active lease峰值、reset/shutdown顺序、超时/强制终止、retired state/callback/cache entries和最终 leak census。没有 receipt 的 exit 0不能证明多实例、hot reload或unload正确。

## 6. P1 重构项

### GS-P1-001 · 建立 GlobalStateInventory 单一真源

收录所有 static、lazy singleton、thread-local、module-level mutable、FFI static、C#/C++ static、ZR全局、环境快照和隐藏service locator；每条绑定source/build fingerprint。

### GS-P1-002 · 定义并类型化 StateScope taxonomy

实现第2.2节作用域枚举和owner规则，禁止自由文本 `global/shared/default/current`直接成为生命周期证明。

### GS-P1-003 · 为每个可变scope分配不可复用实例身份

引入 Process/Library/Runtime/EditorHost/Project/World/Play/Window/Viewport/Document身份；handle必须携带scope和generation，禁止仅靠path、thread或裸整数推断owner。

### GS-P1-004 · 统一 generation、epoch 与 retirement语义

区分 content revision、publication generation、owner epoch和sequence；owner关闭后retire epoch，旧snapshot/handle/cache key必须 fail-close。

### GS-P1-005 · 将状态分类为 immutable、authority、cache、registry、service或metric

每类使用不同规则：immutable不可后加mutation，metric不参与业务判断，cache不成为authority，registry不强拥有业务lifetime，service必须有owner和teardown。

### GS-P1-006 · 把 static admission 加入 required convention gate

新增可变static必须声明inventory ID、scope、owner和termination；`static mut`保持零容忍，宏生成static同样解析。

### GS-P1-007 · 建立跨语言 GlobalState source set

Rust AST先落地，再接入Tooling30定义的C/C++/C#/TypeScript/PowerShell/ZR role；禁止只审Rust后宣称全产品完成。

### GS-P1-008 · 绑定 BuildSet、feature/cfg 与产品可达性

inventory区分declared、compiled、linked、loaded和reachable，避免test-only static污染产品数字，也避免宏/cfg隐藏shipping state。

### GS-P1-009 · 对 process scope实施负面准入

process state不得持有project document、world entity、window/GPU handle、DLL callback或未世代化provider；例外必须有不可变/content-addressed证明或显式unload lease。

### GS-P1-010 · 禁止 service locator 隐藏依赖

业务service通过constructor/context/typed handle注入；只允许根composition或ABI trampoline查registry，lookup必须带caller scope和capability。

### GS-P1-011 · 统一 current/default getter的语义

审计 `process_default/shared/current/active/global` API；要么改为显式scope参数，要么命名为不可歧义的ProcessStartup/CompileTimeImmutable并写清限制。

### GS-P1-012 · 建立 DomainOwner 与修复路由

每条inventory指定runtime/editor/plugin/app/interface owner和canonical report；本篇不成为所有业务global的第二owner。

### GS-P1-013 · 建立 InitializationDAG manifest

声明节点、依赖、phase、thread affinity、config/provider input、ready条件、failure policy和teardown reverse edge。

### GS-P1-014 · 把隐式 first-use initialization移出热路径

可失败、可配置或拥有线程/I/O的 `get_or_init`在composition阶段显式创建；热路径只读已发布handle/snapshot。

### GS-P1-015 · 对 initialization cycle 和重入 fail-close

检测A初始化读取B、B又读取A，以及callback重入同一节点；输出依赖链typed diagnostic，不阻塞或缓存半初始化值。

### GS-P1-016 · 实现 startup transaction 与逆序rollback

任一节点失败时只回滚本次已创建generation，按DAG逆序停止admission、drain、unpublish和destroy；保留last-good与失败receipt。

### GS-P1-017 · 定义 OnceLock失败缓存政策

区分 infallible immutable、sticky fatal和retryable initialization；禁止用 `OnceLock<Option<T>>`把瞬态I/O/插件/asset失败永久降级且丢失错误。

### GS-P1-018 · 冻结 ProcessStartupConfig

环境变量、cwd、CPU parallelism、locale和平台capability在明确phase解析，保存provenance和diagnostic；后续process default只消费snapshot。

### GS-P1-019 · 为嵌入式、多runtime和测试提供显式配置入口

`CoreRuntimeBuilder`接收TaskPools/Timer/StartupConfig和共享政策；测试不得通过顺序抢先初始化process singleton来配置后续case。

### GS-P1-020 · 区分 initialized、ready、degraded 与 unavailable

一次构造成功不等于依赖ready。每个节点发布typed lifecycle state与reason，Capability Truth继续由Tooling16拥有。

### GS-P1-021 · 生成 InitializationReceipt

记录实际DAG、resolved config、duration、thread、provider/build identity、retry/rollback和最终generation；source字符串测试不能替代。

### GS-P1-022 · 对动态新增/删除节点实施同一DAG

plugin hot reload、device recreate、project open/close与world/PIE创建都必须走依赖图和reverse teardown，不走旁路setter。

### GS-P1-023 · 强制所有生命周期owner拥有显式 shutdown

shutdown返回结果与receipt；Drop只作无panic、有限兜底。必须能区分正常关闭、timeout、forced detach和process-abort。

### GS-P1-024 · 让根owner自动触发且等待 teardown

Runtime/App/EditorHost/DLL wrapper在terminal path统一调用shutdown；审计普通in-process `CoreRuntime`当前未自动调用module shutdown的问题。

### GS-P1-025 · 保留 CoreRuntime实例边界并补 RuntimeInstanceId

给 `CoreRuntimeInner/CoreHandle/CoreWeak/ServiceHandle`绑定runtime identity/generation；跨runtime handle使用必须可诊断拒绝。

### GS-P1-026 · 将 TaskPools共享政策移入 CoreRuntimeBuilder

显式选择ProcessShared或RuntimeOwned，绑定线程预算和startup receipt；不得由首个 `TaskPools::default()`静默决定所有runtime。

### GS-P1-027 · 让 TaskTimer拥有可关闭generation

process-shared timer采用log controller式lease和last-session shutdown，或改为runtime-owned；DLL unload前取消callback并join，不能由永久OnceLock clone阻断Drop。

### GS-P1-028 · 推广 ProcessLogController正向模式

抽取generation lease/unpublish-before-join协议供profiler、native discovery、timer和callback registry复用，但不强行共享同一巨型manager。

### GS-P1-029 · 世代化 dynamic session/allocation registry

在现有session绑定基础上加入LibraryInstanceId、opaque generation handle和unload census；旧library的裸handle不得命中新代registry。

### GS-P1-030 · 将 wake registry 模式标准化

保留RAII unregister与panic containment，补Host/Library generation、token exhaustion policy和最终registry-empty receipt。

### GS-P1-031 · 删除 Editor plugin manager双authority

builtin catalog可为immutable process snapshot；lifecycle、project registration与selection必须归EditorHost/CommandletSession实例，commandlet不得共享进程mutable manager。

### GS-P1-032 · 给 plugin project publication绑定ProjectSessionId

replace/clear/apply manifest都校验active project generation；project close/rollback强制clear并等待plugin lifecycle，不依赖调用者顺序。

### GS-P1-033 · 将 NodeProjectionSession降到Host/Window/Document scope

由UI Asset Editor pane/session持有surface，key包含template/theme/font/document generation和size；并行窗口不共享可变surface。

### GS-P1-034 · 重构静态 EditorUiHostRuntime 初始化

用host-owned template service或versioned immutable compiled artifact替换 `OnceLock<Option<...>>`；失败保留typed cause并支持新generation重试。

### GS-P1-035 · 完整化 builtin template cache identity

key加入content digest、compiler/schema/build identity和必要scope；旧path/mtime/len条目不能跨project/build错误命中，owner termination触发retire。

### GS-P1-036 · 给 native discovery authority增加host/library lease

保留root/generation/32上限，补显式root retire、refresh worker drain、snapshot release和DLL unload receipt；identity eviction不得遗留无owner snapshot。

### GS-P1-037 · 将 project字体状态移出进程唯一数据库

packaged/system face可共享immutable/content cache；project composite、default UI family和asset owner放入Project/Renderer context，跨renderer共享必须显式同project lease。

### GS-P1-038 · 将 design token/theme/DPI绑定EditorHost和Window

settings生成host theme snapshot，window/viewport持有scale-specific projection；thread-local只作调用栈scope，不作为authority或跨线程传输。

### GS-P1-039 · 为所有 thread-local cache加入owner/generation key

projection/font/script/profiler等cache声明业务scope、reset hook和worker-exit cleanup；Tooling24继续拥有并发与线程终止实现。

### GS-P1-040 · 统一 static cache/registry retirement

所有cache在Project/World/Device/Library generation退役后不可命中并可回收；capacity/bytes/pressure门引用Tooling25，不在本篇复制。

### GS-P1-041 · 提供按scope reset API

至少支持Project close、World unload、PIE stop、Window close、Device recreate、Plugin reload、Library unload与TestCase end；reset必须幂等、世代化且可观测。

### GS-P1-042 · 建立 lease census 与 quiescence gate

shutdown列出active reader/callback/task/handle/cache lease；未归零则超时失败，不卸载DLL或发布“clean shutdown”。

### GS-P1-043 · 实现 unload/reload generation barrier

动态库、native plugin和script VM重载前阻止新调用、等待在途调用、retire旧callback/table/allocation，再加载新generation。

### GS-P1-044 · 建立双ProjectSession同时运行隔离测试

同进程打开A/B项目，使用不同plugin/font/theme/asset root并交错操作；证明snapshot、cache、diagnostic和关闭互不污染。

### GS-P1-045 · 建立多World/PIE隔离测试

Editor World、PIE World、preview World同时创建、暂停、重载和销毁；所有service/cache/timer callback绑定正确WorldId/PlaySessionId。

### GS-P1-046 · 建立close/reopen A-B-A测试

关闭A、打开B、再打开路径相同但内容/身份不同的A；验证template/path revision/plugin/font/projection不继承错误process历史。

### GS-P1-047 · 建立hot reload、device recreate与DLL unload测试

循环加载/卸载多generation，注入初始化失败、timeout、callback重入和stale handle；最终线程、callback、allocation和registry census归零。

### GS-P1-048 · 以 StateScopeReceipt 作为产品资格门

receipt绑定source/build/product/test scenario；partial、forced、missing owner或stale lease不得报告isolated/clean/reload-safe。

## 7. P2 完善项

### GS-P2-001 · 生成 global-state inventory dashboard

按scope、kind、owner、mutation、termination和waiver展示趋势，不以static总数做质量KPI。

### GS-P2-002 · 建立命名规范

`PROCESS_`、`LIBRARY_`、`RUNTIME_`等前缀只作可读提示，机器scope仍来自manifest/type；禁止模糊 `GLOBAL/CURRENT/DEFAULT`。

### GS-P2-003 · 提供 InitializationDAG 可视化

从manifest生成依赖、phase、critical path与reverse teardown图，方便review cycle和启动成本。

### GS-P2-004 · 增加 active scope 调试面板

展示实例ID、generation、owner、lease count和shutdown state；不得暴露credential或secure document内容。

### GS-P2-005 · 增加 stale handle provenance诊断

开发构建报告创建/retire位置、旧新generation和caller scope，release保持bounded structured error。

### GS-P2-006 · 为 cache记录scope命中指标

区分same-owner、cross-owner content-addressed、stale reject和retire reclaim；与Tooling25 memory metrics关联。

### GS-P2-007 · 为 thread-local提供统一scope guard helper

支持嵌套enter/restore、panic unwind和worker exit清理，避免每个UI/VM/profiler自行实现。

### GS-P2-008 · 对DAG顺序做property test

随机合法依赖图验证topological init、failure rollback和reverse teardown；不替代真实产品场景。

### GS-P2-009 · 增加状态生命周期fault injection catalog

覆盖I/O失败、plugin missing、worker hang、poison、generation race、device loss和owner提前drop。

### GS-P2-010 · 建立scoped与shared方案性能基线

测量实例内注入、snapshot、content cache和process lookup成本，避免以性能为由保留未经测量的全局可变状态。

### GS-P2-011 · 发布scope与lifecycle API文档

为构造、lookup、lease、reset、shutdown、unload和receipt提供rustdoc与工程手册，并绑定currentness。

### GS-P2-012 · 定期对参考引擎scope模型做currentness复核

通过Tooling33 Snapshot/Citation机制更新Unreal subsystem、Bevy World、Godot lifecycle、Fyrox executor和Unity domain-reset对照，不机械复制API。

## 8. 参考引擎差异与适用性

### 8.1 Unreal

Unreal `USubsystem`明确声明“与某类engine construct共享生命周期”，并提供 `ShouldCreateSubsystem(Outer)`、`Initialize(Collection)`、`Deinitialize()`；Engine、GameInstance、World和LocalPlayer各有独立base scope。`FSubsystemCollectionBase`保存Outer、显式初始化依赖、支持add/remove initialize/deinitialize并跟踪deinit complete。Zircon不需要复制UObject/Outer，但必须达到同等作用域可见性、依赖初始化和反初始化能力。

### 8.2 Bevy

Bevy `World`拥有唯一 `WorldId`，resource与non-send resource都在World内，且提供clear resources/non-send/all与Drop清理command queue。Zircon已有实例Core和WorldHandle基础；差距是许多font/cache/service仍绕过World/Project/Runtime owner进入process static。适用结论是“把可变资源放回owner”，不是把所有Zircon系统改成Bevy ECS resource。

### 8.3 Godot

Godot仍大量使用engine/server singleton，不能把它当作“全局状态都合理”的依据；但其Engine singleton有add/remove registry和destructor unpublish，SceneTree finalize会断开root、清timer connection、清tween，destructor再清process groups并将singleton置空。Zircon的process static至少要达到可注销、可finalize、可证明清空，而不是永久OnceLock即完成工程化。

### 8.4 Fyrox

Fyrox `Executor`实例持有Engine、event loop与plugin集合，graphics context create/destroy事件回到同一engine/plugin owner。它说明Rust引擎可以由显式executor实例组合生命周期；Zircon应保留Core/EditorManager实例组合，不引入全局Engine facade。

### 8.5 Unity Graphics

RenderGraph object pool同时存在per-instance temp allocation lists和static generic pools；它显式 `ReleaseAllTempAlloc/Cleanup`，Editor assembly/domain load后还用 `RuntimeInitializeOnLoadMethod`清static pool但保留注册列表。Zircon可以保留高价值process cache，但必须有owner release、domain/library reload reset和不清错注册元数据的策略。

## 9. 实施顺序

### M0 · Inventory 与不变量

- 实现Rust AST GlobalStateInventory和scope taxonomy；
- 冻结 `static mut=0`、禁止未登记可变static；
- 给现有253个声明分类，先标unknown而不机械改代码。

### M1 · Identity 与 root composition

- 引入Process/Library/Runtime/Host/Project/World/Window身份与generation；
- 给CoreRuntimeBuilder、EditorManager和dynamic session root接入；
- 冻结ProcessStartupConfig并移除可失败first-use I/O。

### M2 · InitializationDAG 与 termination

- 将Core module graph推广到process/library/host service；
- 实现startup transaction、retry policy、reverse rollback和receipt；
- 让root terminal path强制shutdown，不依赖调用者记忆。

### M3 · 高风险 process mutable state硬切

- 先切TaskTimer、plugin shared manager、NodeProjectionSession和static EditorUiHostRuntime；
- 再切font/design token/theme/DPI和path/template cache scope；
- 每项删除旧process mutable path，不保留双authority兼容层。

### M4 · Cache/registry/thread-local convergence

- 与Tooling25合并content/budget key，与Tooling24合并worker cleanup；
- registry只持route/weak/lease，不拥有更窄业务lifetime；
- 实现scope reset/retire和stale reject。

### M5 · Isolation 与 unload qualification

- 运行双project、多World/PIE、双window/DPI、A-B-A reopen；
- 循环plugin/DLL/device reload，注入失败与timeout；
- receipt证明线程、callback、allocation、lease和registry最终归零。

### M6 · Required gate 与文档

- static admission、DAG、isolation和receipt进入required CI；
- 迁移waiver有owner、expiry与source fingerprint；
- 只有BuildSet-bound动态证据完成后，报告状态才可从pending变更。

## 10. 验收门

| Gate | 验收内容 |
|---|---|
| G01 | tracked product BuildSet中的所有global/static/thread-local state均进入AST inventory，宏/cfg可追溯 |
| G02 | inventory每条都有StateId、kind、scope、owner、identity、generation和termination |
| G03 | production-like `static mut`保持0；新增可变static无manifest则required gate失败 |
| G04 | compile-time immutable static证明不读取project/world/window/runtime mutable input |
| G05 | process state不持有更窄scope object、callback、GPU/window/DLL handle |
| G06 | Runtime/Host/Project/World/Window身份不可跨generation复用 |
| G07 | stale handle/cache/registry lookup返回typed failure而非命中新owner |
| G08 | service lookup只发生在composition/ABI route，业务依赖显式注入 |
| G09 | `global/shared/current/default` API均有机器scope或完成重命名/硬切 |
| G10 | ProcessStartupConfig在first runtime前冻结并记录env/cwd/CPU/locale provenance |
| G11 | 可失败或有I/O/线程的初始化不在热路径lazy getter执行 |
| G12 | `OnceLock<Option<T>>`不再缓存retryable failure；错误cause可观测 |
| G13 | InitializationDAG无cycle、missing dependency和隐式顺序依赖 |
| G14 | 初始化失败只回滚本代节点并按逆依赖序执行 |
| G15 | initialized/ready/degraded/unavailable状态与reason分离 |
| G16 | 动态plugin/project/world/device节点使用同一DAG add/remove协议 |
| G17 | 每个根owner terminal path显式调用shutdown并等待receipt |
| G18 | Drop兜底不panic、不无限阻塞且不伪报clean shutdown |
| G19 | CoreRuntime实例registry/state/config/clock互相隔离 |
| G20 | TaskPools显式选择ProcessShared或RuntimeOwned并绑定预算receipt |
| G21 | process timer最后lease释放可unpublish、cancel、join并创建新generation |
| G22 | dynamic log/profiler/discovery worker在DLL unload前无旧callback/thread |
| G23 | dynamic session/allocation handle绑定LibraryInstanceId与generation |
| G24 | wake registry在host关闭后旧token无效且最终entry count为0 |
| G25 | GUI与commandlet不共享project-mutable plugin manager |
| G26 | project plugin publication绑定ProjectSessionId，close/rollback自动clear |
| G27 | 两个UI Asset Editor pane不共享可变surface/document/size authority |
| G28 | builtin template runtime失败可按新generation重试并保留typed error |
| G29 | template cache key含content/compiler/schema/build identity且scope可retire |
| G30 | native discovery root、snapshot、worker在host/library关闭后可清退 |
| G31 | project font/composite/default family不会传播到另一个ProjectSession |
| G32 | host theme/design token按EditorHost隔离，DPI按Window/Viewport隔离 |
| G33 | thread-local cache key含业务owner/generation，scope exit和worker exit可清理 |
| G34 | path revision/config gate在project/test结束后不继承错误历史且可回收 |
| G35 | 双project交错测试证明plugin/font/theme/asset/cache/diagnostic隔离 |
| G36 | Editor World、PIE World、preview World并存与销毁无状态串扰 |
| G37 | A-B-A close/reopen在同路径内容变更后不命中旧owner generation |
| G38 | plugin/DLL/device多轮reload后thread/callback/allocation/lease census归零 |
| G39 | StateScopeReceipt绑定source/build/product/scenario，partial/forced不得标clean |
| G40 | `git diff --check`、frontmatter路径、finding ID、severity与索引/coverage计数全部通过 |

## 11. 状态与产出记录

| 里程碑 | 状态 | 日期 | 证据 |
|---|---|---|---|
| production-like Rust lexical inventory | review_complete | 2026-08-16 | HEAD `25e09a23...d404`；11,849 files / 1,316,739 physical lines / 253 static declarations / 39 thread-local macros / 0 static mut |
| representative state owner/caller review | review_complete | 2026-08-16 | Core、TaskPools/Timer、log、dynamic registry、wake、plugin、projection、theme/font、template/path cache |
| source/reference evidence fingerprint | review_complete | 2026-08-16 | 43 unique paths / 808,312 bytes / SHA-256 `bb80393a...eb3b9` |
| reference scope/lifecycle comparison | review_complete | 2026-08-16 | Unreal subsystem、Bevy World、Godot Engine/SceneTree、Fyrox Executor、Unity RenderGraph pool |
| StateScope/InitializationDAG/Receipt architecture | design_complete | 2026-08-16 | 本篇第5节；未实现schema、builder、lease或validator |
| production refactor与动态isolation tests | pending | - | 本篇只review，不修改production/tests |

当前结论仍是 `review_complete / implementation_pending`。在M0-M6和G01-G40完成前，Zircon不能把进程内“没有数据竞争”当成project/world/window隔离，也不能把进程退出当成plugin reload、DLL unload、PIE stop和Editor host关闭的生命周期证明。
