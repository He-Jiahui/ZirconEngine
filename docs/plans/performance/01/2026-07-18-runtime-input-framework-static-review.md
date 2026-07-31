---
related_code:
  - zircon_runtime/src/core/framework/input
  - zircon_runtime/src/input
  - zircon_app/src/entry/runtime_entry_app
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
reference_sources:
  - dev/bevy/crates/bevy_input/src/button_input.rs
  - dev/bevy/crates/bevy_input/src/gamepad.rs
  - dev/godot/core/input/input_map.cpp
  - dev/godot/core/input/input_map.h
tests:
  - twenty-six of twenty-six framework input Rust files reviewed
  - twenty-nine of twenty-nine runtime input Rust files reviewed
  - source-guard RED to GREEN for owned release, compiled contexts and single-pass axes
  - rustfmt and scoped git diff check passed
  - current-source Cargo, WPR, allocation counters and F2/F4 traces pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime input framework与实现逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读`core/framework/input/**`当前Rust文件26/26（1,466行）及`zircon_runtime/src/input/**`当前Rust文件29/29（3,436行，含所有tests），并追到app host bridge、dynamic session清帧顺序、manager resolver与action manager调用。此前账本的22文件是旧计数；当前29文件包含新拆出的event buffer、binding/frame-axis index和record/replay owners。

现有PERF-MVP-003止损仍成立：frame events按帧清理，连续cursor/mouse motion合并；raw recording默认关闭，开启后使用有界VecDeque并发布discard计数。该结论不等于完整input验收，current-source Cargo、125/500/1,000 Hz WPR与产品消费trace仍缺失。

## PERF-MVP-334：action evaluation重复索引与owned frame复制

原action evaluator虽已有action→binding index，但每个action仍线性扫描全部contexts查询enabled；每个axis binding分别遍历一次求current value、再遍历一次求transition。`ButtonInputState::release`先remove并丢弃集合内owned key，再clone lookup key到just_released；input module descriptor还无条件clone整份owned config后只使用clone。

本轮已把context enabled-state编译进binding index，axis current/transition在一次binding-axis遍历中求值并保持transition-only frame旧语义；release使用`BTreeSet::take`把存量key直接move进just-released；descriptor把owned config直接move进factory capture。source guards先确认旧形态RED，再在新源码GREEN；已有10/100/1,000 binding/frame-axis source visit测试继续作为行为与规模锚，rustfmt和scoped diff check通过。Cargo FIFO未开放，因此不写动态通过。

## 剩余架构热点与参考引擎结论

`DefaultInputManager`把全部设备状态、frame queues、IME/file payload和recording放在一个Mutex；每event在锁内更新状态、可选取系统时间/clone raw event并写queue。`frame_snapshot`在锁内深clone按钮三态及十余个Vec/字符串payload，即使action evaluator只需buttons/gamepad axes/transitions；`DefaultInputActionManager`又以全局Mutex串行只读evaluation。每次evaluation仍重建两个FrameAxisIndex BTreeMap、consumed/context sets以及String keyed action outputs；gamepad axis transition按Vec线性find，同帧多设备可退化。`drain_events`以`mem::take`把frame Vec容量交给短命consumer，下帧可能重新分配。

Bevy以HashSet-backed `ButtonInput`提供近O(1) membership，并将gamepad digital/analog state按设备组件组织；Godot InputMap把action/event关系集中在注册数据而非每帧字符串全表解析。Runtime12/07应据此硬切compiled dense ActionId/ContextId与immutable generation map，evaluation消费借用`InputFrameView`和caller-owned/reused scratch；manager按frame双buffer或device shard采集，边沿事件有序、motion latest/delta累计，snapshot只按订阅域物化。UI consumed input继续遵守Runtime12既有“UI先处理、gameplay只消费unhandled”判词。

## 验收要求

按events 125/500/1k/10k Hz、devices 1/8/64、buttons/axes/actions/contexts/bindings 1/10/100/1k/10k、payload 0/16 B/1 KiB/1 MiB、threads 1/8/64、recording off/on记录manager/action lock wait、event clones/timestamps、snapshot clone bytes、axis/context/binding visits、alloc/realloc、queue depth/drop/coalesce、CPU p50/p95/p99与RSS：recording off timestamp/event clone=0，action-only view不clone IME/file/window payload，stable map rebuild=0，context lookup近O(log C)或dense O(1)，每axis binding visits=1，concurrent read evaluation不被单Mutex串行。frame edge/order、focus loss、touch/gamepad disconnect、deadzone/hysteresis、IME/host requests、record/replay、UI consume、Cargo及F2/F4产品trace全部通过前，两目录留在`pending.md`。
