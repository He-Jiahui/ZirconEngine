---
related_code:
  - zircon_editor/src/core
  - zircon_editor/src/core/gateway
  - zircon_editor/src/core/editor_message
  - zircon_editor/src/core/editor_event
  - zircon_editor/src/core/jobs
  - zircon_editor/src/core/runtime_event_consumer
  - zircon_editor/src/core/notifications
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
  - docs/plans/zircon_editor/editor/12-plugin-management.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
  - docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
reference_sources:
  - dev/bevy/crates/bevy_tasks/src/usages.rs
  - dev/bevy/crates/bevy_tasks/src/task_pool.rs
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Async/TaskGraphInterfaces.h
  - dev/godot/editor/file_system/editor_file_system.cpp
  - dev/Fyrox/editor/src/plugin.rs
tests:
  - zircon_editor/src/core/gateway/session.rs::tests::owned_output_decode_does_not_repeat_validation
  - zircon_editor/src/core/editor_plugin.rs::tests::descriptor_runtime_manifest_matching_uses_one_index
  - zircon_editor/src/core/editor_extension.rs::tests::extension_registry_validation_does_not_collect_path_segments
  - zircon_editor/src/core/editor_message/inbox.rs source guards
  - zircon_editor/src/core/editor_event/retention.rs source guards
  - zircon_editor/src/core/editor_event/listener/registry.rs source guards
  - zircon_editor/src/core/notifications/decision/tests.rs::resolving_a_notification_releases_pending_capacity
  - current-source Windows Cargo and editor product traces pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor core gateway、消息事件与任务逐文件性能静态审查（2026-07-22）

## 范围与覆盖

`zircon_editor/src/core`当前257个Rust文件、约32,046物理行。本轮已逐文件阅读root 7/7、`gateway` 8/8、`editor_message` 27/27、`editor_event` 32/32、`jobs` 20/20生产/内联夹具文件、`runtime_event_consumer` 6/6、`notifications` 7/7生产文件，共 **107/257**；其余150个文件及`jobs/tests`、`notifications/decision/tests.rs`等外部测试仍在`pending.md`。这只是静态生产代码覆盖，未冒充动态验收。

受管Cargo再次申请时被Session `runtime07-diagnostic-log-process-lifecycle-20260722`的reservation `7ea414f887334e3e8d1f7736c04c9184`占用，没有运行raw Cargo。当前切片不直接提交GPU命令；viewport真实GPU resident/import与readback问题继续归PERF-MVP-023，未伪造RenderDoc capture。

## 已确认的性能形状

- `SessionGateway`过去对owned ABI output先validate、再由`bytes()`重复validate；本轮删除重复检查。更大的P0仍在：`tick_frame`验证`ZrRuntimeFrameDemandV1`后丢弃kind/delay并恒返`true`，编辑器无法按runtime demand降频；`capture_frame`必须把runtime-owned RGBA复制到新`Vec<u8>`再释放foreign buffer，仍属于PERF-MVP-424/023。
- `EditorPluginCatalog::from_descriptors`过去以descriptor×runtime manifest线性匹配，本轮改为first-wins borrowed index。operation/menu-path/extension唯一性验证改为流式借用和单次`BTreeMap::entry`。但是每插件extension registry与merged registry仍同时深存全部descriptor/contribution，mutation后全量merge；继续由PERF-MVP-538与Editor12统一immutable generation收口。
- message inbox已经有lossless/bounded/latest分类和共享delivery payload，但容量判断过去每次enqueue扫描混合`Vec`；本轮维护三类depth计数。剩余P0是一个全局bus mutex包住subscription、所有inbox、dirty set及fanout，latest coalesce仍线性扫、interior `Vec::remove`移位，request还先深clone custom JSON。
- event retention已经按class有记录/字节/年龄预算并共享`Arc` payload。本轮让ack单遍累计removed bytes、首末sequence直接读单调队列两端，listener status不再clone+merge+sort全部records。剩余P0是每条事件先用serde counting writer完整遍历一次JSON求精确bytes，LatestState线性找key，record/notify在全局listener锁下逐listener filter/enqueue。
- editor jobs复用runtime `JobScheduler`，已有priority/category/dependency ready index、类别并发配额、64 events/1ms泵预算及progress latest coalesce；这是正确基线。本轮把内建job topic从每tick parse改为构造期一次。剩余P0是submission和lifecycle event无全局entry/bytes/age背压，`emit`逐事件跨lifecycle/progress/queue三锁并clone稳定label，storm仍可在主线程消费侧堆积。
- runtime event consumer已有execution generation guard、锁外gateway/callback、round-robin、256 events/4ms与64/consumer预算。但gateway先无上限drain成完整Vec，之后全部append到无界pending，再按预算apply；因此预算不约束ABI搬运/RSS。每delivery还各做take与commit两次active-map加锁。PERF-MVP-069保持open。
- decision notification center的pending/receipt容量分别固定为128/256，payload广泛使用`Arc`，不属于MVP高频瓶颈。本轮仅把publish的全entries pending计数扫描改为维护O(1)计数，并增加“resolve后释放容量”回归。

## 参考引擎核对

- Bevy `bevy_tasks/src/usages.rs`按Compute/AsyncCompute/Io完成期限与资源特征共享池；Zircon editor job继续复用runtime scheduler，而不是按业务再造线程池。
- Unreal `TaskGraphInterfaces.h`以命名线程和优先级显式表达回主线程；Zircon应把有界completion batch提交到editor owner，不让worker直接持UI状态或让无界队列成为隐式调度器。
- Godot `editor_file_system.cpp:3354-3370`把批量reimport作为一个worker group提交并统一等待；可借鉴“批次+共享池+明确完成边界”，但Zircon还必须补entry/bytes/age背压与帧内pump预算。
- Fyrox editor集中在单一sync hook执行模型同步；Zircon保留集中同步点，但必须以generation/dirty/budget做增量，不能恢复为每帧全量拉。

## 本轮直接止损

1. gateway owned output只校验一次；plugin runtime manifest一次索引；extension/operation/menu validation删除临时集合、重复查表和无效String clone。
2. message inbox维护三类depth；event retention ack/diagnostics/status删除重复全队列扫描与records物化排序。
3. job topic构造期解析一次；decision notification维护pending O(1)计数，并覆盖resolve释放容量。

源码守卫、scoped `rustfmt --edition 2021 --check`与`git diff --check`通过；Decision core current-source独立`rustc --test`为 **16 passed / 0 failed**，包含新增容量释放回归。该独立folder门不替代整crate Cargo；Cargo、规模counter、产品交互和RenderDoc均未完成，因此不进入`review.md`。

## 动态验收

以subscribers/listeners/consumers/jobs **1/100/1k/10k**、payload **64B/1MiB/64MiB**、producer threads **1/16**、consumer stall **0/1/60s**运行：记录queue entries/bytes/oldest age、drop/coalesce、JSON traversal/clone bytes、global/per-owner lock wait/hold、ABI drain bytes、callback p95、main-thread pump p95与RSS。要求idle/stable generation无重复parse/full projection；paused consumer内存硬有界；单个慢插件不能吞掉其他consumer预算；job submit/lifecycle backlog有entry+bytes+age政策；frame demand能真正控制wake。F0/F4、reload/unload、order/sequence、request/ack/cancel/shutdown与真实viewport像素通过后方可迁入`review.md`。
