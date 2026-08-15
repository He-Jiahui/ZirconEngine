---
related_code:
  - zircon_runtime/src/core/framework/audio
  - zircon_runtime/src/core/framework/sound
  - zircon_runtime/src/asset/assets/sound.rs
  - zircon_plugins/sound/plugin.toml
  - zircon_plugins/sound/runtime
  - zircon_plugins/sound/editor
  - zircon_plugins/sound/dist
  - zircon_plugins/sound/features
  - zircon_plugins/audio_importer
  - zircon_plugins/asset_importers/audio
  - zircon_plugins/opus_importer
  - zircon_plugins/ai/runtime/src/perception
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/zircon_plugins/02-sound.md
  - docs/plans/zircon_plugins/02/2026-07-18-m1-kira-hardcut-current-source.md
  - docs/plans/zircon_plugins/02/failure-2026-07-19-kira-send-frame-capture-routing.md
  - docs/plans/performance/01/2026-07-18-runtime-core-framework-audio-ui-static-review.md
  - docs/plans/performance/01/2026-07-30-runtime-framework-sound-static-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Public/AudioMixerDevice.h
  - dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Private/AudioMixerSourceManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Sound/SoundWave.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/AudioDevice.h
  - dev/godot/servers/audio/audio_server.h
  - dev/godot/servers/audio/audio_server.cpp
  - dev/godot/servers/audio/audio_stream.h
  - dev/godot/scene/3d/audio_stream_player_3d.h
  - dev/Fyrox/fyrox-sound/src/engine.rs
  - dev/Fyrox/fyrox-sound/src/buffer/streaming.rs
  - dev/Fyrox/fyrox-sound/src/context.rs
  - dev/Fyrox/fyrox-sound/src/source.rs
  - dev/bevy/crates/bevy_audio/src/audio_output.rs
  - dev/bevy/crates/bevy_audio/src/audio_source.rs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 08B · Audio Runtime 工程化差距

## 1. 结论

Zircon Sound 已经完成了一段真实且应保留的基础工程：生产播放核心硬切到 Kira 0.12.2，CPAL stream 和音频线程由 Kira owner 管理；重复播放能够复用 `Arc<[Frame]>`；mixer graph 在 manager 锁外编译，以 revision 与 active-state 复核避免把过期计划发布到 Kira；参数更新使用 10 ms Tween；descriptor validation、typed error、稳定 track id、容量预检、设备 stopped/started/unavailable 状态和按 world 有界的 gameplay emission journal 都已存在。AI perception 也确实消费 `SoundGameplayEmission`，不是只有声音插件内部自测。后续不能为了补功能重新引入自研 callback/mixer，也不能抹掉这些边界。

但当前产品并没有形成“场景里的 AudioSource 会在游戏中发声”的最小闭环。Sound plugin 注册的是 component descriptor、options、event catalog 和 lazy manager，没有 `RuntimeSceneSystem`，没有计划中声明的 `sound.spatial_update`，没有生产路径从 scene world 读取 source/listener/volume。默认 manager 以 inactive Kira engine 创建，全仓唯一生产 `start_output_device()` 调用在 sound editor live-output controller；该 controller 又只在自身测试中构造。`zircon_app`、runtime session 与 export 没有启动声音输出。组件、mixer、timeline、dynamic event 和 acoustic debug 的 API 面因此显著大于产品执行面。

当前 Kira bridge 仍是明确的 M1 子集：任何 effect、track pan/左右增益/delay/phase invert/solo/bypass、pre-effect send 都被 validation 拒绝；结构图在存在 active playback 时拒绝更新；source 只作为 centered static sound 播放。旧 `engine/dsp`、HRTF、occlusion、convolution 与 volume 代码处理独占 `&mut [f32]` block，却没有接到 Kira callback 或 custom effect。`SoundAsset` 和 importer 全量解码 PCM，播放又生成第二份 stereo frame；没有 streaming、cook artifact、residency/eviction、voice stealing/virtualization、hotplug supervisor 或真实 callback telemetry。

因此本轮登记 20 项 P1 和 5 项 P2，没有新增 P0。最先要做的不是继续添加效果 DTO，而是把 product bootstrap、per-world scene ownership、配置 generation、prepared/streamed asset、Kira graph/spatial adapter、设备监督与真实观测做成一个可运行、可关闭、可重载、可测量的系统。只有相同语义和质量条件下的 workload 数据才能支持“性能优于 Unreal”；静态代码量、mock test 数量和减少功能都不能作为证据。

## 2. 审查边界与覆盖

### 2.1 已读范围

| 范围 | Rust 文件 | Rust 行数 | `#[test]` | 证据等级 |
|---|---:|---:|---:|---|
| `core/framework/audio` | 3 | 331 | 2 | E3：layout、channel 与 frame conversion contract |
| `core/framework/sound` | 28 | 2,114 | 8 | E3：manager、source/listener/volume、graph、timeline、event 与 config contract |
| sound runtime production | 229 | 10,971 | 0 | E3：Kira bridge、engine state、configuration、output、automation、timeline、events、package |
| sound runtime tests | 1,035 | 12,411 | 344 | E2：合同、结构守卫与 mock 行为；真实设备和产品证据缺失 |
| sound editor/dist/features | 28 | 2,186 | 18 | E3：registration、authoring descriptor、live-output DTO、optional feature projection |
| audio importer families | 12 | 1,695 | 23 | E3：WAV/Symphonia decode、descriptor catalog 与 Opus diagnostic path |

运行时生产子树中，`service_types` 约 3,501 行、`engine` 约 2,628 行、`kira_bridge` 约 2,204 行、`automation` 约 740 行、`descriptor_validation` 约 427 行、`output` 约 278 行、`dynamic_events` 约 214 行。沿产品调用链还复核了 asset `SoundAsset`、plugin catalog、app/runtime bootstrap、scene APIs、AI perception consumer、Editor extension/viewport provider 和既有 performance/failure 记录。

测试范围共有 1,035 个 Rust 文件和 344 个 test 属性，其中约 132 个使用 `include_str!`、528 处 `.contains(...)`；没有发现 Criterion/`#[bench]`、property test、Loom、sanitizer、soak 或真实 DefaultBackend/device 启动证据。大量细粒度 source-shape test 能约束拆分和声明，但不能证明 Kira/CPAL、场景、设备、streaming、故障和规模行为。

对照范围包括 Unreal AudioMixer device/source manager 和 SoundWave cooked/streaming surface；Godot AudioServer driver、bus/effect/playback 和 realtime-safe lifetime；Fyrox engine/context/source 与 streaming buffer；Bevy 基于 `GlobalTransform` change detection 的 emitter/listener ECS integration 和 encoded-byte asset。Unity Graphics 参考树拥有 SRP、render graph、shader 和 GPU resource lifetime，不是音频 owner，因此不制造错误类比，Unity 对照留给 09/10 graphics 单元。

### 2.2 明确未做

- 没有改 production code，没有运行 Cargo、真实声卡、Editor/App/Export、音频分析仪、跨平台、性能或长时稳定性测试。本篇是 current-source 静态审查与重构计划，不是实现验收。
- 当前 focused sound source 没有工作区修改，但仓库整体存在大量其他 Session 写入；M1 历史记录与仍 open 的 send frame-capture failure 也不一致。实现前必须重新取指纹并运行 current-source gate，因此标记 `source_recheck_required`。
- 没有把 AI、animation、physics、network 或 renderer 内部实现计入 Audio 完成度。AI gameplay emission consumer 是已确认的 boundary；physics occlusion、animation timeline 和 network voice 分别仍由所属系统计划拥有。
- 没有以 Unreal 的类数量作为目标。参考它的 owner、streaming/cook、device、virtualization 与 telemetry 边界，并用 Zircon/Kira 架构取得更少复制、更低控制延迟或更高吞吐。

## 3. 当前闭环与必须保留的能力

### 3.1 Kira 是唯一生产执行核心

Kira/CPAL 已拥有 realtime callback、音频线程和 handle command semantics。Zircon 的 graph compiler 在状态锁外构建计划，进入锁后核对 graph revision 与 Kira active state，最多重试八次；稳定参数通过 Tween 更新。这是正确方向。后续应把缺失能力映射为 Kira track/spatial/effect/clock/modulator/streaming adapter，或明确扩展 Kira backend，不得恢复手工 `render_mix`、producer thread、owned-block callback 和第二套 mixer。

必须准确描述锁边界：当前大的 `SoundEngineState` mutex 会串行控制面、graph apply、timeline/event/snapshot，但 Kira audio callback 不会取得这把 mutex。问题是控制面规模、发布一致性和可观测性，不应错误宣称 callback 直接被 Zircon manager mutex 阻塞。

### 3.2 graph validation、typed error 和 bounded gameplay journal 已有基础

Graph track/send id 稳定，操作有排序和容量预检，descriptor validation 对有限数、引用和多种参数有较完整覆盖。source/playback/device API 返回 typed error，而不是一律 bool。gameplay emission journal 按 `WorldHandle` 有容量和序列缺口报告，AI perception 已有生产读取路径。这些合同应迁入新的 generation/slot 架构，不应为了换 owner 而退化为字符串命令或无界 Vec。

### 3.3 manifest 对 core runtime 的 `partial` 判断比 completed 可靠

core sound capability 没有写 complete，默认 backend 已硬切为 `kira-cpal`，旧 software mixer 不是 alias/default。`docs/plans/zircon_plugins/02-sound.md` 也只把 Kira M1 列为当前基础，M2-M5 checkbox 尚未完成。问题在于 beta maturity、optional feature projection 和 Editor surface 仍会让产品层误以为能力可用。重构应继续采用精确 capability，不要通过删掉 `partial` 状态来制造完成感。

## 4. P1 差距清单

### P1-1：产品 bootstrap 没有启动音频，也没有场景执行系统

Sound runtime descriptor 提供 lazy module/manager，却没有任何 `RuntimeSceneSystem`。全仓没有 `sound.spatial_update` 注册，没有 sound production code 调用 `node_records()` 或 `world_transform()`；`create_source`、`update_source`、`update_listener`、`update_volume` 没有 app/runtime scene caller。`DefaultSoundManager::new` 创建 inactive Kira，唯一生产 `start_output_device()` caller 是 Editor live-output controller，而该 controller 只在自身 tests 构造。

目标增加明确的 `AudioRuntimeSupervisor` 与 `AudioSceneSystem`。session activate 根据 runtime profile/config 启动或选择 headless backend，成功后发布 capability generation；scene extraction 在定义清楚的 stage 处理 source/listener/volume create/change/remove；session shutdown 先关闭 admission、停止/淡出 voices、drain completion，再 drop Kira。Editor preview、Play、standalone app、exported client 和 dedicated server 必须各有明确策略，不能由用户偶然调用 manager API 才开始工作。

### P1-2：一个全局 SoundEngineState 混合所有 world、session 与 preview 状态

clips、playbacks、sources、listeners、volumes、graph、events、timeline、parameters、meters 和 Kira manager 都位于单个全局 state；只有 gameplay emission journal 以 WorldHandle 分桶。source/listener/volume descriptor 没有 world owner，scene unload、world replacement、Editor preview 与 Play 切换不会自然释放其状态。clip 也没有 unload/remove，生命周期等于 manager。

目标是 global backend/device owner + per-session/per-world `AudioWorldSlot`。slot 记录 replacement epoch、scene generation、source/listener/volume bindings、voices、timeline/event cursors、bus routing 与 health；global clip cache只持有引用计数和 residency。world close 原子关闭 admission、取消异步 decode/ticket、停止或按 policy 淡出 voices、释放 bindings；旧 completion 不得提交到新 world。Editor preview 与 Play 默认不同 slot，可显式 route 到同一 device mixer，而不共享 gameplay state。

### P1-3：plugin options 和项目配置没有进入 manager factory

`DefaultSoundManager::new` 无条件采用 `SoundConfig::default()`；`SoundConfig::from_plugin_options` 的生产消费者为零。package options 目前只是 metadata。`default_spatial_scale`、HRTF profile/enable、convolution enable/budget、ray-tracing quality、default mixer preset、timeline integration 和 dynamic events enable 等字段在生产代码中没有执行消费者。global volume/spatial scale 只改内存，没有 `ConfigStore` generation/persistence transaction。

目标建立 typed `AudioConfigGeneration`：project defaults、runtime profile、platform/device overrides、user settings 与 session overrides 按明确优先级合成；validate/prepare device+graph+world transition，持久化成功后一次 publish desired generation，再由每个 owner 报 applied generation。每个字段标记 live Tween、next-frame、graph rebuild、device restart 或 process restart 语义。失败保持 last-good config/device/graph，不得出现持久化失败但运行态已变。

### P1-4：Kira graph 只实现 M1 子集，效果与 live structural mutation 均未兑现

`validate_m1_surface` 拒绝任何 track effect，也拒绝 pan、left/right gain、delay、phase invert、solo、bypass 与 pre-effect send。公共合同和 Editor operation 描述了 12 类效果，但 Kira track 未安装 effect handle。存在 active playback 时，结构 graph edit 直接失败；没有 graph generation crossfade、atomic route swap 或 effect state migration。曾接受的 send routing 记录与仍 open 的 current-source failure 还没有统一结论。

目标由 `AudioGraphCompiler` 把 validated immutable graph 编译为 Kira track/send/effect plan，生成 stable binding slots与能力诊断。参数 update 走 handle/Tween；可增量结构变更构建 shadow bindings 后在明确 frame/clock boundary publish；不可迁移效果定义 crossfade/tail policy。旧 graph 保持工作直到新 graph ready，失败不影响 last-good。M2 效果必须通过 impulse/frequency/dynamic-range 金样、live edit 无 click/pop 和 send routing product capture，不能用 DTO/operation test 标完成。

### P1-5：AudioSource、Listener、Volume、HRTF 与 occlusion 没有接入 Kira execution

`update_listener_impl` 和 `update_volume_impl` 仅验证后插入 HashMap；source playback data 只设置 centered pan。位置、forward、velocity、distance attenuation、doppler、air absorption、volume priority、HRTF 和 physics occlusion 均未写到 Kira spatial track或 custom effect。旧 `engine/source_environment`、HRTF、occlusion、convolution 对 `&mut [f32]` block 运算，没有 callback consumer；output `render_mix`/callback pull 又明确 Unsupported，因为 callback 已交给 Kira。

目标由 scene change extraction 维护 stable listener/emitter handle，基于 transformed scene generation 只更新 changed transform/velocity。简单空间化映射 Kira spatial track；自定义 attenuation、doppler、air absorption、occlusion、volume 与 HRTF 通过参数/track/effect adapter组合。Physics query 使用有预算、固定 generation 的 batch结果，并对缺失/过期结果定义 hold/fade/fallback。旧离线 DSP 要么迁为 Kira custom effect并通过 realtime约束，要么删除；禁止维持一套永远到不了输出的“完整算法”。

### P1-6：两个 optional sound feature 只是空 ModuleDescriptor

`sound.ray_traced_convolution_reverb` 和 `sound.timeline_animation_track` runtime feature 的 `register()` 只注册空 module descriptor，没有 system、service、provider 或 executable behavior；dist 诊断还宣称实现由 runtime module hosted，但 module为空。Editor/dist projection 因而只是包与菜单层声明，无法证明功能存在。

目标把 capability 分成 `Unsupported/Stub/Partial/Ready` 并附 executable provider id、dependencies 和 runtime probe。未实现期间从 production/export profile移除，maturity降为 experimental/stub；实现时分别接入 acoustic query/IR cook/effect runtime 和 animation/timeline bridge，提供真实 product scene。禁止保留空 module 来满足 manifest test。

### P1-7：SoundAsset/importer 全量解码，缺少 prepared/streamed cook artifact

`SoundAsset` 保存完整 interleaved `Vec<f32>`。WAV/MP3/OGG/FLAC/AIFF importer 通过 Symphonia 把全部源数据解码到增长 Vec；播放再转换为 `Arc<[kira::Frame]>`，导致 source PCM 与 stereo frame 双份常驻，mono 还会复制到两个 channel。大音乐、语音和长 ambience 的导入延迟、峰值内存与运行时 residency 不受控制。

Unreal SoundWave 明确区分 cooked platform data、streaming chunks、first chunk、loading behavior、seek offset和derived data；Fyrox 有 streaming buffer；Bevy asset至少保留 encoded `Arc<[u8]>` 而非强制全量 PCM。Zircon 应建立 `AudioClipCookArtifact`：source/import setting/codec/backend/platform/version hash、metadata、loudness/peak、loop/marker、seek table和chunk manifest。短音效 prepared、长内容 streamed，策略由 profile/budget/cook决定；runtime 不重新解释原始文件。

### P1-8：clip load 没有 single-flight、unload、eviction 与 residency budget

`load_clip` 在取得 state lock 前完成 resolve/load/decode/frame conversion，随后才检查 locator dedup。并发或重复请求可浪费整次加载与内存。LoadedClip 同时保留 asset 和 Kira frame，没有 unload/remove、LRU、reference count、pin、memory class、streaming buffer high-water 或 hot-reload generation。

目标由 AssetManager 和 `AudioClipResidencyManager` 共同拥有 locator+artifact-generation single-flight。prepared PCM/page和stream decoder分别计费，支持 pin/lease/refcount、cancel、evict、last-good reload与错误负缓存。voice持有明确 lease，释放后资源可回收；Editor waveform/preview与runtime播放使用同一artifact但不同residency class。诊断必须显示decoded bytes、encoded/chunk bytes、pending decode、cache hit、eviction与超预算拒绝。

### P1-9：超过 voice 容量只报错，没有优先级、并发组、stealing 或 virtualization

`max_voices` 只在创建时形成硬上限；满容量后调用失败。没有 source priority、distance/audibility score、owner concurrency group、per-sound limit、oldest/quietest/lowest-priority stealing、virtual voice、restart/resume offset或inaudible update budget。大量环境声或射击声会以 API失败暴露给 gameplay，而不是可预测地退化。

目标建立 `VoiceAllocator`：real/virtual/pending/stopping状态，priority与audibility score，per-world/global/group budgets，deterministic tie-break，steal/fade policy，loop virtual advancement和promotion。Kira handle只对应real voice；virtual voice保存轻量timeline。所有拒绝/steal/virtualize有reason telemetry。与Unreal比较时统一真实voices、virtual voices、效果/空间质量和输出buffer，禁止把少播声音当性能优势。

### P1-10：playback completion 依赖任意 manager 调用轮询，source update 会 stop/restart

Kira completion 清理只在之后调用 manager API时发生，没有每帧 scene/runtime pump；finished event和completion action不会自动进入ECS。direct playback/source map可能长期保留已结束项。source descriptor update通过移除、停止、按起点重启实现，而不是对音量、位置、rate、loop等做属性diff，容易产生时间跳变和爆音。没有sample-accurate scheduled start、同步transition或scene entity completion policy。

目标让 AudioRuntimeSupervisor 固定频率poll Kira handle/status并写有界 completion ring，AudioSceneSystem按world cursor消费，执行 stop/despawn/remove/retrigger等typed action。source update编译为live handle commands，只有输入asset/不可迁移graph变化才按policy重建voice并保留position/crossfade。scheduled play/stop/transition通过Kira Clock或等价sample timeline，游戏frame只提交目标时间，不承担采样级轮询。

### P1-11：timeline 是手动调用的全量解释器，不是 sample-accurate automation

全仓没有 runtime system 调用 `advance_timeline`。调用时 manager持全局state mutex，`mem::take`全部sequences，逐advance验证/采样curve，clone bindings并创建sample/application/report Vec。它没有编译direct parameter slot，没有Kira Clock/Modulator，也没有固定audio timeline与game clock之间的转换、seek/scrub/loop/late command合同。optional timeline animation feature又是空module。

目标在schedule/graph generation阶段一次验证并编译curve/binding为stable slot、segment cursor和Kira Tween/Clock/Modulator计划；runtime只推进active cursors，scratch复用，详细sample report按需。定义game/editor/animation/audio clock映射、pause/time-scale、scrub、loop、late event与graph generation失效语义。timeline worker不能持全局manager mutex执行sequence×track×keyframe工作。

### P1-12：dynamic event pending无界，dispatch无预算并同步执行foreign executor

dynamic events同样没有production pump。pending是无界 `Vec<SoundDynamicEventInvocation>`；submit clone catalog后线性扫描；dispatch一次drain全部pending，每event clone/sort handlers并clone payload。executor map被clone，任意executor在caller线程同步调用，没有affinity、timeout、cancel、late result或故障隔离。突发marker/impact可产生无界内存和frame hitch。

目标按catalog generation编译 event-id 到 pre-sorted handler/executor slots；payload使用单一shared owner。ingress按entries/bytes/age/per-world设预算，dispatch按count/time/bytes分片；foreign executor通过Runtime task/bridge提交bounded ticket并声明affinity、deadline、cancel与late-result policy。event执行结果形成有界telemetry，不能阻塞audio callback或scene frame。

### P1-13：设备枚举、选择和重启不是可持续的 device supervisor

CPAL catalog只取display name，id为 `kira-cpal:{name}`，重复名称、重命名和系统切换不稳定；descriptor主要复制当前config，而非枚举真实supported formats。available判断基本只限制channel count，未验证sample rate/fixed buffer兼容。启动强制固定buffer/rate/channel，设备可能拒绝。没有hotplug/default-device change、fallback、reopen backoff、seamless migration或device-lost事件。

`configure` 会停止direct playback并deactivate Kira；`start_output_device` 在config+state锁下执行 `AudioManager::new`、graph/source sync，可能阻塞其他控制调用。目标建立单一 `AudioDeviceSupervisor` actor/state machine：enumerate stable fingerprint与supported configs，prepare新device/manager，恢复graph与voices，crossfade/切换后retire旧device；失败保留last-good或进入明确silent/headless fallback，带指数backoff和用户可操作诊断。device operation不在global state mutex内执行。

### P1-14：output diagnostics 的 callback/underrun/meter 数据是静态零值

`rendered_blocks`、`rendered_frames`、`callback_count`、`last_callback_sequence`、`underrun_count` 在production只初始化、reset和snapshot，没有来自Kira/CPAL的更新。`latency_frames`与meters也没有callback数据写入；输出latency只是按请求descriptor/block估算。Editor mixer/health UI因此可能展示稳定的零值，而不是明确unsupported，制造健康假象。

目标从Kira/CPAL可支持边界采集真实callback cadence、xrun/underrun、stream error、device restart、command latency、real/virtual voice、track peak/RMS/clip、decode starvation和ring depth。realtime端只写preallocated SPSC/atomic page，不分配不锁；collector按cursor读取并标明sample window/generation/staleness。无法采集的字段返回Unsupported，不得填0。测量latency与估算latency分字段显示。

### P1-15：控制面集中在一把 mutex，snapshot/timeline/event/graph apply 放大延迟

一个 `Arc<Mutex<SoundEngineState>>` 串行clips、playbacks、sources、listeners、volumes、graph、Kira handles、timeline、events与meters。graph compile虽在锁外，但apply可在锁内执行大量Kira track操作；既有测试甚至容忍1,000-track apply约75ms。snapshot深clone graph/source/binding/event/meter。poison recovery直接 `into_inner` 继续，无法证明半完成graph/Kira binding仍一致。config和state分锁也没有跨owner generation事务。

目标拆分device/backend actor、immutable graph generation、per-world slots、clip residency、bounded command ingress和telemetry ring。调用方提交typed command/ticket，昂贵prepare在worker/actor完成，短临界区只publish generation或handle table。snapshot默认返回Arc page/delta/cursor，full capture显式预算。poison或Kira apply故障把对应owner标Faulted并回滚last-good，不得盲目继续。

### P1-16：External、Synth 与 Silence source input 暴露在合同中却没有生产adapter

`SoundSourceInput::External`、`SynthParameter` 和 Silence可被创建，external block也可提交/存储，但播放返回 `UnsupportedAdvancedFeature`，没有Kira M1 adapter。没有procedural generator、pull/push ring、format negotiation、clock drift、backpressure、microphone或voice path。公共beta surface因此允许调用方构建永远不能发声的source。

目标在capability里逐input kind声明Unsupported/Ready并在authoring/admission提前拒绝。真正实现时使用bounded realtime-safe ring或Kira custom sound data，定义producer affinity、sample format/layout、timestamp、underrun/overflow、resample/drift、shutdown和ownership；Silence若仅用于test，应移到test helper，若用于timeline占位则定义virtual duration/completion语义。

### P1-17：Editor 是注册与DTO脚手架，尚无真实authoring/preview/debug产品闭环

Sound editor注册view/template/operation/inspector customization，存在mixer console、source/listener/volume drawer与acoustic debug ZUI。可是 live-output controller没有production构造，测试只用FakeSoundManager；acoustic debug没有 `ViewportOverlayProviderRegistration`。Editor也拿不到真实meter/callback telemetry，scene component本身又没有runtime system，因此操作descriptor不能证明Play/preview会生效。

目标由Sound-owned Editor extension构造/销毁preview `AudioWorldSlot`，绑定project config/device supervisor和undo transaction；Inspector的asset/source/spatial/volume/mixer/timeline编辑使用runtime同一typed validator。viewport overlay消费scene/audio generation，显示listener、attenuation、cone、volume、occlusion rays和stale状态，有预算与selection过滤。preview、Play、stop、scene reload、device change和plugin unload必须有真实product测试。

### P1-18：三个 importer 家族形成重复authority，Opus高优先级仍是诊断占位

`audio_importer` 是builtin catalog中的真实WAV/Symphonia importer，同时提供diagnostic-only Opus；`opus_importer` 也在builtin catalog，priority更高但同样因缺libopus只返回诊断。`asset_importers/audio` 又定义平行descriptor/manifest identity，却没有runtime register实现，也不在canonical builtin catalog。发现、优先级和文档会呈现多个“音频 importer”，但真正authority不清晰。

目标硬切为一个Audio Import/Cook package owner和一个codec registry。descriptor identity、extension/codec probe、priority、platform support、license/native dependency、version/cache key都从同一generation发布。Opus要么接入可发布的codec并通过decode/seek/error/fuzz/cross-platform验证，要么从ready capability与默认catalog移除；descriptor-only重复包删除或迁移后删除，不能长期以alias保留。

### P1-19：运行时只支持mono/stereo静态播放，多声道导入会到播放阶段才失败

importer可以接受超过2 channel的资产，但Kira playback conversion只支持mono/stereo，错误推迟到运行时。设备config也把channel限制在2；没有speaker layout、downmix matrix、ambisonics、channel bed/object audio或per-platform cook裁决。计划曾将5.1/7.1退出v1，这是可以接受的MVP范围，但当前导入与产品capability必须保持一致。

目标在import/cook阶段按target profile决定reject、validated downmix或保留encoded multichannel artifact；runtime surface报告output/source/layout capability。stereo路径建立ITU/项目定义的downmix、loudness/headroom和phase测试，不能简单丢channel。未来多声道若Kira backend不支持，应作为独立backend extension，经device layout、mixer、effect、asset和product全链验收，而不是在现有Frame类型旁加临时数组。

### P1-20：现有完成记录偏重库内tests，current-source与产品证据治理不一致

sound runtime有344个test属性，M1记录称当时managed tests全绿；但多数是mock/source-shape测试，没有DefaultBackend真实设备启动、scene system、app/export、streaming、fault、soak或benchmark。`2026-07-18-m1-kira-hardcut-current-source.md` 的open failures为0，而次日send frame-capture failure仍是open；可以是时间顺序变化，但当前索引没有给出统一last-good/current-source结论。不能据此推断现在存在routing bug，也不能声称已验收。

目标采用 Contract/Backend/Scene/Product/Acceptance 五层证据：DTO/mock是Contract；Kira/CPAL real backend是Backend；world lifecycle/transform是Scene；Editor/App/Export是Product；platform/fault/scale/quality是Acceptance。每个capability保存current source fingerprint、artifact/build id、命令、设备/profile、结果和expiry。历史failure不删除，由最新current-source receipt明确supersede或保持open。

## 5. P2 扩展差距

### P2-1：音频采集、麦克风、voice chat 与回声处理没有架构owner

当前output-only架构没有capture device、permission/lifecycle、AEC/NS/AGC、encoding、jitter buffer、network clock、spatial voice、privacy或recording artifact。它不能通过复用ExternalSource一个Vec临时完成。Network08E完成transport/identity/security基线后，应单独建立Audio Capture/Voice模块，以bounded packet/audio rings、codec worker、device supervisor和per-world spatial bridge接入。

### P2-2：ray-traced acoustics 与 convolution IR 尚无可执行资产/调度系统

ray tracing provider/status/config与optional feature有合同，但没有geometry extraction、material acoustic data、probe/IR bake、runtime ray budget、temporal filter、async cancellation或Kira convolution effect。实现应在Physics/Graphics query owner稳定后建立独立Acoustic Scene generation，离线bake与runtime rays共享material/geometry identity；结果以低频参数或IR transition喂给Audio，而不是让audio callback发射查询。

### P2-3：交互音乐、sample-accurate graph 和大型内容工作流仍未形成产品系统

现有timeline/marker/event合同不足以覆盖tempo map、beat/bar quantization、section/stem transition、sync group、pre-roll、branching、latency compensation与authoring audition。应在M0-M6基础完成后建立Music Runtime，以AudioClock为唯一sample timeline，资产cook预计算beat/seek数据，Editor提供非破坏编辑和可重复capture；不能把这些语义继续塞入通用dynamic event字符串。

### P2-4：离线渲染、确定性回放与自动音质回归缺少owner

Kira DefaultBackend路径不提供当前合同中的owned-block `render_mix`，这是避免第二套production mixer的正确结果，但CI仍需要可控offline renderer/capture用于金样、mix comparison和bug复现。应通过Kira mock/custom offline backend或受支持capture tap建立同一graph/effect执行链，记录clock/commands/assets/config digest；禁止维护与production不同的DSP实现来生成测试金样。

### P2-5：高级平台音频、可访问性与专业制作能力尚未规划

当前没有platform spatial audio、object/ambisonic output、dynamic range presets、dialogue ducking、loudness normalization、hearing-accessibility mix、localized dialogue/streaming package或console/mobile lifecycle细节。它们应在产品需求明确后以profile、asset cook、mixer policy和平台backend capability分层实现，不通过扩大`SoundConfig`未消费字段来“预留完成度”。

## 6. 参考引擎差距裁决

| 工程问题 | Zircon 当前 | 参考源码给出的边界 | Zircon 裁决 |
|---|---|---|---|
| ECS/scene integration | 无scene system，组件不驱动manager | Bevy query source/listener `GlobalTransform`并按Changed更新 | per-world extract + stable Kira emitter/listener binding |
| execution owner | Kira owner正确，但只由Editor测试路径启动 | Fyrox engine/context明确output/headless lifecycle；Godot AudioServer拥有driver | global backend/device supervisor + product bootstrap |
| asset memory | 完整PCM + stereo Frame双份常驻 | Unreal SoundWave cooked/chunk/first-chunk/loading behavior；Fyrox streaming buffer | prepared/streamed cook artifact + residency manager |
| mixer/effects | M1 graph，effect/structural live edit拒绝 | Godot bus/effect/playback由AudioServer集中管理；Kira提供track/effect handle | immutable graph generation + Kira effect/route adapter |
| realtime lifetime | Zircon control state集中，callback由Kira管理 | Godot使用atomic playback state和delayed deallocation；Kira command handle | callback不锁不分配，control actor发布generation |
| devices | name id、请求格式、无hotplug/recovery | Unreal/Fyrox/Godot都有明确device/driver生命周期owner | device supervisor、真实format probe、fallback/reopen |
| voices | 容量满直接error | Unreal SoundWave/AudioDevice拥有virtualization/loading policy | priority/concurrency/steal/virtual voice allocator |
| telemetry | callback/meter/underrun静态0 | AudioMixer device/source manager拥有实时统计边界 | realtime ring + collector + unsupported/stale语义 |

参考源码不要求 Zircon 复制 Unreal 的AudioDevice类层次，也不要求替换Kira。需要吸收的是：内容在cook时决定加载策略，长期owner持有device/world/voice状态，realtime线程只做有界工作，scene变化增量进入执行，能力与产品证据一致。若Zircon架构更简单，应以更低command latency、更少resident bytes、更低xrun和更好的scale curve证明。

## 7. 目标架构

### 7.1 owner 与数据流

| Owner | 职责 | 禁止承担 |
|---|---|---|
| AudioRuntimeSupervisor | session admission、global backend、device supervisor、shutdown | scene node扫描、Editor transaction |
| AudioDeviceSupervisor | CPAL device/format probe、Kira manager、restart/fallback/health | 全局clips/timeline/events mutex |
| AudioWorldSlot | world epoch、source/listener/volume/voice、timeline/event cursors | 跨world共享可变真相 |
| AudioSceneExtract | changed component/transform到bounded commands | 每帧全world snapshot或直接持Kira handle |
| AudioGraphRuntime | immutable graph generation、Kira binding、effect/route migration | asset decode和device枚举 |
| VoiceAllocator | real/virtual/pending voices、priority/concurrency/budget | PCM所有权和scene mutation |
| AudioClipResidency | artifact single-flight、prepared/streamed lease、decode/cancel/evict | fixed/frame线程同步解码 |
| AudioClock/Automation | sample timeline、compiled curves/events、late policy | 每advance重复验证全curve |
| AudioTelemetry | realtime ring、cursor pages、capture artifact | callback中分配/锁/full snapshot |
| Sound Editor Extension | typed authoring、preview slot、overlay、meter UI | 私建第二个audio runtime |

建议主链为：

1. import/cook将源文件变为profile-keyed `AudioClipCookArtifact`，发布metadata、prepared/chunk/seek数据。
2. session activate由RuntimeSupervisor读取typed config generation，DeviceSupervisor准备Kira manager和last-good graph。
3. scene change tick输出source/listener/volume create/update/remove command，附WorldHandle、replacement epoch与scene generation。
4. AudioWorldSlot解析asset lease、voice policy和graph route；VoiceAllocator决定real/virtual，real voice通过短Kira command创建或更新。
5. AudioClock推进sample timeline，completion/dynamic events进入bounded pages；scene/AI/animation按cursor消费，不能反向轮询全状态。
6. callback/Kira只读取预先发布的声音、track和effect state；telemetry通过preallocated ring流出。decode、device open、graph prepare和foreign executor均不在callback或scene锁内。
7. world/session/device close按generation停止admission、取消异步工作、淡出/停止、等待deadline并retire；旧ticket/completion因epoch不匹配被丢弃并计数。

### 7.2 硬切要求

- 删除或迁移所有无法进入Kira production execution的旧DSP/HRTF/source-environment block path；不能保留“未来可能接线”的第二套mixer。
- 删除global `SoundEngineState`作为所有子域共同可变owner，迁移后不保留旧manager双写兼容层。
- importer authority收敛后删除descriptor-only重复package与diagnostic-only ready能力，不保留永久alias。
- `SoundAsset`不再以完整`Vec<f32>`作为所有音频的唯一runtime artifact；prepared与streamed必须从cook profile决定。
- output counter/meter若没有真实producer必须删除字段或返回Unsupported，禁止继续以0伪装有效测量。
- 未实现的source kind、effect、optional feature在admission前拒绝并降级capability，不能等到播放时才暴露。

## 8. 分层重构里程碑

### M0：能力真相、配置与产品启动基线

- 重新核对Kira M1和open send routing failure的current-source状态，生成Contract/Backend/Scene/Product/Acceptance矩阵。
- 收敛SoundPluginOptions到typed AudioConfigGeneration，修正maturity、optional feature和unsupported source capability。
- 接入Editor/App/Export/Server RuntimeSupervisor；真实设备与headless策略明确，shutdown闭环可观测。

退出条件：最小scene中的AudioSource在Editor Play、runtime app与export发声；server明确headless；所有profile都能报告实际backend/config/device/graph generation，不靠手工manager调用。

### M1：per-world scene slot 与生命周期

- 建立AudioWorldSlot、replacement epoch、bounded scene command和completion/event pages。
- 实现source/listener/volume create/change/remove，component/transform增量更新。
- Editor preview/Play隔离，world reload/session teardown取消旧工作并释放voice/binding。

退出条件：scene零变化frame不扫描全world；create/update/remove和world replace无stale voice；finished action与AI emission按cursor工作；1,000次Play/reload无handle/thread/memory泄漏。

### M2：资产cook、streaming、residency与voice management

- importer authority硬切；建立AudioClipCookArtifact、codec registry、chunk/seek/loudness/loop metadata。
- prepared/streamed single-flight、decode worker/ring、prefetch/seek/cancel、lease/eviction/last-good reload。
- VoiceAllocator实现priority/concurrency/steal/virtualization与reason telemetry。

退出条件：短SFX和数小时音乐使用不同策略；stable playback无同步decode和realloc；内存/IO有硬预算；满voice不以随机API error退化；Opus能力与实际codec一致。

### M3：Kira graph、effects 与 live mutation

- graph compiler产出immutable generation和stable Kira bindings；修复/关闭send routing evidence。
- 12类effect逐项映射Kira built-in或custom Effect，迁移可用旧DSP金样后删除旧执行路径。
- 参数Tween、结构shadow generation、tail/crossfade和failure rollback。

退出条件：active playback期间route/effect编辑无click/pop且last-good可回滚；callback分配/锁为0；效果golden、频响、动态范围与product capture通过。

### M4：空间化、HRTF、occlusion 与AudioVolume

- Kira spatial emitter/listener接入changed transform/velocity；attenuation/doppler/air absorption参数化。
- HRTF作为Kira custom effect，profile/SOFA/IR进入cook/residency；AudioVolume驱动bus/effect transition。
- Physics/acoustic query使用bounded batch和generation，明确stale/fallback。

退出条件：移动source/listener连续更新无restart；occlusion/volume/HRTF确实改变输出而非只改HashMap；CPU/quality/transition均有可重复capture。

### M5：AudioClock、timeline 与dynamic events

- sample timeline映射game/animation/editor clock，curve/binding编译为stable slot和Kira Tween/Clock/Modulator计划。
- dynamic catalog编译、bounded ingress/drain、shared payload和异步executor ticket。
- completion/marker/music transition按sample时间与world epoch发布。

退出条件：scheduled start/stop/marker误差有量化上限；大规模sequence/event不会持global mutex或无界drain；pause/seek/scrub/loop/reload语义可重放。

### M6：device、telemetry、Editor 与产品验收

- DeviceSupervisor支持真实format probe、stable identity、hotplug/default change、reopen/fallback和graph/voice迁移。
- realtime telemetry ring、diagnostic collector、mixer meter、capture artifact和故障注入。
- Editor inspector/mixer/timeline/preview/overlay全部连接canonical runtime。

退出条件：真实Windows/Linux/macOS目标按仓库支持矩阵验证；拔插/切换/拒绝格式/stream error可恢复；UI不显示伪造0；Editor/App/Export长时运行和卸载通过。

### M7：高级音频能力

- 按产品需求选择voice chat/capture、ray-traced acoustics、interactive music、offline deterministic render与multichannel/platform audio。
- 每项建立独立owner、asset/cook、security/privacy、quality/performance和product gate。

退出条件：能力不再通过未消费config字段或空optional module预占；每项有真实需求、预算和与core audio的generation合同。

## 9. 验收矩阵

### 9.1 正确性与生命周期

| 场景 | 必测内容 |
|---|---|
| bootstrap | Editor preview/Play、standalone app、export client、headless server、plugin/session shutdown |
| scene | source/listener/volume create/change/remove、hierarchy transform、world replace、reload、multiple worlds |
| playback | play/pause/resume/seek/loop/stop/finish、live parameter diff、asset reload、scheduled transition |
| graph/effect | route/send、pre/post effect、mute/solo/bypass、tail、live structural edit、rollback |
| spatial | attenuation/cone/doppler/air absorption/HRTF/volume/occlusion、stale physics result |
| asset | prepared/streamed、seek/loop/chunk loss、cancel、evict、corrupt artifact、codec unsupported |
| device | no device、unsupported format、hotplug/default change、restart failure、fallback、latency |
| events | completion、timeline marker、dynamic executor timeout、AI emission cursor/overflow/world close |

### 9.2 音质与性能

至少覆盖voices为1/100/1k/10k，real/virtual比例0/50/99%；tracks/effects/sends为1/100/1k；clips为1 KiB短SFX、10 MiB ambience、数小时music；world为1/8/64；transform changed ratio为0/0.1/1/100%；dynamic events为1/1k/100k burst；device buffers覆盖目标平台支持的低/中/高延迟档。

采集callback p50/p95/p99/max、deadline miss/xrun/underrun、command-to-audible latency、graph prepare/publish、scene extract visits/commands、real/virtual voices、steal reasons、decode throughput/queue/starvation、stream bytes/prefetch hit/seek latency、resident/peak bytes、alloc/realloc、control lock/actor queue wait、event queue depth/drop/age、meter/capture overhead、device recovery time和shutdown deadline。

硬门禁：Kira/callback realtime路径动态分配与blocking lock为0；zero-change scene全node/source扫描为0；稳定stream ring和telemetry ring realloc为0；pending decode/event/command有entry+bytes+age预算；长clip resident memory由prefetch/chunk预算而非duration线性决定；device/codec/graph失败保持last-good或明确silent状态。与Unreal比较必须固定硬件、codec、sample rate、buffer、real voices、spatial/effect质量、场景运动与采集方式。

音质至少使用impulse/frequency response、gain/pan/downmix、effect golden、loop boundary、resample alias/noise、HRTF localization dataset、click/pop detector、loudness/peak和A/B capture。性能更快但丢失send/effect/spatial/voice不算通过。

### 9.3 平台、故障与产品

- Windows MSVC、Linux及仓库声明支持的macOS/其他target分别验证DefaultBackend init/play/drop、真实设备format和export产物；无设备CI使用明确Mock/Headless profile，不冒充real backend。
- 故障注入覆盖device open/stream error、decoder error/slow IO/chunk缺失、graph apply失败、config persistence失败、world replacement race、executor timeout、queue overflow和plugin unload。
- 30分钟高voice/effect/stream soak、1,000次world/Play create-drop、连续device restart和asset reload，检查thread/handle/lease/decoder/Kira binding泄漏。
- 保存artifact manifest、source fingerprint、Kira/CPAL/codec版本、device/profile、音频capture、telemetry和命令；没有这些证据时状态只能是static review或dynamic pending。

## 10. 实施约束

- 当前仓库MVP仍未完成，本报告只授权review；实现必须按M0依赖顺序进入现有Plugins02/Runtime owner计划，不能先做voice chat或ray tracing来绕过最小产品闭环。
- `zircon_runtime::core::framework::sound` 保持backend-neutral contract，Kira和codec/device具体类型继续由sound plugin拥有；不要新建平行root audio crate逃避现有边界。
- Kira是唯一production execution core。需要缺口效果、spatial或offline capture时优先使用其正式extension/backend surface，禁止复活第二套callback/mixer。
- 实现前重新authorize/claim相关文件，复核当前open failure与source fingerprint；本轮不覆盖任何外部Session工作。
- 每个milestone保存current-source命令、产物、产品路径、音质和性能artifact。mock/source-string测试只能提供Contract证据，不能提升Backend/Product状态。

## 11. 本轮状态

本轮完成Audio framework、sound runtime/editor/dist/features、三个importer family、asset与产品调用点的首轮E3静态审查，未改production code。Audio进入 `review_complete / implementation_pending / source_recheck_required`；08总单元继续审查Animation、Navigation和Network，Graphics/RHI/renderer不由本篇覆盖。
