---
related_code:
  - zircon_editor/src/core/notifications
  - zircon_editor/src/ui/host/play_pending_decision
  - zircon_editor/src/ui/retained_host/app/workbench_notifications.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/tick.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/notifications.rs
base_reports:
  - docs/plans/performance/01/2026-08-15-editor-notification-generation-projection-current-architecture-review.md
  - docs/plans/performance/01/2026-08-22-editor-notification-center-retained-row-generation-architecture-review.md
owner_plans:
  - docs/plans/zircon_editor/editor/04-pie-and-simulation.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Notifications/NotificationManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Notifications/SlateAsyncTaskNotificationImpl.cpp
tests:
  - tools.tests.test_editor_notification_center_row_allocation_contract
  - tools.tests.test_editor17_decision_notification_center_contract
doc_type: currentness-revalidation
status: static_current_revalidated_contract_repaired_dynamic_pending_structural_cutover_required
---

# Editor core notifications currentness重验（2026-08-23）

## 当前冻结与结论

| scope | Rust文件 | physical lines | bytes | tests | ordered path + NUL + raw bytes + NUL SHA256 |
|---|---:|---:|---:|---:|---|
| `zircon_editor/src/core/notifications/**` | 25/25 | 3,471 | 110,931 | 38 | `2b855a73d953fb8923bfc0f2e512ea748a9719d9588e818d5ac7ac87b8f5e3a8` |

25/25当前Rust文件以及Play adapter、retained tick、dispatch side effects、activity projection和
workbench bridge调用链已完整复读。当前HEAD为
`f79dc502a1e8db5f7cbcc17fbeb297af1e193f7e`；相对2026-08-15冻结，core production没有变化，
只在`decision/tests.rs`增加2行常量导入。若干调用方存在其他Session的未提交改动，本轮按当前字节
审查并保持不动。

结论仍为`static_complete / dynamic_pending`。容量界、typed identity、ticket/cursor线性化和
observer panic恢复是正确基础；P0瓶颈仍是稳定tick在变化判断前重建三套全量owned projection。
因此没有用局部cache修改production，避免固化多authority轮询结构。

## 当前算法与结构瓶颈

Decision最多128 pending、256 receipts；Toast最多128 live；Progress最多64；bridge history最多64。
在一次稳定activity sync中：

- Decision先锁中心并clone pending snapshots，再锁Play adapter并clone retained decisions，随后以
  nested `find`匹配ticket。上界`D=128, A=256`时仅ticket匹配就可达32,768次比较，之后仍需本地化、
  message placeholder replace和owned UI string构造。
- Toast每次`live_toast_snapshot`都锁中心、对整个map执行expiry `retain`并clone全部live rows；在
  `next_expiry`之前也没有O(1) `NotModified`路径。
- Progress每次先锁中心构造`BTreeMap<NotificationId, JobId> + Vec<JobId>`，查询job snapshots，再锁
  中心并构造第二个`BTreeMap<JobId, Snapshot>`、retain和clone projection。发布虽然是observer驱动，
  消费仍是每tick全量轮询。
- bridge在上述工作之后才format最多64条pipe string、解析unread/id/kind并与旧数组比较。稳定帧能
  抑制最终invalidation，却不能撤销已发生的锁、clone、本地化、format和parse。

当前active Workbench tick固定调用一次`sync_activity_notifications`；toast publish立即再调用，
dispatch side effects完成后还会再调用。因此同帧接受一次通知变化仍可执行多次完整projection。
现有复杂度仍约为`O(D + A*D + T + P log P + H*W)`，其中`W`是encoded row宽度；内存有界但CPU和
分配不与变化量成比例。

## Unreal源码依据

- `NotificationManager.cpp:244-290`要求直接UI创建发生在game thread，其他线程只把notification
  推入pending queue；`342-380`由manager tick drain pending并维护现有window，而不是从多个producer
  每帧重建一套encoded history。
- `NotificationManager.cpp:292-324`向progress handler提供直接start/update/cancel。Zircon应保留更强
  的JobId与generation权威，但由权威发布changed rows，而不是UI再次全量snapshot。
- `SlateAsyncTaskNotificationImpl.cpp:262-302`捕获变化并安排一次game-thread更新；`314-365`消费
  optional pending state，且只在状态变化时mutate widget。这直接支持“source generation + one-frame
  coalesced apply”，不支持稳定帧轮询后比较完整字符串投影。

## 依赖有序优化计划

1. Editor17先为Decision pending、Toast live/expiry和统一activity projection定义monotonic generation、
   immutable typed rows与`next_toast_expiry`；空generation必须能清除旧UI。
2. Editor04建立ticket/notification/selection direct index。Decision或locale generation变化时只构造
   一次本地化choice，删除`A*D` nested matching。
3. Editor14从job progress authority暴露generation/shared rows。notification surface不得每tick重建
   两个map；PERF-MVP-017状态栏继续消费同一source generation，不建立第二authority。
4. EditorUI08保存last-applied revision tuple。tick只读compact token；dispatch/toast publish只mark dirty，
   每帧最多apply一次。临时string-array ABI若仍存在，只能在changed generation编码一次，不能反向解析
   成authority。
5. 在结构cutover前后加入stage counters，再做current-source managed Cargo、F4 WPR/xperf、allocator、
   RSS与package power。RenderDoc只验证通知变化后的paint/overdraw和draw/resource parity。

## 量化验收

| matrix | 必须记录 | acceptance |
|---|---|---|
| stable 1/1M ticks，D/A/T/P为0、1和上限，30/60/120 Hz | locks、rows visited/cloned/localized、encoded/parsed bytes、sync calls/frame、invalidations、CPU/RSS/power | 初次apply后至下一expiry，全部projection work为0；空generation只clear一次 |
| publish/resolve/cancel/expiry/progress/locale change，1/16 producers | source/unified generations、build/apply count、stale reject、input-to-present p50/p95/p99 | 每个accepted source change只增代一次；每个统一generation最多build/apply一次；同帧duplicate sync=0 |
| D=128、A=256、T=128、P=64，row 64 B/2 KiB/256 KiB | ticket probes、map builds、clone/format bytes、peak RSS | decision匹配近`O(D+A)`；stable O(1)；Progress stable map build=0；changed bytes受显式预算控制 |
| F4 before/after | WPR CPU stacks/contention/allocations/context switches/package power，相关帧RenderDoc | Play选择、cursor gap、toast expiry、progress refill/retire、locale switch与像素/draw parity全部通过 |

## 本轮静态门

- `rustfmt --edition 2021 --check`：25/25通过。
- 两个Python模块初次为9 pass、1 failure、1 error，原因是契约仍读取已经模块化删除的
  `play_pending_decision/tests.rs`和旧`jobs/system/mod.rs` owner。路径修复后11/11通过，断言语义未变。
- scoped `git diff --check`通过，仅有现存LF/CRLF提示。
- docs convention检查3,134 documents / 83,697 checked paths；全库既有801 violations影响275份
  文档，本轮两份performance产物为0 violation。
- 未运行Rust/Cargo、WPR、allocator、功耗或RenderDoc：managed validator session已归档，且没有
  current-source可执行文件。不得写入`review.md`，不得声明性能瓶颈已消失，也不触发里程碑commit/企微。
