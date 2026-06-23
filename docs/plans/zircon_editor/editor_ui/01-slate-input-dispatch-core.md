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
  - zircon_runtime/src/ui/platform_input/mod.rs
  - zircon_runtime/src/ui/platform_input/keyboard_map.rs
  - zircon_runtime/src/ui/platform_input/winit_translation.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/platform_input.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/events.rs
  - zircon_editor/src/tests/host/retained_window/platform_input_translation.rs
  - docs/zircon_editor/ui/retained_host/host_contract/platform_input.md
  - zircon_editor/src/ui/retained_host/shell_pointer/bridge.rs
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Application/SlateApplication.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Input/Reply.h
plan_sources:
  - .codex/plans/ZirconEngine 宿主编辑器 UI 基础能力计划.md
  - .codex/plans/Shared Slate-Style UI Layout, Render, And Hit Framework.md
  - .codex/plans/Drawer_Window_Menu Slate 化推进计划.md
  - .codex/plans/布局系统.md
status: in-progress
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

- `zircon_editor/src/ui/retained_host/host_contract/native_input_translation.rs` 与 `native_input_translation/**`（M1.S3，翻译收口后；已核验 `native_keyboard.rs` 是 retained popup keyboard command owner，不属于 winit 翻译删除项）
- `zircon_editor/src/ui/retained_host/host_contract/window/event_loop/input.rs` 中的指针按钮/滚轮 winit 翻译小函数（M1.S3）
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
| M1.S2 | `platform_input::winit_translation` 单实现落地，rhi/rhi_wgpu ui_surface 翻译段并入 | 新增 platform_input/；核验 rhi/ui_surface.rs、rhi_wgpu/ui_surface.rs 当前仅有 surface descriptor 逻辑，无 winit 输入翻译段可迁 | `cargo check -p zircon_runtime --lib --locked` | rhi 两处输入翻译段按当前代码核验为不存在；S3 删除 editor 本地翻译 |
| M1.S3 | editor/app 切换调用方；删除 editor 本地翻译 | event_loop/platform_input.rs；删 native_input_translation.rs 与 native_input_translation/**；保留经核验非翻译的 native_keyboard.rs | `cargo test -p zircon_editor --lib --locked` | 删 editor-local 翻译树 + event_loop/input.rs 指针/滚轮翻译段 |
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
| M5.S1 | route_intent 适配层 + 首迁 shell_pointer 家族（8 文件改读 route id，drag/resize 捕获走 reply） | 完成：新增 route_intent/；改 shell_pointer/* | `cargo test -p zircon_editor --lib shell_pointer --locked`（locked 被 lockfile 阻断；offline focused 通过） | shell_pointer 内命中码删除 |
| M5.S2 | 迁 document_tab / drawer_header / menu / activity_rail 四桥 | 完成：四桥改读 `route_intent`；删除四桥本地 target/conversion 命中码 | `cargo test -p zircon_editor --lib --locked`（locked 被 lockfile 阻断；offline focused 通过） | 四桥命中码删除 |
| M5.S3 | 迁 hierarchy / asset / detail / host_page / viewport_toolbar / welcome_recent + tab_drag / drawer_resize | 完成：hierarchy/detail/host_page/viewport_toolbar/welcome_recent 五桥改读 `route_intent`；tab_drag 复用 shell_pointer route intent；asset_pointer 无独立 runtime-surface 桥；drawer_resize 已在 M5.S1 shell route 中收编 | `cargo check -p zircon_editor --lib --offline`、`route_intent`、`retained_list_pointer`、`retained_detail_pointer`、`retained_host_page_pointer`、`retained_viewport_toolbar_pointer`、`retained_tab_drag` offline focused 均通过；`--locked` 仍被根 lockfile 漂移阻断 | 五桥本地 target/map 命中码删除 |
| M5.S4 | 清残：删除 surface_hit_test 被取代部分；实机回归 | `host_contract/surface_hit_test/` 仅保留 template-node owner；viewport toolbar 原生命中改走已投影 `UiSurfaceFrame` + `route_intent` | `cargo check -p zircon_editor --lib --offline`、`cargo test -p zircon_editor --test integration_contracts --features integration-contracts --offline` 27/27、`cargo check -p zircon_app --no-default-features --features target-editor-host --bin zircon_editor --offline` 均通过；`--locked` 仍被根 lockfile 漂移阻断；真实窗口交互未执行 | 残余 toolbar 手写 surface_hit_test 命中删除 |

## 状态与产出记录

| 日期 | 切片 | 状态 | 产出 | 验证 | 后续 |
|---|---|---|---|---|---|
| 2026-06-23 | 01.M1.S1 native input baseline and gap matrix | 完成（代码/文档记录；定向验证通过） | `zircon_editor/src/tests/host/retained_window/native_input_translation.rs` 新增 touch phase -> pointer contract 测试，锁定 `UiWindowPlatformInputEvent::touch(...)` 对 Started/Moved/Ended/Canceled 的 pointer kind、primary button、touch pointer id/source 映射；新增 editor event-loop gap matrix 测试，明确 live event pump 当前只接 KeyboardInput/MouseWheel/IME Commit，尚未消费 Touch、IME Preedit/Disabled/DeleteSurrounding；同步 `docs/zircon_editor/ui/retained_host/host_contract/native_input_translation.md`，记录 interface 已具备 touch/IME 载体但旧 editor-local 入口不继续堆新行为；验证时暴露并修复 Editor UI 10 的 `apply_presentation` 测试 helper 可见性漂移。 | `cargo fmt -p zircon_editor --check` 通过；scoped `git diff --check` 通过，仅既有 LF/CRLF warning；`cargo test -p zircon_editor --lib native_input_translation --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui01-native-input-0623 --message-format short --color never -- --test-threads=1 --nocapture` 通过（7 passed，2064 filtered out；既有 warning noise）。首次 test run 1200s 超时无诊断，复跑暴露并修复 `to_host_contract_host_scene_data` 测试 re-export 可见性后通过。 | 01.M1.S2 应新增 runtime-owned `platform_input::winit_translation` 单实现并迁入 editor/rhi 重复翻译；01.M1.S3 再删除 editor-local `native_input_translation` 和旧 native keyboard/pointer 翻译段。 |
| 2026-06-23 | 01.M1.S2 runtime platform_input winit single implementation | 完成（runtime 单实现落地；workspace `--locked` 被既有 Cargo.toml/Cargo.lock 漂移阻断） | 新增 `zircon_runtime/src/ui/platform_input/{mod,keyboard_map,winit_translation}.rs`，并在 `zircon_runtime/src/ui/mod.rs` 以 `platform-winit` feature 暴露；`translate_winit_window_event(...)` 覆盖 close/resize/move/cursor/keyboard/IME/wheel/redraw/focus/occluded/touch pointer 模型，`translate_winit_modifiers(...)` 单独提供 modifiers 映射；键盘 legacy/name/scan/state helper 收归 runtime。核验 `zircon_runtime/src/rhi/ui_surface.rs` 与 `zircon_runtime/src/rhi_wgpu/ui_surface.rs` 当前没有 winit 输入翻译段，只有 surface descriptor 转换，因此本切片无 rhi 删除项；editor-local 翻译保留到 01.M1.S3 调用方切换后硬删。同步新增 `docs/zircon_runtime/ui/platform_input.md` 并更新 editor native-input 边界文档。 | `rustfmt --edition 2021 --check zircon_runtime/src/ui/platform_input/mod.rs zircon_runtime/src/ui/platform_input/keyboard_map.rs zircon_runtime/src/ui/platform_input/winit_translation.rs` 通过；外部临时验证项目 `cargo check --manifest-path E:\cargo-targets\zircon-runtime-platform-input-scratch-0623\Cargo.toml --offline --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-platform-input-scratch-target-0623 --message-format short --color never` 通过；同临时项目 `cargo test ... -- --test-threads=1` 通过 2/2；scoped `git diff --check` 通过，仅既有 LF/CRLF warning；workspace `cargo check -p zircon_runtime --lib --locked` 因既有 Cargo.toml/Cargo.lock 需要更新 lockfile 被 Cargo 拒绝，未修改 lockfile；`cargo fmt -p zircon_runtime --check` 仍受既有 `core/runtime/events/prune.rs`、`graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/output_access.rs`、`scene/level_system.rs` 格式漂移阻断。 | 01.M1.S3：editor/app 调用方切到 runtime `platform_input::translate_winit_window_event`，然后删除 editor-local `host_contract/native_input_translation*` 与旧 native keyboard/pointer 翻译段。 |
| 2026-06-23 | 01.M1.S3 editor retained host platform_input cutover | 实现完成（代码/文档记录；locked Cargo 被既有锁文件漂移阻断，离线补充验证超时） | 新增 `zircon_editor/src/ui/retained_host/host_contract/window/event_loop/platform_input.rs`，live `WindowEvent` 输入先调用 `zircon_runtime::ui::platform_input::translate_winit_window_event`/`translate_winit_modifiers`，再把 runtime pump event 交给 retained keyboard/text/pointer 行为；`dispatch_keyboard_event` 改收 `UiKeyboardInputEvent`，未消费键盘不再调用 editor-local 翻译；指针移动/按钮/滚轮从 runtime `UiWindowInputPumpEvent` 读取，event_loop/input.rs 删除 editor-local button/state/wheel 翻译小函数；runtime public keyboard translation 修复 `is_synthetic` 标记保留。硬删 `host_contract/native_input_translation.rs`、`native_input_translation/**` 与旧 `src/tests/host/retained_window/native_input_translation.rs`，新增 `platform_input_translation.rs` 和边界测试断言旧路径不回流；新增 `docs/zircon_editor/ui/retained_host/host_contract/platform_input.md` 并更新 runtime platform_input 文档。核验计划中 `native_keyboard.rs` 实为 retained workbench popup keyboard command owner，不是 winit 翻译，未删除。 | `cargo fmt -p zircon_editor --check` 通过；touched-file `rustfmt --edition 2021 --check` 通过；scoped trailing-whitespace scan 通过；scoped `git diff --check` 通过（仅既有 LF/CRLF warning）。`cargo test -p zircon_editor --lib platform_input_translation --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui01-m1s3-0623 --message-format short --color never -- --test-threads=1 --nocapture` 在编译前被既有 `Cargo.toml`/`Cargo.lock` 漂移拒绝；为取 Rust 诊断启动的离线补充验证超时 600s 无诊断，已停止残留 cargo/rustc 并恢复 `Cargo.lock` 到验证前 hash。 | 01.M1.S4：关闭 retained host 仍未应用的 IME preedit/cancel 与 touch use-site/interface 缺口；随后进入 M2 input_manager 批入口收编。 |
| 2026-06-23 | 01.M1.S4 interface touch/IME variant closure | 完成（代码/文档记录；locked Cargo 被既有锁文件漂移阻断，离线接口验证通过） | 确认 touch phase carrier 已在 `UiWindowTouchPhase`/`UiWindowPlatformInputEvent::touch_*` 中具备；补齐 IME delete-surrounding shared carrier：新增 `UiImeInputEventKind::DeleteSurrounding`、`UiImeDeleteSurrounding`、`UiImeInputEvent.delete_surrounding` 与 `UiWindowPlatformInputEvent::ime_delete_surrounding(...)`，`runtime_event_adapter` 将 ABI `ZR_RUNTIME_IME_STATE_DELETE_SURROUNDING_V1` 映射进 window pump，runtime winit `Ime::DeleteSurrounding` 不再丢弃；runtime editable text 目前只记录 owner route 和诊断，不执行文本删除策略。同步更新 runtime/interface/editor platform-input 测试和 `docs/zircon_runtime_interface/ui/window.md`、`docs/zircon_runtime/ui/platform_input.md`、`docs/zircon_runtime/ui/surface/input.md`、`docs/zircon_editor/ui/retained_host/host_contract/platform_input.md`。 | `rustfmt --edition 2021 --check` 覆盖本切片触及 Rust 文件通过；trailing-whitespace scan 通过；scoped `git diff --check` 通过（仅既有 LF/CRLF warning）；`cargo test -p zircon_runtime_interface --locked -- --nocapture` 与 `cargo test -p zircon_runtime --locked winit_translation -- --nocapture` 均在编译前被既有 Cargo.toml/Cargo.lock 漂移拒绝，`Cargo.lock` hash 保持 `BDB375A62160443186167DBFACDFED661982C6751374E43727CAE3A28A066707`；补充离线验证 `cargo test -p zircon_runtime_interface --offline window_input_contracts -- --nocapture` 通过 5/5，`cargo test -p zircon_runtime_interface --offline window_runtime_event_adapter_contracts -- --nocapture` 通过 7/7，`cargo test -p zircon_runtime_interface --offline ui_input -- --nocapture` 通过 4/4。 | 01.M2.S1：新增 input_manager 骨架并把 window_pump 批入口收编到 manager；后续文本策略切片再实现 delete-surrounding 对 retained surrounding text 的实际删除。 |
| 2026-06-23 | 01.M2.S1 input_manager window-pump ownership | 完成（代码/文档记录；locked Cargo 被既有锁文件漂移阻断，离线补充验证超时） | `UiInputManager` 作为 pointer/navigation dispatcher 单 owner，统一承接 normalized input 与 window-pump batch；`UiSurface::dispatch_window_input_pump_event(...)` / `dispatch_window_input_pump_batch(...)` 现在必须传入 `&mut UiInputManager`，旧的 `UiPointerDispatcher`/`UiNavigationDispatcher` 参数签名不再存在；`surface/input/window_pump.rs` 仅保留 `dispatch_window_event(...)` 内部 leaf，`UiInputManager::dispatch_window_input_pump_event(...)` 对 `Input(...)` 复用 `dispatch_input_event(...)` 以共享 timer arming，对 `Window(...)` 委托 window leaf；删除/核验 duplicate test-only `RuntimeUiInputRouter` 不再挂载。同步更新 runtime UI manager、surface input、asset surface index、runtime graphics integration、surface/assets rules 等文档的 `ui/runtime_ui/*` 旧路径，结构约定守卫改查新的 `UiInputManager` 导入。 | touched-file `rustfmt --edition 2021 --check` 通过；旧 window-pump 签名、`*_with_manager`、`RuntimeUiInputRouter` 源码扫描无残留；stale `ui/runtime_ui/input_router.rs`/production re-export 文档扫描无残留；trailing-whitespace scan 通过；scoped `git diff --check` 通过（仅既有 LF/CRLF warning）。`cargo test -p zircon_runtime --lib window_pump --locked --jobs 1 --message-format short --color never -- --test-threads=1 --nocapture` 在编译前被既有 `Cargo.toml`/`Cargo.lock` 漂移拒绝；补充 `cargo test -p zircon_runtime --lib window_pump --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui01-m2s1-offline-0623 --message-format short --color never -- --test-threads=1 --nocapture` 904s 超时无诊断，未留下本轮 cargo/rustc 残留，`Cargo.lock` 恢复到 `BDB375A62160443186167DBFACDFED661982C6751374E43727CAE3A28A066707`。 | 01.M2.S2：把七阶段路由次序收口到 `routing.rs` 单一权威，并删除 `dispatch.rs` 内散落次序逻辑。 |
| 2026-06-23 | 01.M2.S2 route-order single authority | 完成（代码/文档记录；locked Cargo 被既有锁文件漂移阻断） | `zircon_runtime/src/ui/dispatch/input_manager/routing.rs` 现在同时拥有 `UI_INPUT_ROUTE_ORDER`、`UiInputRouteStage`、`route_policy_uses_stage(...)`、`route_stage_name(...)`、`route_stage_names_for_policy(...)`；`surface/input/route_authority.rs` 删除本地 policy-stage 匹配表，仅通过 manager routing authority 写入 `route_authority=...;stages=...` 诊断；`surface/input/route_policy.rs` 在 direct/capture trace target 推导中复用同一 stage predicate，避免 surface helper 再维护阶段语义。`runtime_input_manager` 增加 policy-stage 顺序单元测试，runtime absorption 守卫改查 `input_manager/routing.rs` 才是阶段枚举/映射来源；`docs/zircon_runtime/ui/surface/input.md` 同步记录 M2.S2 边界。 | touched-file `rustfmt --edition 2021 --check` 通过；surface input hard-cutover scan 确认不再定义 `UI_INPUT_ROUTE_ORDER`、`UiInputRouteStage::*` 或本地 `route_policy_uses_stage`/`route_stage_name`；trailing-whitespace scan 通过；scoped `git diff --check` 通过（仅既有 LF/CRLF warning）；`cargo check -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never` 在编译前被既有 `Cargo.toml`/`Cargo.lock` 漂移拒绝，`Cargo.lock` hash 保持 `BDB375A62160443186167DBFACDFED661982C6751374E43727CAE3A28A066707`。 | 01.M2.S3：补 capture 抢占、popup 外点关闭、preview 先于 bubble、focus-path 键盘、default-action 兜底的路由矩阵测试。 |
| 2026-06-23 | 01.M2.S3 input_manager route matrix | 完成（代码/文档记录；locked Cargo 被既有锁文件漂移阻断，offline 编译验证超时） | `zircon_runtime/src/ui/tests/runtime_input_manager.rs` 新增 manager-owned 路由矩阵：`input_manager_route_matrix_capture_preempts_hit_target` 覆盖 pointer capture 抢占后续 hit target；`input_manager_route_matrix_popup_outside_closes_top_only` 覆盖外点 release 只关闭最上层 popup；`input_manager_route_matrix_preview_stops_before_bubble` 覆盖 PreviewTunnel handler 在 target/bubble 前停止；`input_manager_route_matrix_keyboard_uses_focus_path` 覆盖 focused keyboard `FocusPath`；`input_manager_route_matrix_popup_open_uses_default_action` 覆盖 popup open 的 `DefaultAction`。为解除本切片验证前置阻塞，最小修复 `zircon_runtime/src/ui/tests/v2_asset/demo_and_builder.rs` 的 editor asset `include_str!` 上跳层级、`zircon_runtime/src/tests/runtime_diagnostics/mod.rs` 的 `snapshot.render.stats.as_ref()` 借用，并补全当前未跟踪 `zircon_runtime/src/tests/runtime_diagnostics/graph_execution.rs` 的闭合调用。`docs/zircon_runtime/ui/surface/input.md` 同步记录 M2.S3 落点。 | `rustfmt --edition 2021 --check zircon_runtime/src/ui/tests/runtime_input_manager.rs` 通过；`git diff --check -- zircon_runtime/src/ui/tests/runtime_input_manager.rs` 通过（仅既有 LF/CRLF warning）；旧 manager/window-pump helper 残留扫描无匹配；`cargo test -p zircon_runtime --lib input_manager --locked --jobs 1 --message-format short --color never -- --test-threads=1 --nocapture` 在编译前被既有 `Cargo.toml`/`Cargo.lock` 漂移拒绝；补充 offline 验证第一次暴露并修复 `graph_execution.rs` 闭合错误，第二次暴露并修复 v2 asset include 路径和 runtime diagnostics partial-move；第三次 `cargo test -p zircon_runtime --lib input_manager --offline --jobs 1 --message-format short --color never -- --test-threads=1 --nocapture` 304s 超时无 Rust 诊断，已停止本轮 runtime cargo/rustc 残留，仅保留其它会话插件测试进程，`Cargo.lock` 恢复到 `BDB375A62160443186167DBFACDFED661982C6751374E43727CAE3A28A066707`。 | 01.M3.S1：实现 `UiDispatchEffect` 应用矩阵的 input-manager/route-result 驱动测试，继续避免在 surface helper 中新增第二套路由顺序逻辑。 |
| 2026-06-23 | 01.M3.S1 dispatch effect matrix | 完成（代码/文档记录；locked Cargo 被既有锁文件漂移阻断，offline 过滤验证通过） | 新增 `zircon_runtime/src/ui/tests/runtime_dispatch_effect_matrix.rs` 并注册到 runtime UI 测试集合；通过 `UiSurface::apply_dispatch_reply(...)` 覆盖 `UiDispatchEffect` 全 16 个 variant：focus/pointer/high-precision/pointer-lock/dirty-redraw、drag/drop、navigation、popup/tooltip/transient-dismiss、input-method、clipboard、component-event 的成功应用，以及 SetFocus/ClearFocus/CapturePointer/ReleasePointerCapture/LockPointer/UnlockPointer/UseHighPrecisionPointer/DragDrop/RequestNavigation/Popup/Tooltip/RequestInputMethod/RequestClipboard/DirtyRedraw/EmitComponentEvent 的拒绝矩阵。拒绝断言保留原 effect、effect_index 与非空 reason，且 rejected path 不产生 host request 或 component event。`docs/zircon_runtime/ui/surface/input.md` 同步记录 M3.S1 落点。 | `rustfmt --edition 2021 --check zircon_runtime/src/ui/tests/runtime_dispatch_effect_matrix.rs zircon_runtime/src/ui/tests/mod.rs` 通过；`git diff --check -- zircon_runtime/src/ui/tests/runtime_dispatch_effect_matrix.rs zircon_runtime/src/ui/tests/mod.rs` 通过（仅既有 LF/CRLF warning）；`cargo test -p zircon_runtime --lib dispatch --locked --jobs 1 --message-format short --color never -- --test-threads=1 --nocapture` 在编译前被既有 `Cargo.toml`/`Cargo.lock` 漂移拒绝；补充 `cargo test -p zircon_runtime --lib dispatch_effect_matrix --offline --jobs 1 --message-format short --color never -- --test-threads=1 --nocapture` 通过 3/3（既有 warning noise），`Cargo.lock` 恢复到 `BDB375A62160443186167DBFACDFED661982C6751374E43727CAE3A28A066707`。 | 01.M3.S2：把 tooltip/double-click 等计时 ownership 移交到 `UiInputManager` timers，并删除旧计时态。 |
| 2026-06-23 | 01.M3.S2 timer ownership transfer | 完成（代码/文档记录；locked Cargo 被既有锁文件漂移阻断，offline test harness 被当前工作树其它测试编译错误阻断） | `UiInputTimerState` 现在接管 tooltip deadline、double-click candidate、double-click timeout 清理；`UiInputManager::tick(...)` 在帧首清理过期双击窗口并注入 synthetic `UiTooltipTimerInputEvent::Elapsed`；hover delivered component event 会通过 `UiSurface::tooltip_timer_for_component_node(...)` 读取 widget/attribute tooltip 合约并 arm retained tooltip candidate，后续 pointer/keyboard/text/IME/navigation 等输入活动会取消 pending tooltip；primary release 前由 manager/timers 根据上一 release candidate 修正 pointer `click_count`，surface 仍复用既有 double-click default action。`surface/input/tooltip_timer.rs` 不再拥有计时态，仅保留 `TooltipTimer` event reducer 和 stale-retained-state guard。新增/扩展 `runtime_input_manager` 与 input-manager 模块内测试：`tooltip_hover_arms_and_clears_manager_timer_candidate`、`tooltip_hover_timer_tick_dispatches_elapsed_default_action`、`tooltip_candidate_clears_on_following_input_activity`、`input_manager_double_click_count_is_owned_by_timer_state`。`docs/zircon_runtime/ui/surface/input.md` 同步记录 M3.S2 边界。 | `rustfmt --edition 2021 --check zircon_runtime/src/ui/dispatch/input_manager/manager.rs zircon_runtime/src/ui/dispatch/input_manager/timers.rs zircon_runtime/src/ui/surface/surface/default_interactions.rs zircon_runtime/src/ui/tests/runtime_input_manager.rs` 通过；scoped `git diff --check` 通过（仅 LF/CRLF warning）；trailing-whitespace scan 通过；`cargo check -p zircon_runtime --lib --offline --jobs 1 --message-format short --color never` 通过（既有 warning noise），并恢复 `Cargo.lock` 到 `BDB375A62160443186167DBFACDFED661982C6751374E43727CAE3A28A066707`。计划命令 `cargo test -p zircon_runtime --lib tooltip --locked --jobs 1 --message-format short --color never -- --test-threads=1 --nocapture` 仍在编译前被 lockfile 漂移拒绝；补充 offline `cargo test -p zircon_runtime --lib tooltip --offline --jobs 1 --message-format short --color never -- --test-threads=1 --nocapture` 进入 test harness 编译后被当前工作树其它测试错误阻断：`runtime_absorption/structure_convention/test_file_budget.rs` 缺少 `rhi_command_list`、`rhi_device_contract`、`runtime_diagnostics` 模块文件，且 native plugin live-host 测试缺少 `NativePluginBehavior::{registration_manifest, registration_manifest_schema}` 初始化字段；未出现本切片 Rust 诊断，`Cargo.lock` 已恢复。 | 01.M3.S3：在当前外部 test-harness 阻断解除后，跑 drawer/menu/drag 回归闸门；继续避免把旧 tooltip 计时态留在 surface leaf。 |
| 2026-06-23 | 01.M3.S3 drawer/menu/drag regression gate | 阻断（无代码变更；等待当前工作树 lockfile/test-harness 外部错误解除） | 本切片按计划只要求既有 drawer/menu/drag 回归闸门，不新增代码。由于 M3.S2 的 offline tooltip test 已确认当前 test harness 在进入目标测试前被其它工作树错误阻断，本轮未启动全量 offline 回归；只执行计划列出的 locked 闸门命令确认状态。 | `cargo test -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never -- --test-threads=1` 与 `cargo test -p zircon_editor --lib --locked --jobs 1 --message-format short --color never -- --test-threads=1` 均在编译前被 `Cargo.lock` 需要更新且 `--locked` 禁止更新拒绝；editor 命令还短暂等待 package-cache file lock。`Cargo.lock` hash 保持 `BDB375A62160443186167DBFACDFED661982C6751374E43727CAE3A28A066707`。 | 01.M4.S1：继续实现 `UiActivePointerTable` per-pointer hover/capture 状态；M3.S3 回归闸门待外部锁文件与 test harness 错误修复后补跑。 |
| 2026-06-23 | 01.M4.S1 active pointer table state | 完成（代码完成；Cargo 行为闸门被当前工作树 lockfile/test-harness 状态限制） | `UiActivePointerTable` entry 保留 per-pointer last point、hover route、pressed button mask、pressed target、capture target、source 与 primary flag，`UiInputManager::dispatch_input_event(...)` 按 normalized pointer id 同步真实 dispatch 结果并在 `Pointer(Cancel)` 移除 entry。surface input 的旧单指针 active-capture 字段已硬删；capture state 只由 `UiPointerId -> UiNodeId` map 表示，text/drag/drop reducers 统一写入 `set_pointer_capture_for_id(...)`，high-precision pointer 只接受 `has_pointer_capture_for_owner(...)` 的 indexed capture 事实。现有 capture/text/popup/touch 测试改用 per-pointer helper/assertion，pointer dispatch 修正为仅当 incoming pointer 没有自己的 capture 且其它 pointer 仍有 capture 时旁路旧 captor，避免多指针 capture 表第一项误判。 | touched Rust `rustfmt --edition 2021 --check` 通过；source scan 确认 UI 源码无旧单指针字段残留、runtime 源码无旧 unindexed fallback API 名残留；计划命令 `cargo test -p zircon_runtime --lib pointer_table --locked --jobs 1 --message-format short --color never -- --test-threads=1 --nocapture` 在编译前被当前 `Cargo.lock` 漂移拒绝；offline pointer-table 测试曾 600s 超时无 Rust 诊断；后续 `cargo check -p zircon_runtime --lib --offline --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-m4s1-check-0623 --message-format short --color never` 在 M4.S2 复跑通过（既有 warning noise）。每次 Cargo 后 `Cargo.lock` 均恢复为 `BDB375A62160443186167DBFACDFED661982C6751374E43727CAE3A28A066707`。 | 01.M4.S2：实现 primary touch 鼠标语义合成 + cancel 清理；M3.S3 回归闸门待外部 lockfile/test-harness 修复后补跑。 |
| 2026-06-23 | 01.M4.S2 primary touch mouse semantics | 完成（代码完成；locked 测试被 lockfile 阻断，offline lib-test 被外部缺模块阻断） | `UiInputManager` 在 dispatch 前用 `UiActivePointerInputEvent` 快照当前 pointer id/source/kind/point/button，并通过 `UiActivePointerTable` 判定 touch primary；secondary touch 发送给 surface 前移除 primary button，因此不产生 mouse-style press/click activation，同时 active pointer table 仍用原始 touch button 记录该 pointer 的 pressed mask/target。`Pointer(Cancel)` 继续移除对应 active pointer entry，并在 surface release path 清理 indexed capture。新增 `runtime_input_manager` 测试覆盖 primary touch click、secondary touch table press without activation、touch cancel entry/capture cleanup。 | `rustfmt --edition 2021 --check zircon_runtime/src/ui/dispatch/input_manager/manager.rs zircon_runtime/src/ui/tests/runtime_input_manager.rs` 通过；`cargo check -p zircon_runtime --lib --offline --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-m4s1-check-0623 --message-format short --color never` 通过（既有 warning noise），并恢复 `Cargo.lock` 到 `BDB375A62160443186167DBFACDFED661982C6751374E43727CAE3A28A066707`。计划命令 `cargo test -p zircon_runtime --lib touch --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-m4s2-touch-0623 --message-format short --color never -- --test-threads=1 --nocapture` 在编译前被 lockfile 漂移拒绝；offline rerun 首次暴露并修复本轮 `UiValue` import，复跑进入 lib-test compilation 后被当前工作树外部 `runtime_absorption/structure_convention/test_file_budget.rs` 缺少 `asset_gltf_importer`、`asset_importer`、`asset_scene`、`asset_tests`、`rhi_command_list`、`rhi_device_contract`、`runtime_diagnostics` 模块文件阻断；未出现 M4.S2 Rust 诊断，锁文件已恢复。 | 01.M4.S3：补多指针矩阵测试：两指独立 hover/press、cancel 清理、capture 隔离。 |
| 2026-06-23 | 01.M4.S3 multi-pointer matrix | 完成（代码完成；offline lib-test 被外部 test-harness/asset fixture 阻断） | `runtime_input_manager` 新增多指针矩阵：`input_manager_two_touch_pointers_keep_independent_hover_and_press` 覆盖两根 touch contact 分别保留 hover path、pressed target、button mask、primary flag 和 last point，取消第一根 pointer 只移除第一根 entry，第二根 press 仍保留；`input_manager_multi_pointer_capture_isolation_survives_cancel` 覆盖两个 indexed capture 并存、move 按 pointer id 路由到各自 captor、cancel 第一根只释放第一根 capture 且保留第二根 capture/focus snapshot。 | `rustfmt --edition 2021 --check zircon_runtime/src/ui/dispatch/input_manager/manager.rs zircon_runtime/src/ui/tests/runtime_input_manager.rs` 通过；`cargo check -p zircon_runtime --lib --offline --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-m4s1-check-0623 --message-format short --color never` 通过（既有 warning noise），并恢复 `Cargo.lock`。共享计划命令 `cargo test -p zircon_runtime --lib touch --locked ...` 仍被 lockfile 漂移拒绝；offline `touch` rerun 进入 lib-test compilation 后被当前工作树外部错误阻断：`runtime_absorption/structure_convention/test_file_budget.rs` 缺多个 child module，且 `asset/tests/assets/gltf_primitive_fixtures.rs` 的 GLTF fixture helper re-export 可见性触发 E0364/E0603；未出现 M4.S3 Rust 诊断，`Cargo.lock` hash 保持 `BDB375A62160443186167DBFACDFED661982C6751374E43727CAE3A28A066707`。 | 01.M5.S1：新增 route_intent 适配层并迁 shell_pointer 家族。 |
| 2026-06-23 | 01.M5.S1 route_intent shell_pointer cutover | 完成（代码完成；locked 计划命令被 lockfile 阻断，offline focused 测试通过） | 新增 `zircon_editor/src/ui/retained_host/route_intent/{mod.rs,map.rs}`，提供 `UiRouteId`/`UiNodeId` 到 `EditorRouteIntent` 的薄适配；`shell_pointer` drag/resize surface 构建期为每个交互节点注册稳定 synthetic route id，`HostShellPointerBridge` 改走 `UiSurface::dispatch_input_event(...)` 并从 `UiDispatchReply` 的 capture/release effect、handler 或 route target 经 `EditorRouteIntentMap` 解析 `HostShellPointerRoute`；删除 `drag_route_from_node(...)` 和 `resize_group_from_dispatch(...)`，source contract 新增 `shell_pointer_bridge_uses_route_intent_only` 与 `drawer_resize_capture_goes_through_reply`。 | touched Rust `rustfmt --edition 2021 --check` 通过；shell_pointer 源码扫描确认无 `drag_route_from_node`、`resize_group_from_dispatch`、`dispatch_pointer_event(` 残留；scoped `git diff --check` 通过（仅既有 LF/CRLF warning）；`cargo check -p zircon_editor --lib --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-m5s1-check-0623 --message-format short --color never` 通过（既有 warning noise）；计划命令 `cargo test -p zircon_editor --lib shell_pointer --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-m5s1-shell-pointer-0623 --message-format short --color never -- --test-threads=1 --nocapture` 在编译前被 lockfile 漂移拒绝；补充 `cargo test -p zircon_editor --lib shell_pointer --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-m5s1-shell-pointer-offline-0623 --message-format short --color never -- --test-threads=1 --nocapture` 通过 13/13，`Cargo.lock` 恢复并保持 `BDB375A62160443186167DBFACDFED661982C6751374E43727CAE3A28A066707`。 | 01.M5.S2：迁 document_tab / drawer_header / menu / activity_rail 四桥到 route_intent。 |
| 2026-06-23 | 01.M5.S2 document/drawer/menu/activity route_intent cutover | 完成（代码完成；locked 计划命令被 lockfile 阻断，offline focused 测试通过；`workbench_projection_cutover` 仅剩非本切片 dock-header 资产契约失败） | `EditorRouteIntent` 扩展为 `DocumentTab`、`DrawerHeader`、`Menu`、`ActivityRail` 四类语义 route，并由 `EditorRouteIntentMap` 统一从 dispatch result 解析 route id；document tab、drawer header、menu、activity rail 四桥均在 surface 构建期绑定稳定 route id，bridge 不再读取 `handled_by`/`route.target` 或维护本地 target 表；删除四桥旧 `host_*_pointer_target` 与 route conversion 命中模块，menu 保留私有 `HostMenuPointerRouteIntent` 以承载 submenu item path。 | `cargo check -p zircon_editor --lib --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-m5s2-check-0623 --message-format short --color never` 通过（既有 warning noise）；计划命令 `cargo test -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-m5s2-locked-0623 --message-format short --color never -- --test-threads=1 --nocapture` 在编译前被 lockfile 漂移拒绝；补充 offline focused：`route_intent_only` 5/5、`retained_document_tab_pointer` 7/7、`retained_drawer_header_pointer` 9/9、`retained_menu_pointer` 24 passed / 4 ignored、`retained_activity_rail_pointer` 8/8 均通过；`workbench_projection_cutover` 9/10，通过 source-hit-test 更新，仅 `workbench_main_interface_entries_are_template_backed_and_reflected` 因生产代码未引用 `/assets/ui/editor/workbench_dock_header.v2.ui.toml` 失败，属于 08/10 模板资产契约而非本切片 route_intent 迁移；每次 Cargo 后 `Cargo.lock` 恢复并保持 `BDB375A62160443186167DBFACDFED661982C6751374E43727CAE3A28A066707`。 | 01.M5.S3：迁 hierarchy / asset / detail / host_page / viewport_toolbar / welcome_recent + tab_drag / drawer_resize 到 route_intent。 |
| 2026-06-23 | 01.M5.S4 surface_hit_test viewport-toolbar cleanup | 完成（代码清残、focused 行为、集成契约与 editor-host 编译烟测完成；真实窗口交互未执行） | 删除 `host_contract/surface_hit_test/viewport_toolbar.rs` 与对应测试 owner，`surface_hit_test/mod.rs` 不再导出 viewport toolbar 命中；native pane toolbar routing 改用 projected `pane.viewport.toolbar_surface_frame` + runtime `hit_test_surface_frame(...)` 只取得真实 `control_id` 供 damage 使用，host callback 对外只传 `surface_key + point + size`；`ViewportToolbarPointerBridge` 新增 surface-frame sync，把 projected controls 转成 route-intent-backed runtime surface，再按点击点分发；profiling route check 同步改走 shared surface-frame hit test。为解除 package check 下层阻塞，已同步收窄修复 runtime `service_lists` registration 可见性、`scene/world/render.rs` typed `BTreeMap` 推断，以及 app viewport-toolbar click import/dock toolbar frame 类型推断。集成契约同步到当前 componentized `.zui`/folder-backed owner 结构，并为 `integration-contracts` 增加只在契约功能门下公开的 workbench-geometry drag-target resolver。 | `rustfmt --edition 2021 --check` 覆盖 M5.S4 touched retained-host source/tests 与本轮 runtime/editor 编译修复文件通过；source scan 确认 `ViewportToolbarPointerHit`、`hit_test_viewport_toolbar`、`surface_hit_test::hit_test_viewport_toolbar`、`active_controls`、`ActiveViewportToolbarControl`、旧 `PanePointerTarget::ViewportToolbar(...)` 与旧 viewport-toolbar `Callback8` 回调签名无源码残留；`surface_hit_test/` 目录仅剩 template-node/surface-frame owners。`cargo check -p zircon_runtime --lib --no-default-features --features core-min --offline --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-service-lists-0623 --message-format short --color never` 通过；`cargo check -p zircon_editor --lib --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-m5s3-check-0623 --message-format short --color never` 通过（既有 warning noise）；M5.S3 focused 补跑：`retained_host_page_pointer` 8/8、`retained_viewport_toolbar_pointer` 7/7、`retained_tab_drag` 37/37 通过；`cargo test -p zircon_editor --test integration_contracts --features integration-contracts --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-m5s4-integration-0623 --message-format short --color never -- --test-threads=1` 27/27 通过；`cargo check -p zircon_app --no-default-features --features target-editor-host --bin zircon_editor --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-realhost-0623 --message-format short --color never` 通过。计划 locked 命令仍在编译前被根 `Cargo.lock`/`Cargo.toml` 漂移拒绝；每次 Cargo 后根 `Cargo.lock` 均恢复为 `BDB375A62160443186167DBFACDFED661982C6751374E43727CAE3A28A066707`。 | 后续在可交互窗口环境中执行真实 editor host 点击/拖拽回归；若需要关闭剩余 `--locked` 阻断，先统一刷新根 `Cargo.lock`。 |

## 8. 测试矩阵（代表性用例）

- **M1**：`translate_winit_keyboard_matrix_matches_editor_baseline`、`runtime_touch_pointer_events_map_pointer_id_source_kind_and_button`、`runtime_ime_translation_maps_preedit_commit_and_disable`（runtime `platform_input` 模块测试 + editor `src/tests/host/retained_window/platform_input_translation.rs`）
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

## 13. 参考实现对照（dev/ 源码锚点）

实现切片前先读对应锚点，不确定的行为语义以参考实现为准（在 PR 说明中注明出处）；禁止凭印象实现、禁止引用未核实路径。

| 设计点 | 主参考 | 次参考 | 参考什么 |
|--------|--------|--------|---------|
| 统一入口与路由次序 | `dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Application/SlateApplication.h` | `dev/UnrealEngine/Engine/Source/Runtime/ApplicationCore/Public/GenericPlatform/GenericApplicationMessageHandler.h` | RoutePointerDownEvent 等的 capture/preview(tunnel)/bubble 路由编排、合成 click/double-click 规则 |
| Reply 副作用模型 | `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Input/Reply.h` | — | CaptureMouse/SetUserFocus/BeginDragDrop/ReleaseMouseCapture 的声明式副作用集合与互斥规则 |
| 命中/焦点路径 | `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Layout/WidgetPath.h`（+ .inl） | `dev/slint/internal/core/item_focus.rs` | widget path 的构建/失效与按路径派发；Slint 的焦点链遍历 |
| winit 事件翻译单实现 | `dev/bevy/crates/bevy_winit` | `dev/bevy/crates/bevy_input` | winit → 引擎事件的归一化分层（converters）、设备 id/触摸 phase 处理 |
| Tab/方向键导航 | `dev/bevy/crates/bevy_input_focus` | `dev/Fyrox/fyrox-ui/src/navigation.rs` | 焦点遍历策略与可聚焦判定 |
| 消息式 UI 事件对照 | `dev/Fyrox/fyrox-ui/src/message.rs`、`input.rs`、`key.rs` | — | Fyrox 的 routed message（Direction::FromWidget/ToWidget）与本计划 preview/bubble 的对应关系 |
| popup 外点关闭/菜单 | `dev/godot/scene/gui/popup.cpp`、`popup_menu.cpp` | `dev/Fyrox/fyrox-ui/src/dropdown_menu.rs` | 瞬态层关闭判定、菜单键盘导航与 hover 展开 |
| 拖拽阈值/延迟拖拽 | `dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/DelayedDrag.h` | `dev/godot/scene/gui/control.cpp` | 按下后位移阈值才进入 drag 的标准行为 |
