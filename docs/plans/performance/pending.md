---
related_code:
  - zircon_app
  - zircon_runtime
  - zircon_editor
  - zircon_plugins
  - zircon_runtime_interface
  - zircon_hub
  - zircon_reflect_derive
  - examples
  - tools
implementation_files:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
plan_sources:
  - user: 2026-08-14 MVP-first whole-workspace performance review
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
tests:
  - docs/plans/performance/review.md
  - tools/audits/check_conventions.py
doc_type: milestone-detail
---

# 性能审查未验收模块

本表只记尚未同时完成“逐 `.rs` 文件复审 + 静态风险扫描 + 当前源码测试/产品剖析 + 问题处置”的模块。历史逐文件笔记与热点细节保留在编号计划及 `docs/plans/performance/01/`，不在总账重复展开。

结构性总门已转入 [`02-unreal-aligned-engine-system-hard-cutover.md`](02-unreal-aligned-engine-system-hard-cutover.md)：当前临时实现只作为瓶颈证据，最终 owner 固定为 app/runtime/editor 三根系统包；所有模块在 hard-cut 目标 owner、生命周期、线程和 extract 边界冻结前保持未验收。Unreal 源码基线见 [`02/2026-08-15-unreal-aligned-architecture-baseline.md`](02/2026-08-15-unreal-aligned-architecture-baseline.md)。

## 快照与守恒

- 快照：2026-08-14 当前工作树，现存受跟踪文件与 `git ls-files --others --exclude-standard` 可见文件取并集，排除 `dev/**`、`.codex/**`；Cargo build `target` 由Git ignore排除，不再按任意同名源码路径段过滤。
- 计数：受跟踪现存 `.rs` **16,504**，可见未跟踪 `.rs` **602**；集合去重并排除 `dev/**`、`.codex/**` 后为 **17,106**。旧规则曾误排除59个合法`src/**/target/**.rs`，本快照已全部纳入。
- 守恒：`review.md` **0** + 本表 **17,106** = **17,106**。同一文件只归一个表项；新增文件先进入本表，已验收目录出现源码漂移时整项退回本表。
- 优先级：先验收 F0 启动/退出、F2 场景帧与最小渲染、F4 编辑器基本作者路径，再处理非 MVP 运行时、插件、Hub、示例与工具。

## P0 · MVP 运行时与基本编辑器

| module folder | files | current state / acceptance focus |
|---|---:|---|
| `zircon_app/src/entry/**` | 159 | **159/159 current-source静态覆盖完成**：root+CLI 11、runner/runtime-library 25、runtime entry app 79（6,243行、97条内联测试）、entry tests 44；动态均未验收。host request当前仍为3个Vec take→第4个Vec→JSON encode/owned ABI/JSON decode→主线程全应用，且`tick_frame`内`begin_frame`清空早于App drain，真实`produce -> tick -> drain`行为门缺失；继续归PERF-MVP-425/Runtime10。78条entry tests中42行为/36源码形状；需current-source Cargo、WPR/Tracy、generation/queue counter、failure export与parity matrix |
| `zircon_app/src/bin/**` | 17 | 17/17 current-source静态审查完成；按需redraw、直呈、v4 fixture与单project open成立；RenderDoc新work-dir/无bridge证据缺口、启动阶段profile、warm HDRI源read、40条源码形状测试替换及Cargo/WPR/alloc/energy/capture待验收 |
| `zircon_app/src/plugins/**` | 5 | 5/5 current-source静态审查完成；局部builder非热路。入口assembly≥2、排序≥3，feature路径catalog≥2的跨层重复归`PERF-MVP-427`；0/1/100/1000插件WPR/alloc/Cargo待验收 |
| `zircon_app/src/tests/**` | 2 | 2/2 current-source静态审查完成；prelude/profile真实API行为成立，受管Cargo未执行 |
| `zircon_app` 其余 | 7 | 7/7 current-source静态审查完成；native路径先present且fallback counter应为0；4K/60Hz forced fallback的RGBA读+XRGB写静态下限3.71 GiB/s，GPU readback/WPR/timestamp/RenderDoc/energy待验收；process-log源码guard待行为测试替换 |
| `zircon_runtime/src/core/framework/picking/**` | 23 | **23/23、2,069 行生产代码已复审；动态未验收。** 单批 projection 已共享；primitive rays x all-primitives、event-state 扫描/临时分配待规模计数 |
| `zircon_runtime/src/tests/picking/**` | 6 | **6/6、889 行测试已复审；focused Cargo 0 次执行。** 受 unmanaged-artifact preflight 阻塞，仍不得进入 `review.md` |
| `zircon_runtime/src/core/**`（不含 Picking） | 835 | **TaskGraph current 32/32（5,119行、41 tests）及module/service lifecycle生产67/67（7,348行、11 inline tests；直接测试44文件、7,727行、107 tests）已逐文件/定向测试合同复审，动态未验收。** TaskGraph的三物理池、private pool、单pump平方退化和无界Drop归PERF-MVP-627；module/service的3,351行计数特化（范围45.6%）、stable拓扑重编、可重入状态、caller sleep-poll、假rollback、锁内析构、String/HashMap/global-condvar steady path和process-global index归PERF-MVP-628。证据见`02/2026-08-15-runtime-{taskgraph,module-service-lifecycle}-current-architecture-review.md`，由Plan02 M1与Runtime02/06/11、Editor01、Plugins01承接；两组生产/直接测试均rustfmt全绿，current Cargo/WPR/xperf/allocator/线程/功耗仍未验收。其余F0 frame/service继续复审 |
| `zircon_runtime/src/input/**` | 40 | **40/40、6,714 行、81 tests 已完成 current-source 静态复审；动态未验收。** Gamepad transition 的逐事件线性扫描已改为 frame-local index，语义测试已补；managed Cargo 已进入执行但由 foreign `zr_rhi_wgpu/render_pass_validation.rs:455` 缺失 `texture_view_layer_count` 阻塞，input tests 为 0 执行。空 `InputDriver`、逐事件 manager resolve/全局 mutex、无界 frame buffer/静默 clear、锁内完整 snapshot clone、无产品 action schedule 与无界 recording/replay 仍需 Runtime12/10 硬切。证据见 `01/2026-09-01-runtime-input-current-source-closure.md` |
| `zircon_runtime/src/scene/**` | 1,075 | **ECS schedule 52/52+测试13/13、query 29/29+测试21/21、deferred command 22/22+测试11/11、archetype/storage 37/37+测试11/11、observer/event/messaging 38/38+测试8/8、derived-state调用链11/11+测试9/9，以及render-extract直接生产链12/12+产品锚点8/8+测试13/13已静态复审；另以27/27个current跨owner锚点（8,992行、16 tests）完成帧级架构对账，动态未验收。** 604..620保留细项门，PERF-MVP-632要求`FrameScheduleGeneration -> WorldStorageGeneration -> WorldCommitGeneration -> SceneRenderGeneration -> FrameExtractGeneration`单链硬切；当前stage plan仍包裹String/move-out runner、data-bearing ECS主线程、逐target/lifecycle发布、五bool dirty、World/DTO clone及三套extract入口。证据见`01/2026-08-14-runtime-ecs-{schedule,query,deferred-structural-commands,archetype-storage,observer-event-messaging}-current-review.md`、`01/2026-08-14-runtime-scene-{derived-state,render-extract}-current-review.md`与`02/2026-08-15-world-ecs-frame-extract-current-architecture-review.md`；由Plan02 M2、Runtime03/07/08/10/11、Plugins01、Editor05、Render04/07/12/17承接。Cargo/counter/allocator/WPR/xperf/Tracy/energy/F2/F4/RenderDoc产品trace未完成 |
| `zircon_runtime/src/graphics/**` | 1,539 | **`visibility/**` 62/62、`scene/gpu_scene/**` 18/18、`build_mesh_draws/**` 33/33、`mesh_pass/**` 28/28、`core/.../render_compiled_scene/**` 28/28、`graph_execution/**` 59/59、`post_process/resources/**` 142/142、`temporal/**` 10/10、`history/**` current 7/7，以及`scene_renderer/hzb/**` current 7/7（1,660行、17 tests）已静态复读，动态未验收。** Temporal velocity覆盖已静态修成camera Clear+Store→object Load+Store，focused validator因协调器超时未进Cargo；TAA 4K源码模型207.36M loads/94.92 MiB写入。History确认graph后第二套owner copy为264.15 MiB/4K frame、15.48 GiB/s@60。HZB确认3个四mip批次仍执行成11/12个逐mipcopy/bind/pass，1024² single/MSAA4 logical traffic 37.33/85.33 MiB、2048² 149.33/341.33 MiB；四phase为12 clears+4 passes，且stale test反向要求旧staging copy。证据见`01/2026-08-14-graphics-{visibility,gpu-scene,build-mesh-draws,mesh-pass,compiled-scene,graph-execution,post-process-resources}-current-review.md`与`01/2026-08-15-graphics-{temporal,history,hzb}-current-review.md`，由PERF-MVP-346/350/366..376/378/381..389/391/395/399/405/419/420/622..625及Runtime04/11、Render01..09/17/18、Plugins01/04承接。其余graphics继续覆盖F2 CPU/GPU数据流、资源/图重建、draw/upload；current Cargo、counter、WPR/Tracy/energy/GPU timestamp及MVP RenderDoc capture未完成 |
| `zircon_runtime/src/render_graph/**` | 16 | **16/16、5,243逻辑行、49 tests已逐文件静态复审，动态未验收。** 产品authoring以`previous -> pass`形成全图总序；bare `usize` handle无graph generation/write version，WAW全传递闭包与unversioned culling保留被覆盖writer；compiled cache miss在framework锁内编译且精确extent key可抖动；compile plan和物理pool均exact-descriptor分桶；每帧stats重复`O(P^2*A)` store lint，7/49测试锁源码形状。证据见`02/2026-08-15-render-graph-current-architecture-review.md`，PERF-MVP-633由Plan02 M3、Render01/02/17、Runtime11承接generation/version RDG、linear frame arena、锁外publish、fence-aware resource authority与compile-time diagnostics。current Cargo/WPR/xperf/GPU timestamp/RenderDoc/功耗未验收，仍不进入`review.md` |
| `zircon_runtime/src/tests/**`（不含 Picking） | 1,251 | 当前行为门与复杂度门；删除仅锁定源码形状的 false-green |
| `zircon_runtime` 其余 | 2,841 | **plugin catalog/extension/bridge/profile current 135/135（12,342行、46 tests）及native loader current 88/88（27,846行、284 tests）已逐文件静态复审，动态未验收。** 外层重复catalog/target-only cache/full-registry revoke归PERF-MVP-629；native五个平行registry、跨plugin build lock、loaded锁内parse/prepare与逐plugin非原子batch归630。证据见`02/2026-08-15-runtime-{plugin-catalog-extension-bridge,native-plugin-discovery-live-host}-current-architecture-review.md`，由Plan02 M1/M5、Runtime06/11、Plugins01/11、Editor01/12/14承接。135/135与88/88 rustfmt通过；asset/UI/text/script/platform/bin/外部RHI继续复审，current Cargo/F0/F4/WPR/allocator/线程/功耗未验收 |
| `zircon_editor/src/core/**` | 470 | F0/F4 gateway、message、jobs、settings、save/export；Play controller当前在transition gate内同步plugin discovery/load/state callback/enter-exit，归PERF-MVP-631与Plan02 M1/M5，需typed lifecycle ticket、主线程零I/O/foreign callback/wait及有deadline rollback |
| `zircon_editor/src/scene/**` | 139 | **current 139/139（9,079行、60 tests）已通过旧证据+当前manifest/delta静态对账，动态未验收。** stable cache/ray broad phase已成立；changed generation仍有mesh深复制、per-renderable UiTree与post-submit owner树/二次surface，point event持shared mutex跨query/resolve并逐event建树，框选全扫M+G，scene-mode/provider callback位于shell/World主锁域。证据见`01/2026-08-14-editor-scene-viewport-interaction-current-review.md`；PERF-MVP-221/222/332/620/621由Editor05、Plugins01、Editor12、Runtime11、Runtime07/Render04承接。Cargo/WPR/Tracy/energy/F4/RenderDoc产品trace未完成 |
| `zircon_editor/src/ui/**` | 4,437 | **retained-host recompute current 13/13（1,608行、20 tests）+直接链13个文件（1,632行、21 tests），pane-payload current 3/3（332行、2 tests）+source/visibility/projection链7个文件（1,832行）已静态复读，动态未验收。** 纯`RENDER`误入Full已RED→GREEN剥离；scoped view仍`O(V*(F+W+sum(Fw)))`扫描/missing clone。pane targeted path已单source且callback锁外，但Full仍调用全部S个enabled source；native development host还为每plugin私建thread，并在watch-registry mutex内replace/Drop无deadline join，归PERF-MVP-631。PERF-MVP-103/106/113/595/626/631交接EditorUI08/01、Editor12/14、Plugins01、Editor05、Runtime11、Render17；focused Cargo/F4/WPR/线程峰值/energy/RenderDoc未完成 |
| `zircon_editor/src/tests/**` | 646 | F4 产品行为、规模 counter 与像素/交互门 |
| `zircon_editor` 其余 | 21 | crate root、build 与外部 tests |

P0 小计：**13,518**。

## P4 · 插件生命周期

| module folder | files | current state / acceptance focus |
|---|---:|---|
| `zircon_plugins/**` | 2,835 | MVP 首方 catalog/SDK/native hosting 先验收；navigation runtime lazy manager私建默认async-compute pool归PERF-MVP-627。插件module/service必须共用Plan02 M1/M5唯一catalog generation、compiled graph、strict phase、owner-indexed extension、TaskGraph affinity/deadline和stable ABI；native backend不得保留平行registry/global build lock/per-plugin watcher，batch原子发布且callback锁外，归PERF-MVP-628..631、Runtime06/11、Plugins01/11、Editor01/12/14；其余按可达性测发现/加载/注册/回调/卸载、全局锁、线程和队列预算 |

P4 小计：**2,835**。

## P5 · 非 MVP 与工具

| module folder | files | current state / acceptance focus |
|---|---:|---|
| `zircon_runtime_interface/**` | 416 | ABI/DTO copy、序列化与跨边界所有权 |
| `zircon_hub/**` | 130 | 启动、项目扫描、进程与 Tauri 命令路径 |
| `examples/**` | 174 | 只在对应产品功能验收后验证示例，不反向定义引擎架构 |
| `tools/**` | 28 | 构建/验证工具的扫描、进程与 I/O 成本 |
| `zircon_reflect_derive/**` | 5 | 宏展开与编译期成本 |

P5 小计：**753**。全表总计：**17,106**。

## 2026-08-31 closure detail

The following sub-scopes are retained as pending details of the existing
`zircon_runtime/src/graphics/**` and `zircon_runtime` rows; they do not add
duplicate files to the conservation totals:

- Framework budget/capability/frame-profiler/query-stats:
  `zircon_runtime/src/graphics/runtime/render_framework/{budget/**,capability_summary/**,frame_profiler.rs,frame_profiler/**,query_stats/**}`;
  12 files, 1,481 physical lines, SHA256
  `101999a880f0531cab7fb2fd2d5c08185b92bb8f3a2da083d306822d4df0c9f7`.
  Mixed traffic/residency budgets, viewport-count degradation cadence,
  camera/profile/query lifetime and split capability eligibility remain open.
- Asset payload/decode/assembly:
  `zircon_runtime/src/asset/assets/**`; 114 files, 24,135 physical lines, SHA256
  `f9665b59df0d5314230783e8a350fac671e2834e21bd59065cb61f29e1104eca`.
  Mesh/model duplication, multi-generation TOML conversion, texture assembly
  peaks, external/zcube face-cap bypass and deep payload clones remain open.

Evidence and architecture gates are recorded in
[`01/2026-08-31-framework-budget-and-assets-closure.md`](../_archive/performance/01/2026-08-31-framework-budget-and-assets-closure.md).
Both scopes remain pending until current-source Cargo, scale counters and F0/F2/F4
product validation are available; no static-only acceptance is recorded.

### 2026-08-31 runtime event consumer closure

See [`01/2026-08-31-runtime-event-consumer-closure.md`](../_archive/performance/01/2026-08-31-runtime-event-consumer-closure.md) for the 16-file retained-host event closure (3,754 lines; SHA256 `4a2024eec6a2cc655519f541223d6b238b8fc0d76385f7024a32bf34f2947add`). The current foreign implementation is preserved, but host admission still follows destructive producer drain, retained-byte accounting omits decoded/object peaks, stable capability reconciliation and empty drains repeat per tick, callbacks lack enforceable affinity/deadlines, lifecycle reconcile is not atomic, and delivery/session terminal generations are split. Route to Editor01/12/14, Runtime06/10/11 and Plugins01; remain pending until managed Cargo and retained-host F0/F4 evidence.

### 2026-08-31 editor gateway closure

See [`01/2026-08-31-editor-gateway-closure.md`](../_archive/performance/01/2026-08-31-editor-gateway-closure.md) for the current gateway/session/route scope: 25 core Rust files (4,351 lines) plus 11 external test files (2,332 lines). ArcSwap gateway origins and operation/pick routes are positive, but world watches, invalidation drains, capture and other child lifetimes are not universally identity-pinned; request serialization/output projection lacks one end-to-end proposal; in-process callbacks hold the world lock across arbitrary work; and capability/lease generations can become stale or repeat work. Route to Editor01/02/04/12/14, Runtime06/10/11 and Plugins01; remain pending until managed Cargo, gateway scale/lock tests and F4 evidence.

### 2026-08-31 editor message closure

See [`01/2026-08-31-editor-message-closure.md`](../_archive/performance/01/2026-08-31-editor-message-closure.md) for the current 38-file core message scope plus 14 external test files. Arc payload fan-out, handler-outside-lock dispatch, indexed inbox eviction and typed topic/schema validation are positive. UI delta queues remain unbounded under the bus mutex; inbox byte estimates omit nested allocations and full drains hold a consumer lock; fanout/report vectors scale with subscribers; and message/runtime-event/gateway callback lifetimes do not share one terminal delivery lease. Route to Editor01/09/12/14, Runtime06/10/11 and Plugins01; remain pending until managed Cargo, delta/drain scale tests and F4 evidence.

### 2026-08-31 editor event closure

See [`01/2026-08-31-editor-event-closure.md`](../_archive/performance/01/2026-08-31-editor-event-closure.md) for the current 40-file event core scope plus 32 external test files. Three retention classes, Arc-shared records, cursor pages, normalized listener filters and lock-free listener projection are positive. Per-event JSON serialization for byte measurement, successful-path full record cloning, full journal snapshot rebuild under lock, listener fan-out without one end-to-end lease, and saturating identity counters remain open. Route to Editor01/09/12/14, Runtime06/10/11 and Plugins01; remain pending until managed Cargo, event scale/lock/identity tests and F4 evidence.

### 2026-08-31 editor jobs closure

See [`01/2026-08-31-editor-jobs-closure.md`](../_archive/performance/01/2026-08-31-editor-jobs-closure.md) for the current `zircon_editor/src/core/jobs/**` scope: 58 Rust files, 11,423 physical lines, 10,211 nonempty lines, 149 test attributes, 14 ignore attributes and 19 include sites; sorted raw-content SHA256 `b919c077e0d73fa8ca5aee1d1f0a6277958fdc2e19de2a6cae74acb246f23adb`. Finite category/pending limits, indexed fairness and dependencies, keyed latest-work replacement, indexed terminal retention, Arc labels, bounded event journal and panic-contained observers are positive. Editor still owns a peer scheduler in front of Runtime TaskGraph; admission does not reserve result/lifecycle/message delivery capacity as one lease; completion can recursively drive observer and promotion; named product callers can block on `JobTicket::wait`; lock scopes cross progress/journal/bus delivery; shutdown and identity exhaustion lack terminal-generation receipts; and stable status/full snapshots retain a cloning escape hatch. Route to Editor01/02/09/12/14 and Runtime06/10/11; remain pending until the TaskGraph facade hard cut, terminal-lease/affinity tests, managed Cargo and F0/F4 evidence.

### 2026-08-31 editor asset closure

See [`01/2026-08-31-editor-asset-closure.md`](../_archive/performance/01/2026-08-31-editor-asset-closure.md) for the current `zircon_editor/src/core/asset/**` revalidation: 53 Rust files, 11,628 physical lines, 10,419 nonempty lines, 384,429 bytes, 113 test attributes, 12 ignore attributes and 16 include sites; sorted raw-content SHA256 `396a016e7aee0f75fdadfcf38a27a2dcf39a98a02f35cd42a7a20bc41346cc7e`. Dirty generation cursors, import single-flight, runtime topology preflight, typed refactor tickets and staged type-registry batches are positive. Dirty/save/import/index/catalog remain parallel authorities; dirty deltas clone effect maps under lock; import flight/result waits and observer clones are not one nonblocking lease; index/catalog projections remain full-build or dormant paths; production capability materialization bypasses the batch core; and source/decode/result/output budgets and identities are split. Route to Editor03/09/14, Runtime04/11, Plugins12 and the editor asset/UI owners; remain pending until the shared AssetOperationProposal/AssetPayloadGeneration, managed Cargo and F1/F4 evidence.

### 2026-08-31 runtime text document and layout closure

See [`01/2026-08-31-runtime-text-document-layout-closure.md`](../_archive/performance/01/2026-08-31-runtime-text-document-layout-closure.md) for the current 70-file document/layout scope: 18,072 physical lines, 16,665 nonempty lines, 640,024 bytes, 265 tests, three ignored manual tests and one include site. Piece storage, prepared edits, store budgets, local hard-line updates, conservative ASCII grapheme splicing, bounded caches and plain no-wrap viewport materialization are positive. Editable content remains split between full Strings and the piece store; replacement admission follows candidate vectors; line/index candidates are not aggregate-budgeted; layout cache hits deep-clone line/run payloads; artifacts duplicate retained lines/text; rich tables shape cells twice; and viewport materialization is domain-narrow. Route to Text09, EditorUI03, Runtime10/11 and Render17; remain pending until one content/edit/layout generation, pre-admitted peaks, Arc-shared rows and managed F4 evidence.

### 2026-08-31 runtime UI layout closure

See [`01/2026-08-31-runtime-ui-layout-closure.md`](../_archive/performance/01/2026-08-31-runtime-ui-layout-closure.md) for the current 33-file UI layout scope: 9,672 physical lines, 8,924 nonempty lines, 324,559 bytes, 60 tests, two ignored manual tests and two include sites. Indexed slot lookup, local arranged/hit/render/navigation patches and first-party materialized fixed-list virtualization close the earlier blanket full-scan/full-rebuild claims. Taffy still reconstructs nodes for every affected container; engine-selection diagnostics are unconditional; the generic virtual-scroll fallback scans children and clears offscreen subtrees; responsive dirty rows retain owned metadata; scratch high-water residency is unaccounted; and downstream patch generations remain split. Route to EditorUI02/08, Runtime10/11 and the shared diagnostics owner; remain pending until one surface/Taffy/virtual-provider generation and current F4 evidence.

### 2026-08-31 runtime UI input and dispatch closure

See [`01/2026-08-31-runtime-ui-input-dispatch-closure.md`](../_archive/performance/01/2026-08-31-runtime-ui-input-dispatch-closure.md) for the current 118-file UI input/dispatch scope: 21,811 physical lines, 20,207 nonempty lines, 750,080 bytes, 152 tests, 20 ignored tests and 21 include sites. Cached routes, inline visited nodes, bounded text updates, checked number-field revisions and Arc drag payloads are positive. Default Full diagnostics and incomplete Summary gating still own route/event/label projections; multi-effect rollback clones broad surface state; focused text/IME changes can rebuild the complete surface render extract; timer maps are fully scanned and expired bursts drain synchronously; and clipboard/pointer/analog identities retain split payload and dynamic-key work. Route to EditorUI08, Text09, Runtime10/11 and the shared diagnostics/platform-input owners; remain pending until one input/surface/text-layout generation, narrow mutation journal, bounded timer/host leases and current F4 evidence.

## 当前全局阻塞

- 当前产品bundle入口在Cargo前稳定失败：`tools/build/build-editor.ps1:130`以PowerShell字面量`'\\'`拼接approved root，导致合法D/E/F输出无法通过containment匹配。默认D盘和显式E盘调用均复现；`tools/tests/build-editor.Tests.ps1`当前15项为**9 pass / 6 fail**。精确移交见`01/failure-2026-08-15-build-editor-approved-root-separator.md`与PERF-MVP-634。该脚本及测试已有foreign-dirty改动，Performance01未越权覆盖；在15/15、managed bundle与`--help`恢复前，WPR/xperf/RenderDoc仍未启动。
- Windows managed validator dry run已生成D盘target命令。自建E盘空temp目录曾被`unmanaged_artifacts_detected`拒绝，已由coordinator精确删除且artifact audit为0；build-only `zircon_app`矩阵在D盘managed target运行324.2秒，以212条warning、6个foreign-dirty `zircon_runtime`错误失败。随后focused `zircon_runtime` lib-test又运行843.4秒，以361个编译错误、1,520条warning失败，0条test执行；tests/WPR/Tracy/RenderDoc current binary均未运行。本Session Cargo作业已结束，不抢占、不改用裸Cargo。
- `python tools/audits/check_conventions.py --repo-root . --only docs --json` 在并发硬切期间从 **612 / 212 documents** 漂移到本轮末次 **638 / 229 documents**（2,382 documents、74,308 checked paths）；本次持有并复核的 performance 文档为 **0 violation**。全局问题保留在 `01/failure-2026-08-02-doc-structured-path-owner-drift.md`，不以批量猜路径消除。
- RenderDoc 1.44 可用，但已有高级 volumetric capture 不是当前 F2 MVP 场景；在可运行产品入口和 managed build 恢复前，不用旧 capture 冒充当前 GPU 基线。
- 当前首轮受管 Runtime baseline `cargo check -p zircon_runtime --lib --locked` 使用 `E:/cargo-targets/zircon-perf/m0-runtime`，编译至 `fontsdf`/`zstd-sys` 时因 Cargo 在 Windows 路径混合分隔符下无法写入 dep-info/fingerprint（os error 3）失败；没有任何测试执行，不进入 `review.md`。下一步改用仓库既有 coordinator/Windows approved target 机制重跑，避免并行共享 target。
- 2026-09-02 文档约定门使用 `python tools/audits/check_conventions.py --repo-root . --only docs --json`，报告写入 `E:/cargo-targets/zircon-perf/m0-docs-conventions.json`：受当前全工作树影响为 **1,619 violations / 5,294 documents / 91,836 checked paths**，含 22 repository escape、1 absolute path、1,596 missing path；性能计划自身不能据此宣称 GREEN。该全局问题路由既有 documentation-path failure，禁止在本计划批量猜路径或覆盖其他 owner 文档。
- 2026-09-02 复跑非 Cargo 契约：App 请求编码性能契约与 Cargo product receipt 契约共 **74/74 passed**；插件结构审计 `E:/cargo-targets/m0-plugin-structure-rerun.json` 为 `classified-and-clear`、`sample-clean-migration-debt-clear`、`runtime-capability-single-source-clean`，但均为静态/源码契约，不构成 Runtime/Editor/Plugin 动态验收。
- 2026-09-02 受管 Runtime 动态门再次被 coordinator 拒绝：`maintenance_hold_active`、`state=draining`、`lastReasonCode=tray.recovery_circuit_open`；不得绕过 coordinator。Runtime/Editor/Plugin Cargo 测试、CPU/GPU/RenderDoc/功耗数据继续保持 pending。


### 2026-10-09 current-source 调度与编辑器复核

沿用[当前执行流 7.25](2026-10-04-architecture-audit-flow.md#725-当前调度生命周期与-ui-owner-复核2026-10-09-utc)。本轮不改变 2026-08-14 的历史文件计数、不增加已验收模块。

Runtime03/ECS 的 44 文件候选正在关闭整帧/整回放归属、有效注册发布和替换 World 的 stale tail。有限复核还定位到 direct spawn 生命周期事件丢失、三个候选编译类型错误及取消报告丢失已记录错误。原始物理字节重放与格式解析只是实现切片证据；新的生产路径回归、当前 Runtime package 检查和 scene/product 验证仍 pending。项目激活九文件已经在 Main，直接 mutation 的旧 activation 拒绝回归等待同一当前输入批次。

[UI Core 独立判断](../../../.codex/sessions/performance-audit/reviews/ui-core/core-owner-decision-20261009-r1.json) 绑定 48 个 current raw pins，保留 18 个 foreign 已采用 postimage，旧候选严格投影 11/12、toolbar 原 before 缺失。十个 workspace 文档实际事件数为 139，另有 17 个 chrome producer 义务；当前缺失资产测试、选项派发 payload、样例完整 host dispatch、lazy-mount 三项缺陷及 96 个组件强可达门保持开放，归现有 owner 的下一窗口。Details V3 为已集成源码，仍非动态接受。

本轮三个 Luna/max 文件审查因子代理缺少文件工具而未执行，不计覆盖。Sol/max 核心实现与独立复核继续。当前聊天无法向指定 Jenkins chat 发送消息，verified active 状态、当前产物、真实工具捕获与五次前/五次后性能样本均未完成；当前编译先满足 35 GiB 容量及物理盘根门槛，结果只能按其实际证据范围记录。

### 2026-10-09 App V8 与公开生命周期有限结果（当前仍待验收）

[owner／调用链及准确读取范围](../../../.codex/sessions/performance-audit/m0-owner-boundaries-r1.md) 已补。remove_entity、typed remove 与 compact publication 的原 owner 替换风险仍为 preexisting P1；当前 44 文件 ECS 集成不关闭这些公开路径。App 的 CR-APP-ENTRY-0003 在 target-editor-host 下包含旧 linked API 消费者及基础身份失配；Root 保留现有非零身份契约，候选编写中，Main 未采用。两份核心有限审查均无 Cargo 或动态行为执行；搜索 pin 不提升为完整源码覆盖。三 Luna 委派的工具阻塞不计审查，整模块已验收计数不变。F drive-root 与实际 Jenkins 发送／接受、当前原生基线和性能门仍开放。


### 2026-10-09 App r2 已集成，公开 lifecycle 完整消费者仍开放

沿用[执行流 7.28](2026-10-04-architecture-audit-flow.md#728-app-身份源码集成与取消消费者窗口2026-10-09-utc)。App 五叶已获精确独立复核并集成 Main；5/5 after、scoped diff 0、Runtime 59 输入和共享 index/lock 保持，当前没有编译/行为结果。永久非零 owner 身份与真实 Gateway provider 回归保留；旧 report.remove 消费者静态修复，真实 navigation bootstrap 和双 provider resource_scope 执行仍 pending，不关闭 CR-APP-ENTRY-0003。

公开生命周期 13 后像仅有界源码批准、仍 RAM；18 条新测试未执行。records/detached replay、Editor Unchanged/Applied 补偿与 fault/history/replay、move-only 取消 payload、Physics cleanup 是下一统一消费者窗口。已发布 rows 的取消不承诺回滚；restore preflight intact batch 与消费后取消必须分开；不把更换 World 后的旧 batch 自动补偿回新 World。entry flush 替换假设已按实际无 callback 链排除。两个核心设计任务分属 scene cancellation 和 dynamic plugin bootstrap；非 MVP 实施继续遵守基础门槛。

原生检查仅有 D/F 控制准备和真实 prelaunch 失败，无 Cargo 启动。D 35 GiB 空间/F 父目录、Jenkins 实际消息送达和接受、当前产品与工具捕获、五次性能样本仍开放。不同 target lane 的并行 package 验证不受 Root 旧全局 any-Cargo 前检约束；性能采样单独独占。三 Luna 明文重试仍无有效审查；整模块已验收计数和 review.md 不变。


### 2026-10-10 当前原生基线修复与取消消费者

沿用[当前执行流 7.29](2026-10-04-architecture-audit-flow.md)。原 D 生产 Cargo check exit 0、外层控制因十项外来 UI 输入漂移 exit 1；后续 Runtime libtest no-run 实际 exit 101，五条诊断、四个原因，列表/测试执行 0。旧终态、Job 身份、EOF、选定输入和输出均保留，不登记当前编译接受。

[五个最小测试消费者修复](../../../.codex/sessions/performance-audit/runtime-native-baseline-fix/integration-r2.json) 已在 Main，独立审查和 5/5 after 匹配；失败写入 sourceWrites 0 的旧 receipt 保留。新的 [source context r2](../../../.codex/sessions/performance-audit/ecs-current-source-context-r2.json) 精确继承 59 项并覆盖四个测试 pin、显式加入 UI 测试。修复后的当前编译及行为等待 F 原生批次。

[生命周期统一范围](../../../.codex/sessions/performance-audit/lifecycle-cancellation-source-scopes-r3.json) 为 59 个唯一文件，其中 57 个仍为 RAM 候选，另两项是已合入基线：Shared10/Core22/Engine6/Editor6/Asset9/Physics4。Core25 新声明、Engine/Editor 与 Physics 新行为尚未执行；有限源码批准不进入已验收账本。Shared 的两项旧源码错误 token 消费者已补十叶候选。Asset r2 的首次取消 marker 未覆盖公开事件入口，被独立复核要求修改；第九 events 叶和真实 transform/wheel owned transport 回归由原 owner 修复中。Context 同时重复 callback/dispatch Err 的诊断丢失保留下一窗口，没有在本批改写 Context。

[F 正向物理和写入前检](../../../.codex/sessions/performance-audit/native-f-preflight/receipt-r1.json) 通过，F 实际现有父目录和授权 zircon-local 下唯一 probe 保留，空闲 102,910,431,232 bytes ≥35 GiB；不代表 native 或完整权限 enforcement。F 原生控制准备后先验证当前 Main，执行期间暂停本轮源码集成。Jenkins 消息仍未送达；E:\zircon-profiles 写入、当前产品及工具捕获、五次前后性能、MVP 00/F0–F5、模块和里程碑接受全部开放。历史计数、外来字节、共享 index/lock 与 review.md 保持。


### 2026-10-10 独立取消复核与第二次输入漂移

沿用[执行流 7.30](2026-10-04-architecture-audit-flow.md)。F-r3 与唯一一次 F-r4 launch 均在 sourceObservation 前检因外来输入漂移退出，原生命名空间未创建，Cargo/list/tests 0；不误记 Rust 失败或通过，不重提封存请求。r3 的三 Picking 叶是已核对的注释/空行变化，仍保留真实观察；r4 新漂移待分类。F-r4 packet-only 错误及 source-focused successor 独立保留，原完整 fresh receipt 未取得的事实不改。

[Shared10 独立源码复核](../../../.codex/sessions/performance-audit/reviews/lifecycle-unified-review/shared-r2-bounded-r1.json) 有界通过。[Asset9 r4 独立复核](../../../.codex/sessions/performance-audit/reviews/lifecycle-unified-review/asset-r4-bounded-r1.json) 静态关闭公开事件入口、真实测试 API 与两项 state 行数预算；state 796，十条声明执行 0。Root 57 叶统一 RAM raw replay/Git check 成功后又发现 Core1 typed_api 801 越界及第三 World restore 夹具未真实建立兼容 storage；统一包不获集成接受。原 owner 正在同 22 叶内修两生产、一测试，剩 19 后像保持；Core2 及独立再审 pending，Main 的最小五项基线修复保留。

[Bootstrap 独立设计](../../../.codex/sessions/performance-audit/reviews/runtime-bootstrap-independent/runtime-bootstrap-independent-review-r1.json) 确认 Runtime construction 为最低启动 owner，before-first-effect 清理/保留、companion 配置与 public export、真实产品 caller 仍未闭合；当前 abort 与 V8/V3/V4 外层契约不改，新的 ABI/layout/实现未批准。Context 诊断合同另作只读，不写入该窗口。当前 native、产品/工具捕获、五次前后性能、Jenkins 送达与接受、MVP 00/F0–F5 和整模块/里程碑仍开放；旧 Source/diff、共享 index/lock、失败回执及已验收计数保持。


### 2026-10-10 统一 57 文件已集成，当前编译与行为待验收

沿用[执行流 7.31](2026-10-04-architecture-audit-flow.md)。Core2 三叶修正经独立有界复核，Root [r2c 实际集成](../../../.codex/sessions/performance-audit/lifecycle-unified-integration/integration-r2c.json) 完成 57 文件，最终字节与 scoped diff-check 通过；新 [context r3](../../../.codex/sessions/performance-audit/ecs-current-source-context-r3.json) 记录 110 个唯一当前 pin 和十二项 supersession。HEAD/index/原空 lock 保持；没有 compiler-input seal 或当前原生接受。

[验证计划](../../../.codex/sessions/performance-audit/reviews/lifecycle-validation-closure-r1/package-validation-plan-r1.json) 覆盖原始 31 项、本轮 68 项新增声明及完整四 package 消费者，三编译阶段合计 248 普通测试加八项 guard，执行仍待完成。先恢复原始 Runtime31。F-r5b 控制修正与独立复核尚未 prepare/launch，旧 r3/r4 前检失败保持。event_processing 的既有全局行数预算风险仍开放。Context5 仅 RAM 候选和独立复核，不混入基线。Bootstrap 的新输入、before-first-effect owner 和真实产品调用方尚未实施；现有 abort 与 V8/V3/V4 契约保持。当前源码/native、产品、性能、Jenkins 送达/接受、模块及里程碑未验收，已验收计数不增。


### 2026-10-11 当前原生失败终态与 E 盘产物门槛

沿用[执行流 7.32](2026-10-04-architecture-audit-flow.md)。实际 F-r5c 一次 Cargo 依赖获取因 Schannel SEC_E_NO_CREDENTIALS 退出 101；Job、进程与 EOF 已归档，列表/测试执行为 0。921 个锁定 archive 与 807 个 sparse 文件的离线供应已核验；1,729 文件的 E 盘供应后继尚未执行，不登记编译通过。

当前 context r4 精确核对 110 项并显式保留一项外来测试 helper 修正；外部 Git 提交、index 与 lock 变化保留历史，不制造跨历史保护一致性。Context5＋event2 的七文件集成尚未完成；r2 证据读取比较错误将 atime 变化当成修改，原失败 Source=0，r3 校验修正进行中。

新产物按实际配置使用 E 盘，并先登记稳定任务 artifacts.json，状态 active、实际释放 0。E 空间低于 35 GiB 门槛约 14.36 GiB；共享 E 默认构建树和未知历史产物保留。拟执行根工作区 249 项与另行 Physics11 项均未接受；当前产品、性能、Jenkins 送达/接受、MVP 与模块里程碑门槛仍开放，review.md 和已验收计数不变。


### 2026-10-11 七文件与离线供给已完成操作核对，原生验收待执行

沿用[执行流 7.33](2026-10-04-architecture-audit-flow.md)。Root 已实际写入 Context5＋event2 七文件；原始 primary 与 protocol/dispatch 顺序保留、event_processing/task_admission 为 658/165 行。实际写入、截断、读回及保护输入经独立操作核对通过，父 context 110 项加有限 delta 后为 114 项；静态操作接受未提升为 Rust 类型、全局 guard、行为或产品接受。

E 供应根已实际复制并读回 1,729 文件、208,979,433 payload 字节，645 子目录及两个辅助回执，child exit 0、wait/EOF 完成。独立审计中的整数报告显示舍入已用有限 delta 更正，未来校验使用原始准确整数回执。Cargo 尚未执行该供给；不可变供给和各阶段私有 CargoHome 的职责保持。

用户释放空间后 Root 核实 E 剩余约 205 GiB，35 GiB 磁盘门已恢复，launch 时再查。稳定 artifacts.json revision 2 登记实际交付，初始清单原样归档；Root 实际释放 0、任务 active，外来与共享产物保留。当前 E 原生控制与独立审查进行中：Runtime 191＋3、Interface 2、Editor 48＋5，共 241 功能＋8 guard，功能全部先于 guard。Physics11 与 72 个额外版本供应继续开放。三个 Luna/max 的实际有限模块阅读进行中，不计整模块验收。

当前原生编译/测试、MVP、产品与性能均待验收；Jenkins 消息仍未送达，原始 profile 写入 grant 未具备。已验收计数和 review.md 保持。


### 2026-10-12 当前完整准备通过，原生恢复与工具能力仍待验收

沿用[执行流 7.34](2026-10-04-architecture-audit-flow.md)。当前 prepared 完整观察 20,119 个源码/相关输入并通过；随后 runner 的一次启动在 CONFIG 初始化前因 WindowsStorage create=True 对只读 cargo-targets 祖先请求写入失败，exit 2、wait/EOF 归档。禁止 FILE_CREATE 的有限诊断确认 Errno 5；空 execution 身份已绑定原 failed command 与 prepared，CONFIG/task-parent/TEMP 仍 absent。R5 将仅从授权 zircon-local 锚点相对创建同一配置，Root explicit recovery launch、完整 guard、旧 metadata custody、新 control/review pins 和逐层创建结果仍待复核与执行。没有当前编译或测试通过。

资产准入、普通编辑诊断和启动 custody 的三个 Luna 审查已实际完成，静态风险分别归并到 queue owner、既有诊断消费者、启动 construction owner；动态复现和完整模块处置保持开放。新增有限阅读确认 Hub→App→retained Editor 入口与 cleanup 可见，但首帧、World 激活、保存重开与退出仍未执行。RenderDoc v1.44 安装可见但 MCP disabled；Tracy feature/caller、GPU timestamp 实现及 WPR 命令存在不证明当前产品可采集，原始 E profile 写入授权仍未具备。

Root 新一轮释放 0，任务与稳定产物保留。Physics11/额外版本供应、当前 native、MVP、产品与五次前后性能、Jenkins 送达/接受及模块/里程碑继续 pending；已验收计数和 review.md 不变。实际 checkout 校验与产物 staging 沿用当前用户授权，不创建计划历史措辞中的 Source validation copy。

### 2026-10-12 恢复启动因共享源码变化终止，后继绑定新 epoch

沿用[执行流 7.35](2026-10-04-architecture-audit-flow.md)。R5-r2 build-parent 单组件创建方案完成 Root 阅读和 Sol 独立复核；既有未 join 异常的 Job/目录持有 caveat 保持开放。实际恢复 child PID 2344 运行 684.105 秒后 exit 2，wait/EOF 完成，失败为 original prepared snapshot 不同。CONFIG/task/TEMP/request/receipt 仍 absent，Cargo/test 未开始；未把完整准入 passed 当编译或产品通过。

当前 full-admission Source fingerprint 50f2ac678f882b82a86edff74d13fbfbeb458a57290a319a9cce23e9b37e49b3，20119 项；16 个共享 Editor/input 叶变化，20103 项及其余 snapshot 字段、五组 protected/tools/environment/supply/supplyIdentity 保持。精确 before/after 与现稳定身份由 Root drift 4b710fa4445eb02a0012aff1899ffb2ba5c46237a0554a5f3b0bcceaa154a302/18614 记录。保留所有已有修改；r2 claim 已消费，新 Root Source-epoch token/控制/独立复核待完成，不重发旧启动。三路 Luna 对 16 叶只读，Sol 辅助最小更新；供给、critical114、选择和 Native tail 不改，后续仍对新 baseline 严格校验。

新增 App 生命周期和 typed cleanup 责任风险继续只读归因；Tracy feature 默认未启用、调用方忽略安装结果，日志 sink 自有实现不足以证明全局 subscriber 冲突根因。当前正常/失败退出和 worker/callback quiescence、首帧、World、保存重开仍需实际产品证据。工具捕获及 E 原始 profile 写入 grant、五次前后性能、Jenkins 送达/接受和 Physics 供应均开放。Root 本轮释放 0，任务 active，失败证据保留；当前 native/MVP/产品/模块/里程碑未接受。

### 2026-10-12 epoch 复核完成，当前编译入口变化需重新交接

沿用[执行流 7.36](2026-10-04-architecture-audit-flow.md)。R5-r3 精确 16/20103 投影、旧 22 metadata 和 bounded control 已经 Root/Sol 独立复核，未提升为 Native 接受。final contract 75c653b5…/25904 只换两个 review refs。实际 child19932 在 06:31:09 UTC、0.534 秒后 exit2、wait/EOF 完成，before-admission 被 local_cargo.py 指纹变化拒绝；命令回执 f2ccc854…/4091。r3claim/admission/build/TEMP/request/receipt absent，Cargo/tests=0，原 approval 保留、不重试。工具当前 e9259f18…/29471 的 main 转普通 Jenkins；三路 Luna 核对 driver、身份消费者与 helper，Sol 核对固定 pins，后继据实际可用入口及原任务授权交接，不改配置、不创建源码副本或恢复退役服务。

Input12＋Editor4 的 epoch 全文阅读完成 6379 行，但同帧手柄断连重连、混合单位、设备代际和 hide 部分失败/token 等仅静态风险。CR-INPUT-FRAME-0001 已确认测试查找不存在的 drain_host_requests marker，现实现为 collect_host_requests；这是守卫缺陷，非产品帧顺序故障，且不在本次 249 选集，完整 Input 回归仍开放。App 最低方案为 typed failure 的原线程、本 owner 条件重试；全局 ThreadId registry 强持有及原线程退出后的恢复缺口仍未解决，现无 owner 测试不能验收 DLL 生命周期。

当前完整基线仍须新观察；114 Source 与 build/related 固定 pin 核对已完成，六项预期缺失全同，仍不替代当前全树观察。Root 本轮未写生产源码/共享 helper/Git，释放 0，任务 active；供给、原控制及失败证据保留。Native、MVP、产品、性能、工具、Physics 供应、Jenkins 送达/接受及整模块/里程碑继续 pending，review.md 和已验收账本不增加。

### 2026-10-12 App 清理与 Input 守卫切片已集成，等待合法原生交接

沿用[执行流 7.37](2026-10-04-architecture-audit-flow.md)。App 创建失败现在在创建线程仅重试本错误的 pending owner 一次，保留 primary 与清理次级诊断；三项真实 session/owner 回归已写入并经 Sol/Luna 独立静态复核。完整测试 features 为 dynamic-api,platform-winit 且关闭默认 features，后续必须确认三例实际编入并运行；默认配置和单 dynamic-api 都会过滤这组。DLL 卸载、持续失败后原线程退出的恢复与 CR-APP-ENTRY-0003 整体仍开放。

CR-INPUT-FRAME-0001 的故障重复用例删除，真正 tick→begin_frame owner 静态 guard 与 manager 行为用例保留；三个消费者完整迁移。event_buffer 纳入二十三例总清单，失焦 marker 和两处旧文档路径更新；重复泛化说明精简后 frame_state 为 759 行，原 <800 门槛保持。静态路径/锚点、RAM inverse 与 scoped diff-check 通过；二十三例、三项契约/预算守卫、owner guard 和真实 Level observer 行为均尚未执行。

当前六叶 before/after 与受管参数记录在[既有 Jenkins 交接正文](../../../.codex/outbox/engine-audit-20261009-01a12097/jenkins-runtime-baseline-handoff-r2.md)，尚未发送。E 约 201.6 GiB，35 GiB 门恢复；Root 原始 Jenkins identity/authorization 未具备，等待合法交接，不借其他 chat actor、扩权限或重试旧 runner。当前 Native=0，完整源码/产物基线未建立；MVP、产品、工具与五次前后性能、Jenkins 和整模块/里程碑接受继续开放。无新构建/采样产物，实际释放 0，供给及历史/共享证据保留，已验收账本不增。

### 2026-10-12 Jenkins 本地任务消息已留存，接收与原生验证待回执

沿用[执行流 7.38](2026-10-04-architecture-audit-flow.md)。按用户指定，已在 `.jenkins/records` 写入并读回核验[任务原消息（现存归档）](../../../.jenkins/state/records-queue/archive/8cb75329f06f6fe219e1641ac3a7b0fb53135fd40b7f7a463d5deff9eb4452a6/engine-audit-20261009-01a12097-to-jenkins-01a0f6e1.md)，SHA256 `ccfeb8c1fe2e04209f8e4e878494ff54d437b136db065b8d9c1e9d8d539cf142`，5965 字节；接收聊天为 `01a0f6e1-e9a8-7b92-aed8-5220c71a9c43`。六叶及 outbox 哈希在写入前重新核对；artifact registry revision3 和精确历史 revision2 已保留。此前未发送状态只属历史 outbox；本节其余待返回状态为当时记录，后续授权、归档与当前补充交接见7.41。Native=0，各门槛继续开放；无新增构建/采样产物，实际释放0字节，任务active，已验收账本不增。

### 2026-10-12 调用方复核撤回候选风险，F1 事务两项待复现

沿用[执行流 7.39](2026-10-04-architecture-audit-flow.md)的真实阅读范围与十三叶当前哈希。existing 项目在 admission 后重验证 receipt，UI finish 错误进入有终态/隔离区分的 rollback，路径解析器拒绝 dangling link；三项先前候选疑虑撤回，Welcome 与 runtime owner 完整补偿仍需下游及动态证据。PAM readiness 只检查发布门，不据 poison 分支提出 registry 健康缺陷。

PAM 在 `RecoveryDeferred` 成功 outcome 后发布新 active/resources/watchers，再由 `ensure_durable` 返回错误的静态顺序已获 Root/Sol 确认；创建失败的 staging 删除错误丢失已获 Root/Luna 确认。两项进入既有统一问题队列，分别归复合激活/磁盘事务和 ProjectAuthority owner，均无实际故障执行或数据丢失结论。实现依00/F0顺序；需真实 PAM 一次性提交点失败 seam 与原目标保护回归，不以已有发布前失败测试代替。

Runtime 主程序创建失败返回进程 main，不把 creator线程风险列为直接F0线程泄漏；Editor embedded Play 的 typed error→String 边界并入 CR-APP-ENTRY-0003，snapshot清理和native恢复分别验收。最终 GUI handler/线程、host rollback、真实DLL与退出仍有明确未读/未测项。五个助手只读任务均收束；无 F1 源码更改或新增 Native/性能/模块接受，Jenkins ACK仍pending，释放0字节，任务active。

### 2026-10-13 恢复责任与当前源码路径补充

[执行流 7.40](2026-10-04-architecture-audit-flow.md)补充 Play Factory、停机、GUI error projection、host rollback 和磁盘 journal 的真实阅读范围，区别同字节路径迁移与当前已经变化的叶。CR-APP-ENTRY-0003 的后续 owner handoff 方案保持 typed failure 到 backend 的每次调用独立恢复责任；原创建线程、库生命周期、scene cleanup 与 native terminal 都需验收，尚未实施公共契约迁移。

RecoveryDeferred 仍为统一队列的静态待复现项。移到 publication 前判定持久性尚不足以保证同 root 的旧 active 与磁盘一致；须隔离写入、按 journal phase 恢复并重新对齐 generation。RestoreOriginal 仅回滚 Active，AllCommitted 保留新目标；test cfg FailCommitPointSync 改写 outcome 不能证明真实磁盘同步故障。Root 已重新读当前 publication 和恢复入口，Luna 重新读取当前 PAM open_prepared_project，确认 generation 发布与 watcher 处理之后才判定 durability；同 root 故障仍未复现。00/F0 之后才实施 F1，Native/产品/性能和模块已验收计数不变，释放0字节。

### 2026-10-13 当前切片格式与静态检查通过，补充交接已登记

见[执行流7.41](2026-10-04-architecture-audit-flow.md)与[当前Jenkins补充消息](../../../.jenkins/records/engine-audit-20261009-01a12097-current-source-r2-to-jenkins-01a0f6e1.md)。消息10122 B、SHA256 `e1538d9546b8150c6561e72ab69c29a5288addd459b7a2c7c2f7cbcb473c6781`已读回核验；此前授权回复和原消息的归档字节一致，当前已有validation授权。原grant的三个迁移路径仍需合法owner更新，直接checkout执行能力、完整输入seal和正常request/attempt尚待核对。

### 2026-10-13 所选provider的checkout执行目录缺口已确认

见[执行流7.42](2026-10-04-architecture-audit-flow.md)与[capability补充消息](../../../.jenkins/records/engine-audit-20261009-01a12097-execution-capability-r3-to-jenkins-01a0f6e1.md)，消息6358 B、SHA256 `ef4f696fe9224d57d7bef6b03208b632a3585bb834a08fe679f84a26e07c85ea`已回读核验。Root/Sol已读materialize_source正文并与runner的Cargo cwd串联，确认当前所选路线编译准备池，sourceView=checkout没有直接执行实际checkout；底层copy/link callee未读且不影响cwd结论。已要求Jenkins owner经正常provider提供实际checkout模式和完整输入/物理输出/receipt证据，并沿用已有授权更新三个实际消费者路径。没有执行该路线、创建源码副本或提交新job；ACK与native仍pending，MVP/产品/性能/模块接受未增加，Root释放0 B，任务active。

### 2026-10-13 records回复已关联，原请求和当前安装身份已校正

见[执行流7.43](2026-10-04-architecture-audit-flow.md)，R2/R3回复分别`1495c710...`／22014 B、`9c23edc5...`／15191 B；消息与task匹配，records关联已有证据，native intake和formal acceptance均false。Root纠正此前遗漏安装指针、把历史ebf37胶囊称为当前的错误；实际installed指向7936cd76，Root重读单命令/batch的pool cwd与materialize调用后确认direct checkout仍未支持。回复保留原App compile/cleanup两个a1引用和receiver失败历史，没有Cargo终态；三消费者grant、完整seal、source稳定时段、入口完整性及原runtime恢复待合法owner处理。Root未重发、重启、改私有授权或新增发布权限；native/性能=0，已验收账本不增，保留产物，释放0 B，任务active。

六个当前App/Input叶通过rustfmt与change-guard静态检查；新App fixture格式哈希为 `0fd942d2...`。三Luna实际审查完成，source guard和测试声明不当动态验收；Root最终六叶哈希全部匹配消息。artifact registry revision4及精确历史revision3已登记，传递状态保留旧证据与新的消息身份，最终自有命令exit0/EOF。未创建构建/采样/源码副本，Native=0、释放0字节、任务active；00/F0、产品、性能及模块接受仍开放。

### 2026-10-13 关闭责任与当前正文覆盖登记

见[执行流7.44](2026-10-04-architecture-audit-flow.md)与[当前关闭阅读](2026-10-13-shutdown-ownership-current-audit.md)。初版51源码pin及46生产全文/8856行、3部分、2测试全文/383行由独立Luna复核无阻断；追加后累积57生产全文/11104行、6部分/624正文行，214行切片`36c4da3331946d239b432fd1f851d5a3c4e57921295e44dccd0b4898feb64dd0`的65源码pin及新增结论已独立复核无阻断。后续正文须另记范围，当前仍无native、产品或性能验收。Core unload之后的retirement错误、真实watcher退出、外层App/interface与DLL持库仍是该切片的待证边界。ABI slot保留原handle/owner重试，未完成的failed-startup abort路径单列，不把它写成可返回pending handle。

Graph owner-lifetime静态候选覆盖prelaunch与dispatcher runner submission，Node terminal与observer delivery/Scope retire分开；没有保留handle的graph失效终态补偿。它不同于CR-RUNTIME-TASKS-0001的DetachOnDrop/FinishOnShutdown策略TODO，先保留独立待复现与policy依赖，没有生产修改或回归执行。E盘当前约173.976 GiB可用，磁盘门槛解除；Jenkins当前安装仍选7936cd76，direct checkout和原请求执行仍pending。源文档在artifact registry revision6预登记，供给/失败/共享/未知产物继续保留；native/产品/性能和模块接受=0，Root释放0 B，goal与清理active。

### 2026-10-13 R4回复与当前公开执行能力待验收

见[执行流7.46](2026-10-04-architecture-audit-flow.md)及[R4监视器回复](../../../.jenkins/state/records-queue/archive/3c138524391989e2a185645aa4d28a0ba18011be88cf7b1e8bb7c64e59387863/engine-audit-20261009-01a12097-current-state-r4-to-jenkins-01a0f6e1.monitor-response-20261013T102109Z.json)。消息指纹匹配，状态`prepared-not-submitted-public-entry-pin-changed`，不等于指定recipient ACK或native接纳；原App compile/cleanup a1及receiver历史保持。84d所选single/batch正文均使用准备池cwd，sourceView=checkout仅输入选择；无实际checkout执行或源码副本。公开local-cargo也直接拒绝独立/evidence参数，不能用私有helper替代原请求。入口、driver/runtime、scope迁移及真正checkout模式仍待合法owner完成，当前未提交、编译或测试。

[当前关闭阅读](2026-10-13-shutdown-ownership-current-audit.md)的a66／282行切片及随后9e文字增量均经三Luna按范围独立复核，无阻断；79唯一source pin、68生产全文/14238行、7部分/969行、4测试全文/1037行已核。本轮将真实PAM/watcher安装、partial-startup与ready错误用例前提写清，App Library edge不推广未读gateway；图仍保留App退出/failed-startup外层待证。后续补读另记范围，未增加生产修复、native/产品/性能/模块验收或已验收账本。25个历史产物缺失保留开放；供给/失败/共享/未知产物保留，Root释放0 B，goal与清理active。

### R5当前状态与关闭调用链待动态验收

当前main HEAD1de7f1af1f86b124645acdb2743bfe9064f6da8b；原六叶App/Input pin匹配，完整compiler输入封存仍未通过。SourceDoc §10的93a59041／58272 B/363行切片增量三路复核通过，当前e06e0ba1／58343 B/363行仅两句状态归档。84生产全文、5部分、4测试全文和93引擎pin；外部3pin/774部分正文单列。Win32普通返回handler撤销链静态闭合，panic下游、真实线程/Library关闭与行为回归仍开放。

R5已在指定records目录创建并读回，SHA b4cb3ec933af53ce77bded0386b1d003bcad248be371633656dc3e84583f1072／6723 B/42行；当前不具聊天ACK、matching reply或native提交。新0f4040 capsule的普通single/batch同旧runner/source字节，仍prepared pool cwd，actual-checkout缺口未修；由正常provider owner在原A1/receiver31记录内核入口/grant/输入/真实产物和六叶消费者，执行原App编译与聚焦回归。

R4及monitor两个原路径当前FileNotFound，归档位置/原因未证；扩大文件名定位遇拒绝访问且无可判定候选，不据此称全树无记录。历史投递/匹配回复与此前25缺失href保留旧时点，本次两项证据缺口保持开放。当前native/产品/性能验收仍0，Root释放0 B并保留供给/失败/共享/未知产物；goal及清理active。


### 2026-10-14 原请求输入与actual-checkout执行契约仍待闭合

见[执行流7.48](2026-10-04-architecture-audit-flow.md)和[源码记录11–12](2026-10-13-shutdown-ownership-current-audit.md)。§11的f0898b4e切片三路复核无阻断：95引擎pin，85生产全文、5部分、5测试全文；本轮records/工具阅读不增加引擎覆盖。R4与R5消息已精确定位归档；R5状态回应的intakeAccepted=false和后续受理receipt的true分别保留，formal/native均未接受，指定聊天ACK未证。

原task State version35仍waiting。两个原formal DTO仍App测试旧hash f263bc47（实际0fd942d2）及三个旧runtime_absorption消费者路径；实际六叶pin已重核，当前src/tests/contracts迁移scope与完整compiler/external zr_vm HEAD/index seal仍须正规provider协调。selected formal实现明确复制源码、固定prepared cwd且以recipe_cwd_unsealed拒绝live checkout；须由合法owner完成贯通source/metadata/planner/pool/host的实际checkout契约，保持批准产物根与原身份，不以私有helper或新请求替代。

R6登记为原App A1/cleanup A1、receiver31的具体补充交接；原session、repoID76b56a19e17c4b8203865215a6d1c078、attempts2/8/14/23/26和失败证据保留。当前main d588b90c仅追加frontend测试提交，R6当前六个checkout pin再次匹配，原formal DTO差异待协调；指针变动不等于runtime能力。Luna45DTO已限定复核本节d9caede7切片，无阻断；本次仅更新措辞和复核状态；MVP00/F0及之后门槛继续pending，Root本批次native/产品/性能样本和释放空间0，供给/失败/共享/未知产物保留。仅更新已核消息归档链接，未关闭全部历史缺失href；goal和清理active。

#### R6受理与当前路线补充

[源码记录12.4](2026-10-13-shutdown-ownership-current-audit.md)与[执行流7.49](2026-10-04-architecture-audit-flow.md)记录实际归档和followup36回执：R6字节3c071557／7936 B/51行匹配，intake=true、formal/native=false、newNativeRequest=false；原task version36仍waiting，原A1因external HEAD/index输入登记pending，App旧hash与三个迁移scope需owner协调。新的ordinary capsule报告actual-checkout continuous-read路线，canonical formal仍prepared-source；metadata、受理、静态读取均不构成native/product/performance验收。新World/render/Gateway限定M0文档已在registry15预登记，源码正文结果待整合/独立复核；本批Rust实施、native、产品、性能样本与释放空间均0，MVP00/F0及之后门槛保持开放。

### 2026-10-14 World、渲染与Gateway局部M0记录

[当前限定架构阅读](2026-10-14-world-render-gateway-current-audit.md)与[执行流7.50](2026-10-04-architecture-audit-flow.md)：31生产pin/6273去重正文行、6测试pin/353行；整个模块、全引擎M0和动态验收未完成。EA46-WORLD-001重复entities反序列化panic只经两方静态根因复核，待有效fixture与产品load可达性证明；兼容World clone、恢复扫描频率、多view LOD合同、Gateway重入/线程/输出诊断仍开放。已存在fixed-origin operation route，初步产品P1定级撤回，consumer用法待核。新文档6ab8fc1e切片三路复核通过；本条d86f2c6d切片经Luna45Archive限定复核通过，本次仅更新行政状态；F0原A1输入/迁移scope/执行协调仍优先，后续实现遵守MVP门槛。Root本批Rust/native/产品/性能样本和释放空间均0；已验收账本不增。


### World/ECS、共享渲染与Gateway下游增量（2026-10-14，静态待动态）

[当前架构文档](2026-10-14-world-render-gateway-current-audit.md)§10新增28生产叶6689行全文；范围累计59生产pin/12962行、6测试pin/353行仅读，不是全引擎M0或模块验收。EA46-WORLD-001最低owner为World恢复投影且公开文件/LevelManager API静态可达；Editor Open到World恢复桥仍未闭合；duplicate回归与失败保护待验。EA46-WORLD-002的postcommit错误、EA46-WORLD-003的epoch/events失败原子性条件、render共享scene COW/LOD与fullfallback成本、pick cancel/终态identity和整个退出链继续开放。沿用ZCA-HST-001 malformed status无reject/fuse静态候选，不另造重复故障；所有产品/性能结论待动态。

R7具体7项externalGitlinks经Root/Sol独立核，7530 B/72行、SHA941c619c，已按用户指定records投递；12:47:03Z原task36仍waiting/pending且R7匹配受理空，不算ACK/native。普通same-key原payload digest不得修改；provider/授权owner须走公开合法迁移处理App旧hash与3scope、完整输入/原身份/continuous guard，不能用新request/g1覆盖旧pending。当前E盘115.952 GiB只为12:35:36Z DriveInfo观察，native前预算准入reserve+amount仍待；Rust/编译/测试/产品/采样0、源码副本0、实际释放0 B、旧未知handles/供给失败共享产物保留。§10与本增量文档已通过三路限定独立复核；未知只读handle/EOF独立开放，goal/cleanup active，已验收账本不增。

### R7补充材料已受理，原生基线保持待验（2026-10-14）

[架构证据](2026-10-14-world-render-gateway-current-audit.md)§10.7核到R7 #42/queue1844补充intake及真实archive/response（941c619c /7530 B/72行；98dae15c /24739 B/1行），原records file不重建。13:50:39Z taskv37仍waiting/formal=false；actualFlow/native/formal/newNative均false、named-chat/operator delivery和正式registration/compile receipts均null。public-entry drift、原AppHash/3scope合法迁移及完整compiler/driver-runtime/continuousguard仍pending，MVP00/F0未验。Root实际释放0 B，未知handles/供给失败共享证据保留；本条和对应§10.7/Flow7.52已通过限定独立复核，goal/cleanup active。

### World失败合同排除与producer汇聚续审（2026-10-14，待动态）

[限定文档](2026-10-14-world-render-gateway-current-audit.md)§11累计74源码pin、68生产14778正文行、6测试353仅读行；新增9生产1816行。EA46-WORLD-003的epoch/events导致重复binding校验普通Err假设经Sol与Root核源撤销，不再进入修复队列。001保持World持久化投影静态P2，typed预检设计无公开API/v2迁移要求，真实duplicate/save-reopen/保全回归待00/F0；002提交后target变化的消费者合同开放。实际World producer/cache/dirty汇聚已读，正式session调用、锁范围、热路径成本与Editor SceneAsset恢复桥仍待。原A1/native、R7 named-chat/operator送达、完整compiler seal沿既有交接开放；本新增段及补计5行的行政差异均已通过Luna50限定复核，未知EOF保留、Root释放0 B、goal/cleanup active，无模块验收。

### 正式producer与Editor场景准备桥，原A1续接待验（2026-10-14）

[限定文档](2026-10-14-world-render-gateway-current-audit.md)§12累计87pin，81生产16154正文行+6测试353仅读行；新增13生产1376行（7全文、6部分）。Scene=World alias，SceneAsset converter与EditorState实际安装owner待读，WORLD-001不据wrapper名排除Editor风险；prepare/文档激活不等当前slot发布。正式session失配直接持锁prepared extraction，兼容World clone不等每帧主路径；snapshot交错仍仅条件合同，成本待实测。R8 daea0b5a/6529 B/39行已独立复核并按用户指定records16:35:36Z投递，16:40:03Z仍可读，无intake/chat/operator/native receipt。public历史query被新driver/runtime准入静态阻挡，existing-record同digest约束仍条件；实际A1 State/checkpoint未查，Jenkins owner须原身份合法reconcile/contract迁移。MVP00/F0和native、模块、产品/性能仍pending，本增量已通过Luna50限定独立复核；未知EOF保留，Root实际释放0 B，goal/cleanup active。

### SceneAsset身份前检与Editor安装owner续审（2026-10-14，待动态）

[限定文档](2026-10-14-world-render-gateway-current-audit.md)§13累计91pin：85生产17052正文行、6测试353仅读行；新增4pin+旧SceneAsset扩读898行。World插入对最终record ID重复已有typed前检，已核直接链不走WORLD-001 bulk restore；serializer回调/外部component-reference及完整Editor影响仍待。实际PreparedLevel消费是authoring gateway绑定，旧guard不保证history/selection/current-slot整体恢复；reload seed取走后的Err不可盲重试，loaded/exclusive/history/gateway helpers待读，未报新已复现故障或性能收益。R8 17:19:51Z仍records SHAdaea0b5a/6529 B，无限定matching response/archive或observed chat/operator/native receipt；原A1与MVP00/F0未验，旧未知EOF保留，Root实际释放0 B。本增量已通过三路限定复核，goal/cleanup active。

### DTO与authoring回调合同、R8受理续接（2026-10-14，待动态）

[限定文档](2026-10-14-world-render-gateway-current-audit.md)§14累计92pin：86生产17504正文行、6测试353仅读行；新增1pin及旧authoring全文净增452行。DTO验证仅逐entity直引，正式读取不同于test helper；gateway替换自身显式Err在slot store前，mutable callback后的错误随业务result保留，实际消费者补偿、并发writer和history/exclusive恢复合同待读。R8已核真实archive daea0b5a/6529 B与canonical相等、response fb5b345f/13383 B及intake=true；formal/native/newNative=false，#51实际终态及chat/operator送达未核，字段缺失不补null。两原A1的operation/marker absence为外部报告，checkpoint/actualnative EOF未证明；原owner合法历史query/合同迁移、完整input/guard和compile→cleanup仍pending，MVP00/F0未验。§13三路文档复核通过，本新增§14已通过三路限定复核；未知EOF与共享/失败/交付证据保留，Root实际释放0 B，goal/cleanup active。

### Editor事务恢复与DTO引用续审（2026-10-14，待动态）

[限定文档](2026-10-14-world-render-gateway-current-audit.md)§15累计96pin：90生产18832正文行、6测试353仅读行；本批1328新正文、4新pin及两旧partial升full。selection失败恢复/history成功后移除已核，Core clear_scene非selection残留怀疑撤回；Editor发布后settings/viewport条件失败由上层owner协调，callee真实失败合同待读。DTO validator只核generic row，builtin安全性不代表custom；已核最终record DuplicateEntity前检不退回unknown。publication身份两步窗口仅API条件风险，生产writer串行待证；不得据外层Err盲重试/补偿覆盖新slot。§14三路复核通过，本§15已通过三路限定复核。R8 actual intake不是原A1 native完成，精确pilot active缺失不等formal route停止；原请求、旧unknown/共享/失败证据保留，MVP00/F0与goal/cleanup开放，Root实际释放0 B。

### Settings准入、取消与资产引用槽位续审（2026-10-14，待动态）

[限定文档](2026-10-14-world-render-gateway-current-audit.md)§16累计106pin、20965仅读正文；本批1780新行/10新生产pin。settings bind/clear真实显式Err为busy（bind另有overflow），Invalid安装不可写绑定；产品busy触发、World发布前后协调及真实callee继续开放。cancel的take→reset→sync→传播Err本地前缀已核，World恢复合同未核；generic row仅marker/槽位完整性，registry resolver内部及custom未读。CoreEditContext默认fresh play与EditorContext显式注入分开，两个field不证明owner隔离；新增Builder产出owner隔离已核，Host局部传参已核，中间manager安装链及线程待证。§15限定复核通过，本§16已通过三路限定复核；R8 reply已完整消费、支持证明#51/queue2002 SUCCESS仅属intake；原A1 compile/cleanup仍pending，worker logical移除6529B不等Root物理释放，exact pilot absence不等formal route停止。historical A1选定字段只作报告时点证据，未导入异任务batch13门槛。保留旧unknown/共享/失败证据，MVP00/F0、模块/产品/性能与goal/cleanup开放，Root实际释放0 B。

### manager入口、同步观察者与导入准入（2026-10-15，待动态）

[限定文档](2026-10-14-world-render-gateway-current-audit.md)§17当前125pin/25754仅读行；19新全文叶4758行，generation旧148→当前179净+31，旧pin保留。manager链/observer锁与错误范围已核，resources lookup、sink/adapter/toast合同、restore验证及候选隔离待核；不报告新复现或性能收益。R9补交映射已消费，entry因process39236 unavailable拒绝，原A1授权仍active，部署/授权owner需具体恢复/公开迁移方案；原输入不重绑，完整seal和native exit/EOF/cleanup未证。三domain RAM事实复核非三路物理文档review；旧unknown/失败与共享/外来证据保留。MVP00/F0、全引擎M0、模块/产品/性能、goal/cleanup开放，Root实际释放0 B。

### Core 查询、Toast 与提交 durability（2026-10-15，待动态）

[限定文档](2026-10-14-world-render-gateway-current-audit.md)§18累计133pin/26816仅读行，本批8生产pin/1062行，4全文4部分。Core typed/name查询与同handle交付已核，Core安装/activation与失败/线程合同未闭合；Toast同ID存活期间拒绝、不同generation数据可并存但UI消费者未读。scan/watch和targeted单次的candidate发布/ensure顺序不同，RecoveryDeferred仅表明commit-marker durability未定，底层commitpoint/同步/回滚未读，不报确认缺陷或性能收益。

R10本地交接已有精确monitor消费回信，非原Jenkins owner维护授权或native验收；当前selected driver与installed management绑定不符，公开capability拒绝，服务live不等Ready/可恢复，不无条件restart。未重提A1或生成控制请求；原身份/预算/unknown、完整seal和native exit/EOF继续开放。M1Python9叶905行（R10的879加guard26）独立，不增Rust覆盖。旧失败/共享/外来证据均保留，MVP00/F0、全引擎M0、模块/产品/性能与goal/cleanup开放，Root实际释放0B。

## S19（2026-10-15 UTC）：事务核心与公开 installer 门槛待验收

[S19源码与证据记录](2026-10-14-world-render-gateway-current-audit.md#19-s19事务核心续审与公开-installer-预检2026-10-15-utc) 当前139 Rust pin/28727行；生产132 pin/28257行、测试7 pin/470行，122全文/17部分。六个新核心全文及两个旧部分补完新增1911行，source-removal 父模块与 targeted 单方法证据已分清。

- EA60-ASSET-001：targeted 单方法 RecoveryDeferred 后旧内存视图的静态合同候选，产品级未定。当前限定精确符号12命中只见测试调用；已核无测试门控，完整产品可达性、错误后使用合同及所需动态回归仍开放。未据此删除或修改代码。
- Core 的 PublishedWithoutSync、普通失败尝试回滚、恢复解析边界已定位；frame_codec/discovery 尚未读，现有故障钩子不能替代真实 sync/恢复实验。
- M1：一次公开 installer CheckOnly 实际拒绝于第222行 Unreviewed public entry change，包装 exit2、native退出码空；六pin稳定。R11a具体门槛已放入授权本地records，原owner消费、受审迁移和真实维护回执待实证。原 A1 compile/cleanup 的输入身份、append-only准入、终态及清理保持 pending，不重绑/重提。
- M0全引擎、MVP 00/F0、native/产品/性能及模块验收均未通过；未做构建或产品运行。实际释放0B，cleanup.closed=false，保留foreign与未知归属。

## S20（2026-10-15 UTC）：client pin补丁已交接，执行待验

[S20证据记录](2026-10-14-world-render-gateway-current-audit.md)在刷新foreign world_time pin并核对后记录141 Rust pin/28964行，生产134/28494、测试7/470，124全文/17部分；新增两个事务叶235行及旧版本净增2行。覆盖并集不代表模块或M0全引擎验收。

- M1：installer新增独立submit raw pin，三处owned增量与新契约测试已独立源码复核；语法、owned patch reverse/whitespace与change-guard通过。六个unittest方法的37个gate场景及SHA向量尚未执行；Jenkins真实测试、修复后公开CheckOnly与安装/接纳分别pending。
- R12最终文件已落指定records，原owner实际消费未证实；正确摘要与monitor代读均不代替维护授权。原A1 compile/cleanup输入及历史身份不变，append-only准入、真实编译/回归与清理仍pending，不重绑/重提。
- frame/discovery已读，但EOF完整帧坏checksum的失败模型、恢复截断调用方与序列化、真实sync/恢复实验仍开放。EA60-ASSET-001仍是targeted单方法静态合同候选，产品级未定。
- MVP00/F0、M1/native、产品、性能、模块与M0全引擎均未验收；未运行构建、测试、产品或采样。实际释放0B，cleanup.closed=false，foreign、共享缓存及未知归属保留。

- R12最新回信：monitor消费已证实，原owner接纳仍未证实。新工具followup正式提交在input/marker/State创建前因旧public entry pin拒绝，聚焦测试执行0次；原generation运行记录为stopped。先由原owner处理运行代次和公开工具迁移，再分别取得测试、CheckOnly/安装与原A1证据；运行时退出信息不计作测试/native结果。原始dispatch全文与operation选定字段已核pin，详见主文§20.5。

## S21（2026-10-15 UTC）：时间与持久化续审，动态验收开放

[S21证据](2026-10-14-world-render-gateway-current-audit.md)新增30个全文Rust叶5780行；当前去重并核pin的范围为171 pin/34744行（生产161/33165、测试10/1579，154全文/17部分）。这是源码阅读覆盖，模块与M0全引擎验收保持开放。

- S21-WTIME-ENDPOINT：提交累计饱和后，减步长重建前一端点存在条件性静态不一致；补齐Virtual/Level真实链后，以成功提交、abort/reset与消费者插值验收。尚未实施修复或动态复现。
- S21-WTIME-STEP-BUDGET：明确暂停预约单步是否优先于正常预算，再验证零预算、无效外帧、消费与abort重试；当前为合同待决。
- S21-ATOMIC-RECOVERY-IDENTITY：Windows恢复用UTF-8失败后的共同stem识别唯一备份，可能在条件成立时错配目标；真实产品输入、Windows文件名及备份状态仍需受管fixture验证。静态条件已独立复核，产品级未定。
- journal尾帧坏checksum并入既有失败模型问题；批量恢复路径/摘要门槛已读，但域callback、路径身份底层及外部替换边界未闭合。测试源码覆盖不代替执行；测试helper输出变量和物理目录须在执行前核实。
- M1公开工具迁移与原A1仍依赖原Jenkins owner的实际回执。本轮未重提/重绑、运行构建或测试、安装/维护服务、改State/auth或采样；MVP00/F0、native、产品、性能与模块验收保持pending。
- 实际释放0B，无本轮自有临时脚本/源码副本/夹具；foreign、共享缓存、未知归属及历史未知进程保留，cleanup.closed=false。

## S22（2026-10-15 UTC）：owner 续审完成，动态门槛开放

[S22证据](2026-10-14-world-render-gateway-current-audit.md)新增16全文叶2707行；当前去重范围187 pin/37451行（生产173/35284、测试14/2167，170全文/17部分），仅为源码覆盖。

- S21-WTIME-ENDPOINT：Virtual、MAX缩放和WorldDriver提交/abort直接owner已闭合，条件问题保留；正式Runtime/handle输入、调度成功及消费者影响仍待核。下一owner仅定位，未读正文。
- S21-WTIME-STEP-BUDGET仍待合同决策；已读controller/Driver仅中止当前时间提交，不提供通用World回滚；系统写入可能保留，runner/system自身是否补偿尚未读。实际重试与线程合同待验。
- S22-CONFIG-PATH-KEY：Windows有损路径键可能让不同target共享epoch并取消旧manager，产品级未定；需无损身份合同和真实双目标fixture。与S21-ATOMIC-RECOVERY-IDENTITY分别跟踪。
- 配置fence失败/panic/回收测试源码及资源路径/锁测试已读，未执行；外部替换、alias、跨进程/重启和stage故障清理仍需真实验收。stage局部Err已追到外层清理，不据此报告live数据损坏。
- 原R12 owner接纳/迁移与原A1回执未取得；测试、编译、产品运行、采样均0，MVP00/F0、M1/native、M0、模块及性能验收保持开放。无新请求/重绑或服务维护。
- 实际释放0B；canonical源码、补丁、账本、交接与失败证据保留，无本轮临时脚本/副本/夹具；foreign/共享/未知与历史unknown不清除，cleanup.closed=false。

## S23：Runtime 时间入口与场景阶段失败（待动态验收）

- 新增9完整叶、identity完整叶、paths.rs部分31行并补全DynamicSession，新增2753行（生产2421、测试332）；按已读版本累计198 pins / 40994正文行。另4个配置生产叶完整刷新736行替换旧612行，净+124行；本任务cases.rs消费者迁移净−4行，随后foreign组件registry85→220净+135、cases331→465净+134；另编辑command净+10、World records旧55行→完整446净+391；metadata/locator/重复读不计增量。
- S21-WTIME-ENDPOINT公开MAX原样输入链已闭合；产品MAX、完整固定阶段成功及插值影响未证，等级未定。controller实际端点历史修复待MVP顺序与受管回归；零预算暂停预约仍合同待决。
- 正式DynamicSession走tick_time采样，不能以manual API证明产品MAX；系统Instant及自定义source的MAX可达性未证。
- Runner局部worker清理不等于通用World回滚；既有World/time owner未提供该回滚，系统自补偿、join、merge/apply最低callee待核。不新定P1；与EA46-WORLD-002目标替换分别保留。
- owner/plan绑定与context域路由已读，不证明generation校验、序列化或pause来源。panic测试仅源码阅读，执行0。
- 既有配置四叶已变化并重读；S22路径键原实现退场，identity叶及resolve_identity局部已读，Windows比较与resolver其他最低callee、别名/I/O等价及动态验收仍待核。
- 当前配置测试旧API/类型不匹配已作单文件迁移，本任务331行后像保留39个assert、6个test及ignore属性，R13有界补丁与静态guard通过。随后foreign追加为465行/9test/62静态assert，含3项owner lease测试与E物理binary根前提；全读确认旧行为保留，动态未验。R13只能作为历史限定delta，受管runtime验证须绑定当前465行，不盲目重套；编译/测试待回执。原R12 owner接纳/迁移及原A1尚无新回执，原请求身份保持；R13指定records已落盘且独立文字复核通过，接纳与执行待回执。测试、构建、产品和性能均0，MVP00/F0、M1/native、M0、模块及性能验收开放。
- 实际释放0B；canonical交付、共享/foreign/未知及失败证据保留；首段续行丢失reader终态仍unknown，无新临时脚本/源码副本/夹具，cleanup.closed=false。

- 最新component registry（d5a878ee…e047c9/226行）与resource/io facade（bf2d9985…ec5026/17行）后续foreign版本未补读；40994为已读版本账本，当前198-pin闭包未通过，模块与当前源码验收不关闭。

- R13监控回执已消费消息，但原owner正式接纳=false、新提交/测试/构建=0；入口仍ordinary_entry_unproven／Pinned public entry source changed，State42既有intake不代表本次R13接纳。保留未完成inbox，原服务恢复及正式测试待原owner回执。

### S24：命令窗口、路径身份与M1恢复交接（限定静态续读）

- 已读版本209 pins / 43655正文行；新增2644行、版本刷新净17行。component registry、I/O facade和35行配置path_key已读；另15份foreign新版本待刷新，当前源码闭包与模块验收不关闭。
- Sol复核：原panic payload恢复以析构/恢复完成为前提；Windows保守writer身份与实际operation I/O分开。两项无充分证据升级独立P2，特殊目录/别名、containment和实际产品分发仍需原生覆盖。
- R14已写原Jenkins owner本地records，说明专用Python进程可能阻挡部署owner check；没有新增Cargo请求。原A1/R12/R13及R14原owner接纳仍待对账，构建/测试/产品/采样均未执行。
- Luna58内层exit/session缺失及Luna59后续429保留未知，不用重读关闭历史进程。释放0B，保留必要交付、复用与unknown产物，cleanup.closed=false。主文§24与流程§7.66记录范围和验收缺口。

S25 续读（2026-10-15）：所选已读版本为 213 pin / 46,202 行（生产 194 / 43,127；测试 19 / 3,075），201 full / 12 partial。16 个版本替换净增 1,777 行，4 个新 pin 增加 770 行；重复阅读未另计。精确范围、旧版本与收据留在 canonical task ledger，见[审查 §25](2026-10-14-world-render-gateway-current-audit.md#25-2026-10-15-续读当前版本保存回执与生产屏障报告)。

实际保存 gate 已反证投影失败错误标 clean 的条件 P1，未创建 issue；普通 deferred 报告的生产丢弃仍是政策/覆盖缺口。WORLD001/002、激活 effect executor、恢复/平台 helper 和剩余部分叶继续开放。原 A1/R12/R13/R14 未被新请求替代；R13 差异监控回应不构成 owner 接纳。原生、模块、产品、性能验收未晋级，cleanup=false、释放 0 B。

S26 续读（2026-10-15）：所选版本为 218 pin / 49,799 行（生产 199 / 46,724；测试 19 / 3,075），213 full / 5 partial。7 个同版本部分叶净增 2,064 行，5 个新 pin 增加 1,533；重复阅读未计。M56 观察到 level_system 当前新版本未读，保留旧 pin 和偏移记录，见[审查 §26](2026-10-14-world-render-gateway-current-audit.md#26-2026-10-15-续读启动归属world-事务与激活补偿)。

激活执行器及下层收据分类反证发布后无条件丢弃 guard 的条件 P1；WORLD001/002、原子写/lease/native 终止边界继续开放。资产 staging/chunk 残件为合并的静态 P3 候选，清理 owner 和实际影响未核实。原 A1/R12/R13/R14 未替换；原生、模块、产品和性能未验收，E 盘固定观察约 26.1 GiB、尚低于 35 GiB。cleanup=false，释放 0 B。

S27 续读（2026-10-15）：所选版本 222 pin / 52,726 行（生产 203 / 48,352；测试 19 / 4,374），222 full / 0 partial；5 项同版本补全净增 2,671、1 项版本刷新净增 5、4 项新 pin 增加 251。editing/engine/command.rs 又出现未读外部新版本，当前源码、模块和 M0 不关闭，见[当前审查](2026-10-14-world-render-gateway-current-audit.md)§27。

WORLD002 同 owner Schedule 收尾方案仅完成有限静态复核，未实现、未动态验收；不同 World owner 必须保留。atomic_file 六文件已有历史全文收据，旧措辞的范围已更正，重复复读新增计零，调用方恢复仍开放。Gateway recorder 为条件静态 P3，实际 runner 隔离待核查；Play retirement 待 helper 补证。M1 约 28.0 GiB 仍低于 35 GiB，driver 更换但原 owner/native 未接纳；原 native16 不变，构建/测试/产品/性能运行 0，cleanup=false、释放 0 B。

- **S28 — Play 退役与命令报告合同（2026-10-15，静态阅读归档）**：所选 Rust 227 pins／53,722 行（225 全文、2 部分；生产 207／49,323，测试 20／4,399），新增 Play 915 行、command 当前版本净增 22、scope／recovery 尾部 34／25 行；manifest 12 行分列，必要复读不重复计数；context.rs／exclusive_transition.rs 的两个外来新版本仍未读。详见 [§28](2026-10-14-world-render-gateway-current-audit.md)。
  - 已确认 Play 生产恢复入口存在；泵内不重试只保留为局部合同性质，外部调度／回执门控／真实 backend 失败未验收。新增 `S28-COMMAND-RECOVERY-REPORT` 仅为嵌套错误报告字段问题，等级未定，不与历史 malformed-status 合并；Gateway fixture P3 仍待真实 runner 与并发复现。
  - 22:26 UTC E 可用约 21.5 GiB，35 GiB 构建／测试门槛未满足，原 Jenkins R14 核对仍开放；无 Rust/Cargo 修复与运行验收。释放 0 B，保留 7 组历史未知归属；M0／模块／native／产品／性能验收均未关闭。

- **S29 — 事务状态与项目关闭（2026-10-16 UTC，静态阅读，待动态验收）**：所选 230 pins／55,618 行（227 全文、3 部分）；刷新两个版本，新读 engine_state／project_close 全文及旧 pin 的 core/editing/command 部分。编辑命令换版后停止并拒绝拼接，四处已观测未读版本保留。详见 [§29](2026-10-14-world-render-gateway-current-audit.md)。
  - Play 关闭重入入口已读，effect 幂等与实际重试仍待最低 owner；S28 报告项仍为合同问号。新增 S29-TX-UNWIND 条件静态风险：先核对 operation／context 与 retained-provider 的 panic 合同、实际 containment；未复现、不定产品等级，不与 WORLD panic／报告项按名称合并。
  - 02:32 UTC E 约 11.3 GiB，35 GiB 构建门槛未满足；active.json 仍缺，R14 原 owner 核对及 native／产品／性能验收开放。无 Rust/Cargo 修改与运行；释放 0 B，未知 reader／产物归属保留，清理未关闭。

### S30 后续：实际动作与外层恢复 owner

[源码审查 §30](2026-10-14-world-render-gateway-current-audit.md) 已回答有效同 operation Closing store 的重复转换未知项；下一最低 owner 为重放外部动作、Play receipt 推进／关闭重试驱动，以及 invoke_operation 路由和 receipt-consumer／runtime-event callback 的 containment 与同 engine 复用。当前覆盖 249 pins／59,036 行、4 部分；未读版本与 TX-UNWIND 条件风险保持开放。MVP 00/F0、原 A1／R12／R13／R14、M1 native／产品／性能验收继续待验；E 盘约 4.9 GiB 未达 35 GiB，driver selector 不构成验收。无新 native 请求、源码修改或运行；释放 0 B，cleanup 未关闭。

### S31：保持开放的验收条件

静态增量已取得 9 pins / 2,178 行，规范候选累计 258 pins / 61,214 行（250 full / 8 partial）；当前 archive 待独立核对，详见 current audit §31。factory→既有事务实例绑定与 model-import completion→一次 commit 重入已证；仍需真实下游 unwind containment/同实例恢复合同、Play 非 ready 后重试来源、取消底层合同及 UI 失败中途恢复，未新增已确认产品缺陷或可安全删除项。原四个未刷新版本、A1/R12/R13/R14、M1/native 和模块/产品/性能验收均不关闭。2026-10-16T05:54Z E 约 6.3 GiB，低于 35 GiB；driver selected 状态、monitor 回执和当前工具可见性均不能替代验收。释放 0 B；保留三组本轮 spawn_ready 与历史未知归属证据，cleanup=false。

### S32 待验收续接（2026-10-16）

S32-TICK-TEXT-CONTRACT-DRIFT：retained host_lifecycle 的三项文本测试保留两条 production 已无的连续调用表达式，静态条件已独立核对；cfg(test) module 声明存在，未执行测试、未定产品等级。实施等待 MVP00，并以真实次数、普通错误分支和状态合同设计行为回归。S29-TX-UNWIND 已排除已读底层自动归还守卫，外层 catch/同实例复用/恢复政策仍开放。新增 6 full pins + 1 same-pin extension，并完整复读 lifecycle 当前552行（version refresh净+1），累计 264 / 62,825 / 257 full + 7 partial；M0、模块、native、产品、性能与 cleanup 均未关闭，释放 0 B。原请求续接消息在 .jenkins/records；磁盘门槛已通过、195-file capsule 静态完整性匹配；自动本地登记和静态 hash 检查不授予当前 runtime/Cargo 接纳。

## S33 待执行：原 A1 查询身份与当前版本动态验收（2026-10-16）

四当前版本已完整重读，canonical264pins/63445行/258full/6partial；ENDPOINT当前静态实现取代旧减法根因，MAX/reset断言未运行。STEP-BUDGET优先级、S33-COMMAND-POST-ERROR的真实合规producer及既有TX-UNWIND复用合同继续开放，均不新增P2或实施。
原A1已恢复完整originalInput/buildRoot配置，尚缺真实attempt/ref/receipt、native exit/EOF及跨runtime映射；不得用默认attempt、startup/unknown-stop或采样absence替代。原owner按records交接补齐身份后封存输入并受管验证；MVP00/M1、R12/R13/R14及动态/产品/性能门槛不变，Root释放0B。

### S34待验收：资产ticket关闭、恢复边界与当前版本

六份partial按实际正文补全，canonical264/65922行；另22新版未读，不能提升全引擎当前覆盖。保留admission耗尽后同identity重试政策、资产已提交但projection/placement失败通知、真实ticket终态与close非ready恢复问题；普通InProcess+Level单次回调路径已排除重复/后造dispatch Err，custom producer及unwind仍条件开放，无P2/补丁。

recovery/pending cases测试未跑；ordinary intake ready回复不替代原A1 attempt/native exit/EOF及append-only跨runtime映射。下一步续原身份与当前输入封存，按owner只读更新22新版；MVP00/M1、R12/R13/R14、产品/性能待验收。未知进程/共享缓存保留，释放0B。

### S35待验收：真实owner与动态门

22新版已全量替换，canonical264/66337行；12:17:22 UTC的264路径metadata无drift，12:23:10 UTC登记已有1个drift：dynamic_api/session/state.rs新版809行未读，旧796行pin保持。上述窗口不构成全引擎或compiler seal。当前测试设计未执行，dense duplicate component恢复、postcommit Err重试、CoreHandle/World时间提交和真实render submit仍待核验。

下一组只读owner为SessionGuard release/Drop与隔离caller、scene路由串行/state、atomic helper与ChunkReader/residency；验证须覆盖durable projection失败不标clean、prepare拒绝不flush、故障恢复、generation/cache、cancel/close终态。原A1 attempt/native terminal与当前输入封存、MVP00/M1、R12/R13/R14及产品/性能仍待验收。未知进程/共享缓存保留，释放0B。

### S36待验收：残留准入、关闭消费者与当前版本

273个登记版本full/67774行；Root登记窗口有12个当前drift，准确路径/pin见s36_registered_pin_metadata_observation，旧版本证据保持。S35所列atomic六文件及guard早已同pin全读，重复不计新增；新9文件和两个新版已完成静态正文阅读。

下一组追ProjectRecoveryAssessment/admission与quarantined消费契约、state shutdown/host输出消费者，以及剩余当前版本的最低共享owner。运行层继续原A1终态/lineage与current compiler输入封存，随后一次受影响批次及真实产品/性能基线。所有回归设计未运行，R12/R13/R14/MVP00/M1/模块/产品/性能保持开放；历史失败、unknown和共享缓存保留，释放0B。

### S37 续读状态

- 12 个变化版本与新增 8 个文件的正文已读；规范覆盖 281 个完整文件／69523 行，范围统计不等于全引擎验收。
- 已补齐恢复评估 Terminal 接管策略并独立复核；没有确认需修复的 P2。World producer／generation 消费者、外层关闭提交、debugger 状态及 begin／end 保持待查。
- ordinary release Err、激活失败及 durability uncertainty 的聚焦故障回归尚未运行；捕获等待尚未取得测量成本。
- 本地 S36 消息已有监控读取回执；原 A1 attempt／PID birth／exit／EOF／checkpoint 未对齐，M1/native、产品与性能验收继续开放。
- 归档复核结果以本轮独立回执为准；释放 0 B，未知与共享产物保留，生命周期未关闭。

### S38：当前待执行边界

- M1：Jenkins owner修复entry/receiver的跨driver原身份观察校验，保持原A1 payload/driver/runtime及unknown，返回原queue/build/全部attempt、PID+birth、exit、stdout/stderr EOF和检查点；具体交接见本地records的s38-original-a1-observer-contract文件。当前marker缺失不足以关闭请求。当前新M1仅在受管输入封印和源码/缓存/native资源准入成立后独立执行。
- 源码：本轮14新全文/709行、state.rs刷新809行，累计所选295全文/70232行；291/295当前metadata匹配、4漂移。EA46-WORLD-002查发布后Err的真实重试消费者；WORLD-001查restore/projection及Name可达性；Editor查真正UI调用方和recovery owner；S38-CAPTURE-PAIRING先核实实际submit/Drop与错误可达性。均不以静态范围替代动态验收。
- 本轮源码修改、构建、测试、native请求、捕获和清理均0；释放0B，保留共享/未知/历史证据。原A1、R12/R13/R14、模块、M1、产品和性能验收开放。

### S39：下一步执行范围

- M1：Jenkins owner正常部署historical_query修复并核对六client pin；capability ready后仅按原身份query-only，恢复最终payload及全部attempt/native/EOF。两份已准备候选来自S38 reply，非authoritative；原unknown与owner ACK保持开放，新当前源码M1仍需实际资源隔离和输入准入。
- 源码：所选302全文/73046行。Scene查level_manager lifecycle与artifact I/O；Editor查recovery controller和project_close_failure；capture查实际scheduler executor/camera-loop锁与Drop并复现backend wait错误。已有21项问题范围保留，S38-CAPTURE-PAIRING未分级，无新增已证P2。
- 本轮无引擎源码修改、构建、测试执行、native请求或捕获；清理释放0B，保留共享、未知与历史失败证据。静态记录不关闭M0/M1、模块、产品或性能验收。

### S40：当前执行范围

- M1：部署入口已恢复，原A1查询结果为original_operation_missing/pending，保留完整原payload、native退出和双EOF缺口。按S40本地交接准备独立当前源码zircon_app --lib --no-run批次；实际pool/cache、build-script资源及旧A1隔离和source/deps/MSVC封印未证前不得启动。
- 源码：所选316全文/76098行。Scene继续验证实际保存ticket、关闭drain或取消承诺；恢复继续查RestoreFlow/Executor与通知失败；capture继续验证backend wait错误与panic恢复契约。shutdown不等待和setter锁环假设已排除；S40-SCENE-FENCE-BUDGET及S38-CAPTURE-PAIRING均未分级，已有21项问题范围保留。
- 本轮引擎源码修改、Root构建、测试执行、native请求、捕获与清理均0，释放0B；未知、共享和历史失败证据保留。M0/M1、模块、产品、性能验收保持开放。

### S41：当前执行范围

- M1优先：当前入口可用，monitor未准入；核实management/pool writer、reservation、cache generation与未知A1的资源关系，再取得当前source/deps/MSVC seal、read guard、物理输出及峰值预算证据。新requestId或requestedTargetDir不能单独证明隔离；原A1未知及目标owner ACK继续开放。
- 保存与关闭：继续读取Editor scoped commit/receipt.is_durable、DocumentLifecycleAuthority Saved/clean条件和菜单桥；Scene直接产品路径与取消反馈仍待建立。所选328文件（324全文、4部分）/77103行，21项问题范围保留；S40-SCENE-FENCE-BUDGET及S38-CAPTURE-PAIRING未分级。
- 无引擎源码修改、构建、测试执行、产品运行、native提交、捕获或清理；仅WPR两次只读查询，释放0B。保留共享、未知及历史失败证据，M0/M1与模块、产品、性能验收不关闭。

### S42/S43：当前执行范围更新

- M1 优先：受支持入口可执行，但两次完整捕获均在 management POST 前失败，尚无当前源码编译产物。下一步协调实际 main/HEAD/完整 index 与外部 zr_vm 稳定写入窗口；具体变更项、writer owner 和目标 Jenkins owner ACK 未知。旧 A1 unknown 保留，不重投或清理原身份。
- 只读结论：JobHandle 等待终态；持久化以 Durable receipt 判定；Saved 与 history clean 分别发布和校验。累计所选 350 文件（336 全文、14 部分）/80521 行，非全引擎阅读完成。仍追 task_graph/result/TaskHandle、pool shutdown、消息总线/dispatcher、DirtyRegistry 与独立 menubridge。
- 既有 21 项问题保留；S43-EDITOR-FAILED-COMMIT-REPORT 是未评级条件性诊断漏报，安排在 M1 基线之后。S40-SCENE-FENCE-BUDGET 和 S38-CAPTURE-PAIRING 不关闭。
- 三名 Luna 并行审查及 Sol 核心复核均已结束；没有引擎代码补丁、构建、测试执行、产品运行或性能采样。保留受管入口失败日志、源码行段证据和本地交接记录；无自有临时文件，释放 0 B，M0/M1 与模块、产品、性能验收保持开放。

## S44 待验收：observer 测试与文档事实背压

- S44-TEST-OBSERVER-DOMAIN：已修正 task_graph/tests/cases.rs:452、464 的关闭 census 域为 AsyncCompute 及两条提示，保持业务 Compute 与行为断言。change guard／差异检查／独立全文静态复核通过；当前源码原生聚焦测试仍待执行，不能标记已验收。
- S44-LIFECYCLE-BACKPRESSURE（P2 候选，静态契约违例）：发布者忽略 Lossless 拒绝报告，缺少事实重试 owner。先明确有限 pending、提交边界、顺序、session 退出和永久拒绝，再迁移全部调用者；以真实 helper＋bridge＋第二订阅者容量 1 的恢复／恰好一次通知回归验收。未证明磁盘损坏、错误 clean 或默认容量耗尽。
- DirtyRegistry／close lease 的正常权威链已读；layout 后计数耗尽、跨 registry snapshot、实际 pump 线程／重入和其他订阅消费者继续开放，不升级为已复现问题。
- 阅读账本 388 个指纹、88,408 行去重正文，359 全文／29 部分；旧事件控制器全文不迁移到新指纹。原 21 条队列前缀保持，追加两项。覆盖量不等于模块或全引擎验收。
- Jenkins 交接已追加到既有 .jenkins/records 文件，等待实际写入方的稳定窗口；当前监控回复不足以启动第三次源码捕获。S44 编译、测试、产品及性能采样均为 0，M0/M1 和动态门继续开放。保留自有证据和共享缓存；没有清理产物，释放 0 B。

### S45 待验收更新

- M0：事件循环→tick→manager pump 与文档所有生产 helper 提交边界已读，当前405 pins、91072 unique body lines；全引擎/模块验收继续开放。
- S44-LIFECYCLE-BACKPRESSURE：维持单一静态 P2 候选，未有默认容量耗尽、具体插件功能损坏、磁盘故障或错误 clean 证据；有限容量与身份/顺序合同及可执行复现仍待完成。
- M1：第三次原身份 strict 公开请求在捕获 guard 拒绝，compiler0；其前后 query-only 均 original_operation_missing。Jenkins owner 待定位 exact metadata/membership 差异，保留 guard 与原 lineage；不再盲目捕获。
- S44 observer 测试四行修复：独立静态复核通过，Runtime 聚焦原生测试未执行；App lib no-run 即使未来通过也不代替该测试。
- 旧 A1 unknown、frozen native16、产品/原生/性能验收均保持开放；S45 自有进程已退出，保留报告和两份原始日志，释放0 B。

### S46 待验收更新（2026-10-17T04:00:18.034966+00:00）

- S32-TICK-TEXT-CONTRACT-DRIFT：四处旧文本已修复，原断言保留；Sol78/Luna105 静态复核与 change guard 通过。Jenkins zircon_editor --lib retained_tick_ 仍待执行，不关闭此项。
- S46-PLUGIN-RETIREMENT：Loaded 成功后 Enabled 失败、或正常 Disabled 后替换遗漏 Unloaded；静态 P2，未证明产品资源泄漏，最小 progress/retry/HotReloaded 方案待 MVP 顺序允许后实施。
- M1：原 strict operation missing，准备沿原身份严格捕获；S45 raw logs、旧 A1 unknown 与 frozen native16 原样保留。下一次客户端日志预登记，当前无 native/product/performance 验收。

S46 当前受管尝试补记（2026-10-17T04:28:34.147540+00:00）：原 strict request 的第4次客户端在 2026-10-17 04:03:35–04:09:21 UTC 完成两轮捕获后，被 entry.source_digest:369 的前后不一致检查拒绝（exit2；ordinary_entry_unproven）。本次 management POST0、编译0；失败后原身份 query 仍是 original_operation_missing/resubmitted:false。主仓库外层 HEAD/成员/索引摘要相同，但两轮捕获总字节相差98，不能据此宣称源码稳定；外部 zr_vm HEAD 从030d8d3变为2e2c95d。具体内部差异和写者尚未归属；已补入 Jenkins 本地交接，先协调实际写者窗口并完善差异诊断，再继续原请求。没有第5次提交，M0/M1/native/product/performance 门槛全部保持开放。

### S47 待验收更新（2026-10-17T05:38:17.808383+00:00）

- M1 capture drift：有界诊断与三个拒绝合同回归已落盘，独立实际静态 PASS；需 public managed installer 更新客户端源 pin、当前版本 ordinary-flow suite，然后在协调静默窗口沿原 strict 身份捕获/编译。诊断不等于根因复现或 native PASS。
- 当前工具能力：Tracy、RenderDoc capture/replay、GPU timestamp、WPR ETL 未建立当前 App 证据；现有 UI preflight 的 Editor/runtime DLL 约束不能直接匹配当前 App BuildSet。
- S46 Editor tick 与 S44 Runtime observer 聚焦 native、产品全流程和性能重测仍 open；本轮无模块晋级、提交或发布。

### S48 待验收更新（2026-10-17T06:53:27.872599+00:00）

- 已验：S47 三个隔离诊断行为用例，复用原 foreign #54 的逐项真实执行；R13 八个 fence 用例，复核原 #72 的原生证据。仅这些片段通过。
- M1：entry seal 已匹配，保留原 strict App request/operation；在协调静默窗口查询后继续捕获与编译。旧 pytest route 的 schema1 限制不否定实际 unittest 收据。
- 开放：当前 App 编译/启动、Editor tick 与 Runtime observer 聚焦 native、完整产品回归、Tracy/RenderDoc/GPU timestamp/WPR 当前能力和性能重测。无模块晋级，compound sidecar 候选暂不定级。

## S49 原生准备冲突与默认包验证范围

S49 完成三路 Luna/max 只读范围及后续生命周期/schema/测量能力核查，由 Sol/max 复核 feature 与MVP00次序。原App strict捕获两次一致并提交queue3662/#82，实际终态preparation-conflict，compileStarted=false；原marker与receipt逐项比较仅external E:/Git/zr_vm 的filesDigest及派生digest变化，engine/cache/HEAD/index/gitlinks相等。具体文件和actor未证明，后续commit/lock漂移不追溯归因。原身份、摘要和native16/未知A1保留。

本轮当前源码阅读并集 497 pins（429 full / 68 partial），107027 行；相对S48净增2422行。只按实际同pin正文ranges合并，locator保留partial；没有模块、全引擎、MVP00、产品或性能验收。统一代码问题24项未新增。

原App dynamic-api,platform-winit不启用App/native-host；后续close_owner行为需追加该App feature、prefix entry::tests::product_composition::close_owner::、testThreads=1（3普通+1ignored，不include-ignored）。Runtime同名feature不反向启用App。受管checkout普通batch的首失败停止未获证明，因此下一合同先单Runtime默认check，成功后再Editor/App；最终argv/TEMP/物理产物路径由原生收据核验。具体 [MVP00合同](2026-10-17-engine-audit-mvp00-default-validation.md) 与独立新request留存；新请求尚未提交。

当前WPR可定位且版本10.0.26100.9711，未启动；当前实际12个tools没有RenderDoc MCP。工具安装/历史查询不代表当前产品附着、捕获、GPU timestamp或ETL有效性。优化前后各5独立run、预热后30秒仍待产品就绪。原始性能证据目录E:/zircon-profiles尚无本轮采样。

实际证据 E:\cargo-targets\zircon-local\reports\engine-audit-20261009-01a12097\s49-native-source-conflict-app-feature-validation.json；原生操作和external组件差异已通过用户指定 .jenkins/records 交接，peer monitor独立确认，完整稳定窗口尚未证明。Root本轮未改Rust/Python、未本地Cargo、未复制源码、未清缓存；删除0文件/释放0B。

## S51–S52 待验收更新

- M1：S50 单 Runtime 默认 check 未登记操作；第一次客户端在 engine capture 收尾被复合输入 guard 拒绝，同身份 QueryOnly 确认 missing。等待 Jenkins 全输入安静屏障/真实 current-source candidate 能力；不放宽 guard、不更换 identity、不盲重试。
- 回归：loader/session owner、Runtime04 resolver/recovery/metadata authority 现有测试已读，当前 native 执行仍 pending；先绑定 fixture_root 与物理 TEMP/CARGO_TARGET_DIR。
- 能力：Tracy 日志抢占假设已排除；实际 subscriber 安装/连接、GPU 有效 readback、RenderDoc 附着及 WPR 当前采样仍待产品就绪。
- 证据和范围见[统一审查记录](2026-10-14-world-render-gateway-current-audit.md)。源码选读不等于模块验收；代码问题24项不变，原生/产品/性能不晋级，清理释放0B。


### S56（2026-10-17）：实现已归档，仍待当前源码验证

| 项目 | 当前处置 | 关闭条件 |
| --- | --- | --- |
| CR-APP-ENTRY-0017 | typed proof、显式 F2、生产 completion 与24文件消费者补丁已实现并独立静态复核；cfg 类型 owner 已修正 | 当前源码 Runtime/Editor/App 验证，合法普通项目与严格 F2 产品路径、回归和实际输入/捕获证据通过；问题等级仍依动态证据确认 |
| S56-MVP-FIXTURE-RETIRED-DEPENDENCY | 已确认 helper 所需 resolver/zircon-session 路径缺失；非 native 复现，统一 helper 与 Jenkins lifecycle owner 负责 | 先核实际登记/物理根/保留与清理接口，再同步迁移 helper 和旧字符串契约测试，执行真实 fixture 与 staging/acceptance 回归 |

最终补丁 SHA-256 `71b0c5ab5c4e0127b5ec368db5d57b45bcbe3286d86f08e27cf6282fac306e4e`；[实现报告](E:/cargo-targets/zircon-local/reports/engine-audit-20261009-01a12097/s56-cr0017-explicit-product-proof-implementation.json) SHA-256 `0e96d54f3d54286aeda327e39b539a32928ff26b313f2a64eafc8f0d22f04b31`。572源码 pins 的阅读记录不是全模块验收。26项统一队列保持开放，原 S50/失败与清理证据保留；受管编译、产品、性能和 M0/MVP00 均未关闭。本轮释放0B。
