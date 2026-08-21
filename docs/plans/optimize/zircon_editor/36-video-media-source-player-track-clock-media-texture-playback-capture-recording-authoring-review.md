---
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
  - zircon_runtime_interface/src/runtime_api/requests.rs
  - zircon_app/src/entry/runtime_entry_app/frame_capture.rs
  - zircon_editor/src/core/gateway
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback/ui_diagnostics/observability.rs
  - zircon_plugins/first_party_runtime_catalog
  - zircon_plugins/first_party_editor_catalog
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
  - docs/plans/optimize/zircon_tooling/07-performance-benchmark-profile-capture-symbol-crash-evidence-baseline-review.md
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

# 36 · Video / MediaSource / Player / Track / Clock / MediaTexture / Playback / Capture / Recording Authoring 工程化差距

## 1. 结论

Zircon当前没有可称为视频/媒体系统的产品边界。全仓production Rust与manifest的精确符号扫描未找到`MediaPlayer`、`MediaSource`、`MediaTexture`、`MediaTrack`、`VideoStream`、`VideoDecoder`、`VideoFrame`、`MediaClock`或`MovieWriter`；`ResourceKind`、`ImportedAsset`、first-party catalogs和App features也没有媒体身份。Cargo manifests/locks未装配FFmpeg、GStreamer、Theora、dav1d、OpenH264、libvpx、Media Foundation或VideoToolbox等解码后端。这里不是功能偏少，而是source/provider/player/sample/clock/render/audio/editor/recording整条链尚未建立。

当前唯一看似支持媒体资源的接口反而会制造错误认知。`UiResourceKind`会按字段名或`.mp3/.ogg/.wav/.flac/.mp4/.webm/.mov`推断为`Media`，但Runtime resolver立即把`Media`与`GenericAsset`一起降格为`ResourceKind::Data`。因此模板能够接受“media”引用，却没有open、decode、play、track、clock、frame或audio sink语义。这是公共接口承诺与执行能力不一致，不能计为媒体支持。

Sound存在一个可能用于解码音频接入的雏形：`SoundSourceInput::External`和`SoundExternalSourceBlock`。然而提交实现只在`HashMap`中以handle替换整块`Vec<f32>`，没有PTS、duration、sequence、EOS、ring buffer、watermark、backpressure或underrun政策；更关键的是voice同步路径对`External`明确返回`UnsupportedAdvancedFeature("source input has no Kira M1 runtime adapter")`。因此外部音频API目前只接受数据，不能把媒体音频送到实际输出。普通静态clip的pause/resume/seek/speed/position基础可保留，但不能被推导成流式媒体音频已成立。

Runtime时间系统同样只是可复用基础。`FrameClock`提供`Instant` delta，`RuntimeTimeClocks`提供real/virtual/fixed clock、pause、speed与fixed-step drain；Sound timeline再按调用方传入的`delta_seconds`推进automation。它们没有媒体PTS、音频设备clock、presentation deadline、discontinuity epoch、seek flush、drift estimator、timecode/genlock或A/V master selection。游戏simulation time与媒体presentation clock不能继续共用一个“seconds”字段来掩盖语义差异。

图形侧拥有真实且值得保留的帧抓取基础。`GpuReadbackQueue`有3槽staging ring、row padding unpack、ticket/cancel、异步map completion、slot reuse拒绝与in-flight统计；Viewport mailbox按generation配对pending/completed并只提升更新帧；`CapturedFrame`带generation、capture report、graph dump和profile，内部还区分线性RGBA16F的`CapturedHdrFrame`。这些能力说明RenderGraph readback并非空白，但它们服务于单帧诊断/截图，不是视频sample transport。

跨Runtime ABI的`ZrRuntimeFrameV1`只输出width、height、generation和owned RGBA bytes，没有pixel format、row stride、color primaries/transfer/matrix/range、HDR metadata、PTS/duration、camera identity、sequence、drop reason、GPU fence或release callback。App将单帧RGBA8原子发布为PNG，这个durable evidence writer应保留；Editor gateway也正确验证ABI与RGBA shape并维护foreign ownership。但production Editor中只找到capture定义/包装，没有高层workbench执行者；“Capture Frame”命令返回固定`Frame 1234 / CPU 7.1 ms / GPU 9.2 ms`文本。它是静态反馈，不是可工作的capture或recorder。

Editor profiling artifact导出已经先做job admission，再物化UI screenshot，这一内存边界值得复用；但geometry使用`fs::write`、PNG使用`save_buffer_with_format`直接写最终路径，没有原子commit。这条证据链也只导出单张软件绘制UI图，不具备逐camera capture、frame cadence、audio tap、encoder/muxer、queue budget、drop/duplicate policy、stop/finalize/recovery或container validation。

参考源码表明目标必须先定义稳定分层，而不是先绑定一个codec库。Unreal把`IMediaPlayer`、controls、tracks、samples、texture sample、cache/view和clock sink拆成独立接口，再由Wmf/WebM/AVF/Electra等provider适配；Godot较小，但仍有`VideoStream -> VideoStreamPlayback`实例化、texture/audio mix/buffering/seek/loop和可扩展`MovieWriter`；Unity Graphics本地源码只证明逐camera捕获action应在RenderGraph末端集成，不能据此声称Unity视频产品已被覆盖。Bevy本地源码没有通用媒体产品，Fyrox所见`fbx/scene/video.rs`只是FBX记录，不应被误用为成熟播放器参考。

目标边界应为：`MediaSourceAsset + versioned OpenOptions -> admitted MediaProvider -> generation-qualified MediaSession -> typed TrackCatalog + timestamped SampleQueues -> MediaClock/SyncController -> AudioStreamSink + VideoSampleConverter/MediaTexture -> Editor toolkit/runtime consumer`；录制则必须独立为`CaptureSource -> timestamped CaptureSample -> bounded RecorderSession -> Encoder/Muxer provider -> atomic/finalizable RecordingArtifact`。播放与录制可以共享sample、clock、color和provider基础，但不得共享模糊的“frame bytes”DTO。

## 2. 审查边界与证据

### 2.1 当前工作树物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| Resource/UI identity与产品装配 | 14 / 2,439 / 90,903 | E3逐枚举/映射/manifest：ResourceKind、ImportedAsset、UI Media推断与Data降格、Editor type registry、first-party catalogs与App feature；2个test attributes |
| Runtime time与Sound接入 | 22 / 1,559 / 56,349 | E3逐字段/调用：frame/real/virtual/fixed time、Sound source DTO、external block store、voice adapter、clip controls与timeline advance；1个test attribute |
| Render capture、readback与ABI | 18 / 4,365 / 174,949 | E3逐资源生命周期：CapturedFrame/HDR、RenderFramework capture、Viewport mailbox、3-slot GPU readback、ABI owned output和App PNG publication；27个test attributes |
| Editor gateway/capture产品面 | 13 / 3,347 / 145,006 | E3逐command/ownership/job：gateway wrapper、静态workbench反馈、template binding、profiling artifact admission/export；10个test attributes |
| Focused contract tests | 6 / 2,319 / 88,551 | E3静态阅读：readback、render debugger/visual export、gateway demand/ownership与viewport editing；57个test attributes，1个ignored |
| selected combined scope | 73 / 14,029 / 555,758 | 当前工作树fingerprint `34e57da870bb6175341661412da52cb91f9e448e14bcbf97a73cf7bbdd9628f4`；97个test attributes、1个ignored、5个在途文件 |

5个在途文件为`zircon_app/Cargo.toml`、`zircon_editor/src/core/gateway/session/frame.rs`、`zircon_editor/src/core/gateway/session/output.rs`、`zircon_editor/src/tests/gateway/session/output_ownership.rs`和`zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/ui_diagnostics/observability.rs`，均非本轮产生。本报告按读取时当前工作树事实编写；实施前必须重导73文件manifest、重算fingerprint，并复核App feature、gateway foreign output和Capture Frame binding终态。

### 2.2 Resource、Source、Provider与产品装配静态事实

1. `ResourceKind`没有MediaSource、MediaTexture、MediaPlaylist、Subtitle、RecordingPreset或VideoClip身份。
2. `ImportedAsset`没有媒体variant，`asset_kind_for_imported_asset()`也没有媒体映射。
3. 全仓production Rust精确符号扫描没有通用MediaPlayer/VideoPlayer/VideoDecoder/MediaClock/MovieWriter类型。
4. Cargo manifests/locks没有常见软件或平台媒体后端依赖，不能以“未来可接库”描述当前能力。
5. first-party runtime/editor catalogs没有媒体package，App manifest没有媒体feature装配。
6. `UiResourceKind::Media`会由字段名或音视频扩展名推断，但resolver统一降为`ResourceKind::Data`。
7. UI模板收集和surface index接受字符串`media`，却没有对应runtime consumer或播放状态。
8. 没有source URL/file/archive/byte stream/device/camera/live stream的统一open合同。
9. 没有provider probe、priority、capability、supported schemes/extensions、forced provider或fallback decision record。
10. 没有versioned open options、network headers、timeout、cache、credentials、sandbox或path policy。
11. 没有import/probe阶段的container、codec、track、duration、resolution、frame rate和color metadata工件。
12. 没有对malformed container、oversized dimensions、decompression bomb、hostile URL或protocol redirect的安全边界。
13. 没有Media asset dependency/reference规则，Scene/UI无法证明引用的是可播放对象。
14. Editor type registry没有媒体type、thumbnail/waveform/filmstrip provider或专用toolkit。
15. 没有first-party plugin maturity、license/patent、redistribution、platform support或codec availability诊断。

### 2.3 Player、Track、Clock与Audio静态事实

1. `SoundSourceInput::Clip`具有静态clip播放基础，Kira路径支持常见pause/seek/speed/status控制。
2. `SoundSourceInput::External`与`SoundExternalSourceBlock`只表达sample rate、channel count/layout和interleaved samples。
3. External block没有sample format version、frame count、PTS、duration、sequence、discontinuity、EOS或producer generation。
4. `submit_external_source_block_impl()`把handle对应的上一块整体替换，不是bounded FIFO/ring。
5. External block store没有watermark、capacity、backpressure、drop policy、underrun/overrun或lifetime receipt。
6. `sync_source_voice()`对External/Synth明确返回“no Kira M1 runtime adapter”。
7. `validate_source_runtime_surface()`再次将External标记为未来Sound M3能力，证明当前执行面未接通。
8. 因此媒体解码器即使产生PCM，也没有可工作的Runtime audio sink。
9. Sound timeline只按调用者给定`delta_seconds`累加，与audio device clock或媒体PTS无关联。
10. Runtime real/virtual/fixed clocks支持simulation pause/speed/fixed steps，但没有media epoch或presentation timestamp。
11. 没有player state machine：Closed/Preparing/Ready/Playing/Paused/Buffering/Seeking/Ended/Error均不存在。
12. 没有async open completion、event ordering、close cancellation、late callback fencing或session generation。
13. 没有track catalog、selected track、format variant、language/role/default/forced flags或runtime reselection。
14. 没有video/audio/subtitle/caption/metadata track的typed sample queue。
15. 没有duration/time/rate/loop/seekable ranges/live edge/buffered ranges或cache状态。
16. 没有seek flush、decoder drain、preroll、keyframe alignment、exact/fast seek或post-seek first-frame规则。
17. 没有A/V master clock、drift估计、audio resample、video drop/duplicate或late frame policy。
18. 没有timecode、genlock、external clock、pause-on-buffer、clock sink阶段或deterministic offline clock。

### 2.4 Video Sample、GPU转换与MediaTexture静态事实

1. production代码没有VideoFrame/VideoSample/MediaTexture/ExternalTexture类型。
2. 精确搜索未发现NV12、P010、I420或YCbCr runtime frame contract；DDS header的YUV flag不等于视频像素支持。
3. 没有plane count、plane stride/offset、chroma subsampling/siting或odd extent规则。
4. 没有limited/full range、matrix coefficients、primaries、transfer function、mastering display、CLL/FALL或ICC metadata。
5. 没有coded size、display aperture、sample aspect ratio、rotation/mirror或clean aperture。
6. 没有PTS、DTS、duration、decode order、presentation order、keyframe、corrupt/discontinuous flags。
7. 没有CPU buffer ownership、GPU external image/import handle、fence/semaphore或release callback。
8. 现有`queue.write_texture`与普通Texture upload服务于静态/常规资源，不是timestamped media frame queue。
9. 没有YUV->RGB compute/render conversion、tone map、gamut map、deinterlace或scaler provider。
10. 没有MediaTexture front/back frame、sample selection、late latch、render-thread handoff或generation install。
11. 没有mip generation policy、sampler/color-space binding或material/UI consumer的媒体纹理语义。
12. 没有resolution/format change重配、device loss恢复、decoder surface pool或zero-copy fallback。
13. 内部`CapturedHdrFrame`保持线性RGBA16F是正确区分，但它是capture result，不是decoder sample。
14. `CapturedHdrFrame`未进入Runtime ABI、Editor gateway或recording writer，不能被当作HDR video pipeline。
15. 普通Texture/RenderTarget的详细shape、format、streaming缺口由Editor35负责；本轮只新增媒体动态样本合同。

### 2.5 Capture、Recording与Editor静态事实

1. `GpuReadbackQueue`以3槽ring处理buffer与RGBA texture readback，具备异步map、cancel和统计。
2. texture readback固定按4 bytes/pixel解包到连续RGBA CPU `Vec<u8>`。
3. Viewport mailbox按generation保存pending/completed，并淘汰超过ring容量的旧请求。
4. mailbox只提升最新ready generation，避免回退，是可保留的单帧正确性基础。
5. `CapturedFrame`带capture source/report、graph dump和frame profile，可将诊断证据与generation关联。
6. `ZrRuntimeFrameV1`只暴露width/height/generation/owned RGBA bytes，跨ABI元数据不足。
7. ABI没有stride/format/color/timestamp/duration/sequence/drop/camera/fence字段，也没有批量或stream接口。
8. Editor gateway正确校验ABI、RGBA shape并要求foreign owner显式release。
9. production Editor capture精确调用只到trait/handle/session wrapper，没有workbench高层消费者。
10. Performance “Capture Frame”命令返回固定frame 1234与CPU/GPU数字，没有调用gateway或等待artifact receipt。
11. App frame capture严格检查RGBA shape，写staging PNG、flush/sync后原子替换目标，是可靠单图证据writer。
12. App writer没有连续frame admission、cadence、timestamp、encoder、audio、mux、finalize或partial recovery。
13. `surface_present`可消费capture作softbuffer fallback/首帧证据，但不是按录制preset采样多个camera。
14. Editor profiling artifacts在截图物化前进行shared job admission，避免被拒任务先分配大图。
15. profiling screenshot是软件绘制UI快照，geometry JSON与PNG直接写最终路径，不是atomic recording artifact。
16. 没有Record/Stop/Pause状态机、source selector、resolution/fps/codec/container/audio track、output path或overwrite policy。
17. 没有CFR/VFR、frame pacing、drop/duplicate、encoder queue watermark、disk pressure或shutdown finalize。
18. 没有crash/interrupted recording repair、container validation、checksum、sidecar manifest或reproducibility receipt。
19. 没有逐camera capture action registry；Unity Graphics的camera-scoped hook可作为RenderGraph接入参考。
20. 没有Media preview、transport bar、scrub、track/language chooser、buffer/cache/decoder diagnostics或frame inspection。

### 2.6 动态证据边界

本轮是review-only，没有修改production Runtime/Editor/interface/plugin/App代码或tests，也没有运行新的动态测试。媒体产品类型与provider整体缺失、UI Media降格为Data、External audio adapter显式拒绝和静态Capture Frame反馈都可由当前源码直接证明，不需要用一个无法触达产品行为的Cargo lane重复确认。

此前`zircon_editor --lib`测试编译在617.2秒后被239个既有错误和122个warning阻断，当前源码未改变该阻断条件，本轮没有重复同一lane。动态证据因此不能证明Editor测试通过；后续实施必须先恢复可编译基线，再按本报告门禁建立provider fixture、deterministic clock、A/V sync、GPU conversion、recording finalize与跨平台设备矩阵。

### 2.7 参考边界

1. Unreal `IMediaPlayer`将cache、controls、samples、tracks、view、open/close和stats分离，说明backend provider不应直接泄漏给asset/UI。
2. Unreal controls明确表达duration/rate/state/status/time/seek/loop，samples按时间范围获取audio/caption/metadata/subtitle/video。
3. Unreal texture sample表达coded/output dimensions、format、stride、duration、timestamp/timecode与output sRGB，远大于裸RGBA bytes。
4. Unreal tracks表达track数量、format数量、language、selection与video format，clock通过sink组织阶段性tick。
5. Unreal MediaAssets把MediaSource、MediaPlayer和MediaTexture作为独立产品对象，provider plugins再按平台/codec拆分。
6. Godot `VideoStreamPlayback`至少包含play/pause/seek/position/audio-track/texture/update与mix callback。
7. Godot `VideoStreamPlayer`包含audio resampler/mix buffer、loop、autoplay、speed、buffering和audio bus集成。
8. Godot `MovieWriter`按extension选择writer并定义begin/frame/end以及同步audio mix输入，可作为最小录制生命周期参考。
9. Unity Graphics `CameraCaptureBridge`只提供camera-scoped action registry，URP `CapturePass`在RenderGraph末端读取active color texture；本地仓不包含Unity VideoPlayer/Recorder产品，引用范围到此为止。
10. Bevy本地源码没有通用媒体产品，不能用其缺失降低Zircon目标；Fyrox本轮只见FBX video record，也不能代表成熟播放链。

## 3. 必须保留的真实基础

1. 保留Runtime real/virtual/fixed clocks及其pause/speed/fixed-step语义，但为媒体建立独立clock domain与correlation。
2. 保留静态Sound clip的Kira playback controls/status，将其作为音频输出能力而非流媒体sink冒充。
3. 保留`AudioChannelLayout`和External source概念，但重建为有界、带时间戳、可消费的stream contract。
4. 保留`GpuReadbackQueue`的3-slot staging、alignment、ticket/cancel、callback isolation和in-flight telemetry。
5. 保留Viewport capture generation/mailbox规则，扩展为明确的capture session queue而非复用单帧slot。
6. 保留`CapturedFrame`与`CapturedHdrFrame`的SDR/HDR产品区分和capture report/profile关联。
7. 保留Runtime ABI owned output与Editor foreign ownership/shape validation，新增versioned media/capture stream DTO。
8. 保留App PNG staging、flush/sync与atomic replace实现，复用其durable publication原则。
9. 保留Editor job admission-before-materialization边界，用于filmstrip、waveform、transcode和recording finalize。
10. 保留RenderGraph末端capture hook位置与offscreen/texture target capture report，扩展camera-scoped source选择。
11. 保留插件catalog/admission总体架构，将codec/container/platform providers纳入同一truthful maturity模型。
12. 保留Texture Editor35定义的typed format/color/artifact方向，让MediaTexture消费同一color与GPU资源基础。

## 4. 目标架构与Owner边界

```text
MediaSourceAsset + MediaOpenOptions + security policy
    -> MediaProviderRegistry::probe/admit/open
    -> MediaSession(generation, state, events, diagnostics)
       -> TrackCatalog + selected formats
       -> timestamped Audio/Video/Subtitle/Metadata SampleQueues
       -> MediaClock + SyncController + seek/discontinuity epochs
       -> AudioStreamSink
       -> VideoSampleConverter -> MediaTexture
       -> Runtime material/UI/cinematic consumers
       -> Editor Media Toolkit

Camera/Viewport/RenderTarget/AudioBus CaptureSource
    -> timestamped CaptureSamples + bounded admission
    -> RecorderSession(clock, pacing, drop policy)
    -> EncoderProvider + MuxerProvider
    -> staging container + sidecar receipt
    -> finalize/validate/atomic RecordingArtifact
```

| Owner | 必须拥有 | 不得拥有 |
|---|---|---|
| `zircon_runtime_interface` | 稳定Media/Recording identities、versioned DTO、timestamp/color/ownership/status codes | codec对象指针、wgpu handle、Editor widget字符串 |
| `zircon_runtime::asset` | MediaSource/playlist/subtitle/preset metadata、dependency、versioned recipe/artifact identity | live decoder/session、平台设备句柄 |
| `zircon_runtime::core::media` | provider registry、session state/event、track/sample/clock/sync抽象 | 具体FFmpeg/MF/AVF实现、Editor状态 |
| `zircon_plugins/*media*` | container/codec/platform provider、probe/open/decode/encode/mux capability | 改写公共resource identity、绕过admission |
| `zircon_plugins/sound` | bounded timestamped external audio sink与device-clock反馈 | 决定媒体seek/track/source生命周期 |
| `zircon_runtime::graphics` | VideoSample GPU import/conversion、MediaTexture install、capture sources | container解析、Editor export path |
| `zircon_app` | platform media device/window integration、durable artifact publication、shutdown finalize | codec-specific authoring policy |
| `zircon_editor` | Media toolkit、transport/track/preview/diagnostics、Recorder UI、transaction/job/receipt | 自建decoder、伪造frame/status结果 |
| Tooling/CI | fixtures、license/package、determinism/security/performance/device/release gates | 通过静态字符串推断maturity |

所有session、sample、seek、track selection、MediaTexture install和recording artifact都必须携带稳定source/session identity与generation。任何late callback、旧seek epoch、旧decoder surface或被取消job都不得覆盖新session状态。所有队列必须有容量、byte/time budget、admission、drop/backpressure政策、telemetry和shutdown语义。

## 5. P0：必须先关闭的正确性与产品真实性缺口

### P0-1：公共UI接受Media引用，Runtime却无媒体资源并降格为Data

必须在同一硬切中决定真实产品合同：要么在Media runtime可用前拒绝`UiResourceKind::Media`和音视频扩展名，要么引入稳定MediaSource identity、loader/provider和consumer。禁止继续让`.mp4`模板通过解析后以普通Data进入运行时。迁移必须给旧模板提供明确diagnostic和conversion command，不能静默改变含义。

### P0-2：External audio公开接受PCM，实际voice adapter明确拒绝

`submit_external_source_block()`的成功不能继续被解释为可播放。应先将maturity降为unsupported，或一次性实现bounded timestamped ring、Kira streaming adapter、device-clock反馈、underrun/overrun/EOS和shutdown drain，并为submit返回可观察receipt。任何仅删除错误字符串或让voice保持silence的改动都不合格。

### P0-3：没有timestamped video sample与MediaTexture合同

在接入任何decoder前必须定义plane layout、format、color/HDR、coded/display geometry、PTS/duration、ownership/fence/release和generation。MediaTexture必须按clock选择sample，并在render线程generation-qualified安装。禁止先把每帧转RGBA8后调用普通`queue.write_texture`，那会固化CPU copy、色彩错误与无同步架构。

### P0-4：没有MediaClock、sample queue、seek epoch和A/V同步

不能用Runtime virtual time或每帧`delta_seconds`直接驱动媒体。必须定义audio/video/external/offline master clock、PTS correlation、buffering/preroll、drift/drop/duplicate/resample政策及seek/discontinuity flush。所有异步open/decode/seek callback必须按session generation和seek epoch拒绝过期结果。

### P0-5：Capture产品是单帧RGBA/PNG，Editor命令仍返回固定假结果

“Capture Frame”命令必须在产品未接通时诚实失败，不得继续显示固定frame 1234。连续录制必须建立独立RecorderSession、timestamped samples、有界队列、camera/audio source、encoder/muxer、finalize/recovery和atomic artifact receipt；不得在每帧循环调用单帧ABI再拼图片序列冒充工程级录像。

## 6. P1：Source、Provider、Open与安全边界

### P1-1：建立稳定`MediaSourceAsset`

Source必须区分file、URL、archive、memory、camera/device、live endpoint和generated stream，并保存稳定identity、display name、origin与可序列化配置。资源本身不得持有decoder/session或平台句柄。

### P1-2：建立versioned `MediaOpenOptions`

Options应覆盖desired provider、play-on-open、loop、precache/buffer、track preferences、latency mode、network timeout/headers和security policy。字段必须有schema/version/default/migration，不能散落为Editor字符串。

### P1-3：Provider registry需要probe与admission

Provider必须声明schemes、extensions、containers、codecs、track types、seek/live/HDR/hardware capabilities与平台条件。Probe返回confidence和结构化reason，registry产出可追溯selection record，禁止first-match无诊断。

### P1-4：Open必须异步且可取消

网络、container probe和decoder初始化不得阻塞render/UI线程。Open ticket需要cancel、deadline、generation和completion event；Close必须阻止late callback重新激活session。

### P1-5：Container probe工件缺失

导入/打开前应形成container、duration、tracks、codec/profile/level、bitrate、dimensions、frame rate、color、seekability和warning的versioned probe result。Editor、cook与Runtime必须消费同一结果或记录重新probe原因。

### P1-6：网络与协议安全政策缺失

必须限制scheme、redirect、DNS/private network、credential forwarding、header allowlist、download size、timeout和cache location。HTTP provider不得绕过项目sandbox与secret redaction。

### P1-7：恶意媒体输入边界缺失

需要dimension/sample-rate/channel/track count/duration/table size/metadata size/packet allocation上限，以及parser/decoder隔离策略。Malformed fixture、fuzz corpus和timeout必须成为required lane。

### P1-8：Source dependency与cook政策缺失

Local source、sidecar subtitle、playlist child、poster、proxy和external URL应有明确dependency与package规则。Cook必须区分embed、copy、transcode、streaming URL和forbidden external dependency。

### P1-9：Codec许可与分发事实缺失

每个provider应声明license、patent/royalty注意事项、binary origin、platform redistribution和feature gate。Editor要显示当前构建实际可用的decoder/encoder，不能只看扩展名。

### P1-10：Provider fallback语义缺失

Fallback应区分unsupported container、unsupported codec、device init failure、corrupt source与transient network failure。不得在已消费stream后无条件换provider，也不得让fallback改变track/color/seek语义而无记录。

### P1-11：Media metadata与poster/thumbnail工件缺失

Poster、filmstrip、waveform、duration和track metadata应由后台job从probe/decode工件生成，带source hash、provider/version和atomic publication。禁止Editor每次打开时同步解码首帧。

### P1-12：Playlist与sidecar subtitle身份缺失

Playlist、subtitle/caption和timed metadata不能作为无类型Data数组处理。需要稳定引用、language/role、ordering、missing item policy、reimport与dependency propagation。

## 7. P1：Player、Track、Clock、Queue与同步

### P1-13：建立严格Player状态机

至少定义Closed、Opening、Ready、Playing、Paused、Buffering、Seeking、Ended、Closing与Error，并列出合法转换、事件顺序和重复命令语义。UI状态必须从session snapshot派生。

### P1-14：事件必须generation-qualified

Opened、OpenFailed、TracksChanged、Buffering、SeekCompleted、EndReached、Error和Closed都要携带session generation、source identity与stable code。旧session事件不得进入新player。

### P1-15：Controls能力查询缺失

Play/Pause/Seek/Rate/Loop必须先由provider/session报告capability，返回accepted/completed/error不能混为一个bool。Live、non-seekable或反向播放应诚实拒绝。

### P1-16：Track catalog缺失

Track需表达Audio/Video/Subtitle/Caption/Metadata、language、label、role、default/forced、codec与format variants。Index只在当前catalog generation内有效。

### P1-17：Track与format选择缺失

选择操作要可取消、generation-qualified，并明确是否触发decoder重建、buffer flush或时钟连续性变化。自动选择policy必须可解释、可覆盖、可持久化。

### P1-18：Timestamp与time range类型缺失

不能继续用裸`f32 seconds`承载PTS。需要有理数time base或checked duration、epoch/sequence、validity与range类型，避免长视频精度损失和跨轨比较错误。

### P1-19：Sample queue必须有界

Audio/video/subtitle/metadata分别需要time/byte/count容量、watermark、producer admission、consumer selection与flush。Queue状态必须纳入buffering判断与telemetry。

### P1-20：MediaClock domain缺失

Clock应支持audio-device、external timecode、monotonic wall、engine-correlated和deterministic offline模式，表达rate、pause、epoch与correlation sample。Simulation fixed clock不是默认media master。

### P1-21：A/V sync controller缺失

需要估计video PTS相对master的误差，执行present/drop/hold/duplicate；audio侧执行受限resample或buffer correction。阈值、连续性和恢复策略必须可配置且可测。

### P1-22：Seek生命周期缺失

Seek必须产生新epoch，flush旧packets/samples、cancel旧decode、定位keyframe、preroll并在目标附近首帧/音频ready后完成。Fast/exact seek需独立能力和误差报告。

### P1-23：Buffering与live edge缺失

Session应公开buffered/seekable ranges、target latency、live edge distance、rebuffer count和network throughput。Pause-on-buffer与resume hysteresis必须单一owner。

### P1-24：External audio sink必须真正流式

把External source改为timestamped packet/ring interface，支持format negotiation、channel layout、resample、watermark、EOS、flush epoch和device-clock observation。Kira adapter必须消费队列，而不是每次读取最后一个block。

## 8. P1：Video Sample、Color、GPU与MediaTexture

### P1-25：定义typed `VideoSampleFormat`

Closed set至少覆盖常见planar/semi-planar YUV、packed RGB(A)、bit depth与float HDR，并明确plane layout和alignment。Unknown/vendor format必须通过provider capability处理，不能变成字符串。

### P1-26：定义coded/display geometry

Sample必须分别表达coded extent、visible aperture、output dimensions、pixel aspect、rotation和mirror。所有crop/scale/rotate进入可追踪conversion plan。

### P1-27：颜色与HDR metadata必须端到端

Range、matrix、primaries、transfer、chroma siting、mastering display、MaxCLL/MaxFALL要从container/sample传到shader/output policy。缺失metadata应有deterministic default与warning。

### P1-28：Sample timing与decode flags缺失

PTS、DTS、duration、decode/presentation sequence、keyframe、corrupt、discontinuity和end flags必须完整。B-frame重排不能依赖到达顺序。

### P1-29：CPU sample ownership缺失

CPU plane buffer需定义immutable lifetime、stride/offset、allocator、pool return和跨线程规则。Decoder不得把临时slice交给render线程。

### P1-30：GPU external sample ownership缺失

Hardware decoder surface需要typed external image、device compatibility、acquire fence、release fence/callback和device-loss语义。Zero-copy失败时要有可观察fallback，不得隐式永久回落CPU。

### P1-31：Video conversion pipeline缺失

YUV->RGB、range expand、chroma reconstruct、scale/rotate、deinterlace、gamut/tone map要形成compiled conversion plan与shader variant key。转换结果和耗时必须可诊断。

### P1-32：MediaTexture需要独立资源身份

MediaTexture不是普通静态Texture。它应绑定player/video track、输出format/color policy、clear/hold-last-frame、sampler与mip策略，并维护session generation。

### P1-33：MediaTexture sample selection缺失

Render准备阶段应按presentation time选择最佳sample，丢弃过期epoch/旧generation，并报告held/dropped/late/missing。UI与Material consumer必须读同一front buffer。

### P1-34：Resolution/format change重配缺失

自适应流或camera可改变extent/format/color。Surface pool、bind group和conversion pipeline重建必须异步、generation-qualified，并在切换帧保持旧资源可用。

### P1-35：MediaTexture mip与filter政策缺失

大屏/3D表面可能需要mip，UI视频通常不需要。应按consumer与预算选择GPU mipgen/none，避免每帧CPU生成或无条件全链写入。

### P1-36：GPU资源预算与性能基线缺失

需要按session跟踪decoder surfaces、queued samples、conversion targets、staging bytes和copy bandwidth；建立1080p/4K/8K、SDR/HDR、多流并发与device-loss基线。

## 9. P1：Capture、Recorder、Encoder、Mux与Editor

### P1-37：建立typed CaptureSource

Source应覆盖camera、viewport、RenderTarget、final output、HDR scene color、UI overlay和audio bus，并明确pre/post tone-map、resolution与include-UI政策。不得用一个bool选择“当前帧”。

### P1-38：逐camera capture registry缺失

参考Unity Graphics，在camera RenderGraph末端维护稳定action/session registration，并覆盖offscreen/intermediate/backbuffer条件。Registration必须generation-qualified且可取消。

### P1-39：Capture sample需要timestamp与metadata

每帧必须携带source/session identity、sequence、PTS、duration、format/color、extent/stride和drop status。Audio sample使用同一recording clock correlation。

### P1-40：Recorder状态机缺失

定义Idle、Starting、Recording、Paused、Stopping、Finalizing、Completed、Failed与Aborted，明确重复stop、shutdown、disk full和encoder failure语义。UI只展示真实receipt。

### P1-41：CFR/VFR pacing政策缺失

Recorder需选择constant或variable frame rate，决定late frame drop、duplicate、timestamp quantization和pause gap。离线deterministic capture不得依赖wall-clock抖动。

### P1-42：Encoder provider合同缺失

Provider需声明video/audio codecs、profiles、levels、pixel formats、rate control、bitrate/quality、GOP/B-frame、hardware/software和platform availability。Admission应返回实际协商值。

### P1-43：Muxer provider合同缺失

Container需声明支持的tracks/codecs/time base/metadata和streamability，负责header、packet interleave、trailer/finalize与validation。Encoder和muxer失败必须有稳定错误归属。

### P1-44：Recorder队列与backpressure缺失

GPU readback、conversion、encode、mux和disk write每阶段都要有byte/time/count budget、watermark和policy。Render线程不得因encoder或磁盘无界阻塞。

### P1-45：Recording artifact publication缺失

输出应写唯一staging path，周期flush/checkpoint，stop后finalize、reopen/probe/validate，再原子发布final path与sidecar receipt。Interrupted staging要可发现、恢复或安全清理。

### P1-46：Editor Media Toolkit缺失

Toolkit需提供真实transport、timeline scrub、frame step、track/language/format、loop/rate、buffer/cache、waveform/subtitle和decoder diagnostics。所有操作通过session command与事件回执。

### P1-47：Editor Recorder面板缺失

面板需选择capture sources、resolution/fps、SDR/HDR、audio buses、codec/container/preset/output，并显示queue/drop/encode/disk/finalize状态。不可用能力必须在开始前阻止。

### P1-48：Preview与thumbnail产品链缺失

Filmstrip、poster、waveform和subtitle preview必须后台生成、可取消、带provider/version key并atomic cache；toolkit打开时不得同步扫描完整视频。Preview color与MediaTexture必须一致。

## 10. P1：Plugin、Diagnostics、测试与发布资格

### P1-49：确定Media package owner

建立唯一core media contracts owner，再按container/codec/platform拆provider packages。禁止在App、Editor和Sound各自引入私有decoder类型。

### P1-50：First-party装配必须可追溯

Runtime/editor catalog、App feature、package distribution和license manifest必须共同证明provider实际安装。缺provider时Media asset应给出结构化unsupported状态。

### P1-51：Maturity必须由能力门派生

Decoder/encoder/container/HDR/hardware/live/seek/recording maturity需由required tests与platform matrix生成，不能由manifest字符串自报stable。

### P1-52：诊断stable codes缺失

Open/probe/decode/network/buffer/sync/GPU conversion/audio sink/encode/mux/disk/finalize应有分层错误码、source/session/generation、provider、track和remediation。

### P1-53：Media telemetry缺失

至少记录demux/decode latency、queue depth/time、buffered ranges、A/V drift、dropped/duplicated frames、audio underrun、conversion/GPU copy、network throughput和provider fallback。

### P1-54：日志与隐私政策缺失

URL query、headers、credentials、本地路径和媒体metadata需redaction。Crash/evidence bundle只收集必要provider/codec/track事实，不得泄漏内容或密钥。

### P1-55：Deterministic provider fixture缺失

需要纯内存reference provider，可生成指定PTS、track change、seek、buffer stall、resolution/color change与错误，用于状态机和同步测试，避免required lane依赖系统codec。

### P1-56：Malformed/fuzz矩阵缺失

Container probe、packet/sample metadata、subtitle parser和network playlist需要fuzz；oversized、truncated、invalid time base、track explosion、decoder hang必须有预算与超时证据。

### P1-57：A/V sync与seek golden缺失

用deterministic clocks验证不同frame/audio rates、drift、stall、pause/rate、loop、fast/exact seek和discontinuity，给出最大误差与drop/resample边界。

### P1-58：GPU/color visual golden缺失

覆盖NV12/P010/RGB、range/matrix/primaries/transfer、SDR/HDR、odd dimensions、rotation、chroma siting、tone map与format change，并比较CPU reference和GPU输出。

### P1-59：Recording fault-injection缺失

覆盖queue saturation、GPU readback failure、encoder crash、mux failure、disk full、permission loss、cancel、window/device loss和process interruption，验证不发布损坏final artifact。

### P1-60：Cross-platform/package/release矩阵缺失

Windows/Linux/macOS和目标mobile/web需分别证明可用provider、software/hardware fallback、codec redistribution、headless/offline capture、installed build路径与rollback。开发机能播不能替代发布资格。

## 11. P2：高级能力与长期竞争力

### P2-1：Adaptive bitrate streaming

在基础track/clock/buffer成熟后，引入HLS/DASH manifest、representation选择、throughput/latency estimator、seamless switch和segment cache。切换必须维持timeline与color/track连续性。

### P2-2：DRM与受保护媒体

CDM/license、secure decoder、protected GPU surface、output protection和evidence redaction需独立安全架构。不得让protected content落入普通CPU readback或截图链。

### P2-3：Hardware decode/encode调度

按adapter/device/codec/profile/resolution/session并发限制做admission，记录zero-copy兼容与fallback成本。多session资源竞争要纳入GPU/OS预算。

### P2-4：Ultra-low-latency live media

支持可配置jitter buffer、live edge追赶、低延迟协议、late packet和clock recovery。低延迟不能通过无界丢帧或禁用同步获得。

### P2-5：Camera与capture device ingestion

建立device catalog、permission、format negotiation、hotplug、orientation、exposure metadata和clock correlation，使camera成为MediaSource而非平台私有回调。

### P2-6：360/180、stereo与projection metadata

支持equirect/cubemap、stereo layout、view orientation与spatial metadata，并与Material/Camera/VR输出对接。Projection转换需走GPU plan和visual golden。

### P2-7：Spatial/multichannel media audio

保留ambisonics/object/multichannel layout、language/role和loudness metadata，通过Sound mixer路由，而不是在decoder处强制downmix stereo。

### P2-8：Timed metadata、subtitle与accessibility

字幕、caption、chapter、cue、karaoke与timed metadata需要样式、安全解析、locale/fallback、screen reader和Editor timeline authoring。

### P2-9：Remote broadcast与pixel streaming

将render/audio capture接到低延迟encoder、transport、congestion control和input authority；与离线Recorder共享sample contract，不共享错误的文件finalize语义。

### P2-10：Nonlinear transcode与proxy workflow

支持proxy、trim、concat、audio normalize、color transform、subtitle burn-in和batch transcode build graph，产出可追溯recipe/artifact而非破坏源文件。

### P2-11：Distributed/offline deterministic encoding

离线capture按deterministic engine clock生成frame/audio samples，交由可重试分片或remote workers编码，并校验segment边界、artifact hash与最终mux一致性。

### P2-12：跨引擎质量与性能基准

建立与Unreal provider/media texture、Godot VideoStream/MovieWriter和平台原生backend的启动、seek、A/V drift、CPU/GPU、memory、power、HDR与recording质量对比，按硬件档位保存证据。

## 12. 当前Authority与断路清单

| 当前入口/字段 | 当前authority | 实际断路 | 目标owner |
|---|---|---|---|
| `UiResourceKind::Media` | UI template path/extension推断 | resolver降为`ResourceKind::Data`，无播放consumer | stable MediaSource identity + UI media consumer |
| `ResourceKind` / `ImportedAsset` | 通用资源系统 | 没有任何Media variant | Runtime interface/asset Media model |
| first-party catalogs/App feature | plugin装配 | 没有媒体provider/package | Media provider catalog + admission receipt |
| `SoundSourceInput::External` | Sound public DTO | voice runtime显式unsupported | Sound timestamped stream sink |
| `SoundExternalSourceBlock` | handle到单块PCM map | 每次替换，无FIFO/PTS/backpressure/EOS | bounded AudioSampleQueue |
| Runtime real/virtual/fixed time | gameplay simulation clocks | 无media epoch/device clock/PTS correlation | MediaClock/SyncController |
| Sound timeline `delta_seconds` | caller-driven automation | 无shared media/audio master | correlated timeline consumer |
| ordinary Texture upload | static/cooked texture resource | 无timestamp/plane/color/fence/release | VideoSampleConverter/MediaTexture |
| `CapturedFrame` | 单帧RGBA8诊断product | 无stream pacing/audio/encoder | CaptureSample + RecorderSession |
| `CapturedHdrFrame` | 内部线性RGBA16F capture | 未跨ABI/Editor/recorder | typed HDR CaptureSample path |
| `GpuReadbackQueue` | 3-slot async diagnostic readback | 固定RGBA、无recording queue policy | shared readback primitive under recorder admission |
| Viewport capture mailbox | newest generation single-frame cache | 老请求淘汰但无drop receipt/cadence | Recorder capture queue |
| `ZrRuntimeFrameV1` | width/height/generation/RGBA owned bytes | 无format/color/time/stride/source | versioned Frame/Media stream ABI |
| App PNG writer | durable one-shot evidence | 无continuous session/finalize/container | shared atomic artifact publisher |
| Editor gateway capture wrapper | ABI ownership/shape validation | 无高层production caller | Viewport/Recorder controller |
| Workbench Capture Frame feedback | static template response | 固定frame 1234，不执行capture | real command -> job/session receipt |
| Profiling artifact export | bounded Editor export job | 单图软件UI且最终路径直写 | capture evidence/recording publication service |

硬切规则：旧的UI `media -> Data`映射不得与新MediaSource并存；External audio旧的单block语义不得作为新stream sink兼容层长期保留；Recorder不得复用单帧`ZrRuntimeFrameV1`作为稳定协议。迁移期只允许显式version adapter、warning和one-shot conversion，禁止两套authority长期双写。

## 13. 分层重构里程碑

### M0：Truthfulness与可编译基线

移除/禁用静态假Capture反馈，明确UI Media当前unsupported，恢复Editor/Runtime required lane可编译；冻结Media术语、owner、stable error和73文件fingerprint。此阶段不接真实codec。

### M1：Stable Media Identity、Source、Provider与Probe

引入MediaSource/playlist/subtitle/recording preset identity、versioned open options、provider registry/probe/admission和deterministic memory provider；完成Data降格迁移与catalog/App装配。

### M2：Session、Track、Sample、Clock与Seek

建立严格player state/event、track catalog/selection、timestamp/time range、bounded queues、MediaClock/SyncController、seek/discontinuity epoch与deterministic tests。

### M3：Streaming Audio Sink

重构External audio为timestamped bounded ring，接通Kira stream adapter、format negotiation/resample/device clock、watermark/underrun/EOS/flush，并删旧单block authority。

### M4：Video Sample、GPU Conversion与MediaTexture

定义CPU/GPU sample ownership、YUV/RGB/color/HDR metadata、conversion plan、surface pool、generation install和MediaTexture sample selection；完成format change/device loss路径。

### M5：Editor Media Toolkit与Asset Workflow

实现probe metadata、poster/filmstrip/waveform cache、toolkit transport/track/scrub/diagnostics、reimport/cook/package和background job/transaction integration。

### M6：Capture Source与Recorder Core

建立camera/viewport/RenderTarget/audio bus CaptureSource、recording clock、CFR/VFR pacing、bounded multi-stage queues、Recorder state/event和headless deterministic provider。

### M7：Encoder、Muxer与Durable Artifact

实现software reference encoder/muxer provider、协商、staging/checkpoint/finalize/reopen validation/atomic publish、interrupted artifact recovery与Editor Recorder UI。

### M8：Platform Providers与发布资格

按Windows/Linux/macOS及目标mobile/web接平台decoder/encoder，验证hardware/software fallback、license/package、installed build、device loss、fault injection和rollback。

### M9：Advanced Streaming、Devices与Distributed Workflows

在基础门全部通过后再进入ABR/DRM/low-latency/camera/360/spatial audio/pixel streaming/proxy/transcode/distributed offline encoding与跨引擎benchmark。

## 14. 验收门禁

### G01：Public truthfulness

没有Media runtime时，UI/asset/editor必须在导入或绑定阶段返回结构化unsupported；启用后`.mp4/.webm/.mov`解析为真实MediaSource，禁止降为Data。

### G02：Provider admission

同一source/options/platform产生确定provider选择、capability与reason receipt；forced provider、unsupported和fallback路径均有测试。

### G03：Open/close generation

连续open A、close、open B后，A的所有late callbacks/events/samples均被拒绝，B状态不回退；cancel/deadline行为可重复。

### G04：Probe security

Malformed/oversized/redirect/private-network/credential fixtures在预算内被拒，日志和evidence不泄漏headers、query或secret path。

### G05：Track catalog/selection

多audio/video/subtitle/metadata track fixture准确报告language/role/format，切换后只消费新catalog generation并给出完成事件。

### G06：Timestamp precision

长时媒体、非整数frame rate和不同time base转换无不可接受累积误差；invalid/overflow time被结构化拒绝。

### G07：Bounded sample queues

所有队列在count/bytes/time上有硬上限；producer过快、consumer暂停和shutdown时执行已声明backpressure/drop/flush且telemetry一致。

### G08：Media clock modes

audio-device、engine-correlated、external与deterministic offline clock的pause/rate/epoch/correlation行为通过同一contract suite。

### G09：A/V sync

多frame/audio rate、人工drift与stall下，误差保持在平台目标内；drop/duplicate/resample次数与policy匹配，无无界追赶。

### G10：Seek/discontinuity

Fast/exact seek、loop、track switch和network discontinuity会flush旧epoch、正确preroll并在目标误差内完成，旧帧不闪回。

### G11：External audio playback

External PCM经过真实Kira output adapter消费，支持format negotiation、watermark、EOS和flush；submit成功不再对应silence/unsupported。

### G12：Audio underrun/overrun

受控producer stall与burst下，underrun/overrun计数、恢复、静音/保持政策和device-clock反馈可观察且无死锁。

### G13：Video sample layout

NV12/P010/I420/RGB fixtures覆盖odd extent、plane stride/offset、lifetime和invalid layout；越界/不一致在GPU提交前拒绝。

### G14：Color/HDR fidelity

Range/matrix/primaries/transfer/chroma siting及HDR metadata通过CPU reference与GPU visual golden，SDR/HDR输出无静默错误默认。

### G15：GPU ownership/fences

External surface acquire/release、decoder reuse、render completion、cancel和device loss无use-after-free；zero-copy/fallback状态可观测。

### G16：MediaTexture presentation

按clock选帧，旧generation/epoch被拒；hold/clear/drop/late策略与telemetry一致，Material/UI读取同一可见帧。

### G17：Dynamic reconfiguration

播放中resolution/format/color变化不崩溃、不引用旧surface，旧帧持续到新pipeline ready，并记录重配latency。

### G18：Decode/GPU performance

1080p/4K/8K、SDR/HDR与多流矩阵给出CPU/GPU/memory/copy/power基线，队列与surface预算不会无界增长。

### G19：Capture source correctness

camera/viewport/RenderTarget/final output/HDR/UI overlay选择得到正确RenderGraph阶段、orientation、extent和color metadata。

### G20：Recorder pacing

CFR/VFR、pause/resume、late frame、offline deterministic模式的PTS、drop/duplicate和audio correlation符合preset并可复现。

### G21：Recorder admission/backpressure

readback/conversion/encode/mux/disk各阶段硬预算生效；encoder或disk变慢不阻塞render线程，也不丢失drop reason。

### G22：Encoder/mux negotiation

codec/profile/level/pixel format/rate control/audio/container组合返回实际协商值；不兼容组合在Start前拒绝。

### G23：Durable finalize

正常stop后container可reopen/probe、track/duration/timestamp有效，final与sidecar原子发布；staging不会冒充完成artifact。

### G24：Interrupted recording recovery

disk full、cancel、encoder/mux crash、process interruption和device loss不覆盖旧final；staging可识别、修复或清理并保留原因。

### G25：Editor command truth

Capture/Record命令必须调用真实controller并显示job/session/artifact receipt；源码和截图中不再出现固定frame 1234假结果。

### G26：Media Toolkit workflow

Open/play/pause/seek/frame-step/loop/rate/track/subtitle/diagnostics由真实session驱动，关闭文档会cancel并释放provider/surfaces/jobs。

### G27：Preview artifacts

Poster/filmstrip/waveform/subtitle cache有source/provider/version key、后台admission/cancel、atomic publication、corruption rebuild与color一致性。

### G28：Diagnostics/privacy

所有失败具有stable code/source/session/generation/provider/track/stage；URL、header、credential和内容metadata按政策redact。

### G29：Malformed/fuzz/fault matrix

Parser、sample metadata、subtitle、playlist、queue、GPU import、encoder/mux和publication均有malformed/fuzz/fault required lanes，无hang/panic/unbounded allocation。

### G30：Cross-platform/device matrix

目标OS/GPU/codec/device组合证明software/hardware provider、hotplug、device loss、fallback和capability truth；缺失能力诚实降级。

### G31：Headless/package/license

Cooked/installed/headless builds能按声明播放或离线录制，provider binaries与licenses随包分发，开发路径或系统偶然codec不会掩盖缺件。

### G32：Release/rollback

Media schema、provider、artifact、preset和codec包升级有migration/canary/rollback；旧项目与录制artifact不会被不兼容版本静默破坏。

## 15. 禁止的临时修补

1. 禁止只在`ResourceKind`加`Media`枚举而没有Source/Provider/Session/consumer全链。
2. 禁止继续把音视频扩展名解析为`Data`并声称UI支持媒体。
3. 禁止删除External audio的unsupported错误但仍不消费PCM。
4. 禁止用单个`Vec<f32>`/`Vec<u8>`覆盖式mailbox冒充stream queue。
5. 禁止用Runtime virtual time或裸`f32 seconds`直接承担媒体PTS/A-V同步。
6. 禁止每帧解码为RGBA8 CPU buffer再调用普通Texture upload作为最终MediaTexture架构。
7. 禁止忽略YUV range/matrix/primaries/transfer/HDR metadata。
8. 禁止把decoder-owned surface交给GPU而没有fence/release/generation合同。
9. 禁止把`CapturedHdrFrame`存在解释成HDR录像已成立。
10. 禁止通过循环调用`ZrRuntimeFrameV1`和PNG writer冒充工程级Recorder。
11. 禁止在render线程同步等待GPU readback、encoder、mux或磁盘。
12. 禁止无界channel、queue、segment cache、filmstrip或waveform生成。
13. 禁止Recorder直接写final path并在trailer/finalize前显示成功。
14. 禁止用固定frame/CPU/GPU文本、静态toast或不存在的ZUI补齐产品面。
15. 禁止把Unity Graphics capture hook当作Unity完整视频播放器/Recorder参考。
16. 禁止把Bevy/Fyrox本轮未见成熟媒体产品作为降低目标的理由。
17. 禁止将FFmpeg/GStreamer等单一库直接暴露为公共engine API。
18. 禁止在App、Editor、Sound和Graphics维护四套互不兼容的媒体时钟或sample DTO。
19. 禁止在required tests依赖网络、系统codec或真实摄像头而没有deterministic provider fixture。
20. 禁止在license/package/device矩阵未通过前把媒体插件标记stable。

## 16. 本轮产出边界

本轮只新增审查与重构计划，没有修改production Runtime/Editor/interface/plugin/App代码或tests。静态证据覆盖73个显式文件、14,029行、555,758 bytes、97个test attributes和1个ignored，读取时fingerprint为`34e57da870bb6175341661412da52cb91f9e448e14bcbf97a73cf7bbdd9628f4`。

5个在途文件均非本轮产生，实施前必须重算物理范围并复核其终态。本轮没有运行动态测试；此前Editor lib测试编译仍被239个既有错误/122个warning阻断。后续实现必须从M0 truthfulness/可编译基线开始，不得直接接入codec或制作静态Media Editor页面跳过架构与资格门。
