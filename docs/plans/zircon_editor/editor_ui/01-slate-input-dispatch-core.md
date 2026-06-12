---
related_code:
  - zircon_runtime/src/ui/dispatch/mod.rs
  - zircon_runtime/src/ui/dispatch/pointer/dispatcher.rs
  - zircon_runtime/src/ui/dispatch/navigation/dispatcher.rs
  - zircon_runtime/src/ui/surface/input/mod.rs
  - zircon_runtime/src/ui/surface/input/window_pump.rs
  - zircon_runtime/src/ui/surface/input/dispatch.rs
  - zircon_runtime/src/ui/surface/input/route_policy.rs
  - zircon_runtime/src/ui/surface/input/route_steps.rs
  - zircon_runtime/src/ui/surface/input/tooltip_timer.rs
  - zircon_runtime/src/ui/surface/input/state/mod.rs
  - zircon_runtime/src/ui/surface/focus.rs
  - zircon_runtime/src/ui/surface/interaction_gate.rs
  - zircon_runtime/src/ui/tree/hit_test.rs
  - zircon_runtime_interface/src/ui/dispatch/input/event.rs
  - zircon_runtime_interface/src/ui/dispatch/input/reply.rs
  - zircon_runtime_interface/src/ui/dispatch/input/effect.rs
  - zircon_runtime_interface/src/ui/dispatch/input/result.rs
  - zircon_runtime_interface/src/ui/window/pump.rs
  - zircon_runtime_interface/src/ui/window/input.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_input_translation.rs
  - zircon_editor/src/ui/retained_host/shell_pointer/bridge.rs
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Application/SlateApplication.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Input/Reply.h
plan_sources:
  - .codex/plans/ZirconEngine 宿主编辑器 UI 基础能力计划.md
  - .codex/plans/Shared Slate-Style UI Layout, Render, And Hit Framework.md
  - .codex/plans/Drawer_Window_Menu Slate 化推进计划.md
  - .codex/plans/布局系统.md
status: planned
---

# 01 Slate 式输入与事件内核

## 1. 目标

把现有「pointer 分发器 + navigation 分发器 + editor 侧逐区域 pointer bridge」收束为一个 Unreal `FSlateApplication` / `GenericApplicationMessageHandler` 风格的**统一输入管理层**：平台事件先归一化为 Zircon UI 事件，再按命中路径、焦点路径、捕获目标、弹窗层级路由；所有副作用通过 `UiDispatchReply` 声明式表达。鼠标、键盘、触摸、文本输入、IME、拖拽、弹窗关闭、Tooltip 计时都走同一个入口。

## 2. 现状（按代码核实修正）

### 2.1 已存在的内核设施（不重做，只收编）

| 能力 | 落点 | 证据 |
|------|------|------|
| 平台事件载体：鼠标/键盘/字符/IME（含 cursor range）/触摸 phase/导航/模拟量/手柄/drag-drop/popup/tooltip-timer/accessibility | `zircon_runtime_interface/src/ui/window/{pump,input}.rs` | `UiWindowInputPumpBatch`（pump.rs:28）、`UiWindowPlatformInputEvent`（input.rs:70）、`UiWindowPlatformInputEventKind`（input.rs:751）、`UiWindowTouchPhase`（input.rs:820） |
| 归一化事件模型（11 类 variant） | `zircon_runtime_interface/src/ui/dispatch/input/event.rs` | `UiInputEvent`（event.rs:13）：Pointer/Keyboard/Text/Ime/Navigation/Analog/MouseMotion/DragDrop/Popup/TooltipTimer/Accessibility |
| Reply 模型 | `.../dispatch/input/reply.rs` | `UiDispatchReply`（:42）、`UiDispatchPhase`（:8）、`UiDispatchDisposition`（:32）、`UiDispatchReplyStepTrace`（:237） |
| 副作用集合：focus/capture（含 reason、lock policy）/drag/popup/tooltip/clipboard/IME/redraw/transient-dismissal | `.../dispatch/input/effect.rs` | `UiDispatchEffect`（:17）及 :99–:287 各配套枚举 |
| 路由策略与诊断 | `.../dispatch/input/result.rs` | `UiInputRoutePolicy{Unrouted, PreviewTunnel, Bubble, Direct, FocusPath, PointerCapture, DefaultAction}`（:16）、`UiInputRouteTrace`（含 popup_stack，:43）、`UiInputDispatchDiagnostics`（:56）、`UiDispatchHostRequest(Kind)`（:82/:113） |
| runtime 批入口与窗口生命周期效果 | `zircon_runtime/src/ui/surface/input/window_pump.rs` | `dispatch_window_input_pump_event`（:19）、`dispatch_window_input_pump_batch`（:38）、窗口 focus/scale/close 效果（:139–:211） |
| Reply 统一应用器 | `zircon_runtime/src/ui/surface/input/` | `apply_dispatch_reply`（window_pump.rs:16 引入，dispatch 邻域实现） |
| 子域模块 | `zircon_runtime/src/ui/surface/input/` | popup.rs、drag_drop.rs、tooltip_timer.rs、keyboard_navigation.rs、editable_text/、text_keyboard/、analog.rs、accessibility.rs、route_policy.rs、route_steps.rs、state/（`UiSurfaceInputState`、`UiSurfaceDragDropState`） |
| 双分发器 | `zircon_runtime/src/ui/dispatch/` | `UiPointerDispatcher::dispatch`（pointer/dispatcher.rs:48）、`UiNavigationDispatcher::dispatch`（navigation/dispatcher.rs:32） |

即：Slate 对应物（Reply、路由策略枚举、host request、hit path）在 interface 层**已经定稿过一轮**，本计划不是从零新建，而是**收编与补全**。

### 2.2 真实缺口

1. **winit 翻译双实现**：editor 侧 `host_contract/native_input_translation.rs`（190 行，直接用 winit 类型）+ `native_keyboard.rs` + `native_pointer/` 一份；runtime 预览侧 `zircon_runtime/src/rhi/ui_surface.rs`、`rhi_wgpu/ui_surface.rs` 另一份。editor 直接依赖 `zircon_runtime`（zircon_editor/Cargo.toml:24，rlib），单实现可落 runtime owner 模块。
2. **无统一 manager 门面**：批入口以 `(surface, pointer_dispatcher, navigation_dispatcher)` 参数对穿透各层（window_pump.rs:19–56）；tooltip/双击计时、多指针实例表没有统一 owner。
3. **触摸不成体系**：`UiWindowTouchPhase` 已进平台事件，但 per-pointer-id 活动指针表、primary touch 鼠标语义合成、cancel 清理未实现（`state/` 只有单指针痕迹）。
4. **路由次序未单点固化**：策略枚举齐全，但 capture→popup→preview→direct→bubble→focus-path 的全链次序与外点关闭判定散在 dispatch.rs 与 route_policy.rs，缺一处权威实现与矩阵测试。
5. **editor 手写命中**：11 个 pointer bridge 家族 + tab_drag + drawer_resize 自带命中/hover/press 状态机：`activity_rail_pointer`、`asset_pointer`、`detail_pointer`、`document_tab_pointer`、`drawer_header_pointer`、`hierarchy_pointer`、`host_page_pointer`、`menu_pointer`、`shell_pointer/`（bridge/common/drag_frames/drag_surface/effects/node_ids/resize_surface/route 共 8 文件）、`viewport_toolbar_pointer`、`welcome_recent_pointer`（均在 `zircon_editor/src/ui/retained_host/`）。

## 3. 设计

### 3.1 事件归一化层（对应 GenericApplicationMessageHandler）

- `UiWindowInputPumpBatch` 维持唯一平台事件载体地位；按 M1 盘点结论补缺 variant（重点核对触摸与 IME preedit 区段表达）。
- winit → Zircon 翻译收口为 runtime 单实现（新增 `zircon_runtime/src/ui/platform_input/`，editor 与 runtime 预览两宿主共用）；editor host 只持有 EventLoop 并喂 batch，不再解释 winit 语义。

### 3.2 路由层（对应 FSlateApplication）

`zircon_runtime/src/ui/dispatch/input_manager/`（新增 owner 模块）：

- 入口 `UiInputManager::dispatch_window_batch(surface, batch) -> UiInputDispatchOutcome`，内部收编双分发器为阶段。
- 路由顺序固定且单点实现：**capture target → popup 层级（含外点关闭判定）→ preview/tunnel（root→leaf）→ direct（leaf）→ bubble（leaf→root）→ focus-path（键盘）→ default-action**，与现有 `UiInputRoutePolicy` 枚举一一对应。
- 鼠标：hover enter/leave 成对、press/release/click/double-click 合成、wheel 沿 hit path 冒泡到第一个可滚动节点、capture 期间只发 capture 目标。
- 键盘：focus path 路由；Tab/Shift+Tab 走 navigation 阶段；Enter/Space 激活；Escape 自顶向下关闭最上层 popup；其余字符进文本编辑链（计划 03）。
- 触摸：pointer id 映射为 `UiActivePointerTable` 独立条目，与鼠标共享 hit/route 核心；primary touch 合成鼠标语义。
- Tooltip：hover 驻留计时归 `input_manager::timers`，`tick()` 在帧首注入 `UiTooltipTimerInputEvent`；任何 pointer/keyboard 活动取消。

### 3.3 Reply 模型（对应 FReply）

`UiDispatchReply` / `UiDispatchEffect` / `apply_dispatch_reply` 已存在，本计划任务是**补全而非新建**：

- effect 覆盖盘点：`UiDispatchEffect` 每个 variant 必须有应用路径与测试；`UiDispatchRejectedEffect` 必须带可读 reason（result.rs:75 已有字段）。
- editor 不得自行实现副作用：editor 现有 bridge 内的 capture/hover/拖拽状态全部改经 reply。

### 3.4 Editor 收编

- 新增 `zircon_editor/src/ui/retained_host/route_intent/`：从 `UiComponentEventReport` / route id 反查 `EditorIntent`，pointer bridge 家族整体改造为该薄适配的消费者。
- 手写命中路径硬切换删除（按 M5 切片分批，每批同变更删除）。

## 4. 接口与数据结构草案

```rust
// 新增 zircon_runtime/src/ui/dispatch/input_manager/manager.rs
pub struct UiInputManager {
    pointer: UiPointerDispatcher,          // 现有，收编为内部阶段
    navigation: UiNavigationDispatcher,    // 现有，收编为内部阶段
    pointers: UiActivePointerTable,        // 新增：多指针实例表
    timers: UiInputTimerState,             // 新增：tooltip/双击计时（收编 tooltip_timer.rs 状态）
}

impl UiInputManager {
    /// 唯一批入口；取代 window_pump 中以分发器参数对穿透的旧形态
    pub fn dispatch_window_batch(
        &mut self,
        surface: &mut UiSurface,
        batch: UiWindowInputPumpBatch,
    ) -> Result<UiInputDispatchOutcome, UiTreeError>;

    /// 帧首计时驱动：到期 tooltip、双击窗口过期等，注入合成事件
    pub fn tick(
        &mut self,
        surface: &mut UiSurface,
        now: UiInputTimestamp,                 // 现有类型（metadata.rs:69）
    ) -> Result<Vec<UiInputDispatchResult>, UiTreeError>;
}

// 新增 input_manager/outcome.rs
pub struct UiInputDispatchOutcome {
    pub results: Vec<UiInputDispatchResult>,        // 现有类型（result.rs:129）
    pub host_requests: Vec<UiDispatchHostRequest>,  // 聚合出站请求（IME/剪贴板/光标形状）
    pub redraw_requested: bool,
}

// 新增 input_manager/pointer_table.rs
pub struct UiActivePointerTable {
    entries: Vec<UiActivePointerEntry>,
}
pub struct UiActivePointerEntry {
    pub pointer_id: UiPointerId,            // 现有类型
    pub source: UiPointerSource,            // 现有类型（metadata.rs:54）
    pub last_point: Option<UiPoint>,
    pub pressed_buttons: u8,
    pub capture_target: Option<UiNodeId>,
    pub is_primary: bool,                   // primary touch 负责鼠标语义合成
}

// 新增 zircon_runtime/src/ui/platform_input/winit_translation.rs（winit→batch 单实现）
pub fn translate_winit_window_event(
    context: UiWindowInputContext,          // 现有类型（window/input.rs:26）
    event: &winit::event::WindowEvent,
) -> Option<UiWindowInputPumpEvent>;
pub fn translate_winit_modifiers(state: winit::keyboard::ModifiersState) -> UiInputModifiers;

// 新增 zircon_editor/src/ui/retained_host/route_intent/map.rs
pub struct EditorRouteIntentMap { /* route id（编译期产物）→ EditorIntent 表 */ }
impl EditorRouteIntentMap {
    pub fn intent_for(&self, event: &UiComponentEventReport) -> Option<EditorIntent>;
}
```

## 5. 模块与文件落点

**新增**

| 路径 | 内容 |
|------|------|
| `zircon_runtime/src/ui/dispatch/input_manager/{mod.rs, manager.rs, routing.rs, pointer_table.rs, timers.rs, outcome.rs}` | 统一入口、路由次序单点实现、多指针表、计时、outcome；mod.rs 只做声明 |
| `zircon_runtime/src/ui/platform_input/{mod.rs, winit_translation.rs, keyboard_map.rs}` | winit→batch 翻译单实现（从 editor 翻译逻辑平移） |
| `zircon_editor/src/ui/retained_host/route_intent/{mod.rs, map.rs}` | route id → EditorIntent 薄适配 |

**修改**

| 路径 | 改什么 |
|------|--------|
| `zircon_runtime/src/ui/surface/input/window_pump.rs` | 批入口改走 `UiInputManager`；窗口生命周期效果（:139–:211）保留原位 |
| `zircon_runtime/src/ui/surface/input/dispatch.rs` | 路由次序改由 input_manager::routing 驱动；route_steps 注解保留 |
| `zircon_runtime/src/ui/surface/input/tooltip_timer.rs` | 计时 ownership 移交 input_manager::timers，本文件退化为纯事件构造或删除 |
| `zircon_runtime_interface/src/ui/window/input.rs` | 按 M1.S1 盘点补触摸/IME variant 缺口（集中一次，过 serde 兼容测试） |
| `zircon_editor/src/ui/retained_host/app.rs`、`event_bridge.rs` | 调用 platform_input 翻译，只喂 batch |
| 11 个 pointer bridge 家族 | 改读 route_intent；删除内部命中/状态机（见 M5 切片） |

**删除（硬切换义务）**

- `zircon_editor/src/ui/retained_host/host_contract/native_input_translation.rs`、`native_keyboard.rs`（M1.S3，翻译收口后）
- `zircon_editor/src/ui/retained_host/host_contract/native_pointer/` 中的翻译段（M1.S3）
- `zircon_runtime/src/rhi/ui_surface.rs`、`rhi_wgpu/ui_surface.rs` 中的重复 winit 翻译段（M1.S2）
- 各 pointer bridge 内手写命中/hover/press 状态机（M5 分批）；`host_contract/surface_hit_test/` 中被取代部分（M5.S4）

## 6. 管线时序

```
winit EventLoop（editor UiHostWindow / runtime preview window）
  → platform_input::translate_winit_window_event（单实现）
  → UiWindowInputPumpBatch（push_coalesced 合并 move）
  → UiInputManager::tick（帧首：tooltip/双击计时注入）
  → UiInputManager::dispatch_window_batch
      每事件：capture → popup(外点关闭) → preview/tunnel → direct → bubble → focus-path → default-action
      命中经 ui/tree/hit_test + interaction_gate（disabled 唯一守门员）
  → apply_dispatch_reply（effect 落 dirty domain；host_requests 聚合出站）
  → 帧管线后续：state reduce → layout → text → render extract → GPU command stream
```

## 7. 里程碑切片化

| # | 切片 | 交付物 / 涉及文件 | 验证命令 | 硬切换 |
|---|------|------------------|---------|--------|
| M1.S1 | 翻译盘点矩阵：以 editor `native_input_translation` 现行为金标准，固化 winit↔`UiWindowPlatformInputEventKind` 全 variant 映射测试；列出触摸/IME 缺口清单 | `zircon_editor/src/tests/host/retained_window/native_input_translation.rs`（扩充） | `cargo test -p zircon_editor --lib native_input_translation --locked` | 无删除 |
| M1.S2 | `platform_input::winit_translation` 单实现落地，rhi/rhi_wgpu ui_surface 翻译段并入 | 新增 platform_input/；改 rhi/ui_surface.rs、rhi_wgpu/ui_surface.rs | `cargo check -p zircon_runtime --lib --locked` | 删 rhi 两处翻译段 |
| M1.S3 | editor/app 切换调用方；删除 editor 本地翻译 | app.rs、event_bridge.rs；删 native_input_translation.rs、native_keyboard.rs | `cargo test -p zircon_editor --lib --locked` | 删 2 文件 + native_pointer 翻译段 |
| M1.S4 | interface 触摸/IME variant 补缺（按 S1 清单，集中一次） | `zircon_runtime_interface/src/ui/window/input.rs` | `cargo test -p zircon_runtime_interface --locked` | 无删除 |
| M2.S1 | input_manager 骨架：UiInputManager 持双分发器，window_pump 批入口改走 manager（行为等价收编） | 新增 input_manager/{mod,manager,outcome}.rs；改 window_pump.rs | `cargo test -p zircon_runtime --lib window_pump --locked` | window_pump 旧签名删除 |
| M2.S2 | 路由次序单点化：routing.rs 固化七阶段次序；dispatch.rs 改由其驱动 | 新增 routing.rs；改 dispatch.rs、route_policy.rs | `cargo check -p zircon_runtime --lib --locked` | dispatch.rs 内散落次序逻辑删除 |
| M2.S3 | 路由矩阵测试：capture 抢占、popup 外点关闭只关最上层、preview 先于 bubble、focus-path 键盘、default-action 兜底 | input_manager 模块内 #[cfg(test)] | `cargo test -p zircon_runtime --lib input_manager --locked` | 无删除 |
| M3.S1 | effect 覆盖盘点：`UiDispatchEffect` 每 variant 应用测试；RejectedEffect 全部带 reason | apply_dispatch_reply 所在模块 + 测试 | `cargo test -p zircon_runtime --lib dispatch --locked` | 无删除 |
| M3.S2 | 计时 ownership 移交：timers.rs 接管 tooltip/双击；tick() 注入合成事件 | 新增 timers.rs；改/删 tooltip_timer.rs | `cargo test -p zircon_runtime --lib tooltip --locked` | tooltip_timer.rs 旧计时态删除 |
| M3.S3 | 既有 drawer/menu/drag 测试在新路径全绿（回归闸门） | 无新文件 | `cargo test -p zircon_runtime --lib --locked`、`cargo test -p zircon_editor --lib --locked` | 无删除 |
| M4.S1 | UiActivePointerTable：per-pointer hover/capture 状态 | 新增 pointer_table.rs；改 state/mod.rs | `cargo test -p zircon_runtime --lib pointer_table --locked` | state 内单指针旧字段删除 |
| M4.S2 | primary touch 鼠标语义合成 + cancel 清理 | routing.rs、pointer_table.rs | `cargo test -p zircon_runtime --lib touch --locked` | 无删除 |
| M4.S3 | 多指针矩阵测试：两指独立 hover/press、cancel 清理、capture 隔离 | input_manager 测试 | 同上 | 无删除 |
| M5.S1 | route_intent 适配层 + 首迁 shell_pointer 家族（8 文件改读 route id，drag/resize 捕获走 reply） | 新增 route_intent/；改 shell_pointer/* | `cargo test -p zircon_editor --lib shell_pointer --locked` | shell_pointer 内命中码删除 |
| M5.S2 | 迁 document_tab / drawer_header / menu / activity_rail 四桥 | 各 bridge 文件 | `cargo test -p zircon_editor --lib --locked` | 四桥命中码删除 |
| M5.S3 | 迁 hierarchy / asset / detail / host_page / viewport_toolbar / welcome_recent + tab_drag / drawer_resize | 各 bridge 文件 | 同上 | 六桥命中码删除 |
| M5.S4 | 清残：删除 surface_hit_test 被取代部分；实机回归 | host_contract/surface_hit_test/ | `cargo test -p zircon_editor --test integration_contracts --features integration-contracts --locked` + 实机 | 残余手写命中全删 |

## 8. 测试矩阵（代表性用例）

- **M1**：`translate_winit_keyboard_matrix_matches_editor_baseline`、`translate_winit_touch_phase_maps_pointer_id`、`translate_winit_ime_preedit_carries_cursor_range`（platform_input 模块测试 + editor `src/tests/host/retained_window/native_input_translation.rs`）
- **M2**：`routing_capture_preempts_hit_path`、`popup_outside_click_dismisses_topmost_only`、`preview_tunnel_runs_before_bubble`、`focus_path_routes_unconsumed_keyboard`、`default_action_handles_window_transient`（input_manager 测试）
- **M3**：`reply_effect_capture_pointer_updates_state`、`reply_effect_open_popup_pushes_stack`、`rejected_effect_records_reason`、`tooltip_armed_after_hover_dwell_tick`、`tooltip_canceled_on_pointer_activity`
- **M4**：`two_touch_pointers_keep_independent_hover`、`touch_cancel_clears_pointer_entry`、`primary_touch_synthesizes_mouse_click`、`secondary_touch_does_not_move_mouse_cursor`
- **M5**：`shell_pointer_bridge_uses_route_intent_only`、`drawer_resize_capture_goes_through_reply`、editor 集成契约既有用例全量回归

测试落点惯例：runtime 侧用模块内 `#[cfg(test)]`（沿 surface/input 现状）；editor 侧用 `zircon_editor/src/tests/host/retained_window/`（已存在同名基线文件）。

## 9. 风险与对策

| 风险 | 对策 / 探测信号 |
|------|----------------|
| 输入是全编辑器交互的根，回归面极大 | M1.S1 先以现行为金标准固化矩阵测试；每切片跑 drawer/menu/drag 既有测试；M3.S3 设回归闸门 |
| editor 与 runtime preview 两宿主 winit 行为差异（modifiers、IME、DPI） | 单实现后以两宿主各自的集成测试覆盖；差异进 platform capability matrix 而非翻译分支 |
| 触摸无真实设备可验 | 以合成事件测试为主；实机触摸标注为后置验证项，不阻塞里程碑 |
| pointer bridge 收编与 08 区域切换顺序耦合（同区域双改冲突） | M5 按区域分批，与 08 同区域切换合并推进或明确先后；同一区域不同时开两个改动 |
| interface DTO 变更影响 ABI/序列化 | M1.S4 集中一次改动；过 interface serde 兼容测试（`cargo test -p zircon_runtime_interface --locked`） |

## 10. 里程碑级依赖表

| 里程碑 | 前置 | 被依赖 |
|--------|------|--------|
| M1 | 无 | 01 M2；03 M4（IME 链以 M1 事件为载体） |
| M2 | 01 M1 | 01 M3、01 M4 |
| M3 | 01 M2 | 04 M2（状态写入唯一生产者）、06 各组件行为、01 M5 |
| M4 | 01 M2 | 06（触摸组件行为，弱依赖） |
| M5 | 01 M3 | 08 M2/M3（区域切换以 bridge 收编为前提） |

## 11. 完成定义

- winit 语义解释只剩一份（runtime platform_input）；editor 只持 EventLoop 与喂 batch。
- 路由七阶段次序有单点实现与全矩阵测试；`UiDispatchEffect` 全 variant 有应用路径。
- 触摸双指测试全绿；tooltip 计时由 runtime tick 驱动。
- editor 11 个 pointer bridge 全部只消费 route_intent，无手写命中。
- 实机：shell tabs / drawer 开合 / 树选中 / 资产列表 / 多级菜单 / 拖拽改宽无交互回归。
- 验收命令组：`cargo test -p zircon_runtime --lib --locked`、`cargo test -p zircon_runtime_interface --locked`、`cargo test -p zircon_editor --lib --locked`、`cargo test -p zircon_editor --test integration_contracts --features integration-contracts --locked`。

## 12. 边界约束

- 事件路由不按控件名称特判；语义经 route id / component descriptor 表达。
- pointer-only 状态变化只 damage 旧/新 hit path（既有 damage 规则继续有效）。
- `interaction_gate`（disabled 判定）保持唯一守门员；input manager 不绕过它。
- 本计划不改 viewport 内 3D 交互（picking/gizmo 归 scene 路径），只保证 viewport 节点能拿到原始指针事件并声明 capture。
- interface 层 DTO 变更集中在 M1.S4 一次完成，禁止里程碑间反复改 ABI。
