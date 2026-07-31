# 04 · Animation 插件完善计划（骨骼 / Clips / 状态机 / GPU Skinning / Avatar Mask）

> 状态：工程化细化版 v2 · 优先级：P1 · 前置：[01 插件架构核心](01-plugin-architecture-core.md) M1
> 关联计划：`.codex/plans/Physics + Full Animation Support 新计划.md` · 现状文档：`docs/zircon_plugins/animation/runtime.md`
> 参考实现：Bevy `bevy_animation`（AnimationGraph/AnimationTarget/mask 位掩码）、Fyrox ABSM（多层状态机 + Layer Mask）、Unreal AnimGraph（蒙太奇/同步组仅作形态参考）
> 最新进度（2026-06-14 00:21 +08:00）：`runtime_physics_animation_tick_contract` 通过临时 manifest 与外部 target-dir 复跑，16 项全通过。timeline event 相关测试已在 tick 前显式连接 `EventSubscription<AnimationClipEvent>`，与 [11](11-plugin-call-bridge.md) M2 dormant event channel 语义一致：无订阅者不写缓冲，有订阅者时 clip/graph/state-machine/transition 事件均可观测。

## 1. 目标

把 `zircon_plugins/animation` 推进到完整动画系统：骨骼 clip 采样与混合、动画图（blend tree）、多层状态机、avatar mask、GPU skinning、timeline tracks 闭环、动画事件，并为 physics ragdoll 与 sound 自动化提供稳定姿态/事件通道。

## 2. 现状基线（实查）

成熟度 Beta / Partial。

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

```bash
cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_animation_runtime --locked
cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_animation_graph_editor --locked
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

- M1：M1-T1/T2 的正式 WSL Cargo 已通过 target table 10/10、weighted SoA/PosePool 4/4，生产 clip/graph/state-machine 姿态采样已接入 revision-aware compiled evaluator。M1-T3 已硬切 folder-backed 五阶段 pipeline；graph topology、parameter、skeleton mask 与 state/transition condition 已编译为 dense slot/row 并接入生产有界 revision cache。最终 `AnimationPoseOutput`/LevelSystem handoff 现按稳定 entity/topology 原位复用 bone Vec 与 bone-name String 容量，graph base blend 也改为消费首个 owned pose，避免首姿态整包 clone。Windows nightly locked/offline allocation contract 2/2 通过，证明 PosePool 256 次稳定循环和 final-pose copy 均为零分配。M1 实现闭环；post-hard-cut Windows/WSL 全套仍是最终里程碑门。具体事实见 [Plugins 04 M1 编号产出记录](04/2026-07-10-animation-m1-output-records.md)。
- 当前状态锚点：`plugins_04_m1_final_pose_reuse_allocation_2_of_2_passed`；M1-T2 正式测试锚点：`plugins_04_m1_t2_weighted_pose_formal_cargo_4_of_4_passed`。
- M2：M2-T1 已实现 `.avatar_mask.toml` typed 解析与 skeleton-scoped dense `MaskWeights`，subtree 继承/覆写和边界渐变 focused 合同 2/2 通过；M2-T2 已实现 layers × masks × override/additive 的 dense `PoseBuffer` 分层混合，focused 4/4 与全 tests type-check 通过。M2-T3 已按 03/04 对偶计划新增 Physics 中立 `SkeletalPoseTargets`/`SimulatedPoseFeed`，由 Physics 注册双资源，Animation 在 `animation.evaluate` 发布局部骨骼目标姿态；定向桥 1/1、Animation 全量 78/78、Physics 默认全量 43/43 通过。Animation 侧已在最终 layer blend 后、IK 前按 mode × mask × interpolation alpha 消费 feed。Physics 侧已完成严格 `.ragdoll.toml` profile、拓扑化 body/collider/Generic6Dof 生成、Animated/Simulated/Blended 三模式、Animated→物理无跳变速度继承，以及 `physics.sync_to_scene` parent-local feed writer。计划精确命名的 `ragdoll_drop_golden_snapshot` 与 profile/rollback/mode/blend 合同在 Windows nightly locked/offline 聚焦复跑 5/5 通过；生产模块 `cargo check --lib` 先行通过。M2-T3 尚待 Animation 跨插件生产 Tick 场景取得 executable 后最终关闭。当前增量锚点 `plugins_04_m2_t3_physics_ragdoll_focused_5_of_5_passed`。
- M3：M3-T1 的 compiled state/Condition-AND-OR-NOT 已完成；M3-T2 的 duration/crossfade、normalized exit-time gate 和四种 interruption policy 已从资产进入 compiled representation 与生产路径。M3-T3 的五种 StateKind 均已闭环；SubMachine 实例键加入父状态 owner，并允许父 machine 从嵌套状态迁出。多层已硬切为唯一 `AnimationStateMachineAsset.layers`，current/v3/v2/v1 迁移、binary roundtrip、dense layer compile 与生产 `PoseLayer` stack 均已接线；每层复用主状态机 interruption policy、已混合源姿态连续性与独立事件时间窗，完成/退出时清理 interruption source。Layer 骨数/骨名、pose transform 与 mask/blend shape 错误不再静默丢层，而是发布 typed `AnimationStateMachineLayerDiagnostic`。layer 合同 4/4、Windows 完整 `cargo check --tests` 通过；最新完整 production Tick executable 34/34 中，masked layer、SubMachine parent-transition 与 layer-interruption continuity 均通过。M3 实现与 Windows 定向行为已闭环，post-hard-cut Windows/WSL 全套仍是最终里程碑门。当前增量锚点 `plugins_04_m3_production_tick_34_of_34_passed`；详见 [Plugins 04 M3 编号产出记录](04/2026-07-11-animation-m3-output-records.md)。
- M4：M4-T1/T2 已完成 256-joint `SkinningPalette`、readiness 判定、CPU fallback 及诊断，focused 4/4 通过。M4-T3 已将 Render ABI 从旧 uniform 硬切为 group3 bindings 3/4 read-only storage，并按 stable instance 实现两个 `STORAGE | COPY_DST` WGPU buffer 的持久双槽：仅成功 submit 后交换，第三帧复用第一槽；source 未就绪但 palette 合法的 CPU fallback 帧仍会 stage/commit 当帧 palette（仅不绑定 draw），保证下一帧恢复 GPU 时 previous buffer 不陈旧；palette 本身缺失时则丢弃失败帧暂存槽。fallback 边界修正后的 Windows Runtime nightly locked/offline production lib check 已通过（3 分 06 秒）；无 GPU slots 3/3、真实 WGPU `Arc` 双槽/CPU fallback previous 连续性 1/1（7611 filtered，且无 adapter skip）、Naga 生产 shader 拼接验证均通过；真实 WGPU 1000 实例/2000 buffers/32,800,000 bytes 基准为 123,884 µs。结构/审查聚焦门已通过：生产 owner 最大 766 行、root/mod 只做接线、storage 固定 ABI 生产路径无 `expect/panic`，独立类型编译通过。通过后的协调器复跑因共享资产迁移再次变化，被 `migrate_legacy_persisted_asset_reference_with` 导出漂移与 `AssetReference::guid()` 测试漂移抢先阻断，未运行 M4 测试；不覆盖此前实际 1/1 结果。当前锚点 `plugins_04_m4_wgpu_palette_cpu_fallback_1_of_1_passed`；具体事实见 [Plugins 04 M4 编号产出记录](04/2026-07-11-animation-m4-output-records.md)。
- M5：M5-T1 已补齐 Runtime 中立 `AnimationIkCommand`、逐 World 有界 Manager 队列、稳定 `AnimationTargetId` → skeleton-scoped dense `TargetSlot` 编译，以及 `PoseBlend/StateMachine Layer → IK → SkeletalPoseTargets/PoseApply` 生产后处理顺序；TwoBoneIK 与 LookAt 使用 skeleton model-space target，失败通过 `AnimationIkDiagnostic` typed event 上报。低内存独立 target 的 Windows nightly locked/offline 行为复验已取得数学 3 条 + Manager 队列 1 条共 4/4 绿灯；最新完整 production Tick executable 34/34 中，两条真实 TwoBone/LookAt 场景均通过。M5-T2 已正式注册 `AnimationClipEvent` 与 `animation.events.clip` catalog，注册 focused 1/1 通过。M5 实现与 Windows 定向行为已闭环，post-hard-cut Windows/WSL 全套仍是最终里程碑门。当前增量锚点 `plugins_04_m5_production_tick_34_of_34_passed`；具体事实见 [Plugins 04 M5 编号产出记录](04/2026-07-11-animation-m5-output-records.md)。
- M6：Animation Graph 已闭环 graph/state-machine editor、node palette 与 BlendSpace1D/2D graph node；Timeline Sequence 已闭环 sequence editor 与 transform/component-property/event-marker 三类 track。通用 Animation Editor 现独占 BlendSpace1D、BlendSpace2D、AvatarMask 三类资产 drawer，graph editor 不再反向拥有通用动画资产。Windows nightly locked/offline 三包实际 unit test 为 Animation Editor 2/2、Animation Graph Editor 9/9、Timeline Sequence Editor 9/9，合计 20/20；M6-T1/T2/T3 实现及 Windows 定向验收闭环，最终仅保留 Plugins 04 跨平台全套门。当前锚点 `plugins_04_m6_editor_packages_windows_20_of_20_passed`；具体事实见 [Plugins 04 M6 编号产出记录](04/2026-07-11-animation-m6-output-records.md)。
- 正式测试阶段：hard-cut 前 Animation runtime Windows/WSL nightly locked/offline 全套均 75/75 通过；transition 与姿态桥接入后，Windows nightly/offline 全套更新为 78/78，Physics 默认全量 43/43 同步通过。post-hard-cut Windows 与 WSL 现均在各自专属 D: target 执行相同的 16 个 executable，分别 101/101 通过、0 failed；跨平台最终锚点更新为 `plugins_04_post_hard_cut_cross_platform_101_of_101_passed`。WSL 日志保存在 `D:\cargo-targets\plugins04-post-hardcut-wsl-20260712\wsl-full-abcdc380a79c498885c5c9f113a2d0d0.{out,err}`。早期事实见 [Plugins 04 正式测试阶段记录](04/2026-07-11-animation-testing-stage.md)，最终门见 [Plugins 04 post-hard-cut 产出记录](04/2026-07-12-animation-post-hard-cut-output-records.md)。M1-M6 实现及 post-hard-cut 跨平台测试门已闭环，进入里程碑 closeout。
- 2026-07-18 Runtime skin/morph性能交接：render submission当前会为每instance/frame线性查pose，重建bone-name表、bind world/inverse bind并在GPU palette路径前CPU skin/clone全部primitive；morph静态delta和current/previous weights也全量重建上传。Plugins04须发布compiled skeleton dense index/parent topo/inverse-bind与per-instance pose/weight generation，只让Render03准备palette/dirty weight slots；CPU skin仅能力/关节上限fallback并进入有界worker。见PERF-MVP-385/386及`docs/plans/performance/01/2026-07-18-graphics-build-mesh-draws-root-static-review.md`。
- 2026-07-18 palette ABI补充交接：Runtime storage固定256×mat4约16 KiB，64骨也初始化/复制/上传全块；current+previous×1k实例约32.8 MiB。Plugins04发布的pose generation必须携带active joint count/dirty identity，不能要求render把ABI最大容量当有效payload；与Render03共同验收stable bytes=0、changed bytes近active bones。见PERF-MVP-386及mesh root静态证据。
- 2026-07-18 GPUScene animation history交接：current/previous palette、skinned source与morph weights当前分散在多张HashMap，每成功帧全量扫描/clone，weights还深拷贝。Plugins04须发布pose/weight generation与active dirty ranges，Render03以dense slot/buffer epoch翻转历史；stable history scan/clone/upload=0，changed近dirty joints/weights，失败帧保持committed epoch。见PERF-MVP-405及GPUScene静态证据。
- 2026-07-22 property binding热路径交接：Runtime/Plugins04 sequence target fallback仍按binding全entity×ancestor×同名扫描，apply每track/frame让World重新规范化component/segments并字符串分派；segment compare临时String已止损但根因未解。Plugins04在clip compile产物持Runtime08发布的generation-bound dense entity/property accessor，stable frame禁止字符串resolve/normalized alloc，generation mismatch只增量rebind；见PERF-MVP-329与Runtime08 `08/failure-2026-07-22-scene-property-path-compiled-dispatch.md`。
- 2026-07-30 framework animation性能交接：PERF-MVP-581要求binary writer借用payload、current格式header-first单次schema dispatch，删除source/DTO/document/bytes多owner和兼容解码多遍扫描；PERF-MVP-582要求sequence target/property、avatar mask和runtime status复用现有compiled generation，stable editor/debug不得重复track/String/full snapshot clone。规模与current/legacy save-load门见`docs/plans/performance/01/2026-07-30-runtime-framework-animation-ai-navigation-tasks-static-review.md`。
