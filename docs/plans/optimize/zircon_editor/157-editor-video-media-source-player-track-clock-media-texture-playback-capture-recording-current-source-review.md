---
title: Editor Video、MediaSource、Player、Track、Clock、MediaTexture、Playback、Capture 与 Recording 当前源码复核
category: zircon_editor
report_id: Editor157
review_date: 2026-08-27
baseline_head: 5eb80f437dc655eb169c45942e50453f7a116368
verification_head: 6ed0702b9a4d865698c0f1b11af6cd668ce8ebb6
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
canonical_owner: Editor36
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/36-video-media-source-player-track-clock-media-texture-playback-capture-recording-authoring-review.md
  - docs/plans/optimize/zircon_editor/110-editor-video-media-player-track-clock-media-texture-playback-capture-recording-current-source-review.md
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
  - docs/plans/optimize/zircon_editor/139-editor-sound-audio-clip-mixer-routing-effect-spatial-acoustic-timeline-audition-current-source-review.md
  - docs/plans/optimize/zircon_editor/144-editor-render-pipeline-render-graph-frame-debugger-capture-lighting-bake-reflection-probe-post-process-debug-current-source-review.md
  - docs/plans/optimize/zircon_editor/147-editor-runtime-diagnostics-performance-timeline-console-telemetry-observability-authoring-current-source-review.md
  - docs/plans/optimize/zircon_editor/156-editor-texture-image-cubemap-render-target-sampler-compression-streaming-preview-current-source-review.md
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

# Editor157 · Video / MediaSource / Player / Track / Clock / MediaTexture / Playback / Capture / Recording 当前源码复核

## 1. 结论

当前 Zircon 仍没有工程级视频或媒体产品。生产路径没有稳定的 `MediaSourceAsset`、`MediaProvider`、`MediaSession`、`TrackCatalog`、timestamped audio/video sample、`MediaClock`、A/V sync controller、`MediaTexture`、`RecorderSession`、encoder/muxer provider 或可恢复的 recording artifact。这里不是再接一个 codec 即可完成，而是 source、probe、provider、session、sample transport、clock、GPU conversion、Editor product 和发布资格全部没有共同 owner。

Editor110 之后，单帧捕获基础确有工程进展，必须保留。`GpuReadbackQueue`已有三槽 staging ring、row padding 解包、request/frame/pending count 与 byte budget、ticket cancel、frame abort、slot reuse 拒绝、异步 map、callback panic containment、Drop shutdown 终态和动态扩缩容；viewport mailbox按 generation 配对 pending/completed，只提升更新帧；`CapturedFrame`携带 typed `RenderCaptureSource`、capture report、graph dump 与 profile，另有保持线性 RGBA16F 的 `CapturedHdrFrame`。Editor Play controller现在会通过 gateway 调用真实 `capture_frame()`，retained viewport使用不等待GPU的 poll路径，App单图writer以staging、flush/sync和原子替换发布PNG。

这些能力仍只是截图、Play preview 和 render evidence 底座。跨DLL输出已从旧报告的 `ZrRuntimeFrameV1`硬切为`ZrRuntimeFrameV2`，但字段仍只有ABI版本、width、height、generation和owned RGBA bytes；没有stride、pixel format、color/HDR metadata、PTS、duration、sequence、source identity或GPU fence。Performance Workbench的“Capture Frame”仍直接返回固定`Frame 1234 / CPU 7.1 ms / GPU 9.2 ms`，没有调用上述真实controller。连续录制的状态机、cadence、audio、encoder、mux、finalize和recovery仍为零。

旧报告关于依赖的表述也需要纠正。仓内存在Symphonia音频解码和由`ravif`引入的`rav1e`，但Symphonia把完整文件离线展开为常驻`SoundAsset<Vec<f32>>`，`rav1e`服务AVIF静态图片依赖；二者都没有形成媒体播放、视频解码或录制provider。Opus package有NativeDynamic distribution metadata，却仍是importer边界。这些依赖事实只能证明可复用的plugin/package模式，不能证明Media runtime可达。

因此Editor36继续是canonical owner，本轮只刷新currentness，不重复增加finding：**5个P0全部Open；60个P1为49 Open / 11 Partial / 0 Closed；12个P2全部Open；32门为25 Fail / 7 Partial / 0 Pass**。目标边界保持两条独立产品链：

`MediaSourceAsset + versioned OpenOptions -> admitted MediaProvider -> generation-qualified MediaSession -> TrackCatalog + timestamped SampleQueues -> MediaClock/SyncController -> AudioStreamSink + VideoSampleConverter/MediaTexture -> Runtime/Editor consumers`

`CaptureSource -> timestamped CaptureSample -> bounded RecorderSession -> Encoder/Muxer providers -> atomic/finalizable/recoverable RecordingArtifact`

## 2. 当前物理范围与证据

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 本轮证据 |
|---|---:|---|
| Zircon Runtime/Interface/Plugin/App/Editor selected | **309 / 37,233 / 33,825 / 1,298,466 / 255 / 0** | resource/UI、time/sound、audio importer、capture/readback、ABI/gateway/Play、profiling、catalog/App；fingerprint `eaea25ceb186bd723f8d8a9835c42b5dfeac33e1d79df3684af7ad27463591f0` |
| Unreal/Godot/Fyrox/Bevy/Unity Graphics reference | **40 / 10,244 / 8,693 / 347,261 / 0 / 0** | Unreal Media与MovieSceneCapture、Godot VideoStream/MovieWriter、Bevy screenshot、Unity camera capture、Fyrox负证据；fingerprint `df6d60fee695c22622f118b33185a0d7d97e1fcc163d802b3b4767f8f1c7d706` |
| 全部选择集 | **349 / 47,477 / 42,518 / 1,645,727 / 255 / 0** | 当前共享working tree的去重选择集；未把`Cargo.lock`计入源码范围，但单独扫描了codec依赖 |

Zircon分布为Runtime 90文件/12,169行、Interface 10/788、Plugin 84/7,363、App 3/1,322、Editor 122/15,591。本轮逐文件读取选择集完整文本，并执行全生产路径产品符号、caller、extension、Cargo dependency、catalog/App assembly、TODO/FIXME/unimplemented/unsupported和测试声明扫描。`tests`只表示静态test attribute，不表示执行或通过；参考切片中没有显式测试声明，不能把接口/实现存在换算成已验证质量。

本轮是review-only，没有修改production源码或tests，也没有运行Cargo、真实codec、GPU playback、A/V sync、capture/recording、fuzz、fault、scale、soak、跨平台或headless package动态lane。Tooling优化按用户要求排除；没有查询、轮询、等待或实时跟踪协调器。

## 3. 当前实现事实

### 3.1 Identity、Source、Provider与装配

1. `ResourceKind`和`ImportedAsset`没有MediaSource、MediaTexture、Playlist、Subtitle、VideoClip、RecordingPreset或RecordingArtifact身份。
2. `UiResourceKind::Media`仍由`media|video|audio`字段名以及mp3/ogg/wav/flac/mp4/webm/mov扩展名推断，resolver继续将`Media | GenericAsset`映射为`ResourceKind::Data`。
3. `builtin_catalog/asset_rows/media.rs`只列Texture Importer、Audio Importer和Opus Importer；`importer_classification/media.rs`只分类texture/audio importer。目录名“media”不是Media产品或provider registry。
4. first-party Runtime/Editor catalog和App feature没有Media package、player backend、decoder、encoder或muxer装配。
5. 全生产路径没有通用`MediaPlayer`、`MediaTexture`、`VideoDecoder`、`VideoSample`、`MediaClock`、`RecorderSession`或`RecordingArtifact`类型；唯一`MediaSourceStream`是Symphonia内部输入包装。
6. Symphonia provider只选择一个可解码audio track，循环读packet并把全部decoded sample累积成`SoundAsset`；没有流式session、seek epoch、bounded residency或Media track catalog。
7. Opus importer发布NativeDynamic dist合同和missing-backend diagnostic，但没有Media provider capability、runtime sample output或codec许可账本。
8. `rav1e`由`ravif`静态AVIF图片链带入，没有生产Recorder caller、video frame input、rate control、muxer或artifact receipt。

### 3.2 Player、Track、Clock与External Audio

1. `SoundExternalSourceBlock`只有sample rate、channel count/layout和`Vec<f32>`；缺sample format version、frame count、PTS、duration、sequence、EOS、discontinuity与producer generation。
2. `submit_external_source_block_impl()`按handle替换HashMap中的整块数据，不是FIFO/ring；没有capacity、watermark、backpressure、drop、underrun、overrun或consumer receipt。
3. `sync_source_voice()`对External和Synth返回“no Kira M1 runtime adapter”，另一处validation仍写明external playback由未来Sound M3启用。submit成功不等于可听。
4. Runtime的`Time<MonotonicReal/Virtual/Fixed>`、FrameClock rebase和clock-domain stamp服务simulation frame；没有media time base、PTS correlation、audio device position、presentation deadline、seek epoch、live edge或offline movie clock。
5. 没有Closed/Opening/Ready/Playing/Paused/Buffering/Seeking/Ended/Closing/Error状态机，也没有async open completion、event ordering、late callback fence或close cancellation。
6. 没有track catalog、language/role/default/forced flags、format selection和audio/video/subtitle/metadata分队列。
7. 没有A/V master、drift estimator、audio resample、video drop/duplicate、preroll、exact/fast seek或discontinuity恢复。

### 3.3 Video Sample、GPU与MediaTexture

1. 没有NV12/P010/I420等plane layout、stride/offset、chroma subsampling/siting或odd extent规则。
2. 没有coded/display size、clean aperture、sample aspect、rotation/mirror、PTS/DTS/duration、decode/presentation order和keyframe/corrupt/discontinuous flags。
3. 没有range、matrix、primaries、transfer、mastering display、CLL/FALL或source color-space metadata。
4. 没有CPU sample lease、GPU external image/import handle、decoder surface pool、fence/semaphore、release callback和zero-copy/fallback receipt。
5. 没有YUV到RGB转换、tone/gamut map、deinterlace、scaler或dynamic format/reconfigure pipeline。
6. 没有独立MediaTexture identity、sample queue、clock-based selection、front/back install、late latch、hold/clear/drop policy或Material/UI同帧可见性。
7. `CapturedHdrFrame`保持线性RGBA16F并携带capture provenance是正确基础，但未进入DLL ABI、Editor gateway或writer，也没有原始媒体HDR metadata。

### 3.4 Capture、ABI、Artifact与Editor

1. `RenderCaptureSource`区分primary offscreen、texture direct import、writeback conversion/copy等来源；`RenderCaptureReport`携带target kind、source和output size。
2. `GpuReadbackQueue`以三槽异步ring执行4 bytes/pixel RGBA readback，具count/bytes admission、cancel、abort、shutdown、callback隔离和in-flight/rejection统计。
3. viewport mailbox按generation关联request/result，容量跟随三槽ring，只提升更新ready帧；nonblocking poll使用`try_lock`且不调用finish/wait。
4. 同步capture仍会finish pending submission并等待readback；HDR capture读取renderer-owned linear scene color。这适合显式单帧请求，不适合直接循环成为Recorder热路径。
5. `ZrRuntimeFrameV2`仍只表达width/height/generation/owned RGBA。Runtime producer验证尺寸和bytes上限，Editor gateway验证ABI/shape并在Runtime owner仍存活时显式release foreign output。
6. Editor Play的`capture_preview_frame()`真实调用default viewport gateway capture；retained viewport会拒绝zero size、RGBA长度错误和旧generation，并测试不等待并发submit。
7. App PNG writer以create-new partial、encode、flush、sync和Windows ReplaceFileW/rename发布，失败清理staging；测试覆盖roundtrip、shape、encode、replace、flush与sync失败。
8. 该writer没有frame cadence、timestamp、audio、encoder/muxer、container reopen、manifest/checksum、crash repair或resume，因此只是durable single-image foundation。
9. Performance Workbench命令仍固定返回`Capture Frame   Frame 1234   CPU 7.1 ms   GPU 9.2 ms`，与真实Play/viewport capture并未接线。
10. 没有Media toolkit、transport、scrub/frame-step、track/subtitle选择、decoder/cache diagnostics、Recorder面板、preset、job/session progress或artifact browser。

## 4. 与参考引擎的工程差异

### 4.1 Unreal

1. `IMediaPlayer`将cache、controls、samples、tracks、view、open/close、metadata、stats和player plugin identity拆开；open/close明确允许异步完成。
2. `IMediaPlayerFactory`提供URL/options probe、warning/error、confidence、feature与platform能力，并由`IMediaModule`注册factory、capture support、clock和ticker。
3. `IMediaControls`有state/status、duration/time/rate、supported rate、loop、seek和playback range，不把UI状态寄托在按钮本地变量。
4. `IMediaSamples`按time range获取audio/caption/metadata/subtitle/video，支持flush、peek、discard、purge、queue depth和drop statistics。
5. `TMediaSampleQueue`有producer/consumer锁、max sample count、admission、flush generation、time-range selection、old sample purge和drop count；默认queue depth区分audio 512与video 8。
6. `IMediaAudioSample`携带format、channels、frames、sample rate、time和duration；`IMediaTextureSample`同时支持CPU buffer或RHI texture，并携带format、coded/output dimensions、stride、time、duration、orientation、aspect、YUV matrix、range、source color space和HDR metadata。
7. `FMediaClock`将sink add/remove与Input/Fetch/Output/Render阶段tick分离，支持timecode lock。它不是游戏virtual delta的别名。
8. MediaSource、MediaPlayer和MediaTexture是独立asset/product，平台player插件再拆成Electra、WMF、AVF等backend。
9. MovieSceneCapture至少有settings、capture protocol生命周期、video/audio协议和output owner。Zircon的单次RGBA ABI与原子PNG不等价于该产品层。

### 4.2 Godot

1. `VideoStream`可实例化`VideoStreamPlayback`，playback至少表达play/stop/pause/seek/length/position/audio track/texture/update和audio mix callback。
2. `VideoStreamPlayer`拥有loop、autoplay、speed、buffering、audio bus、resampler和mix callback，并在stream变化时重建playback和texture。
3. `MovieWriter`按extension registry选择writer，定义begin/frame/end、固定fps、audio mix rate/channel和统一frame size。
4. PNG+WAV writer同时推进image sequence和audio，Theora writer执行video/audio encode、keyframe设置、Ogg page timestamp排序与final EOS flush。它仍不是Zircon最终目标，但已超过“逐帧调用截图API”的临时实现。

### 4.3 Bevy、Unity Graphics与Fyrox

1. Bevy本地版本没有通用媒体产品，但Screenshot具typed RenderTarget、异步entity lifecycle、Captured event、GPU transfer buffer、row-unpadding和自动terminal despawn。它只证明单帧capture应该有明确request/result lifecycle，不能降低Media/Recorder标准。
2. Unity Graphics `CameraCaptureBridge`按Camera维护capture action set，URP `CapturePass`在RenderGraph末端读取正确方向的active color texture，并强制不可cull的unsafe pass。Zircon缺逐camera registry与capture-stage合同。
3. Fyrox本地版本的“Video”只出现在FBX内嵌texture record解析，没有可对标的player/clock/recorder。该负证据同样不能作为Zircon验收基线。

## 5. 必须保留的真实基础

1. 保留`GpuReadbackQueue`的三槽ring、预算、ticket、cancel/abort/shutdown、异步map、row unpack、panic containment和统计，但把它定位为通用readback stage，不扩写成Recorder。
2. 保留viewport generation mailbox和nonblocking poll；Recorder另建不会静默覆盖中间帧的有界队列与明确drop policy。
3. 保留`RenderCaptureSource/Report`、graph dump、frame profile和linear HDR distinction；扩展通过新CaptureSample合同完成，不继续膨胀模糊RGBA DTO。
4. 保留`ZrOwnedResultV2`显式release和Runtime owner lifetime；媒体sample需独立versioned lease、plane ownership和generation/epoch。
5. 保留App PNG writer的staging、flush/sync、atomic replace与failure cleanup；RecordingArtifact publication复用事务模式，但必须有container/sidecar/finalize/recovery。
6. 保留Play preview真实gateway调用和retained viewport currentness检查；Performance Capture命令必须复用真实controller并返回receipt。
7. 保留Symphonia importer的track validation、channel layout、scratch复用与bounded preallocation思路；流式Media provider不得复用“整文件解码到Vec”所有权模型。
8. 保留plugin package declaration、capability和NativeDynamic dist metadata模式；Media provider需要独立owner、许可、平台和安装证据。

## 6. P0当前状态

| ID | 状态 | 当前证据与必须动作 |
|---|---|---|
| P0-1 公共UI接受Media引用，Runtime却无媒体资源并降格为Data | **Open** | `.mp4/.webm/.mov`仍推断Media后映射Data。M0必须在真实MediaSource可用前结构化拒绝，或同一硬切引入identity/provider/consumer。 |
| P0-2 External audio公开接受PCM，实际voice adapter明确拒绝 | **Open** | submit只替换一块`Vec<f32>`，Kira adapter明确unsupported。必须先truthful fail，或完整实现timestamped bounded stream、device feedback和terminal receipt。 |
| P0-3 没有timestamped video sample与MediaTexture合同 | **Open** | RGBA/HDR capture不是decoder sample；format/planes/color/PTS/ownership/fence/MediaTexture全部缺失。 |
| P0-4 没有MediaClock、sample queue、seek epoch和A/V同步 | **Open** | simulation clocks没有media domain；player/track/queue/seek/sync产品类型为零。 |
| P0-5 Capture产品是单帧RGBA/PNG，Editor命令仍返回固定假结果 | **Open** | Play preview已接真实单帧capture，但Performance命令仍固定1234且Recorder全链缺失，未满足canonical断路的任何退出条件。 |

## 7. P1当前状态

### 7.1 Source、Provider、Open与安全

| ID | 状态 | 当前差距 |
|---|---|---|
| P1-1 建立稳定`MediaSourceAsset` | Open | Resource/ImportedAsset无identity、revision、locator或dependency。 |
| P1-2 建立versioned `MediaOpenOptions` | Open | 无scheme/header/cache/timeout/credential/provider options。 |
| P1-3 Provider registry需要probe与admission | Open | importer registry不等于runtime player provider registry。 |
| P1-4 Open必须异步且可取消 | Open | 无Media open operation、deadline、cancel或completion event。 |
| P1-5 Container probe工件缺失 | Open | Symphonia选择audio track但不发布container/track/color/duration probe artifact。 |
| P1-6 网络与协议安全政策缺失 | Open | 无URL redirect/private network/credential/cache策略。 |
| P1-7 恶意媒体输入边界缺失 | Open | audio importer仍可累计完整PCM；无媒体parser dimension/time/packet budget和fuzz。 |
| P1-8 Source dependency与cook政策缺失 | Open | UI Data降格不能证明cook后可播放。 |
| P1-9 Codec许可与分发事实缺失 | **Partial** | Opus importer已有NativeDynamic dist/ABI/missing-backend metadata；无Media codec license/patent/binary/platform closure。 |
| P1-10 Provider fallback语义缺失 | Open | 无forced provider、confidence、fallback order或decision receipt。 |
| P1-11 Media metadata与poster/thumbnail工件缺失 | Open | 无source-qualified metadata、poster、filmstrip或waveform。 |
| P1-12 Playlist与sidecar subtitle身份缺失 | Open | 无playlist entry、subtitle/metadata track identity。 |

### 7.2 Player、Track、Clock、Queue与同步

| ID | 状态 | 当前差距 |
|---|---|---|
| P1-13 建立严格Player状态机 | Open | 无状态、合法转换、command idempotence或snapshot。 |
| P1-14 事件必须generation-qualified | Open | capture generation不能替代Media session/event generation。 |
| P1-15 Controls能力查询缺失 | Open | 无duration/time/rate/seek/loop/range capability。 |
| P1-16 Track catalog缺失 | Open | 无audio/video/subtitle/metadata catalog。 |
| P1-17 Track与format选择缺失 | Open | 无language/role/default/forced/format selection。 |
| P1-18 Timestamp与time range类型缺失 | Open | `Duration` simulation time和capture generation不是Media timestamp。 |
| P1-19 Sample queue必须有界 | Open | readback queue不是per-track sample queue。 |
| P1-20 MediaClock domain缺失 | Open | real/virtual/fixed clock没有PTS correlation与media epoch。 |
| P1-21 A/V sync controller缺失 | Open | 无master/drift/drop/duplicate/resample。 |
| P1-22 Seek生命周期缺失 | Open | 无flush/preroll/keyframe/exact-fast/old epoch fence。 |
| P1-23 Buffering与live edge缺失 | Open | 无buffered/seekable ranges、watermark或live latency。 |
| P1-24 External audio sink必须真正流式 | Open | HashMap replace和explicit unsupported仍在。 |

### 7.3 Video Sample、Color、GPU与MediaTexture

| ID | 状态 | 当前差距 |
|---|---|---|
| P1-25 定义typed `VideoSampleFormat` | Open | 无plane format/layout。 |
| P1-26 定义coded/display geometry | Open | 无coded/output/aperture/aspect/orientation。 |
| P1-27 颜色与HDR metadata必须端到端 | **Partial** | internal HDR capture保留linear RGBA16F；source metadata、YUV range/matrix、ABI、conversion和output fidelity均缺。 |
| P1-28 Sample timing与decode flags缺失 | Open | 无PTS/DTS/duration/order/keyframe/discontinuity。 |
| P1-29 CPU sample ownership缺失 | **Partial** | V2 owned result有shape validation、显式release和owner lifetime；仍是单blob RGBA，无sample/plane lease、allocator/reuse和epoch。 |
| P1-30 GPU external sample ownership缺失 | Open | readback方向相反；无external surface、fence、decoder reuse或release callback。 |
| P1-31 Video conversion pipeline缺失 | Open | 无YUV conversion/deinterlace/scaler/tone/gamut map。 |
| P1-32 MediaTexture需要独立资源身份 | Open | ResourceKind/asset/runtime handle均无MediaTexture。 |
| P1-33 MediaTexture sample selection缺失 | Open | 无clock window、hold/clear/drop、late frame或same-frame consumer。 |
| P1-34 Resolution/format change重配缺失 | Open | 无decoder surface/pipeline generation和old-frame handoff。 |
| P1-35 MediaTexture mip与filter政策缺失 | Open | 普通Texture政策不能表达动态sample和conversion输出。 |
| P1-36 GPU资源预算与性能基线缺失 | **Partial** | readback有request/frame/pending bytes预算和stats；无decode surface/conversion/multi-stream/copy/power基线。 |

### 7.4 Capture、Recorder、Encoder、Mux与Editor

| ID | 状态 | 当前差距 |
|---|---|---|
| P1-37 建立typed CaptureSource | **Partial** | `RenderCaptureSource/Report`已typed；没有稳定camera/viewport/RT/final-output source handle、stage/color contract。 |
| P1-38 逐camera capture registry缺失 | Open | viewport handle和Unity式camera action registry不是同一能力。 |
| P1-39 Capture sample需要timestamp与metadata | **Partial** | generation/source/report/graph/profile存在；缺PTS/duration/sequence/format/color/camera/overlay identity。 |
| P1-40 Recorder状态机缺失 | Open | 无prepare/start/pause/resume/stop/finalize/error。 |
| P1-41 CFR/VFR pacing政策缺失 | Open | 无cadence、drop/duplicate、offline clock和audio correlation。 |
| P1-42 Encoder provider合同缺失 | Open | rav1e静态图片依赖不是encoder provider。 |
| P1-43 Muxer provider合同缺失 | Open | 无container/track negotiation和interleave/finalize。 |
| P1-44 Recorder队列与backpressure缺失 | **Partial** | readback source stage已有硬预算、cancel/abort和nonblocking poll；conversion/encode/mux/disk队列与drop reason均缺。 |
| P1-45 Recording artifact publication缺失 | **Partial** | 单PNG具有staging、sync、atomic replace和cleanup；无container/sidecar/finalize/probe/recovery receipt。 |
| P1-46 Editor Media Toolkit缺失 | Open | 无document/controller/transport/track/frame/decoder inspection。 |
| P1-47 Editor Recorder面板缺失 | Open | 无source/preset/output/job/session/artifact UI。 |
| P1-48 Preview与thumbnail产品链缺失 | Open | Play preview不是media poster/filmstrip/waveform cache。 |

### 7.5 Plugin、Diagnostics、测试与发布

| ID | 状态 | 当前差距 |
|---|---|---|
| P1-49 确定Media package owner | Open | 名为media的catalog分组只装texture/audio importer。 |
| P1-50 First-party装配必须可追溯 | Open | 无Media runtime/editor package、feature、binary或provider closure。 |
| P1-51 Maturity必须由能力门派生 | Open | importer Stable/Partial分类不能代表Media capability。 |
| P1-52 诊断stable codes缺失 | **Partial** | readback/UI resolver/Opus有typed error或diagnostic；无Media source/session/provider/track/stage stable code体系。 |
| P1-53 Media telemetry缺失 | **Partial** | readback in-flight/rejection stats与frame profile存在；无open/buffer/decode/sync/drop/reconfigure/record metrics。 |
| P1-54 日志与隐私政策缺失 | Open | 无URL/header/query/credential/content metadata redaction。 |
| P1-55 Deterministic provider fixture缺失 | Open | 无可生成PTS、stall、seek、track/color change的provider。 |
| P1-56 Malformed/fuzz矩阵缺失 | Open | 音频partial-frame test和shape validation不等于container/sample/subtitle/playlist fuzz。 |
| P1-57 A/V sync与seek golden缺失 | Open | 无Media clock/provider，不能建立golden。 |
| P1-58 GPU/color visual golden缺失 | Open | HDR texel test不覆盖YUV/color/HDR端到端视觉结果。 |
| P1-59 Recording fault-injection缺失 | **Partial** | readback cancel/abort/shutdown/budget和PNG encode/flush/sync/replace失败有测试；无encoder/mux/disk-full/process-interrupt/device-loss Recorder lane。 |
| P1-60 Cross-platform/package/release矩阵缺失 | Open | 无provider package可进入矩阵。 |

## 8. P2当前状态

12项全部Open：adaptive bitrate streaming、DRM/受保护媒体、hardware decode/encode调度、ultra-low-latency live media、camera/capture device ingestion、360/180/stereo projection metadata、spatial/multichannel media audio、timed metadata/subtitle/accessibility、remote broadcast/pixel streaming、nonlinear transcode/proxy、distributed deterministic encoding，以及跨引擎质量/性能基准。任何P2实施都不得越过P0、source/provider、sample/clock和release gates。

## 9. Authority断路清单

| 表面 | 当前写入/承诺 | 实际执行owner | 断路 |
|---|---|---|---|
| UI resource ref | `.mp4/.webm/.mov`推断Media | generic ResourceManager | Media被降为Data，无open/play consumer |
| builtin `media.rs` | Texture/Audio/Opus importer rows | asset importer registry | 文件分组名制造Media package错觉 |
| Symphonia import | 解码audio container/packet | SoundAsset importer | 完整PCM常驻，无stream/player/track/clock |
| External audio submit | 成功存入PCM block | Sound manager HashMap | voice adapter明确unsupported，成功对应silence/不可达 |
| Runtime Time | real/virtual/fixed/clock stamp | simulation runtime | 无Media time base、PTS、seek epoch或device correlation |
| Render capture | source/report/generation/HDR/readback | RenderFramework/GpuReadbackQueue | 无timestamped CaptureSample、camera registry或Recorder cadence |
| Runtime DLL frame V2 | owned RGBA result | dynamic session/gateway | ABI version升级未增加format/color/PTS/source/fence |
| Play preview | 真实default viewport capture | PlayController/gateway | 可保留单帧consumer，不是Media/Recorder |
| Performance Capture Frame | queued/Frame1234/CPU/GPU文本 | static feedback table | 未调用真实capture controller或artifact job |
| PNG publication | 原子单图文件 | App frame capture writer | 无连续sample、audio、encoder/mux/finalize/recovery |
| rav1e dependency | AV1 encoder crate存在 | ravif/AVIF image chain | 无Recorder caller、video settings、mux或artifact |

## 10. 目标Owner与合同

| Owner | 应拥有 | 不应拥有 |
|---|---|---|
| `zircon_runtime_interface::media` | versioned source/session/track/time/sample/capability/error DTO与ABI-safe lease | platform decoder对象、Editor view state |
| `zircon_runtime::core::media` | provider registry、open/session state、track/sample queues、clock/sync、budget/telemetry | FFmpeg/MF/AVF具体实现、Editor command |
| Media provider packages | probe、decode、platform surface、codec/license/platform capability | player UI、global resource authority |
| `graphics::media` | external sample import、conversion、surface pool、fence、MediaTexture install | demux/seek/session state |
| Sound streaming adapter | timestamped PCM consume、device position、watermark、underrun/overrun/EOS | media source/provider selection |
| Capture service | stable CaptureSource、stage selection、timestamped CaptureSample、source budget | encoder/mux/container policy |
| Recorder service | state/cadence/queue/backpressure、encoder/mux negotiation、finalize/recovery | raw RenderGraph internals |
| Editor Media Toolkit | document/session controller、transport/track/frame/diagnostics/preview | decoder state authority |
| Editor Recorder | preset/source/output/job/session/artifact controller | 每帧直接同步readback |
| App/package | provider closure、binary/license、headless/offline capability与release evidence | 依赖偶然存在即宣称可用 |

所有open、event、sample、seek、MediaTexture install、capture和recording artifact必须携带source/session identity、generation和必要epoch。所有队列必须声明count/bytes/time预算、admission、drop/backpressure、telemetry和terminal行为。任何late callback、旧seek epoch、旧decoder surface、旧capture generation或已取消job都不得覆盖新状态。

## 11. 分层重构里程碑

### M0 Truthfulness与owner硬切

删除或禁用Media->Data静默降格、External audio假成功和固定Capture Frame结果；冻结Media/Capture/Recorder owner、capability与stable error namespace。没有provider时所有入口结构化unsupported。

### M1 Stable Media Identity、Source、Provider与Probe

引入versioned MediaSource/OpenOptions、provider factory/probe/admission/fallback receipt、container/track metadata artifact、path/network/security/license/package政策。

### M2 Session、Track、Sample、Clock与Seek

建立generation-qualified player state/event、typed track catalog、rational timestamp/time range、bounded per-track queues、MediaClock、seek epoch、buffering和A/V sync policy。

### M3 Streaming Audio Sink

以有界timestamped PCM queue接通Kira/audio callback，提供device position、format negotiation、watermark、underrun/overrun/EOS、flush和shutdown drain。

### M4 Video Sample、GPU Conversion与MediaTexture

冻结plane/color/HDR/geometry/timing/ownership合同，接入CPU/GPU sample lease、conversion、surface pool/fence、clock-based selection、dynamic reconfigure与device-loss recovery。

### M5 Editor Media Toolkit与Asset Workflow

建立Media document/controller、transport、scrub/frame-step、track/subtitle、decoder/cache diagnostics、poster/filmstrip/waveform jobs、atomic cache和cook/reference workflow。

### M6 Capture Source与Recorder Core

在现有readback之上建立camera/viewport/RT/final/HDR/UI-overlay CaptureSource、timestamped CaptureSample、Recorder state、CFR/VFR/offline clock和分阶段bounded queues。

### M7 Encoder、Muxer与Durable Artifact

建立provider negotiation、actual settings receipt、audio/video interleave、container finalize/probe、sidecar/checksum、atomic publication、cancel/crash/disk-full recovery和artifact browser。

### M8 Platform Providers与发布资格

完成software/platform/hardware provider矩阵、binary/license/package closure、headless/offline模式、malformed/fuzz/fault/soak、device loss与release/rollback evidence。

### M9 Advanced Streaming、Devices与Distributed Workflows

在M0-M8 gates通过后再做ABR、DRM、live/capture device、hardware scheduling、360/stereo、timed metadata、broadcast、proxy/transcode与distributed encoding。

## 12. 验收门禁当前状态

| Gate | 状态 | 当前证据 |
|---|---|---|
| G01 Public truthfulness | Fail | Media继续降Data，External submit与静态Capture结果不真实。 |
| G02 Provider admission | Fail | 无Media provider/factory/probe receipt。 |
| G03 Open/close generation | Fail | 无Media session/open/close。 |
| G04 Probe security | Fail | 无恶意container/network required lane。 |
| G05 Track catalog/selection | Fail | 无catalog/selection。 |
| G06 Timestamp precision | Fail | 无Media timestamp/time base。 |
| G07 Bounded sample queues | Fail | readback budget不是per-track queue。 |
| G08 Media clock modes | Fail | simulation clock不满足Media clock。 |
| G09 A/V sync | Fail | 无sync controller。 |
| G10 Seek/discontinuity | Fail | 无seek epoch/flush/preroll。 |
| G11 External audio playback | Fail | Kira adapter明确unsupported。 |
| G12 Audio underrun/overrun | Fail | 无stream consumer和device feedback。 |
| G13 Video sample layout | Fail | 无NV12/P010/I420合同。 |
| G14 Color/HDR fidelity | Fail | RGBA16F distinction没有source metadata/conversion/golden。 |
| G15 GPU ownership/fences | Fail | 无decoder external surface/fence/release。 |
| G16 MediaTexture presentation | Fail | 无MediaTexture或clock-based selection。 |
| G17 Dynamic reconfiguration | Fail | 无decoder/conversion surface generation。 |
| G18 Decode/GPU performance | **Partial** | readback有预算和统计；无decode/conversion/multi-stream矩阵。 |
| G19 Capture source correctness | **Partial** | typed source/report/extent/HDR存在；无camera registry、orientation/color/stage完整矩阵。 |
| G20 Recorder pacing | Fail | 无Recorder clock/cadence。 |
| G21 Recorder admission/backpressure | **Partial** | readback阶段硬预算且poll不阻塞submit；后续四阶段不存在。 |
| G22 Encoder/mux negotiation | Fail | 无provider。 |
| G23 Durable finalize | **Partial** | 单PNG原子发布可靠；无container/sidecar/track/timestamp finalize。 |
| G24 Interrupted recording recovery | **Partial** | 单图失败清理和readback cancel/abort存在；无进程/encoder/mux/device recovery。 |
| G25 Editor command truth | Fail | 固定Frame1234仍在。 |
| G26 Media Toolkit workflow | Fail | 产品为零。 |
| G27 Preview artifacts | Fail | 无source/provider/version-qualified media cache。 |
| G28 Diagnostics/privacy | **Partial** | 有readback/UI/import typed diagnostics；无Media stable code与redaction。 |
| G29 Malformed/fuzz/fault matrix | **Partial** | readback/PNG/audio importer有局部failure tests；Media/encoder/mux/fuzz为空。 |
| G30 Cross-platform/device matrix | Fail | 无Media provider或device ingestion。 |
| G31 Headless/package/license | Fail | importer dist metadata不能证明Media runtime/Recorder package。 |
| G32 Release/rollback | Fail | 无Media schema/provider/artifact/preset可迁移。 |

## 13. 禁止的临时修补

1. 禁止只新增`ResourceKind::Media`却继续以Data payload、普通Texture或SoundAsset执行。
2. 禁止把Symphonia整文件解码、rav1e依赖存在或Opus dist metadata写成Media provider已接入。
3. 禁止让External source保持silence后把unsupported错误改成成功。
4. 禁止用simulation `delta_seconds`直接当PTS或MediaClock。
5. 禁止把所有视频帧先转RGBA8并逐帧`queue.write_texture`，从而固化CPU copy和色彩丢失。
6. 禁止为MediaTexture复用普通Texture identity而没有sample lease、clock selection和generation install。
7. 禁止每帧调用`ZrRuntimeFrameV2`或单帧capture再拼PNG序列冒充Recorder。
8. 禁止通过无限queue、额外线程或扩大readback ring掩盖encoder/disk backpressure。
9. 禁止先制作静态Media/Recorder面板、固定progress或成功文本，再等待Runtime未来接线。
10. 禁止直接把codec/platform对象泄漏到Editor asset/UI，绕过provider capability和lifecycle。
11. 禁止以microbenchmark、截图或单个HDR texel test宣称性能/画质优于Unreal。
12. 禁止在缺malformed/fault/package/license/rollback证据时把maturity升级为Stable/Complete。

## 14. 实施顺序与跨计划边界

1. Editor36拥有Media/Playback/Capture/Recorder产品链和本报告状态；Editor119/Runtime99zy继续拥有Cinematic/Take/Movie Render orchestration，不重复Media sample/encoder owner。
2. Runtime99zn拥有Sound device/mixer/streaming基础；本报告只要求Media timestamped PCM adapter，不接管完整Audio重构。
3. Runtime90拥有neutral RHI、submission/completion/readback/device loss；Media GPU surface/fence必须建立在其qualified identity上，不能另建第二套GPU lifetime。
4. Editor156拥有普通Texture/RenderTarget/Sampler/artifact/streaming；本报告只拥有动态video sample、conversion和MediaTexture语义。
5. Editor120拥有gateway/session/foreign output通用代际与shutdown；Media ABI lease应复用其owner lifetime，但不能继续使用无媒体元数据的frame DTO。
6. Editor144/147拥有Frame Debugger与Diagnostics产品；Performance Capture命令接线应共享真实capture/artifact controller，不复制第二套静态反馈。
7. 实施必须按M0到M8顺序推进。M0 truthfulness未关闭前，不接codec、不建静态toolkit；M2 sample/clock未关闭前，不接GPU MediaTexture；M6 recorder state未关闭前，不接encoder/mux。

## 15. 本轮产出边界

本轮只新增current-source review、状态表、owner断路、分层里程碑和门禁，不修改任何Runtime/Interface/Plugin/App/Editor production代码或tests。报告中的`review_complete`表示选定范围已完成静态取证与差距建账，不表示Media/Recorder实现完成，也不表示任何动态资格门通过。

后续实现前必须重导309文件manifest并复核共享working tree drift；尤其检查`ZrRuntimeFrameV2`、readback budget、Play preview、Performance Capture反馈、Sound External adapter、catalog/App feature和audio importer终态。性能和表现优于Unreal只能在同功能、同内容、同平台、同codec/quality、同统计方法下完成correctness、latency、memory、copy、power、stability和visual/audio golden后声明。
