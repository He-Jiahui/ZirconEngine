---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: runtime-ui-input-route-diagnostics-and-effect-ownership
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/surface/input/route_policy.rs
  - zircon_runtime/src/ui/surface/input/route_authority.rs
  - zircon_runtime/src/ui/surface/input/route_steps.rs
  - zircon_runtime/src/ui/surface/input/effect.rs
  - zircon_runtime/src/ui/surface/input/pointer_reply.rs
  - zircon_runtime/src/ui/surface/input/state
  - zircon_runtime/src/ui/tests/runtime_input_manager.rs
  - zircon_runtime/src/ui/tests/runtime_input_manager
  - zircon_runtime/src/ui/tests/runtime_ui_support
  - zircon_runtime/src/ui/tests/runtime_ui_window_event_routes
  - zircon_runtime/src/ui/tests/runtime_window_input_pump.rs
  - zircon_runtime/src/ui/tests/runtime_window_input_pump
  - zircon_runtime/src/ui/tests/event_routing.rs
  - zircon_runtime/src/ui/tests/event_routing
  - zircon_runtime/src/ui/tests/runtime_input_ownership.rs
  - zircon_runtime/src/ui/tests/runtime_input_ownership
tests:
  - route diagnostics opt-in and bounded capture test
  - effect arena ownership and linear merge test
  - million-event state budget and edge-order test
  - typed batch barrier, geometry commit, and render-only coalescing test
---

# Runtime UI输入路由诊断、effect与state所有权

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_runtime/src/ui/surface/input` 57/57；`tests/runtime_input_manager*`与`tests/runtime_ui_support/**` 10/10
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md`
- 联动责任：Runtime12拥有typed input/coalescing contract；runtime interface拥有dispatch/effect DTO。
- 交接原因：route、reply/effect分类、capture/popup/timer/drag state均属于EditorUI01统一输入authority。

## 失败现象与复现证据

PERF-MVP-293/294/297/314：默认每event生成多份owned route diagnostics与route-step DTO，keyboard/text尾部clone完整event；effect payload在reply/applied/host/component/rejected重复拥有，merge原先另建index map并线性重映射；analog/popup/timer/drag/capture state使用String key、线性容器或缺budget。测试`RuntimeUiManager`的batch API又逐项调用单事件入口并在每项后可能rebuild，无法验证frame内合并；整批末尾一次rebuild则会破坏resize后pointer依赖的新几何。

本轮已删除specialized trace被覆盖的generic构造、effect remap临时map/O(E²)查找、非terminal pointer capture map clone与owner清理临时Vec，并让navigation key无分配匹配。完整证据见`docs/plans/performance/01/2026-07-18-runtime-ui-surface-input-static-review.md`。

## 最低共享层根因

route、effect与input state没有单一generation/lifecycle authority；diagnostics被建模为每event必有的owned product，effect分类复制payload而不是引用一次执行记录，临时状态没有统一容量与过期策略。

## 架构修复验收

- 单event route authority=1；release默认只产轻量summary，完整route/stage/steps仅显式capture并受entry+byte+age预算。
- effect payload authoritative owner=1，reply/applied/host/component/rejected仅保存stable index+状态；merge近O(E)，ABI/serde兼容。
- typed/interned control id与indexed popup/timer/drag/capture state；所有长期state有entry/byte/age hard budget、drop/coalesce counters和shutdown flush。
- pointer/analog可frame内latest/delta coalesce；press/release/cancel/capture与drag/drop边沿不丢不重且顺序不变。
- 产品adapter与测试helper共用typed batch barrier：window/geometry变化先提交layout，render-only dirty在barrier或帧尾合并；纯move/analog burst的layout/render/hit rebuild为常数预算，resize后首个pointer使用新几何，错误保留原始batch index。
- route depth 1/16/64、effects 1/10/1k、125/500/1000 Hz与连续1M事件记录clone/alloc bytes、visits、payload owners、state bytes/age和CPU p95；current-source Cargo与F4产品trace通过。

## 禁止临时方案

- 不得仅在release关闭全部diagnostics；轻量summary与显式bounded capture都必须可观测。
- 不得新增另一份effect分类cache或复制arena payload到host queue。
- 不得通过静默丢弃press/release/cancel来满足队列上限。

## 修复结果与回传

Open state: `等待EditorUI01联动Runtime12/runtime interface回传shared route、single-owner effect、bounded indexed input state、typed batch barrier/coalescing、规模counter与current-source Cargo/产品trace`。
