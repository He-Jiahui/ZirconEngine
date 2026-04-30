---
related_code:
  - zircon_runtime/src/core/framework/mod.rs
  - zircon_runtime/src/core/framework/sound/mod.rs
  - zircon_runtime/src/core/framework/sound/error.rs
  - zircon_runtime/src/core/framework/sound/ids.rs
  - zircon_runtime/src/core/framework/sound/manager.rs
  - zircon_runtime/src/core/framework/sound/mix.rs
  - zircon_runtime/src/core/framework/sound/playback.rs
  - zircon_runtime/src/core/framework/sound/status.rs
  - zircon_runtime/src/core/manager/mod.rs
  - zircon_runtime/src/core/manager/resolver.rs
  - zircon_runtime/src/core/manager/service_names.rs
  - zircon_runtime/src/core/resource/marker.rs
  - zircon_plugins/sound/runtime/src/mod.rs
  - zircon_plugins/sound/runtime/src/config.rs
  - zircon_plugins/sound/runtime/src/module.rs
  - zircon_plugins/sound/runtime/src/service_types.rs
  - zircon_plugins/sound/runtime/src/tests.rs
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/asset/assets/imported.rs
  - zircon_runtime/src/asset/assets/sound.rs
  - zircon_runtime/src/asset/artifact/store.rs
  - zircon_runtime/src/asset/importer/ingest/import_from_source.rs
  - zircon_runtime/src/asset/importer/ingest/import_sound.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/loading/load_asset.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/loading/acquire_asset.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/loading/load_imported_asset.rs
  - zircon_runtime/src/asset/pipeline/manager/resource_sync/register_project_resource.rs
  - zircon_runtime/src/asset/project/manager/asset_kind.rs
  - zircon_runtime/src/asset/tests/project/manager.rs
  - zircon_runtime/src/asset/tests/support.rs
implementation_files:
  - zircon_runtime/src/core/framework/mod.rs
  - zircon_runtime/src/core/framework/sound/mod.rs
  - zircon_runtime/src/core/framework/sound/error.rs
  - zircon_runtime/src/core/framework/sound/ids.rs
  - zircon_runtime/src/core/framework/sound/manager.rs
  - zircon_runtime/src/core/framework/sound/mix.rs
  - zircon_runtime/src/core/framework/sound/playback.rs
  - zircon_runtime/src/core/framework/sound/status.rs
  - zircon_runtime/src/core/manager/mod.rs
  - zircon_runtime/src/core/manager/resolver.rs
  - zircon_runtime/src/core/manager/service_names.rs
  - zircon_runtime/src/core/resource/marker.rs
  - zircon_plugins/sound/runtime/src/mod.rs
  - zircon_plugins/sound/runtime/src/config.rs
  - zircon_plugins/sound/runtime/src/module.rs
  - zircon_plugins/sound/runtime/src/service_types.rs
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/asset/assets/imported.rs
  - zircon_runtime/src/asset/assets/sound.rs
  - zircon_runtime/src/asset/artifact/store.rs
  - zircon_runtime/src/asset/importer/ingest/import_from_source.rs
  - zircon_runtime/src/asset/importer/ingest/import_sound.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/loading/load_asset.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/loading/acquire_asset.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/loading/load_imported_asset.rs
  - zircon_runtime/src/asset/pipeline/manager/resource_sync/register_project_resource.rs
  - zircon_runtime/src/asset/project/manager/asset_kind.rs
plan_sources:
  - user: 2026-04-21 PLEASE IMPLEMENT THIS PLAN
  - user: 2026-04-21 Later Milestones / M2 基础子系统补齐
  - user: 2026-04-21 继续
  - .codex/plans/M1 主链收口与文本底座计划.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/全系统重构方案.md
tests:
  - cargo test -p zircon_runtime asset::tests::project::manager::project_manager_imports_sound_assets_into_runtime_library --locked
  - cargo test -p zircon_runtime zircon_plugin_sound_runtime::tests::sound_manager_loads_project_wav_and_mixes_playback_to_stereo --locked
  - cargo test -p zircon_runtime --lib --locked sound_manager_ --offline
  - cargo test -p zircon_runtime tests::extensions::manager_handles::externalized_runtime_plugins_keep_manager_handles_under_core_manager_contracts --locked
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1
doc_type: module-detail
---

# Runtime Sound Extension

## Purpose

这份文档记录 `M2` 第二个真实子系统起手：把 `zircon_plugin_sound_runtime` 从只有 module/config/service type 壳的占位实现，补成可通过项目资产加载 `.wav`、创建播放实例、并输出真实混音样本的最小音频闭环。

当前完成线不是平台音频设备、空间音频、bus graph、streaming decode 或压缩格式支持，而是更窄的一层：

- `core::framework::sound` 定义共享音频合同
- `core::manager` 暴露稳定 `SoundManager` contract / handle
- `asset` 管线认识 `.wav` 并落成 `SoundAsset`
- `zircon_plugin_sound_runtime` 提供默认 `software-mixer` runtime

这正对应 `M2` 里“音频先做资源加载与播放闭环”的完成线。

## Ownership

这一轮之后声音子系统的 ownership 固定为：

- `zircon_runtime::core::framework::sound`
  - `SoundClipId`
  - `SoundPlaybackId`
  - `SoundClipInfo`
  - `SoundPlaybackSettings`
  - `SoundMixBlock`
  - `SoundBackendStatus`
  - `SoundError`
  - `SoundManager`
- `zircon_runtime::core::manager`
  - `SOUND_MANAGER_NAME`
  - `SoundManagerHandle`
  - `resolve_sound_manager(...)`
  - `ManagerResolver::sound()`
- `zircon_runtime::asset`
  - `ResourceKind::Sound`
  - `SoundAsset`
  - `.wav` importer
  - `ProjectAssetManager::load_sound_asset(...)`
- `zircon_plugin_sound_runtime`
  - `SoundModule`
  - `SoundDriver`
  - `DefaultSoundManager`

也就是说：

- framework 只拥有中性 DTO 和 manager trait
- core manager 只拥有稳定 resolver surface
- asset 只拥有 `.wav -> SoundAsset` 的导入和 library artifact 管线
- runtime extension 才拥有 clip cache、playback state 和 software mixing 行为

## Contract Shape

`SoundManager` 当前故意保持很小，只覆盖 `M2` 这一步必须成立的最小动作：

- `backend_name()`
- `backend_status()`
- `load_clip(locator)`
- `clip_info(clip)`
- `play_clip(clip, settings)`
- `stop_playback(playback)`
- `render_mix(frames)`

对应 DTO 也只保留最小必需集：

- `SoundClipId`
- `SoundPlaybackId`
- `SoundClipInfo { locator, sample_rate_hz, channel_count, frame_count, duration_seconds }`
- `SoundPlaybackSettings { gain, looped }`
- `SoundMixBlock { sample_rate_hz, channel_count, samples }`
- `SoundBackendStatus { requested_backend, active_backend, state, detail, sample_rate_hz, channel_count }`
- `SoundError::{InvalidLocator, BackendUnavailable, NoProjectOpen, UnknownClip, UnknownPlayback, InvalidMixRequest, Decode, Io}`

这里没有提前引入 source bus、DSP node、listener、spatialization、streaming source、Ogg/MP3、bus automation 或 mixer graph。那些都属于后续音频系统层级，而不是这条 `M2` 起手 contract 该承载的范围。

## Asset Path

这轮真正的底层补点不在 `SoundManager` 自己，而在 asset importer。

如果只给 `DefaultSoundManager` 加一个“直接读文件”的特例路径，`ProjectAssetManager::open_project()` 仍会在遇到 `.wav` 时因为 `UnsupportedFormat` 失败。那样只是让上层 feature 走一条旁路，并没有把共享下层补齐。

所以这里先把 shared support 层补全：

- `ResourceKind` 新增 `Sound`
- `ImportedAsset` 新增 `Sound(SoundAsset)`
- `AssetImporter::import_from_source()` 识别 `.wav`
- `import_sound.rs` 负责把 WAV bytes 解析成 `SoundAsset`
- `ArtifactStore` 为 `Sound` 分配 `lib://sound/...` artifact 路径
- `ProjectAssetManager` 增加 `load_sound_asset(...)` / `acquire_sound_asset(...)`

`SoundAsset` 目前持有：

- `uri`
- `sample_rate_hz`
- `channel_count`
- `samples: Vec<f32>`

也就是导入后直接保存归一化 PCM 样本，而不是把源文件 bytes 原样留给 runtime 每次再解码。

## WAV Support

当前 `.wav` 导入支持以下最小集合：

- PCM 8-bit
- PCM 16-bit
- PCM 24-bit
- PCM 32-bit
- IEEE float 32-bit

`SoundAsset::from_wav_bytes(...)` 负责：

- 校验 `RIFF/WAVE` 头
- 解析 `fmt ` chunk
- 查找 `data` chunk
- 检查 `channel_count` / `sample_rate_hz` / `block_align`
- 把样本归一化成 `f32`

这一轮没有支持 ADPCM、压缩 WAVE 扩展、metadata chunk 透传、streaming chunk decode 或超大文件分页加载。

## Runtime Implementation

默认实现 `DefaultSoundManager` 当前基于 `software-mixer`：

- 通过 `ProjectAssetManager` 按 locator 解析 `SoundAsset`
- 维护 `locator -> SoundClipId` clip cache
- 维护 `SoundPlaybackId -> ActivePlayback` 播放实例表
- `render_mix(frames)` 按当前播放游标把 clip 样本混到输出 buffer
- mono clip 输出到 stereo 时复制到左右声道
- multi-channel clip 输出到 mono 时先做均值
- `stop_playback(playback)` 会让播放实例从后续混音中消失
- `looped = true` 的播放实例会跨 clip 结尾回绕继续混音
- 播放结束且 `looped = false` 的实例会在混音后自动回收

默认配置是：

- `backend = "software-mixer"`
- `sample_rate_hz = 48000`
- `channel_count = 2`
- `master_gain = 1.0`

这让它满足两个条件：

- 真正能走完 `project asset -> load_clip -> play_clip -> render_mix`
- 不依赖本机是否存在可用音频设备，所以测试环境稳定

## Module Wiring

`SoundModule` 不再走 `module_descriptor_with_driver_and_manager::<_, _>(...)` 的占位 helper。

现在和 `net` 一样显式注册三层服务：

1. `SoundDriver`
2. `DefaultSoundManager`
3. manager handle `SoundManagerHandle`

并且 `DefaultSoundManager` 明确依赖：

- `SoundDriver`
- `AssetModule.Manager.ProjectAssetManager`

这样上层 app / editor / runtime host 继续只认 `core::manager::resolve_sound_manager(...)`，而具体默认实现、asset support 和将来的平台 backend 替换都留在 runtime 内部。

## Validation

这轮至少跑过的回归包括：

- `cargo test -p zircon_runtime asset::tests::project::manager::project_manager_imports_sound_assets_into_runtime_library --locked`
  - 证明最底层 `.wav` importer、`SoundAsset`、artifact store 和 project registry 已经闭合
- `cargo test -p zircon_runtime zircon_plugin_sound_runtime::tests::sound_manager_loads_project_wav_and_mixes_playback_to_stereo --locked`
  - 证明正常路径已经成立：`open_project -> load_clip -> play_clip -> render_mix`
- `cargo test -p zircon_runtime --lib --locked sound_manager_ --offline`
  - 证明播放生命周期 contract 已经覆盖停止后不再混音、looped playback 跨 clip 结尾回绕，以及项目 WAV 加载混音路径
- `cargo test -p zircon_runtime tests::extensions::manager_handles::externalized_runtime_plugins_keep_manager_handles_under_core_manager_contracts --locked`
  - 证明 `SOUND_MANAGER_NAME`、`SoundManagerHandle` 和 `resolve_sound_manager(...)` 已进入 core manager 稳定表面

## Next Steps

这一轮收口后，后续声音方向可以继续分层推进：

1. 在 `SoundAsset` 之上加压缩格式和 streaming decode
2. 把 `software-mixer` 接到真实平台音频输出 driver
3. 再往上才是 bus、spatialization、listener、effect chain 和 scene-attached audio source

关键是这些后续层都应该建立在当前已经收口的 `SoundManager` contract 和 `SoundAsset` importer 之上，而不是重新回到“只有 module 壳、没有真实 runtime 行为”的状态。
