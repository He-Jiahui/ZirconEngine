---
title: Editor Video、MediaSource、Player、Track、Clock、MediaTexture、Playback、Capture 与 Recording 当前源码复核
category: zircon_editor
report_id: Editor110
review_date: 2026-08-26
baseline_head: 590376671b8745a0d230304c94432857c669bfbd
baseline_epoch: 524
canonical_owner: Editor36
refreshes:
  - docs/plans/optimize/zircon_editor/36-video-media-source-player-track-clock-media-texture-playback-capture-recording-authoring-review.md
related_code:
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_runtime/src/asset/assets/imported.rs
  - zircon_runtime_interface/src/ui/template/asset/resource_ref/resource_kind.rs
  - zircon_runtime/src/ui/template/asset/resource_ref
  - zircon_runtime/src/core/runtime/time.rs
  - zircon_runtime/src/core/framework/sound
  - zircon_plugins/sound/runtime/src/service_types
  - zircon_plugins/sound/runtime/src/timeline
  - zircon_runtime/src/core/framework/render/capture.rs
  - zircon_runtime/src/graphics/runtime/render_framework/capture_frame
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record
  - zircon_runtime/crates/zr_rhi_wgpu/src/gpu_readback_queue
  - zircon_runtime_interface/src/runtime_api/session/requests.rs
  - zircon_app/src/entry/runtime_entry_app/frame_capture.rs
  - zircon_editor/src/core/gateway
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback/ui_diagnostics/observability.rs
  - zircon_plugins/first_party_runtime_catalog
  - zircon_plugins/first_party_editor_catalog
tests:
  - zircon_runtime/crates/zr_rhi_wgpu/src/gpu_readback_queue/tests.rs
  - zircon_runtime/src/core/framework/render/capture.rs
  - zircon_runtime/src/core/framework/sound/tests.rs
  - zircon_runtime/src/graphics/runtime/render_framework/capture_frame/capture_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/capture_mailbox.rs
  - zircon_runtime_interface/src/runtime_api/session/requests.rs
  - zircon_app/src/entry/runtime_entry_app/frame_capture.rs
  - zircon_editor/src/core/gateway/session/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/environment/tests.rs
  - zircon_plugins/sound/runtime/src/service_types/automation_timeline.rs
  - zircon_plugins/sound/runtime/src/timeline/advance.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/08b-audio-runtime-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/09d-render-asset-streaming-residency-review.md
  - docs/plans/optimize/zircon_runtime/09h2-exposure-color-bloom-dof-motion-blur-ssr-terminal-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/17-sound-audio-clip-mixer-routing-effect-spatial-acoustic-timeline-authoring-review.md
  - docs/plans/optimize/zircon_editor/22-render-pipeline-frame-capture-lighting-bake-reflection-probe-post-process-debug-authoring-review.md
  - docs/plans/optimize/zircon_editor/25-runtime-diagnostics-performance-timeline-console-telemetry-observability-authoring-review.md
  - docs/plans/optimize/zircon_editor/35-texture-image-cubemap-render-target-sampler-compression-streaming-preview-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Media/Public/IMediaPlayer.h
  - dev/UnrealEngine/Engine/Source/Runtime/Media/Public/IMediaControls.h
  - dev/UnrealEngine/Engine/Source/Runtime/Media/Public/IMediaSamples.h
  - dev/UnrealEngine/Engine/Source/Runtime/Media/Public/IMediaTextureSample.h
  - dev/UnrealEngine/Engine/Source/Runtime/Media/Public/IMediaTracks.h
  - dev/UnrealEngine/Engine/Source/Runtime/Media/Public/IMediaClock.h
  - dev/UnrealEngine/Engine/Source/Runtime/MediaAssets/Public/MediaPlayer.h
  - dev/UnrealEngine/Engine/Source/Runtime/MediaAssets/Public/MediaSource.h
  - dev/UnrealEngine/Engine/Source/Runtime/MediaAssets/Public/MediaTexture.h
  - dev/godot/scene/resources/video_stream.h
  - dev/godot/scene/gui/video_stream_player.h
  - dev/godot/servers/movie_writer/movie_writer.h
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Utilities/CameraCaptureBridge.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/Passes/CapturePass.cs
  - dev/Fyrox/fyrox-impl/src/resource/fbx/scene/video.rs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 110 · Editor Video / MediaSource / Player / Track / Clock / MediaTexture / Playback / Capture / Recording 工程化差距

## 1. 结论

当前 Zircon 没有可称为视频/媒体系统的产品边界。当前 Runtime、Interface、Plugin、Catalog 与 App 中没有可执行的 `MediaPlayer`、`MediaSource`、`VideoFrame`、`VideoDecoder`、`MediaClock`、`MediaTexture`、`MovieWriter` 或等价 typed contract，也没有已装配的 FFmpeg、GStreamer、Media Foundation、VideoToolbox、dav1d、OpenH264 或 libvpx provider。这里不是“暂时少一个 codec”，而是 source/probe/provider/session/track/sample/clock/audio/video/editor/recording 全链条尚未建立。

已有底座必须准确命名而不能被夸大：Sound 有静态 clip、pause/seek/speed 的窄路径；FrameClock 有 real/virtual/fixed simulation clock；GPU readback 有 3-slot staging ring、row padding、ticket/cancel、异步 map 和 in-flight 统计；Viewport mailbox 有 generation 配对；`CapturedFrame` 区分普通 RGBA 与 HDR capture；App 能把单帧 RGBA 原子发布成 PNG。这些是可复用的静态音频、单帧诊断和 render evidence 基础，不是媒体播放或录制系统。

更危险的是接口产生了错误承诺。`UiResourceKind::Media` 会按字段名或 `.mp3/.ogg/.wav/.flac/.mp4/.webm/.mov` 推断，但 Runtime resolver 把 `Media` 与 `GenericAsset` 一并降级为 `ResourceKind::Data`；模板可以接受 media 字符串，却没有 open/decode/play/track/clock/frame/audio sink。Sound 的 `ExternalAudioSource` 只存一块 interleaved `Vec<f32>`，没有 PTS、duration、sequence、EOS、watermark、backpressure，voice sync 对它明确返回 unsupported。跨 Runtime ABI 的 `ZrRuntimeFrameV1` 只有 width、height、generation 和 owned RGBA bytes，不能承载 timestamped media sample。

Editor 的 Capture Frame 命令仍是固定 frame/CPU/GPU 文本；gateway 和 profiling artifact 只是 session/ownership/单张 UI screenshot 包装，没有 Media toolkit、transport、track inspector、decoder diagnostics、camera capture action、recorder state machine、encoder/muxer 或 finalizable recording artifact。不能通过补一张视频预览图、加一个 `media` ResourceKind 或把连续 PNG 写盘称为录制来收敛。

目标边界应明确分开播放与录制：

`MediaSourceAsset + versioned OpenOptions -> admitted MediaProvider -> generation-qualified MediaSession -> typed TrackCatalog + timestamped SampleQueues -> MediaClock/SyncController -> AudioStreamSink + VideoSampleConverter/MediaTexture -> runtime/editor consumers`

`CaptureSource -> timestamped CaptureSample -> bounded RecorderSession -> Encoder/Muxer provider -> atomic/finalizable RecordingArtifact`

两条链可共享 provider、PTS、color metadata 和 artifact infrastructure，但不得继续共享只有 RGBA bytes 的模糊 DTO。

## 2. 审查范围与证据

### 2.1 当前工作树物理范围

| 范围 | 文件 / 行 / 非空行 / bytes / tests | 指纹 |
|---|---:|---|
| Zircon Runtime/Editor/Plugin selected | **197 / 19,044 / 17,076 / 642,750 / 113** | resource identity、sound/time、capture/readback/ABI、gateway/profiling、catalog；`a1d9b9b35b5f1353ffb2d056c259010d86b915f56abba6dcecbf742be8e48f04` |
| Unreal/Godot/Fyrox/Unity reference | **15 / 4,513 / 3,834 / 151,813 / 0** | Unreal Media interfaces/assets、Godot VideoStream/VideoStreamPlayer/MovieWriter、Unity camera capture passes、Fyrox FBX video record；`541e3061919caa8d2de561d79e29428fc151b05d121d09bc912846f44d13caaa` |
| Zircon selected union | **212 / 23,557 / 20,910 / 794,563 / 113** | current physical working tree union；`05fa6515ff6641ad8e6011d9281c7e19998d830d7d3bf27087a1f6a99709f928` |

统计按 selected root 去重后按相对路径排序，以 UTF-8 内容计算 SHA-256；测试数只表示 test attribute 数量，不表示通过。当前 baseline epoch 为 524，工作树有与本轮无关的在途修改，实施前必须重新导出 manifest/fingerprint。没有运行 Cargo、codec backend、GPU playback、A/V sync、capture/recording、fuzz 或跨平台动态验证。

### 2.2 Resource、Source、Provider 与装配事实

1. `ResourceKind` 没有 MediaSource、MediaTexture、MediaPlaylist、Subtitle、RecordingPreset 或 VideoClip；`ImportedAsset` 没有 media variant。
2. 全仓 selected production path 未发现通用 MediaPlayer、VideoPlayer、VideoDecoder、VideoFrame、MediaClock 或 MovieWriter 类型。
3. Catalog/App 没有媒体 package/feature；当前 manifests/locks 不能证明任意解码、编码或平台 media backend 被装配。
4. `UiResourceKind::Media` 仅是 template inference，按名称或扩展名分类；resolver 最终把它降为 Data，形成公共接口与执行能力断路。
5. 没有 source file/URI/byte stream/device/live stream 的统一 open contract、sandbox/path/network policy、timeout、credentials 或 cache policy。
6. 没有 provider probe/priority/capability、scheme/extension/codec support、forced provider 或 fallback decision receipt。
7. 没有 container probe artifact：codec、track、duration、resolution、frame rate、color primaries/transfer、HDR metadata 均无持久化合同。
8. 没有 malformed container、oversized extent、decompression bomb、hostile URL、redirect、license/patent 或 redistribution diagnostics。
9. Editor type registry 没有 media subtype、waveform/filmstrip/video thumbnail provider 或专用 toolkit。
10. 资产引用、Scene/UI property、cook manifest 与 runtime handle 无法证明一个 media URI 是可打开、可播放的对象。

### 2.3 Player、Track、Audio 与 Clock 事实

1. 静态 sound clip 支持局部 pause/seek/speed/status，`SoundSourceInput::Clip` 和 automation timeline 是可保留的窄底座。
2. `SoundSourceInput::External`/`SoundExternalSourceBlock` 只有 sample rate、channel layout、interleaved samples 和 handle。
3. external block 缺 sample format version、frame count、PTS、duration、sequence、discontinuity、EOS、producer generation。
4. `submit_external_source_block_impl()` 按 handle 替换上一整块 samples，不是 bounded FIFO/ring；没有 watermark、backpressure、drop、underrun、overrun 或 lifetime receipt。
5. `sync_source_voice()` 对 External/Synth 返回 no Kira M1 runtime adapter；解码器即使生成 PCM 也没有生产 audio sink。
6. Sound timeline 按调用方 `delta_seconds` 推进，不连接 audio device clock、media PTS、sample position 或 drift estimator。
7. `FrameClock` 和 real/virtual/fixed clocks 是 simulation time，不含 media epoch、presentation deadline、seek flush、timecode、genlock 或 external clock。
8. 没有 Closed/Preparing/Ready/Playing/Paused/Buffering/Seeking/Ended/Error 的 player state machine、async open completion、close cancellation 或 late callback fence。
9. 没有 track catalog、selected track、variant、language/role/default/forced flags 或 audio/video/subtitle/metadata typed queues。
10. 没有 duration/time/rate/loop/seekable/live-edge/buffered ranges、keyframe/fast/exact seek、preroll、decoder drain 或 post-seek first-frame contract。
11. 没有 A/V master clock、audio resample、video drop/duplicate、late frame、buffer underrun 或 discontinuity recovery policy。
12. 没有 deterministic offline clock、capture timecode、fixed-frame stepping 或 replayable media session receipt。

### 2.4 Video Sample、Color、GPU 与 MediaTexture 事实

1. 没有 `VideoFrame`/`VideoSample`/`MediaTexture` 或等价 timestamped GPU handoff；普通 Texture upload 不能替代它。
2. 没有 NV12、P010、I420/YV12 等 plane layout、stride/offset、chroma subsampling/siting、odd extent 合同。
3. 没有 limited/full range、matrix、primaries、transfer、mastering display、CLL/FALL、rotation、clean aperture 或 sample aspect ratio。
4. 没有 PTS/DTS/duration/decode order/presentation order/keyframe/corrupt/discontinuous flags。
5. 没有 CPU ownership、GPU external image/import handle、fence/semaphore、release callback、surface pool 或 zero-copy/fallback policy。
6. 没有 YUV->RGB compute/render conversion、tone/gamut map、deinterlace、scaler、HDR output 或 device format tier。
7. 没有 MediaTexture front/back sample、late latch、render-thread handoff、generation install、resolution/format reconfigure 或 device-loss recovery。
8. `CapturedHdrFrame` 的 RGBA16F distinction 是 capture evidence 的正确底座，但它不是 decoder sample，也没有进入 media ABI/recording writer。
9. 现有 `GpuReadbackQueue` 服务单帧 readback；它没有 cadence、PTS、sample queue、encoder backpressure 或 audio synchronization。
10. Texture/RenderTarget 的 subtype、streaming 和 sampler 缺口已经由 Editor109 负责；本报告只新增动态 media sample 与 capture/recording 合同。

### 2.5 Capture、Recording 与 Editor 事实

1. GPU readback 有 3-slot staging ring、row padding unpack、ticket/cancel、async map completion、slot reuse rejection 和 in-flight stats。
2. Viewport mailbox 按 generation 配对 pending/completed，并只提升最新 ready frame；这是可复用的单帧正确性基础。
3. `CapturedFrame` 带 capture report、graph dump、profile 和 generation；capture 与诊断关联是可保留的 evidence boundary。
4. `ZrRuntimeFrameV1` 只有 width/height/generation/owned RGBA bytes，没有 stride、format、color、timestamp、duration、sequence、drop、camera、fence 或 release batch。
5. Editor gateway 会验证 ABI/RGBA shape 并要求 foreign owner release，但没有 stream subscription、recorder session 或 sample lifetime。
6. App frame capture 能 flush/sync 后原子替换单帧 PNG；没有连续 cadence、CFR/VFR、audio tap、encoder/muxer、finalize、repair 或 manifest。
7. profiling artifacts 在 materialize UI screenshot 前做 shared job admission，是可保留的 bounded job pattern；geometry/PNG 直接写最终路径，不能成为 recording artifact。
8. Editor “Capture Frame” operation 返回固定 Frame 1234/CPU/GPU 文字，没有调用 gateway、等待 GPU fence、选择 camera 或提交 artifact receipt。
9. 没有 Record/Stop/Pause/Finalize 状态机、resolution/fps/codec/container/audio track/output overwrite policy。
10. 没有 encoder queue watermark、disk pressure、drop/duplicate accounting、crash recovery、container validation、checksum 或 sidecar manifest。
11. 没有 camera-scoped capture action registry、render graph capture pass 与 output ownership 的 typed contract。
12. 没有 Media preview 的 transport bar、scrub、track/language chooser、buffer/cache/decoder diagnostics、frame probe 或 waveform/filmstrip。

## 3. P0：必须先关闭的断路（5 项，全部 Open）

### P0-1：Media identity 与 provider backend 不存在

没有 ResourceKind/ImportedAsset/catalog/App feature/MediaSource/MediaSession，也没有可执行 decoder/provider。必须先建立 provider-neutral source、probe、capability、license/platform admission，不能先添加一个字符串 Media kind。

### P0-2：`UiResourceKind::Media` 对外承诺被降为 Data

模板可以从名称或 `.mp4/.webm/.mov` 推断 Media，但 Runtime resolver 把它与 GenericAsset 一起映射为 Data；下游没有 open/play/track/frame/audio sink。必须 fail-close 或接通完整 typed reference，不能保持假阳性。

### P0-3：External audio 不是 media audio sink

External block 没有 PTS/queue/EOS/backpressure，提交会覆盖旧 block，voice sync 明确返回 unsupported。必须先有 timestamped PCM block、bounded ring、audio device clock、underrun/discontinuity receipt，才能让任何 decoder 声称可播放。

### P0-4：Capture ABI 与录制路径只支持单帧 RGBA evidence

`ZrRuntimeFrameV1` 缺 color/stride/PTS/duration/sequence/fence/ownership metadata；App 只原子发布单张 PNG，Editor command 只返回固定文本。必须先定义 `CaptureSample`、camera/view source、timestamp、format/color、bounded recorder queue、encoder/muxer/finalize/recovery。

### P0-5：Editor 媒体与录制产品完全缺失

没有 Media toolkit、preview、transport、track inspector、decoder diagnostics、recording document/preset、factory/controller/catalog/App closure。必须由真实 MediaSession/CaptureSample/RecordingArtifact 驱动 Editor，不得补静态 ZUI 或固定 feedback。

## 4. P1：Runtime、ABI、Editor 与发布（60 项，全部 Open）

1. 引入 `MediaSourceAsset` 与 versioned `MediaOpenOptions`，保留 unknown fields 与 migration。
2. 定义 URI/file/bytes/device/live source 的 sandbox、network、timeout、credential、cache policy。
3. 建立 provider probe、priority、capability、supported scheme/codec/container 与 fallback receipt。
4. 引入 source probe artifact，记录 tracks、codec、duration、extent、rate、color/HDR metadata。
5. 建立 MediaProvider plugin SPI 与 platform/license/redistribution admission。
6. 建立 generation-qualified `MediaSession` open/close/cancel/error lifecycle。
7. 建立 `MediaPlayerState` 与 ordered state/event journal。
8. 建立 typed TrackCatalog、track selection、language/role/default/forced flags。
9. 建立 audio/video/subtitle/metadata sample queue 和 producer ownership。
10. 为每个 sample 定义 PTS、DTS、duration、sequence、keyframe、discontinuity、EOS、corrupt flags。
11. 建立 bounded queue capacity、watermark、backpressure、drop/duplicate/underrun policy。
12. 建立 decoder worker、surface pool、shutdown drain 与 late completion generation fence。
13. 实现 exact/fast/keyframe seek、flush、preroll、decoder drain 与 first-frame receipt。
14. 实现 buffered ranges、live edge、stall/rebuffer、timeout 与 retry/backoff。
15. 建立 `MediaClock`、master selection、pause/rate/seek epoch 与 presentation deadline。
16. 接入 audio device clock、resampler、drift estimator、A/V sync controller。
17. 建立 video late/drop/duplicate、audio underrun、discontinuity recovery diagnostics。
18. 建立 deterministic offline clock、fixed-frame stepping、timecode/genlock adapter。
19. External audio block 改为 typed PCM format、frame count、PTS、duration、sequence。
20. External audio 使用 bounded ring/lease/watermark/backpressure，不覆盖上一块数据。
21. 将 External source 接入真正的 audio backend/sink，并提供 device loss/underrun receipt。
22. 建立 NV12/P010/I420 等 plane、stride、offset、subsampling、siting、odd extent contract。
23. 建立 color primaries/transfer/matrix/range/HDR metadata 与 conversion policy。
24. 建立 rotation、clean aperture、sample aspect ratio、coded/display extent 处理。
25. 实现 YUV->RGB compute/render converter、tone/gamut map、deinterlace、scaler provider。
26. 建立 `VideoSample` CPU/GPU ownership、fence/semaphore、release callback 和 surface pool。
27. 实现 zero-copy external image import 与 CPU/upload fallback，记录 actual path。
28. 建立 `MediaTexture` front/back sample、late latch、render generation、reconfigure。
29. MediaTexture 绑定 sampler/color/format policy，不复用静态 Texture source identity。
30. 处理 resolution/format/HDR change、device loss、decoder reset 与 old view eviction。
31. 将 render graph camera/view capture 定义为 typed `CaptureSource`，支持 multi-viewport/camera。
32. 扩展 capture ABI：format、stride、color、HDR、timestamp、duration、sequence、camera、fence、owner。
33. 建立 capture source admission、frame cadence、CFR/VFR、drop/duplicate 与 queue budgets。
34. 将 readback ticket、GPU fence、sample lifetime、release callback 绑定到 generation receipt。
35. 建立 capture CPU/GPU path、HDR/linear/encoded output 与 color conversion artifacts。
36. 建立 `RecorderSession` 状态机、preset、source selection、start/stop/pause/finalize。
37. 建立 Encoder/Muxer provider SPI，支持 capability/format/profile/bitrate/keyframe selection。
38. 建立 audio tap、video track、subtitle/metadata track 与 mux timestamp policy。
39. 建立 bounded encoder queue、watermark、disk pressure、drop/retry/backpressure telemetry。
40. 写入 atomic temp+fsync+rename recording artifact，禁止最终路径半成品。
41. 增加 interrupted recording repair、container validation、checksum、sidecar manifest。
42. 建立 source/recipe/provider/encoder/platform key、artifact provenance、quality/latency receipt。
43. 让 capture/recording artifact 支持 GC、size/age budget、cancel、rollback、resume policy。
44. 将 preview、waveform、filmstrip、thumbnail 由真实 sample/artifact provider 生成。
45. 建立 Media AssetTypeId、source/toolkit/factory/controller 与 create/open/import/reimport commands。
46. 建立 Media document revision、dirty/save/autosave/recovery/conflict/undo transaction。
47. 建立 player toolkit：transport、scrub、rate、loop、track/language、buffer/decoder diagnostics。
48. 建立 MediaTexture/track/sample inspector：PTS、format、color、queue、drop、latency。
49. 建立 recording toolkit：preset、camera/view、fps/resolution、codec/container/audio、output receipt。
50. 删除或 fail-close `UiResourceKind::Media` 的 Data fallback，改为 typed reference validation。
51. first-party runtime/editor catalogs 与 App 明确声明 provider、backend、factory、toolkit、target feature。
52. provider missing/unsupported/license/platform 任一项都在 admission 返回稳定 diagnostic code。
53. import/open/decode/seek/play/capture/record jobs 使用 bounded scheduler、cancel、shutdown drain。
54. 统一 event/diagnostic journal，关联 source/session/track/sample/generation/receipt。
55. 做 malformed container、oversized dimensions、hostile URL、partial read、decoder panic/OOM fuzz。
56. 做 audio/video queue、clock、seek、drop、device loss、late completion fault injection。
57. 做 visual/audio golden：YUV/color/HDR/rotation/alpha、A/V sync、frame cadence、seek boundary。
58. 做 clean/warm cache、cold open、slow I/O、network jitter、buffer starvation、100 sessions scale matrix。
59. 做 desktop/mobile/web backend capability、package/redistribution、headless client/server admission。
60. 建立与 Unreal/Godot 可比的 startup latency、buffering、A/V drift、GPU upload、recording throughput/quality 基准。

## 5. P2：长期能力（12 项，全部 Open）

1. hardware decode/encode、zero-copy surface 与 sparse GPU interop。
2. live streaming、adaptive bitrate、DASH/HLS/WebRTC provider 与 live-edge recovery。
3. HDR10/HLG/Dolby metadata、wide-gamut output、tone-map calibration 与 display profile。
4. subtitle/closed-caption/telemetry track、localization、accessibility 与 timed text layout。
5. multi-angle/multi-camera、synchronized capture group 与 genlock/timecode。
6. deterministic replay、offline render farm、frame-accurate seek 与 render-to-video artifact。
7. shared media cache、dedupe、range fetch、patch/chunk download 与 resumable recording upload。
8. procedural/video texture producer graph、dirty region、double buffer 与 budgeted update。
9. neural/upscale/interpolation provider，纳入同一 artifact/quality/fallback receipt。
10. collaborative media edit、clip/track transaction、field merge、lock/presence 与 review note。
11. crash-safe recording recovery、canary encoder rollout、old artifact pin/rollback/migration。
12. cross-engine media conformance suite，公开 codecs、color、clock、throughput、quality 方法学。

## 6. 分层重构顺序

### M0：Truthfulness 与 owner 收敛

将 `UiResourceKind::Media` 在没有 provider 时改为明确 unsupported；从 catalog/App/manifest 移除虚假 media capability；保留静态 sound/capture evidence，但禁止命名为 player/recorder。

### M1：Source、Probe、Provider 与 Session

建立 `MediaSourceAsset`、`MediaOpenOptions`、probe artifact、provider SPI、capability/license admission、generation-qualified MediaSession 和 ordered state journal。

### M2：Track、SampleQueue 与 Clock

建立 typed TrackCatalog、PTS/DTS/duration/sequence、bounded queue、A/V master clock、audio device sink、seek/discontinuity/drift policy；External audio 只能在此之后接通。

### M3：Video Conversion 与 MediaTexture

建立 planar video sample、color/HDR metadata、YUV converter、GPU surface/fence ownership、zero-copy/fallback 和 generation-safe MediaTexture。

### M4：Capture ABI 与 Recorder

扩展 capture sample/ABI，定义 camera/view source、timestamp/cadence、bounded recorder queue、Encoder/Muxer SPI、atomic artifact、finalize/recovery/manifest。

### M5：Editor Toolkit 与 Preview

将 source/probe/session/track/sample/artifact 接入 AssetType、document/transaction、transport/inspection、MediaTexture preview、recording preset/toolkit 和 first-party catalog/App。

### M6：Platform、Fault、Scale 与 Release

完成 codec/provider license、desktop/mobile/web capability、malformed/fault/determinism/visual/A-V/scale/headless package 门禁；未通过前 capability 维持 experimental/unsupported。

## 7. 验收门禁（32 门，当前全部 Fail）

1. Media source/probe/provider/session identity typed 且 generation-safe。
2. unsupported scheme/codec/platform/license 在 admission 早失败并带 stable code。
3. state/event order、open/close/cancel、late callback 不会覆盖新 session。
4. track catalog、selection、language/role/default/forced 结果可复现。
5. PTS/DTS/duration/EOS/discontinuity/keyframe 字段贯穿 decode、queue、render、audio、record。
6. bounded queues、watermark、backpressure、drop/duplicate/underrun 可观测且预算有界。
7. MediaClock master、pause/rate/seek epoch、audio device sync 与 drift 收敛。
8. exact/fast seek、flush/preroll/drain、post-seek first sample 有 deterministic receipt。
9. NV12/P010/I420 plane/stride/range/matrix/color/HDR 通过 CPU 数值 golden。
10. YUV conversion、tone/gamut/HDR、rotation/aperture、GPU/CPU fallback 通过 framebuffer golden。
11. MediaTexture front/back/fence/late-latch/reconfigure/device-loss 无旧 generation 使用。
12. capture source、camera/viewport、multi-view、timestamp/cadence、CFR/VFR 合同有效。
13. ABI stride/format/color/HDR/PTS/sequence/fence/ownership 与 foreign release 无泄漏。
14. readback cancel/slot reuse/device loss/late completion 不回退 mailbox generation。
15. Recorder state、preset、source selection、start/stop/finalize 可恢复。
16. Encoder/Muxer capability、profile/bitrate/keyframe、audio/video/subtitle timestamp 合法。
17. temp+fsync+rename、checksum、manifest、interrupted repair 无半成品。
18. preview/waveform/filmstrip 由真实 artifact 生成，cache key/GC/GC budget 正确。
19. Editor transaction、undo/save/autosave/recovery/conflict 不丢 source/recipe/preset。
20. missing toolkit/factory/controller/catalog/service 会拒绝 Editor plugin admission。
21. default/client/server/editor/App target 的 provider/backend feature 矩阵与 manifest 一致。
22. malformed/hostile/oversized/partial/decoder panic/OOM fuzz 无越界和无界分配。
23. audio/video queue、clock、seek、drop、device loss、cancel fault injection 可恢复。
24. capture/record shutdown drain、process kill、disk full、worker panic 不发布混合 artifact。
25. color/HDR/rotation/alpha、A/V sync、frame cadence、seek boundary 通过 visual/audio golden。
26. cold/warm open、slow I/O、network jitter、buffer starvation、many sessions 预算达标。
27. desktop/mobile/web format/device/provider/redistribution matrix 由 clean headless package 验证。
28. runtime 无 Editor cache 也能打开 source、播放或明确 unsupported。
29. capture/record artifact source/provider/encoder/platform key 与 provenance 完整。
30. diagnostics 可按 source/session/track/sample/generation/receipt 筛选导出。
31. performance 记录 startup/buffering/drift/GPU upload/VRAM/encoder throughput p50/p95/p99。
32. Stable/Complete 只能由 compile、registration、runtime、Editor、fault、platform、scale evidence 派生。

## 8. 禁止的临时修补

1. 禁止把 `UiResourceKind::Media` 或扩展名推断当作可播放媒体支持。
2. 禁止只增加 Media/Video ResourceKind 而没有 source/provider/session/track/sample/clock 消费者。
3. 禁止把静态 sound clip、External `Vec<f32>` 或 simulation delta 当作媒体 streaming audio。
4. 禁止把单帧 RGBA readback、PNG 序列或固定 Capture Frame 文本当作录制系统。
5. 禁止只添加 codec enum、manifest capability 或 UI transport 而没有真实 backend/receipt。
6. 禁止在 render thread 同步解码、读文件、等待 encoder 或跨帧复制无界 sample。
7. 禁止丢弃 PTS、color/HDR、stride、ownership、generation、EOS 和 discontinuity。
8. 禁止把普通 Texture/RenderTarget handle 直接宣称为 MediaTexture。
9. 禁止让 encoder/muxer 直接写最终路径或在取消/崩溃后留下可被误读的 recording。
10. 禁止用测试属性数量、静态 ZUI、ignored screenshot 或手工播放器替代 32 门资格。

## 9. 本轮产出边界

本轮只新增 Editor110 review、索引与分层计划，没有修改 Runtime、Editor、Interface、Plugin、App 或 tests production code，也没有运行 Cargo、codec/GPU playback、A/V sync、capture/recording 或跨平台动态验证；未查询或实时跟踪协调器。实施必须从 M0 开始，先重算当前 197-file manifest/fingerprint、恢复编译基线并建立 provider fixture，再实现任何 Media UI 或 codec 选项。
