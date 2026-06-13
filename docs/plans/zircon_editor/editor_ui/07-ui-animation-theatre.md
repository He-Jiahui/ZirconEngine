---
related_code:
  - zircon_runtime/src/animation/mod.rs
  - zircon_runtime/src/animation/sequence/interpolation.rs
  - zircon_runtime/src/animation/sequence/channel_sample.rs
  - zircon_runtime/src/animation/sequence/time.rs
  - zircon_runtime/src/ui/component/catalog/material_foundation/layout_transitions.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/surface/timeline.rs
  - zircon_runtime_interface/src/ui/style.rs
  - zircon_editor/src/ui/animation_editor/session.rs
  - zircon_editor/src/ui/animation_editor/presentation.rs
  - dev/theatre/packages/core/src
  - dev/theatre/packages/studio
plan_sources:
  - .codex/plans/UI Asset Editor 与共享 Layout 未完成内容归档.md
  - .codex/plans/Zircon 性能时间轴与 Tracy 集成设计.md
status: planned
---

# 07 UI 动画与 theatre 式时间轴

## 1. 目标

补齐 UI 动画引擎（归档 M19 的正面解决）：(a) runtime 侧属性插值引擎，驱动 transitions（Collapse/Fade/Grow/Slide/Zoom 目前只有组件定义、没有时间插值）与交互微动画；(b) 数据模型对齐 theatre 的 sheet/object/track/keyframe 结构，使同一份动画资产既能 runtime 播放、又能在 editor 时间轴面板（theatre 式）编辑。

## 2. 现状（按代码核实修正）

### 2.1 已存在的设施

| 能力 | 落点 | 证据 |
|------|------|------|
| 场景动画系统 | `zircon_runtime/src/animation/` | clip_event.rs、manager/、module.rs、scene_hook/、sequence/{apply, channel_sample, conversion, interpolation, target, time}.rs——**插值/通道采样设施可参考**（与 ECS 耦合，UI 侧不直接复用） |
| transition 组件定义（无插值） | `catalog/material_foundation/layout_transitions.rs` | `transition("Collapse"/"Fade"/"Grow"/"Slide"/"Zoom", ...)`（:5–:9），仅组件登记 |
| editor 时间轴会话命令模型 | `zircon_editor/src/ui/animation_editor/session.rs` | `from_path`（:56）、`save`（:113）、`pane_presentation`（:139）、`add_key`（:236）、`remove_key`（:269）、`create_track`（:288）、`remove_track`（:325）、`rebind_track`（:352）、`scrub_timeline`（:421）、`AnimationTrackPath` |
| surface 帧入口 | `zircon_runtime/src/ui/surface/surface.rs` | `surface_frame()`（:165）→ `UiSurfaceFrame` |
| **命名占用警示** | `zircon_runtime/src/ui/surface/timeline.rs` | `UiDebugTimelineStore`（:9）是调试帧时间线，与 motion 无关——新模块命名避让 |

### 2.2 真实缺口

无 UI 属性插值引擎、无缓动函数库、无动画 clock 接入 UI 帧循环、无 keyframe 资产格式（`UiMotionDocument`）、editor 时间轴面板无 UI 动画数据适配。

## 3. 设计

### 3.1 属性插值引擎（runtime）

- `zircon_runtime/src/ui/motion/`（新 owner 模块）：
  - 可动画属性集：opacity、transform（translate/scale/rotate，渲染期变换不触发布局）、颜色、以及显式声明的布局属性（width/height 用于 Collapse 等，走计划 02 增量布局）。
  - `UiMotionEngine`：每 surface 一个活动动画表；帧循环 tick → 求值 → 写入 component state / render extract 覆盖层 → 只 damage 受影响节点。
  - 缓动库：standard/emphasized/decelerate/accelerate（Material 曲线）+ cubic-bezier 自定义 + spring（可选后置）。
- 触发模型：
  - **状态过渡**：计划 04 的状态集变化（hover 进出、open/close）声明 `transition(property, duration, easing)`，写在 style/theme 层（类 CSS transition）。
  - **显式动画**：组件/编辑器代码请求播放某段 keyframe 序列（toast 进出、drawer 展开）。

### 3.2 动画资产与 theatre 对齐

- 资产格式 `UiMotionDocument`（TOML，走计划 05 资产管线）：对齐 theatre 概念——sheet（动画簿）→ object（目标节点/组件绑定路径）→ track（属性轨）→ keyframe（time、value、easing、handle）。
- 绑定路径用组件树稳定 id + 属性通道名，编译期校验目标存在。
- 播放控制：play/pause/seek/loop/speed；与本地化/主题热重载一致地支持热重载。

### 3.3 Editor 时间轴编辑（theatre 式，落在计划 09 的动画模块）

- TimelinePanel（L4 组件）+ 轨道行（复用 TreeRow/TableRow 语法）+ keyframe 手柄（Canvas 容器）+ 播放头。
- 编辑会话沿用 `animation_editor/session` 既有命令模型（add_key/remove_key/create_track/rebind_track/scrub_timeline 已存在）：为 `UiMotionDocument` 增加数据适配器，命令走 editor command + undo/redo。
- 同一面板服务两类目标：UI 动画资产（本计划）与场景动画 clip（既有 animation 系统）——面板组件共享，数据适配各自实现。

## 4. 接口与数据结构草案

```rust
// 新增 zircon_runtime/src/ui/motion/{mod.rs, engine.rs, track.rs, easing.rs,
//   transitions.rs, document.rs, binding.rs}
pub struct UiMotionEngine {                       // 每 surface 一个（避让 timeline.rs 命名）
    active: Vec<UiMotionInstance>,
}
pub struct UiMotionInstance {
    pub binding: UiMotionBinding,                 // 节点稳定 id + 通道
    pub track: UiMotionTrack,
    pub started_at: UiInputTimestamp,             // 现有类型（01）
    pub speed: f32,
    pub looping: UiMotionLoop,                    // Once | Loop | PingPong
    pub state: UiMotionPlayState,                 // Playing | Paused | Finished
}
pub enum UiMotionChannel {
    Opacity, TranslateX, TranslateY, ScaleX, ScaleY, Rotate,
    Color,                                        // 渲染期通道
    LayoutWidth, LayoutHeight,                    // 显式布局通道（走 02 增量布局）
}
pub struct UiMotionTrack { pub channel: UiMotionChannel, pub keyframes: Vec<UiMotionKeyframe> }
pub struct UiMotionKeyframe {
    pub time: f32,
    pub value: UiMotionValue,                     // Scalar(f32) | Color(UiRgbaColor)
    pub easing: UiEasing,
    pub handle: Option<UiBezierHandle>,           // theatre 式手柄
}
pub enum UiEasing { Standard, Emphasized, Decelerate, Accelerate, CubicBezier(f32, f32, f32, f32) }
                                                  // Spring 后置，不进第一版枚举语义承诺

impl UiMotionEngine {
    /// 帧循环 tick：active 为空直接返回（空闲零开销验收点）
    pub fn tick(&mut self, surface: &mut UiSurface, now: UiInputTimestamp) -> UiMotionTickReport;
    pub fn play(&mut self, instance: UiMotionInstance) -> UiMotionHandle;
    pub fn pause(&mut self, handle: UiMotionHandle);
    pub fn seek(&mut self, handle: UiMotionHandle, time: f32);
}
pub struct UiMotionTickReport { pub evaluated: u32, pub damaged_nodes: u32, pub finished: u32 }

// transitions.rs：状态过渡声明（style/theme 层语法，04 衔接）
pub struct UiTransitionSpec { pub channel: UiMotionChannel, pub duration: f32, pub easing: UiEasing }
pub fn transitions_for_state_change(
    /* style 解析结果, */ old: UiPainterState, new: UiPainterState,  // 04 现有类型
) -> Vec<UiTransitionSpec>;

// document.rs：UiMotionDocument（TOML，05 管线注册）
// [sheet]                    id = "drawer-open"
// [[object]]                 binding = "shell.drawer.left"        # 组件树稳定 id
// [[object.track]]           channel = "layout_width"
// [[object.track.keyframe]]  time = 0.0   value = 0.0    easing = "emphasized"
// [[object.track.keyframe]]  time = 0.18  value = 320.0  easing = "decelerate"
pub struct UiMotionDocument { pub sheet_id: String, pub objects: Vec<UiMotionObject> }
pub fn validate_motion_bindings(doc: &UiMotionDocument, /* prototype store */) -> Vec<UiMotionBindingDiagnostic>;
```

## 5. 模块与文件落点

**新增**：`zircon_runtime/src/ui/motion/{mod.rs, engine.rs, track.rs, easing.rs, transitions.rs, document.rs, binding.rs}`、`zircon_runtime/src/asset/assets/ui_motion.rs`（资产注册，照 05 样板）、`zircon_editor/src/ui/animation_editor/motion_adapter.rs`（UiMotionDocument ↔ session 命令模型适配）

**修改**：

| 路径 | 改什么 |
|------|--------|
| `zircon_runtime/src/ui/surface/surface.rs` 帧管线 | 插入 motion tick 阶段（§6） |
| `catalog/material_foundation/layout_transitions.rs` | Collapse/Fade/Grow/Slide/Zoom 接 UiTransitionSpec 真实插值 |
| 04 的 theme/style 解析 | transition 声明字段解析 |
| `zircon_editor/src/ui/animation_editor/{session.rs, presentation.rs}` | 泛化目标类型（场景 clip / UI motion 双适配） |

**删除（硬切换义务）**：layout_transitions.rs 中「仅登记无行为」的占位路径（M2.S3 接通同变更收编）；editor 侧任何手写的 UI 渐变/动画临时代码（盘点后列清单删除）。

## 6. 管线时序

```
input pump → dispatch → state reduce
  → 状态变化触发 transitions_for_state_change → UiMotionEngine.play（即时 track）
→ motion tick（新阶段，布局前）：
    布局通道（LayoutWidth/Height）求值 → 写组件布局属性 → mark layout dirty（增量）
→ layout（Taffy/Zircon 容器）→ text
→ 渲染期通道（opacity/transform/color）求值 → render extract 覆盖层 → 只 damage 受影响节点
→ GPU command stream
空闲（active 为空）：tick 直接返回，零求值零 damage。
```

## 7. 里程碑切片化

| # | 切片 | 涉及文件 | 验证命令 | 硬切换 |
|---|------|---------|---------|--------|
| M1.S1 | 缓动库 + 插值核心（Material 四曲线 + cubic-bezier；参考 animation/sequence/interpolation 设施但 UI 侧独立实现） | motion/easing.rs、track.rs | `cargo test -p zircon_runtime --lib motion_easing --locked` | 无删除 |
| M1.S2 | UiMotionEngine + tick + opacity/transform 通道（渲染期覆盖层） | motion/engine.rs、surface.rs | `cargo test -p zircon_runtime --lib motion --locked` | 无删除 |
| M1.S3 | 帧报告 damage 范围测试 + 空闲零开销断言 | 测试 | 同上 | 无删除 |
| M2.S1 | transition 声明解析（style/theme 层字段，04 M3 解析链） | transitions.rs、04 解析链 | `cargo test -p zircon_runtime --lib transition --locked` | 无删除 |
| M2.S2 | 状态过渡接通：hover 进出 / open-close 自动生成 transition instance | transitions.rs、state reduce 接缝 | 同上 | 无删除 |
| M2.S3 | Collapse/Fade/Grow/Slide/Zoom 真实动起来（布局通道走增量布局） | layout_transitions.rs | `cargo test -p zircon_runtime --lib layout_transitions --locked` + 实机 drawer/menu 过渡 | 占位登记收编 |
| M3.S1 | `UiMotionDocument` TOML + 绑定路径编译期校验 | motion/document.rs、binding.rs | `cargo test -p zircon_runtime --lib motion_document --locked` | 无删除 |
| M3.S2 | 资产注册 + 播放控制 play/pause/seek/loop/speed | asset/assets/ui_motion.rs | `cargo test -p zircon_runtime --lib motion --locked` | 无删除 |
| M3.S3 | 热重载（05 M2 链）+ round-trip 测试 | watch 分流 | 同上 + 实机改文件 | 无删除 |
| M4.S1 | motion_adapter：UiMotionDocument 接 session 命令模型（add_key/remove_key/create_track/rebind_track/scrub_timeline 复用） | animation_editor/motion_adapter.rs | `cargo test -p zircon_editor --lib animation_editor --locked` | 无删除 |
| M4.S2 | TimelinePanel 实机：编辑 keyframe → 即时回放（依赖 08 面板承载与 09 批次 2 进度） | editor 模块侧 | 实机 | 无删除 |

## 8. 测试矩阵（代表性用例）

- **M1**：`easing_curves_match_reference_samples`、`opacity_track_interpolates_between_keyframes`、`idle_engine_tick_is_noop`、`tick_damages_only_animated_nodes`
- **M2**：`hover_exit_reverses_transition_midway`、`collapse_transition_drives_layout_height_incrementally`、`open_close_transition_respects_easing`
- **M3**：`motion_document_round_trips_toml`、`invalid_binding_path_reports_diagnostic_not_panic`、`seek_evaluates_deterministic_value`、`hot_reload_replaces_active_sheet`
- **M4**：`motion_adapter_add_key_updates_document`、`scrub_timeline_previews_value`

落点：runtime `motion/` 各文件 `#[cfg(test)]`；editor 沿 animation_editor 邻近测试。

## 9. 风险与对策

| 风险 | 对策 / 探测信号 |
|------|----------------|
| 布局通道动画每帧触发布局，性能劣化 | 只走增量布局且限定显式声明的通道；帧报告记录重排节点数，超阈值报警 |
| transform 覆盖层与命中几何不一致（动画期间点击错位） | 命中默认用布局几何；动画期间命中策略显式声明（边界约束沿用），测试覆盖 |
| 状态过渡与 reducer 状态机抢写（hover 快速进出） | 同 channel 同节点新 transition 替换旧 instance（从当前值续插），测试 `hover_exit_reverses_transition_midway` |
| 双时间轴概念混淆（debug timeline vs motion） | 命名避让（UiMotionEngine）；文档显式区分 |
| M4 依赖 08/09 进度 | M4 排最后且只依赖面板承载；引擎与资产（M1–M3）不被 editor 阻塞 |

## 10. 里程碑级依赖表

| 里程碑 | 前置 | 被依赖 |
|--------|------|--------|
| M1 | 02 M1（布局属性，布局通道）、01 M1（UiInputTimestamp） | 07 M2/M3 |
| M2 | 07 M1、04 M3（style 解析链）、04 M2（状态变化源） | 08 M6（toast/drawer 过渡，弱） |
| M3 | 07 M1、05 M1（资产注册）、05 M2（热重载） | 07 M4 |
| M4 | 07 M3、08 M2（面板承载）、09 批次 2（动画模块协同） | E3（完整编辑器） |

## 11. 完成定义

- 实机：drawer 展开/收起、菜单淡入、Collapse/Fade/Grow/Slide/Zoom 全部有时间插值；空闲帧 motion tick 零开销（帧报告佐证）。
- `UiMotionDocument` 资产可建/可播/可热重载；editor 时间轴可加删 keyframe 并即时回放。
- 验收命令组：`cargo test -p zircon_runtime --lib --locked`（motion/transition/layout_transitions 过滤）、`cargo test -p zircon_editor --lib animation_editor --locked`、实机过渡验收。

## 12. 边界约束

- 动画 tick 在 UI pipeline 固定阶段执行（§6），不引入独立线程；性能预算：空闲（无活动动画）零开销。
- transform 动画走渲染期覆盖，不污染 arranged geometry；命中默认用布局几何（动画期间命中策略显式声明）。
- 不复制 theatre 的运行时/前端代码，只对齐数据模型与编辑交互。
- spring 缓动为后置项，第一版枚举不承诺其语义。

## 13. 参考实现对照（dev/ 源码锚点）

实现切片前先读对应锚点，不确定的行为语义以参考实现为准（在 PR 说明中注明出处）；禁止凭印象实现、禁止引用未核实路径。

| 设计点 | 主参考 | 次参考 | 参考什么 |
|--------|--------|--------|---------|
| sheet/object/track/keyframe 数据模型 | `dev/theatre/packages/core/src`（projects/ 下 sheet/sequence 模型，TS 源） | — | `UiMotionDocument` 的概念对齐：对象绑定路径、轨道、keyframe handle（只对齐数据模型，不复制代码） |
| 时间轴编辑交互 | `dev/theatre/packages/studio` | — | 播放头/框选/拖 keyframe/改 easing 的编辑器交互范式（M4 面板） |
| UI 侧曲线动画 | `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Animation` | — | FCurveSequence/FCurveHandle：UI 动画句柄、播放控制与 widget 生命周期的关系 |
| 曲线采样架构 | `dev/bevy/crates/bevy_animation` | `zircon_runtime/src/animation/sequence/`（仓内场景动画） | 通用曲线/采样器的 Rust 形态；与仓内 sequence 设施的取舍对照 |
| UI 动画/状态机 | `dev/Fyrox/fyrox-ui/src/{animation.rs, absm.rs}` | `dev/Fyrox/fyrox-animation` | UI 属性动画与动画状态机在 UI crate 内的组织 |
| 补间/过渡语义 | `dev/godot/scene/animation`（Tween 等） | `dev/material-ui/packages/mui-material/src/Collapse`（及 Fade/Grow/Slide/Zoom 目录） | Tween 的属性通道与缓动组合；MUI transition 组件的 enter/exit 语义（M2.S3 的行为标准） |
