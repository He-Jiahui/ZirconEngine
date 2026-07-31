---
related_code:
  - zircon_runtime_interface/src/runtime_api
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
  - docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
  - docs/plans/zircon_editor/editor/12-plugin-management.md
reference_sources:
  - dev/bevy/crates/bevy_winit/src/winit_config.rs
  - dev/bevy/crates/bevy_winit/src/state.rs
  - dev/godot/main/main.cpp
tests:
  - zircon_runtime_interface/src/tests/contracts.rs
  - zircon_runtime_interface/src/tests/abi_safety_contracts.rs
  - zircon_runtime_interface/src/tests/runtime_operation.rs
  - current-source Windows runtime-api tests pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime interface runtime-api 性能静态审查（2026-07-23）

## 范围与覆盖

`zircon_runtime_interface/src/runtime_api/**`当前源 **10/10** 个 Rust 文件、**1,554** 行已逐文件阅读，并读取 V3 layout/reactive-wake、host-request、plugin-event与operation合同测试。生产反查覆盖 runtime FFI producer、App frame loop、Editor gateway与 runtime-event consumer。`api_table.rs`为其他会话修改，`frame_demand.rs`、`session.rs`为其他会话未跟踪新增文件；本轮只读保留，未吸收或修改。

## 性能结论

- V3 `ZrRuntimeFrameDemandV1`以 Copy raw carrier表达 Idle/Immediate/After，App runtime entry已把它转换为 `RuntimeFrameDemand`并驱动 `Wait/WaitUntil`；但 Editor `SessionGateway::tick_frame`只验证 demand后恒返回 `true`，上层继续按外部 tick调用且无法消费 kind/delay。该 current-source边界继续确认 **PERF-MVP-424**，静态 interface落地不能替代 Editor产品 cadence验收。
- `EditorHostEventController::tick_runtime_event_consumers`每个 active play tick都深克隆完整 `EditorCapabilitySnapshot`（enabled/disabled/diagnostics），再对 enabled capabilities执行第二次 `to_vec`；随后 `reconcile_enabled_capabilities`全量克隆 registrations、构建 desired `BTreeMap`、active key `BTreeSet`与removed/added临时 Vec，即使 capability与registrations均未变化。stable 60/120/240 Hz持续产生主线程锁、O(C+R log R)比较和 String/registry clone，新增 **PERF-MVP-565**。
- `ZrRuntimeEventV1`是固定大小 Copy event carrier，keyboard/IME/file/gamepad payload借用 caller bytes，interface本身不分配；但函数表仍逐事件同步跨 ABI，没有 motion/axis latest合并或 batch barrier，继续归 **PERF-MVP-426/314**。resize/scale与pointer的几何屏障和按事件边沿保序不能由简单丢事件代替。
- `ZrRuntimeHostRequestBatchV1.requests`与 IME surrounding text为 owned Vec/String，ABI drain没有 count/bytes/deadline参数；非空请求会整批 JSON encode/decode并在主线程应用，继续归 **PERF-MVP-425**。empty fast path已有 PERF-MVP-002证据。
- plugin-event drain函数只接收 subscription和 output pointer，没有 `max_events/max_bytes/deadline/cursor`；DTO batch也允许任意 Vec/JSON Value。runtime producer虽对 encoded page设上限，预算尚未从 typed producer贯穿到 Editor申请，继续归 **PERF-MVP-069/432**。
- operation poll每次返回拥有 `message: String`的 JSON progress；runtime handler、session lock、无界 task/result与 per-poll message allocation继续归 **PERF-MVP-435/430**。frame RGBA owned buffer继续归023，本切片不重复编号。
- wake sink/config与 V3 table的 layout tests已存在，但异步 producer wake coalesce、destroy/quiescence和 idle Editor CPU尚无产品证据；保持 Runtime10 `implemented_static_pending_atomic_runtime_app_migration`边界。

## PERF-MVP-565 设计

1. Editor12把 `EditorCapabilitySnapshot`发布为 `Arc` generation（包含 enabled dense/set索引），manager getter只clone Arc；Plugins01/Runtime06 reload/toggle每次变化发布一次新generation。
2. Editor02 runtime-event consumer host保存 `last_reconciled_capability_generation + registration_generation`，只在 begin play或任一generation变化时计算 desired delta。稳定 tick只执行 runtime tick与有预算 pump，不取 capability/registry/active locks，不建 Map/Set/Vec。
3. changed generation一次性按 compiled required-capability slot更新 affected subscriptions；失败回滚仍保持 generation-safe，session end/reload/unload语义不变。不得用每帧 hash完整 Vec冒充 generation。

其余 cadence、host-request、plugin-event和operation问题继续使用既有 owner，不建立第二套队列或 frame-demand authority。

## 参考引擎对照

Bevy Winit显式区分 Continuous与 Reactive，并把 reactive deadline映射为 `ControlFlow::WaitUntil`；Godot low-processor模式只在 rendering state变化时draw。Zircon V3 demand形态正确，但所有host（包括Editor）都必须把 demand传到唯一 cadence owner，稳定 tick也不能夹带与frame无关的 capability全量 reconcile。

## 动态验收

1. current-source V3 interface/runtime/App/Editor layout、unknown demand、delay clamp、wake/destroy和hard-cut tests；旧 V2 symbol/table/ConfigV1不得回流。
2. focused/unfocused/occluded idle各30秒，demand Idle/Immediate/After，event storm 1/1k/10k：记录tick/redraw/wake、main CPU、delay error与p95；Editor不丢 demand，idle wake接近真实请求。
3. capabilities/registrations/active consumers各1/100/10k，stable 60/120/240 Hz与1% reload：记录snapshot/Vec/String clone、registry/active locks、Map/Set build、subscribe/unsubscribe和p95；stable全部为0，changed reconcile≤1/generation且近 affected。
4. input/host/plugin/operation burst记录ABI calls、encoded/decoded bytes、queue entries/bytes/age、remaining/drop与main-thread apply；满足426/425/069/435各自预算和边沿语义。

动态门禁、atomic V3 migration与 F0/F4产品 trace未完成，因此该目录继续保留在 `pending.md`，不进入 `review.md`。
