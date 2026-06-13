# 02 · Sound 插件完善计划（基于 kira 的 Mixer / DSP / 3D Audio）

> 状态：工程化细化版 v2.1（执行核心裁决：**kira**） · 优先级：P1 · 前置：[01 插件架构核心](01-plugin-architecture-core.md) M1–M2
> 关联计划：`.codex/plans/Sound 插件核心完善计划.md` · 现状文档：`docs/zircon_plugins/sound/runtime.md`
> 参考实现：kira（执行核心，cargo 依赖）、Fyrox `fyrox-sound`（HRTF 算法参照）、Godot `servers/audio/effects`（缺口效果数值参照）

## 1. 目标与核心裁决

**裁决（v2.1）：音频执行核心采用 [kira](https://crates.io/crates/kira)（0.12.x，锁 minor），不再自研混音执行引擎。** 理由：自研栈（混音循环、音频线程资源管理、命令交付、参数平滑、解码）正是最容易产生大量隐蔽错误的部分，kira 已提供经生产验证的实现——AudioManager + cpal 后端、static/streaming 声音（symphonia 解码）、track 混音树（main/sub/send）、内置效果族、自定义 `Effect` trait、spatial track（listener/emitter/距离衰减）、`Tween` 参数平滑、`Modulator`、`Clock` 采样精确调度、mock backend 测试设施。

插件自身职责收敛为四块（**防止自成一套**）：

1. **契约映射层**：`framework::sound` DTO（mixer graph / 效果 / 播放控制）→ kira track/send/effect/handle 的编译与同步；
2. **ECS 集成与 3D 策略**：`sound.spatial_update` 系统、衰减/空气吸收/occlusion/多普勒的参数计算（喂给 kira，不重写混音）；
3. **kira 缺口补齐**：kira 没有的效果（Flanger/Phaser/Chorus/WaveShaper/ConvolutionReverb/Limiter）与 HRTF，以 kira `Effect` trait 自定义实现——**算法直接迁移现有已验证的 `engine/dsp` 数值代码，金样测试随迁**；
4. **资产管线与自动化**：audio_importer 对接、timeline 曲线 → `Tween`/`Clock` 映射、dynamic events。

## 2. 现状基线（实查）与退役清单

现状（v2 实查结论保持）：自研栈功能面完整——`engine/render/`（混音编排）、`engine/dsp/`（12 种效果）、`engine/hrtf/` + `engine/source_environment/`（空间化/HRTF/occlusion）、`output/cpal/`（producer 线程 + ring buffer）、timeline/automation/dynamic_events/descriptor_validation；但热路径每 block 重验证+排序、每效果 `to_vec()` 分配、`try_lock Mutex` 线程模型（v2 曾计划自研重构修复，**现裁决整体由 kira 替代**）。

**退役清单（硬切删除，由 kira 对应能力替代）**：

| 自研模块                                                                            | kira 替代                                                          |
| ----------------------------------------------------------------------------------- | ------------------------------------------------------------------ |
| `engine/render/**`（orchestration/routing/playback/sampling/source）              | track 树求值 + send 路由 + 声音播放/重采样                         |
| `engine/dsp_state/**`、`engine/dsp/effects/chain.rs`                            | track effect 链（kira 音频线程内管理效果状态）                     |
| `output/cpal/**`（producer/callback/ring_buffer/session）、`output/software.rs` | `AudioManager<DefaultBackend>`（内置 cpal）；测试用 mock backend |
| `engine/validation/ordering.rs` 等图执行期验证                                    | 图合法性验证保留在**配置编译期**（见 §3.2），执行期交 kira  |
| 直接 `cpal = "0.15"` 依赖                                                         | 移除，由 kira 携带                                                 |

**保留并改造**：`framework::sound` 契约层（不动）、`engine/dsp/` 中的效果**算法数值**（迁移为 kira 自定义 Effect）、`engine/hrtf/` 与 `engine/source_environment/spatial/` 的**策略计算**（迁移到 ECS 系统侧）、`mixer_configuration/`、`automation/`、`dynamic_events/`、`descriptor_validation/`、`timeline/`、`components.rs`。

**范围裁决（kira 约束，写明不回避）**：kira 的 `Frame` 为立体声——**5.1/7.1 环绕声输出退出 v1 范围**（移入后续池：等 kira 上游多声道支持，或届时以自定义 `Backend` 实现下混输出）；多声道**源资产**仍支持（解码期下混 stereo）。现有 `engine/render/channel_layout/` 的 5.1 下混测试矩阵随自研渲染退役归档。双耳 HRTF 不受影响（stereo 输出，§3.4）。

## 3. 架构设计

### 3.1 KiraEngine 生命周期（`runtime/src/kira_bridge/manager.rs` [新增]）

```rust
pub struct KiraEngine {
    manager: kira::AudioManager<kira::DefaultBackend>,   // 内置 cpal，音频线程归 kira 管理
    tracks: TrackBindings,        // SoundTrackId → kira TrackHandle/SendTrackHandle（dense）
    playbacks: PlaybackBindings,  // PlaybackId → kira 声音 handle
}
```

- `AudioManager` 在插件 `activate` 创建、`deactivate` drop（kira 自行关停音频线程）；设备枚举/选择经 `output/lifecycle/`（保留模块，内部改调 kira backend 设置）。
- 线程模型：**不再自建任何音频线程**；主线程对 kira 的全部操作经其 handle API（kira 内部为无锁命令通道）。

### 3.2 Mixer Graph 映射编译（`runtime/src/kira_bridge/graph_compile.rs` [新增]）

- `SoundGraph` DTO（契约现类型）→ 编译期验证（保留 `engine/validation/` 的引用/环/track 校验，移至此处）→ kira track 树构建计划：Track → `TrackBuilder`（含效果链）、Send → kira send track、master → main track。
- 图变更（编辑器/运行期 CRUD）→ diff 计算 → 最小 kira 操作序列（新建/挪移/参数 Tween）；**不做整树重建**（避免爆音），不可 diff 的结构变更（如环路重排）整树重建并交叉淡化。
- 效果参数热更：`SoundEffectKind` 参数 → 各 kira effect handle 的 set 调用，平滑一律走 `Tween`（默认 10ms 线性），消除 zipper noise——自研 `SmoothedParam` 方案废弃。

### 3.3 效果映射表（`runtime/src/kira_bridge/effect_map.rs` [新增] + `effects_custom/` [新增]）

契约 `SoundEffectKind`（12 种，现有）→ kira 对位：

| SoundEffectKind                  | 实现                                                   | 说明                                                                                      |
| -------------------------------- | ------------------------------------------------------ | ----------------------------------------------------------------------------------------- |
| Gain                             | kira `VolumeControl`                                 | 内置                                                                                      |
| PanStereo                        | kira `PanningControl`                                | 内置                                                                                      |
| Filter（LP/HP/BP/Notch）         | kira `Filter`                                        | 内置                                                                                      |
| ParametricEq（v2 新增 DTO 保留） | kira `EqFilter` ×N 串联                             | 内置（bell/low-shelf/high-shelf）                                                         |
| Compressor                       | kira `Compressor`                                    | 内置（sidechain 不在 kira 内置范围，sidechain 字段在编译期报 Unsupported 诊断，进后续池） |
| Delay                            | kira `Delay`                                         | 内置                                                                                      |
| Reverb                           | kira `Reverb`                                        | 内置                                                                                      |
| Limiter                          | 自定义 Effect [新增 `effects_custom/limiter.rs`]     | 迁移 `engine/dsp/dynamics.rs` 现数值                                                    |
| WaveShaper                       | kira `Distortion` 对位评估；不满足则自定义           | 数值对照后裁决，金样判定                                                                  |
| Flanger / Phaser / Chorus        | 自定义 Effect [新增 `effects_custom/modulation.rs`]  | 迁移 `engine/dsp/modulation.rs` 现数值                                                  |
| ConvolutionReverb                | 自定义 Effect [新增 `effects_custom/convolution.rs`] | 迁移 `engine/dsp/effects/apply/convolution.rs`；IR 资产管线不变                         |

- 自定义效果实现 kira 的 `Effect`/`EffectBuilder` trait（在 kira 音频线程内执行，**实现体禁止分配/锁**——kira 的 trait 契约即如此，迁移时保留现有零状态分配写法）；现有 deterministic 金样测试（`tests/dsp_state/deterministic/*`）随算法迁移改挂自定义 Effect 单测，期望值不变。

### 3.4 3D 空间化策略（`runtime/src/spatial/` [新增，迁移自 engine/source_environment]）

```
sound.spatial_update (PostUpdate, after animation.evaluate)   ← 01 锚点不变
  ├─ listener/source transform+速度收集（ECS 查询）
  ├─ 策略计算（我们侧）：距离衰减曲线 → 目标音量
  │                    空气吸收 → 目标 LPF cutoff
  │                    occlusion（WeakBridge<PhysicsQueryInterface>，11 计划）→ 目标增益/滤波
  │                    多普勒 → 目标 playback_rate
  └─ 输出：对 kira handle 的 set + Tween（帧间平滑）
```

- 双耳/声像：默认走 kira spatial track（listener/emitter，内置距离衰减与立体声 panning，简单场景零成本接入）；启用高级策略（自定义衰减曲线/occlusion/空气吸收）时改用普通 track + 上述参数驱动——两档由 `SoundSourceDescriptor.spatial` 配置决定，编译期选定。
- HRTF（`sound.hrtf` 选项）：自定义 kira `Effect`（`effects_custom/hrtf.rs` [新增]），分段卷积算法迁移自 `engine/hrtf/`（数值参照 `dev/Fyrox/fyrox-sound/src/` 校核）；SOFA 数据集经 audio_importer 资产管线（维持 v2 决策）。
- AudioVolume（区域混响/滤波影响）：保留组件与优先级求解（`engine/source_environment/volume/` 策略部分迁移），输出为对所属 track 效果参数的驱动。

### 3.5 ECS 集成与事件（对接 01/11，维持 v2 设计）

- `register_native_system::<…>("sound.spatial_update", SystemStage::PostUpdate, …).after("animation.evaluate")`；
- `register_resource::<SoundDeviceState>`；`register_event::<SoundPlaybackEvent>`（started/finished 由 kira handle 状态轮询产生——在 spatial_update 内统一收割，underrun 指标改取 kira/cpal 诊断）；
- AudioSource/AudioListener/AudioVolume 静态类型组件路径不变；physics occlusion 经 [11 调用桥](11-plugin-call-bridge.md) `WeakBridge` 弱依赖。

### 3.6 Timeline 自动化与动态事件

- timeline 曲线 → 关键帧段编译为 kira `Tween` 序列 + `Clock`（采样精确起止）调度；周期调制（LFO 类）→ kira `Modulator` 绑定；`automation/target/effect/*` 的绑定目标表保留，落点从自研参数改为 kira handle set 调用。
- `dynamic_events/`（Impact/Marker/AmbientStinger）与 `dynamic_event_abi/` 保持，执行端改调 kira 播放 API。

## 4. 模块文件树

```
zircon_plugins/sound/runtime/
  Cargo.toml                         [改造] +kira = "0.10"（锁 minor）；-cpal 直接依赖
  src/kira_bridge/manager.rs         [新增] KiraEngine/生命周期/绑定表
  src/kira_bridge/graph_compile.rs   [新增] SoundGraph → track 树编译 + diff 同步
  src/kira_bridge/effect_map.rs      [新增] SoundEffectKind → kira effect 映射
  src/effects_custom/{limiter,modulation,convolution,hrtf}.rs  [新增] kira Effect 实现（算法迁移）
  src/spatial/{mod,attenuation,occlusion,doppler,volume}.rs    [新增] 策略计算（迁移自 engine/source_environment）
  src/engine/render/**、engine/dsp_state/**、output/cpal/**、output/software.rs  [删除]
  src/engine/dsp/**                  [删除]（数值迁入 effects_custom 后）
  src/automation/**、timeline/**、dynamic_events/**、mixer_configuration/**、descriptor_validation/**  [改造] 落点改 kira handle
  src/runtime_plugin/registration.rs [改造] register_system/resource/event（维持 v2 设计）
```

## 5. 里程碑与任务分解

### M1 kira 接入与 Mixer Graph 映射（替换自研执行栈）

| 任务  | 内容                                                                | 改动文件                                             | 依赖  | 新增测试                                                               |
| ----- | ------------------------------------------------------------------- | ---------------------------------------------------- | ----- | ---------------------------------------------------------------------- |
| M1-T1 | kira 依赖引入 + KiraEngine 生命周期（activate/deactivate/设备选择） | Cargo.toml、kira_bridge/manager.rs、output/lifecycle | —    | `engine_starts_and_stops_with_mock_backend`                          |
| M1-T2 | SoundGraph → track/send 编译 + 验证迁移                            | graph_compile.rs                                     | M1-T1 | `graph_compiles_to_expected_track_tree`、既有 graph 验证测试迁移保绿 |
| M1-T3 | 播放控制映射（play/stop/seek/pause + PlaybackId 绑定）              | manager.rs                                           | M1-T2 | `playback_lifecycle_round_trips_through_kira`                        |
| M1-T4 | 图 diff 同步 + 参数 Tween 热更；删除自研渲染/输出栈                 | graph_compile.rs、删除清单                           | M1-T3 | `graph_edit_applies_minimal_diff`、`param_change_is_tweened`       |

### M2 效果映射与缺口效果迁移

| 任务  | 内容                                                                | 改动文件                        | 依赖  | 新增测试                                        |
| ----- | ------------------------------------------------------------------- | ------------------------------- | ----- | ----------------------------------------------- |
| M2-T1 | 内置效果映射（Gain/Pan/Filter/Eq/Compressor/Delay/Reverb）          | effect_map.rs                   | M1    | `builtin_effect_params_round_trip` 矩阵       |
| M2-T2 | 自定义 Effect：Limiter/Flanger/Phaser/Chorus（数值迁移 + 金样随迁） | effects_custom/*                | M1    | 既有 deterministic 金样改挂新实现全量保绿       |
| M2-T3 | ConvolutionReverb 自定义 Effect + IR 资产                           | effects_custom/convolution.rs   | M2-T2 | `convolution_impulse_response_matches_golden` |
| M2-T4 | WaveShaper 对位裁决（Distortion vs 自定义，金样判定）               | effect_map.rs 或 effects_custom | M2-T2 | 金样比对报告                                    |

### M3 ECS 与 3D 空间化策略

| 任务  | 内容                                                         | 改动文件                  | 依赖         | 新增测试                                                                             |
| ----- | ------------------------------------------------------------ | ------------------------- | ------------ | ------------------------------------------------------------------------------------ |
| M3-T1 | sound.spatial_update 系统 + resource/event 注册              | registration.rs、spatial/ | 01-M2、M1    | `spatial_update_registered_after_animation_evaluate`                               |
| M3-T2 | 衰减/空气吸收/多普勒策略 → kira 参数驱动                    | spatial/*                 | M3-T1        | `attenuation_curve_drives_volume_within_tolerance`、`doppler_sets_playback_rate` |
| M3-T3 | occlusion 经 WeakBridge（physics 弱依赖 + AudioVolume 退化） | spatial/occlusion.rs      | 11-M1、M3-T2 | `occlusion_falls_back_without_physics`                                             |
| M3-T4 | AudioVolume 优先级与区域效果驱动                             | spatial/volume.rs         | M3-T2        | 既有 volumes 优先级测试迁移保绿                                                      |

### M4 HRTF 与资产管线

| 任务  | 内容                                            | 改动文件                     | 依赖  | 新增测试                                     |
| ----- | ----------------------------------------------- | ---------------------------- | ----- | -------------------------------------------- |
| M4-T1 | HRTF 自定义 Effect（分段卷积迁移）              | effects_custom/hrtf.rs       | M2    | 既有 hrtf deterministic kernels 金样改挂保绿 |
| M4-T2 | SOFA 数据集资产管线 + sound.hrtf 选项接线       | audio_importer、registration | M4-T1 | `hrtf_profile_loads_from_asset_pipeline`   |
| M4-T3 | 多声道源资产解码下混策略（stereo 输出裁决落地） | 资产解码配置                 | M1    | `surround_source_downmixes_to_stereo`      |

### M5 Timeline 自动化与 Editor

| 任务  | 内容                                                              | 改动文件                                   | 依赖                                 | 新增测试                                                                                      |
| ----- | ----------------------------------------------------------------- | ------------------------------------------ | ------------------------------------ | --------------------------------------------------------------------------------------------- |
| M5-T1 | timeline 曲线 → Tween/Clock 编译；Modulator 绑定                 | automation/、timeline/                     | M2                                   | `automation_curve_compiles_to_tween_sequence`、`clock_scheduled_start_is_sample_accurate` |
| M5-T2 | Mixer Console 面板实化（电平表取 kira track 监测、send 矩阵编辑） | sound/editor、`mixer_console.v2.ui.toml` | M1、[10 规范](10-editor-integration.md) | editor 契约测试                                                                               |
| M5-T3 | View/Debug Overlays/Sound：AudioVolume/衰减球 overlay             | sound/editor                               | 10-规范                              | overlay 注册快照测试                                                                          |

## 6. 验收命令

```bash
cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sound_runtime --locked
cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sound_editor --locked
```

## 7. 风险

- **kira 版本演进**：0.9→0.10 曾有破坏性重构（spatial 模型重做）；锁 minor + 升级单列任务，`kira_bridge/` 作为唯一接触面隔离升级冲击（引擎其余部分零 kira 类型泄漏——契约层不变保证了这一点）。
- **stereo 限制**：5.1/7.1 输出退出 v1 已在 §2 裁决并写明；若项目后期硬需求出现，路径是自定义 kira `Backend`（届时单独立项，不回退自研混音）。
- **金样迁移**：整体混音金样（自研 render 路径）退役；效果级金样随算法迁移保留——M2 期间新旧实现并存对照跑完才删旧码。
- **sidechain compressor**：kira 内置不支持，编译期 Unsupported 诊断 + 后续池（自定义 Effect 可实现但需 kira route 旁路，等真实需求）。

## 8. 附录 · dev/依赖 参考源码对位

实现各任务前**必须先读对应参考实现**，禁止凭空实现：

| 设计点                   | 参考源码                                                                                                                                      | 看什么                                                                                                                                                                                         |
| ------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| kira 全 API 面（最重要） | kira crate 源码（`cargo vendor` 或 docs.rs/kira；examples 目录）                                                                            | AudioManager/TrackBuilder/send/effect handle 形态、`Effect`/`EffectBuilder` trait 契约（音频线程约束）、Tween/Clock/Modulator、spatial track、mock backend 测试写法——M1–M5 全程第一参考 |
| 缺口效果数值（迁移校核） | 仓内现有 `engine/dsp/{dynamics,modulation}.rs`、`effects/apply/convolution.rs` + `dev/godot/servers/audio/effects/`（同类效果第二参照） | 迁移时数值逐项对照金样；Godot 实现用于交叉验证系数公式                                                                                                                                         |
| HRTF 分段卷积/ITD        | 仓内现有 `engine/hrtf/` + `dev/Fyrox/fyrox-sound/src/`                                                                                    | kernel 插值与 tail 处理、与 kira Effect 帧驱动模型的适配                                                                                                                                       |
| ECS 组件建模             | `dev/bevy/crates/bevy_audio/`（基于 rodio，仅取 ECS 组件/系统挂点形态，不参考其后端）                                                       | 组件与 transform 同步的系统形态；kira 的 ECS 封装先例为社区 crate bevy_kira_audio（不在 dev 内，必要时 cargo vendor 查阅）                                                                     |
