---
related_code:
  - zircon_runtime_interface/src/ui/window
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
  - docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
reference_sources:
  - dev/bevy/crates/bevy_input/src/mouse.rs
  - dev/bevy/crates/bevy_input/src/lib.rs
tests:
  - zircon_runtime_interface/src/tests/window_input_contracts.rs
  - zircon_runtime_interface/src/tests/window_runtime_event_adapter_contracts.rs
  - zircon_runtime_interface/src/tests/window_transient_contracts.rs
  - zircon_runtime/src/ui/tests/runtime_window_input_pump.rs
  - current-source Windows window/input tests pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime interface UI window 性能静态审查（2026-07-23）

## 范围与覆盖

`zircon_runtime_interface/src/ui/window/**`当前源 **8/8** 个受跟踪且clean的 Rust 文件、**2,058** 行已逐文件阅读，并反查runtime winit翻译、`UiInputManager`批分发、surface入口、14条interface合同及14条runtime window-pump合同。本轮未修改Rust源码。

## 性能结论

- `event.rs`、`impact.rs`、`metrics.rs`和metadata facade主要是值类型合同；impact判词与transient-effect iterator无额外集合分配。`normalize(self)`大多移动既有字段，正向避免二次深复制；drag payload转成`Box`仍会发生一次堆分配。
- `UiWindowInputPumpBatch`是无entry/bytes/age上限的`Vec`。`push_coalesced`只删除**相邻 redraw**，没有合并pointer move、raw mouse motion、touch move、wheel、analog、drag-over或resize；ABI adapter甚至统一调用`push`。EditorUI01现有管线图“push_coalesced 合并 move”与当前源码不符，必须纠正。
- `runtime_events_to_window_input_pump_batch`逐项转换，遇到末尾非法事件会丢弃此前已完成的全部转换；没有事件索引、partial/transaction policy、reserve、count/time预算或overflow语义。下游`UiInputManager::dispatch_window_input_pump_batch`又按N逐项同步路由并分配容量N的result Vec，producer burst可直接放大主线程工作和驻留。
- 每个ABI input event的`input_context()`先在`window_metadata()`克隆一次`UiWindowId(String)`，`from_window_metadata()`再克隆一次，临时第一份随后丢弃。keyboard每event物化physical/logical两个String；`key_char`与controller button再复制同一正文，axis也每event分配control String。`payload_bytes()`无条件`to_vec()`；accessibility仅为JSON解析建立整份临时bytes，文本/IME/file-drop虽最终需要owned正文，也缺payload bytes上限。
- 以上不新增重叠编号：批预算、typed barrier和move/axis合并补强 **PERF-MVP-314**；typed control/window identity和高频状态补强 **PERF-MVP-297**；跨ABI payload/identifier分配与输入storm补强 **PERF-MVP-426**；逐事件完整路由结果/diagnostics继续回链 **PERF-MVP-293**。Runtime09拥有UI geometry/render barrier，Runtime12拥有edge-preserving accumulator，EditorUI01拥有window-pump批入口。
- Bevy在`PreUpdate`读取本帧全部mouse messages，把motion/scroll累加到Copy resource并每帧重置。Zircon采用该“frame accumulator”原则，但必须分离lossless边沿与可合并连续量：press/release/cancel/key/text/IME严格保序，pointer position取latest，raw motion/scroll累加delta，axis按device/control取latest；resize/scale必须形成geometry barrier，不能把resize后的首个pointer路由到旧布局。

## 动态验收

1. events 1/100/1k/100k、125/500/1000 Hz、devices/windows 1/8/64、window-id 0/64/4KiB、payload 0/1KiB/1MiB记录queue entries/bytes/oldest-age、coalesced/dropped、window/key/control String alloc、payload copied bytes、route/result count、layout/render/hit rebuild和主线程p95。
2. stable事件的window identity深clone=0，named keyboard/gamepad/control String alloc=0；accessibility JSON临时payload copy=0。batch ingress具entry+bytes+age硬上限，drain具count+time预算，overflow有可观测且可恢复的策略。
3. 纯move/axis burst每pointer/device/control每帧常数事件；raw delta守恒、latest position/value正确。resize/scale后首个位置相关事件必须先越过geometry barrier；press/release/cancel/key/text/IME/popup/drag drop边沿与error index保持顺序。
4. 当前14条interface与14条runtime window-pump合同通过；补100k late-invalid、mixed barrier、multi-device storm与allocation counter。运行current-source Windows Cargo gate及F4真实window/input trace。

current-source Cargo、规模counter与F4产品trace未完成，因此该目录继续保留在 `pending.md`，不进入 `review.md`。
