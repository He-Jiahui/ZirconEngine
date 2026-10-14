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
