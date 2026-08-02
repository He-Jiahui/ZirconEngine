# 04 · Animation 插件完善计划（骨骼 / Clips / 状态机 / GPU Skinning / Avatar Mask）

> 状态：工程化细化版 v2 · 优先级：P1 · 前置：[01 插件架构核心](01-plugin-architecture-core.md) M1
> 关联计划：`.codex/plans/Physics + Full Animation Support 新计划.md` · 现状文档：`docs/zircon_plugins/animation/runtime.md`
> 参考实现：Bevy `bevy_animation`（AnimationGraph/AnimationTarget/mask 位掩码）、Fyrox ABSM（多层状态机 + Layer Mask）、Unreal AnimGraph（蒙太奇/同步组仅作形态参考）
> 历史进度（2026-06-14 00:21 +08:00）：`runtime_physics_animation_tick_contract` 通过临时 manifest 与外部 target-dir 复跑，16 项全通过。该回执只描述当时快照，不代替当前受管验收。
> 当前状态（2026-08-01）：`in_progress / resolving_failure`。M1-M6 的历史实现与验证证据仍有效，但协调器仍有 3 个开放 failure；在它们返回并完成当前源码受管验证前，不进入 closeout。

## 1. 目标

把 `zircon_plugins/animation` 推进到完整动画系统：骨骼 clip 采样与混合、动画图（blend tree）、多层状态机、avatar mask、GPU skinning、timeline tracks 闭环、动画事件，并为 physics ragdoll 与 sound 自动化提供稳定姿态/事件通道。

## 2. 历史基线与当前源码校准

以下 A1-A7 是 M1-M6 实施前的历史基线，用于解释设计动机，不再代表 2026-08-01 当前缺口。

中立契约 `zircon_runtime/src/core/framework/animation/` 已有（DTO 层比早期假设完整）：`avatar_mask.rs`（`AnimationAvatarMask`：**字符串 target 寻址** `allows_target(&str)` + `normalized_weight()`）、`gpu_skinning.rs`（`AnimationSkinningBackend` / `AnimationGpuSkinningReadiness` readiness 契约）、`graph_blend_mode.rs`、`graph_evaluation.rs`、`graph_clip_instance.rs`、`state_machine_evaluation.rs`（`AnimationStateMachineEvaluation` / `AnimationStateTransitionEvaluation`）、`parameter_map.rs`/`parameter_value.rs`、`pose_bone.rs`/`pose_output.rs`/`pose_source.rs`、`event.rs`、`timeline.rs`、`track_path.rs`、`playback_settings.rs`、`manager.rs`（`AnimationManager` trait）。

插件实现 `zircon_plugins/animation/runtime/src/`：`manager/`（graph/parameters/pose/sampling/state_machine）、`sequence/`（apply/channel_sample/conversion/interpolation/target/time）、`clip_event.rs`、**`scene_hook/` 9 个文件**（tick/scan/pose/node_pose/graph/state_machine/sequences/events/pending，∈ PostUpdate）。`animation/editor` 与 `animation_graph/editor` 均为骨架。`.zranim` 与模型派生 skeleton/clip 的资产管线已定（重导入 locator/uuid 稳定规则维持不变）。

缺口（按严重度，依实查校准）：

| # | 缺口 | 证据 |
|---|------|------|
| A1 | 求值全链路字符串寻址（target path / mask `allows_target(&str)`），每帧哈希/比较；无 dense id、无 SoA pose buffer、分配未管控 | `framework/animation/avatar_mask.rs:25`、`sequence/target.rs` |
| A2 | GPU skinning：readiness 契约在（`gpu_skinning.rs`），实际 matrix buffer/shader 路径完全缺失，蒙皮全 CPU | `framework/animation/gpu_skinning.rs` |
| A3 | 状态机：评估 DTO 在，执行器缺迁移条件表达式、中断策略、多层、BlendSpace | `manager/state_machine.rs` |
| A4 | Avatar mask：DTO 在，未编译为 dense 权重、未参与 PoseBlend、未与 ragdoll Blended 连通 | `framework/animation/avatar_mask.rs` |
| A5 | IK 无（TwoBoneIk/LookAt） | runtime 无 ik 模块 |
| A6 | Ragdoll 姿态读写通道未定义（与 [03 Physics](03-physics.md) §3.4 对偶） | — |
| A7 | scene hook 9 文件形态，无 `animation.evaluate` 系统锚点（01-M1 首批迁移对象） | `scene_hook/` |

### 2.1 2026-08-01 当前源码

- `zircon_plugins/animation/runtime/src/evaluation/` 已拥有 `TargetTable`、`PosePool`、compiled clip/graph、pose buffer 与 folder-backed pipeline；A1/A7 的“完全缺失”描述已被实现取代。
- `mask/`、`state_machine/`、`gpu_skinning/` 与 `ik/` 已存在，`SkinningPalette`、dense `MaskWeights`、`AnimationIkCommand` 和有界逐 World 队列均有生产 owner。
- `runtime_system.rs` 已注册 `AnimationClipEvent`，`evaluation/pipeline/events.rs` 提供事件投影；M1-M6 的详细历史结果已迁入编号记录，不在父计划重复展开。
- 当前剩余关闭条件以 3 个 canonical failure 为准：[`runtime-animation-fallback-evaluator-divergence`](04/failure-2026-07-22-runtime-animation-fallback-evaluator-divergence.md)、[`animation-frame-diagnostics-hardcut-omission`](04/failure-2026-07-29-animation-frame-diagnostics-hardcut-omission.md)、[`animation-sequence-caller-root-drift`](04/failure-2026-07-29-animation-sequence-caller-root-drift.md)。三者保持 `open`。

## 3. 架构设计

中立契约维持在 `zircon_runtime::core::framework::animation`；求值实现在 `zircon_plugins/animation/runtime`。

### 3.1 求值管线（解决 A1，`runtime/src/evaluation/` [新增]）

固定四段、单次遍历，全部在 `animation.evaluate` 系统内完成：

```
ParameterApply → StateMachineStep(per layer) → GraphEvaluate(per layer) → PoseBlend(layers × masks) → PoseApply(targets)
```

```rust
/// import 期由 skeleton 内路径稳定哈希派生；运行期经 TargetTable 解析为 dense 索引。
#[derive(Clone, Copy, PartialEq, Eq, Hash)] pub struct AnimationTargetId(u64);   // 契约层 [新增 target_id.rs]
#[derive(Clone, Copy)] pub struct TargetSlot(u32);                               // 实体内 dense 索引

/// SoA 布局：混合算子按通道向量化；零分配——求值期 buffer 全部来自 PosePool。
pub struct PoseBuffer {
    pub translations: Box<[Vec3]>,   // len = skeleton 骨骼数
    pub rotations:    Box<[Quat]>,
    pub scales:       Box<[Vec3]>,
    pub weights:      Box<[f32]>,    // 本 buffer 各骨骼有效权重（部分采样时 < 1）
}
pub struct PosePool { /* 按 skeleton 尺寸分桶的空闲链，acquire/release，帧间复用 */ }

pub enum BlendOp { Override, Additive /* 参考姿态差分 */, Masked { mask: MaskSlot } }
```

- `TargetTable`：skeleton 资产加载时一次构建 `AnimationTargetId → TargetSlot`；clip 的 channel 在加载期预解析为 `TargetSlot` 序列（`sequence/conversion.rs` [改造]），**运行期零字符串**。
- 各段输入输出：`ParameterApply`（参数表 → 层参数快照）→ `StateMachineStep`（快照 → 各层 `AnimationStateMachineEvaluation`，契约现名）→ `GraphEvaluate`（评估 → 每层 `PoseBuffer`）→ `PoseBlend`（层栈按 `BlendOp` 折叠为最终 `PoseBuffer`）→ `PoseApply`（写 transform + skinning palette）。

### 3.2 Avatar Mask（解决 A4，`runtime/src/mask/` [新增]）

- `.avatar_mask.toml` 资产：骨骼名/子树 → 权重（0..1，支持渐变边界，子树继承可覆写）；解析为契约 `AnimationAvatarMask`（现 DTO 保留为资产/编辑器视图）。
- 加载期编译：`AnimationAvatarMask × TargetTable → MaskWeights`（dense `Box<[f32]>`，与 `PoseBuffer` 同长同序）；`PoseBlend` 的 `Masked` 算子逐元素乘权。`allows_target(&str)` 仅保留给编辑器/诊断路径。
- mask 同时供 ragdoll `Blended` 模式限定物理权重范围（经 §3.6 通道传递 `MaskWeights` 引用）。

### 3.3 状态机（解决 A3，`runtime/src/state_machine/` [manager/state_machine.rs 改造为目录]）

```rust
pub struct StateMachineLayer {
    pub mask: Option<MaskSlot>,
    pub blend: BlendOp,                 // Override / Additive
    pub machine: CompiledStateMachine,  // 加载期编译：状态/迁移 dense 数组
}
pub enum StateKind { Clip(ClipSlot), BlendSpace1D(BlendSpace1D), BlendSpace2D(BlendSpace2D), SubMachine(MachineSlot), GraphRef(GraphSlot) }
pub struct TransitionDesc {
    pub conditions: CompiledConditionExpr,   // 参数比较的 AND/OR 树，加载期编译为后缀式求值
    pub duration: f32, pub exit_time: Option<f32>,
    pub interruption: InterruptionPolicy,    // None | CurrentToNext | NextToNext | Both
}
```

- 多层：每层独立 SM + mask + 混合模式；迁移期双姿态交叉淡化（PosePool 借两 buffer）。
- BlendSpace1D/2D：样本点 Delaunay 加权（2D）/线段插值（1D），样本权重在加载期排好序。
- 参数表：复用契约 `parameter_map.rs`/`parameter_value.rs`（float/bool/int/trigger）；VM 与行为树（[06 AI](06-ai.md) PlayAnimation 节点）经 `AnimationManager` 同一参数接口驱动。

### 3.4 GPU Skinning（解决 A2，`runtime/src/gpu_skinning/` [新增]）

- `PoseApply` 产出 skinning matrices 写入逐实体 storage buffer：

```rust
pub struct SkinningPalette {
    pub joint_matrices: [Mat4; MAX_SKIN_JOINTS],   // MAX_SKIN_JOINTS = 256
    // 双缓冲：本帧 + 上一帧矩阵（motion vector 用），交替写入两个 GPU buffer 区段
}
```

- 渲染对接：经契约 `gpu_skinning.rs` 的 readiness 协议——插件在 `finish` 阶段查询 `AnimationGpuSkinningReadiness::ready_for_gpu_skinning()`，不 ready 时（缺 storage buffer 特性/缺 shader 变体）整体走 CPU 回退并出诊断（`with_missing_gpu_resource` 现有路径）。蒙皮在 vertex shader 完成（render framework 的 mesh skinning 变体，与渲染计划协调，缺口先补 readiness 报告）。
- 超 256 骨的 skin：CPU 回退 + 诊断，不做分 palette 拆分（v1 裁决：拆分复杂度收益比差）。

### 3.5 IK（解决 A5，`runtime/src/ik/` [新增]，v1 最小集）

```rust
pub struct TwoBoneIkJob { pub root: TargetSlot, pub mid: TargetSlot, pub tip: TargetSlot,
                          pub target: Vec3, pub pole: Option<Vec3>, pub weight: f32 }
pub struct LookAtJob    { pub bone: TargetSlot, pub target: Vec3, pub axis: Vec3,
                          pub clamp_degrees: f32, pub weight: f32 }
```

作为 `PoseBlend` 之后、`PoseApply` 之前的后处理 pass；job 来自组件字段或脚本（经 Manager 命令）。

### 3.6 Ragdoll 姿态通道（解决 A6，与 [03 Physics](03-physics.md) §3.4 对偶，双方逐字一致）

- animation 在 `animation.evaluate`（PostUpdate）把目标姿态写入 physics 注册的 `SkeletalPoseTargets` 资源（契约 `framework/physics/skeletal_pose.rs`，physics 侧 [新增]）；
- animation 在 `PoseBlend` 前读取 `SimulatedPoseFeed` 资源（physics 于 `physics.sync_to_scene` ∈ FixedPostUpdate 写入，带插值 alpha），以 `Masked` 算子按 ragdoll mask 混合；
- animation 不直接访问 physics backend；两资源的读写声明进入系统 `SystemParamAccess`，由调度器保证顺序。

### 3.7 Timeline 与事件

- clip event track 保持（`clip_event.rs`）；事件经 `register_event::<AnimationEvent>`（契约 `event.rs` 现类型）进总线，sound 动态事件与 AI 感知均可订阅。
- `timeline_sequence`（editor 配套 crate）对接 sequence DTO，不新增运行时旁路。

### 3.8 ECS 集成（01-M1 首批验证对象）

- `scene_hook/` 9 文件删除，迁移为 `animation.evaluate` ∈ PostUpdate（01 锚点表；before `sound.spatial_update` 由 sound 侧 after 约束表达）；tick/scan/pending 等子步骤变为系统内部相位，不再是独立 hook。
- `register_resource::<AnimationParameterStore>`；`register_event::<AnimationEvent>`；组件走静态类型组件路径。

## 4. 模块文件树

```
zircon_runtime/src/core/framework/animation/
  target_id.rs                 [新增] AnimationTargetId
  avatar_mask.rs               [改造] 保留 DTO，标注编译产物语义
zircon_runtime/src/core/framework/physics/skeletal_pose.rs   [physics 计划新增，本计划消费]
zircon_plugins/animation/runtime/src/
  evaluation/mod.rs            [新增] 四段管线编排
  evaluation/pose_buffer.rs    [新增] PoseBuffer SoA + PosePool
  evaluation/target_table.rs   [新增] TargetTable / TargetSlot
  evaluation/blend.rs          [新增] Override/Additive/Masked 算子
  mask/compile.rs              [新增] AnimationAvatarMask → MaskWeights
  state_machine/{compiled,transition,blend_space}.rs  [新增，manager/state_machine.rs 收编]
  gpu_skinning/{palette,upload,fallback}.rs           [新增]
  ik/{two_bone,look_at}.rs     [新增]
  sequence/conversion.rs       [改造] channel 预解析为 TargetSlot
  manager/*                    [改造] 命令式接口对接新管线
  scene_hook/**                [删除] 迁移为 animation.evaluate 系统
```

## 5. 里程碑与任务分解

### M1 求值管线重构（与 01-M1 同窗口，scene hook 一次换血）

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M1-T1 | AnimationTargetId + TargetTable；clip channel 预解析 | target_id.rs、target_table.rs、sequence/conversion.rs | — | `target_id_stable_across_reimport`、`channel_resolution_has_no_string_lookup_at_runtime` |
| M1-T2 | PoseBuffer SoA + PosePool；混合算子 | pose_buffer.rs、blend.rs | M1-T1 | `blend_override_additive_golden_values`、`evaluate_performs_zero_allocations` |
| M1-T3 | 四段管线编排 + animation.evaluate 系统注册；删 scene_hook/ | evaluation/mod.rs、注册路径 | 01-M1-T4、M1-T2 | `animation_evaluate_anchor_in_post_update`、采样金样保绿 |

### M2 Avatar Mask 与多层混合（[03 Physics](03-physics.md) M5 的前置）

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M2-T1 | .avatar_mask.toml 解析 + MaskWeights 编译 | mask/compile.rs | M1-T1 | `subtree_weight_inherits_and_overrides`、`mask_boundary_gradient_values` |
| M2-T2 | 分层 PoseBlend（layers × masks × additive） | blend.rs、evaluation/mod.rs | M2-T1 | `upper_lower_body_split_blend_scenario` |
| M2-T3 | ragdoll 姿态通道（SkeletalPoseTargets 写 / SimulatedPoseFeed 读混合） | evaluation/mod.rs | M2-T2、03-M5-T1 | `simulated_pose_blends_under_ragdoll_mask` |

### M3 状态机完整化

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M3-T1 | CompiledStateMachine + 条件表达式编译 | state_machine/compiled.rs | M1 | `condition_expr_and_or_matrix` |
| M3-T2 | 迁移：duration/exit time/中断策略/交叉淡化 | state_machine/transition.rs | M3-T1 | `interruption_policy_matrix_contract`、`transition_crossfade_pose_continuity` |
| M3-T3 | BlendSpace1D/2D + 多层 | state_machine/blend_space.rs | M3-T1 | `blend_space_2d_triangulation_weights_sum_to_one` |

### M4 GPU Skinning

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M4-T1 | SkinningPalette + 双缓冲 upload | gpu_skinning/palette.rs、upload.rs | M1 | `palette_double_buffer_swaps_per_frame` |
| M4-T2 | readiness 探测 + CPU 回退 + 诊断 | gpu_skinning/fallback.rs | M4-T1 | `not_ready_falls_back_to_cpu_with_diagnostic`、`over_256_joints_falls_back` |
| M4-T3 | shader 蒙皮变体对接与一致性 | render framework 协调 | M4-T1 | `gpu_cpu_skinning_parity_within_tolerance`、1000 实例基准 |

### M5 IK 与事件总线

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M5-T1 | TwoBoneIk/LookAt 后处理 pass | ik/* | M1 | `two_bone_ik_reaches_target_within_epsilon`、`look_at_clamps_to_limit` |
| M5-T2 | AnimationEvent 接 register_event | 注册路径、clip_event.rs | 01-M2 | `clip_event_reaches_event_store_once` |

### M6 Editor

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M6-T1 | 状态机图编辑器（animation_graph crate 实化，register_graph_editor + node palette） | animation_graph/editor | M3、[10 规范](10-editor-integration.md) | editor 契约测试 |
| M6-T2 | blend space 编辑 + mask 骨骼树编辑（布局 `ai-blend-space-layout.png`） | animation/editor | M6-T1 | 扩展点注册快照 |
| M6-T3 | sequencer（布局 `ai-sequencer-layout.png`、register_timeline_editor） | timeline_sequence、animation/editor | M6-T1 | timeline track type 注册测试 |

## 6. 验收命令

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -ManifestPath zircon_plugins/Cargo.toml -Package zircon_plugin_animation_runtime -SkipBuild -LibTests
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -ManifestPath zircon_plugins/Cargo.toml -Package zircon_plugin_animation_graph_editor -SkipBuild -LibTests
```

## 7. 风险

- M1 重构触及 scene_hook 9 个文件，与 01-M1 调度迁移叠加——两者必须同一里程碑窗口完成，避免两次迁移成本（01-M1-T5 即本计划 M1-T3）。
- GPU skinning 依赖 render framework 的 storage buffer 与 shader 接缝；若 mesh 特性侧接口不足，先在 render framework 补 skinning palette 描述符（与 `docs/plans/zircon_runtime/render` 计划集协调），M4-T3 单列其后。
- 字符串寻址 → dense 化会改变 clip 资产加载产物；`.zranim` 格式不动，仅运行期表示变更，重导入稳定规则不受影响。

## 8. 附录 · dev 参考源码对位

实现各任务前**必须先读对应参考实现**，混合数学与状态机语义对照真实代码核对，禁止凭空实现：

| 设计点 | 参考源码（已核验存在） | 看什么 |
|--------|----------------------|--------|
| AnimationTarget 稳定寻址/graph/mask | `dev/bevy/crates/bevy_animation/src/` | AnimationTargetId 哈希派生、AnimationGraph 节点权重传播、mask 位掩码求值 |
| 多层状态机（ABSM）/pose/track | `dev/Fyrox/fyrox-animation/src/`（`machine/`、`pose.rs`、`track.rs`） | 层混合（含 LayerMask）、迁移交叉淡化、BlendSpace 采样点加权 |
| 状态机迁移/中断/同步组语义 | `dev/UnrealEngine/Engine/Source/Runtime/AnimGraphRuntime/`、`AnimationCore/` | transition interruption 模式、additive 差分姿态数学、IK（TwoBone/LookAt 节点实现） |
| GPU skinning buffer 布局 | `dev/bevy/crates/bevy_pbr/`（mesh skinning 路径）与 `dev/bevy/crates/bevy_mesh/` | joint matrices buffer 双缓冲与 motion vector 上一帧矩阵的传递 |

## 9. 状态与产出记录

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

- fixed 已修复：[animation-state-machine-infallible-conversion](../zircon_editor/editor/14/fixed-2026-07-11-animation-state-machine-infallible-conversion.md)
- 当前状态：M1-M6 实现与历史跨平台验收均已有编号记录，但 3 个开放 failure 尚未返回，因此计划保持 `in_progress / resolving_failure`。
- 开放 failure：[`runtime-animation-fallback-evaluator-divergence`](04/failure-2026-07-22-runtime-animation-fallback-evaluator-divergence.md)、[`animation-frame-diagnostics-hardcut-omission`](04/failure-2026-07-29-animation-frame-diagnostics-hardcut-omission.md)、[`animation-sequence-caller-root-drift`](04/failure-2026-07-29-animation-sequence-caller-root-drift.md)。
- 历史 M1-M6、测试阶段和性能交接的 13 条直接记录已无损迁入 [`04/2026-08-01-current-state-and-performance-handoffs.md`](04/2026-08-01-current-state-and-performance-handoffs.md)。
