---
title: Runtime Dynamic Session / Registry / FFI / Frame / Event / UI / World Sync / Shader Prewarm 当前源码复审
category: zircon_runtime
report_id: Runtime131
review_date: 2026-08-24
baseline_head: b6f4872cf421a585015897ee448105ba646dda1e
baseline_epoch: 389
verification_head: 457b304730e18824323869b2a2f8358430f482af
verification_epoch: 390
supersedes_currentness_of:
  - docs/plans/optimize/zircon_runtime/43-dynamic-runtime-session-registry-ffi-frame-event-extract-host-request-world-sync-ui-shader-prewarm-product-integration-review.md
related_code:
  - zircon_runtime/src/dynamic_api
  - zircon_runtime_interface/src/runtime_api
  - zircon_app/src/entry/runtime_library
  - zircon_app/src/entry/runtime_entry_app/frame_loop.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests
  - zircon_editor/src/core/gateway
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/material_sources.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/run.rs
tests:
  - zircon_runtime/src/dynamic_api/tests
  - zircon_runtime/src/dynamic_api/session/tests
  - zircon_runtime/src/dynamic_api/session/registry/tests.rs
  - zircon_runtime/src/dynamic_api/shader_prewarm/tests.rs
plan_sources:
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/01-core-runtime-lifecycle-registry-review.md
  - docs/plans/optimize/zircon_runtime/02-core-runtime-events-tasks-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/07-script-plugin-runtime-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/22-time-clock-domain-fixed-step-determinism-rng-replay-scheduling-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/41-operation-service-handler-registry-admission-prepare-apply-progress-cancel-deadline-harvest-retention-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/42-builtin-runtime-module-catalog-profile-target-feature-selection-extension-registration-capability-load-report-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/43/2026-08-19-world-invalidation-batch-tail-queue.md
  - docs/plans/optimize/zircon_runtime/43/2026-08-19-world-invalidation-item-tail-queue.md
  - docs/plans/optimize/zircon_runtime_interface/01-runtime-dll-abi-ffi-version-handle-foreign-ownership-review.md
  - docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
  - docs/plans/optimize/zircon_runtime_interface/05-runtime-host-foreign-output-safe-api-ownership-admission-budget-fuse-observability-review.md
  - docs/plans/optimize/zircon_app/01-product-host-bootstrap-loop-dynamic-runtime-shutdown-review.md
  - docs/plans/optimize/zircon_app/06-vampire-roguelite-example-project-asset-script-gameplay-evidence-product-integration-review.md
  - docs/plans/optimize/zircon_plugins/05-shader-wgsl-family-importer-compiler-artifact-native-product-integration-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Launch/Private/LaunchEngineLoop.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/Engine.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/GameEngine.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Modules/ModuleManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Private/ShaderPipelineCache.cpp
  - dev/bevy/crates/bevy_app/src/app.rs
  - dev/bevy/crates/bevy_app/src/sub_app.rs
  - dev/bevy/crates/bevy_app/src/plugin.rs
  - dev/bevy/crates/bevy_app/src/schedule_runner.rs
  - dev/godot/main/main.cpp
  - dev/godot/scene/main/scene_tree.cpp
  - dev/godot/core/extension/gdextension_manager.cpp
  - dev/Fyrox/fyrox-impl/src/engine/mod.rs
  - dev/Fyrox/fyrox-impl/src/engine/executor.rs
  - dev/Fyrox/fyrox-impl/src/plugin/mod.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraph.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraphCompilationCache.cs
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Runtime131 · Dynamic Runtime Session / Registry / FFI / Frame / Event / UI / World Sync / Shader Prewarm 当前源码复审

## 1. 结论

当前`dynamic_api`比Runtime43首次审查时多了真实的局部修补：foreign allocation有outstanding count/bytes与high-water census；host request分页测试证明超过页上限时不会丢项；plugin event页会携带`remainingDeliveries`与oldest age并复用预编码descriptor；world invalidation提交已经由头部`remove(0)`改成尾部`truncate`；extract byte扫描只在重建时计算；UI消费前提交physical input state的新测试也直接覆盖了卡键根因。结构审计同时确认60/60源码文件、12/12 FFI表、23/23 session operation wrapper、12/12 headless入口和9/9 panic boundary没有形状漂移。这些底座应保留。

但这仍不是工程级Runtime Session Control Plane。V7函数表和Host V1没有capability、allocator、task/thread domain、clock、platform surface或lifecycle negotiation；API table使用exact size，扩展只能再切版本。session、allocation仍由进程全局`OnceLock<Mutex<HashMap>>`统领，handle是裸递增`u64`，单个`RuntimeDynamicSession` mutex仍包住event、tick、query、watch、encode、capture、UI与operation adapter。destroy无限等待action/callback，模块drain却是零预算；App `Drop`在destroy失败时仍会`process::abort()`。

产品和渲染语义也没有收敛。Dynamic层继续维护第二套六profile，Headless/Minimal仍不能证明能力闭包；project scene存在prepare后物理重读的TOCTOU，navmesh与startup script走硬编码/隐式全加载。只有default viewport和Win32 surface；resize销毁重建viewport；render不可用与pipeline未就绪仍可返回成功黑帧。extract cache key遗漏asset/material/plugin/render/device/UI generation，命中仍深clone完整extract。Runtime UI仍扫描全项目UI、忽略alias错误、按顺序分配surface ID、mask node ID、只保留最后一个raster scale且没有hot reload；无UI时仍伪造accessibility tree。HUD/menu还硬编码`gameplay.hud_text`、`gameplay.menu_state`、`Vampire Roguelite`、`Blood Bolt`和`Retry`。

event/input/host/world/shader同样只有适配器级能力。event envelope没有timestamp、sequence、device/user/window generation；logical key只覆盖修饰键、数字和WASD，touch继续退化成单cursor/mouse-left。host request分IME/rumble/cursor排空，没有全局sequence、request ID或ack。plugin page有backlog metadata但不wake，App还丢弃该metadata；world tail queue消除了局部二次复杂度，却仍无cursor/remaining/dropped/resync且在session锁内试探encode。shader prewarm仍挂在dynamic模块之外的独立同步路径，CLI硬编码`naga-29.0.1`、`wgpu-29.0.1`、`wgpu-runtime`，WGPU validation每类pass新建offscreen backend，没有共享device、取消、进度、driver identity或产品PSO usage闭包。

Runtime43的64项P1本轮重判为 **58 Open、6 Partial、0 Closed**；16项P2为 **16 Open、0 Partial、0 Closed**；42项资格门为 **38 Fail、4 Partial、0 Pass**。Partial只表示局部机制与测试真实存在，不代表ABI、产品或性能资格闭合。本文不新增P0，也不复制Runtime01/02/05/07/09A/09C/11A/22/24/41/42、Interface01/03/05、App01/06或Plugins05的canonical ownership。

## 2. 审查边界、方法与currentness

### 2.1 当前物理范围

| 范围 | 文件 / 行 / 非空行 / bytes / test declarations / ignored | 本轮证据 |
|---|---:|---|
| `zircon_runtime/src/dynamic_api`完整实现与测试 | **77 / 18,762 / 17,228 / 682,974 / 195 / 11** | root、session、registry、FFI、frame、event/input、host request、UI、world、operation adapter、bounded JSON、prewarm及全部direct tests；fingerprint `12ffd5daccc7b28b3481250bd6d66469c6ef73ab33dcb3417defba200cb95673` |
| `zircon_runtime_interface/src/runtime_api`完整V7合同 | **11 / 2,251 / 2,033 / 68,849 / 7 / 0** | API table、session/event/viewport/frame demand/host request/plugin event/world/operation固定布局；fingerprint `83cea40ac89ddc31f650418f22d14d2e5019b7129b41e846256f2d26743acc53` |
| App runtime library、frame loop与host request | **26 / 5,480 / 4,958 / 196,084 / 90 / 0** | loader、session wrapper、foreign output、operation、wake、teardown、frame cadence与IME/cursor routing；fingerprint `19442d57abe26b2db45b0e0682fa62e2b0701a79d439c6a452b6e61037dd091a` |
| Editor core gateway完整子树 | **21 / 2,934 / 2,635 / 95,550 / 11 / 0** | copied table、manual capability、frame/viewport/world/plugin/operation/profile/overlay与in-process分叉；fingerprint `ca15222141aba22f3115469c8c2ceb0afa0260d12037f12ae7ec94f0ab82d9ea` |
| shader prewarm CLI聚焦consumer | **3 / 1,692 / 1,593 / 63,052 / 8 / 0** | manifest/material source/run的identity、fallback、quality与asset scan链；fingerprint `d068ae4ccecbce69d97ecdd51c4196b07a9fb562f4b876cd79b275b0e3c5e494` |
| 五引擎参考集 | **18 / 43,341 / 36,845 / 1,614,620 / 24 / 0** | Unreal 6、Bevy 4、Godot 3、Fyrox 3、Unity Graphics 2；fingerprint `84d5fbaff047ea1f51a5e8473400aff5ec5a1dc4ad72c5e1e57323bffd83ee63` |
| selected combined scope | **156 / 74,460 / 65,292 / 2,721,129 / 335 / 11** | 上述组按normalized path去重；fingerprint `6c7ef24cab91d3d5d1b4a9a0bc56b65be1da0093dab263ccf32109a5da8efaa5` |

指纹算法与Runtime43一致：对每个normalized relative path对应的文件取lowercase SHA-256，按path排序，以`path|hash`和LF连接且无末尾LF，再取整体SHA-256。测试数字是静态declaration计数，不表示已编译或通过。11项ignored由1项world tail performance、1项frame diagnostics和9项Vampire gameplay/HUD/menu组成。

该快照以`b6f4872cf421...`为审查HEAD并包含共享工作树中的未提交变化；最终验证时HEAD已由其他session推进到`457b304730e1...`，但selected combined fingerprint保持不变。Dynamic目录当前有77个文件，其中`physical_input_ownership.rs`是其他active session新增的测试。本报告只读取并如实冻结它，不取得其源码所有权，也不把未集成测试冒充已通过证据。后续实施必须重新取fingerprint。

### 2.2 检查方法

本轮沿`get_api -> host validation -> create/linked config -> composition/project startup -> global registry publish -> action admission -> event/input/UI -> tick/demand -> bind/present/capture/extract -> host/plugin/world/operation output -> foreign allocation -> destroy/unload -> shader CLI/prewarm/cache`逐段阅读，并反向检查App与Editor两套consumer。每段分别核对identity、owner、generation、capability truth、thread/lock、admission、ordering、transaction、failure disposition、wake、teardown、observability、product reachability和规模复杂度。

`audit_runtime_structure.py --json`还对当前源码执行了专项动态边界审计：60/60 source、12/12 tables、全部`repr(C)`和字段布局、23/23 operation wrappers、12/12 headless入口、9/9 panic boundary、12/12 loader failures及21/21 behavior anchors通过，未发现重复UI public type或结构风险。全局module convention审计仍报告其他crate的大文件/migration debt；本文不把无关债务复制进Runtime43，也明确不把结构门禁通过等价为语义工程化完成。

### 2.3 动态证据边界

本轮为review-only，只修改本报告与三个索引。没有运行Cargo、当前未提交Dynamic测试、真实DLL reload/unload、多session contention、hung callback、allocation flood、多viewport、多触点、Linux/macOS surface、百万world invalidation、GPU device loss、shader cold/hot或同硬件同负载benchmark。Runtime43两个tail-queue记录也明确把combined Cargo/release P95留为pending；ignored benchmark不能升级为资格证据。tooling按用户当前要求排除。

## 3. 必须保留的工程基础

1. 保留单一静态API table、入口panic containment、alignment/version/size/function validation，但后继版本改成append-compatible capability negotiation。
2. 保留session action admission与destroy quiescence屏障，扩展为有deadline、cancel、isolation和terminal census的状态机。
3. 保留foreign allocation的session owner验证、显式release、outstanding/high-water census，增加generation、age、kind和累计budget。
4. 保留bounded JSON的byte/depth/item/time防线，改为一次受预算decode并移出session owner锁。
5. 保留host/plugin/world分页方向以及当前无丢项/tail-commit修补，统一成sequence/cursor/remaining/dropped/resync/wake receipt。
6. 保留frame demand的Idle/Immediate/After和wake registration底座，由WakeArbiter聚合全部producer并由host ack消费。
7. 保留project prepare先于activation、Core composition compiler与project manager snapshot方向，升级为唯一immutable startup plan和rollback transaction。
8. 保留runtime retained UI的tree/layout/input/a11y共用模型与physical-first input修补，删除伪造fallback与产品专用HUD/menu。
9. 保留extract cache及clone/byte diagnostics，改为完整generation key和immutable arena/chunk lease。
10. 保留shader variant key、manifest、disk cache与真实WGPU validation，迁入正式shader build/cache/operation service并复用device/compiler pool。
11. 保留Runtime41 operation service作为唯一owner；dynamic/App/Editor只实现有wake、cancel、deadline和two-phase receipt的transport。
12. 保留App01作为最终process/DLL shutdown coordinator，dynamic session负责在unload前给出可证明terminal或isolated结果。

## 4. 当前链路事实与工程裁决

| 链路 | 当前源码事实 | 工程裁决 |
|---|---|---|
| API/Host | V7固定25个字段；Host V1只有ABI、size、diagnostic callback、resource fetch；loader要求exact size | 没有capability、allocator、task/thread、clock、platform、logging/lifecycle negotiation；下一版需hard cutover或明确append规则 |
| Session config | V3只含profile、project root、play scene、report pipe、wake sink | 没有BuildSet、host identity、budget、executor、platform provider、principal或startup receipt target |
| Registry/identity | 进程全局session/allocation `OnceLock<Mutex<HashMap>>`；raw monotonic `u64` | 无host scope、slot generation、DLL epoch、capacity、ABA/exhaustion政策 |
| Action/locking | 23个wrapper都通过session action guard，但业务整体持有session mutex | 结构集中但慢decode/query/capture/GPU/UI会互相阻塞，也无法隔离不同read lane |
| Destroy | 等待action和callback无期限；allocation阻止销毁重试；App failure会abort | 缺ShutdownBudget、task/watch/viewport census、quarantine与host-safe terminal receipt |
| Allocation | 有owner、outstanding/high-water count/bytes | 没有count/bytes admission、age/kind/epoch、pressure、revoke/quarantine；只能诊断一部分事实 |
| Composition/startup | 第二套profile；linked availability可自动选择；script merge仍可强塞builtin；模块/project串行激活 | 未消费单一composition hash；无startup transaction、rollback和报告ack；Headless/Minimal闭包不可信 |
| Project | play scene先resolve后物理重读；固定navmesh；空startup list等于全加载 | 绕过VFS/cook artifact provenance，存在TOCTOU、隐式工作量和部分发布 |
| Viewport/surface | 一个`ActiveViewport`、default handle、Win32-only；resize destroy/recreate | 没有多窗口/平台/generation、surface lease、in-flight retire或device-lost disposition |
| Frame/capture | 无render/pipeline not-ready时可返回黑RGBA；bind/present可成功 | false success继续污染产品资格；必须区分Unavailable/NotReady/Stale/DeviceLost |
| Extract | key只有world/visibility/camera/viewport；hit与rebuild都clone；stats仍记`full_clones=1` | cache correctness与steady allocation都未闭合；logical bytes不是resident allocator truth |
| Event/input | 一个wide struct；default viewport；有限logical key；repeat=false；touch映射单mouse；部分未知值Ok | 不能支持多设备/用户/窗口、layout、replay、multi-touch、stale rejection或route receipt |
| Host request | IME text已做32 KiB和UTF-8 boundary remap；每类分页无丢项 | 分类队列仍重排跨类因果；没有统一sequence/request ID/ack/deadline；stale manager仍可Empty |
| Runtime UI | 全asset registry扫描；alias error被忽略；顺序surface ID；48-bit local mask；last scale wins；无hot reload | 启动成本、identity、冲突、multi-surface composition与state migration均不合格 |
| Accessibility/product | 无UI返回伪preview tree；HUD/menu内置Vampire组件、文案和点击写World | 引擎制造不存在产品状态并携带示例ABI，必须迁App06/plugin/assets |
| Plugin event | page含sequence/remaining/age且descriptor预编码；subscription仍裸ID/poll-only | 无capacity/generation/wake/drop-resync；App丢backlog metadata，Editor与App语义分裂 |
| World sync | tail truncate消除头删；测试与ignored perf存在 | 无page envelope/cursor/remaining/dropped/resync/wake；单大项和锁内重复encode仍可阻塞 |
| Operation adapter | 复用Runtime41 submit/poll/harvest | V7无cancel；submit不wake；adapter仍忽略harvest commit failure；销毁不取得operation fence |
| Shader prewarm | 固定pass族、batch-local cache、同步WGPU validation；CLI支持asset scan/quality参数 | identity仍literal；每类validation新建offscreen backend；无真实PSO usage、共享device、cancel/progress/shutdown |
| Consumers | App和Editor分别包装同一V7；Editor复制table并手工注入capability；linked path绕FFI | 合同语义、validation和错误政策重复且分叉；surface state只是单AtomicBool，不是per viewport |
| Tests | 195项Dynamic declaration，含无丢项分页、tail queue和physical-first test | 11 ignored；缺多host/ABA/pressure/timeout/platform/device loss/resync/replay/soak与release证据 |

## 5. 对Runtime43旧结论的纠正

| 旧结论 | 当前裁决 |
|---|---|
| allocation只有单payload上限且无累计事实 | 部分过时；已有outstanding/high-water count/bytes census，但仍无admission、age、kind、epoch与pressure policy |
| host request分页可能丢失页尾 | 当前focused test证明`max + 1`会形成`[max, 1]`且无丢项；跨类别顺序、sequence和ack仍未解决 |
| plugin event页没有remaining/backlog信息 | 部分过时；当前页含remaining、oldest age和sequence，仍不wake，App consumer还丢弃metadata |
| world invalidation commit逐项`remove(0)` | 已过时；两个Runtime43子记录和当前源码改为尾部`truncate`，但resync/receipt/单项预算与release P95仍缺 |
| extract每次都重复扫描bytes | 部分过时；byte estimate随rebuild缓存，hit仍完整clone且estimate不是allocator truth |
| UI消费会在physical state提交前早退 | 当前未提交test与source变化开始锁定physical-first事实；仍无InputRouter context、route receipt、timestamp/device identity与replay |
| shader identity、fixed pass、新backend和同步执行 | 保留；targeted current-source scan逐项仍成立 |
| 全局registry、裸handle、单mutex、无限destroy、黑帧/伪tree、产品HUD/menu | 保留；当前控制流与测试仍直接证明 |

## 6. 参考实现给出的边界

### 6.1 Unreal

`FEngineLoop`把PreInit、Init、Tick与Exit拆成显式阶段；`FWorldContext`携带PIE/current world/travel state；`FModuleManager`区分Unload、Abandon和shutdown ordering。Slate维护input preprocessors、users、focus与accessibility。`FShaderPipelineCache`提供Background/Fast/Precompile batch mode、Pause/Resume、batch size/time、stats与save。Zircon不需要照搬宏或全局对象，但session/world、窗口输入、module和shader work必须拥有独立state、budget和terminal receipt。

### 6.2 Bevy

Bevy `App`拥有唯一runner，plugin经历Adding/Ready/Finished/Cleaned；`SubApp`拥有独立World、schedule与extract function；schedule runner明确区分Once、Loop和wait duration。它反证了把headless/windowed runner、render extract和plugin finalize都藏入一个session mutex与profile字符串的做法。

### 6.3 Godot

Godot `Main`执行setup/setup2/start/iteration/cleanup，extension按InitializationLevel初始化并逆序反初始化；`SceneTree`分开physics/process、timer、tween与frame signal；GDExtension manager区分load/reload/unload。Zircon至少需要同等级别的session phase、world tick domain、extension generation和reload terminal，并额外处理跨DLL budget/receipt。

### 6.4 Fyrox

Fyrox Executor拥有event loop、graphics context、plugin lifecycle和fixed-update lag；Plugin contract覆盖register、init、loaded、OS event、graphics context created/destroyed与deinit。window/event/render/plugin不是“某个同步FFI成功返回”这一种动作，Zircon需要typed lane和lifecycle receipt。

### 6.5 Unity Graphics

Unity Graphics RenderGraph显式维护record/compile/execute/resource cleanup，CompilationCache按hash复用compiled graph，管理bounded entry、oldest replacement、pool与error recovery。该镜像不代表完整Unity Player，但足以反证每次prewarm创建backend、batch-local cache和literal compiler identity不是成熟pipeline cache。

### 6.6 Zircon超越目标

目标不是继续扩张同步V7表，而是建立`RuntimeHostInstance -> generation SessionRegistry -> RuntimeSessionActor -> bounded control/event/frame/read/job lanes`。所有异步producer进入coalesced WakeArbiter；render/UI/world/plugin/operation输出带sequence、remaining、disposition、owner和release receipt；startup绑定composition/project/BuildSet hash；read侧消费immutable snapshot；shader prewarm复用正式device/compiler/cache。性能资格必须证明慢请求不阻塞其他session/read lane，steady frame不全量clone，backlog分页线性且prewarm冷/热可追踪。

## 7. P0唯一归属与依赖路由

本篇不新增P0。以下root blockers继续由canonical owner交付，Runtime131只定义dynamic adapter的联合责任：

| Canonical owner | 根阻断 | Runtime131联合责任 |
|---|---|---|
| Runtime Interface01/05 | ABI协商、FFI ownership、foreign output、budget/fuse与unload安全 | generation session/allocation、typed disposition、累计budget、callback terminal |
| Runtime Interface03 | input/UI/a11y/status公共合同 | event envelope、route receipt、authoritative empty/unavailable，删除伪tree/黑帧 |
| Runtime01/02 | lifecycle、task、cancel/deadline/shutdown | session actor、bounded lanes、slow decode/query/prewarm job和shutdown census |
| Runtime05 | World lifecycle、snapshot与mutation | generation-bound immutable read snapshot和transactional mutation ingress |
| Runtime07/42 | script/plugin generation与唯一composition | construction只消费一个frozen plan，不自动选择linked或重建script truth |
| Runtime09A/09C | device/surface/frame disposition与shader/PSO cache | multi-viewport/platform provider、typed frame result、shared-device prewarm |
| Runtime11A | UI identity/input/a11y/hot reload | stable surface/node generation、dependency closure、rebase transaction |
| Runtime22/24 | clock/replay与stable identity | event/session/viewport/subscription/allocation统一clock、sequence、generation |
| Runtime41 | Operation control plane | dynamic/App/Editor暴露wake/cancel/deadline/two-phase result与shutdown fence |
| App01/06 | process shutdown和Vampire产品 | host在DLL unload前消费terminal receipt；产品HUD/menu完全迁出engine |
| Plugins05 | shader compiler/artifact/native provider | prewarm使用真实compiler/backend/device/driver/BuildSet identity |

## 8. Runtime43 P1当前状态

### 8.1 Session registry、identity、lifecycle与foreign ownership

| ID | 状态 | 当前证据与裁决 | 必须重构 |
|---|---|---|---|
| DYN-P1-001 | Open | session handle仍是递增裸`u64`，wrap后回0，无generation/host/DLL epoch | `RuntimeSessionId { slot, generation, owner, runtime_epoch }`及stale/foreign拒绝 |
| DYN-P1-002 | Open | session与allocation registry仍为进程全局`OnceLock<Mutex<HashMap>>` | `RuntimeHostInstance`拥有registry、runtime epoch、reset与unload census |
| DYN-P1-003 | Open | session数、创建速率与session resident memory无admission | host/project/profile census、rate和resource budget |
| DYN-P1-004 | Open | action guard存在，但完整业务仍持有单session mutex | owner executor与control/event/read/frame/job lanes，明确thread affinity |
| DYN-P1-005 | Open | destroy仍无限等action/wake callback；测试用blocking callback证明该政策 | deadline、cancel、progress、force-isolate与typed timeout terminal |
| DYN-P1-006 | Open | 外层无限等待，module shutdown drain仍为零预算 | 单一`ShutdownBudget`分配所有phase deadline并报告owner |
| DYN-P1-007 | Open | outstanding allocation仍阻止destroy并要求host retry | shutdown receipt列出generation/kind/age/bytes，并定义revoke/quarantine |
| DYN-P1-008 | Partial | 已有outstanding/high-water allocation count/bytes与focused tests；没有上限、age/kind/epoch/pressure | per-session/global arena admission、retention age、pressure diagnostic与policy |
| DYN-P1-009 | Open | wake callback同步直调，无coalesce/rate/deadline/slow-host隔离 | edge-triggered WakeArbiter经host dispatcher执行并记录latency/failure |
| DYN-P1-010 | Open | poison路径继续恢复并服务，测试仍把它锁成预期 | invariant fault隔离受影响generation，不允许继续mutation |
| DYN-P1-011 | Open | RuntimeLoop/viewport Drop忽略错误；App session Drop在destroy失败时abort | 显式close/receipt；Drop仅最后隔离并记录unresolved census |
| DYN-P1-012 | Open | session lifecycle测试继续保护process-abort补偿政策 | 返回HostSafetyViolation并隔离runtime generation，库不得杀宿主进程 |

### 8.2 Composition、Profile与project startup

| ID | 状态 | 当前证据与裁决 | 必须重构 |
|---|---|---|---|
| DYN-P1-013 | Open | Dynamic仍定义第二套六profile与字符串parse | 只接受Runtime42 typed profile/composition ID与hash |
| DYN-P1-014 | Open | empty profile bytes仍静默等于Runtime | host显式profile；兼容默认只在产品配置层解析并写receipt |
| DYN-P1-015 | Open | dynamic profile仍固定`max_fixed_steps_per_frame = 8` | Runtime22 ClockDomain/Profile policy及overrun/determinism receipt |
| DYN-P1-016 | Open | Minimal/Headless仍映射ClientRuntime，缺capability closure；builtin script仍可被强制加入 | capability solver唯一决定module/system闭包 |
| DYN-P1-017 | Open | linked registration absent于manifest时仍可作为自动启用候选 | provider只声明availability，effective plan显式选择 |
| DYN-P1-018 | Partial | 当前merge避免重复linked phase，但仍可强制builtin并重构system rows | construction直接消费冻结plan的resolved rows，不二次解析或补默认 |
| DYN-P1-019 | Open | module/project/UI/operation依次激活，失败无统一transaction/receipt | prepare/stage/validate/commit/publish/rollback startup transaction |
| DYN-P1-020 | Open | `play_report_pipe`校验后仍被丢弃 | 删除合同或绑定host-owned sink、request/session与delivery ack |
| DYN-P1-021 | Open | play scene prepare后activation重新读物理文件并创建World | VFS/cooked artifact snapshot与content hash关闭TOCTOU |
| DYN-P1-022 | Open | navmesh路径固定；空startup script列表仍隐式加载全部 | manifest closure、dependency order、并发budget、failure/rollback receipt |

### 8.3 Frame、surface、extract与demand

| ID | 状态 | 当前证据与裁决 | 必须重构 |
|---|---|---|---|
| DYN-P1-023 | Open | Headless/Minimal bind/present可成功，capture可返回黑RGBA | `Unavailable(NoRenderCapability)`或显式software/test provider |
| DYN-P1-024 | Open | pipelined尚无completed frame时仍可返回黑RGBA | `NotReady { submitted, completed, next_wake }`或last valid generation |
| DYN-P1-025 | Open | 生产路径仍只有default viewport与单`ActiveViewport` | session-owned viewport registry、generation及per-view state |
| DYN-P1-026 | Open | native surface仍只支持Win32 | platform tagged descriptor、provider capability与Windows/Linux/macOS/headless矩阵 |
| DYN-P1-027 | Open | resize仍destroy/recreate并重置capture generation | 保留逻辑identity，迁移surface/device generation并retire in-flight frame |
| DYN-P1-028 | Open | key只有world tick、visibility、active camera、viewport | asset/material/plugin/render config/device/UI generation或dependency snapshot |
| DYN-P1-029 | Open | cache hit返回`entry.extract.clone()`，rebuild还会保存和返回两份 | immutable snapshot、arena/chunk reuse或Arc lease，steady state零全量clone |
| DYN-P1-030 | Partial | rebuild会缓存byte estimate，避免hit重复扫描；仍遗漏capacity/nested residency且`full_clones=1` | tagged allocator/arena提供resident/peak/retained truth，logical payload另列 |
| DYN-P1-031 | Open | demand仍只观察asset reload与animation | operation/UI timer/network/task/plugin/world/render completion全部接WakeArbiter |
| DYN-P1-032 | Open | tick clear/失败语义仍不能证明并发producer request不会丢 | generation/atomic merge且只在host ack后消费 |
| DYN-P1-033 | Open | background/suspended只映射FocusLost，其他lifecycle状态被忽略 | session/viewport lifecycle machine按owner暂停、恢复、重建设备并出receipt |

### 8.4 Event、input与host request

| ID | 状态 | 当前证据与裁决 | 必须重构 |
|---|---|---|---|
| DYN-P1-034 | Open | event仍无host timestamp、sequence、device/user和window generation | V2 envelope含clock domain、sequence、qualified source和viewport generation |
| DYN-P1-035 | Open | Runtime UI metadata sequence继续`saturating_add`，溢出后重复 | generation rollover或typed exhaustion，明确replay/dedup合同 |
| DYN-P1-036 | Open | logical key只覆盖Shift/Control/Alt、0-9、WASD；repeat=false | 完整physical/logical/text/repeat/modifier/layout contract |
| DYN-P1-037 | Open | unknown keyboard action仍可Ok；gamepad/value/range admission不完整 | 所有enum和float先admit，unknown返回typed unsupported/invalid并计数 |
| DYN-P1-038 | Open | touch contact仍共享cursor和mouse-left camera drag | per-contact state、primary pointer、gesture arena、capture/cancel与multi-touch route |
| DYN-P1-039 | Partial | 当前source/test开始保证physical press/release先进入InputManager，再允许UI stop propagation；仍无context/route receipt/replay | InputRouter按surface/focus/context/priority路由并发布consumed-by/fallback/capture结果 |
| DYN-P1-040 | Open | IME/rumble/cursor仍分组drain后拼接；分页无丢项不解决跨类顺序 | unified ordered request queue、sequence、causal parent、deadline和ack |
| DYN-P1-041 | Open | manager generation失效时仍可返回空成功 | `ManagerGenerationStale/Unavailable`，不得伪装Empty |
| DYN-P1-042 | Open | FFI同步处理event，无ingress capacity/drop/coalesce receipt | bounded event ingress、per-kind coalescing与accepted/dropped sequence receipt |

### 8.5 Runtime UI与产品污染

| ID | 状态 | 当前证据与裁决 | 必须重构 |
|---|---|---|---|
| DYN-P1-043 | Open | startup仍扫描asset registry中的全部UI文件 | 从declared root解析dependency closure，复用asset index/DDC并预算预取 |
| DYN-P1-044 | Open | `let _ = builder.insert_with_aliases(...)`仍吞alias/schema conflict | conflict使startup失败或进入显式degraded receipt |
| DYN-P1-045 | Open | surface ID仍由manifest/root数组index推导 | asset ID + declared surface ID + generation，重排不改identity |
| DYN-P1-046 | Open | global node ID仍使用surface高16位并mask local低48位 | structured `UiNodeHandle { surface, local, generation }`并拒绝overflow |
| DYN-P1-047 | Open | commands拼接后仍只有last surface raster scale | per-surface scale/viewport/layer/clip和显式compositor order |
| DYN-P1-048 | Open | `RuntimeUiSurfaceSet`仍没有asset hot reload/rebase | prepare tree、state/focus migration、atomic publish、LKG rollback |
| DYN-P1-049 | Open | 无UI时preview仍构造假的“Zircon Runtime Preview”a11y tree | authoritative Empty/Unavailable，不制造节点 |
| DYN-P1-050 | Open | project UI存在时整体隐藏legacy HUD/menu，不是layer composition | 删除legacy branch，以capability/layer组合project/debug/overlay |
| DYN-P1-051 | Open | engine仍含Vampire组件、文案、颜色、布局与dynamic component写入 | 全迁App06/plugin/assets/actions，runtime只保留通用UI/command contract |

### 8.6 Plugin event、world sync与operation适配

| ID | 状态 | 当前证据与裁决 | 必须重构 |
|---|---|---|---|
| DYN-P1-052 | Open | subscription仍为裸递增`u64` HashMap，无generation/capacity | generation handle、subscription/filter budget和owner teardown |
| DYN-P1-053 | Open | delivery继续把raw session handle当`play_session_id` | stable play/world context identity，禁止泄漏registry slot |
| DYN-P1-054 | Partial | plugin page现有sequence、remaining、oldest age并减少重复encode；仍poll-only、不wake，App丢metadata | backlog 0->nonzero coalesced wake，所有consumer保留page receipt |
| DYN-P1-055 | Open | empty plugin page仍返回zero-byte output | typed page header/disposition区分Empty/CursorExpired/OwnerGone/More |
| DYN-P1-056 | Open | world query/watch/drain仍在session mutex同步decode/execute/encode | immutable snapshot read lane、watch admission/cost/generation/initial cursor |
| DYN-P1-057 | Partial | commit已用tail `truncate`消除`remove(0)`；有focused test和ignored perf；仍无receipt/resync且试探encode | deque/segments + one-pass sizing，page携cursor/remaining/dropped/resync |
| DYN-P1-058 | Open | operation submit仍不wake；adapter仍可能发布output后忽略commit失败 | Runtime41 request/wake/cancel/deadline与严格two-phase harvest |

### 8.7 Bounded JSON与shader prewarm

| ID | 状态 | 当前证据与裁决 | 必须重构 |
|---|---|---|---|
| DYN-P1-059 | Open | nesting、syntax graph、typed deserialize和business traversal仍是多pass | single budgeted stream/arena decode或复用validation result，移出session锁 |
| DYN-P1-060 | Open | wall clock只在chunk/closure边界检查，长encode/item closure不可中断 | cooperative work units、CPU budget、cancel token和test clock |
| DYN-P1-061 | Open | prewarm仍公开在`dynamic_api`模块外侧，不属于V7/session lifecycle | 迁Runtime09C/Plugins05 service，dynamic只暴露正式async operation或不暴露 |
| DYN-P1-062 | Open | CLI仍literal Naga/WGPU/platform；fallback仍可人工加入，identity不含driver/BuildSet | 实际compiler/backend/device/driver/target identity；source/template error原样失败 |
| DYN-P1-063 | Open | builtin manifest仍是固定pass族/default Medium，未从真实scene/material/target usage闭包生成 | cook收集真实variant/PSO usage、quality/platform/provider closure |
| DYN-P1-064 | Open | WGPU validation每类路径调用`new_offscreen()`；cache batch-local、同步、无cancel/progress | shared device/compiler pool、cross-job artifact cache、bounded operation和shutdown drain |

P1连续编号检查：001-064共64项；本轮Partial为008、018、030、039、054、057，共6项，其余58项Open，无Closed。

## 9. Runtime43 P2当前状态

| ID | 状态 | 当前证据与必须改进 |
|---|---|---|
| DYN-P2-001 | Open | status仍使用thread-local 4096-byte借用diagnostic buffer；后继ABI改owned/caller buffer并报告truncation |
| DYN-P2-002 | Open | wide event struct继续复用`x/y/delta/button/state/key/scan`；改tagged payload union与per-kind size |
| DYN-P2-003 | Open | camera controller仍硬编码到dynamic session；迁可选viewport/controller plugin policy |
| DYN-P2-004 | Open | camera left-button仍有无动作分支；删除或赋予明确selection/capture owner |
| DYN-P2-005 | Open | camera transform write failure仍被吞；进入input/render diagnostic与route outcome |
| DYN-P2-006 | Open | scene reload/count diagnostic仍把整数写为`f64`；diagnostic store提供u64 counter |
| DYN-P2-007 | Open | project identity仍依赖display name字符串；改project UUID + manifest generation + BuildSet |
| DYN-P2-008 | Open | profile与failure仍大量使用自由字符串；统一stable code/stage/owner/correlation/remediation |
| DYN-P2-009 | Open | invalid UTF-8在部分event/file/gamepad路径降为None；分离reject/raw identity/lossy display |
| DYN-P2-010 | Open | empty plugin delivery仍以zero-byte buffer表达；迁typed Empty page |
| DYN-P2-011 | Open | structure tests仍大量`include_str!`匹配源码；迁behavior、trait contract和compile-fail |
| DYN-P2-012 | Open | Dynamic现有11项ignored：9项Vampire、1项frame diagnostics、1项world perf；移交产品owner并建立可运行release资格 |
| DYN-P2-013 | Open | default viewport handle `1`仍散布App/dynamic compatibility path；只保留host创建的qualified viewport |
| DYN-P2-014 | Open | preview/a11y仍声称dynamic preview但无真实surface；使用typed reason，删除误导文本 |
| DYN-P2-015 | Open | startup日志仍缺duration/input hash/terminal outcome；接入structured startup trace |
| DYN-P2-016 | Open | alpha cutoff fallback仍不是material schema唯一真相；默认值与source provenance进入variant key |

P2连续编号检查：001-016共16项，全部Open。

## 10. 资格门当前状态

| Gate | 状态 | 当前证据与未闭环 |
|---|---|---|
| DYN-GATE-001 | Fail | 无slot generation，不能证明slot复用后stale handle被拒绝 |
| DYN-GATE-002 | Fail | 无`RuntimeHostInstance` scope，跨host session/allocation/subscription隔离不存在 |
| DYN-GATE-003 | Fail | session/watch/subscription无容量；allocation有census但无admission |
| DYN-GATE-004 | Fail | 单session mutex，未测慢query/capture对tick/read SLO |
| DYN-GATE-005 | Fail | destroy仍可被hung action/wake无限阻塞 |
| DYN-GATE-006 | Fail | shutdown仍混用无限等待和零时长module drain |
| DYN-GATE-007 | Partial | foreign output已有owner、bytes、count和high-water；缺age/kind/generation/revoke/quarantine |
| DYN-GATE-008 | Fail | poison继续服务且App teardown仍可abort宿主 |
| DYN-GATE-009 | Fail | Dynamic未消费与App/Core完全一致的composition hash/receipt |
| DYN-GATE-010 | Fail | Headless/Minimal未证明render/script能力闭包 |
| DYN-GATE-011 | Fail | 未选择的linked registration仍可能注入 |
| DYN-GATE-012 | Fail | scene prepare与activate未绑定同一content hash |
| DYN-GATE-013 | Fail | navmesh/scripts/UI root未全部来自VFS/cooked closure |
| DYN-GATE-014 | Fail | startup没有跨阶段rollback transaction与无部分发布证明 |
| DYN-GATE-015 | Fail | `play_report_pipe`仍无交付/ack且合同未删除 |
| DYN-GATE-016 | Fail | fixed-step/overrun未由Runtime22唯一配置与重放 |
| DYN-GATE-017 | Fail | 无render capability仍可capture/present成功 |
| DYN-GATE-018 | Fail | pipeline pending、device lost、surface stale没有不同typed disposition |
| DYN-GATE-019 | Fail | 同session四viewport不存在 |
| DYN-GATE-020 | Fail | 只有Win32 surface，缺Linux/macOS/headless provider矩阵 |
| DYN-GATE-021 | Fail | resize仍销毁逻辑viewport且无in-flight retire receipt |
| DYN-GATE-022 | Fail | extract key未覆盖asset/material/plugin/render/device/UI generations |
| DYN-GATE-023 | Fail | current stats明确每次交付`full_clones=1`，无steady allocator资格 |
| DYN-GATE-024 | Fail | frame demand没有operation/UI timer/network/task/plugin/world/render producers |
| DYN-GATE-025 | Fail | suspend/resume/low-memory/device restore无owner ordered receipt |
| DYN-GATE-026 | Fail | event无timestamp/sequence，无法做确定性replay |
| DYN-GATE-027 | Fail | 无qualified device/user/window generation和stale event拒绝 |
| DYN-GATE-028 | Fail | touch仍退化为单mouse state |
| DYN-GATE-029 | Partial | physical-first press/release source/test是局部正确性底座；route receipt、focus/context replay仍不存在 |
| DYN-GATE-030 | Fail | host request无跨IME/rumble/cursor全局sequence/causal order |
| DYN-GATE-031 | Fail | event burst的coalesce/drop/backpressure无host-visible counters/receipt |
| DYN-GATE-032 | Fail | UI startup仍随项目全部UI资产线性扫描 |
| DYN-GATE-033 | Fail | alias错误仍被忽略且surface reorder改变identity |
| DYN-GATE-034 | Fail | multi-surface仍last scale wins且node local ID被mask |
| DYN-GATE-035 | Fail | UI没有hot reload state/focus migration与LKG rollback |
| DYN-GATE-036 | Fail | preview伪tree和Vampire/HUD/menu产品特判仍在engine |
| DYN-GATE-037 | Partial | plugin page已有sequence/remaining/age；0->nonzero wake和一致consumer metadata仍缺 |
| DYN-GATE-038 | Partial | tail truncate消除了`remove(0)`；百万页release evidence、remaining/cursor/drop/resync仍缺 |
| DYN-GATE-039 | Fail | world query/encode不可取消且持session主锁 |
| DYN-GATE-040 | Fail | dynamic adapter不能证明commit成功后才发布，submit也不wake |
| DYN-GATE-041 | Fail | manifest不覆盖真实产品/平台PSO，identity不含实际driver/BuildSet |
| DYN-GATE-042 | Fail | prewarm无shared device/cache、cancel/progress/shutdown/device-loss矩阵 |

Gate连续编号检查：001-042共42项；Partial为007、029、037、038，共4项，其余38项Fail，无Pass。

## 11. 目标架构

```text
Host Process / Editor / Product
        |
        v
RuntimeHostInstance
  { runtime_epoch, BuildSet, capabilities, dispatcher, clock, budgets }
        |
        +--> SessionRegistry [slot + generation + owner]
        |         |
        |         v
        |   RuntimeSessionActor
        |   Created -> Preparing -> Active -> Quiescing -> Draining -> Terminal/Isolated
        |      |          |          |             |
        |      |          |          |             +--> ShutdownReceipt / LeakCensus
        |      |          |          +--> immutable Snapshot Read Lane
        |      |          +--> bounded Control / Event / Frame / Job lanes
        |      +--> RuntimeCompositionPlan + ProjectStartupPlan
        |
        +--> WakeArbiter [edge-triggered, coalesced, observable]
        +--> ForeignAllocationArena [owner/generation/kind/count/bytes/age]
        +--> ViewportRegistry [platform surface lease + generation + disposition]
        +--> OutputStreams [sequence/cursor/remaining/dropped/resync]
        +--> OperationService [cancel/deadline/progress/two-phase harvest]
        +--> ShaderBuildCache [compiler/backend/device/driver/BuildSet identity]
```

最小合同必须包含：

```text
RuntimeSessionId { slot, generation, owner, runtime_epoch }
ViewportId { slot, generation, session }
RuntimeRequestHeader { request_id, session, sequence, deadline, budget, principal }
RuntimeDisposition { Accepted, Pending, Ready, Empty, NotReady, Unavailable, Stale, Rejected }
RuntimePageReceipt { cursor, first_sequence, last_sequence, remaining, dropped, resync }
SessionStartupReceipt { composition_hash, project_hash, activated, degraded, rollback }
SessionShutdownReceipt { actions, callbacks, allocations, jobs, watches, subscriptions, viewports, terminal }
ShaderBuildIdentity { BuildSet, compiler, backend, device, driver, target, quality }
```

不能用`RwLock`替换`Mutex`来伪造并发。World、render bridge和UI mutation仍应由明确owner串行commit；并发来自不同session、有界ingress、异步prepare/encode、不可变snapshot和独立completion lane。每条lane都必须有capacity、deadline、cancel、shutdown owner和pressure telemetry。

## 12. 分阶段重构计划

### M0 · False-success止血与合同冻结

1. 为black capture、fake a11y、stale-manager Empty、ignored harvest commit和App abort建立RED测试。
2. 冻结V8候选的session/viewport/subscription/allocation identity、disposition、page receipt与capability negotiation。
3. 建立V7 App/Editor/linked/DLL consumer deletion matrix，禁止在旧表内继续语义扩张。

### M1 · Host instance、registry与lifecycle

1. 引入`RuntimeHostInstance`和generation slot registry，删除业务对进程全局map的依赖。
2. 建立session/allocation/watch/subscription admission、ShutdownBudget和terminal receipt。
3. wake进入host-dispatched arbiter，slow/hung callback不再冻结owner。

### M2 · 唯一composition与startup transaction

1. Dynamic construction只消费Runtime42冻结的composition rows/hash。
2. scene/nav/scripts/UI通过VFS/cooked artifact snapshot形成`ProjectStartupPlan`。
3. prepare/stage/commit/publish失败逆序rollback，report sink必须ack。

### M3 · Event、input、demand与host request

1. 新event envelope加入clock、sequence、device/user/viewport generation和source。
2. physical state、semantic routing、focus/capture/gesture分层，输出route receipt。
3. 统一有序host request lane和WakeArbiter，宿主ack后消费demand。

### M4 · Viewport、frame与extract

1. 多viewport registry和跨平台surface provider替代default/Win32特例。
2. Ready/NotReady/Unavailable/Stale/DeviceLost替代黑帧成功。
3. 完整generation dependency与immutable arena lease替代steady deep clone。

### M5 · UI、plugin event与world sync

1. UI只解析reachable closure，使用stable generation identity并支持hot reload transaction。
2. 删除engine内HUD/menu/Vampire和preview伪tree，改项目plugin/asset/action。
3. plugin/world统一sequence/cursor/remaining/drop/resync/wake，world读走snapshot lane。

### M6 · Operation与bounded work

1. 对接Runtime41 cancel/deadline/progress/wake/two-phase harvest/shutdown fence。
2. JSON decode/query/encode变成cooperative budgeted jobs，禁止长任务持session锁。
3. 建立per-session CPU、queue、output、watch和callback SLO/pressure telemetry。

### M7 · Shader prewarm归位与产品资格

1. prewarm迁正式shader build/cache service，identity绑定实际compiler/backend/device/driver/BuildSet。
2. cook汇总真实scene/material/PSO usage，runtime复用device/compiler pool异步预热。
3. 关闭cold/hot、cache corruption、cross-build rejection、cancel、shutdown和device-loss矩阵。

## 13. 测试与证据矩阵

| 层级 | 必需证据 |
|---|---|
| Contract | V8 layout、append/hard-cutover policy、capability negotiation、tagged event、disposition/page receipt、C/C++ consumer |
| Identity | slot复用、ABA、跨host、跨DLL epoch、stale viewport/subscription/allocation、ID exhaustion |
| Concurrency | 多session、slow callback/query、event burst、destroy/release race、wake coalesce、poison isolation |
| Startup | composition/project hash、TOCTOU、partial failure rollback、report ack、script/nav/UI closure |
| Rendering | headless unavailable、pipelined not-ready、4 viewport、resize race、device loss/restore、多平台surface |
| Performance | extract allocations/copies、mutex hold、queue latency、world page complexity、UI closure scaling、prewarm cold/hot |
| Input/UI | timestamp replay、多设备/用户、multi-touch、focus/capture/IME ordering、multi-scale、a11y authority、hot reload |
| Event/world/operation | wake edge、cursor expiry、remaining/drop/resync、cancel/deadline、two-phase harvest、terminal drain |
| Shader | real compiler/backend/device/driver key、scene PSO usage、shared device、cancel/progress、cache corruption |
| Product | App06无engine特判；App01在DLL unload前证明session/job/callback/allocation/watch/subscription/viewport terminal |

## 14. 逐文件审查台账

### 14.1 Dynamic API根、frame与prewarm

| 文件 | 当前裁决 / 后续owner |
|---|---|
| `zircon_runtime/src/dynamic_api/mod.rs` | V7/session/frame与prewarm公开面混杂；prewarm迁Runtime09C/Plugins05 service |
| `zircon_runtime/src/dynamic_api/exports.rs` | 静态表与panic wrapper保留；Host V1缺capability/thread/allocator/platform/lifecycle协商 |
| `zircon_runtime/src/dynamic_api/bounded_json.rs` | byte/depth/item/time防线保留；多pass与非合作closure迁budgeted job |
| `zircon_runtime/src/dynamic_api/camera_controller.rs` | 硬编码orbit/pan和吞transform error迁viewport/controller policy |
| `zircon_runtime/src/dynamic_api/frame.rs` | owned bounded RGBA与精确item count可保留；frame结果需要typed disposition |
| `zircon_runtime/src/dynamic_api/runtime_loop.rs` | 单`ActiveViewport`和同步runner应消费统一session demand/receipt |
| `zircon_runtime/src/dynamic_api/session.rs` | 模块边界清楚，但session仍聚合过多owner并形成单锁 |
| `zircon_runtime/src/dynamic_api/surface.rs` | Win32-only翻译迁platform surface provider/capability |
| `zircon_runtime/src/dynamic_api/shader_prewarm.rs` | 固定pass/template与batch-local执行不是产品prewarm service |
| `zircon_runtime/src/dynamic_api/shader_prewarm/execution_budget.rs` | worker/source byte budget是基础；未接共享scheduler/cancel/deadline |
| `zircon_runtime/src/dynamic_api/shader_prewarm/module_validation_cache.rs` | 单批RefCell去重可保留；缺跨job/build/device共享与并发owner |
| `zircon_runtime/src/dynamic_api/shader_prewarm/wgpu_validation.rs` | 真实WGPU compile/pipeline验证有价值；六条路径分别新建offscreen backend |
| `zircon_runtime/src/dynamic_api/shader_prewarm/tests.rs` | 覆盖pass/source/disk hit；缺真实PSO、driver skew、cancel、device loss与shutdown |

### 14.2 Session construction、state与适配模块

| 文件 | 当前裁决 / 后续owner |
|---|---|
| `zircon_runtime/src/dynamic_api/session/construction.rs` | construction更少重复script phase，但仍无单一composition/startup transaction |
| `zircon_runtime/src/dynamic_api/session/diagnostics.rs` | runtime/render/reload汇总保留；补session generation、queue/lock/budget和lifecycle receipt |
| `zircon_runtime/src/dynamic_api/session/error.rs` | typed source方向保留；自由字符串与abort policy收敛stable failure contract |
| `zircon_runtime/src/dynamic_api/session/event_mirror.rs` | remaining/age、descriptor预编码和坏payload隔离是真进展；裸ID、poll-only、App metadata loss仍在 |
| `zircon_runtime/src/dynamic_api/session/events.rs` | UI-first与physical-first修补并存；default viewport、宽struct、touch/lifecycle/product route仍不完整 |
| `zircon_runtime/src/dynamic_api/session/extract.rs` | 入口集中；实际交付仍完整clone |
| `zircon_runtime/src/dynamic_api/session/extract_cache.rs` | revision cache保留；key不全且hit/rebuild都clone |
| `zircon_runtime/src/dynamic_api/session/extract_stats.rs` | rebuild缓存byte scan与diagnostic anchors可保留；仍非allocator truth且每call记full clone |
| `zircon_runtime/src/dynamic_api/session/ffi.rs` | admission/pointer检查集中且structural audit通过；业务仍全在session锁内且有false success |
| `zircon_runtime/src/dynamic_api/session/highlight_set.rs` | latest-value提交集中；补viewport generation、capacity与receipt |
| `zircon_runtime/src/dynamic_api/session/host_requests.rs` | IME UTF-8边界和分页无丢项保留；统一全序queue与stale owner error |
| `zircon_runtime/src/dynamic_api/session/hud.rs` | sparse lookup可保留到generic adapter；Vampire检测和固定布局迁App06 |
| `zircon_runtime/src/dynamic_api/session/input_events.rs` | InputManager桥集中；键位、repeat、device、finite/range与layout不足 |
| `zircon_runtime/src/dynamic_api/session/linked_plugins.rs` | linked registration收集可保留；availability不能自动变selection |
| `zircon_runtime/src/dynamic_api/session/linked_session.rs` | in-process复用construction；必须与DLL共享host identity/generation/API语义 |
| `zircon_runtime/src/dynamic_api/session/menu.rs` | Vampire start/game-over/Blood Bolt/Retry与component write整体迁App06/plugin |
| `zircon_runtime/src/dynamic_api/session/operation.rs` | 复用Runtime41，但补wake/cancel/deadline并严格处理harvest commit |
| `zircon_runtime/src/dynamic_api/session/preview.rs` | 黑frame与伪a11y tree删除，改typed unavailable/empty |
| `zircon_runtime/src/dynamic_api/session/profile.rs` | 六profile/固定步重复定义；只消费Runtime42/22唯一合同 |
| `zircon_runtime/src/dynamic_api/session/project.rs` | prepare snapshot方向保留；消除物理重读、固定nav与隐式全script加载 |
| `zircon_runtime/src/dynamic_api/session/runtime_ui.rs` | retained surface/input/a11y链存在；全扫描、吞冲突、顺序ID、mask、last scale、无reload需重构 |
| `zircon_runtime/src/dynamic_api/session/scene_asset_reload_diagnostics.rs` | reload指标较完整；整数f64、静态store和缺artifact/generation/latency关联需修 |
| `zircon_runtime/src/dynamic_api/session/script_systems.rs` | linked phase dedup是局部进展；仍须直接消费冻结composition rows |
| `zircon_runtime/src/dynamic_api/session/state.rs` | God object与单锁核心；拆session actor、viewport/UI/world snapshot/job handles |
| `zircon_runtime/src/dynamic_api/session/status.rs` | status转换集中；thread-local borrowed diagnostic与自由字符串归Interface治理 |
| `zircon_runtime/src/dynamic_api/session/world_sync.rs` | tail `truncate`修补保留；加page receipt/resync、snapshot lane与one-pass sizing |

### 14.3 Session registry

| 文件 | 当前裁决 / 后续owner |
|---|---|
| `zircon_runtime/src/dynamic_api/session/registry/mod.rs` | owner拆分清楚；进程全局state迁HostInstance scope |
| `zircon_runtime/src/dynamic_api/session/registry/action_guard.rs` | RAII action exit保留；携request/session generation/deadline/cancel |
| `zircon_runtime/src/dynamic_api/session/registry/allocation_registry.rs` | owner与high-water census保留；补admission、age/kind/generation和quarantine |
| `zircon_runtime/src/dynamic_api/session/registry/frame_activity.rs` | reentrancy guard与demand carrier保留；不能替代callback dispatch/deadline |
| `zircon_runtime/src/dynamic_api/session/registry/frame_demand.rs` | demand merge方向保留；补全producer并用ack/generation消费 |
| `zircon_runtime/src/dynamic_api/session/registry/session_slot.rs` | action/callback census是quiescence基础；等待要有budget/state/terminal receipt |
| `zircon_runtime/src/dynamic_api/session/registry/session_store.rs` | create/get/remove/retry集中；raw ID、global scope、无capacity/timeout不合格 |
| `zircon_runtime/src/dynamic_api/session/registry/wake_registration.rs` | disable/callback census保留；同步callback改host dispatcher与coalescing |
| `zircon_runtime/src/dynamic_api/session/registry/tests.rs` | owner/destroy/wake/allocation覆盖强；同时固化poison recovery和无限等待政策，需改测试目标 |

### 14.4 Dynamic API边界测试

| 文件 | 当前裁决 / 缺失证据 |
|---|---|
| `zircon_runtime/src/dynamic_api/tests/mod.rs` | 测试分域组织可保留 |
| `zircon_runtime/src/dynamic_api/tests/support.rs` | 统一FFI fixture保留；增加host instance/generation/fault injection |
| `zircon_runtime/src/dynamic_api/tests/api_table.rs` | V7表/panic wrapper覆盖；缺capability negotiation/callback fault/version skew |
| `zircon_runtime/src/dynamic_api/tests/accessibility.rs` | ABI/viewport校验存在；不应继续接受伪preview snapshot |
| `zircon_runtime/src/dynamic_api/tests/frame_demand.rs` | wake pair/carrier覆盖；缺producer completeness、ack/race/lost demand |
| `zircon_runtime/src/dynamic_api/tests/host_request_payloads.rs` | IME/rumble/cursor encoding覆盖；缺跨类全序/request ack |
| `zircon_runtime/src/dynamic_api/tests/host_requests.rs` | 当前证明分页`[max,1]`无丢失；缺concurrent producer、global sequence、stale manager |
| `zircon_runtime/src/dynamic_api/tests/input_events.rs` | wheel/scale/IME边界覆盖；缺full keyboard、repeat、finite/range、multi-device/touch |
| `zircon_runtime/src/dynamic_api/tests/linked_plugins.rs` | linked event/Editor profile覆盖；缺effective selection isolation与wake |
| `zircon_runtime/src/dynamic_api/tests/operation.rs` | submit/poll/harvest/layout覆盖；缺wake/cancel/deadline/commit fault |
| `zircon_runtime/src/dynamic_api/tests/profile_control.rs` | bounded JSON/snapshot覆盖；缺slow decode cancel与session budget/concurrency |
| `zircon_runtime/src/dynamic_api/tests/session_entry_points.rs` | invalid/destroyed handle覆盖；缺slot reuse、cross-host和epoch |
| `zircon_runtime/src/dynamic_api/tests/session_lifecycle.rs` | bootstrap validation/quiescence存在；abort期望应改isolated receipt |
| `zircon_runtime/src/dynamic_api/tests/session_profiles.rs` | Headless bridge absence覆盖；仍未证明script closure和capture unavailable |
| `zircon_runtime/src/dynamic_api/tests/structure.rs` | source-shape guard可防漂移；不能替代运行时工程资格 |
| `zircon_runtime/src/dynamic_api/tests/viewport.rs` | default/Win32前置校验覆盖；缺multi-view/generation/resize/device/platform矩阵 |

### 14.5 Session集成测试

| 文件 | 当前裁决 / 缺失证据 |
|---|---|
| `zircon_runtime/src/dynamic_api/session/tests/mod.rs` | owner清楚；Vampire组应迁产品测试包 |
| `zircon_runtime/src/dynamic_api/session/tests/foundation_render.rs` | basic scene/PNG/steady stats是真实底座；扩multi-view/device loss/zero-clone threshold |
| `zircon_runtime/src/dynamic_api/session/tests/frame_demand.rs` | animation demand覆盖；补其余producer和failure retention |
| `zircon_runtime/src/dynamic_api/session/tests/frame_diagnostics.rs` | 直接证明cache hit仍一次full clone；performance证据仍ignored |
| `zircon_runtime/src/dynamic_api/session/tests/highlight_set.rs` | canonical latest value覆盖；补capacity/stale viewport/concurrent submit |
| `zircon_runtime/src/dynamic_api/session/tests/lock_poison.rs` | 当前要求poison后继续；改为generation isolation与diagnostic |
| `zircon_runtime/src/dynamic_api/session/tests/physical_input_ownership.rs` | 新RED/behavior目标正确：UI stop前提交physical release；当前未集成且不等于route/replay闭环 |
| `zircon_runtime/src/dynamic_api/session/tests/runtime_errors.rs` | typed source chain保留；扩stable code/stage/correlation |
| `zircon_runtime/src/dynamic_api/session/tests/runtime_ui_surface.rs` | root/import/action/a11y覆盖强；缺alias conflict、stable ID、multi-scale、hot reload |
| `zircon_runtime/src/dynamic_api/session/tests/vampire_gameplay.rs` | 4项真实VM测试ignored且产品规则不属于engine；迁App06/plugin |
| `zircon_runtime/src/dynamic_api/session/tests/vampire_hud.rs` | 3项ignored，证明engine与Vampire HUD耦合；迁产品owner |
| `zircon_runtime/src/dynamic_api/session/tests/vampire_menu.rs` | 2项ignored，锁定start/retry产品协议；迁产品UI测试 |
| `zircon_runtime/src/dynamic_api/session/tests/vampire_runtime_support.rs` | 大型产品fixture位于engine；整体迁App06或example plugin test package |

### 14.6 Runtime Interface固定合同

| 文件 | 当前裁决 / 后续owner |
|---|---|
| `zircon_runtime_interface/src/runtime_api/abi/api_table.rs` | 25-field V7 frozen table；exact-size政策妨碍append compatibility，且create/destroy/operation无cancel扩展 |
| `zircon_runtime_interface/src/runtime_api/constants.rs` | ABI/version/default viewport/event常量集中；默认viewport兼容alias应限时退役 |
| `zircon_runtime_interface/src/runtime_api/session/session.rs` | config V3信息不足；补BuildSet/host/budget/executor/platform/report sink |
| `zircon_runtime_interface/src/runtime_api/session/events.rs` | wide event V1改tagged envelope、timestamp/sequence/device/user/view generation |
| `zircon_runtime_interface/src/runtime_api/session/viewport.rs` | None/Win32 surface改platform provider descriptor与generation lease |
| `zircon_runtime_interface/src/runtime_api/frame/frame_demand.rs` | Idle/Immediate/After保留；增加ack/generation/producer reason |
| `zircon_runtime_interface/src/runtime_api/frame/highlight_set.rs` | bounded highlight payload存在；handle需绑定viewport generation/owner和submit receipt |
| `zircon_runtime_interface/src/runtime_api/host/host_requests.rs` | batch仅ABI+Vec且只IME/rumble/cursor；补global sequence/page/ack/causal parent |
| `zircon_runtime_interface/src/runtime_api/session/plugin_event_mirror.rs` | page remaining/age方向保留；补typed empty/cursor/drop/resync和wake contract |
| `zircon_runtime_interface/src/runtime_api/session/requests.rs` | profile/world/diagnostic等JSON请求固定布局集中；增加request header、budget/deadline和typed output disposition |
| `zircon_runtime_interface/src/runtime_api/session/operation.rs` | submit/poll/harvest V2保留；增加cancel/request V2/catalog/subscription/paged result |

实际目录文件名可能通过`mod.rs`内联多个类型，以上按当前11个physical owner归档；实施前由Interface01/03/05重新冻结layout与跨语言consumer。

### 14.7 App与Editor consumer

| 范围 | 当前裁决 / 后续owner |
|---|---|
| `zircon_app/src/entry/runtime_library/loaded_runtime.rs`及loader/error/tests | alignment/version/exact-size/required function校验保留；改capability negotiation与BuildSet/unload epoch |
| `zircon_app/src/entry/runtime_library/runtime_session.rs`及`runtime_session/*` | Arc foreign output、request owner和operation wrapper保留；去除raw JSON旁路、单surface bool与abort Drop，增加cancel/page receipt |
| `zircon_app/src/entry/runtime_library/wake_registry.rs` | wake registry可保留；与runtime WakeArbiter建立edge/ack/coalesce/latency合同 |
| `zircon_app/src/entry/runtime_entry_app/frame_loop.rs` | cadence消费demand方向保留；必须聚合所有producer并支持多window/view |
| `zircon_app/src/entry/runtime_entry_app/host_requests/*` | IME/cursor/rumble应用链存在；无window时不能静默return，apply failure需ack回runtime |
| `zircon_editor/src/core/gateway/contract.rs`、`capabilities.rs`及root wrappers | gateway复制函数表并手工注入capability；改runtime-negotiated immutable contract |
| `zircon_editor/src/core/gateway/session/gateway.rs`、`protocol.rs`、`output.rs` | 与App重复serialization/foreign validation；抽共享consumer library且保持typed page metadata |
| `zircon_editor/src/core/gateway/session/frame.rs`、`viewport.rs` | 单AtomicBool surface state；改per-qualified viewport lease/disposition |
| `zircon_editor/src/core/gateway/session/world_sync.rs` | request用raw `serde_json::to_vec`且drain只返Vec；改bounded request与page receipt/resync |
| `zircon_editor/src/core/gateway/session/plugin_events.rs` | 比App更完整地保留remaining/age；继续加wake/cursor/drop/resync与ack |
| `zircon_editor/src/core/gateway/session/operations.rs` | submit/poll/harvest存在；缺cancel/deadline/subscription与shutdown fence |
| `zircon_editor/src/core/gateway/session/profile.rs`、`overlay.rs`、tests | capability用字符串查找，profile/overlay identity弱；改typed capability/viewport generation并扩fault tests |
| `zircon_editor/src/core/gateway/in_process.rs`、`detached.rs` | linked/in-process与DLL路径语义分叉；必须共享同一host/session contract而非旁路ABI truth |

### 14.8 Shader CLI聚焦consumer

| 文件 | 当前裁决 / 后续owner |
|---|---|
| `zircon_runtime/src/bin/zircon_shader_prewarm/manifest.rs` | asset scan/registry/quality扩展是真实工具底座；literal compiler/platform identity和fallback路径不可信 |
| `zircon_runtime/src/bin/zircon_shader_prewarm/manifest/material_sources.rs` | material alpha/source digest进入key方向正确；schema default/provenance与real pipeline state闭包仍缺 |
| `zircon_runtime/src/bin/zircon_shader_prewarm/run.rs` | 多manifest merge和budget参数可保留；同步execution无operation lifecycle/shared device |

## 15. 首个实施切片

首个切片不应从“加更多FFI函数”开始，而应同时建立五组RED证据和删除矩阵：

1. ABA/cross-host：slot复用、跨host、跨DLL epoch的session/allocation/subscription stale handle。
2. False success：Headless capture/present、pipeline pending、fake a11y、stale manager empty、harvest commit fault。
3. Bounded lifecycle：hung action/callback、outstanding allocation/watch/subscription、operation prepare下的destroy deadline与isolation receipt。
4. Ordering/identity：event timestamp/device/view generation、host request跨类sequence、plugin/world cursor/drop/resync、physical/UI route receipt。
5. Performance truth：steady extract zero-full-clone、百万world page release P95、UI reachable closure scaling、shared-device shader cold/hot。

该切片通过前，禁止以“结构审计60/60”“FFI返回Ok”“黑RGBA尺寸正确”“单元测试数量增加”或“ignored benchmark存在”宣称Runtime43已实现。

## 16. 完成定义

Runtime43/Runtime131只有在以下条件同时满足时才能由`pending`改为`implemented`：

1. 64项P1全部Closed，16项P2完成或有明确defer owner/退出版本，42个资格门全部Pass。
2. V7到后继合同有App/Editor/linked/real DLL/C或C++ consumer矩阵，旧表按hard-cutover政策退出。
3. 多host、多session、多viewport、多平台、device loss、hung callback与pressure测试给出稳定receipt。
4. project startup只消费同一composition/project/cooked artifact hash并可完整rollback。
5. Runtime engine不再包含Vampire/HUD/menu产品字符串、component协议或点击规则，也不制造黑帧/伪tree成功。
6. DLL unload前证明action、callback、job、allocation、watch、subscription、operation和viewport全部terminal或isolated。
7. plugin/world/host output具备sequence/cursor/remaining/dropped/resync/wake/ack，慢query/encode不持session主锁。
8. shader prewarm绑定实际BuildSet/compiler/backend/device/driver并复用正式cache/device service。
9. steady frame、world paging、UI startup与prewarm冷/热路径达到预算并保存allocator/trace/release证据。
10. 重新执行source fingerprint、ABI layout、结构审计、Markdown link、P1/P2/Gate连续编号和文档格式验证。

在上述条件关闭前，`dynamic_api`应被描述为“已有安全底座但仍未工程化完成的V7同步适配层”，不能称为与Unreal等成熟引擎同级的Runtime Session Control Plane。
