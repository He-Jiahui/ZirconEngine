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
  - tools/check_conventions.py
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
| `zircon_runtime/src/input/**` | 29 | **29/29、3,074 行已静态复审；动态未验收。** 普通未消费 ABI 输入静态调用图至少 6 次 mutex acquisition/事件；gamepad transition 为 `O(events x active_axes)`，snapshot/action collections 与真实 storm 数据待 Runtime12/10 收敛 |
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

## 当前全局阻塞

- 当前产品bundle入口在Cargo前稳定失败：`tools/build-editor.ps1:130`以PowerShell字面量`'\\'`拼接approved root，导致合法D/E/F输出无法通过containment匹配。默认D盘和显式E盘调用均复现；`tools/tests/build-editor.Tests.ps1`当前15项为**9 pass / 6 fail**。精确移交见`01/failure-2026-08-15-build-editor-approved-root-separator.md`与PERF-MVP-634。该脚本及测试已有foreign-dirty改动，Performance01未越权覆盖；在15/15、managed bundle与`--help`恢复前，WPR/xperf/RenderDoc仍未启动。
- Windows managed validator dry run已生成D盘target命令。自建E盘空temp目录曾被`unmanaged_artifacts_detected`拒绝，已由coordinator精确删除且artifact audit为0；build-only `zircon_app`矩阵在D盘managed target运行324.2秒，以212条warning、6个foreign-dirty `zircon_runtime`错误失败。随后focused `zircon_runtime` lib-test又运行843.4秒，以361个编译错误、1,520条warning失败，0条test执行；tests/WPR/Tracy/RenderDoc current binary均未运行。本Session Cargo作业已结束，不抢占、不改用裸Cargo。
- `python tools/check_conventions.py --repo-root . --only docs --json` 在并发硬切期间从 **612 / 212 documents** 漂移到本轮末次 **638 / 229 documents**（2,382 documents、74,308 checked paths）；本次持有并复核的 performance 文档为 **0 violation**。全局问题保留在 `01/failure-2026-08-02-doc-structured-path-owner-drift.md`，不以批量猜路径消除。
- RenderDoc 1.44 可用，但已有高级 volumetric capture 不是当前 F2 MVP 场景；在可运行产品入口和 managed build 恢复前，不用旧 capture 冒充当前 GPU 基线。
