---
related_code:
  - zircon_runtime/src/input/mod.rs
  - zircon_runtime/src/input/runtime/default_input_manager.rs
  - zircon_runtime/src/input/runtime/input_driver.rs
  - zircon_runtime/src/input/runtime/recording.rs
  - zircon_runtime/src/input/runtime/input_state.rs
  - zircon_runtime/src/input/tests/input_manager/frame_state.rs
  - zircon_runtime/src/input/tests/input_manager/host_requests.rs
  - zircon_runtime/src/input/module
  - zircon_runtime/src/core/framework/input
  - zircon_runtime/src/tests/runtime_absorption/input_stack.rs
  - zircon_runtime/src/tests/runtime_absorption/input_stack/contracts.rs
  - zircon_runtime/src/tests/runtime_absorption/input_stack/action_mapping.rs
  - zircon_runtime/src/tests/runtime_absorption/input_stack/gamepad_bridge.rs
  - zircon_runtime/src/tests/runtime_absorption/input_stack/inventory/mirror_docs.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/cargo_gates/late/runtime_12.rs
  - tools/tests/test_runtime_input_stack_audit.py
  - tests/acceptance/runtime-input-stack-audit-owner-sync.md
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/input_stack_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/input_stack_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/input_stack_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/input_stack_anchor_inventory.py
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/session/events.rs
  - zircon_runtime/src/dynamic_api/session/events/keyboard_ime.rs
  - zircon_runtime/src/dynamic_api/session/events/gamepad.rs
  - tools/tests/test_runtime_dynamic_event_input_owner_structure.py
  - zircon_runtime/src/ui/surface/interaction_gate.rs
  - zircon_runtime/src/ui/dispatch
  - zircon_app/Cargo.toml
  - dev/bevy/crates/bevy_input/src
  - dev/godot/core/input
plan_sources:
  - docs/plans/zircon_runtime/runtime/index.md
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
status: in_progress
last_refined: 2026-07-23
---

# 12 输入栈与动作映射对齐

Runtime 12 current child-owner sync (2026-08-30): `input_stack_boundary` reports `expected_runtime_module_count = 27`, `expected_framework_module_count = 44`, `expected_test_module_count = 7`, `expected_guard_file_count = 6`, `missing_guard_files = []`, `public_surface_anchors = 31/31`, `runtime_12_guard_anchors = 5/5`, `missing_gamepad_abi_anchors = []`, `missing_cursor_host_request_anchors = []`, `missing_doc_anchors = []`, `missing_test_anchors = []`, `behavior_test_anchor_count = 25`, `missing_behavior_test_anchors = []`, `missing_cargo_gate_anchors = []`, `oversized_modules = []`, `mirror_docs_guard_present = true`, and `risks = []`. The runtime owner set explicitly includes `input/camera_controller/{free,orbit,pan}` and the indexed action evaluator/event-buffer children. Current status anchors are `Frame Input Contract`, `input_frame_contract_static_passed_cargo_pending`, `arbitration_judgement_documented_static_passed`, `action_contract_static_passed_cargo_pending`, `action_evaluator_static_passed_cargo_pending`, `action_context_static_passed_cargo_pending`, `action_axis_value_static_passed_cargo_deferred`, `action_config_static_passed_cargo_deferred`, `action_manager_registration_static_passed_cargo_deferred`, `action_axis_consumption_static_passed_cargo_deferred`, `input_recording_replay_static_passed_cargo_deferred`, `cursor_host_request_static_passed_cargo_deferred`, `gamepad_bridge_static_passed_cargo_pending`, and `runtime_12_input_stack_cargo_pending_gate_stays_explicit_until_input_validation`. Pending command anchors remain `cargo test -p zircon_runtime --lib input --locked -- --nocapture`, `cargo test -p zircon_runtime --lib action_map --locked -- --nocapture`, `cargo test -p zircon_runtime --lib gamepad --locked -- --nocapture`, and `cargo test -p zircon_app --locked`. `runtime_12_input_stack_mirror_docs_match_structure_audit_counts` keeps the plan, runtime index, input module doc, M0 review, and interface-convergence mirror aligned; production input behavior is unchanged.

输入域当前没有任何子计划认领：09 只管 `ui/surface/input` 的 UI 路由侧，本计划管其上游——原始输入契约、帧输入状态语义、动作映射层缺口、gamepad 接入路径。

## 现状与证据（2026-06-13 实仓盘点）

- **分层已成形**：契约在 `core/framework/input`（`InputButton/InputEvent/InputEventRecord/InputSnapshot`，经 `input/mod.rs:6,17` 再导出）；运行时在 `input/runtime/{default_input_manager.rs,input_driver.rs,input_state.rs}`（`DefaultInputManager`/`InputDriver`，:22 导出）；模块注册在 `input/module/{config,descriptor,module_type}.rs`。
- **缺口 1——无动作映射层**：公共面只有按键/事件/快照原语，无"动作（action）→ 绑定（binding）→ 上下文（context）"层——对照 Godot `InputMap`、UE Enhanced Input、Unity Input System 的 action asset，玩法代码当前只能硬编码物理按键。
- **缺口 2——帧语义未声明**：`input_state.rs` 的 just_pressed/just_released/按住时长语义、输入清帧时机与 03 帧循环的关系（`tick_frame_drives_loaded_level_before_clearing_frame_input` 测试名旁证：清帧在 level tick 后）未文档化为权威契约。
- **缺口 3——gamepad 在 app 侧**：gilrs 0.11 是 `zircon_app` 的 optional（`gamepad-gilrs` feature）——手柄事件进 runtime 契约的路径、热插拔与多手柄语义未入任何计划。
- **缺口 4——与 09 的交接面**：输入先到玩法还是先到 UI（UI 吃掉输入的仲裁）需要单点声明，09 的 interaction_gate 是 UI 侧闸，全局仲裁 owner 未定。
- 参考锚点（2026-06-13 实测核验，动工前先读——index 公约 §7.9）：
  - bevy_input 全家（按键/轴/手柄/触摸 + ButtonInput just_pressed 语义）— `dev/bevy/crates/bevy_input/src`
  - Godot InputMap/InputEvent（动作映射 + 事件冒泡仲裁）— `dev/godot/core/input/`（执行时核验子文件：`ls dev/godot/core/input/`）
  - UE Enhanced Input（概念对照；源码路径执行时核验：Glob `**/EnhancedInput/**`，无则仅语义锚）

## 目标

1. 帧输入语义权威化：snapshot/just_* 语义、清帧时机（与 03 帧序图互引）、事件 vs 状态双读口的使用判据文档化 + 测试锚。
2. 动作映射层立项决策：最小 action/binding/context 模型设计（数据驱动、可序列化、ABI-safe 跨 dynamic_api），实现排期独立切片。
3. gamepad 接入路径定稿：app 侧 gilrs 事件 → runtime `InputEvent` 契约的桥接面与热插拔语义。
4. 输入仲裁单点：UI 消费与玩法消费的优先级裁决（与 09 interaction_gate 交接）。
5. 输入录制/回放最小 runtime helper：复用 `InputEventRecord` 与 frame boundary，服务 headless 确定性回放和后续工具链。

## 非目标

- 不动 UI 内部路由（09 地盘）；不改 winit 事件抽取（app/平台宿主侧职责，只定契约）；录制/回放只做 runtime helper，不做 editor 录制 UI、资产文件格式或跨进程输入流。

### 全局硬约束（继承总计划 §4）

- 不新增 crate；硬切换；动态边界只传 ABI-safe 值；非网络语义 server 命名是 blocker。

## 执行前检查清单

1. `git status --porcelain -- zircon_runtime/src/input/ zircon_runtime/src/core/framework/input/`；活动会话避让。
2. 事实重核：`ls zircon_runtime/src/input/runtime/`；Grep `just_pressed|JustPressed`，path `zircon_runtime/src/input`（帧语义现状）；Grep `gilrs`，path `zircon_app/src`（gamepad 现状面）。
3. 基线记录：`cargo test -p zircon_runtime --lib input --locked` 通过数。

## 里程碑

### M0 输入链路审计与帧语义文档化

- 切片 0.1（纯文档 + 测试锚）：`docs/zircon_runtime/input/`（执行时核验镜像文档存在性）落输入链路图——platform/app 事件源 → `InputDriver` → `DefaultInputManager` → `InputSnapshot` → 玩法/UI 双消费；清帧时机标到 03 帧序图的 stage 位。验收测试锚：`input_snapshot_just_pressed_is_true_for_exactly_one_frame`、`frame_input_clears_after_level_tick_not_before`（归属 `input/tests/` 既有树）。DoD：链路图 + 两测试绿。
- 切片 0.2（裁决）：输入仲裁单点判词——UI 先吃（capture）还是玩法先吃，焦点态如何切换；与 09 的 interaction_gate 契约互引。DoD：判词落文档，09 计划交叉引用更新。

#### M0.2 输入仲裁判词（2026-06-13）

- Owner 分工：`zircon_runtime::input` 只负责 platform/app 原始事件归一、帧内 transition/accumulator 维护，以及 `InputSnapshot` / `InputFrameSnapshot` 读口；它不判断 UI 与玩法谁消费事件，也不复制 UI 的 capture、popup、focus、direct-target、bubble 路由规则。
- UI surface 优先：当事件命中活动 UI surface、存在 pointer capture、popup stack、文本/导航 focus，或 09 的 `interaction_gate` 能给出 UI route decision 时，事件先进入 09 的 UI 路由权威链。09 内部继续拥有 `frame_hit_test -> interaction_gate -> dispatch -> 组件/焦点/popup_stack` 的顺序和旁路清理。
- 玩法 fallback：headless、无活动 UI surface、UI 未处理、capture 释放、popup 关闭、focus 清空，或明确 route result 为 unhandled 时，事件进入玩法/action mapping。M1 的动作映射层必须消费“UI 过滤后的未处理输入流”或等价的 consumed/unhandled 标记，禁止重新实现 UI 路由。
- 当前硬判词：`玩法/action mapping 只消费 UI 未处理` 的输入流；该判词是 Runtime 12 与 Runtime 09 UI 路由的单一交接面。
- 焦点切换：pointer capture 与 popup scope 在生命周期内优先于玩法；文本输入与导航 focus 优先供 UI 控件处理；玩法恢复依赖 UI route unhandled、显式 focus clear、capture release、popup close，或无 UI/headless profile。全局调试/宿主快捷键若需要越过两者，必须另立 host-command 通道，不能塞进 gameplay action map。

### M1 动作映射层设计与最小实现

- 切片 1.1（设计定稿）：`InputAction`/`InputActionContext`/`InputBinding`/`InputActionMap`/`InputActionState` 签名草案（serde 可序列化、跨 dynamic_api ABI-safe；对照 Godot InputMap 的 action 字符串键 + bevy ButtonInput 泛型）；数据来源（项目配置文件）与 runtime 注册路径。落 `core/framework/input`（契约）+ `input/runtime`（求值）。
- 切片 1.2（实现 + 测试）：`action_map_resolves_chords_and_reports_just_activated`、`replacing_action_map_rebuilds_bindings_automatically`、`action_contexts_filter_gameplay_and_menu_maps_without_rebinding`（数据驱动锚）。调用方迁移：无强制（新增层；既有原语保留为底层读口）。
- DoD：`cargo test -p zircon_runtime --lib input --locked` 全绿；文档含"何时用 action vs 原始按键"判据。

### M2 gamepad 桥接与热插拔

- 切片 2.1：app 侧 gilrs 事件 → `InputEvent` 契约扩展（手柄按钮/轴/连接断开事件枚举补全，ABI-safe）；热插拔语义测试（`gamepad_disconnect_clears_held_state_without_panic`）。调用方迁移：`zircon_app` gamepad feature 路径（执行时枚举：Grep `gilrs`，path `zircon_app/src`）。
- DoD：手柄事件经统一契约可达动作映射层；`cargo test -p zircon_app --locked` 无回归。

### M4 输入录制/回放 backlog 收束

- 切片 4.1：`zircon_runtime/src/input/runtime/recording.rs` 新增 `InputRecording`、`InputRecordingFrame`、`InputReplayCursor`、`InputReplayFrameReport`。录制侧只消费 `InputManager::drain_event_records()`，回放侧只经 `InputManager::begin_frame()` 与 `submit_event(...)` 重放，不绕过 `DefaultInputManager` 状态机。
- 调用方迁移：无强制迁移；`zircon_runtime::input` 与 prelude 导出新 helper，headless/session 工具可按需采用。
- 验收：行为锚 `input_recording_captures_drainable_event_records_by_frame` 与 `input_replay_restores_frame_snapshots_in_recorded_order`；结构审计同步 runtime/test/public-surface 计数。
- DoD：runtime helper、文档、结构审计和状态行同步；Cargo input/action_map/gamepad/app gates 继续由 M3 验证门统一后置。

### 测试阶段（milestone-first，每里程碑末）

- `cargo test -p zircon_runtime --lib input --locked -- --nocapture`；M2 加 `cargo test -p zircon_app --locked`。

## 状态与产出记录

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

本子计划产出记录已超过 10 条，具体记录已迁入编号子目录。

- 迁入记录：[`12/2026-07-09-input-stack-and-action-mapping-output-records.md`](12/2026-07-09-input-stack-and-action-mapping-output-records.md)
- 2026-07-18 picking性能交接：`core/framework/picking` 23/23静态审查已把per-pointer重复全outputs扫描/排序、owned hit二次clone、previous hover与release state深clone局部止损。Runtime12需与Runtime07把pointer inputs、resolved hits、hover/event/report固化为单帧事务与可复用双buffer workspace，并对drag targets×hovered事件放大设置计数/预算；UI仲裁判词不变，scene picking仍只消费UI未处理输入。验收矩阵与剩余架构项见PERF-MVP-332。
- 2026-07-18 input性能交接：framework input 26/26与runtime input当前29/29静态审查已完成；context线性查找、axis双遍历、release key clone及descriptor config clone已局部止损。Runtime12需与Runtime07硬切dense ActionId/ContextId generation、borrowed domain `InputFrameView`、reused evaluation scratch及frame/device-sharded collector，避免global manager/action Mutex和每帧完整IME/file/window payload clone；PERF-MVP-003的bounded raw recording/coalescing继续作为下层合同，UI先处理判词不变。规模门禁见PERF-MVP-334。
- 2026-07-22 export host输入补充：Plugins09 generated WebGPU/WASM仍逐`pointermove`同步JS→WASM，Android/iOS仍逐pointer/touch跨JNI/Swift ABI，viewport metrics也未帧内合并。Runtime12负责定义统一edge-preserving contract：begin/end/cancel/key严格保序，move为latest-position并累计raw delta，metrics每帧至多一次；Plugins09在host边界先合并以避免跨语言调用。125/500/1000 Hz counter、active-pointer线性上限、desktop/browser/mobile snapshot parity完成前，PERF-MVP-052与对应failure保持open。
- 2026-07-23 interface window输入补充：`ui/window/**`当前batch仅合并相邻redraw，move/raw-motion/touch/wheel/axis/drag-over都逐项保留；每input还双clone `UiWindowId(String)`，keyboard/gamepad/control逐event分配，ABI payload无条件复制。Runtime12按PERF-MVP-297/314/426发布typed key/control/window identity与per `(window,device,pointer/control)` accumulator：press/release/cancel/key/text/IME保序，position取latest、raw motion/scroll累加delta、axis取latest；ingress以entries+bytes+age硬限，drain以count+time预算，late-invalid报告index且不得制造无界废弃partial work。参考Bevy `AccumulatedMouseMotion/Scroll`的per-frame Copy resource，但不得跨Runtime09 geometry barrier合并。验收125/500/1000 Hz与100k burst的String alloc、payload copy、coalesce、edge、queue与p95。
- 2026-07-23 interface dispatch输入补充：`ui/dispatch/**` clean 18/19确认metadata/key/control/timer identity仍为per-event String，normal result可保留full event/reply/diagnostics；IME surrounding text虽有4,000-byte上限，composition rect Vec仍无预算。Runtime12把typed identity与move/analog accumulator贯穿window→dispatch，不在dispatch result重新复制；IME request按platform entry/bytes/rect count设硬限并在边界物化shared text ranges。回链PERF-MVP-296/297/314/426，验收1000 Hz输入与10k rect恶意/错误payload的queue age、String/Vec bytes、edge保序和typed rejection。
- 2026-07-23 clean contract tests性能补充：interface window/runtime-adapter测试只验证相邻redraw合并、单事件映射与3-event batch顺序；没有move/raw-motion/wheel/axis coalesce、entries+bytes+age硬限、late-invalid partial work或p95门禁。Runtime12把PERF-MVP-297/314/426的验收扩为125/500/1000 Hz和100k mixed events，记录event/result clone bytes、coalesce、edge、queue/drop/age与p95；press/release/cancel/key/text/IME严格保序，Runtime09 geometry barrier不得跨越。
- 2026-07-23 App current-source纠偏：`runtime_entry_app/**`74/74确认keyboard fallback已用`fmt::Write`零中间String、gamepad drain已有256 events/2ms预算、rumble已有32 effects/gamepad上限与清理；这些保持static pending Cargo/backend/storm，不得继续写成“无预算/无界”。PERF-MVP-426剩余owner是gilrs producer wake、pointer/raw-motion/wheel/axis帧内accumulator、逐事件ABI/identity/payload copy收敛及queue peak/age/drop/coalesce观测；沿用[`12/failure-2026-07-19-app-entry-input-and-gamepad-storm-budget.md`](12/failure-2026-07-19-app-entry-input-and-gamepad-storm-budget.md)。

## 2026-08-27 Dynamic Input Adapter Owner Split

状态：`runtime_10_12_15_dynamic_event_keyboard_ime_gamepad_owner_split_static_passed_cargo_deferred`。

Dynamic ABI 到 neutral `InputEvent` 的 adapter 已按输入域拆分：keyboard/IME owns text/IME
payload 解析与 UI input projection，gamepad owns connection/button/axis projection 以及共享 UI
navigation/analog mapping；734 行 event root 只路由各输入域并保留 pointer/window/lifecycle
协调。父/child 都继续调用同一个 `RuntimeDynamicSession::submit_input_event` 与 runtime UI
dispatch owner，没有建立第二个 input manager、action map 或 event queue。

本切片不改变当前 physical input 与 UI dispatch 顺序，不调整 gamepad threshold、IME byte
range、payload limit、coalescing 或录制策略，也没有性能样本。Python structure/status guard 2/2、
定向 rustfmt/diff check 通过；Cargo、app host、UI consumed/unhandled 和设备后端验证延后，
Runtime12 仍为 `in_progress`。
