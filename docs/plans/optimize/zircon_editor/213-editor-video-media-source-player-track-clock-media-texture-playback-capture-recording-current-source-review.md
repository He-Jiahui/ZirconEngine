---
title: Editor Video、MediaSource、Player、Track、Clock、MediaTexture、Playback、Capture 与 Recording 当前源码复核
category: zircon_editor
report_id: Editor213
review_date: 2026-08-29
baseline_head: a2d8d811c4a3a1fc1db6f5375c491e7e4502533f
verification_head: a2d8d811c4a3a1fc1db6f5375c491e7e4502533f
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
canonical_owner: Editor36
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/36-video-media-source-player-track-clock-media-texture-playback-capture-recording-authoring-review.md
  - docs/plans/optimize/zircon_editor/110-editor-video-media-player-track-clock-media-texture-playback-capture-recording-current-source-review.md
  - docs/plans/optimize/zircon_editor/157-editor-video-media-source-player-track-clock-media-texture-playback-capture-recording-current-source-review.md
related_code:
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_runtime_interface/src/buffer.rs
  - zircon_runtime_interface/src/runtime_api/session/requests.rs
  - zircon_runtime_interface/src/ui/template/asset/resource_ref
  - zircon_runtime/src/asset/assets/imported.rs
  - zircon_runtime/src/asset/assets/sound.rs
  - zircon_runtime/src/ui/template/asset/resource_ref
  - zircon_runtime/src/core/framework/time
  - zircon_runtime/src/core/framework/sound
  - zircon_runtime/src/core/framework/render/capture.rs
  - zircon_runtime/src/graphics/runtime/render_framework/capture_frame
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record
  - zircon_runtime/src/dynamic_api/frame.rs
  - zircon_runtime/src/dynamic_api/session
  - zircon_runtime/crates/zr_rhi_wgpu/src/gpu_readback_queue
  - zircon_plugins/sound/runtime/src/service_types
  - zircon_plugins/sound/runtime/src/timeline
  - zircon_plugins/audio_importer/runtime
  - zircon_plugins/opus_importer/runtime
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/asset_rows/media.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/importer_classification/media.rs
  - zircon_plugins/first_party_runtime_catalog
  - zircon_plugins/first_party_editor_catalog
  - zircon_app/src/entry/runtime_entry_app/frame_capture.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_editor/src/core/gateway
  - zircon_editor/src/core/play
  - zircon_editor/src/core/asset/type_registry
  - zircon_editor/src/ui/retained_host/viewport
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback/ui_diagnostics/observability.rs
tests:
  - zircon_runtime/crates/zr_rhi_wgpu/src/gpu_readback_queue/tests.rs
  - zircon_runtime/src/core/framework/render/capture.rs
  - zircon_runtime/src/core/framework/sound/tests.rs
  - zircon_runtime/src/graphics/runtime/render_framework/capture_frame/capture_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/capture_mailbox.rs
  - zircon_plugins/audio_importer/runtime/src/lib.rs
  - zircon_plugins/sound/runtime/src/tests
  - zircon_runtime_interface/src/tests/runtime_owned_result.rs
  - zircon_app/src/entry/runtime_entry_app/frame_capture.rs
  - zircon_editor/src/core/gateway/session/tests.rs
  - zircon_editor/src/core/play/tests.rs
  - zircon_editor/src/ui/retained_host/viewport/tests/controller_polls_latest_captured_frame_from_render_framework.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/90-runtime-rhi-wgpu-adapter-device-capability-resource-command-queue-submission-completion-readback-surface-device-loss-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99zn-runtime-audio-sound-clip-streaming-device-mixer-bus-effect-spatial-occlusion-reverb-timeline-event-voice-chat-editor-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99zy-runtime-cinematic-sequencer-sequence-shot-track-section-binding-hierarchy-evaluation-camera-cut-audio-event-take-recorder-movie-render-queue-network-save-scalability-editor-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/119-editor-cinematic-sequencer-shot-track-binding-take-recorder-movie-render-queue-current-source-review.md
  - docs/plans/optimize/zircon_editor/120-editor-runtime-gateway-session-event-consumer-world-sync-generation-backpressure-reconnect-shutdown-current-source-review.md
  - docs/plans/mvp/03/failure-2026-07-30-runtime-frame-capture-sibling-module-projection.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Media/Public/IMediaPlayer.h
  - dev/UnrealEngine/Engine/Source/Runtime/Media/Public/IMediaControls.h
  - dev/UnrealEngine/Engine/Source/Runtime/Media/Public/IMediaSamples.h
  - dev/UnrealEngine/Engine/Source/Runtime/Media/Public/IMediaTracks.h
  - dev/UnrealEngine/Engine/Source/Runtime/Media/Public/IMediaAudioSample.h
  - dev/UnrealEngine/Engine/Source/Runtime/Media/Public/IMediaTextureSample.h
  - dev/UnrealEngine/Engine/Source/Runtime/Media/Public/IMediaPlayerFactory.h
  - dev/UnrealEngine/Engine/Source/Runtime/Media/Private/MediaClock.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/MediaUtils/Public/MediaSampleQueue.h
  - dev/UnrealEngine/Engine/Source/Runtime/MediaAssets/Public/MediaSource.h
  - dev/UnrealEngine/Engine/Source/Runtime/MediaAssets/Public/MediaPlayer.h
  - dev/UnrealEngine/Engine/Source/Runtime/MediaAssets/Public/MediaTexture.h
  - dev/UnrealEngine/Engine/Source/Runtime/MovieSceneCapture
  - dev/godot/scene/resources/video_stream.cpp
  - dev/godot/scene/gui/video_stream_player.cpp
  - dev/godot/servers/movie_writer/movie_writer.cpp
  - dev/godot/servers/movie_writer/movie_writer_pngwav.cpp
  - dev/godot/modules/theora/editor/movie_writer_ogv.cpp
  - dev/bevy/crates/bevy_render/src/view/window/screenshot.rs
  - dev/Fyrox/fyrox-impl/src/resource/fbx/scene/video.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Utilities/CameraCaptureBridge.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/Passes/CapturePass.cs
finding_status:
  p0_open: 5
  p0_partial: 0
  p0_closed: 0
  p1_open: 49
  p1_partial: 11
  p1_closed: 0
  p2_open: 12
  p2_partial: 0
  p2_closed: 0
gate_status:
  fail: 25
  partial: 7
  pass: 0
---

# Editor213 · Video / MediaSource / Player / Track / Clock / MediaTexture / Playback / Capture / Recording 当前源码复核

## 1. 结论

当前 Zircon 仍没有工程级视频或媒体产品。全生产路径精确扫描中，`MediaSourceAsset`、`MediaOpenOptions`、`MediaProvider`、`MediaSession`、`TrackCatalog`、`VideoSample`、`MediaClock`、`MediaTexture`、`RecorderSession`、`RecordingArtifact`、`EncoderProvider`、`MuxerProvider`和`CaptureSample`均为零命中。现有 importer、声音块、读回、截图和 PNG writer 没有组成 source、probe、provider、session、timestamped sample transport、clock/sync、GPU conversion、Editor toolkit、recorder 和 durable artifact 的共同产品链。

Editor157 之后没有任何 P0 满足关闭条件。公共 UI 仍接受 Media 后静默降为 Data；External audio submit 仍只替换一块 PCM，播放层仍明确拒绝；DLL frame V2 仍只有 width/height/generation/owned RGBA；Performance Workbench 仍返回固定 `Frame 1234 / CPU 7.1 ms / GPU 9.2 ms`。开放的 frame-capture failure 已证明旧 sibling E0433 不再复现，但 managed test 又被外部 `WorldQueryResult::TransformSnapshot` 非穷尽匹配阻断，不能当作 capture 动态 GREEN。

应保留的工程底座没有被低估：`GpuReadbackQueue`已有三槽 staging ring、count/bytes admission、ticket、cancel/abort/shutdown、异步 map、row unpack、callback panic containment、slot reuse rejection和动态扩缩容；capture 有 typed source/report、RGBA8 与 linear RGBA16F；Play preview 已调用真实 gateway；App 单 PNG writer 有 staging、flush/sync、atomic replace和失败清理。但这些只支撑单帧证据，不能包装成媒体播放或连续录制。

Editor36继续是canonical owner。本轮只刷新currentness，不重复增加finding：**5个P0全部Open；60个P1为49 Open / 11 Partial / 0 Closed；12个P2全部Open；32门为25 Fail / 7 Partial / 0 Pass**。目标链保持：

`MediaSourceAsset + versioned OpenOptions -> admitted MediaProvider -> generation-qualified MediaSession -> TrackCatalog + timestamped SampleQueues -> MediaClock/SyncController -> AudioStreamSink + VideoSampleConverter/MediaTexture -> Runtime/Editor consumers`

`CaptureSource -> timestamped CaptureSample -> bounded RecorderSession -> Encoder/Muxer providers -> atomic/finalizable/recoverable RecordingArtifact`

## 2. 当前物理范围与证据

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 证据 |
|---|---:|---|
| Zircon selected | **1,430 / 66,386 / 60,320 / 2,313,695 / 762 / 39** | Runtime/Interface/Plugin/App/Editor选择集；fingerprint `41aede9c665614243622c4576c2a965de80f56773e6087be2d3fdf953120831e` |
| 五引擎 reference | **51 / 12,809 / 10,674 / 428,941 / 0 / 0** | Unreal Media/MovieSceneCapture主线，Godot/Bevy/Unity Graphics/Fyrox交叉证据；fingerprint `da37b847cc0919bfa77820eb39aa2976baa33d9c1608cc25bf9edba482cadf9d` |
| 合并选择集 | **1,481 / 79,195 / 70,994 / 2,742,636 / 762 / 39** | 当前共享working tree去重物理集合 |

选择集相对 Editor157 从309增至1,430文件，主要因为`zircon_plugins/sound/runtime/src/tests`被拆成大量细粒度源码；这些文件已逐个读取并计入静态test attribute，但文件数增长不代表Media产品出现。另对排除`dev/docs/tools/target/.codex`后的20,124个生产Rust/TOML/ZUI文件执行13个目标产品符号精确扫描，结果全部为0。

本轮只做静态review，没有修改production或tests，也没有运行Cargo、codec、GPU playback、A/V sync、capture/recording、fuzz、fault、scale、soak、跨平台和headless package lane。Tooling优化继续排除；未查询、轮询、等待或实时跟踪协调器。

## 3. 当前实现事实

### 3.1 Source、Provider与装配

1. `ResourceKind`与`ImportedAsset`没有MediaSource、MediaTexture、Playlist、Subtitle、VideoClip、RecordingPreset或RecordingArtifact。
2. `UiResourceKind::Media`由字段名和mp3/ogg/wav/flac/mp4/webm/mov扩展名推断，resolver仍将`Media | GenericAsset`映射为`ResourceKind::Data`。
3. builtin `media.rs`只列Texture Importer、Audio Importer和Opus Importer；分类器只识别texture/audio importer。目录名不是provider registry。
4. first-party Runtime/Editor catalog和App feature没有Media package、player backend、decoder、encoder或muxer装配。
5. Symphonia接受完整`Vec<u8>`，选择一个audio track，循环所有packet并把全部sample累积成`SoundAsset<Vec<f32>>`。预分配虽封顶，最终PCM residency没有总预算，也没有streaming session、seek epoch或track catalog。
6. Opus只有NativeDynamic importer声明和missing-backend diagnostic；`rav1e`来自AVIF静态图片链。两者都不是Media/Recorder provider。

### 3.2 Player、Track、Clock与External Audio

1. `SoundExternalSourceBlock`只有sample rate、channel layout和`Vec<f32>`；缺PTS、duration、sequence、EOS、discontinuity和producer generation。
2. `submit_external_source_block_impl()`执行`HashMap::insert(handle, block)`，不是FIFO/ring，没有capacity、watermark、backpressure、drop、underrun、overrun或consume receipt。
3. `sync_source_voice()`对External/Synth返回`source input has no Kira M1 runtime adapter`；playing validation仍返回`external source playback is enabled by Sound M3`。API成功写入不等于可听。
4. Runtime real/virtual/fixed clocks服务simulation；无media time base、rational PTS、audio device position、presentation deadline、seek epoch、live edge或offline recording clock。
5. 无Player状态机、异步open/close、late callback fence、track catalog/selection、per-track queue、buffer range、A/V drift控制和exact/fast seek。

### 3.3 Video Sample、GPU与MediaTexture

1. 无NV12/P010/I420等plane layout、stride/offset、chroma subsampling/siting、coded/display size、clean aperture、aspect、rotation/mirror。
2. 无PTS/DTS/duration、decode/presentation order、keyframe/corrupt/discontinuity flags和seek epoch。
3. 无range/matrix/primaries/transfer、mastering display、CLL/FALL和source color metadata。
4. 无CPU sample lease、GPU external image/import handle、decoder surface pool、fence/semaphore和release callback。
5. 无YUV conversion、deinterlace、scale、tone/gamut map或dynamic format reconfiguration。
6. 无MediaTexture identity、clock-based sample selection、front/back install、hold/clear/drop、late latch和Material/UI同帧可见性合同。
7. `CapturedHdrFrame`保留linear RGBA16F是正确基础，但没有进入DLL ABI、Editor gateway或writer，也不携带媒体HDR元数据。

### 3.4 Capture、ABI、Artifact与Editor

1. `RenderCaptureSource/Report`能区分offscreen、texture direct import、writeback conversion/copy，并记录target kind与output size；仍缺稳定camera/viewport/RT handle、capture stage、orientation和color contract。
2. viewport mailbox按generation关联request/result，容量受三槽ring限制，只提升更新ready帧；nonblocking poll使用`try_lock`且不finish/wait。
3. 同步capture仍finish pending submission并等待readback，适合显式单帧，不适合作为Recorder每帧热路径。
4. `ZrRuntimeFrameV2`只有ABI version、width、height、generation和`ZrOwnedResultV2 rgba`；没有format、stride、color/HDR、PTS、duration、sequence、source identity或fence。
5. producer/gateway已有shape、byte cap、owner lifetime与显式release；这是foreign-output底座，不是timestamped media sample lease。
6. App writer能可靠发布单PNG；没有cadence、audio、encoder/mux、container reopen、manifest/checksum、finalize/recovery或resume。
7. Editor Play preview接真实capture；Performance Workbench仍用固定frame和CPU/GPU文本，二者authority断裂。
8. 没有Media Toolkit、transport、scrub/frame-step、track/subtitle选择、decoder/cache diagnostics、Recorder preset/job/session progress和artifact browser。

## 4. 参考引擎差异

### 4.1 Unreal主线

1. `IMediaPlayer`拆分cache、controls、samples、tracks、view、open/close、metadata、stats和plugin identity；open允许异步完成。
2. `IMediaPlayerFactory`表达URL/options probe、warning/error、confidence、feature和platform能力，provider选择可审计。
3. `IMediaControls`表达state/status、duration/time/rate、supported rates、loop、seek与playback range。
4. `IMediaSamples`按time range获取audio/caption/metadata/subtitle/video，并提供flush、peek、discard、purge、queue depth和drop统计。
5. `TMediaSampleQueue`有max sample count、flush generation、time-range selection和old sample purge；audio/video默认深度不同。
6. audio/video sample有time/duration/format；texture sample还表达CPU buffer/RHI texture、coded/output geometry、stride、orientation、YUV matrix/range、source color和HDR metadata。
7. `FMediaClock`将Input/Fetch/Output/Render tick阶段与sink生命周期分离，不复用游戏delta充当媒体时钟。
8. MediaSource、MediaPlayer、MediaTexture是独立产品，Electra/WMF/AVF等backend通过provider边界接入。
9. MovieSceneCapture有settings、protocol lifecycle、video/audio protocol与output owner。Zircon单RGBA DTO和PNG不等价。

### 4.2 次级参考

1. Godot `VideoStream -> VideoStreamPlayback -> VideoStreamPlayer`至少贯通play/stop/pause/seek/length/position/audio track/texture/update/audio mix callback，并有buffering、bus和resampler。
2. Godot `MovieWriter`按extension选择writer，定义begin/frame/end、固定fps、统一尺寸和audio mix；PNG+WAV与Theora路径都有video/audio推进和最终flush。
3. Bevy Screenshot有typed RenderTarget、异步entity lifecycle、Captured event、GPU transfer、row unpadding和terminal despawn，只作为单帧request/result交叉证据。
4. Unity Graphics按Camera维护capture actions，URP CapturePass绑定render loop末端active color texture；Zircon缺camera registry与stage合同。
5. Fyrox本地`Video`只是FBX texture record负证据，不得用来降低Unreal主线标准。

领先目标必须建立在同源、同场景、同输出质量、同错误政策的correctness、latency、memory、power和fault raw evidence上。没有等价媒体语义前，依赖更少、接口更短或单次读回更快都不能证明优于Unreal。

## 5. 必须保留的基础

1. 保留readback三槽ring、预算、ticket、cancel/abort/shutdown、async map、row unpack、panic containment和统计，定位为通用readback stage。
2. 保留viewport generation mailbox与nonblocking poll；Recorder另建不静默覆盖中间帧的有界有序队列。
3. 保留typed capture source/report、graph dump、frame profile和linear HDR distinction；扩展新`CaptureSample`，不继续膨胀模糊RGBA DTO。
4. 保留`ZrOwnedResultV2`显式release和owner lifetime；媒体sample另建versioned plane lease与generation/epoch。
5. 保留PNG publication事务模式；RecordingArtifact必须增加container/sidecar/finalize/probe/recovery。
6. 保留Play真实gateway和viewport currentness；Performance Capture必须接同一controller并返回artifact receipt。
7. 保留Symphonia track/channel validation、scratch复用和bounded preallocation思路；不得复用整文件PCM常驻模型做流式媒体。
8. 保留package/capability/NativeDynamic metadata模式；Media provider必须有独立许可、平台、binary和安装证据。

## 6. P0状态

| ID | 状态 | 当前证据与必须动作 |
|---|---|---|
| P0-1 Media UI被Runtime降格为Data | **Open** | `.mp4/.webm/.mov`仍接受后静默降格；真实MediaSource可用前必须结构化拒绝，或同一hard cut引入identity/provider/consumer。 |
| P0-2 External audio假成功 | **Open** | submit替换`Vec<f32>`后voice adapter拒绝；先truthful fail，或完整实现timestamped bounded stream与terminal receipt。 |
| P0-3 无timestamped VideoSample/MediaTexture | **Open** | RGBA/HDR capture不是decoder sample；format/planes/color/PTS/ownership/fence/texture均缺。 |
| P0-4 无MediaClock/sample queue/seek/A-V sync | **Open** | 目标产品类型为零；simulation clocks不能替代media domain。 |
| P0-5 单帧Capture被包装成录制能力 | **Open** | Preview是真单帧，Performance命令仍固定1234，Recorder全链为零。 |

## 7. P1状态

### 7.1 Source、Provider、Open与安全

| ID | 状态 | 差距/重构 |
|---|---|---|
| P1-1 Stable MediaSourceAsset | Open | identity、revision、locator、dependency。 |
| P1-2 Versioned MediaOpenOptions | Open | scheme/header/cache/timeout/credential/provider options。 |
| P1-3 Provider probe/admission registry | Open | importer registry不能替代player provider。 |
| P1-4 Async cancelable open | Open | operation/deadline/cancel/completion event。 |
| P1-5 Container probe artifact | Open | track/color/duration/codec事实需可持久化。 |
| P1-6 Network/protocol security | Open | redirect、private network、credential、cache政策。 |
| P1-7 Malicious input budget | Open | parser dimension/time/packet/residency预算与fuzz。 |
| P1-8 Cook/dependency policy | Open | Data降格不能证明可播放。 |
| P1-9 Codec license/distribution | **Partial** | Opus dist模式存在；Media codec binary/license/platform closure缺失。 |
| P1-10 Provider fallback receipt | Open | forced provider、confidence、fallback order。 |
| P1-11 Metadata/poster/thumbnail | Open | source-qualified poster、filmstrip、waveform。 |
| P1-12 Playlist/sidecar identity | Open | entry、subtitle/metadata track身份。 |

### 7.2 Player、Track、Clock、Queue与同步

| ID | 状态 | 差距/重构 |
|---|---|---|
| P1-13 Strict Player state machine | Open | 合法转换、idempotence、snapshot。 |
| P1-14 Generation-qualified events | Open | session/open/close generation。 |
| P1-15 Controls capability query | Open | duration/time/rate/seek/loop/range。 |
| P1-16 Track catalog | Open | audio/video/subtitle/metadata。 |
| P1-17 Track/format selection | Open | language/role/default/forced。 |
| P1-18 Rational timestamp/range | Open | capture generation不是媒体时间。 |
| P1-19 Bounded sample queues | Open | per-track count/bytes/time预算。 |
| P1-20 MediaClock domain | Open | PTS correlation、epoch、clock modes。 |
| P1-21 A/V sync controller | Open | master/drift/drop/duplicate/resample。 |
| P1-22 Seek lifecycle | Open | flush/preroll/keyframe/exact-fast/old epoch fence。 |
| P1-23 Buffering/live edge | Open | buffered/seekable ranges、watermark、latency。 |
| P1-24 Streaming external audio sink | Open | 当前HashMap replace且播放unsupported。 |

### 7.3 Video Sample、Color、GPU与MediaTexture

| ID | 状态 | 差距/重构 |
|---|---|---|
| P1-25 Typed VideoSampleFormat | Open | plane format/layout。 |
| P1-26 Coded/display geometry | Open | aperture/aspect/orientation。 |
| P1-27 Color/HDR metadata | **Partial** | linear RGBA16F存在；source metadata、YUV、ABI、conversion/golden缺。 |
| P1-28 Timing/decode flags | Open | PTS/DTS/duration/order/keyframe/discontinuity。 |
| P1-29 CPU sample ownership | **Partial** | owned V2 output可释放；plane lease/reuse/epoch缺。 |
| P1-30 GPU external ownership | Open | surface/fence/reuse/release callback。 |
| P1-31 Video conversion pipeline | Open | YUV/deinterlace/scale/tone/gamut。 |
| P1-32 MediaTexture identity | Open | resource/asset/runtime handle均无。 |
| P1-33 Clock-based selection | Open | hold/clear/drop、late frame。 |
| P1-34 Dynamic reconfiguration | Open | surface/pipeline generation与handoff。 |
| P1-35 Mip/filter policy | Open | 动态sample与conversion output政策。 |
| P1-36 GPU budget/performance | **Partial** | readback预算/统计存在；decode/conversion/multi-stream基线缺。 |

### 7.4 Capture、Recorder、Encoder、Mux与Editor

| ID | 状态 | 差距/重构 |
|---|---|---|
| P1-37 Typed CaptureSource | **Partial** | source/report存在；stable handle/stage/color合同缺。 |
| P1-38 Per-camera registry | Open | viewport handle不能替代camera action registry。 |
| P1-39 Timestamped CaptureSample | **Partial** | generation/provenance存在；PTS/format/color/camera/overlay缺。 |
| P1-40 Recorder state machine | Open | prepare/start/pause/resume/stop/finalize/error。 |
| P1-41 CFR/VFR pacing | Open | cadence、drop/duplicate、offline/audio correlation。 |
| P1-42 Encoder provider | Open | rav1e静态图片依赖不构成provider。 |
| P1-43 Muxer provider | Open | negotiation/interleave/finalize。 |
| P1-44 Pipeline backpressure | **Partial** | readback有界；convert/encode/mux/disk与drop reason缺。 |
| P1-45 Recording artifact | **Partial** | 单PNG原子发布；container/sidecar/finalize/recovery缺。 |
| P1-46 Media Toolkit | Open | document/controller/transport/track/frame/inspection。 |
| P1-47 Recorder panel | Open | source/preset/output/job/session/artifact UI。 |
| P1-48 Preview artifact chain | Open | poster/filmstrip/waveform cache。 |

### 7.5 Plugin、Diagnostics、测试与发布

| ID | 状态 | 差距/重构 |
|---|---|---|
| P1-49 Media package owner | Open | catalog分组只装texture/audio importer。 |
| P1-50 First-party assembly trace | Open | runtime/editor package、feature、binary/provider closure。 |
| P1-51 Capability-derived maturity | Open | importer maturity不能代表Media。 |
| P1-52 Stable diagnostics | **Partial** | 局部typed error存在；Media source/session/provider/stage code缺。 |
| P1-53 Media telemetry | **Partial** | readback stats存在；open/decode/sync/drop/record metrics缺。 |
| P1-54 Logging/privacy | Open | URL/header/query/credential/content redaction。 |
| P1-55 Deterministic provider fixture | Open | PTS/stall/seek/track/color change fixture。 |
| P1-56 Malformed/fuzz matrix | Open | container/sample/subtitle/playlist fuzz。 |
| P1-57 Sync/seek golden | Open | 无provider/clock，无法建立。 |
| P1-58 GPU/color visual golden | Open | HDR texel test不覆盖媒体端到端。 |
| P1-59 Recording fault injection | **Partial** | readback/PNG局部失败测试存在；encoder/mux/disk/process/device lane缺。 |
| P1-60 Platform/package/release | Open | 无provider可进入矩阵。 |

## 8. P2状态

12项全部Open：adaptive bitrate streaming、DRM/受保护媒体、hardware decode/encode调度、ultra-low-latency live media、camera/capture device ingestion、360/180/stereo projection metadata、spatial/multichannel media audio、timed metadata/subtitle/accessibility、remote broadcast/pixel streaming、nonlinear transcode/proxy、distributed deterministic encoding、跨引擎同质量性能基准。P2不得越过P0和M0-M8资格门。

## 9. Authority断路

| 表面 | 实际owner | 断路 |
|---|---|---|
| UI Media ref | generic ResourceManager | 降Data，无open/play consumer。 |
| builtin media catalog | importer registry | 分组名制造Media package错觉。 |
| Symphonia | SoundAsset importer | 完整PCM常驻，无session/clock。 |
| External audio submit | Sound HashMap | voice adapter拒绝。 |
| Runtime Time | simulation runtime | 无PTS/device/seek correlation。 |
| Render capture | RenderFramework/readback | 无timestamp/cadence/Recorder。 |
| Runtime DLL frame V2 | dynamic session/gateway | 无format/color/PTS/source/fence。 |
| Play preview | Play/gateway | 真实单帧，不是Media/Recorder。 |
| Performance Capture | static feedback | 未调用真实capture。 |
| PNG publication | App writer | 无连续frame/audio/mux/finalize。 |
| rav1e | AVIF image chain | 无Recorder caller或artifact。 |

## 10. 目标Owner

| Owner | 应拥有 |
|---|---|
| `zircon_runtime_interface::media` | versioned source/session/track/time/sample/capability/error DTO与ABI-safe lease。 |
| `zircon_runtime::core::media` | provider registry、session state、track/sample queues、clock/sync、budget/telemetry。 |
| Media provider packages | probe、decode、platform surface、codec/license/platform capability。 |
| `graphics::media` | external import、conversion、surface pool、fence、MediaTexture install。 |
| Sound streaming adapter | timestamped PCM、device position、watermark、underrun/overrun/EOS。 |
| Capture service | stable source、stage、timestamped sample、source budget。 |
| Recorder service | state/cadence/backpressure、encoder/mux negotiation、finalize/recovery。 |
| Editor Media Toolkit/Recorder | document/controller、transport、inspection、preset/job/artifact projection。 |
| App/package | provider closure、binary/license、headless/offline/release evidence。 |

所有open、event、sample、seek、texture install、capture和artifact都必须携带source/session identity、generation和必要epoch；所有队列声明count/bytes/time预算、admission、drop/backpressure、telemetry和exact terminal。

## 11. 分层重构

| Milestone | 内容 | 退出条件 |
|---|---|---|
| M0 Truthfulness/hard cut | 移除Media->Data、External假成功、固定Capture结果；冻结owner/capability/error namespace | 所有不可用入口结构化Unsupported，无成功假象 |
| M1 Identity/provider/probe | source/options、factory/probe/admission、metadata、安全/许可/package | 同一source可复现provider decision receipt |
| M2 Session/track/clock/seek | state/events、catalog、timestamp、queues、clock、seek、sync | deterministic fixture通过状态/事件/sync gates |
| M3 Streaming audio | bounded PCM、device feedback、watermark、XRUN/EOS、flush/drain | submit可听且有背压/终态证据 |
| M4 Video/MediaTexture | plane/color/timing/lease、conversion、pool/fence、selection/reconfigure | CPU/GPU sample与HDR/color golden通过 |
| M5 Editor Toolkit | transport/scrub/track/diagnostics、poster/filmstrip/waveform、cook/reference | UI仅投影runtime truth并可导航错误 |
| M6 Capture/Recorder core | stable source、timestamped sample、state、CFR/VFR、分阶段有界队列 | ordered capture无静默覆盖，取消/故障一次终态 |
| M7 Encoder/mux/artifact | negotiation、interleave、finalize/probe、sidecar/checksum、atomic/recovery | 产物可重开，kill/disk-full可恢复或隔离 |
| M8 Qualification/release | software/platform/hardware、license/package、headless、fuzz/fault/soak/device loss | G01-G32全Pass并有raw receipt |
| M9 Advanced | ABR/DRM/live/device/360/timed metadata/broadcast/transcode/distributed | 只在M0-M8关闭后启动 |

## 12. Gate当前状态

| Gate | 状态 | 当前缺口 |
|---|---|---|
| G01 Public truthfulness | Fail | Media降Data、External与静态Capture不真实。 |
| G02 Provider admission | Fail | 无factory/probe receipt。 |
| G03 Open/close generation | Fail | 无session。 |
| G04 Probe security | Fail | 无malformed/network lane。 |
| G05 Track catalog/selection | Fail | 无catalog。 |
| G06 Timestamp precision | Fail | 无time base。 |
| G07 Bounded sample queues | Fail | readback不等于track queue。 |
| G08 Media clock modes | Fail | 无MediaClock。 |
| G09 A/V sync | Fail | 无controller。 |
| G10 Seek/discontinuity | Fail | 无epoch/flush/preroll。 |
| G11 External audio playback | Fail | Kira adapter拒绝。 |
| G12 Audio XRUN | Fail | 无stream consumer/device feedback。 |
| G13 Video layout | Fail | 无NV12/P010/I420。 |
| G14 Color/HDR fidelity | Fail | source metadata/conversion/golden缺。 |
| G15 GPU ownership/fence | Fail | 无external surface lease。 |
| G16 MediaTexture presentation | Fail | 无texture/selection。 |
| G17 Dynamic reconfigure | Fail | 无surface generation。 |
| G18 Decode/GPU performance | **Partial** | readback预算/统计存在；decode/conversion矩阵缺。 |
| G19 Capture source correctness | **Partial** | typed source/extent/HDR存在；camera/stage/color矩阵缺。 |
| G20 Recorder pacing | Fail | 无clock/cadence。 |
| G21 Recorder backpressure | **Partial** | readback有界；后续阶段不存在。 |
| G22 Encoder/mux negotiation | Fail | 无provider。 |
| G23 Durable finalize | **Partial** | 单PNG可靠；container/track finalize缺。 |
| G24 Interrupted recovery | **Partial** | 单图清理/readback abort存在；Recorder recovery缺。 |
| G25 Editor command truth | Fail | 固定Frame1234。 |
| G26 Toolkit workflow | Fail | 产品为零。 |
| G27 Preview artifacts | Fail | 无qualified cache。 |
| G28 Diagnostics/privacy | **Partial** | 局部typed error；Media code/redaction缺。 |
| G29 Malformed/fuzz/fault | **Partial** | 局部failure tests；媒体全链为空。 |
| G30 Platform/device matrix | Fail | 无provider/device ingestion。 |
| G31 Headless/package/license | Fail | importer metadata不足。 |
| G32 Release/rollback | Fail | 无schema/provider/artifact迁移。 |

## 13. 禁止的临时修补

1. 禁止继续把Media映射Data并让UI显示成功。
2. 禁止把整文件PCM importer或External HashMap改名为streaming player。
3. 禁止把`CapturedFrame`添加一个timestamp字段后宣称VideoSample完成。
4. 禁止用游戏delta作为MediaClock，或在seek后接受旧epoch callback。
5. 禁止每帧同步capture+PNG循环伪装Recorder。
6. 禁止因依赖树存在rav1e/Symphonia/Opus就声明codec capability。
7. 禁止Editor自持decoder/player/recorder权威状态。
8. 禁止无count/bytes/time预算、drop reason和terminal receipt的队列。
9. 禁止保留旧ABI/alias/compat路径形成双重真相。
10. 禁止未通过同质量correctness与raw benchmark就宣称优于Unreal。

## 14. 本轮产出边界

本报告完成当前源码、相关owner、开放failure与五参考引擎的静态currentness复核。没有生产实现或测试修改，也不把静态test attribute、依赖存在、源码修复或被外部编译错误阻断的managed replay记作动态通过。后续实现必须从M0 truthfulness开始，并在每个milestone前重新扫描共享working tree；Editor36继续唯一计数，Editor213只作为最新current-source执行入口。
