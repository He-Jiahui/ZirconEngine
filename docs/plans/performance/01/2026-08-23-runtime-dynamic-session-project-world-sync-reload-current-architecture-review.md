---
related_code:
  - zircon_runtime/src/dynamic_api/session/project.rs
  - zircon_runtime/src/dynamic_api/session/world_sync.rs
  - zircon_runtime/src/dynamic_api/session/scene_asset_reload_diagnostics.rs
  - zircon_runtime/src/dynamic_api/session/runtime_ui.rs
  - zircon_runtime/src/asset/pipeline/runtime/construction.rs
  - zircon_runtime/src/scene/inspection/subscription.rs
  - zircon_runtime/src/scene/asset/reload
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/01/2026-08-15-runtime-asset-project-registry-pipeline-current-architecture-review.md
  - docs/plans/performance/01/2026-08-15-editor-core-world-sync-routing-current-architecture-review.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_runtime/runtime/10-runtime-session-contract.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
  - docs/plans/zircon_editor/editor/02-data-sync-and-message-bus.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Private/AssetRegistry.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Private/AssetDataGatherer.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/StreamableManager.cpp
  - dev/UnrealEngine/Engine/Source/Editor/SceneOutliner/Private/SSceneOutliner.cpp
tests:
  - current dynamic session project/world-sync/reload diagnostics 3 of 3 Rust files and 18 inline tests reviewed
  - supporting project generation, runtime UI, inspection subscription and reload queue owners reviewed
  - M0 static performance contract 3 of 3 passed after RED
  - focused rustfmt 1.94.1 plus scoped diff check passed
  - current-source Cargo, WPR, allocator, power and RenderDoc product traces pending
doc_type: implementation-evidence
status: m0_static_complete_dynamic_blocked
---

# Runtime dynamic session项目、WorldSync与热重载复审（2026-08-23）

## 范围与当前性

已逐行复读`dynamic_api/session/{project,world_sync,scene_asset_reload_diagnostics}.rs`当前
**3/3**个Rust文件。实施前合计**1,474行、53,024 B、18 tests**；M0后为
**1,478行、53,530 B、18 tests**，按`path|lines|file-hash`生成的manifest SHA256为
`aace945803226bec1f7d0bc2301999a6cde443a16160626123ca22e837792452`。同时沿调用链复核
project manager snapshot、runtime UI asset load、scene inspection subscription、Level world
invalidation和scene asset reload queue。支持文件已有其他Session改动，本轮只读并保留。

## 当前源码判定

### 项目激活仍复制错误的聚合所有权

`RuntimeProjectConfig::prepare`同步执行`ProjectManager::open_resolved`，随后
`open_project_assets`把prepared project安装进`AssetManager`。安装完成后，`project.rs`又为scene
reload与runtime UI分别调用`current_project_manager()`；该API经`project_read().clone()`返回完整
`ProjectManager`值。它包含路径、manifest、resource/asset/package registry、catalog generation、
importer、artifact store、shader dependency和executor等聚合状态。因此启动已经打开并安装项目后，
仍至少创建两份宽聚合快照。

runtime UI随后遍历完整asset registry并加载所有layout/widget/style资产，而不是从启用的root
surface计算依赖闭包。默认导航配置还同步读文件并解析TOML；startup package过滤把已发现包名先clone
进`HashSet`，再对每项以线性`any`扫描启动包列表。后两项是低频局部成本，不能替代既有
`PERF-MVP-638`要求的单一immutable asset/project generation硬切。

### WorldSync有界fact不等于有界transport work

`SubscriptionTable`当前已有variant direct index、facts count/bytes/age上限、semantic coalescing与
复用的ancestry scratch，这些是有效改进。但watch registration数量本身无硬上限；
`pending_dirty: BTreeSet<WatchToken>`也没有count/bytes/age上限。任意多watch token可持续放大一次
World mutation的dirty集合，即使fact retention已被限制。

`world_sync.rs`把dirty/facts投影为ABI page。首次尝试最多16,384 items/1 MiB；若实际JSON超限，
则从minimum page开始二分候选范围。每次探测都会重建、clone并完整编码候选page，最坏约14次宽
JSON遍历，且25 ms deadline只能终止后续尝试，不能收回已消耗的帧时间。当前tail truncate/pop避免了
front removal的O(N^2)搬移，不能据此认定page seal已达到规模最优。结构修正应先增加encode attempts/
visited items/cloned bytes/wall counters，再选择borrowed range serializer、预计算item byte index或
generation cursor；在没有基线前不直接重写序列化算法。

### 热重载旧结论已失效，剩余问题是动态验收

`PERF-MVP-471`当前仍描述无上限event drain、全pending扫描、无界pending与不可取消旧任务。复核当前
reload owner确认它已有event count/time/bytes限制、per-asset keyed single-flight与supersede、
pending/result/ready/apply byte caps、frame demand、target staging和诊断。因此不得再按旧结论重复实现
另一套队列。未关闭的是ready-to-compiled-target transaction、stale/cancel/slow-consumer与产品帧预算的
current-source动态证据，继续由Runtime04/08/11现有failure与`PERF-MVP-471`验收。

## Unreal源码依据与目标结构

Unreal `AssetRegistry.cpp:4725-4930,5041-5260`把background gather、pause/takeover、result trim和
budgeted result processing分开；`AssetRegistry.cpp:220-224`定义独立background processing预算。
`AssetDataGatherer.cpp:305-332,1945-1948`配置并启动低优先级并行discovery worker。
`StreamableManager.cpp:1931-1966`复用loaded/outstanding streamable state，而非让每个consumer复制整个
project authority。

Unreal `SSceneOutliner.cpp:40-44,750-821`保留pending add/move/remove操作，每帧按5 ms默认预算处理，
每100项检查预算并把余项留到后续帧。这里可采用的是单一typed pending owner、显式budget和可续作cursor，
不是照抄5 ms常量，也不是以源码经验值冒充Zircon测量结果。

目标数据流应为：

1. prepared project只构造一次candidate；Runtime04提交一个immutable `RuntimeAssetGeneration`并发布稳定
   handle/lease，删除`ProjectManager`聚合值快照。
2. scene、runtime UI与startup script从同一generation按root/dependency demand取handle；background task
   只准备changed dependency closure，frame/activation边界短提交generation。
3. World mutation发布一个canonical change generation；subscription只保留受count/bytes/age约束的
   token/page cursor，overflow显式变成resync reason。
4. Runtime10/11在session ordered lane中一次seal有界page，锁外encode；Editor02消费generation/cursor并
   按count/time/bytes续作。不得建立第二套World truth或WorldSync私有线程池。
5. scene reload复用当前keyed single-flight候选与shared job service；动态证据通过前不声明旧P0已完成。

## 本轮M0

场景热重载每帧记录12个count和1个bool时，旧代码为每条series分别调用
`runtime.record_diagnostic`，即13次取得diagnostic store锁并重复generic metadata路径。本轮改为一次
`update_diagnostic_store`，在同一临界区内调用`DiagnosticStore::record_static`，并共享静态tags。路径、
frame、unit、tags和值保持不变；每次frame report的store lock入口从**13降为1**，generic metadata入口
从**13降为0**。

`tools/tests/test_runtime_scene_asset_reload_diagnostics_m0_performance_contract.py`先得到2 failures、
1 pass的RED，实施后**3/3 GREEN**；测试33行、1,228 B、SHA256
`1334399d808fc16d50bf3b856f4181e53ec635f1aadfa6b484378efaac34fa01`。focused
`rustfmt +1.94.1 --edition 2021 --check`与scoped diff check通过。受管Cargo当前不可执行，Rust行为测试
与wall time/功耗没有运行；上述13到1只代表静态锁入口，不冒充真实加速比例。

## 动态验收矩阵

| owner | matrix | 必须采集与验收 |
|---|---|---|
| project/asset | assets/files/UI roots/package roots 1/1K/100K；cold/warm；0/1/1% change | open/read/decode/registry visits、project snapshot与clone bytes、dependency visits、task overlap、commit lock、p50/p95/RSS/energy；收敛后聚合ProjectManager snapshot=0，工作接近changed dependency closure |
| WorldSync | watchers 0/1/1K/100K；dirty duplicates 0/50/99%；facts 0/1K/100K；page 1KiB/1MiB；stall 0/1/60s | registration/maps/dirty entries+bytes+age、page builds、encode attempts/visited items/bytes、session/World/subscription lock与p95；所有队列硬有界，稳定无变更seal/JSON=0，单页每item至多一次生产encode |
| reload | assets/events 1/1K/100K；prepare 0/10/1000ms；payload 1KiB/64MiB；slow consumer/cancel/stale | active jobs、join/supersede/cancel、queued/result/ready/apply entries+bytes+age、wasted work、world-lock/apply wall；per asset active<=1，帧工作受预算且RSS有界 |
| diagnostics M0 | report 1/1K/1M；threads 1/16 | diagnostic lock entrances/wait/hold、series updates、alloc与wall；每report lock entrance=1，13条series语义等价 |

在同一硬件、电源计划、frame cap、前台状态和fixture上至少运行三次，报告median/range和profiler开销。
WPR/ETW负责CPU、线程、I/O、锁、wake和power归因；allocator负责clone/RSS。RenderDoc只在F2/F4稳定帧验证
热重载后upload/copy/pass/draw与像素一致性，不承担项目启动或WorldSync CPU归因。当前没有可启动的
current-source二进制，本切片继续留在`pending.md`，不提交milestone、不发送完成企微。

