---
related_code:
  - zircon_editor/src/core/commandlet
  - zircon_editor/src/core/sync
  - zircon_editor/src/core/tools
  - zircon_editor/src/core/context/tool_scheduler.rs
  - zircon_app/src/entry/cli/launch_args.rs
base_reports:
  - docs/plans/performance/01/2026-07-22-editor-core-commandlet-sdk-export-script-sync-static-review.md
  - docs/plans/performance/01/2026-07-30-editor-core-commandlet-current-review.md
owner_plans:
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
  - docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_editor/editor/16-cli-args-and-hub-integration.md
  - docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Commandlets/Commandlet.h
  - dev/UnrealEngine/Engine/Source/Runtime/Launch/Private/LaunchEngineLoop.cpp
  - dev/UnrealEngine/Engine/Source/Editor/SceneOutliner/Private/ActorHierarchy.cpp
  - dev/UnrealEngine/Engine/Source/Editor/SceneOutliner/Private/SSceneOutliner.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/InteractiveToolsFramework/Private/InputRouter.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/InteractiveToolsFramework/Private/InteractiveToolManager.cpp
tests:
  - tools.tests.test_editor02_world_sync_subscription_table_contract
  - tools.tests.test_editor02_world_sync_watch_map_contract
  - tools.tests.test_editor08_tool_scheduler_contract
  - tools.tests.test_editor_tool_scheduler_unblocked_queue_m0_performance_contract
doc_type: implementation-evidence
status: static_current_revalidated_m0_applied_dynamic_blocked_structural_cutover_required
---

# Editor Commandlet、Sync与Tools currentness复核（2026-08-23）

## 当前冻结与结论

| scope | Rust文件 | physical lines | bytes | tests | ordered path + NUL + raw bytes + NUL SHA256 |
|---|---:|---:|---:|---:|---|
| `zircon_editor/src/core/commandlet/**` | 3/3 | 1,190 | 38,384 | 15 | `9dfe276124ff96f0ade02682a5ba3d394eac79547b957b490f2d576a2a2a10f9` |
| `zircon_editor/src/core/sync/**` | 5/5 | 1,424 | 48,368 | 21 | `77a1cea37aa47f5a2510ce10ebf843f111a48568b5238993e0c45ca7c454189f` |
| `zircon_editor/src/core/tools/**` | 4/4 | 1,416 | 44,479 | 17 | `fc31eba878be8e38cb5d50f77901db4060775acc43189192a1df511bf6e7e51e` |

12/12当前Rust文件及Commandlet入口、WorldSync每tick调用、ToolScheduler service调用链已完整复读。
三组仍为`static_complete / dynamic_pending`，不得进入`review.md`。

本轮保留其他Session对`core/tools/scheduler.rs`和`core/tools/tests.rs`的并发修复：
`ToolResourceSet`从`BTreeSet`改为最多3项的`Vec sort_unstable + dedup`，并补齐set释放/撤销后
unblocked single queues的promotion。该改动方向合理，但不是本轮所有权，也没有current Cargo证据。

## Commandlet控制面

当前正向进展：parser只构造一次`EditorCommandRegistry::default_workbench()`，把已解析descriptor带到
execution；执行阶段不再重建registry或再次线性解析route。plugin-list返回共享`Arc` projection。

剩余成本不是帧热路，但会放大批处理启动和大迁移：

- `EditorLaunchRoute`已经拥有`Vec<String>`，仍以`args.iter().cloned()`交给parser；parser再collect一套
  `Vec`。GUI错误诊断还要求原argv，不能用一次局部签名替换假装完成统一解析owner。
- migration runtime report之后再collect两套changed/issue DTO，逐行复制path/message；入口随后
  `serde_json::to_string`生成第三份完整JSON字符串。规模为`O(rows + encoded bytes)`，峰值同时保留
  runtime rows、commandlet rows和encoded envelope。
- authoring automation显式进入process-owned retained host。它必须由typed capability/profile决定最小
  subsystem集合；不得因名为headless就默认构造完整GUI host、渲染器或插件watcher。

结构目标：`ProcessArgvOwner -> CommandletDescriptorToken -> TypedCommandletRequest`只解析一次；
migration/output使用一个borrowed report projection和增量writer，稳定JSON envelope/exit code不变。

## WorldSync每帧控制面

`WorldSyncPump`在retained host每tick调用。runtime已经提供canonical sorted/unique token快速路径，
正常batch不再建duplicate/seen两套树；非canonical fallback仍建3个`BTreeSet`并为诊断排序。

当前主要瓶颈：

- 一次tick先完整扫描所有batch验证generation，再二次遍历全部batch；没有batch/fact/bytes/elapsed
  budget、age或overflow receipt。transport backlog可一次性占满编辑器主线程。
- 每个fact独立`serde_json::to_value`、独立构造message并独立`bus.publish`；每个batch又独立
  `mark_view_dirty_set`。因此锁/序列化/分配规模至少随`facts + batches`线性增长。
- `token_for`对同一view的token集合线性查找；小规模成立，但大量组件watch会成为
  `O(watches-per-view)`重复注册成本。

Unreal并不在一帧无界清空Scene Outliner变化。`SSceneOutliner.cpp:779-821`按pending operation增量
处理，并每100项检查`GSceneOutlinerProcessingBudgetPerFrame`；`ActorHierarchy.cpp:45-46,713-741`
从actor add/delete delegate产生精确变化。Zircon应保留runtime token/generation边界，但采用同类
bounded incremental apply和direct changed-item projection，而不是每fact JSON bus fanout。

## WorldSync generation clear M0

gateway generation变化意味着旧token不能提交给新runtime。旧实现仍调用`drain_tokens()`，为随后
丢弃的结果分配并填充N项`Vec<WatchToken>`。本轮新增`WorldWatchMap::clear()`，generation失效路径
直接清空两个索引；显式unwatch生命周期继续使用`drain_tokens()`。

静态所有权差值：临时token snapshot由`N elements + 1 allocation`降为`0 elements + 0 snapshot
allocation`；两条路径仍需`O(N)`析构旧binding/tree nodes。这只关闭确定性冗余，不声明每tick
WorldSync瓶颈已关闭。

## ToolScheduler结构复核

该模块只有3种资源、每队列默认最多64项，无线程和I/O；4个`.position(...)`线性查找上限明确，
把`BTreeMap`换成数组只能得到常数收益，不是优先结构优化。

真正风险是全局head-of-line：只要`set_queue`非空，任意single acquire都不能直接取得空闲资源；
blocked set head又阻止后续不相交set。`ToolSchedulerService`再用一个全局Mutex串行所有context，锁外
仍逐事件clone并逐条取得message-bus锁。队列虽有界，但Modal、ViewportInput和SceneModeSlot之间可能
产生与资源不相交的交互饥饿。

Unreal `InputRouter.cpp:12-29,48-74,127-164`按mouse side/keyboard保存active capture owner，只有
无capture时才收集并按priority选择请求；`InteractiveToolManager.cpp:121-168,237-318`用单一active
tool slot显式deactivate/activate，并在shutdown前deregister input source。Zircon目标应是
per-input-context capture lease、modal stack和scene-mode transition各自有owner，再用明确的组合ticket
协调多资源操作；不应继续扩大一个跨域全局FIFO。

## 验收计划与量化门

1. Commandlet：argv 1/100/10K，command descriptors 3/100/10K，migration rows 0/1/1K/1M，
   path/message 16B/4KiB，stdout fast/10ms/blocked。采集argv owners/cloned bytes、registry builds/lookups、
   report owners、encoded bytes、first-byte/total wall和peak RSS。目标为registry build 1、argv整表复制0、
   migration row复制0、full encoded String owner 0。
2. WorldSync：每帧batch/fact/token 0/1/1K/1M，subscriber/view 0/1/100/10K。采集drain age/bytes、
   processed/deferred/dropped、JSON bytes、bus lock acquisitions、alloc/RSS、main-thread p50/p95/p99和
   generation lag。idle目标为0 fact allocation/0 publish lock；active必须受显式items+bytes+elapsed预算
   控制，并有下一帧continuation receipt。
3. Tools：3个资源的single/set交错、64项满队列、focus loss/modal re-entry/tool shutdown。采集每次
   acquire/release扫描项、lock wait/hold、events/cloned bytes、bus locks和oldest wait age。目标为不相交
   context互不阻塞、capture owner直接路由、每次state transition一次批量publish。
4. current-source managed Cargo、F0/F4 startup/authoring产品trace、WPR/xperf/allocator/RSS/package power
   必须完成。RenderDoc只在WorldSync驱动可见场景变化的产品帧做CPU/GPU关联，不用于证明这些CPU控制面。

## 本轮静态门

- `rustfmt --edition 2021 --check --config skip_children=true`：12/12通过。
- 四个相关Python模块：22/22通过。期间修复旧合同对已删除
  `gateway.watch_world + synchronize_gateway_generation`文本的切片，改为验证当前
  `with_current_gateway_generation + runtime.watch_world`原子锁域。
- scoped `git diff --check`通过，仅有现存LF/CRLF提示。
- docs convention检查3,132 documents / 83,692 checked paths，全库既有801 violations影响275份
  文档；本轮两份performance产物为0 violation。
- 未运行Rust/Cargo、WPR、allocator、功耗或产品trace；managed validator session已归档，且没有
  current-source可执行文件。没有在C盘生成产物。
