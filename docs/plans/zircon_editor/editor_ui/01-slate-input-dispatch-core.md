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

1. **winit 翻译双实现**：editor 侧 `host_contract/native_input_translation.rs`（190 行，直接用 winit 类型）+ `native_keyboard.rs` + `native_pointer/` 一份；runtime 预览侧 `zircon_runtime/src/rhi/ui_surface.rs`、`rhi_wgpu/ui_surface.rs` 另一份。editor 直接依赖 `zircon_runtime`（zircon_editor/Cargo.toml:24，rlib），单实现可落 runtime owner 模块。（2026-07-02 评审收口，勘误注）按 M1.S2 核验结论修订现状：`rhi/ui_surface.rs`、`rhi_wgpu/ui_surface.rs` 当前仅有 surface descriptor 转换逻辑，**无 winit 输入翻译段可迁**；「另一份」表述按当前代码不成立，重复翻译实际只存在于 editor 侧一份。收口方向不变（runtime `ui/platform_input` 单实现），本条原文保留作历史对照。
2. **无统一 manager 门面**：批入口以 `(surface, pointer_dispatcher, navigation_dispatcher)` 参数对穿透各层（window_pump.rs:19–56）；tooltip/双击计时、多指针实例表没有统一 owner。
3. **触摸不成体系**：`UiWindowTouchPhase` 已进平台事件，但 per-pointer-id 活动指针表、primary touch 鼠标语义合成、cancel 清理未实现（`state/` 只有单指针痕迹）。
4. **路由次序未单点固化**：策略枚举齐全，但 capture→popup→preview→direct→bubble→focus-path 的全链次序与外点关闭判定散在 dispatch.rs 与 route_policy.rs，缺一处权威实现与矩阵测试。
5. **editor 手写命中**：11 个 pointer bridge 家族 + tab_drag + drawer_resize 自带命中/hover/press 状态机：`activity_rail_pointer`、`asset_pointer`、`detail_pointer`、`document_tab_pointer`、`drawer_header_pointer`、`hierarchy_pointer`、`host_page_pointer`、`menu_pointer`、`shell_pointer/`（bridge/common/drag_frames/drag_surface/effects/node_ids/resize_surface/route 共 8 文件）、`viewport_toolbar_pointer`、`welcome_recent_pointer`（均在 `zircon_editor/src/ui/retained_host/`）。

## 3. 设计

### 3.1 事件归一化层（对应 GenericApplicationMessageHandler）

- `UiWindowInputPumpBatch` 维持唯一平台事件载体地位；按 M1 盘点结论补缺 variant（重点核对触摸与 IME preedit 区段表达）。
- winit → Zircon 翻译收口为 runtime 单实现（新增 `zircon_runtime/src/ui/platform_input/`，editor 与 runtime 预览两宿主共用）；editor host 只持有 EventLoop 并喂 batch，不再解释 winit 语义。

#### 3.1.1 IME 职责分工（2026-07-02 评审收口）

与 `docs/plans/zircon_runtime/text/08` 的职责表**互为镜像**，两处以本裁决为准（U8）：

| 职责 | 归属 |
|------|------|
| winit 基线入站翻译（`Ime::Preedit/Commit/Enabled/Disabled/DeleteSurrounding` → `UiWindowPlatformInputEvent`） | zircon_runtime `ui/platform_input`（**本计划 01 拥有**） |
| 平台特化（TSF/IMM32/IBus/fcitx）与出站 host request 应用（enable/disable、候选窗 anchor rect） | zircon_app 平台层（text/08 IM-M2） |
| iface dispatch DTO（`UiImeInputEvent` 等载体）变更 | 由 01 与 text/08 **协同一次合并**，不各自演进 |

focus→IME 生命周期次序：焦点进入可编辑节点 → `enable + anchor rect`；焦点离开 → `commit preedit → disable`；popup 抢焦期间 Esc **先取消组合再关 popup**（详见 §3.2 键盘/焦点次序矩阵）。

### 3.2 路由层（对应 FSlateApplication）

`zircon_runtime/src/ui/dispatch/input_manager/`（新增 owner 模块）：

- 入口 `UiInputManager::dispatch_window_batch(surface, batch) -> UiInputDispatchOutcome`，内部收编双分发器为阶段。
- 路由顺序固定且单点实现：**capture target → popup 层级（含外点关闭判定）→ preview/tunnel（root→leaf）→ direct（leaf）→ bubble（leaf→root）→ focus-path（键盘）→ default-action**，与现有 `UiInputRoutePolicy` 枚举一一对应。
- 鼠标：hover enter/leave 成对、press/release/click/double-click 合成、wheel 沿 hit path 冒泡到第一个可滚动节点、capture 期间只发 capture 目标。
- 键盘：focus path 路由；Tab/Shift+Tab 走 navigation 阶段；Enter/Space 激活；Escape 自顶向下关闭最上层 popup；其余字符进文本编辑链（计划 03）。
  - focus→IME 生命周期次序矩阵（2026-07-02 评审收口，U8）：

    | 触发 | 次序 |
    |------|------|
    | 焦点进入可编辑节点 | IME `enable` + 上报候选窗 anchor rect |
    | 焦点离开可编辑节点 | 先 `commit preedit`，再 IME `disable` |
    | popup 抢焦期间按 Esc | **先取消当前组合（cancel composition），再关最上层 popup**；两步不可在同一按键内合并跳过 |
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
  → UiWindowInputPumpBatch（当前`push_coalesced`仅合并相邻redraw；move/axis typed coalescing属PERF-MVP-314待办）
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
| M1.S1 | 翻译盘点矩阵：以 editor `native_input_translation` 现行为金标准，固化 winit↔`UiWindowPlatformInputEventKind` 全 variant 映射测试；列出触摸/IME 缺口清单。（2026-07-02 评审收口）盘点矩阵补条款：**IME 激活期间 KeyboardText 抑制**——printable 字符与 `Ime::Commit` 必须去重（IME enabled 时抑制 KeyboardText/字符事件直发，仅走 Commit 通道），防止双输入 | `zircon_editor/src/tests/host/retained_window/native_input_translation.rs`（扩充） | `cargo test -p zircon_editor --lib native_input_translation --locked` | 无删除 |
| M1.S2 | `platform_input::winit_translation` 单实现落地，rhi/rhi_wgpu ui_surface 翻译段并入 | 新增 platform_input/；核验 rhi/ui_surface.rs、rhi_wgpu/ui_surface.rs 当前仅有 surface descriptor 逻辑，无 winit 输入翻译段可迁 | `cargo check -p zircon_runtime --lib --locked` | rhi 两处输入翻译段按当前代码核验为不存在；S3 删除 editor 本地翻译 |
| M1.S3 | editor/app 切换调用方；删除 editor 本地翻译 | event_loop/platform_input.rs；删 native_input_translation.rs 与 native_input_translation/**；保留经核验非翻译的 native_keyboard.rs | `cargo test -p zircon_editor --lib --locked` | 删 editor-local 翻译树 + event_loop/input.rs 指针/滚轮翻译段 |
| M1.S4 | interface 触摸/IME variant 补缺（按 S1 清单，集中一次）。（2026-07-02 评审收口，U8）iface dispatch DTO（`UiImeInputEvent` 等）变更与 text/08 **协同一次合并**，不各自演进 | `zircon_runtime_interface/src/ui/window/input.rs` | `cargo test -p zircon_runtime_interface --locked` | 无删除 |
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

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

本子计划产出记录已超过 10 条，具体记录已迁入编号子目录。

- 迁入记录：[`01/2026-07-09-slate-input-dispatch-core-output-records.md`](01/2026-07-09-slate-input-dispatch-core-output-records.md)
- open 待修复：[plan-output-archive-notice](01/failure-2026-07-13-plan-output-archive-notice.md)
- 2026-07-22 control index交接：PERF-MVP-128已在静态componentized workbench surface增加构造期`control_id→UiNodeId`止损；EditorUI01需把索引下沉为`UiSurface` structure generation权威，支持duplicate id、virtual insert/prune、popup descendants及结构代际失效。event/popup/data-sync lookup不得继续全树scan或持有跨generation私有node id。
- 2026-07-23 binding route交接：PERF-MVP-572确认产品`invoke_binding`每event把path/action/arguments格式化与转义成新String查表，随后clone arguments/binding/result。EditorUI01让input manager、compiled template与`route_intent`直接携generation-scoped `UiRouteId`/typed handle；stable event不得format/parse/hash native binding正文，reload stale handle明确拒绝/重解。native String只在authoring/serde/unknown error物化；1/100/10k routes与1M pointer/keyboard/change events记录String/clone bytes和p95，保留route order/effect/roundtrip合同。
- 2026-07-23 event/control补证：interface `UiInvocationContext/Result/Notification`当前分别拥有binding、arguments与JSON Value，runtime broadcast再clone result。EditorUI01沿PERF-MVP-572让normal input只传route handle/shared payload，route intent不得复制binding graph；unknown/error边界才物化native binding。subscriber fanout/backpressure仍由252，Tree/Node reflection不进入普通input frame；验收1MiB action payload与1M events owner=1、binding/result deep clone=0。
- 2026-07-23 window batch补证：源码纠正了管线图旧假设——`UiWindowInputPumpBatch::push_coalesced`只去相邻redraw，ABI adapter还直接`push`，不存在move合并；manager同步逐项route并保存N个full results。EditorUI01按PERF-MVP-314让winit/ABI adapter与manager共用typed barrier batch：连续move/axis/hover按Runtime12 accumulator压缩，resize/scale先提交Runtime09 geometry barrier，边沿事件严格保序；normal release path只保留轻量aggregate，PERF-MVP-293完整route diagnostics受capture预算。验收100k mixed storm的route/results/diagnostic bytes、barrier generation、queue age和interaction p95。
- 2026-07-23 accessibility root补证：interface `UiAccessibilityTreeSnapshot::node`仍线性扫描wide owned nodes，产品单target action还先同步构建/验证整树。EditorUI01继续按PERF-MVP-256/257发布与tree/layout/component/focus generation一致的accessible node index与changed-node update；action只查目标generation，不为单target构建snapshot。参考Slint按value/label/description/focus分别PropertyTracker并发布对应OS event。验收10k nodes×10k actions的snapshot build=0、lookup近O(1)、无关node visits=0与AccessKit changed nodes近delta。
- 2026-07-23 dispatch clean合同补证：interface pointer/navigation context必须owned route，result还同时保留route/invocations/passthrough/damage/component/binding；`UiInputDispatchResult`持full event+reply+diagnostics+effect分类。产品dispatcher确实route clone进result并逐node clone context。EditorUI01按PERF-MVP-254/293/294让single route owner贯穿handler与result，release只保留compact outcome/effect index，full trace按entries+bytes+age显式capture；1M events验证handler数不增加route bytes、normal full trace alloc=0、effect payload owner=1。
- 2026-07-23 external-current focus/reply补证：interface `focus_chain()`每调用用BTreeSet遍历全部reachable nodes并sort，`UiHitPath`同时持root-to-leaf与反向bubble Vec，`UiDispatchReply::merge_route`无条件建立step trace与merged effects。EditorUI01按PERF-MVP-253让tree/layout generation发布唯一预排序focus index，stable Tab不得调用全树helper；按254/293/294让single route/effect artifact贯穿merge/result，release默认只产compact outcome。10k nodes×10k keys和depth64×1M events记录BTree/sort/route/trace/effect clone，稳定值为0或O(1)。
- 2026-07-30 retained pointer-layout current-source补证：app adapter 11/11确认bridge equality只在上游已完成owned projection后才返回。activity/browser每stable slow path仍clone 2份workspace snapshot并构造8个layout；hierarchy复制scene slice并逐row格式化id；Welcome重收集paths，click还先建完整chrome。EditorUI01按PERF-MVP-109/117接收EditorUI08的typed changed rows/sizes，维护唯一stable row identity、visible range、hit grid与route handle；不得让app先全量构造layout再交bridge深比较。1/100/1k/10k rows记录row visit/String/layout bytes、active hit cells与scroll delta；证据见`../../performance/01/2026-07-30-editor-retained-pointer-layout-current-review.md`。
- 2026-07-30 retained workbench-pointer current-source补证：host-page overflow route已由stable bridge给出，却为翻转单个`open` bool调用`get_host_presentation()`深clone整结构树及viewport RGBA。EditorUI01按PERF-MVP-147让overflow成为interaction generation窄字段，提供read/toggle或CAS API；1M overflow clicks的full presentation clone/RGBA copied bytes=0，不在adapter保留第二snapshot cache。same click仍只请求paint-only并保持open/close语义；证据见`../../performance/01/2026-07-30-editor-retained-workbench-pointer-current-review.md`。
- 2026-07-31 welcome-recent pointer current-source补证：app 4/4与bridge 20/20确认stable move在size sync前后重复三项Slint投影，action hover仍物化owned path；每click先建完整chrome/全部path layout，scroll offset变化重建root/viewport+3N nodes、dispatcher和双path route。EditorUI01按PERF-MVP-117让generation-owned typed row/route、visible range与hit grid成为唯一authority：same-hover move为0 path clone/0 UI write，stable click不建full chrome/path Vec，scroll只patchtransform/visible hit cells。EditorUI08只交changed project-list delta，不在consumer复制cache；证据见`../../performance/01/2026-07-31-editor-retained-welcome-recent-pointer-current-review.md`。
- 2026-07-31 detail-scroll pointer current-source补证：app 4/4与bridge 27/27确认runtime viewport offset已原地更新且scroll不再rebuild两节点surface；但`ScrollSurfaceHostState`把dispatch state-change擦成`Result<()>`，Console/Inspector/Asset Details对zero/clamped scroll仍无条件setter。EditorUI01按PERF-MVP-110/171让shared dispatch返回typed `Ignored/Handled { changed, damage }`，owner unchanged时app setter/redraw为0，不在consumer另建offset cache。稳定source-window focus已有early return；证据见`../../performance/01/2026-07-31-editor-retained-detail-scroll-pointer-current-review.md`。
- 2026-07-31 asset-drag payload current-source补证：route已携content item index/reference row index，但press丢弃后按UUID重扫全list；每left-down在drag threshold前就构造宽payload、双locator和summary/status Strings。EditorUI01按PERF-MVP-109让down只arm generation-owned typed row candidate，真实Begin后才向Editor09 metadata slot O(1) resolve并物化一次payload；stale generation取消，不建第二UUID map。click-only scan/payload/status=0；证据见`../../performance/01/2026-07-31-editor-retained-asset-drag-payload-current-review.md`。
- 2026-07-31 menu-pointer app adapter current-source补证：`menu_pointer.rs`每次move/scroll都接收bridge克隆的完整state并无条件写回全部Slint menu字段，即使hover/path/offset未变；有效scroll/submenu仍可全量重建surface。EditorUI01按PERF-MVP-112让shared dispatch返回typed `{changed, damage, route}`并由唯一visible popup/hit authority增量patch；same hover与zero/clamped scroll的state/path clone、setter和surface rebuild均为0。diagnostics另由EditorUI08/PERF-MVP-601处理，不在adapter建第二state cache；证据见`../../performance/01/2026-07-31-editor-retained-small-input-adapters-current-review.md`。
- 2026-07-31 asset reference state helper current-source补证：retained state selection与drop take本身均为常数工作，但reference bridge每move/scroll返回完整state，host随后为单列表事件重写tree/content/references/used-by全部8项UI字段；有效scroll仍为全部reference rows重建surface。EditorUI01按PERF-MVP-109让dispatch返回typed changed/damage与generation row slot，same hover/zero-clamped scroll的state clone、8 setters、rebuild均为0；drop payload继续只在真实drop move一个owner，不在helper建缓存。证据见`../../performance/01/2026-07-31-editor-retained-state-visibility-helpers-current-review.md`。
