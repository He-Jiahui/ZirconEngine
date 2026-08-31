---
title: Runtime Audio / Sound 当前工作树设备、流式播放、混音图、空间声学、自动化复审与重构计划
category: zircon_runtime
report_id: Runtime168
review_date: 2026-08-30
baseline_head: working-tree
baseline_epoch: 2026-08-30
verification_head: working-tree
verification_epoch: 2026-08-30
supersedes_currentness_of:
  - docs/plans/optimize/zircon_runtime/99zn-runtime-audio-sound-clip-streaming-device-mixer-bus-effect-spatial-occlusion-reverb-timeline-event-voice-chat-editor-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/08b-audio-runtime-review.md
  - docs/plans/optimize/zircon_plugins/11-first-party-sound-source-runtime-editor-dist-catalog-mixer-spatial-reverb-timeline-product-integration-review.md
related_code:
  - zircon_runtime/src/core/framework/audio
  - zircon_runtime/src/core/framework/sound
  - zircon_runtime/src/asset/assets/sound.rs
  - zircon_runtime/src/asset/importer/ingest/import_sound.rs
  - zircon_plugins/sound/runtime
  - zircon_plugins/sound/dist
  - zircon_plugins/sound/features
  - zircon_plugins/audio_importer/runtime
  - zircon_plugins/asset_importers/audio
  - zircon_plugins/opus_importer
  - zircon_plugins/first_party_runtime_catalog
  - zircon_app/src/entry
plan_sources:
  - docs/plans/optimize/zircon_runtime/99zn-runtime-audio-sound-clip-streaming-device-mixer-bus-effect-spatial-occlusion-reverb-timeline-event-voice-chat-editor-product-integration-current-source-review.md
  - docs/plans/zircon_plugins/02-sound.md
  - docs/plans/zircon_plugins/02/failure-2026-07-19-kira-send-frame-capture-routing.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Public/AudioMixerDevice.h
  - dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Private/AudioMixerDevice.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Private/AudioMixerSourceManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Private/AudioMixerSourceManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AudioExtensions/Public/IAudioExtensionPlugin.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Sound/SoundWave.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/AudioDevice.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/AudioComponent.h
  - dev/godot/servers/audio/audio_server.h
  - dev/godot/servers/audio/audio_server.cpp
  - dev/godot/servers/audio/audio_stream.cpp
  - dev/godot/scene/audio/audio_stream_player.cpp
  - dev/godot/scene/3d/audio_stream_player_3d.cpp
  - dev/godot/editor/audio/audio_stream_editor_plugin.cpp
  - dev/godot/editor/audio/editor_audio_buses.cpp
  - dev/Fyrox/fyrox-sound/src/engine.rs
  - dev/Fyrox/fyrox-sound/src/context.rs
  - dev/Fyrox/fyrox-sound/src/bus.rs
  - dev/Fyrox/fyrox-sound/src/source.rs
  - dev/Fyrox/fyrox-sound/src/buffer/streaming.rs
  - dev/Fyrox/fyrox-sound/src/renderer/hrtf.rs
  - dev/Fyrox/fyrox-sound/src/effects/mod.rs
  - dev/bevy/crates/bevy_audio/src/lib.rs
  - dev/bevy/crates/bevy_audio/src/audio_output.rs
  - dev/bevy/crates/bevy_audio/src/sinks.rs
  - dev/bevy/crates/bevy_audio/src/audio.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/package.json
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
tooling_scope: excluded_by_user_request
---

# Runtime168 · Audio / Sound 当前工作树复审

## 1. 结论

Sound 已有相当多的合同、校验和测试，不应再按“没有代码”处理；但它仍不是可资格化的工程级音频系统。当前形态是 Kira/CPAL 静态 PCM 播放器、一个由 `Mutex<SoundEngineState>` 承载的逻辑注册表、部分混音图编译器、尚未接入生产渲染调用链的空间/DSP函数，以及只发布声明的可选 feature。它能证明 API 形状，却不能证明实时线程、资源预算、设备恢复、流式长音频、空间误差、编辑器事务或跨世界生命周期。

当前没有证据支持“性能和表现优于 Unreal”。已有 release-only benchmark 仅测 Vec/图快照或 timeline 临时分配，未测 callback deadline、XRUN、stream underrun、voice stealing、设备热插拔、长时内存、HRTF/卷积误差和多世界并发。

## 2. 复审边界与 currentness

| 范围 | 文件 | 物理行 | bytes | tests | ignored | fingerprint |
|---|---:|---:|---:|---:|---:|---|
| Runtime framework、Sound asset/import | 34 | 3,143 | 101,164 | 19 | 2 | `1bd784e26fcfa5a07e439e6def0e8af0cee4463c402529df9e575b97b6c69459` |
| Sound runtime/dist/features 全量 | 1,290 | 26,546 | 916,629 | 431 | 8 | `74e7cb957a9f034bc8e42bdf18a6166bb4e4412a5ba19b34bd2c6b8103614fa5` |
| Audio importer、legacy audio、Opus importer | 17 | 2,176 | 79,515 | 38 | 3 | `3d33ad9823ff3934e3c842e2817b7978de59c5be04066042499823414cd5e606` |
| Sound editor 与 ZUI、feature editor | 14 | 1,386 | 52,892 | 8 | 0 | `b4c69c33a0f66c16be3ab4644cae82c206aecda449d23f7aeea7f04d72f3aa03` |
| 五引擎参考选择集 | 30 | 33,809 | 1,232,600 | 6 | 0 | `835efb2b8f7cf66ab655b14bc046a71eb4c2324c3b578ee0bcbc09d5caebcfa8` |

工作树中 Sound runtime、Sound editor、audio importer 和 `SoundAsset` 均存在用户/会话修改；因此本报告把旧 99zn 作为历史方向，不把旧 commit 的通过状态当作当前事实。

## 3. 生产级差异

### 3.1 运行时边界与线程模型

1. `SoundManager` 只是十个 trait 的空组合（`zircon_runtime/src/core/framework/sound/manager.rs:23-36`）；`DefaultSoundManager` 同时拥有配置锁和一个包含 Kira、clip、voice、graph、listener、volume、timeline、event、meter 的大状态锁（`zircon_plugins/sound/runtime/src/service_types/manager_state.rs:13-68`）。没有 audio-thread command queue、无锁 snapshot、epoch/receipt 或 callback-safe allocator 契约。
2. Kira 是真正的 callback owner（`zircon_plugins/sound/runtime/src/service_types/output_render.rs:8-19` 明确拒绝手工 render），但 Zircon 仍公开 `render_mix` 和 `pull_output_backend_callback` 两个已退役路径。这让上层无法区分“由 Kira 渲染”“没有渲染器”和“测试桩”。
3. `poll_kira_completions` 只在 manager API 调用时轮询（`zircon_plugins/sound/runtime/src/engine/state/playback.rs:46-50`）。没有独立 observation pump，应用不调用 Sound API 时完成事件、voice 回收和 telemetry 可能无限延迟。
4. `configure_output_device` 先停 Kira、清理状态、修改 config，再尝试后续启动（`zircon_plugins/sound/runtime/src/service_types/output_device/configuration.rs:8-29`）；没有两阶段切换、last-known-good、热插拔重连、默认设备变化和回滚证明。

对比：Unreal AudioMixer 将设备、source manager、submix 和 render thread 分开，并通过扩展接口提交异步操作；Godot AudioServer/AudioStreamPlayer 将 server、bus、stream、节点生命周期分开；Fyrox `SoundEngine` 使用 context/source/buffer/renderer 分工；Bevy 的 `AudioOutput`/sink 由 ECS 资源驱动。这些系统都有明确的音频线程边界，Zircon 目前只有一个大锁和 Kira 隐式边界。

### 3.2 资产、流式与资源预算

1. `SoundAsset` 和 Symphonia importer 都把完整源文件解码为 `Vec<f32>`（`zircon_plugins/audio_importer/runtime/src/lib.rs:32-105,141-257`）；`LoadedClip` 又常驻 `Arc<SoundAsset>` 与一份 Kira `StaticSoundData`（`zircon_plugins/sound/runtime/src/engine/state/playback.rs:13-26`）。同一 clip 至少两份 resident PCM，长音乐没有 stream cursor、decoder pool、分块预取、单飞加载或 eviction。
2. importer 只有 4 MiB 元数据预分配上限，注释明确说 resident/streaming budget 尚未落地（`zircon_plugins/audio_importer/runtime/src/lib.rs:32-34`）。该上限不是项目预算，也没有按平台、优先级、并发 voice 或压缩格式统计。
3. `load_clip_impl` 只按 locator map 去重，并在锁外加载、锁内二次检查（`zircon_plugins/sound/runtime/src/service_types/clip_assets.rs:39-73`）；没有 loading state、失败缓存、取消、引用计数、卸载或 source 依赖保持。
4. 外部源和 synth input 有合同字段，但活动播放明确返回 `UnsupportedAdvancedFeature`（`zircon_plugins/sound/runtime/src/service_types/sources.rs:205-212,279-291`）。多声道 importer 能保留 layout，但 Kira CPAL capability 仍只允许 1-2 声道（`zircon_plugins/sound/runtime/src/kira_bridge/device.rs:15-31,60-66`）。

对比：Unreal `SoundWave`/AudioMixer 有 streaming/cache、虚拟化和 source voice 管理；Godot `AudioStream` 区分静态/流式 resource；Fyrox 有 `buffer::streaming` 和 source 生命周期。Zircon 的“解码完整 PCM + StaticSoundData”只能作为短音效 MVP。

### 3.3 混音图、效果与自动化

1. 图编译器有层级索引、send 展开和容量 preflight，但结构性变更在任何 active playback 存在时整体拒绝（`zircon_plugins/sound/runtime/src/kira_bridge/manager/graph.rs:35-73`）。生产 mixer 需要版本化 graph、双缓冲编译、原子切换和旧图 drain，而不是让游戏先停所有声音。
2. `SoundTrackDescriptor` 暴露 effects、pre-effect send、sidechain、solo、phase、delay 等字段（`zircon_runtime/src/core/framework/sound/graph.rs:49-119`），Kira M1 编译仍只支持有限 track/send/volume；effect/advanced control/真正 pre-effect processing 没有完整 backend mapping，声明面大于执行面。
3. automation 在 Kira active 时直接报 M5 unsupported（`zircon_plugins/sound/runtime/src/automation/target/apply.rs:108-115`）。timeline 由 caller 手工传 `delta_seconds` 推进（`zircon_plugins/sound/runtime/src/timeline/advance.rs:10-45`），没有 sample-accurate clock、audio block scheduling、tempo/time-domain、seek receipt 或跨线程参数平滑。
4. `SoundEngineState` 的 `meters` 和 `latency_frames` 在 mixer configure 时重置为静音/0（`zircon_plugins/sound/runtime/src/mixer_configuration/runtime_state.rs:5-12`），没有从真实 submix/callback 采集。输出 status 的 latency 只是 `block_size * latency_blocks` 估算（`zircon_plugins/sound/runtime/src/output/status.rs:5-24`）。

对比：Unreal submix graph 可异步变更并提供 source/submix effect、envelope 和 virtualization；Godot AudioBus 持有 effect chain 和实时 bus state；Fyrox Bus/Effect/Renderer 直接拥有处理阶段；Zircon 当前是图 DTO 到 Kira track handle 的有限投影。

### 3.4 空间、遮挡、HRTF 与声学

1. `apply_source_environment` 实现了 attenuation、cone、doppler、volume、低通、HRTF/卷积的组合（`zircon_plugins/sound/runtime/src/engine/source_environment/apply.rs:16-71`），但全仓没有 production render caller；`render_mix_impl` 反而明确退役。因此空间代码目前主要是单元测试/未来接口，不是可听功能。
2. active listener 只是按 mixer target 与 ID 选择一个 listener（`.../engine/source_environment/listener.rs:3-20`）；没有多 listener、split-screen、per-world listener、listener transform snapshot 或 render block 时序。
3. volume 选择为优先级、weight、ID 的单个 `max_by`（`.../source_environment/volume/influence.rs:18-37`），不是可组合环境层；occlusion 只提供查询/增益辅助，未接入物理场景的稳定 query budget、缓存、异步 ray batch。
4. HRTF profile、impulse response、ray-traced convolution feature 只存 descriptor/Vec 和 status；feature runtime 的 register 仅注册一个 `ModuleDescriptor`（`zircon_plugins/sound/features/ray_traced_convolution_reverb/runtime/src/plugin.rs:19-32`），没有 ray provider、IR cook、cache、生命周期或错误降级。

对比：Unreal Audio Extensions 可接入 spatialization/occlusion/reverb provider；Godot 3D player 与 AudioServer 有明确空间衰减/bus 路径；Fyrox renderer/HRTF 在真实 render context 中运行。Unity Graphics VFX 的 rigid-body collision binder 也消费稳定的 contact event；Zircon 的声学事件和物理查询尚未形成同等跨系统 contract。

### 3.5 事件、生命周期与可观测性

1. source/playback finished 使用无界 `Vec` 暂存（`.../engine/state/storage.rs:42-46`），只有 gameplay emission 有固定容量 journal；没有全局 sequence、cursor、overflow/missed 标记和消费租约。
2. `SoundSourceDescriptor` 的 entity/world gameplay emitter 是可选字段（`zircon_runtime/src/core/framework/sound/components.rs:14-36`），source identity 与 scene entity 没有 generation/ownership 校验，跨世界重用 ID 的行为未定义。
3. callback status 虽有 sequence、underrun、last error 字段（`zircon_runtime/src/core/framework/sound/output.rs:26-36,101-113`），Kira backend 没有把真实 callback report 写回这些字段；diagnostics 不能区分 stale、approximate、disabled、backend unavailable。
4. dynamic event executor 在状态锁体系内排队/执行，未建立 callback-safe bounded queue、预算、超时、隔离和拒绝策略。长时事件/外部 callback 可阻塞音频控制面。

## 4. 需要重构的内容与顺序

### R0：先建立真实边界

1. 将 `SoundEngineState` 拆为 control-plane registry、immutable `AudioGraphSnapshot`、audio-thread command ring、observation ring；所有 API 返回 `CommandReceipt { sequence, accepted_at_revision }`，完成事件带 world/source/voice generation。
2. 选择单一时钟 owner：audio block clock 负责 sample-accurate automation，simulation clock 只提交带时间戳的控制命令。删除或隔离已退役的手工 render 合同，不能让 Kira 与空实现并存。
3. 设备切换改为 prepare -> open -> warm -> crossfade -> retire；记录 actual sample rate/block size/latency/XRUN，支持 default-device/hotplug/recovery/LKG。

### R1：资源与播放

1. `SoundClipAsset` 分为 metadata、compressed chunks、decoded pages；加入 streaming policy、resident budget、priority、voice virtualization、decoder pool、single-flight/cancel/unload。
2. source/playback 使用 generation handle 和独立 voice allocator；支持 external block、synth、stream、multi-channel layout，明确 downmix/upmix policy。
3. update/seek/pause/stop 变成音频线程命令，不得 stop/restart 破坏播放游标；完成、缺 clip、被虚拟化、设备丢失必须有稳定原因码。

### R2：图与 DSP

1. 建立版本化 mixer graph IR、拓扑排序、cycle/sidechain validation、双缓冲 compile 和旧图 grace period。每个 effect 宣布 backend support、latency、tail、state migration。
2. 将 gain/pan/filter/delay/reverb/HRTF/occlusion 放进真实 block pipeline；meters/latency 从 block 结果发布，不再 reset 为静音/0。
3. 将 volume/occlusion/ray tracing 变成异步 provider，带 query budget、cache key、scene revision、stale policy 和可观测 miss/error。

### R3：产品集成与编辑器合同

1. runtime catalog 只在目标配置显式选择 sound 时 materialize；同时校验 `SoundModule` 的 `SoundConfig` 来自 project effective config，而不是 `from_weak_core` 中硬编码 `SoundConfig::default()`（`zircon_plugins/sound/runtime/src/service_types/manager_state.rs:35-43`）。
2. importer 输出可追踪 artifact、codec/stream metadata、cook receipt 和平台变体；禁止仅用 declaration manifest 宣称 feature 已可用。
3. 为 runtime/editor 共用 operation schema、document revision、preview-world 和 terminal receipt；所有声音资源和 mixer graph 的修改必须可撤销、可恢复、可 diff。

## 5. 资格门（当前均未通过）

| 门 | 必须证明 |
|---|---|
| RT-1 audio thread | 10 分钟 callback 无 allocation/lock/XRUN，P50/P95/P99 deadline 与设备实际值可复现 |
| RT-2 streaming | 2 小时音乐、并发 128 voices、预算/淘汰/seek/设备重启无失声或无界增长 |
| RT-3 graph | active voices 下 graph/effect/send 原子切换，旧图 drain，失败可回滚 |
| RT-4 spatial | 多世界/多 listener、occlusion cache、HRTF/IR provider 的误差、预算、stale 语义有 golden 数据 |
| RT-5 lifecycle | source/playback generation、completion cursor、overflow、hotplug、shutdown 可证明 |
| RT-6 product | client/editor/export 目标下 sound provider closure、artifact/cook、operation receipt 全链路通过 |

结论保持 review-only：本报告没有修改生产代码，也没有把旧报告的 Partial/Pass 状态继承为当前通过。
