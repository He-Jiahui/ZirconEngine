---
title: Editor Video、MediaSource、MediaPlayer、Track、Clock、MediaTexture、Playback、Capture 与 Recording 当前工作树复审
category: zircon_editor
report_id: Editor237
review_date: 2026-08-30
baseline_head: 248b1d484a16f3826933ddb12ad0c75f0ae06223
verification_head: 248b1d484a16f3826933ddb12ad0c75f0ae06223
canonical_owner: Editor36
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/213-editor-video-media-source-player-track-clock-media-texture-playback-capture-recording-current-source-review.md
related_runtime_owner:
  - docs/plans/optimize/zircon_runtime/177-runtime-video-media-source-player-track-clock-media-texture-playback-capture-recording-current-working-tree-review.md
related_code:
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_runtime/src/asset/assets/imported.rs
  - zircon_runtime/src/asset/assets/sound.rs
  - zircon_editor/src/core/asset/type_registry/asset_type_id.rs
  - zircon_editor/src/ui/retained_host/viewport/poll_captured_frame.rs
  - zircon_editor/src/ui/retained_host/viewport/tests/controller_polls_latest_captured_frame_from_render_framework.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/execution/output_capture.rs
  - zircon_editor/src/ui/settings/settings_window_projection/capture.rs
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/rendering/workbench_render_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/diagnostics/workbench_extension_performance_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/diagnostics/workbench_extension_runtime_diagnostics_workspace.zui
  - zircon_editor/assets/ui/editor/material_components/utils_lab/material_use_media_query.zui
  - zircon_plugins/first_party_editor_catalog/src/catalog.rs
  - zircon_plugins/audio_importer/runtime/src/lib.rs
  - zircon_app/src/entry/runtime_entry_app/frame_capture.rs
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/MediaAssets/Public/MediaSource.h
  - dev/UnrealEngine/Engine/Source/Runtime/MediaAssets/Public/MediaPlayer.h
  - dev/UnrealEngine/Engine/Source/Runtime/MediaAssets/Public/MediaTexture.h
  - dev/UnrealEngine/Engine/Source/Runtime/MovieSceneCapture/Public/IMovieSceneCapture.h
  - dev/godot/scene/resources/video_stream.cpp
  - dev/godot/scene/gui/video_stream_player.cpp
  - dev/godot/servers/movie_writer/movie_writer.cpp
  - dev/godot/servers/movie_writer/movie_writer_pngwav.cpp
  - dev/bevy/crates/bevy_render/src/view/window/screenshot.rs
  - dev/Fyrox/editor/src/plugins/animation
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Utilities/CameraCaptureBridge.cs
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Editor237 · Video/MediaSource/Player 与 Capture authoring 当前工程化差距

## 1. 结论

当前 `zircon_editor` 没有 Video/Media authoring 产品。工作树中没有 `MediaSourceAsset`、`MediaPlayerAsset`、`MediaTexture`、`TrackCatalog`、`VideoSample`、`MediaClock`、`RecorderSession`、`EncoderProvider` 或 `RecordingArtifact` 的 editor owner。`ResourceKind`/`ImportedAsset` 也没有视频类型，`zircon_editor/src/core/asset/type_registry/asset_type_id.rs:87-115` 的完整映射在 `sound` 后直接进入 font/physics/animation/UI，没有 `video`/`media.*`。

现有 UI 中的 “Media” 主要是两类不相关内容：材质实验室的 `Use Media Query` 组件，以及 render/performance/runtime diagnostics workbench 的单帧 capture controls。它们说明 retained host 能显示 capture 状态，但不是媒体 source browser/player/track inspector。`workbench_render_workspace.zui:143-169` 仍以固定 `Frame 1234 capture`、`Windows DX12 30 fps GPU 6.24 ms` 投影；performance workbench 的 capture frame 事件只 route 到 callback；runtime diagnostics 只有 Capture Snapshot/Export Report controls。没有 document id、asset identity、player session、PTS、track、sample、clock 或 output artifact。

当前 editor 唯一可复用的产品底座是 viewport captured-frame polling、runtime diagnostics 以及 App 单 PNG writer。Runtime177 说明 `CapturedFrame`/HDR frame、readback generation、capture scheduler 和 atomic PNG 写入具备局部工程性，但 Editor 没有把它们提升为 recorder job、fixed-step preview、A/V capture、encoder/muxer、checkpoint 或 media browser。若把 “Capture” 按钮连接到 PNG 写入，只能得到单帧证据，不能宣称 Video/Recording 完成。

因此本报告刷新 Editor213 当前性，新增 **26 项 P1（26 Open）**、**12 项 P2（12 Open）**、**24 道资格门（22 Fail / 2 Partial / 0 Pass）**，不新增 P0。既有 Editor36 owner 的 P0/P1 计数保持；本报告只负责当前工作树的 Video/Media 差异与重构顺序。

## 2. 当前源码证据

### 2.1 Asset Browser 与 type registry

- `zircon_runtime_interface/src/resource/marker.rs:8-31` 和 `zircon_runtime/src/asset/assets/imported.rs:21-147` 没有 Video/MediaSource/MediaTexture resource kind/variant，Editor 不可能为视频创建 canonical asset type、import result 或 subasset。
- `zircon_editor/src/core/asset/type_registry/asset_type_id.rs:87-115` 将 ResourceKind 映射为 stable string，但没有 `video`、`media.source`、`media.texture`、`media.playlist`、`media.recording`。任何 UI 新增的“媒体”字段都只能落入 data/generic asset。
- builtin catalog 的 `media.rs` 只登记 Texture/Audio/Opus importer；first-party editor catalog 没有 player/decoder/encoder/muxer provider。没有 menu admission、asset toolkit、preview factory、reimport/migration 或 capability matrix。

### 2.2 现有 capture UI 不是 recorder

`zircon_editor/src/ui/retained_host/viewport/poll_captured_frame.rs` 轮询最新 captured frame；其测试验证 generation 和 latest-frame 选择。这个接口没有 frame queue、PTS、sample sequence、drop count、fixed-step clock 或 write receipt。同步/异步 readback 由 runtime owner 管理，Editor 只看到 image bytes/projection。

`workbench_render_workspace.zui` 与 diagnostics workspace 能触发 `capture`/`capture_snapshot` route，但当前 workbench 控制值和事件并不携带 viewport/camera/pass/format/codec/output path。没有 recorder document、capture profile、session state、progress/cancel、failure reason 或 output manifest。`output_capture.rs` 属于 export wizard 的进程输出捕获，不是音视频 recorder。

### 2.3 没有媒体 authoring surface

缺少以下界面和模型：

1. MediaSource inspector：URI/path/live device、open options、security/permission、desired backend、validation、thumbnail/proxy。
2. Player transport：open/prepare/play/pause/stop/seek/scrub/rate/loop、buffer state、error and async operation receipt。
3. Track inspector：audio/video/subtitle/metadata tracks、language/format/duration、selection/fallback、sample/queue health。
4. MediaTexture/material binding：video-to-texture sink、format/color/HDR/orientation、last-good/clear/drop、GPU resource lifetime。
5. Clock/sync panel：media PTS、audio device clock、drift、seek epoch、presentation deadline、offline fixed-step。
6. Recorder/Movie Render UI：source/camera/output/codec/bitrate/frame range/audio mix、queue/job/shot/frame progress、checkpoint/resume/cancel。
7. Artifact browser：partial/final/recovered output、manifest/hash/metadata、open-in-folder/reimport/cleanup。

### 2.4 静态 fixture 和 callback 风险

render/performance workbench 的固定数字与 capture label 容易给用户造成“录像已支持”的假阳性；runtime diagnostics 的 Snapshot button 也没有 runtime provider receipt。Material `Use Media Query` 是 responsive UI prototype，不应当被作为媒体能力证据。没有 fail-closed 的 capability gate 时，UI 必须隐藏 player/recorder action，而不是显示 queued/success 文案。

## 3. 参考编辑器差异

Unreal Media Editor/MediaAssets 将 MediaSource、MediaPlayer、MediaTexture、track/sample/clock 与 capture protocol 分成可发现的 asset/toolkit；MovieSceneCapture 以 capture protocol、frame grabber、video protocol、output settings 和 completion/error delegate 管理连续输出。Godot VideoStreamPlayer 有真实 node lifecycle、transport、loop、finished/error signal，MovieWriter 把 fixed-rate video 和 PNG/WAV mux 作为独立 backend。Bevy screenshot 将 render-world extraction、GPU readback 和 async image receiver 分离；Fyrox animation editor 通过 commands/undo 保持 authoring mutation；Unity camera capture 则把 camera/render context 与 output identity 绑定。Zircon 当前只有 retained controls 和单帧 poll，没有这些 owner 边界。

## 4. P1 重构任务

| ID | 当前问题 | 必须完成 |
|---|---|---|
| ED-MEDIA-01 | 没有 editor media provider | 增加 editor plugin manifest、first-party catalog、App feature、runtime capability handshake；缺 provider 时 action 隐藏/fail-closed。 |
| ED-MEDIA-02 | 没有 asset type | 增加 Video/MediaSource/MediaTexture/Playlist/RecordingArtifact type registry、factory、icons、open handler、subasset identity。 |
| ED-MEDIA-03 | 没有 importer toolkit | probe/import/reimport/thumbnail/proxy、codec/container/platform diagnostics 和 dependency graph 必须进入 Asset Browser。 |
| ED-MEDIA-04 | 没有 source document | 建立 URI/options document、revision、dirty/save/reopen、LKG、migration、source-control lease。 |
| ED-MEDIA-05 | fixture 取代 query snapshot | render/performance/diagnostics workspace 的 capture rows 必须由 live capability/session snapshot 驱动，去掉固定 frame、fps、GPU 文案。 |
| ED-MEDIA-06 | 没有 player toolkit | 实现 async open/close/prepare/play/pause/stop/seek/scrub/rate/loop command，绑定 operation receipt。 |
| ED-MEDIA-07 | 没有 player preview session | PreviewWorld/preview session 必须隔离 runtime world，具有 generation、shutdown/drain、device and permission errors。 |
| ED-MEDIA-08 | 没有 track inspector | 以 TrackCatalog snapshot 显示 audio/video/subtitle/metadata，支持 selection、format、duration、language 和 fallback。 |
| ED-MEDIA-09 | 没有 sample diagnostics | 显示 PTS/DTS/duration, queue depth, dropped/underrun, decode/render latency, format/color/HDR。 |
| ED-MEDIA-10 | 没有 clock panel | 显示 media/audio/render clock、drift、seek epoch、presentation deadline，并支持 fixed-step preview。 |
| ED-MEDIA-11 | 没有 MediaTexture authoring | 提供 player-to-material/UI binding、YUV/HDR/orientation、clear/hold/drop、GPU residency和device-loss diagnostics。 |
| ED-MEDIA-12 | 没有 subtitle/metadata tools | 提供 locale/track/style preview、time range editing、safe text/script display。 |
| ED-MEDIA-13 | capture action 没有 identity | Capture request 必须选择 viewport/camera/pass/format/output path，显示 source/generation/frame receipt。 |
| ED-MEDIA-14 | 没有 recorder document | 定义 RecorderProfile、output preset、frame range、sample policy、audio mix、revision and validation。 |
| ED-MEDIA-15 | 没有 recorder job | Queue/job/session 具 progress、bounded backpressure、cancel/pause/retry、failure detail and shutdown。 |
| ED-MEDIA-16 | 没有 encoder/muxer UI | codec/container/platform capability、bitrate/GOP/keyframe/HDR/audio options 有 typed schema，不能是 free text。 |
| ED-MEDIA-17 | 没有 A/V capture preview | preview 必须显示 audio meters/waveform、video frame, PTS and drift; silent PNG capture 不得冒充 movie preview。 |
| ED-MEDIA-18 | 没有 output artifact browser | staging/partial/final/recovered artifact、manifest/hash/metadata、open/reimport/cleanup 都有 receipt。 |
| ED-MEDIA-19 | 没有 crash recovery UI | journal/checkpoint/reindex/recover/discard 流程必须可审计，磁盘满和编码失败需保留 diagnostic。 |
| ED-MEDIA-20 | 没有 security/permission UI | URI allowlist、camera/mic permission、DRM/output protection、untrusted media warning 与 redaction。 |
| ED-MEDIA-21 | 没有 plugin command/bridge closure | dist manifest 需声明 commands/events/bridge/provider capability；每个 UI route 对应 handler 和 ABI test。 |
| ED-MEDIA-22 | 无 runtime/editor boundary | Editor 不得直接读 decoder/GPU buffer；只接 generation-qualified snapshots/receipts，统一 reconnect/stale handling。 |
| ED-MEDIA-23 | 没有 automation/test fixture | 需要 deterministic media fixtures、fake backend、golden tracks、seek/rate/loop/clock/capture tests。 |
| ED-MEDIA-24 | 没有 scale/fault UX | 4K/HDR/多轨/长录制/100 player、device loss、network jitter、disk full 的 progress/telemetry/P99 证据。 |
| ED-MEDIA-25 | 没有 localization/accessibility | subtitle/audio language、keyboard transport、screen-reader state、color/HDR warnings。 |
| ED-MEDIA-26 | 没有 cinematic integration | Sequencer/Timeline editor 必须以 media track/section artifact 绑定 player/texture/audio and capture output，禁止复制固定控件。 |

## 5. P2 完整度任务

| ID | 必须补齐 |
|---|---|
| ED-MEDIA-P2-01 | Media asset thumbnail/proxy/waveform/spectrogram cache。 |
| ED-MEDIA-P2-02 | 360/VR/stereo projection and camera preview。 |
| ED-MEDIA-P2-03 | webcam/screen/image-sequence/live source setup。 |
| ED-MEDIA-P2-04 | color-management/HDR inspection and tone-map preview。 |
| ED-MEDIA-P2-05 | subtitle/caption authoring, locale QA and font fallback。 |
| ED-MEDIA-P2-06 | remote/HTTP cache, auth, retry and offline source diagnostics。 |
| ED-MEDIA-P2-07 | deterministic export presets, sidecar metadata and content hash。 |
| ED-MEDIA-P2-08 | multi-camera/AOV/tile capture review。 |
| ED-MEDIA-P2-09 | source-control checkout/lock and collaborative recorder profile edits。 |
| ED-MEDIA-P2-10 | batch transcode, proxy farm and queue scheduling dashboard。 |
| ED-MEDIA-P2-11 | analytics for decode latency, dropped frames, drift and output throughput。 |
| ED-MEDIA-P2-12 | platform capability matrix with CI/headless validation. |

## 6. 资格门

| Gate | 当前结果 | 通过条件 |
|---|---|---|
| ED-MEDIA-G01 | Fail | Video/Media asset type 可创建、导入、保存、重开、重导且 identity 稳定。 |
| ED-MEDIA-G02 | Fail | Asset Browser 能展示 provider/backend/codec capability 和失败诊断。 |
| ED-MEDIA-G03 | Fail | Source document 有 revision/dirty/save/reopen/LKG。 |
| ED-MEDIA-G04 | Fail | Player toolkit 的 async lifecycle/transport/seek/loop 有 operation receipt。 |
| ED-MEDIA-G05 | Fail | Preview session 隔离且可取消、重连、报告设备/权限错误。 |
| ED-MEDIA-G06 | Fail | Track inspector 覆盖音频/视频/字幕/metadata 多轨。 |
| ED-MEDIA-G07 | Fail | PTS/queue/drop/latency/color/HDR sample diagnostics 可观察。 |
| ED-MEDIA-G08 | Fail | Media clock/drift/seek epoch/fixed-step 可视化且可复现。 |
| ED-MEDIA-G09 | Fail | MediaTexture binding 到 material/UI 并验证 GPU lifetime/device loss。 |
| ED-MEDIA-G10 | Fail | Capture action 显示 viewport/camera/pass/frame/generation/output receipt。 |
| ED-MEDIA-G11 | Fail | Recorder profile/job 支持 progress/cancel/backpressure/retry。 |
| ED-MEDIA-G12 | Fail | Encoder/muxer capability 和 typed preset 可校验。 |
| ED-MEDIA-G13 | Fail | A/V preview 显示 audio samples/meters 与 video PTS/drift。 |
| ED-MEDIA-G14 | Fail | output artifact 有 staging/final/partial/recovered 状态和 manifest。 |
| ED-MEDIA-G15 | Fail | crash/disk-full/codec/device failure 可恢复或明确 discard。 |
| ED-MEDIA-G16 | Fail | Security/permission/DRM/path admission 在 UI 与 runtime 一致。 |
| ED-MEDIA-G17 | Fail | dist command/event/bridge/provider closure 经过 ABI test。 |
| ED-MEDIA-G18 | Partial | captured-frame polling 与 runtime generation test 已存在；仍没有媒体 receipt/PTS。 |
| ED-MEDIA-G19 | Partial | PNG output staging/flush/sync/atomic commit 已存在；仍没有 recorder session/mux/recovery。 |
| ED-MEDIA-G20 | Fail | Sequencer/Timeline media tracks 能打开、预览、保存并安装 artifact。 |
| ED-MEDIA-G21 | Fail | deterministic fixture 能验证 seek/rate/loop/subtitle/clock。 |
| ED-MEDIA-G22 | Fail | 4K/HDR、多轨、长录制、100 player 的 UI/telemetry/P99 evidence。 |
| ED-MEDIA-G23 | Fail | localization/accessibility and device permission flows。 |
| ED-MEDIA-G24 | Fail | headless/CI/cross-platform backend matrix。 |

## 7. 推荐重构顺序

1. 先补 ResourceKind、AssetTypeId、catalog/provider、document/factory 和 plugin bridge；删除或隐藏没有 backend 的 Media/Recorder action。
2. 再实现 player preview toolkit、track/sample/clock diagnostics，Editor 只消费 runtime snapshot/receipt，不直接持有 decoder/GPU资源。
3. 将现有 viewport capture UI 改成 typed CaptureProfile/RecorderProfile 和 job queue；保留单帧 PNG 作为 image-sequence backend，而不是最终 movie 功能。
4. 接入 Runtime177 的 MediaTexture、A/V clock、encoder/muxer、checkpoint/recovery；再让 Sequencer Runtime176/Editor236 绑定 media sections。
5. 每一步以 reopen/undo/failure/reconnect/device-loss、deterministic fixtures、headless CI 和 P99/soak evidence 完成。固定展示行、静态 queued 文案和“Capture ready”不能作为通过条件。
