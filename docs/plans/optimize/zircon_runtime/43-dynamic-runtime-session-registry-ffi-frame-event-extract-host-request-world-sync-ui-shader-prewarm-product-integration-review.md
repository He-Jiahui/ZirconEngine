---
related_code:
  - zircon_runtime/src/dynamic_api
  - zircon_runtime_interface/src/runtime_api
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
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
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 43 · Dynamic Runtime Session / Registry / FFI / Frame / Event / UI / World Sync / Shader Prewarm 工程化差距

## 1. 结论

`zircon_runtime::dynamic_api`已经不是只有几个裸FFI函数的样板。V7表有统一panic边界，固定布局请求先做ABI和指针检查；session action有销毁屏障；foreign output按session登记并验证owner；JSON输入有byte、depth、item和deadline限制；plugin event与world invalidation能够分页；frame extract有缓存和诊断；App会校验API表尺寸和必需函数。这些底座必须保留。

但当前实现仍是一条为单窗口示例产品打通的同步适配层，不是工程级Runtime Session Control Plane。所有session和allocation落在进程全局`OnceLock<Mutex<HashMap<...>>>`；每个session的tick、render、JSON query、world watch、UI、plugin event和operation都在同一`Mutex<RuntimeDynamicSession>`下串行执行。handle没有generation/owner epoch，registry没有容量，foreign allocation只有单次payload上限而没有session/global累计预算。destroy会无限等待活动action和foreign wake callback，内部module shutdown却使用零时长drain；bootstrap补偿失败甚至调用`std::process::abort()`终止宿主进程。

产品语义同样混杂。Dynamic session又手写一套六Profile；Minimal和Headless仍映射ClientRuntime并无条件注入脚本系统。项目场景、固定navmesh和startup scripts绕过统一cook/asset transaction；`play_report_pipe`校验后被丢弃。无render bridge时bind/present返回成功，capture制造黑帧；pipelined render未完成时也制造黑帧。Runtime UI同步扫描全部UI资产、忽略alias冲突、以manifest顺序决定surface identity，并在不存在UI时制造假的accessibility tree。更严重的是engine runtime内硬编码`gameplay.hud_text`、`gameplay.menu_state`、Vampire Roguelite、Blood Bolt和Retry等示例产品规则。

事件、world sync和shader prewarm也没有形成可扩展控制面。输入包缺timestamp、device/user和window generation，键位只覆盖少量键，touch被压成单一mouse cursor；host requests按IME/rumble/cursor分组排空而丢失跨类别因果顺序。plugin event纯轮询且不触发wake；world invalidation逐项`remove(0)`，分页试探会在session锁内反复clone/encode。shader prewarm虽然能生成固定六pass并验证WGPU模块，却硬编码Naga/WGPU身份、同步创建新offscreen backend、只在单批次缓存模块，并可能把template错误静默变成空manifest或补入fallback shader实现。

本报告新增 **0项P0、64项P1、16项P2和42个资格门**。Runtime Interface01/03/05继续拥有ABI协商、foreign ownership、公共event/status/admission的P0；Runtime01/02/05/07/09A/09C/11A/22/24/41/42拥有核心生命周期、任务、world、插件、render、shader、UI、时间、identity、operation和composition根合同；App01/06拥有宿主停机和示例产品污染P0。本篇不重复计数，负责把当前dynamic adapter重构为有generation、预算、状态机、调度lane、真实capability disposition和可观测receipt的session控制面。

## 2. 审查边界、语料与 currentness

### 2.1 冻结语料

| 子域 | 文件 / 行 / bytes | test属性或宏 / ignored | 结论 |
|---|---:|---:|---|
| `zircon_runtime/src/dynamic_api`完整实现与测试 | 76 / 17,955 / 652,225 | 184 / 10 | E3逐文件检查export、session registry、FFI、frame/event/UI/world/operation、bounded JSON、shader prewarm和测试 |
| `zircon_runtime_interface/src/runtime_api`完整V7合同 | 11 / 2,251 / 68,849 | 7 / 0 | E3反查session/event/viewport/frame demand/host request/plugin event/operation固定布局与能力表达 |
| App动态库宿主与shader prewarm真实consumer | 5 / 3,089 / 114,990 | 9 / 0 | E3核对API装载、session包装、foreign output释放及CLI/cache调用链 |
| 父报告与唯一owner | 18 / 7,851 / 779,967 | 12 / 2 | E2核对P0归属、模块生命周期、任务、world、render、UI、identity、operation、composition和产品边界 |
| Unreal、Bevy、Godot、Fyrox、Unity Graphics | 18 / 43,341 / 1,614,620 | 24 / 0 | E2/E3核对engine/world context、plugin phase、runner/sub-app、input/UI、PSO batch和render graph cache |
| selected combined scope | 128 / 74,487 / 3,230,651 | 236 / 12 | 工作树fingerprint `cdac75859748f3addbe8e09fd43ed0e23214617f52472b501ac4f184c0071850` |

指纹按128个selected path去重排序，对每个文件取lowercase SHA-256，再以`forward/slash/path|hash`和LF连接、无末尾LF后取总SHA-256。测试数字是静态Rust/C++/C#标记，不表示本轮已编译或通过。12个ignored中，10个位于dynamic实现语料，9个集中在Vampire gameplay/HUD/menu与对应性能证据。

### 2.2 检查方法

本轮按`get_api -> host validate -> create config decode -> linked composition -> project prepare/activate -> session registry publish -> action admission -> event/UI/input -> tick/frame demand -> extract/render/present -> host request/plugin event/world sync/operation -> foreign allocation -> destroy/quiesce -> shader prewarm CLI/cache`逐段阅读，并反向搜索全部非`dev/`生产consumer。每段分别核对identity、owner、generation、thread/lock、budget、ordering、transaction、failure disposition、wake、teardown、observability和规模成本。

### 2.3 动态证据边界

1. 本轮是review-only，没有修改Runtime、App、Editor、Plugin、Hub、Interface生产代码、测试、Cargo或产品资产。
2. 未重新运行已知耗时或已被其他域阻断的全工作区编译；Editor、Hub、WOC和plugin metadata的既有阻断保持原状。
3. 黑帧、伪accessibility tree、全局registry、单session mutex、零时长module drain、`remove(0)`、ignored insertion、硬编码Vampire字符串和版本literal均由静态控制流直接证明。
4. 未执行多session争用、hung callback、allocation retention flood、多viewport、多触点、Linux/macOS surface、百万world invalidation、GPU device loss或跨驱动shader cache矩阵，因此相应资格保持未通过。
5. 实施前必须重取fingerprint并复核当前工作树；本报告是2026-08-16审查快照，不是长期稳定基线。

## 3. 必须保留的工程基础

1. 保留单一V7静态函数表和App对alignment、version、size及required function的校验方向。
2. 保留每个FFI入口的panic隔离，但将callback、drop和bootstrap补偿纳入同一failure policy。
3. 保留session action admission与destroy quiescence屏障，不允许以直接删除registry替代安全销毁。
4. 保留foreign allocation的session owner验证和显式release，扩展为generation与累计预算。
5. 保留固定布局小对象与owned bounded payload分工，避免把任意Rust对象穿过DLL。
6. 保留bounded JSON的byte、nesting、item和deadline检查，后续减少重复扫描并移出session主锁。
7. 保留plugin event和world invalidation分页方向，但增加sequence/backlog/wake/cursor receipt。
8. 保留frame demand的Idle/Immediate/After表达，但由统一WakeArbiter聚合所有producer。
9. 保留project prepare先于activation和project manager snapshot复用方向，升级为不可变startup plan。
10. 保留runtime UI surface与retained UI tree/layout/input/accessibility共享数据模型，删除伪造和示例fallback。
11. 保留frame extract cache及clone/byte诊断，改为immutable snapshot/arena与完整generation key。
12. 保留shader variant key、manifest、disk cache和真实WGPU validation，移入正式shader build/cache service。
13. 保留headless profile不创建render bridge的意图，但返回真实Unsupported/NotReady而非黑帧成功。
14. 保留Runtime41 operation service作为唯一领域owner；dynamic层只做有request ID、wake和receipt的传输适配。
15. 保留Runtime42 composition compiler方向；dynamic session只消费其冻结plan和receipt，不重建第二份Profile/plugin truth。
16. 保留App01作为进程宿主与最终shutdown coordinator，dynamic session只负责session-owned drain和可证明terminal。

## 4. 当前实现的核心断路

| 链路 | 当前事实 | 工程后果 |
|---|---|---|
| API入口 | V7表统一panic wrapper，host只给ABI/size和两个可选callback | 安全底座存在，但没有host capability、threading、allocator、platform surface或compatibility negotiation |
| Session identity | 全局HashMap中的递增`u64`，无generation/namespace | stale handle只有“当前不存在”语义，无法防ABA、DLL generation或多host串用 |
| Action execution | 全部操作锁住单个`RuntimeDynamicSession` | 慢JSON、capture、script、world encode和GPU初始化互相头阻塞 |
| Destroy | 禁止wake内自毁，随后无限等active action/callback；module drain为零 | hung callback/action可永久卡宿主，内部shutdown又可能立即误报未排空 |
| Allocation | 每个payload有上限，allocation owner可校验 | host可保留任意数量的有界payload，累计内存无admission或pressure策略 |
| Composition | dynamic profile、linked registration、builtin script再次投影 | 同一产品拥有第二份profile和extension truth，Headless/Minimal也可带Client script |
| Project startup | scene/navmesh/scripts走物理文件和同步串行加载 | cook/VFS/provenance/rollback被绕过，scene prepare与read之间存在TOCTOU |
| Render fallback | 无renderer或pipeline未完成仍返回成功黑帧 | “能力不可用/帧未就绪”被伪装为合法产品输出 |
| Extract | world tick等少量revision作key，缓存仍深clone | 失配时可陈旧，命中时仍有大对象复制，byte统计不是allocator事实 |
| Input | 单一宽struct复用字段，缺timestamp/device/user/window generation | 多用户、多设备、重放、排序、IME/触控和窗口重建无法精确表达 |
| Host requests | 分类别drain后拼接 | IME、rumble、cursor跨类别因果顺序被改变，缺request sequence/ack |
| Runtime UI | 全项目扫描、manifest顺序ID、node bit packing、忽略alias错误 | 启动成本随项目膨胀，identity易漂移/碰撞，错误可静默降级 |
| Product fallback | engine内识别gameplay/Vampire组件和文本 | 示例产品协议成为引擎ABI，其他游戏无法复用或替换 |
| Event/world output | 轮询分页、无统一remaining/wake，world page前移`remove(0)` | reactive host可休眠丢推进；大backlog出现二次复杂度与锁内编码 |
| Shader prewarm | 六固定pass、literal版本、新WGPU backend/批次 | 不代表实际产品/设备/驱动PSO闭包，启动成本和cache identity不可信 |
| Tests | ABI/分页/基础render覆盖较多，产品真实VM用例大量ignored | 可证明适配器局部行为，不能证明多session、故障、平台和规模资格 |

## 5. 参考实现给出的边界

### 5.1 Unreal

`FEngineLoop`把PreInit、Init、Tick和Exit分阶段并在启动/停机记录profiling；`FWorldContext`显式拥有world type、GameInstance、travel/pending state和destroy event；`FModuleManager`区分load failure、unload、abandon和shutdown。Slate维护window/user/input processor、焦点、accessibility和input thread交接。`FShaderPipelineCache`提供Pause/Resume、Background/Fast/Precompile batch mode、每帧batch size/time、统计与保存。Zircon不应复制宏和全局对象，但必须吸收“session/world、窗口输入、模块和PSO任务都有独立状态机、预算、阶段与可查询终态”的边界。

### 5.2 Bevy

Bevy `App`把runner作为唯一主循环owner，plugin有Adding/Ready/Finished/Cleaned阶段；多个`SubApp`拥有独立World、schedule和extract函数，主App按明确顺序抽取并更新。`ScheduleRunnerPlugin`区分run once、loop和wait duration。它不是动态DLL安全上限，但说明headless/windowed runner、render sub-world和plugin finalize不应折叠成一个session mutex和几个profile字符串。

### 5.3 Godot

Godot `Main`明确执行setup/setup2/start/iteration/cleanup，并按Servers/Scene/Editor level初始化和逆序反初始化extension；`SceneTree`分别处理physics/process、timers、tweens和frame signal；`GDExtensionManager`区分load/reload/unload状态。Zircon需要同等明确的session phase、world tick domain、extension generation和reload terminal，且要比Godot进一步提供跨DLL预算和receipt。

### 5.4 Fyrox

Fyrox Executor拥有event loop、graphics context、plugin lifecycle与lag/fixed update协调；plugin contract暴露register、init、loaded、OS event、graphics context created/destroyed和deinit。它证明window/event/render/plugin不是“调用某个FFI就立即完成”的同质动作。Zircon应把这些阶段映射到typed lane与lifecycle receipt，而不是把全部状态藏在`RuntimeDynamicSession`字段里。

### 5.5 Unity Graphics

Unity Graphics的RenderGraph显式维护recording/executing状态、resource registry、object pool、debug registration与compilation cache；调用时会检查非法状态并按hash复用编译结果。该镜像不能证明Unity完整player session或插件生命周期，但足以反证“每次prewarm新建backend、只在当前batch缓存module、以literal版本命名cache”不是成熟shader pipeline cache。

### 5.6 Zircon的超越目标

目标不是增加更多FFI函数，而是让动态runtime成为可嵌入、可隔离、可恢复的控制面：每个host/session有不可伪造generation handle、明确线程模型、累计预算和状态机；write/event/frame/job通过有界lane进入owner executor；read侧消费不可变snapshot；所有异步producer汇入WakeArbiter；render/UI/world/plugin/operation输出带sequence、remaining、disposition和release owner；project/composition/shader cache都绑定BuildSet与receipt。性能目标是慢请求不阻塞无关session或read lane，steady frame不做全extract深clone，prewarm复用真实device和编译缓存。

## 6. P0 唯一归属与依赖路由

本篇不新增P0。以下既有阻断是Runtime43的前置或联合交付：

| Canonical owner | 现有根阻断 | Runtime43责任 |
|---|---|---|
| Runtime Interface01/05 | API协商、FFI ownership、foreign output、budget/fuse与unload安全 | 实现generation session/allocation、typed disposition、累计预算和可终止callback协议 |
| Runtime Interface03 | input/UI/accessibility/status公共合同不完整 | 实现timestamp/device/user/window generation并删除伪tree/黑帧成功 |
| Runtime01 | Core lifecycle、module deactivation和cleanup terminal | 提供session state machine、bounded drain、teardown census和失败receipt |
| Runtime02 | task admission、cancel、deadline和shutdown | 把慢decode/query/prewarm移入有界job lane，不在session锁中同步执行 |
| Runtime05 | World lifecycle、mutation和snapshot | 只暴露generation-bound immutable world snapshot与transactional mutation入口 |
| Runtime07/42 | script/plugin generation与统一composition plan | dynamic构造只消费一个冻结plan，不默认注入或重建linked plan |
| Runtime09A/09C | render device/surface/frame disposition与shader/PSO cache | 实现真实NotReady/Unavailable、多viewport/platform surface和共享device prewarm |
| Runtime11A | retained UI surface、identity、input/a11y和hot reload | 提供稳定surface/node ID、分层composition和依赖闭包更新 |
| Runtime22/24 | clock/replay与稳定handle/generation | event携带时间/sequence；session/subscription/allocation/viewport统一generation identity |
| Runtime41 | operation cancel/progress/wake/result/commit | dynamic适配request ID、wake和commit failure，不复制operation scheduler |
| App01 | 产品host loop、thread owner、shutdown coordinator | 驱动typed demand并在DLL unload前证明全部session/allocation/callback terminal |
| App06 | Vampire示例产品边界与产品证据 | 把HUD/menu/text/component协议迁回项目plugin/asset，engine删除产品特判 |
| Plugins05 | shader importer/compiler/artifact/cache identity | prewarm消费真实compiled artifact与BuildSet，不补写shader source |

## 7. P1 差异与重构项

### 7.1 Session registry、identity、lifecycle与foreign ownership

| ID | 当前事实 | 需要重构的工程合同 |
|---|---|---|
| DYN-P1-001 | Session handle是递增裸`u64`，无generation、host namespace或DLL epoch | 使用`RuntimeSessionId { slot, generation, owner, runtime_epoch }`并验证stale/foreign handle |
| DYN-P1-002 | Session与allocation registry是进程全局`OnceLock<Mutex<HashMap>>` | 由`RuntimeHostInstance`拥有registry，支持多host隔离、reset和unload census |
| DYN-P1-003 | session registry没有最大session数、创建速率或内存admission | 配置并强制host/project/profile级session census和资源预算 |
| DYN-P1-004 | 每个action在单一session mutex内执行完整业务 | 引入owner executor和control/event/read/frame/job lanes，明确哪些状态只可在owner线程变更 |
| DYN-P1-005 | destroy无限等待active action和wake callback | 支持deadline、cancel request、progress snapshot、force-isolate policy和typed Timeout terminal |
| DYN-P1-006 | 外层等待无限，module shutdown drain timeout却固定为零 | 统一由ShutdownBudget分配各phase deadline并记录未排空owner |
| DYN-P1-007 | outstanding foreign allocation阻止destroy且要求host重试 | destroy receipt列出allocation census、age/bytes/owner，并支持显式revoke/quarantine策略 |
| DYN-P1-008 | 只限制单个payload大小，不限制session/global allocation数量与累计bytes | 建立allocation arena的count/bytes/age高水位、拒绝策略和pressure diagnostic |
| DYN-P1-009 | wake callback直接同步调用，无coalesce、rate、deadline或slow-callback处理 | WakeArbiter只在idle到pending边沿触发，callback走host dispatcher并有延迟/失败统计 |
| DYN-P1-010 | poisoned registry/session lock会恢复并继续服务 | 区分可恢复数据与invariant corruption；后者隔离session并拒绝继续mutation |
| DYN-P1-011 | `Drop`忽略session shutdown和viewport destroy失败 | 所有显式owner必须close并消费receipt；Drop只做最后隔离且记录unresolved census |
| DYN-P1-012 | bootstrap后的log lease补偿失败会`process::abort()` | 返回HostSafetyViolation并隔离未卸载runtime generation，不能由库代码杀死宿主进程 |

### 7.2 Composition、Profile与project startup

| ID | 当前事实 | 需要重构的工程合同 |
|---|---|---|
| DYN-P1-013 | Dynamic session手写第二套六Profile及字符串解析 | 只接受Runtime42生成的typed Profile/CompositionPlan ID与hash |
| DYN-P1-014 | 空profile bytes静默等于Runtime | 要求宿主显式提交profile；兼容默认必须在App产品配置层解析并写入receipt |
| DYN-P1-015 | 所有dynamic profile固定`max_fixed_steps_per_frame = 8` | 固定步策略来自ClockDomain/Profile policy，带accumulator、overrun和determinism receipt |
| DYN-P1-016 | Minimal/Headless映射ClientRuntime，builtin script系统仍无条件注入 | capability solver决定模块/system闭包；无Script能力不得出现脚本系统 |
| DYN-P1-017 | 无project manifest时linked registration selection可自动变成启用 | linked provider只声明availability，selection必须来自effective project/profile plan |
| DYN-P1-018 | `script_systems`忽略传入linked plan，克隆registry后重新构造 | construction消费同一个冻结plan和已解析system rows，禁止第二次解析 |
| DYN-P1-019 | 模块、asset、nav、script、scene、UI逐步激活但没有整体transaction/receipt | 建立prepare/stage/validate/commit/publish/rollback的SessionStartupTransaction |
| DYN-P1-020 | `play_report_pipe`完成格式校验后被丢弃 | 要么从ABI移除，要么由host-owned report sink绑定request/session并确认交付 |
| DYN-P1-021 | Play scene在prepare解析路径，activation稍后直接读物理文件并建新World | 通过VFS/asset snapshot读取content hash绑定的scene artifact，关闭TOCTOU和绕cook路径 |
| DYN-P1-022 | navmesh固定`assets/navigation/main.navmesh.toml`，startup scripts同步串行且空列表等于全加载 | manifest声明资源/脚本闭包、并发预算、依赖顺序、failure policy与rollback receipt |

### 7.3 Frame、surface、extract与demand

| ID | 当前事实 | 需要重构的工程合同 |
|---|---|---|
| DYN-P1-023 | Headless/Minimal的bind、unbind、present成功，capture返回全黑RGBA | 返回`Unavailable(NoRenderCapability)`；需要软件/测试renderer时显式选择provider |
| DYN-P1-024 | Pipelined capture尚无completed frame时也返回全黑RGBA | 返回`NotReady { submitted, completed, next_wake }`或最后一个有generation的有效帧 |
| DYN-P1-025 | 生产路径只接受default viewport | 建立session-owned viewport registry、generation handle、per-viewport camera/UI/render state |
| DYN-P1-026 | Native surface ABI与实现只支持Win32 | surface descriptor使用平台tagged payload和capability negotiation，覆盖Windows/Linux/macOS/headless |
| DYN-P1-027 | resize通过destroy/recreate viewport并重置capture generation | 保留surface/viewport identity，显式迁移swapchain/device generation和in-flight frame disposition |
| DYN-P1-028 | extract cache key只含world change、visibility、camera、viewport | 纳入asset/resource/material/plugin/render config/device/UI generation或改为依赖跟踪snapshot |
| DYN-P1-029 | cache rebuild和命中都深clone完整`RenderFrameExtract` | 使用immutable frame snapshot、arena/chunk reuse或Arc lease，steady state不得全量复制 |
| DYN-P1-030 | extract byte估算遗漏capacity与嵌套分配 | 接入allocator/tagged arena真实resident/peak/retained统计，并区分logical payload |
| DYN-P1-031 | frame demand只观察asset reload和animation | WakeArbiter聚合operation、UI timer、network、task、plugin event、world invalidation和render completion |
| DYN-P1-032 | tick开始/失败清空accumulated demand，可能丢失并发producer请求 | demand使用generation/atomic merge，只有宿主ack后才能消费，失败保留未满足请求 |
| DYN-P1-033 | background/suspended只映射FocusLost，无resume/foreground/low-memory/device restore序列 | 生命周期进入session/viewport状态机，逐owner暂停、恢复、重建设备并发布receipt |

### 7.4 Event、input与host request

| ID | 当前事实 | 需要重构的工程合同 |
|---|---|---|
| DYN-P1-034 | Event包缺host timestamp、monotonic sequence、device/user和window generation | V2 event envelope显式携带time domain、sequence、device/user/viewport generation和source |
| DYN-P1-035 | UI metadata sequence用saturating add，到`u64::MAX`后永久重复 | sequence按generation rollover或拒绝溢出，重放/去重合同明确 |
| DYN-P1-036 | logical key只映射修饰键、数字和WASD，repeat硬编码false | 采用完整physical/logical/text/repeat/modifier合同及平台layout转换 |
| DYN-P1-037 | unknown keyboard action可静默成功；gamepad ID截成`u16`，axis/button缺完整finite/range验证 | 所有枚举和值先admit，未知值返回typed unsupported/invalid并计数 |
| DYN-P1-038 | 所有touch contact写入单一cursor/mouse-left camera drag | 保存per-contact state、primary pointer、gesture arbitration、capture/cancel和multi-touch routing |
| DYN-P1-039 | UI先消费事件，consumed事件不进入gameplay input，缺context和route receipt | InputRouter按surface/focus/context/priority路由并发布consumed-by、fallback和capture结果 |
| DYN-P1-040 | host requests分别drain IME、rumble、cursor后拼接 | producer写入统一有序request queue，保留sequence、causal parent、deadline和ack |
| DYN-P1-041 | manager stale时host request drain返回空成功 | 返回ManagerGenerationStale/Unavailable，避免把owner失效伪装为空队列 |
| DYN-P1-042 | FFI调用同步处理事件，没有queue容量、drop/coalesce或burst receipt | 建立有界event ingress、per-kind coalescing、backpressure和accepted/dropped sequence receipt |

### 7.5 Runtime UI与产品污染

| ID | 当前事实 | 需要重构的工程合同 |
|---|---|---|
| DYN-P1-043 | startup同步扫描项目中全部UI资产构造prototype store | 从声明root解析依赖闭包，复用asset index/DDC并并行预取有预算的artifact |
| DYN-P1-044 | `insert_with_aliases`结果用`let _`忽略 | duplicate/alias/schema conflict必须使startup失败或进入显式degraded receipt |
| DYN-P1-045 | surface identity由manifest数组顺序决定 | 使用asset ID + declared surface ID + generation，重排manifest不改变identity |
| DYN-P1-046 | global node ID把surface塞入高16位并截断local ID到48位 | 使用结构化`UiNodeHandle { surface, local, generation }`，拒绝溢出而非mask碰撞 |
| DYN-P1-047 | 多surface commands直接拼接，最终extract只保留最后surface的raster scale | 每个surface保留独立scale/viewport/layer/clip，compositor显式排序与合成 |
| DYN-P1-048 | RuntimeUiSurfaceSet没有UI asset hot reload/rebase路径 | 订阅依赖generation，prepare新tree、迁移state/focus、原子publish并保留失败旧版本 |
| DYN-P1-049 | 未声明UI时capture accessibility返回伪造“Zircon Runtime Preview”tree | 返回Unavailable/EmptyAuthoritativeTree，禁止制造不存在的产品节点 |
| DYN-P1-050 | 项目UI存在时整体隐藏legacy HUD/menu，不做layer composition | 删除legacy分支；正式UI compositor按layer/capability组合project、debug和overlay surface |
| DYN-P1-051 | engine内硬编码Vampire组件、文案、颜色、布局、点击写动态component | 全部迁回App06项目plugin/asset/action binding；runtime只执行通用UI和command合同 |

### 7.6 Plugin event、world sync与operation适配

| ID | 当前事实 | 需要重构的工程合同 |
|---|---|---|
| DYN-P1-052 | plugin subscription是递增裸`u64`且无session generation/capacity | 统一generation handle、subscription budget、filter cost和owner teardown |
| DYN-P1-053 | delivery把raw session handle当`play_session_id` | 发布稳定的play/world context identity，禁止把进程局部slot泄漏为产品identity |
| DYN-P1-054 | plugin events只支持poll drain，事件到达不触发session wake | backlog从0到非0时请求wake并报告oldest/newest sequence与remaining |
| DYN-P1-055 | 空plugin page返回零字节，不返回envelope | 始终返回typed page header/disposition，区分Empty、CursorExpired、OwnerGone和More |
| DYN-P1-056 | world query/watch/drain在session mutex中同步decode、执行和encode，watch admission在dynamic层无资源receipt | 使用snapshot read lane和watch registry budget，输出token generation、cost与initial cursor |
| DYN-P1-057 | world invalidation无remaining/backlog，commit逐项`remove(0)`，分页试探锁内反复encode | 使用deque/immutable segments和一次性page sizing，receipt携带cursor、remaining、dropped、resync |
| DYN-P1-058 | operation submit不wake；harvest先发布output后忽略commit失败 | 适配Runtime41的request/wake/cancel/deadline与two-phase harvest，commit失败不得返回成功output |

### 7.7 Bounded JSON与shader prewarm

| ID | 当前事实 | 需要重构的工程合同 |
|---|---|---|
| DYN-P1-059 | inbound JSON执行nesting scan、syntax graph scan、typed deserialize和业务item traversal | 采用一次受预算的stream/arena decode或缓存验证结果，并移出session主锁 |
| DYN-P1-060 | deadline按wall clock在chunk/closure前后检查，长encode/item closure不可中断且无共享session work budget | 使用cooperative work units、CPU budget、cancel token和确定性测试clock |
| DYN-P1-061 | shader prewarm实现挂在`dynamic_api`公开模块，却不在V7 table/session生命周期中 | 移入Runtime09C/Plugins05 shader build/cache service，dynamic API只暴露正式异步job或完全不暴露 |
| DYN-P1-062 | cache identity硬编码`naga-29.0.1`、`wgpu-29.0.1`和`wgpu-runtime`；template错误可静默返回空manifest或补fallback实现 | identity来自实际compiler/backend/driver/target BuildSet；source/template错误必须原样失败 |
| DYN-P1-063 | manifest固定六pass、static mesh、Medium/default quality，未从产品material/scene/target usage生成 | cook阶段汇总真实variant/PSO usage、quality/platform/provider闭包并生成可追踪manifest |
| DYN-P1-064 | 每次WGPU validation新建offscreen backend，module cache仅当前batch，调用同步且无cancel/progress | 复用device/compiler pool与跨批次artifact cache，走有界异步job、进度、取消和shutdown drain |

## 8. P2 改进项

| ID | 改进项 |
|---|---|
| DYN-P2-001 | status diagnostics使用thread-local 4096-byte借用buffer；在Interface下一版本改为owned或caller buffer，并记录截断。 |
| DYN-P2-002 | `ZrRuntimeEventV1`以`x/y/delta/button/state/key_code/scan_code`复用多类语义；下一版本改为tagged payload union和per-kind size。 |
| DYN-P2-003 | camera controller硬编码到每个dynamic session；迁为可选camera input controller plugin/viewport policy。 |
| DYN-P2-004 | camera left-button press/release是无动作分支；删除无意义路径或实现明确selection/capture owner。 |
| DYN-P2-005 | camera transform update失败被吞掉；写入input/render diagnostic并返回route outcome。 |
| DYN-P2-006 | scene reload count把`usize`转为`f64`；高计数可能丢整数精度，诊断store应支持u64 counter。 |
| DYN-P2-007 | project identity使用可选display name字符串；改为project UUID + manifest generation + build identity。 |
| DYN-P2-008 | runtime profile和很多failure只以自由字符串输出；统一stable code、stage、owner、correlation和remediation。 |
| DYN-P2-009 | event/file/gamepad名称遇到无效UTF-8时部分路径降为None；明确拒绝、lossy display与raw identity分层。 |
| DYN-P2-010 | plugin empty delivery使用owned empty buffer的测试不应锁定零字节协议；迁到typed Empty page。 |
| DYN-P2-011 | 结构测试大量读取`include_str!`并匹配源码片段；改为行为、trait contract和compile-fail测试。 |
| DYN-P2-012 | 10个dynamic ignored中9个是Vampire真实VM/渲染证据；移交App06/plugin owner并建立可运行产品资格，不留永久ignore。 |
| DYN-P2-013 | 默认viewport handle `1`散布在兼容路径；只保留host创建的显式viewport，兼容别名限时退役。 |
| DYN-P2-014 | accessibility fallback诊断文字声称“dynamic preview”但实际没有surface；删除误导命名并使用typed reason。 |
| DYN-P2-015 | project startup日志是阶段字符串，没有duration、input hash或terminal outcome；接入结构化startup trace。 |
| DYN-P2-016 | shader prewarm alpha cutoff缺省为0.0会改变variant语义；缺省值来自material schema并进入key。 |

## 9. 目标架构

```text
Host Process / Editor / Product
        |
        v
RuntimeHostInstance (runtime epoch, capabilities, budgets, dispatcher)
        |
        +--> SessionRegistry [slot + generation + owner]
        |         |
        |         v
        |   RuntimeSessionActor
        |   Created -> Preparing -> Active -> Quiescing -> Draining -> Terminal
        |      |          |          |             |
        |      |          |          |             +--> ShutdownReceipt / LeakCensus
        |      |          |          +--> Snapshot Read Lane
        |      |          +--> Event / Control / Frame bounded lanes
        |      +--> RuntimeCompositionPlan + ProjectStartupPlan
        |
        +--> WakeArbiter [edge-triggered, coalesced, rate/latency metrics]
        +--> ForeignAllocationArena [count/bytes/age/generation]
        +--> AsyncJobService [query/encode/operation/shader work]
        +--> ViewportRegistry [platform surface, generation, disposition]
```

最小合同建议：

```text
RuntimeHostIdentity { runtime_epoch, build_set, platform, capabilities }
RuntimeSessionId { slot, generation, runtime_epoch, owner }
SessionState { Preparing, Active, Quiescing, Draining, Terminal, Isolated }
RuntimeRequestHeader { request_id, session, deadline, budget, sequence }
RuntimeDisposition { Accepted, Pending, Ready, Empty, NotReady, Unavailable, Rejected }
RuntimePageReceipt { cursor, first_sequence, last_sequence, remaining, dropped, resync }
ViewportId { slot, generation, session }
SessionStartupReceipt { composition_hash, project_hash, activated, degraded, rollback }
SessionShutdownReceipt { actions, callbacks, allocations, jobs, viewports, terminal }
```

不要通过把`Mutex`换成`RwLock`制造并发假象。World、render bridge和UI mutation仍应由明确owner actor/thread串行提交；并发来自有界ingress、异步prepare/encode、不可变snapshot read和独立session之间的调度。所有lane都必须有容量、deadline、cancel和shutdown owner。

## 10. 分阶段重构计划

### M0 · 合同冻结与false-success止血

1. 为黑帧、伪accessibility tree、stale manager empty和ignored harvest commit建立失败测试。
2. 定义`RuntimeDisposition`、session/viewport/subscription/allocation generation identity与兼容切换策略。
3. 记录当前V7 consumer矩阵和DLL unload/linked runtime差异，不在旧表内追加字段。

### M1 · Host instance、registry与lifecycle

1. 引入`RuntimeHostInstance`和session actor，移除业务对进程全局registry的直接访问。
2. 创建有界session/allocation registry、shutdown budget和terminal receipt。
3. wake进入edge-triggered arbiter；hung/slow callback不再卡住runtime owner。

### M2 · 唯一composition与project startup transaction

1. Dynamic构造只消费Runtime42的`RuntimeCompositionPlan`。
2. Project scene/nav/scripts/UI通过VFS/asset snapshot生成`ProjectStartupPlan`。
3. prepare/stage/commit/publish失败时逆序rollback，report sink交付完整receipt。

### M3 · Event、input、demand与host request lane

1. 新event envelope加入time/sequence/device/user/viewport generation。
2. 建立有界InputRouter和统一host request queue，保留跨类别顺序。
3. WakeArbiter聚合全部producer，宿主ack后才消费frame demand。

### M4 · Viewport、frame与extract

1. 多viewport registry和跨平台surface provider替代default/Win32特例。
2. 定义Ready/NotReady/Unavailable/DeviceLost frame disposition，删除黑帧成功。
3. Render snapshot使用完整generation依赖和arena lease，移除steady deep clone。

### M5 · UI、event mirror与world sync

1. UI root只解析依赖闭包，surface/node使用稳定generation ID并支持hot reload transaction。
2. 删除engine内HUD/menu/Vampire适配，改由项目plugin/asset注册layer/action。
3. plugin/world page统一cursor/remaining/drop/resync/wake，world read走immutable snapshot。

### M6 · Operation与bounded work

1. 对接Runtime41的operation wake/cancel/deadline/two-phase harvest。
2. JSON decode/encode改为合作预算job，禁止长任务持有session主锁。
3. 建立per-session CPU、queue、output和watch budget及pressure telemetry。

### M7 · Shader prewarm归位与产品资格

1. 将prewarm迁入正式shader build/cache service，identity绑定compiler/backend/device/driver/BuildSet。
2. cook收集真实variant/PSO使用，runtime复用device/compiler pool异步预热。
3. 关闭多session、多平台、多viewport、设备丢失、压力、取消、shutdown和产品证据矩阵。

## 11. 资格门

| Gate | 通过条件 |
|---|---|
| DYN-GATE-001 | stale session handle在slot复用后稳定拒绝，不能命中新generation |
| DYN-GATE-002 | 两个host instance的session/allocation/subscription不能互相访问 |
| DYN-GATE-003 | session count、allocation count/bytes和watch/subscription容量达到上限时typed拒绝 |
| DYN-GATE-004 | 一个session的慢query/capture不阻塞其他session的tick/read SLO |
| DYN-GATE-005 | hung action和hung wake callback下destroy在预算内返回可诊断terminal/isolated结果 |
| DYN-GATE-006 | 所有shutdown phase共享预算并报告未排空owner，不再混用无限和零等待 |
| DYN-GATE-007 | outstanding foreign outputs有age/bytes/owner census和明确revoke/quarantine策略 |
| DYN-GATE-008 | poison/invariant fault只隔离受影响generation，不继续静默mutation也不杀宿主进程 |
| DYN-GATE-009 | Dynamic session消费的composition hash与App/Core最终receipt完全一致 |
| DYN-GATE-010 | Headless/Minimal能力闭包不出现未选择的Client render/script模块 |
| DYN-GATE-011 | 未选择的linked registration不能注入module/system/extension |
| DYN-GATE-012 | scene prepare与activate绑定同一content hash，替换物理文件不会产生TOCTOU |
| DYN-GATE-013 | navmesh、startup script和UI root均来自VFS/cooked artifact闭包 |
| DYN-GATE-014 | startup任一阶段失败会逆序rollback且receipt无部分发布 |
| DYN-GATE-015 | `play_report_pipe`有真实交付/ack，或在硬切版本中从合同删除 |
| DYN-GATE-016 | profile fixed-step和overrun策略由Runtime22统一配置并可重放 |
| DYN-GATE-017 | 无render capability的capture/present返回Unavailable而不是成功黑帧 |
| DYN-GATE-018 | pipelined未完成、device lost和surface stale有不同typed disposition |
| DYN-GATE-019 | 同session至少4 viewport并发resize/present/capture保持独立generation |
| DYN-GATE-020 | Windows、Linux、macOS和headless surface capability矩阵有真实provider测试 |
| DYN-GATE-021 | resize不销毁逻辑viewport identity，in-flight frame有明确retire结果 |
| DYN-GATE-022 | 任意asset/material/plugin/render config/device generation变化都正确失效extract |
| DYN-GATE-023 | steady unchanged frame无完整`RenderFrameExtract`深clone，allocator证据达标 |
| DYN-GATE-024 | frame demand覆盖operation/UI timer/network/task/plugin/world/render producer且不丢并发请求 |
| DYN-GATE-025 | suspend/resume/low-memory/device restore按owner顺序执行并有receipt |
| DYN-GATE-026 | 输入replay按timestamp/sequence得到相同路由和world结果 |
| DYN-GATE-027 | 多键盘/手柄/用户/窗口generation可区分且stale event被拒绝 |
| DYN-GATE-028 | 多触点capture/cancel/gesture不会互相覆盖或退化为单鼠标状态 |
| DYN-GATE-029 | UI/gameplay input消费有route receipt并能复现focus/context决策 |
| DYN-GATE-030 | host requests跨IME/rumble/cursor保持全局sequence和causal order |
| DYN-GATE-031 | event burst达到上限时coalesce/drop/backpressure均有计数和宿主可见结果 |
| DYN-GATE-032 | UI启动成本只随reachable dependency closure增长，不随项目全部UI资产线性增长 |
| DYN-GATE-033 | UI alias/ID冲突不能被忽略，surface reorder不改变identity |
| DYN-GATE-034 | 多surface不同raster scale/layer正确合成且node ID无mask碰撞 |
| DYN-GATE-035 | UI hot reload保留或显式重置focus/state，失败不破坏旧tree |
| DYN-GATE-036 | 无UI时a11y返回authoritative empty/unavailable，engine中无Vampire/HUD/menu产品特判 |
| DYN-GATE-037 | plugin event从空变非空触发一次coalesced wake，page含sequence/remaining |
| DYN-GATE-038 | world invalidation百万记录分页无`remove(0)`二次复杂度且支持resync |
| DYN-GATE-039 | world query/encode慢任务可取消，不持有session主锁阻塞tick |
| DYN-GATE-040 | operation output只有在harvest commit成功后发布，submit会驱动reactive host |
| DYN-GATE-041 | shader manifest覆盖真实产品/质量/平台variant，cache identity与实际compiler/device/driver一致 |
| DYN-GATE-042 | prewarm共享device/cache、支持cancel/progress/shutdown，并通过冷/热启动与设备丢失矩阵 |

## 12. 测试与证据矩阵

| 层级 | 必需证据 |
|---|---|
| Contract | V8候选layout、capability negotiation、tagged event、disposition/page receipt、旧V7 hard-cutover/compat matrix |
| Identity | slot复用、ABA、跨host、跨DLL epoch、stale viewport/subscription/allocation、ID exhaustion |
| Concurrency | 多session、慢callback、慢query、event burst、destroy race、release race、wake coalesce、poison isolation |
| Startup | composition hash、project artifact hash、TOCTOU、partial failure rollback、report delivery、script/nav/UI closure |
| Rendering | headless unavailable、pipelined not-ready、4 viewport、resize race、device lost/restore、multi-platform surface |
| Performance | extract allocation/copy、session mutex hold、queue latency、world page complexity、UI closure scaling、prewarm cold/hot |
| Input/UI | timestamp replay、多设备/用户、multi-touch、focus/capture、IME ordering、multi-surface scale、a11y authority、hot reload |
| Event/world/operation | wake edge、cursor expiration、remaining/drop/resync、cancel/deadline、two-phase harvest、terminal drain |
| Shader | compiler/backend/device/driver key、real scene variants、shared device、cancel/progress、cache corruption、cross-build rejection |
| Product | App06不再依赖engine特判；App01在DLL unload前证明session/job/callback/allocation/viewports全部terminal |

## 13. 逐文件审查台账

### 13.1 Dynamic API根、frame与prewarm

| 文件 | 审查结论 / 后续归属 |
|---|---|
| `zircon_runtime/src/dynamic_api/mod.rs` | 公开V7、session、frame与prewarm；prewarm归属混入dynamic API，应迁Runtime09C/Plugins05 service。 |
| `zircon_runtime/src/dynamic_api/exports.rs` | 静态V7表与panic wrapper方向正确；host协商只看ABI，callback panic/host capability仍缺。 |
| `zircon_runtime/src/dynamic_api/bounded_json.rs` | byte/depth/item/deadline防线可保留；多次扫描、wall clock和非合作closure需重构。 |
| `zircon_runtime/src/dynamic_api/camera_controller.rs` | orbit/pan controller是示例级默认；应变成viewport/plugin policy并传播transform failure。 |
| `zircon_runtime/src/dynamic_api/frame.rs` | captured RGBA foreign output有上限和owner；frame disposition不能表达NotReady/Unavailable。 |
| `zircon_runtime/src/dynamic_api/runtime_loop.rs` | linked loop把宿主调度压成同步tick/capture；应消费session demand/receipt而非自成第二runner。 |
| `zircon_runtime/src/dynamic_api/session.rs` | 模块拆分清楚，但单个RuntimeDynamicSession仍聚合过多owner与产品适配。 |
| `zircon_runtime/src/dynamic_api/surface.rs` | 只翻译Win32 handle；应由platform surface provider和capability表接管。 |
| `zircon_runtime/src/dynamic_api/shader_prewarm.rs` | 固定pass/template、literal版本和静默fallback不具产品可信度；迁正式shader service。 |
| `zircon_runtime/src/dynamic_api/shader_prewarm/execution_budget.rs` | 有execution budget结构是基础；当前只约束串行worker参数，未接共享scheduler。 |
| `zircon_runtime/src/dynamic_api/shader_prewarm/module_validation_cache.rs` | 单批次RefCell cache能去重；缺跨job/device/build共享和并发owner。 |
| `zircon_runtime/src/dynamic_api/shader_prewarm/wgpu_validation.rs` | 真实WGPU validation值得保留；每次创建offscreen backend且同步执行。 |
| `zircon_runtime/src/dynamic_api/shader_prewarm/tests.rs` | 覆盖六pass、source与disk restart hit；缺真实产品variant、取消、设备/驱动/build失配。 |

### 13.2 Session construction、state与适配模块

| 文件 | 审查结论 / 后续归属 |
|---|---|
| `zircon_runtime/src/dynamic_api/session/construction.rs` | 逐步激活Core/project/UI/operation但无整体transaction；profile/composition必须单源。 |
| `zircon_runtime/src/dynamic_api/session/diagnostics.rs` | 能汇总runtime/render/reload诊断；缺session generation、queue/lock/budget和startup/shutdown receipt。 |
| `zircon_runtime/src/dynamic_api/session/error.rs` | typed source保留较好；部分边界仍压成字符串且bootstrap补偿可abort。 |
| `zircon_runtime/src/dynamic_api/session/event_mirror.rs` | subscription与分页已成形；裸ID、poll-only、无wake/remaining和session product identity错误。 |
| `zircon_runtime/src/dynamic_api/session/events.rs` | 覆盖多类host事件；宽struct语义、生命周期缺口和UI/gameplay路由不完整。 |
| `zircon_runtime/src/dynamic_api/session/extract.rs` | 入口小而集中；实际snapshot仍通过完整clone交付。 |
| `zircon_runtime/src/dynamic_api/session/extract_cache.rs` | revision cache可保留；key不完整且命中仍深clone。 |
| `zircon_runtime/src/dynamic_api/session/extract_stats.rs` | 主动暴露clone/byte是好基础；估算不是allocator truth。 |
| `zircon_runtime/src/dynamic_api/session/ffi.rs` | admission和输出指针检查较完整；所有业务仍在session锁内且能力false-success。 |
| `zircon_runtime/src/dynamic_api/session/highlight_set.rs` | FFI已验证后直接提交latest value；需要viewport generation、capacity和owner receipt。 |
| `zircon_runtime/src/dynamic_api/session/host_requests.rs` | payload分页存在；分类drain重排因果顺序，stale manager返回空。 |
| `zircon_runtime/src/dynamic_api/session/hud.rs` | sparse lookup优化可保留；`gameplay.hud_text`和Vampire文本分类必须迁App06。 |
| `zircon_runtime/src/dynamic_api/session/input_events.rs` | InputManager桥接集中；键位/重复/device/value validation不足。 |
| `zircon_runtime/src/dynamic_api/session/linked_plugins.rs` | 能收集linked registration；缺effective selection，易形成Runtime42所述旁路。 |
| `zircon_runtime/src/dynamic_api/session/linked_session.rs` | in-process入口复用construction；仍需host instance/generation和一致API语义。 |
| `zircon_runtime/src/dynamic_api/session/menu.rs` | 完整硬编码Vampire start/game-over UI和动态component写入，应从engine删除。 |
| `zircon_runtime/src/dynamic_api/session/operation.rs` | FFI adapter复用Runtime41 service；submit不wake、harvest commit failure被忽略。 |
| `zircon_runtime/src/dynamic_api/session/preview.rs` | 全黑frame与伪a11y tree制造false success，应改typed unavailable。 |
| `zircon_runtime/src/dynamic_api/session/profile.rs` | 六profile/固定步再次定义；收敛Runtime42/22唯一合同。 |
| `zircon_runtime/src/dynamic_api/session/project.rs` | prepare snapshot方向可保留；scene/nav/scripts/UI仍有直读、TOCTOU、同步与隐式全加载。 |
| `zircon_runtime/src/dynamic_api/session/runtime_ui.rs` | retained surface/input/a11y通路已存在；全扫描、ignored conflict、顺序ID、bit mask、单scale和无hot reload。 |
| `zircon_runtime/src/dynamic_api/session/scene_asset_reload_diagnostics.rs` | reload指标覆盖较好；counter以f64存储且缺artifact/generation/latency关联。 |
| `zircon_runtime/src/dynamic_api/session/script_systems.rs` | 会忽略传入plan并重建registry；必须消费冻结composition system rows。 |
| `zircon_runtime/src/dynamic_api/session/state.rs` | 聚合全部owner导致God object和单锁瓶颈；拆session actor、viewport/UI/world snapshot与job handle。 |
| `zircon_runtime/src/dynamic_api/session/status.rs` | status转换集中；thread-local借用diagnostic和自由字符串归Interface治理。 |
| `zircon_runtime/src/dynamic_api/session/world_sync.rs` | query/watch/page合同已有雏形；锁内decode/encode、无remaining和`remove(0)`必须修复。 |

### 13.3 Session registry

| 文件 | 审查结论 / 后续归属 |
|---|---|
| `zircon_runtime/src/dynamic_api/session/registry/mod.rs` | owner拆文件清晰；仍由进程全局状态统领，需HostInstance scope。 |
| `zircon_runtime/src/dynamic_api/session/registry/action_guard.rs` | RAII action退出可保留；应携带request/session generation和deadline。 |
| `zircon_runtime/src/dynamic_api/session/registry/allocation_registry.rs` | owner验证正确；缺累计budget、age、generation和revoke/quarantine。 |
| `zircon_runtime/src/dynamic_api/session/registry/frame_activity.rs` | thread-local wake reentrancy检测解决自等死锁；不能替代跨线程callback deadline。 |
| `zircon_runtime/src/dynamic_api/session/registry/frame_demand.rs` | demand merge方向可保留；producer不完整且clear/失败语义会丢请求。 |
| `zircon_runtime/src/dynamic_api/session/registry/session_slot.rs` | active action/callback计数是quiescence基础；等待无预算且state过少。 |
| `zircon_runtime/src/dynamic_api/session/registry/session_store.rs` | create/get/remove/retry路径集中；裸递增ID、无capacity和global scope不合格。 |
| `zircon_runtime/src/dynamic_api/session/registry/wake_registration.rs` | disable与callback census是好基础；同步callback无coalesce/rate/slow-host隔离。 |
| `zircon_runtime/src/dynamic_api/session/registry/tests.rs` | 覆盖owner、destroy race、wake reentrancy和allocation；还把poison recovery/无限等待策略固化。 |

### 13.4 Dynamic API边界测试

| 文件 | 审查结论 / 后续归属 |
|---|---|
| `zircon_runtime/src/dynamic_api/tests/mod.rs` | 测试按行为拆分良好。 |
| `zircon_runtime/src/dynamic_api/tests/support.rs` | 统一FFI fixture可保留；后续增加host instance、generation和fault injection。 |
| `zircon_runtime/src/dynamic_api/tests/api_table.rs` | 覆盖V7表和panic wrapper；缺capability negotiation与callback fault。 |
| `zircon_runtime/src/dynamic_api/tests/accessibility.rs` | 校验ABI/viewport；当前测试错误地接受伪preview snapshot。 |
| `zircon_runtime/src/dynamic_api/tests/frame_demand.rs` | 覆盖wake pair和carrier；缺多producer、ack、race和lost demand。 |
| `zircon_runtime/src/dynamic_api/tests/host_request_payloads.rs` | 覆盖IME/rumble/cursor编码；缺跨类别全序和ack。 |
| `zircon_runtime/src/dynamic_api/tests/host_requests.rs` | 覆盖分页不丢请求；缺stale manager、并发producer和sequence证明。 |
| `zircon_runtime/src/dynamic_api/tests/input_events.rs` | 覆盖wheel/scale/IME边界；缺完整键盘、gamepad finite/range、多设备和multi-touch。 |
| `zircon_runtime/src/dynamic_api/tests/linked_plugins.rs` | 覆盖linked event与Editor profile；缺effective project selection隔离和wake。 |
| `zircon_runtime/src/dynamic_api/tests/operation.rs` | 覆盖submit/poll/harvest与固定布局；缺wake、cancel、deadline、commit fault。 |
| `zircon_runtime/src/dynamic_api/tests/profile_control.rs` | 覆盖bounded JSON和snapshot；缺慢decode cancel、session work budget和并发tick。 |
| `zircon_runtime/src/dynamic_api/tests/session_entry_points.rs` | 广泛检查invalid/destroyed handle；缺slot reuse generation和cross-host handle。 |
| `zircon_runtime/src/dynamic_api/tests/session_lifecycle.rs` | 覆盖bootstrap前校验与销毁quiescence；现有abort期望需改隔离receipt。 |
| `zircon_runtime/src/dynamic_api/tests/session_profiles.rs` | 验证Headless不建bridge；未阻止Client/script能力泄漏和false capture。 |
| `zircon_runtime/src/dynamic_api/tests/structure.rs` | 只锁目录结构；不能作为运行时工程资格。 |
| `zircon_runtime/src/dynamic_api/tests/viewport.rs` | 覆盖默认viewport和Win32前置校验；缺多viewport、generation、resize/device/platform矩阵。 |

### 13.5 Session集成测试

| 文件 | 审查结论 / 后续归属 |
|---|---|
| `zircon_runtime/src/dynamic_api/session/tests/mod.rs` | 集成测试owner清楚；Vampire产品组应迁出engine。 |
| `zircon_runtime/src/dynamic_api/session/tests/foundation_render.rs` | 有真实basic scene/PNG/steady stats证据；需扩多viewport、device lost和无深clone阈值。 |
| `zircon_runtime/src/dynamic_api/session/tests/frame_demand.rs` | 验证animation demand；缺其余producer和失败保留。 |
| `zircon_runtime/src/dynamic_api/session/tests/frame_diagnostics.rs` | 直接证明cache命中仍一次full clone；Vampire性能用例ignored。 |
| `zircon_runtime/src/dynamic_api/session/tests/highlight_set.rs` | 覆盖canonical latest value；缺容量、stale viewport和并发提交。 |
| `zircon_runtime/src/dynamic_api/session/tests/lock_poison.rs` | 当前要求中毒后恢复；应改为invariant fault隔离和诊断。 |
| `zircon_runtime/src/dynamic_api/session/tests/runtime_errors.rs` | typed source链测试可保留并扩stable code/stage/correlation。 |
| `zircon_runtime/src/dynamic_api/session/tests/runtime_ui_surface.rs` | 多root/import/action/a11y覆盖较强；未覆盖alias冲突、顺序稳定、multi-scale和hot reload。 |
| `zircon_runtime/src/dynamic_api/session/tests/vampire_gameplay.rs` | 4/5真实VM用例ignored，且产品规则不应由dynamic session owner。迁App06/plugin。 |
| `zircon_runtime/src/dynamic_api/session/tests/vampire_hud.rs` | 3/3 ignored，证明engine与Vampire world HUD耦合；迁产品证据。 |
| `zircon_runtime/src/dynamic_api/session/tests/vampire_menu.rs` | 2/2 ignored，直接锁定engine硬编码start/retry协议；迁产品UI测试。 |
| `zircon_runtime/src/dynamic_api/session/tests/vampire_runtime_support.rs` | 504行产品fixture/support位于engine内部；整体迁App06或示例plugin测试包。 |

## 14. 完成定义

Runtime43只有在以下条件同时满足时才能由`pending`改为`implemented`：

1. 64项P1均有代码、测试、benchmark或明确转交owner的关闭证据。
2. 16项P2已完成或登记可追踪defer owner和退出版本。
3. 42个资格门全部通过，不能以单元测试数量、黑帧、伪tree或ignored产品测试代替。
4. V7到后继合同的切换有App/Editor/linked/DLL consumer矩阵，旧ABI按既定hard cutover策略退出。
5. 多session、多host、多viewport、多平台、device loss、hung callback和pressure测试提供稳定receipt。
6. Engine runtime不再包含Vampire/HUD/menu产品字符串、组件协议或点击规则。
7. Session shutdown在DLL unload前证明action、callback、job、allocation、watch、subscription和viewport全部terminal或隔离。
8. Shader prewarm绑定真实BuildSet/compiler/backend/device/driver并复用正式cache/device service。
9. steady frame、world paging、UI startup和prewarm冷/热路径达到预算并保存allocator/trace证据。
10. 重新执行source fingerprint、frontmatter path、Markdown link、P1/P2/Gate连续编号、BOM/CRLF/trailing whitespace校验。

在这些条件关闭前，不得把当前dynamic API称为工程级Runtime Session Control Plane，也不得把“FFI返回Ok”“能抓到RGBA”“测试文件存在”当作多产品、可嵌入或性能资格。
