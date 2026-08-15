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
  - dev/UnrealEngine/Engine/Plugins/EnhancedInput/Source/EnhancedInput/Private/EnhancedPlayerInput.cpp
  - dev/bevy/crates/bevy_input/src/button_input.rs
  - dev/bevy/crates/bevy_input/src/gamepad.rs
  - dev/godot/core/input/input_map.cpp
  - dev/godot/core/input/input_map.h
tests:
  - 2026-07-18 framework/runtime input source-count snapshot retained as historical scope only
  - 2026-08-15 current-source re-review of action_evaluator generation, workspace, axis, and consumed-input owners
  - source-guard RED to GREEN for owned release, compiled generation, reusable workspace, and single-pass axes
  - current-source Cargo, WPR, allocation counters and F2/F4 traces pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime input framework与实现逐文件性能静态审查（2026-07-18）

## 范围与覆盖

2026-07-18 的逐文件账本覆盖`core/framework/input/**`与`zircon_runtime/src/input/**`、app host bridge、dynamic session 清帧顺序、manager resolver 和 action manager 调用；其中的文件/行数仅描述当日快照。2026-08-15 复审已纳入新拆分的 `action_evaluator/{generation,workspace,frame_axis_index,consumed_input_index}`，本记录以下结论以当前源码为准，而不以旧文件数宣称完整性。

现有PERF-MVP-003止损仍成立：frame events按帧清理，连续cursor/mouse motion合并；raw recording默认关闭，开启后使用有界VecDeque并发布discard计数。该结论不等于完整input验收，current-source Cargo、125/500/1,000 Hz WPR与产品消费trace仍缺失。

## PERF-MVP-334：action evaluation重复索引与owned frame复制

原action evaluator虽已有action→binding index，但每个action仍线性扫描全部contexts查询enabled；每个axis binding分别遍历一次求current value、再遍历一次求transition。`ButtonInputState::release`先remove并丢弃集合内owned key，再clone lookup key到just_released；input module descriptor还无条件clone整份owned config后只使用clone。

当前 evaluator 在 action-map 变更时构建 immutable `ActionEvaluationGeneration`：action 槽、binding ranges 和 context 槽不再在每帧重新解析。每次 evaluation 复用 `ActionEvaluationWorkspace` 的 action/context 存储；axis current/transition 和 UI-consumed input 则加载到可复用的排序 `Vec` 索引后按二分查询，保持 transition-only frame 语义。release 使用`BTreeSet::take`把存量 key 直接 move 进 just-released；descriptor 把 owned config 直接 move 进 factory capture。10/100/1,000/10,000 binding/frame-axis source-visit 和 warmup 后 workspace 不增长的测试是行为与规模锚，但不是 CPU、分配或功耗数据。Cargo FIFO 未开放，因此不写动态通过。

## 剩余架构热点与参考引擎结论

`DefaultInputManager`仍把全部设备状态、frame queues、IME/file payload 和 recording 放在一个 Mutex；每 event 在锁内更新状态、可选取系统时间/clone raw event 并写 queue。`frame_snapshot`仍在锁内深 clone 按钮三态及十余个 Vec/字符串 payload，即使 action evaluator 只需 buttons/gamepad axes/transitions；`DefaultInputActionManager`仍以全局 Mutex 串行只读 evaluation。当前 evaluator 已不再每帧建 `BTreeMap` 或线性查找 axis transition，但仍会按帧排序 frame-axis 与 consumed-input 索引，并在投影 `InputActionState` 时建立 `BTreeSet`/`BTreeMap`、clone String action ID；这些成本、快照复制和锁竞争必须分别测量。`drain_events`以`mem::take`把 frame Vec 容量交给短命 consumer，下帧可能重新分配。

Unreal `UEnhancedPlayerInput` 将 mapping rebuild（`ConditionalBuildKeyMappings_Internal`）与逐 tick 的 `EvaluateInputDelegates` 分开，并在阶段边界放置 CPU profiler scope；这支持当前 Zircon 的 generation/workspace 分层。Bevy 以 HashSet-backed `ButtonInput` 提供近 O(1) membership，并将 gamepad digital/analog state 按设备组件组织；Godot InputMap 将 action/event 关系集中在注册数据而非每帧字符串全表解析。若 WPR 证明当前排序、String 投影或 Mutex 是主导成本，Runtime12/07 的候选结构性方向才是 dense `ActionId`/`ContextId`、借用 `InputFrameView`、caller-owned/reused scratch，以及按 frame 双 buffer 或 device shard 采集；不得把静态 O 记号当成变更授权。UI consumed input 继续遵守 Runtime12 既有“UI先处理、gameplay只消费 unhandled”判词。

## 2026-08-15 算法复审与实施前决策门

本次复审读取了当前 `action_evaluator/{generation,workspace,frame_axis_index,consumed_input_index}`、`default_input_{manager,action_manager}` 与 `InputActionState`，并以 Unreal Enhanced Input 为首要行为参考，Fyrox 的 frame-local `FxHashMap/FxHashSet` 快捷状态和 Bevy `ButtonInput` 的集合成员查询作为 Rust 落地对照。当前数据流是：事件在 `DefaultInputManager` 的状态锁内归并，`frame_snapshot` 在同一锁内物化完整 owned frame；action manager 再持有 evaluator 锁，workspace 为每帧准备 contexts/actions、排序 axis 与 consumed-input 索引，然后投影 `BTreeSet/BTreeMap<String, _>`。action-map 更新路径则一次性构建 generation 的 context 和 binding ranges。该分界正确地避免了稳定 map 的重复 binding/context 重建，但尚不能证明 snapshot、排序、输出字符串或锁不是实际主导项。

因此本轮不以“全部换成 dense ID”作为预设实现。实施必须先用同一产品副本测得以下归因，再只选择命中的一项向前收敛：

1. 若 `frame_snapshot` clone bytes/allocations 主导，先设计只暴露 buttons、gamepad axes/edges 的借用 `InputFrameView`，IME、拖放和窗口 payload 仍由各自订阅者按需物化。
2. 若 `FrameAxisIndex` 或 consumed-input 排序主导，先比较低设备/高设备矩阵；仅在高基数稳定复现时才设计 dense per-device axis slots，不能以稀疏 MVP 手柄状态替换为常驻大表。
3. 若 `project_action_state` 的 String clone/ordered-container 分配主导，先在运行时内部使用稳定 `ActionId` 输出，再在 serialization、diagnostics 或跨 ABI 边界一次性投影文本；公共 `InputActionState` 行为、顺序和 UI-consumed 判词必须保持。
4. 若 manager/action lock wait 主导，才评估 frame-sequenced immutable snapshot 或双缓冲读取。此项必须先证明 focus-loss、edge、record/replay、host request 与 UI consume 的单帧可见性不会被并发读取改变。

建议的受管测量轮次使用现有 `tools/ui-profile-capture.ps1` 的 E:/D:/F: 输出根和 coordinator 分配的 `CARGO_TARGET_DIR`，不在 C: 或仓库 `target/` 写入。每个 workload 至少三次，分别记录 enqueue、`frame_snapshot`、evaluator lock wait/hold、workspace prepare、axis/consumed load-sort-lookup、binding evaluation、`InputActionState` projection 的 p50/p95/p99、allocated bytes/count、RSS、generation rebuild count 和 queue depth。workload 固定为 125/500/1,000/10,000 Hz，devices 1/8/64，bindings/actions/contexts 1/10/100/1k/10k，payload 0/16 B/1 KiB/1 MiB，recording off/on，threads 1/8/64；另用同一 `RenderableEmpty` 产品副本覆盖 F2 输入消费和 F4 UI-consumed 路径。

Windows managed validation 当前由 UI12 独占。本记录不启动 WPR、Cargo、产品进程或截图，并将上述动态轮次保持为待执行；在 UI12 成功 reservation/start 后，才以单个最小 workload 先建立前置基线，随后按命中的归因设计和验证算法变更。

## 验收要求

按 events 125/500/1k/10k Hz、devices 1/8/64、buttons/axes/actions/contexts/bindings 1/10/100/1k/10k、payload 0/16 B/1 KiB/1 MiB、threads 1/8/64、recording off/on 记录 manager/action lock wait、event clones/timestamps、snapshot clone bytes、generation rebuild、axis/consumed-index sort 与 lookup、projected action-ID clone bytes、alloc/realloc、queue depth/drop/coalesce、CPU p50/p95/p99 与 RSS。目标判断包括：recording off timestamp/event clone=0，stable map generation rebuild=0，action-only view 不 clone IME/file/window payload，以及并发 read evaluation 不被单 Mutex 串行；这些都是待量化门槛而非当前通过声明。frame edge/order、focus loss、touch/gamepad disconnect、deadzone/hysteresis、IME/host requests、record/replay、UI consume、Cargo 及 F2/F4 产品 trace 全部通过前，两目录留在`pending.md`。
