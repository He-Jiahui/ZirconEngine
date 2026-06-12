# 02 · Sound 插件完善计划（Mixer / DSP / 3D Audio / Multichannel）

> 状态：工程化细化版 v2 · 优先级：P1 · 前置：[01 插件架构核心](01-plugin-architecture-core.md) M1–M2
> 关联计划：`.codex/plans/Sound 插件核心完善计划.md` · 现状文档：`docs/zircon_plugins/sound/runtime.md`
> 参考实现：Unreal Submix/SoundClass、Godot `servers/audio`（bus + effect 插槽）、Fyrox `fyrox-sound`（bus graph / HRTF）

## 1. 目标

把 `zircon_plugins/sound` 推进到可发行品质的完整音频引擎。**实查修正**：功能面（mixer 执行、12 种 DSP 效果、HRTF、occlusion、多声道下混、timeline 自动化）已大部分实现且有深测试覆盖；本计划的主轴是**热路径性能架构重构（编译期图 + 零分配 + 无锁交付）**与四项真实功能缺口补齐，并接入 01 的 ECS 注册体系。

## 2. 现状基线（实查）

已实现（远超早期规划假设）：

- **契约层**：`zircon_runtime/src/core/framework/sound/effects.rs` 已定义 12 种效果 DTO（`SoundEffectKind`：Gain/Filter/Reverb/ConvolutionReverb/Compressor/WaveShaper/Flanger/Phaser/Chorus/Delay/PanStereo/Limiter）。
- **Mixer 执行引擎**：`engine/render/orchestration.rs` 的 `SoundEngineState::render_mix`（track 拓扑 → send/solo 路由 → 效果链 → master），路由在 `engine/render/routing/`，播放混音在 `engine/render/playback/`。
- **DSP**：`engine/dsp/`（gain/delay/dynamics/reverb/shaper/stereo/modulation/meter + effects/chain + sidechain），biquad 滤波在 `engine/filter/`（coefficients/shelf/state）。
- **3D**：`engine/source_environment/spatial/`（attenuation/cone/doppler/pan/profile）、`engine/occlusion/`（query/gain/ray_traced）、HRTF（`engine/hrtf/` + `engine/source_environment/hrtf/` loaded kernels/preview fallback/tail，含确定性测试）。
- **多声道**：`engine/render/channel_layout/`（downmix/positional/weights/mono/stereo/discrete），quad/5.1 下混测试矩阵完整（`tests/source_inputs/multichannel/`）。
- **输出**：`output/cpal/`（producer_thread 渲染 → `output/ring_buffer.rs` → cpal callback 消费）、`output/software.rs` 离线路径。
- **其余**：timeline（advance/playback/schedule）、automation 全目标树（`automation/target/effect/*` 逐效果绑定）、presets、dynamic_events、descriptor_validation。
- **注册**：`runtime_plugin/registration.rs` 经 `register_module(module_descriptor())`（Manager/Driver/Service 形态，`module.rs`），无 ECS 系统。

真实缺口（按严重度）：

| # | 缺口 | 证据 |
|---|------|------|
| S1 | 热路径违反"注册期重、运行期零开销"：`render_mix` 每 block 重跑 `validate_graph` + `track_render_order` 拓扑排序 + `sync_runtime_states`；`apply_track_effects` 每效果 `let dry = buffer.to_vec()` 堆分配；effect_states/track buffers 走 `HashMap` 查找 | `engine/render/orchestration.rs`、`engine/dsp/effects/chain.rs` |
| S2 | 线程模型：producer 线程 `try_lock Mutex<SoundEngineState>` + `Mutex<SoundConfig>`，主线程持锁时音频生产退避 sleep（underrun 风险、jitter 不可控） | `output/cpal/producer_thread.rs` |
| S3 | 无 ECS 系统锚点：listener/source transform 由模块 driver 轮询，无 `sound.spatial_update` 系统、无类型化事件 | `module.rs`、`runtime_plugin/registration.rs` |
| S4 | ParametricEq 多段缺失（`Filter` 仅单段 biquad/shelf）；track 级 `ChannelLayout` 声明与编译期混音矩阵节点未成体系（渲染按全图统一 `config.channel_count`） | `engine/filter/`、`engine/render/orchestration.rs` |
| S5 | Timeline 曲线 → 参数的逐帧插值链路存在但与渲染 block 粒度未对齐（zipper noise 风险） | `automation/`、`engine/render/sampling/` |

## 3. 架构设计

模块归属维持既定决策：中立契约在 `zircon_runtime::core::framework::sound`，全部 DSP/混音实现在 `zircon_plugins/sound/runtime`。

### 3.1 CompiledMixGraph 与零分配渲染（解决 S1，`engine/compiled_graph/` [新增]）

```rust
/// 控制线程编译产物；音频生产线程只读，零分配零哈希。
pub struct CompiledMixGraph {
    /// 拓扑序 track 槽位（dense，SoundTrackId → TrackSlot 在编译期解析完毕）
    pub track_order: Box<[TrackSlot]>,
    /// 每 track 的效果链：效果种类 + 预解析参数 + 状态槽位
    pub effect_chains: Box<[CompiledEffectChain]>,
    /// send 边（源槽位、目标槽位、pre/post-fader、增益）
    pub sends: Box<[CompiledSend]>,
    /// 编译期按图形状预分配的 block 缓冲池（含每效果 dry/wet 双缓冲）
    pub buffer_pool: BlockBufferPool,
    pub latency_frames: usize,           // 编译期由 latency_frames_for_graph 算定
}
#[derive(Clone, Copy)] pub struct TrackSlot(u32);

/// 交付协议：控制线程编译新图 → ArcSwap 原子替换；旧图由控制线程在
/// 生产线程确认切换（世代号 Acquire 读）后回收，音频侧永不 drop 大对象。
pub struct MixGraphHandoff { /* arc_swap::ArcSwap<CompiledMixGraph>, generation: AtomicU64 */ }
```

- 现有 `validate_graph` / `track_render_order` / `latency_frames_for_graph`（`engine/validation/`、`engine/render/runtime_state.rs`）整体移到编译函数 `compile_mix_graph(&SoundGraph) -> Result<CompiledMixGraph, SoundError>` 内，**渲染路径删除全部验证与排序调用**。
- `engine/dsp_state/`（effect_runtime/track_runtime/delay_line/history）从 `HashMap<SoundEffectStateKey, _>` 改为编译期分配的 dense 槽位数组 `Box<[SoundEffectRuntimeState]>`，索引由 `CompiledEffectChain` 携带；图变更时按 `(track, effect_id)` 键迁移旧状态（延迟线/混响尾不重置）。
- `apply_track_effects`（`engine/dsp/effects/chain.rs`）签名改为从 `BlockBufferPool` 借 dry 缓冲，**删除 `buffer.to_vec()`**；debug 构建挂分配计数断言（`#[cfg(debug_assertions)]` 全局分配器钩子，测试 `render_block_performs_zero_allocations`）。

### 3.2 线程模型与命令队列（解决 S2，`engine/command_queue.rs` [新增]）

```
主线程(ECS systems / Manager API)
   │  SoundCommand (SPSC 无锁环形队列, 预分配)
   ▼
音频生产线程 (producer_thread)  ──块渲染──▶ output/ring_buffer ──▶ cpal callback
   ▲  SoundFeedback (SPSC: 电平表/finished/underrun 计数)
   │
主线程 drain → 事件总线
```

```rust
pub enum SoundCommand {
    SwapGraph(/* 经 MixGraphHandoff，此处仅通知世代号 */),
    SetParam { target: CompiledParamTarget, value: f32 },   // 预解析槽位，非字符串
    Play(PlaybackRequest), Stop(PlaybackId), SeekTimeline { /* … */ },
    SpatialFrame(SpatialSnapshotBlock),  // listener/sources 位姿+速度，按帧一块
}
```

- `Mutex<SoundEngineState>` / `Mutex<SoundConfig>` 从 producer 路径**整体移除**：producer 拥有引擎状态独占所有权，主线程只经命令队列交互；`output/cpal/shared_state.rs` 保留 ring buffer 与原子计数。
- 参数平滑：`SetParam` 进入后在音频侧做 block 内线性斜坡（每参数 `SmoothedParam { current, target, step }`），消除 zipper noise（同时解决 S5 的粒度对齐——timeline 曲线在主线程按帧采样为 SetParam 流）。

### 3.3 DspEffect 统一 trait 与效果补齐（解决 S4 前半）

```rust
// engine/dsp/effect_trait.rs [新增]
pub trait DspEffect: Send {
    fn latency_samples(&self) -> usize { 0 }
    fn apply_params(&mut self, params: &SoundEffectKind);   // 复用契约 DTO，无新参数类型
    fn process(&mut self, block: &mut AudioBlockMut<'_>, ctx: &EffectContext<'_>);
}
pub struct EffectContext<'a> {
    pub sample_rate_hz: u32,
    pub channels: usize,
    pub sidechain: Option<&'a [f32]>,
    pub impulse_response: Option<&'a [f32]>,
    pub scratch: &'a mut BlockScratch,   // 池借调，替代 to_vec
}
```

- 现有 12 个效果的 `apply_*` 自由函数（`engine/dsp/effects/apply/`、`engine/dsp/*.rs`）逐个收编为 `DspEffect` 实现，行为以现有单测金样为回归基线，**删除自由函数路径**。
- 新增 `ParametricEq`（`engine/dsp/parametric_eq.rs` [新增]）：N 段 biquad 串联（复用 `engine/filter/coefficients.rs`），契约层加 `SoundEffectKind::ParametricEq(SoundParametricEqEffect { bands: Vec<SoundEqBand> })`（`framework/sound/effects.rs` [改造]，每段 {kind: Peak/LowShelf/HighShelf/LowPass/HighPass, freq_hz, q, gain_db}，上限 8 段）。
- 效果链 latency 补偿：编译期对并行 send 路径插入 `CompensationDelay`（复用 `engine/dsp_state/delay_line.rs`），对齐 `CompiledMixGraph.latency_frames`。

### 3.4 3D 空间化收口（`engine/spatial_stack.rs` [新增]）

现有四块实现重组为显式 stack（行为不变，结构归一）：

```rust
pub struct SpatializerStack {
    pub attenuation: AttenuationStage,   // 现 source_environment/spatial/attenuation.rs
    pub air_absorption: FilterStage,     // 现 volume/filter.rs 的 LPF 路径
    pub occlusion: Option<OcclusionStage>, // 现 engine/occlusion/*；physics 可选依赖
    pub panner: PannerStage,             // EqualPower | Vbap | Hrtf（现三套实现收编）
}
```

- occlusion 的 physics 依赖按 01 §3.4 在 `finish` 阶段经 `CapabilityView::has("runtime.capability.physics.raycast")` 探测；无 physics 时维持现有 AudioVolume 几何近似路径（`engine/occlusion/query.rs` 已有该退化）。
- 多普勒维持 `spatial/doppler.rs` 现实现，速度来自 §3.5 的 `SpatialFrame` 命令。

### 3.5 ECS 集成（解决 S3，对接 01 定稿 API）

- 注册（`runtime_plugin/registration.rs` [改造]，`RuntimePlugin::register`）：
  - `register_native_system::<…>(owner, "sound.spatial_update", SystemStage::PostUpdate, …)`，约束 `after("animation.evaluate")`（01 锚点表）；系统查询 listener/source/volume 组件 transform 与速度，打包 `SpatialSnapshotBlock` 入命令队列。
  - `register_resource::<SoundDeviceState>`（输出设备/布局协商结果）；
  - `register_event::<SoundPlaybackEvent>`（started/finished/underrun，由 feedback 队列 drain 产生）。
- AudioSource/AudioListener/AudioVolume（`components.rs`）迁移到静态类型组件路径（01 的 `register_component` native storage）。
- Module/Manager/Driver 形态保留为设备生命周期宿主（`output/lifecycle/`），但每帧轮询逻辑移入上述系统。

### 3.6 多声道协商（解决 S4 后半，`engine/channel_negotiation.rs` [新增]）

- 输出布局 ← CPAL 探测（`output/cpal/capability.rs` 已有能力查询）；track 可声明 `ChannelLayout`（契约 `SoundTrackDescriptor` 加 `layout: Option<SoundChannelLayout>` 字段）。
- 编译期（`compile_mix_graph` 内）在布局不匹配的边上插入上/下混矩阵节点（矩阵权重复用 `engine/render/channel_layout/weights.rs` 与 `downmix.rs`），**运行期无布局分支**。

## 4. 模块文件树

```
zircon_runtime/src/core/framework/sound/effects.rs        [改造] ParametricEq DTO
zircon_plugins/sound/runtime/src/
  engine/compiled_graph/mod.rs                             [新增] CompiledMixGraph/TrackSlot/编译入口
  engine/compiled_graph/compile.rs                         [新增] 验证+拓扑+槽位解析+缓冲池
  engine/compiled_graph/handoff.rs                         [新增] ArcSwap 交付与世代回收
  engine/command_queue.rs                                  [新增] SoundCommand/SoundFeedback SPSC
  engine/dsp/effect_trait.rs                               [新增] DspEffect/EffectContext
  engine/dsp/parametric_eq.rs                              [新增]
  engine/dsp/effects/chain.rs                              [改造] 池借调零分配
  engine/dsp_state/*.rs                                    [改造] HashMap → dense 槽位+状态迁移
  engine/render/orchestration.rs                           [改造] 渲染只消费 CompiledMixGraph
  engine/spatial_stack.rs                                  [新增] SpatializerStack 收口
  engine/channel_negotiation.rs                            [新增]
  output/cpal/producer_thread.rs                           [改造] 移除 Mutex，独占引擎状态
  runtime_plugin/registration.rs                           [改造] register_system/resource/event
  components.rs                                            [改造] 静态类型组件
```

## 5. 里程碑与任务分解

### M1 CompiledMixGraph 与零分配渲染

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M1-T1 | compile_mix_graph：验证/拓扑/槽位/缓冲池入编译期 | compiled_graph/* | — | `compiled_graph_matches_legacy_render_order`、离线渲染金样 `render_mix_golden_wav_parity` |
| M1-T2 | dsp_state dense 槽位化 + 图变更状态迁移 | dsp_state/* | M1-T1 | `effect_state_survives_graph_recompile` |
| M1-T3 | 效果链池借调，删 to_vec；分配断言 | dsp/effects/chain.rs | M1-T2 | `render_block_performs_zero_allocations` |
| M1-T4 | ArcSwap 交付 + 世代回收 | compiled_graph/handoff.rs | M1-T1 | `graph_swap_is_glitch_free_across_blocks` |

### M2 DspEffect 统一与效果补齐

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M2-T1 | DspEffect trait + 12 效果收编（金样回归） | effect_trait.rs、dsp/* | M1 | 现有 `tests/dsp_state/deterministic/*` 全量保绿 |
| M2-T2 | ParametricEq（契约 DTO + 实现 + 自动化绑定） | effects.rs、parametric_eq.rs、automation/target/effect/ | M2-T1 | `parametric_eq_band_response_matches_analytic` |
| M2-T3 | latency 补偿延迟节点编译期插入 | compiled_graph/compile.rs | M2-T1 | `parallel_send_paths_aligned_within_one_sample` |

### M3 线程模型与 ECS 集成

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M3-T1 | SoundCommand/SoundFeedback SPSC；producer 去 Mutex | command_queue.rs、producer_thread.rs | M1-T4 | `producer_never_blocks_on_main_thread_contention` |
| M3-T2 | SmoothedParam block 内斜坡 | command_queue.rs、render | M3-T1 | `param_step_produces_no_discontinuity` |
| M3-T3 | sound.spatial_update 系统 + resource/event 注册；组件静态化 | registration.rs、components.rs | 01-M2、M3-T1 | `spatial_update_registered_after_animation_evaluate`、`playback_finished_event_reaches_event_store` |

### M4 多声道协商与 HRTF 资产化

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M4-T1 | track 级 layout 字段 + 编译期混音矩阵节点 | framework/sound、channel_negotiation.rs | M1 | `mismatched_track_layout_gets_matrix_node`、既有 5.1 下混测试保绿 |
| M4-T2 | SOFA HRTF 数据集经 audio_importer 进资产管线（替换内嵌 kernel 路径） | engine/hrtf/*、zircon_plugins/audio_importer | — | `hrtf_profile_loads_from_asset_pipeline` |

### M5 Timeline 自动化闭环与 Editor

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M5-T1 | timeline 曲线 → SetParam 流（帧采样 + 音频侧斜坡） | automation/、timeline/ | M3-T2 | `automation_curve_playback_is_sample_accurate_at_block_boundaries` |
| M5-T2 | Mixer Console 面板实化：电平表（feedback 队列）、send 矩阵编辑 | zircon_plugins/sound/editor/、`mixer_console.v2.ui.toml` | M3-T1、[10 规范](10-editor-integration.md) | editor 契约测试 |
| M5-T3 | View/Debug Overlays/Sound：AudioVolume/衰减球 viewport overlay | sound/editor | 10-规范 | overlay 注册快照测试 |

## 6. 验收命令

```bash
cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sound_runtime --locked
cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sound_editor --locked
```

## 7. 风险

- M1 重构触及渲染全链路，现有 `tests/` 金样矩阵（multichannel/spatial/hrtf/dsp_state deterministic）是回归安全网——重构期间不得修改任何金样期望值；输出差异必须归零而非重录。
- dsp_state 状态迁移（M1-T2）若键策略不当会在图编辑时产生爆音；以 `(track, effect_id)` 稳定键迁移，新增效果零初始化。
- HRTF 数据集体积与许可：选用 MIT/CC0 SOFA 集，资产经 `audio_importer` 进入正常资产管线，不内嵌二进制进 crate。
