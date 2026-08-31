---
title: Runtime Video、MediaSource、MediaPlayer、Track、Clock、MediaTexture、Playback 与 Capture 当前工作树复审
category: zircon_runtime
report_id: Runtime177
review_date: 2026-08-30
baseline_head: 248b1d484a16f3826933ddb12ad0c75f0ae06223
verification_head: 248b1d484a16f3826933ddb12ad0c75f0ae06223
supersedes_currentness_of: []
related_editor_owner:
  - docs/plans/optimize/zircon_editor/237-editor-video-media-source-player-track-clock-media-texture-playback-capture-recording-current-working-tree-review.md
related_code:
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_runtime/src/asset/assets/imported.rs
  - zircon_runtime/src/asset/assets/sound.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/asset_rows/media.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/importer_classification/media.rs
  - zircon_plugins/audio_importer/runtime/src/lib.rs
  - zircon_plugins/audio_importer/dist/src/lib.rs
  - zircon_plugins/opus_importer/runtime/src/lib.rs
  - zircon_runtime/src/core/framework/render/capture.rs
  - zircon_runtime/src/graphics/runtime/render_framework/capture_frame
  - zircon_runtime/src/graphics/runtime/render_framework/environment_capture_scheduler.rs
  - zircon_app/src/entry/runtime_entry_app/frame_capture.rs
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Media/Public/IMediaPlayer.h
  - dev/UnrealEngine/Engine/Source/Runtime/Media/Public/IMediaControls.h
  - dev/UnrealEngine/Engine/Source/Runtime/Media/Public/IMediaSamples.h
  - dev/UnrealEngine/Engine/Source/Runtime/Media/Public/IMediaTracks.h
  - dev/UnrealEngine/Engine/Source/Runtime/Media/Public/IMediaTextureSample.h
  - dev/UnrealEngine/Engine/Source/Runtime/Media/Public/IMediaPlayerFactory.h
  - dev/UnrealEngine/Engine/Source/Runtime/Media/Private/MediaClock.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/MediaUtils/Public/MediaSampleQueue.h
  - dev/UnrealEngine/Engine/Source/Runtime/MediaAssets/Public/MediaSource.h
  - dev/UnrealEngine/Engine/Source/Runtime/MediaAssets/Public/MediaPlayer.h
  - dev/UnrealEngine/Engine/Source/Runtime/MediaAssets/Public/MediaTexture.h
  - dev/UnrealEngine/Engine/Source/Runtime/MovieSceneCapture/Private/FrameGrabber.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/MovieSceneCapture/Private/VideoCaptureProtocol.cpp
  - dev/godot/scene/resources/video_stream.cpp
  - dev/godot/scene/gui/video_stream_player.cpp
  - dev/godot/servers/movie_writer/movie_writer.cpp
  - dev/godot/servers/movie_writer/movie_writer_pngwav.cpp
  - dev/bevy/crates/bevy_render/src/view/window/screenshot.rs
  - dev/Fyrox/fyrox-impl/src/resource/fbx/scene/video.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Utilities/CameraCaptureBridge.cs
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Runtime177 · Video/MediaSource/Player 与 Capture 当前工程化差距

## 1. 结论

当前 Zircon 没有 Video/Media runtime 产品域。运行时资源枚举只有 `Sound`、`Texture` 和 Animation 系列，没有 `Video`、`MediaSource`、`MediaPlayer`、`MediaTrack`、`MediaTexture` 或可绑定到材质的时间变化视频资源。`zircon_runtime/src/asset/assets/imported.rs:21-147` 的 `ImportedAsset` 可以导入 Sound/Texture，却没有视频变体；`zircon_runtime_interface/src/resource/marker.rs:8-31` 的 `ResourceKind` 同样没有 Video/Media；Editor 的 `canonical_resource_kind_id` 也只映射到 `sound`、`texture` 和 animation 类型。

已有音频导入器是有价值的底座：`audio_importer/runtime/src/lib.rs:32-107` 对 metadata 驱动的预分配设上限，使用 Symphonia 解 WAV/OGG，并有解码、溢出和分配性能测试；Opus importer 有插件声明和失败诊断。但这只是离线音频导入，不是媒体播放协议，也没有音视频共同时钟、解码线程、压缩包/网络流、音频环形缓冲、视频帧队列或 seek 语义。现有 `SoundAsset`/sound timeline 应作为 audio provider，不应被命名复用来冒充 media player。

渲染捕获也是真实但不同的能力。`zircon_runtime/src/core/framework/render/capture.rs:8-106` 的 `RenderCaptureReport`/`CapturedFrame`/`CapturedHdrFrame` 携带 target、source、output size、generation、RGBA/HDR、graph dump 和 profile JSON；`graphics/runtime/render_framework/capture_frame` 通过 wait/poll readback 区分同步边界；`environment_capture_scheduler.rs:9-170` 有 capacity、generation stale rejection、supersession、cancel、failure telemetry。`zircon_app/src/entry/runtime_entry_app/frame_capture.rs:22-228` 还把单帧 RGBA 编成 PNG，使用 staging file、flush、sync、atomic rename/ReplaceFile。以上不能替代媒体系统：它没有 PTS、帧率锁定、音频采样、编码器、队列背压、连续 frame receipt、capture session、movie output 或 crash-resumable recording。

因此本报告不把局部音频和截图功能判定为 Video 已完成，而是记录跨资源、时钟、样本、GPU 和输出合同的缺口。当前报告树中没有可作为 Runtime Video/Media canonical owner 的旧报告；本报告建立新的 runtime 基线。新增 **30 项 P1（30 Open）**、**12 项 P2（12 Open）**、**26 道资格门（24 Fail / 2 Partial / 0 Pass）**，不新增 P0。

## 2. 当前源码证据

### 2.1 资源 taxonomy 与 import closure 是空的

- `zircon_runtime_interface/src/resource/marker.rs:8-31,53-113` 没有 `VideoMarker`、`MediaSourceMarker`、`MediaTextureMarker`、`MediaPlayer` 或 track/sample kind。任何资源注册、序列化、插件 capability 和 editor asset type 都无法表达视频。
- `zircon_runtime/src/asset/assets/imported.rs:21-147` 的枚举和 `asset_kind` 映射没有 Video 分支，因而没有 importer output、direct references、dependency closure、cook artifact 或 LKG reload。
- `zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/asset_rows/media.rs:5-39` 只有 Texture Importer、Audio Importer、Opus Audio Importer 三行；`importer_classification/media.rs:7-29` 只把 texture/audio 标成 Partial。没有 video container/codec/plugin、backend capability 或 platform matrix。
- `zircon_plugins/audio_importer/runtime/src/lib.rs:107-190` 入口是 `import_wav`/`import_symphonia_audio`，最终产物为 SoundAsset；代码中的 `MediaSourceStream` 是 Symphonia 的输入流类型，不是 Zircon 媒体 source contract，不能据此推断存在播放器。

### 2.2 音频导入有边界，但没有 A/V playback contract

音频 importer 的 `MAX_AUDIO_SAMPLE_PREALLOCATION` 和 overflow checks 是可保留的工程实践；它仍将完整解码样本累积进 `Vec<f32>`，没有 resident/streaming budget contract（源码注释明确说该合同尚未落地）。没有压缩块索引、按时间范围读取、取消、优先级、设备 clock、underrun/overrun policy、decoder lifetime、seek epoch 或 audio sample receipt。Opus plugin 的 descriptor/diagnostic tests 验证插件注册和坏输入，不验证运行时 audio sink、streaming、latency 或同步。

这意味着 Sound timeline（已经在 Runtime168 复审）可以提供音频事件 adapter，但不能直接承担视频中的 track selection、A/V clock master、buffering、rate change、loop、scrub 或 output mixing。

### 2.3 Render capture 是单帧 readback，不是媒体输出

- `CapturedFrame` 的 `generation` 是 frame/readback provenance，非媒体 PTS 或 decode sequence；RGBA8/HDR16F 只描述两个捕获像素格式，没有 YUV/NV12/P010、HDR metadata、color primaries、transfer function、rotation、pixel aspect 或 external texture handle。
- `capture_frame.rs:23-31,83-103` 在需要时等待 readback completion，poll 路径用 `TryLock`，锁竞争返回 `Ok(None)`。这对 UI/诊断 capture 合理，对固定帧率 recording 不够：没有 per-request deadline、dropped-frame policy、bounded multi-frame ring、consumer backpressure 或 deterministic retry。
- `environment_capture_scheduler.rs:104-170,241-399` 的容量为单 active work item 和有限 pending queue；它能拒绝 stale generation、supersede/cancel 并统计 failed/succeeded，但 work item 是 environment capture，不携带 sequence/shot/frame/sample/tile/pass/output codec，也没有连续 session 的 ordering/flush/drain。
- `frame_capture.rs:22-72,137-228` 只写独立 PNG。staging/flush/sync/atomic rename 解决文件耐久性，未解决编码器生命周期、container header/finalization、audio mux、partial movie recovery、frame numbering、timecode、multi-view 或 output manifest。

### 2.4 参考引擎已经拆出完整契约

Unreal 的 `IMediaPlayer` 将 Open/Close、Cache、Controls、Samples、Tracks、View、Stats 和 asynchronous event sink 分开；`IMediaControls` 有 Closed/Error/Paused/Playing/Preparing/Stopped 状态、Pause/Resume/Seek/Scrub/PlaybackRange、buffering/connecting；`IMediaSamples` 分别 FetchAudio/Video/Subtitle/Caption/Metadata 并提供 Flush、Peek、time range 和 bounded sample queue；`IMediaTracks` 描述音频/视频轨道格式、数量、选择和 duration；`IMediaPlayerFactory` 负责 URL playability、player creation、platform、feature capability。`MediaClock.cpp` 以 Fetch/Input/Output/Render 多阶段 fan-out clock 驱动 sinks，`MediaTextureSample` 还表达 YUV、压缩格式、HDR、orientation、aspect ratio 和 GPU texture conversion。

Godot `VideoStream`/`VideoStreamPlayer` 把资源、节点生命周期、play/pause/seek、loop、finished/error 信号和纹理输出分开；`MovieWriter`/`movie_writer_pngwav.cpp` 把固定帧率视频采样与 WAV 音频混音、flush/finalize 分开。Bevy screenshot API 通过 render-world extraction 和异步 sender 返回 image receipt，而不是同步拿最后一帧。Fyrox FBX video 资源说明 importer 需要把视频引用和 scene/material/animation dependency 绑定。Unity `CameraCaptureBridge`/capture utilities 要求 capture context 与 output identity 一起穿过 render graph。

## 3. P1 重构任务

| ID | 当前问题 | 必须完成 |
|---|---|---|
| RT-MEDIA-01 | 没有媒体 domain owner | 新建 per-runtime `MediaService`，明确 decoder thread、render thread、audio thread、shutdown/drain、service handle 和 generation。 |
| RT-MEDIA-02 | 没有 ResourceKind | 增加 `Video`/`MediaSource`/`MediaTexture`/`MediaPlaylist` 等稳定类型或明确 source/subasset 分层，并完成 marker、serde、asset kind、editor type、catalog、ABI 映射。 |
| RT-MEDIA-03 | 没有 source schema | 定义 file/URI/stream/webcam/image-sequence source、headers/options、desired player、license/security policy、validation 和 provenance。 |
| RT-MEDIA-04 | 没有 player factory | 对齐 URL capability、platform/backend/codec confidence、factory lifetime、plugin registration、feature matrix 和 deterministic selection。 |
| RT-MEDIA-05 | 没有 asynchronous player state | 建立 Closed/Preparing/Playing/Paused/Stopped/Error/Buffering/Connecting 状态、typed event sink、open/close cancellation 和 error retention。 |
| RT-MEDIA-06 | 没有 track model | 音频/视频/字幕/metadata/text track 具 stable id、format list、duration、language、selection、fallback 和 per-track diagnostics。 |
| RT-MEDIA-07 | 没有 sample abstraction | 建立 timestamp/duration/sequence/epoch/format/color metadata、audio/video/overlay/binary sample trait，禁止用 `Vec<f32>` 或 raw RGBA 替代样本合同。 |
| RT-MEDIA-08 | 没有 sample queue | 使用 bounded multi-producer/single-consumer queue，定义 drop/flush/peek/fetch-best、watermark、memory budget、priority 和 telemetry。 |
| RT-MEDIA-09 | 没有 media clock | 引入 monotonic media time、qualified frame/PTS、rate、pause、seek epoch、audio-master/video-master/external-clock 模式和 drift correction。 |
| RT-MEDIA-10 | 没有 seek/scrub semantics | seek 必须取消旧 decode epoch、清理 queue、等待 keyframe/index、发布 completion/error receipt；scrub 不得阻塞主线程。 |
| RT-MEDIA-11 | 没有 decoder backend capability | 建立 native/FFmpeg/平台后端边界、sandbox、codec/container registration、hardware decode、device loss、fallback 和 ABI version。 |
| RT-MEDIA-12 | 音频 importer 不是 streaming | 将完整样本 `Vec<f32>` 拆成 resident/streaming/chunk/index artifact，加入 range decode、ring buffer、underrun、latency 和 memory admission。 |
| RT-MEDIA-13 | 没有 video importer | 支持 container probe、codec metadata、keyframe index、color/HDR/orientation、thumbnail/proxy、dependency and cook artifact。 |
| RT-MEDIA-14 | 没有 MediaTexture | 建立 player-to-texture sink、GPU upload/external texture、YUV conversion、mip/resize、auto-clear、last-good frame、material binding 和 lifetime fence。 |
| RT-MEDIA-15 | GPU frame ownership 不完整 | sample 必须携带 device/queue/fence/generation，避免 decoder buffer 被过早回收；跨线程必须有 explicit acquire/release。 |
| RT-MEDIA-16 | 没有 audio sink/mixer contract | 将 track samples 接入 audio device with bus/volume/pan/3D/spatial policy，明确 clock master、latency、device change 与 mute behavior。 |
| RT-MEDIA-17 | 没有 subtitle/metadata output | 为 caption/subtitle/text/metadata 建立 time range、locale、style、script/security 和 consumer event receipts。 |
| RT-MEDIA-18 | 没有 looping/rate/range | playback range、loop count、reverse/thinning、rate limits、end-of-stream 和 completion event 应由 player artifact 决定。 |
| RT-MEDIA-19 | capture 没有 session | 把单帧 `CapturedFrame` 上升为 capture session/request/receipt，包含 source, PTS, frame index, sample index, view, pass, tile, color/HDR metadata。 |
| RT-MEDIA-20 | capture 没有固定步进 | 增加 fixed-step/offline clock、frame duplication/drop policy、render warmup、deterministic simulation fence 和 deadline。 |
| RT-MEDIA-21 | 没有 movie encoder | 建立 image sequence/video encoder/audio encoder/muxer provider，支持 codec/container capability、bitrate/GOP/keyframe、HDR、multi-view 与 finalization。 |
| RT-MEDIA-22 | 没有 A/V mux | 音视频按 PTS 进行 interleave、drift correction、gap/silence policy、flush and close；失败必须保留可诊断 partial artifact。 |
| RT-MEDIA-23 | 没有 recording recovery | journal、staging manifest、atomic finalize、checkpoint、resume/reindex、crash cleanup、disk budget 和 corruption diagnostic。 |
| RT-MEDIA-24 | 没有 network/live source policy | URL sandbox、redirect/auth/timeout/retry、jitter buffer、clock discontinuity、backpressure、offline fallback 和 telemetry。 |
| RT-MEDIA-25 | 没有 security/privacy boundary | 摄像头/mic permission、DRM/output protection、path/URI allowlist、untrusted container limits、decode sandbox 和 user consent。 |
| RT-MEDIA-26 | 没有 render/capture provenance | 将 `RenderCaptureReport` 与 media output receipt 关联，避免把 graph dump/profile/generation 当成 codec/frame identity。 |
| RT-MEDIA-27 | 没有 runtime integration | MediaService 纳入 world/app phases，明确 update/fetch/input/output/render 顺序，禁止多个 owner 竞争 clock 或 sample queue。 |
| RT-MEDIA-28 | 没有 save/network/replay semantics | source/player state、track selection、media time/seek epoch、loop/cue cursor 和 capture checkpoint 能序列化、复制、重放且拒绝 stale receipt。 |
| RT-MEDIA-29 | plugin dist 无媒体 provider | runtime/editor plugin manifest 必须列出 codec/backend/source/texture/encoder capabilities、commands/events、ABI conformance 和 missing-backend fail-closed。 |
| RT-MEDIA-30 | 没有规模与故障证明 | 对 4K/8K HDR、多音轨、长视频、网络抖动、seek storm、100 路 player、GPU readback saturation、disk full、device loss、crash/reopen 做 P99/soak/故障注入。 |

## 4. P2 完整度任务

| ID | 必须补齐 |
|---|---|
| RT-MEDIA-P2-01 | color management、ICC/primaries/transfer、HDR10/HLG metadata 与 tone-map policy。 |
| RT-MEDIA-P2-02 | 360/VR/stereo video、projection、eye selection 与 late-latch。 |
| RT-MEDIA-P2-03 | image sequence、live webcam、screen capture、NDI/remote source adapter。 |
| RT-MEDIA-P2-04 | thumbnail/proxy generation、waveform/spectrogram 与 cache invalidation。 |
| RT-MEDIA-P2-05 | subtitle authoring/import、font fallback、RTL/locale 与 accessibility。 |
| RT-MEDIA-P2-06 | media analytics、buffer health、decode/render latency、dropped frame and A/V drift dashboard。 |
| RT-MEDIA-P2-07 | media asset virtualized storage、range IO、HTTP cache、signed CDN artifact。 |
| RT-MEDIA-P2-08 | deterministic test vectors for container/codec/color/seek/loop/timecode。 |
| RT-MEDIA-P2-09 | platform camera/mic hot-plug、device route changes and permission revocation. |
| RT-MEDIA-P2-10 | editor preview proxy and low-resolution adaptive decode. |
| RT-MEDIA-P2-11 | media localization, alternate audio, commentary and forced subtitle policy. |
| RT-MEDIA-P2-12 | output presets, metadata sidecars, content hash and reproducible encode manifests. |

## 5. 资格门

| Gate | 当前结果 | 证据/通过条件 |
|---|---|---|
| RT-MEDIA-G01 | Fail | ResourceKind/ImportedAsset/AssetTypeId 没有 Video/Media 类型；通过需完整 serde/import/editor/cook roundtrip。 |
| RT-MEDIA-G02 | Fail | 没有 source/player/factory API；通过需 URI/file/live source 可异步 open/close 且错误可观察。 |
| RT-MEDIA-G03 | Fail | 没有 player state/event contract；通过需所有状态转换和取消具有 receipt。 |
| RT-MEDIA-G04 | Fail | 没有 track selection/format/duration；通过需多音轨、多视频轨、字幕 fixture。 |
| RT-MEDIA-G05 | Fail | 没有 timestamped sample trait；通过需 audio/video/subtitle/metadata 样本跨线程验证。 |
| RT-MEDIA-G06 | Fail | 没有 bounded media queue；通过需 watermarks、drop/flush、OOM/underrun tests。 |
| RT-MEDIA-G07 | Fail | 没有 media clock/seek epoch；通过需 pause/rate/scrub/reverse/loop drift test。 |
| RT-MEDIA-G08 | Fail | 音频 importer 仍以完整 Vec 为中心；通过需 resident/streaming/range decode benchmark。 |
| RT-MEDIA-G09 | Fail | 没有 video importer/container/codec artifact；通过需 keyframe/color/HDR metadata roundtrip。 |
| RT-MEDIA-G10 | Fail | 没有 MediaTexture/GPU sink；通过需 YUV/external texture and device-loss test。 |
| RT-MEDIA-G11 | Fail | 没有 audio device sink/A/V sync；通过需 long-run drift and underrun budget。 |
| RT-MEDIA-G12 | Fail | 没有 captions/subtitle/metadata consumer；通过需 time-range/locale/reverse behavior。 |
| RT-MEDIA-G13 | Fail | capture 只有单帧 RGBA/HDR readback；通过需 continuous request/receipt with PTS/frame identity。 |
| RT-MEDIA-G14 | Fail | capture scheduler single-work-item semantics；通过需 bounded queue/backpressure and multi-output fairness。 |
| RT-MEDIA-G15 | Fail | PNG writer 无 movie encoder/muxer；通过需 valid container with finalized A/V streams。 |
| RT-MEDIA-G16 | Fail | 无固定步进/offline render clock；通过需 bit-identical frame sequence under load。 |
| RT-MEDIA-G17 | Fail | 无 checkpoint/recovery；通过需 crash/disk-full/reopen recoverable partial artifact。 |
| RT-MEDIA-G18 | Fail | 无 network/live policy；通过需 timeout/retry/jitter/discontinuity fixture。 |
| RT-MEDIA-G19 | Fail | 无 security/privacy/DRM boundary；通过需 denied path/device/permission tests。 |
| RT-MEDIA-G20 | Fail | capture provenance 未与 media output 关联；通过需 sequence/shot/frame/sample/pass manifest。 |
| RT-MEDIA-G21 | Fail | 无 save/network/replay participant；通过需 seek/loop/selection replay equivalence。 |
| RT-MEDIA-G22 | Fail | 无 plugin backend capability closure；通过需 missing backend fail-closed and ABI matrix。 |
| RT-MEDIA-G23 | Partial | PNG staging write 已有 flush/sync/atomic commit；仍需 session journal、mux finalization 和 resume。 |
| RT-MEDIA-G24 | Partial | capture scheduler 已有 stale generation、cancel、capacity、telemetry；仍需媒体时钟、PTS 和连续输出。 |
| RT-MEDIA-G25 | Fail | 无 4K/8K、100 player、seek storm、long take P99 benchmark。 |
| RT-MEDIA-G26 | Fail | 无跨平台 backend/codec/color test matrix。 |

## 6. 推荐重构顺序

1. 先冻结 `ResourceKind`、asset/importer、plugin capability 和 `MediaService` 的 owner/ABI；没有这些合同，不应继续添加单个视频按钮或材质字段。
2. 再实现 player factory、source/track/sample/clock/queue 四层，并把现有 SoundAsset 导入重构为 resident/streaming provider，保留其 bounded allocation 和坏输入测试。
3. 之后实现 video importer、MediaTexture GPU sink、字幕/metadata sink 和 A/V sync；所有跨线程对象以 generation/epoch/fence 传递。
4. 最后把现有 `CapturedFrame`/environment scheduler 抽象为 capture session，接入 fixed-step、encoder/muxer、checkpoint/recovery 和 output manifest，再由 cinematic Runtime176 消费。
5. 任何阶段都要以结构化 receipt、failure injection、P99/soak 和 asset reopen/replay gate 为完成条件；静态 queued 文案或单帧 PNG 成功不能关闭上述缺口。
