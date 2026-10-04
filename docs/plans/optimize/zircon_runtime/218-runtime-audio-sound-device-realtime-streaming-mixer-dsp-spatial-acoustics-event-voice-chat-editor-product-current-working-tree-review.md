---
title: Runtime Audio / Sound 当前工作树设备、实时线程、流式资源、混音 DSP、空间声学、事件、Voice Chat、Editor 与产品闭环复审
category: zircon_runtime
report_id: Runtime218
review_date: 2026-09-01
baseline_head: 9963f8eb72e2d725d2536eb50b393b30387a1ffa
baseline_epoch: 2026-09-01-current-working-tree
verification_head: 9963f8eb72e2d725d2536eb50b393b30387a1ffa
verification_epoch: 2026-09-01-current-working-tree
supersedes_currentness_of:
  - docs/plans/optimize/zircon_runtime/168-runtime-audio-sound-current-working-tree-device-streaming-mixer-spatial-automation-review.md
  - docs/plans/optimize/zircon_runtime/99zn-runtime-audio-sound-clip-streaming-device-mixer-bus-effect-spatial-occlusion-reverb-timeline-event-voice-chat-editor-product-integration-current-source-review.md
related_code:
  - zircon_runtime/src/core/framework/audio
  - zircon_runtime/src/core/framework/sound
  - zircon_runtime/src/asset/assets/sound.rs
  - zircon_runtime/src/asset/importer/ingest/import_sound.rs
  - zircon_plugins/sound/runtime
  - zircon_plugins/sound/editor
  - zircon_plugins/sound/dist
  - zircon_plugins/sound/features
  - zircon_plugins/audio_importer/runtime
  - zircon_plugins/asset_importers/audio
  - zircon_plugins/opus_importer
  - zircon_plugins/first_party_runtime_catalog
  - zircon_plugins/first_party_editor_catalog
  - zircon_app
plan_sources:
  - docs/plans/optimize/zircon_runtime/08b-audio-runtime-review.md
  - docs/plans/optimize/zircon_runtime/99zn-runtime-audio-sound-clip-streaming-device-mixer-bus-effect-spatial-occlusion-reverb-timeline-event-voice-chat-editor-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/168-runtime-audio-sound-current-working-tree-device-streaming-mixer-spatial-automation-review.md
  - docs/plans/optimize/zircon_plugins/11-first-party-sound-source-runtime-editor-dist-catalog-mixer-spatial-reverb-timeline-product-integration-review.md
  - docs/plans/optimize/zircon_editor/17-sound-audio-clip-mixer-routing-effect-spatial-acoustic-timeline-authoring-review.md
  - docs/plans/optimize/zircon_editor/228-editor-audio-sound-current-working-tree-mixer-authoring-live-output-acoustic-review.md
  - docs/plans/zircon_plugins/02-sound.md
  - docs/plans/zircon_plugins/02/failure-2026-07-19-kira-send-frame-capture-routing.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Private/AudioMixerSourceManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Private/AudioMixerSourceManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AudioExtensions/Public/IAudioExtensionPlugin.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Sound/SoundWave.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/AudioComponent.h
  - dev/godot/servers/audio/audio_server.h
  - dev/godot/servers/audio/audio_server.cpp
  - dev/Fyrox/fyrox-sound/src/engine.rs
  - dev/Fyrox/fyrox-sound/src/context.rs
  - dev/Fyrox/fyrox-sound/src/buffer/streaming.rs
  - dev/bevy/crates/bevy_audio/src/audio_output.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/package.json
doc_type: current-working-tree-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
tooling_scope: excluded_by_user_request
---

# Runtime218 · Audio / Sound 当前工作树复审

## 1. 结论

当前 Sound 不能被判定为工程级音频引擎，更没有证据支持其性能或表现优于 Unreal。它已经不是空壳：typed contract、Kira/CPAL 静态 PCM 播放、混音图校验与局部增量编译、设备枚举、若干 DSP/空间算法、importer、测试目录和 Editor descriptor 都真实存在；但这些能力没有收敛为一个从产品启动到 audio callback、从 cooked artifact 到 voice、从 Scene/World 到可听输出、从 Editor 文档到运行时 artifact 的完整系统。

当前真实形态仍是：

1. 普通 Client/Editor Host 默认产品 feature 没有装配 base Sound runtime，`SoundModule` 即使被加载也用默认配置创建单个大状态对象。
2. Kira 是实际 callback owner；Zircon 没有显式 bounded command/observation 边界、device supervisor、World slot、generation fence 或 callback telemetry 写入协议。
3. importer 与 `SoundAsset` 全量解码 PCM，`LoadedClip` 再生成 Kira `StaticSoundData`；长音乐流式、residency、decoder pool、eviction、seek 和 voice budget 均未形成。
4. mixer graph 只支持有限 track/send 子集；effect、advanced control、pre-effect send 被 compiler 拒绝，catalog 却仍公开包含这些能力的 preset。
5. attenuation、doppler、filter、HRTF、convolution 与 DSP 算法主要停留在隔离函数/测试，没有进入真实 render block。
6. Editor、可选 ray/timeline feature、dist 与 Voice Chat 没有产品级 owner、状态、桥接或可执行闭环。
7. 当前大量测试和 ignored microbenchmark 不能回答 callback deadline、XRUN、device loss、stream miss、voice pressure、声学误差、长时内存和同负载竞争性能问题。

因此本报告不新增另一套音频债务编号。Runtime139（文件名 `99zn-...`）继续拥有 `AUD-P1-001..048`、`AUD-P2-001..012` 与 `G01..G32`；本报告只冻结 2026-09-01 当前源码、重判状态，并定义依赖有序的重构边界。

## 2. 审查边界与可复现 currentness

### 2.1 当前选择集

| 选择集 | files | lines | non-empty | bytes | test attrs | ignored | fingerprint |
|---|---:|---:|---:|---:|---:|---:|---|
| Runtime framework + Sound asset/import ingress | 34 | 3,143 | 2,844 | 101,164 | 14 | 2 | `806f16e6d7242e5b249ecdb37025a697c80ad54222c0ebfd1127c524b3abe058` |
| Sound runtime + dist + optional feature runtime/dist | 1,283 | 26,711 | 24,220 | 921,208 | 376 | 8 | `0f2305127da42e34127df87705ba103c04c45e4f53b2611caab849a97d1327a1` |
| Sound Editor + optional feature Editor | 25 | 1,572 | 1,401 | 58,874 | 10 | 0 | `c0e4d9c3d7a50815f9d3339ab2b005ff1e93bbca81c5bc27a8194f78e5604dfa` |
| Audio importer + legacy audio importer + Opus importer | 21 | 2,392 | 2,153 | 86,856 | 29 | 3 | `5f557631e22f9116b7c3366deb0a6e6ffe96d20320fd18204a607e74f7461659` |
| 去重 Zircon 联合集 | **1,363** | **33,818** | **30,618** | **1,168,102** | **429** | **13** | `23cd30ce07a07f7509c5ab69e5b3e64d469ca08d7a3e05daf544c1c5de1073e2` |

选择规则如下：

- framework 集包含 `zircon_runtime/src/core/framework/audio`、`sound` 下全部 `.rs/.toml`，以及 `SoundAsset` 与 Sound ingest 入口。
- runtime 集包含 Sound runtime、base dist、两个 optional feature 的 runtime/dist 与 `plugin.toml`，排除 Editor。
- Editor 集包含 base Sound editor 及两个 optional feature editor 的 `.rs/.toml/.zui`。
- importer 集包含 `audio_importer`、`asset_importers/audio`、`opus_importer` 下全部 `.rs/.toml`。
- fingerprint 对归一化小写相对路径排序，逐文件拼接 `path + NUL + lowercase(file SHA-256) + LF` 后再计算 SHA-256；统计读取当前工作树，而不是只读 HEAD blob。

Sound runtime 另有 **1,036** 个位于 `tests`/test-named 路径的文件；其中 **132** 个使用 `include_str!`，**186** 个含 `.contains(` 断言。这个口径不等于“1,036 个独立产品行为门”，也不能用目录数量代替覆盖质量。

### 2.2 基线与参考冻结

- Zircon 当前 HEAD 与 verification HEAD 均为 `9963f8eb72e2d725d2536eb50b393b30387a1ffa`；Sound 最近相关提交为 `5798051603e7f7f565538125c9aba96d5beabae2`。
- Bevy reference revision：`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`。
- Fyrox reference revision：`8d815db36494f1badb347547dfc7094bf4fbbdf8`。
- Godot reference revision：`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`。
- Unity Graphics reference revision：`a7e4c051d256a781ab362c64316b125a1e104694`。
- 本地 Unreal 镜像没有独立 Git 元数据，因此以列出的 owner 文件和本次工作树内容冻结。

### 2.3 证据边界

本轮是 review-only。没有修改 Rust/Cargo/ABI/UI，没有运行 Cargo、真实声卡、双设备切换、Client/Editor、import/cook、callback capture、fault、soak 或竞争 benchmark；也没有查询、轮询、等待或实时跟踪协调器。Tooling 按用户要求排除。

未运行动态门不会使静态断链变得不确定：默认 feature 未装配 Sound、factory 丢弃 plugin options、没有 World consumer、全 PCM resident、compiler 明确拒绝 effect、空间算法零生产 caller、Editor command 无 factory、feature/dist 是 stateless shell，均可由当前源码直接证明。对已有修复候选则保守保持 Partial，不以源码存在代替 fresh GREEN。

## 3. 当前真实产品链路

```text
zircon_app default target-client
  -> sound-contracts only
  -/> first-party-runtime-plugins / base-runtime-plugins
  -/> SoundModule

when SoundModule is explicitly loaded
  -> register ImmediateSoundDriver
  -> register lazy DefaultSoundManager + neutral SoundManager
  -> DefaultSoundManager::from_weak_core
  -> SoundConfig::default()                  (plugin options are not injected)
  -> Arc<Mutex<SoundEngineState>>
  -> Kira AudioManager / CPAL backend
  -> static mono/stereo playback

SoundAsset -> full Vec<f32>
  -> LoadedClip keeps Arc<SoundAsset>
  -> converts/copies into Kira StaticSoundData
  -> playback handle

Scene / World / App gameplay
  -/> AudioWorldSystem
  -/> create/update/remove source/listener/volume lifecycle
  -/> audible spatial/DSP path
```

关键证据：

- `zircon_app/Cargo.toml:26,115-145` 的普通 `target-client` 与 `target-editor-host` 只启用 Sound contracts，没有启用 base runtime Sound provider。
- `zircon_plugins/first_party_runtime_catalog/Cargo.toml:10-17` 只在 `base-runtime-plugins` 下引入 Sound；`first_party_editor_catalog` 当前只 materialize Navigation/Neural。
- `zircon_plugins/sound/runtime/src/config.rs:36-67` 能从 plugin options 构造 typed config，但 `service_types/manager_state.rs:35-37` 的生产 factory 使用 `SoundConfig::default()`。
- 仓库内没有 App/World/Scene 产品系统调用 Sound source/listener/volume/timeline 的证据。接口存在，不等于普通产品可达。
- 非 Tooling 产品代码中没有 Voice Chat/VOIP/microphone/capture-device/AEC/jitter owner。Voice Chat 不能因标题或未来 capability 名称而被标记为实现。

## 4. 自 Runtime139 以来的真实进展

下列内容是应保留的底座，但尚不足以关闭工程门：

1. `kira_bridge/graph_compile/routes.rs:7-73` 缓存展开后的 send route，并将下游 target local gain 纳入 route gain；`manager/graph.rs:164-185` 也能向活动 track/send handle 应用 volume 更新。
2. 三项精确 frame-capture 测试现在存在于 `tests/kira_bridge/graph/routing.rs:78,117,129`，使用 capture backend 覆盖 post-effect send、master gain 与 parent gain active sync。
3. graph snapshot/COW、route index、dynamic handler index、timeline lookup 与若干预分配减少了控制侧 clone/扫描。
4. audio importer 复用 `SampleBuffer<f32>` scratch，并以 4 MiB 限制 metadata 预分配；channel layout 被保留。
5. typed mixer/source/listener/volume/event/timeline contract 与显式 Unsupported 路径至少没有用 silent success 掩盖全部缺口。

但 canonical Sound failure 仍为 `status: open`。三项路由测试本轮没有 fresh 执行，且 failure 还要求 broad/product GREEN、独立 review 与原子里程碑证据；所以 `AUD-P1-018` 与 `G09` 只能从 Open/Fail 提升到 **Partial**，不能 Closed/Pass。

## 5. 工程级差距

### 5.1 产品装配、配置与实例权威

| 当前实现 | 工程差距 | 必须重构为 |
|---|---|---|
| 默认 Client/Editor Host 不装配 base Sound runtime | contracts 可编译，但普通产品没有 provider reachability | 单一 `SoundActivationPlan`，由 Client、Editor、export、NativeDynamic 共同 materialize 并产生可审计 receipt |
| `SoundModule` 工厂直接使用 `SoundConfig::default()` | manifest/plugin options 与运行实例配置断裂 | validated effective config 在任何 device/instance 创建前完成，非法值 fail-close |
| `DefaultSoundManager` 持有全局 config lock + state lock | 所有世界、设备、graph、voice、event 共用一个 owner；没有 instance/world generation | `SoundRuntimeSupervisor` 管 device/provider；每个 runtime/preview 使用 qualified `AudioWorldSlot` 与 generation |
| source/native/dist/feature 各有描述符，但没有等价生命周期 | capability 宣称与真实 provider closure 分裂 | provider generation、state save/restore、drain/unload、capability/error parity contract |
| shutdown 没有 admission-stop/drain/retire 顺序 | late callback、voice、artifact 与 device 的释放关系不可证明 | 显式 `Running -> Draining -> Quiesced -> Retired` 状态机及 terminal receipt |

必须先关闭该层。否则继续增加 DSP、节点或 Editor 面板只会扩大不可达表面。

### 5.2 实时线程、设备与观测

1. `SoundEngineState` 将 Kira、clip、source、listener、volume、graph、timeline、event、meter、IR/HRTF 等集中在一个 `Mutex` 后。Kira 的实际 callback 不获取这个 Zircon 大锁，但 Zircon 也因此没有自己的 callback-safe command/observation 协议；不能把“callback 不锁 Zircon state”误写成“Zircon 已有实时线程架构”。
2. `service_types/output_render.rs:8-19` 明确让手工 render/pull callback 返回 Unsupported。public surface 仍保留这些入口，产品无法通过一个统一状态判断 callback owner、backend availability 和 render generation。
3. completion 只在 manager API 调用时通过 `engine/state/playback.rs:46-50` 轮询。应用长时间不调用 Sound API 时，完成回收与事件可无限延迟。
4. finished playback/source 与 pending dynamic event 使用可增长 `Vec`；只有 gameplay journal 是按 World 有界 `VecDeque`。没有 callback-to-control observation ring、overflow disposition 或 backpressure。
5. device 仅接受 mono/stereo，stable ID 实际是 `kira-cpal:{display name}`；重名设备、名称变化、default-device change 都不能稳定关联。
6. device negotiation 不先查询 supported configs，而是直接请求精确 sample rate/channel/fixed block。`configure_output_device` 先停当前 Kira 再尝试新配置，没有 prepare/open/warm/crossfade/commit、last-known-good 或 rollback。
7. `underrun_count` 等 telemetry 字段只有初始化/重置/投影，没有 callback writer。latency 由 `block_size * latency_blocks` 推算，不是 backend timestamp/queue depth 的实际测量。

目标边界必须是 bounded control command ring、render-owned immutable generation、bounded observation ring、独立 device supervisor 和 telemetry snapshot。任何 callback 路径都不得等待普通 mutex、分配、文件 I/O、decoder I/O 或执行 foreign callback。

### 5.3 资产、cook、流式、residency 与 voice

1. `zircon_runtime/src/asset/assets/sound.rs:122-128` 将完整音频保存为 `Vec<f32>`。`engine/state/playback.rs:13-25` 的 `LoadedClip` 同时持有 `Arc<SoundAsset>` 与 Kira `StaticSoundData`；mono 还会在 `kira_bridge/playback_data.rs:39-64` 复制为 stereo，超过两声道直接拒绝。
2. `clip_assets.rs:39-73` 同步调用 `ProjectAssetManager::load`，加载后再在锁内 double-check。没有 single-flight loading state、failure cache、cancel、unload、refcount、LRU、resident byte budget 或 decode CPU budget。
3. importer 先把整个源复制到 `Cursor`，再将所有 packet 解码进持续增长的 `Vec`。坏 packet 被跳过后继续，没有错误数量、总帧/字节/时长、CPU deadline 或取消预算。
4. 4 MiB metadata reserve cap 只是局部防御，不是 clip/stream/resident 项目预算。三个 importer owner 仍重叠；Opus importer 主要是依赖 NativeDynamic libopus 的诊断入口，不是平台 cooked streaming backend。
5. `max_voices` 只是超限错误。没有 priority、concurrency group、audibility、cost、steal、virtualize、resume 或原因 observation。
6. External/Synth 类型可由 public contract 表达，但 active sync 在 `service_types/sources.rs:209-212,284-288` 明确 Unsupported；admission 没在创建前按 capability fail-close。
7. source create 在证明 Kira voice 实际启动/可听前即可发布 gameplay event；普通 source update 会移除 voice、停止 playback、再 sync/restart。

需要独立 `AudioClipArtifact`：source/settings/compiler/target/dependency/content hash 完整，短音频 resident page 与长音频 stream chunk 分离，decoder pool 有并发/时间/字节预算，seek/remix/layout 是 cook/runtime 明确合同。`VoiceAllocator` 必须是唯一 admission authority，并区分 intended、started、audible、virtualized、stolen、completed。

### 5.4 Mixer graph、DSP 与 preset truth

1. graph compiler 的校验、snapshot、diff、route cache 是真实实现；但 `graph_compile.rs:550-575` 仍拒绝所有 effect、advanced control 与 pre-effect send。
2. `music_sfx` preset 包含 limiter/gain effects，`spatial_room` 包含 reverb effect 和 sends，catalog 仍公开它们。`tests/kira_graph_sync.rs:243-251` 在 manager inactive 时应用 `spatial_room` 并断言 compiler invocation 为 0；这不能证明 preset 可执行。
3. active playback 时 structural graph edit 在 `manager/graph.rs:59-63` 直接 Unsupported。没有 immutable graph generation、block-boundary swap、旧 graph tail drain 与 callback fence retirement。
4. compressor、limiter、gain、wet、delay、shaper、meter 的调用主要存在于 `engine/dsp/tests.rs`；filter/source environment 同样没有进入 Kira 真实 block。
5. graph validator、compiler、preset catalog、Editor palette 没有共享同一个 executable capability matrix，导致“可声明、可序列化、不可执行”。

重构不能继续给 validator 添加 DTO 后就算完成。需要 `MixerGraphDocument -> validated IR -> target capability lowering -> immutable RenderGraphGeneration -> block swap receipt`，并且 preset 只有在同一 compiler/backend 上通过时才能出现在产品 catalog。

### 5.5 Scene/World、空间声学与可听链路

1. 没有 Runtime `AudioWorldSystem` 消费 scene delta、transform、pause/time scale 与 teardown。source/listener/volume API 没有普通产品 caller，也没有 entity/world generation owner。
2. `engine/source_environment/apply.rs:16` 的 attenuation、cone、doppler、volume、lowpass、HRTF、convolution 组合只有定义，没有生产调用；`hrtf_tail_pending_for_source` 同样没有 consumer。
3. HRTF 当前是简化 gain+delay preview；profile load 路径没有接入 audible render source。没有方向插值、tail、multi-listener、profile transition 或 degraded policy。
4. convolution 每次复制 block，并进行 frame × tap 嵌套循环；没有跨 block tail、partitioned convolution、IR crossfade、latency contract、CPU budget 或 fallback。
5. ray provider 只接收外部提交的完整 IR sample/descriptor/status；没有 scene geometry provider、ray scheduler、generation cache、source binding、query budget 或 stale rejection。
6. optional ray/timeline runtime feature 的 `register()` 只登记空 `ModuleDescriptor`；dist 标记 stateless、state schema 0，没有 command/event/invoke/save/restore/unload/bridge/on-host-ready。

需要 `AudioWorldSystem` 先把 scene truth 编译成 per-source render parameters，再由 `AcousticSceneProvider` 发布 versioned geometry/IR field，由 render generation 在预算内消费。空间算法的完成条件必须是改变真实输出 block，并由 offline numerical oracle 与实时 budget 同时验证。

### 5.6 Timeline、automation 与动态事件

1. active Kira automation 在 `automation/target/apply.rs:108-114` 明确 Unsupported；优化 clone 没有改变功能门。
2. timeline 由 caller 传入 `delta_seconds`，`mem::take` 整个事件 `Vec`，逐 sequence 创建 report/sample/application；没有生产 pump、audio sample clock、epoch、seek/scrub/pause/loop 或 missed-event policy。
3. dynamic dispatch 接收可增长 `Vec`，一次 drain all，为 delivery reserve capacity，并为每个 handler clone invocation。没有 items/bytes/time budget、公平性、priority、drop reason 或唯一 pump。
4. external/ABI callback 虽在锁外同步执行，但没有 deadline、cancel、provider generation、in-flight lease 或 unload quiescence。锁外不等于安全卸载，也不等于实时安全。

自动化与时间轴必须编译成 sample-clock command stream，而不是每帧解释 DTO；动态事件必须与 callback observation 明确隔离，并具备 admission、budget、terminal disposition 和 generation。

### 5.7 Editor、dist、optional feature 与 Voice Chat

1. Sound Editor 当前有 **33** 个 command path：20 个 mixer/output/debug、7 个 source、3 个 listener、3 个 volume。仓库只找到 descriptor，没有对应 Sound operation factory；例如 `sound.mixer.track.create` 只出现在 i18n/descriptor。
2. 五份 ZUI 共 43 个节点，其中 29 个是 `Space`，只有 Refresh/Start/Stop 三个 Button route。acoustic/source/listener/volume 面板没有 typed input、provider、document observation 或 terminal state。
3. `SoundEditorLiveOutputController` 能 snapshot/enumerate/configure/start/stop，但生产代码没有构造它；只有 fake test 使用，且部分 render/backend 方法仍是 `unimplemented!`。
4. first-party Editor catalog 没有 Sound；ray/timeline feature editor 只有 descriptor/capability，没有 UI、provider、artifact 或 runtime bridge。
5. base Sound dist 和两个 feature dist 都是 stateless metadata shell：command/event manifest 为空，没有 invoke/save/restore/unload/bridge。
6. Editor228 的六项产品门仍全部 Fail。Runtime218 只映射其结果，不把 Editor 债务重复写进 Runtime canonical 编号。
7. Voice Chat 需要 capture device、permission、AEC/NS/AGC、encoder、jitter buffer、transport/session identity、packet loss concealment、mix-minus、mute/privacy、telemetry 与 Editor/Runtime policy。当前非 Tooling 产品代码没有这些 owner，所以状态是未实现，不是“Sound 框架可扩展后自然拥有”。

### 5.8 测试、故障与性能证据

1. Sound runtime 的 8 个 ignored 测试均是本地 release-only microbenchmark，分布于 automation、dynamic events、volume filter、graph compile/validation、mixer timeline、service timeline、timeline advance。importer 另有 3 个 ignored performance test。
2. 这些测试比较局部旧/新算法或 raw sample 处理，不测真实 callback、设备、end-to-end graph、audio correctness、product startup 或 Editor audition。
3. 选择集及其邻接范围没有 Loom/proptest/criterion/fuzz/sanitizer/soak/hotplug/XRUN/dropout 资格证据。
4. 没有真实硬件 matrix、device loss/recovery、OOM、stream starvation、decoder failure、voice pressure、长时内存、callback deadline distribution 或跨 backend golden output。
5. 没有与 Unreal 在相同设备、sample rate、block size、channel layout、voice/effect/spatial workload 下的 correctness-first benchmark。因此“优于 Unreal”目前是目标，不是结论。

## 6. Canonical 台账重判

### 6.1 唯一债务状态

| Canonical owner | 当前重判 | 说明 |
|---|---|---|
| Runtime139 `AUD-P1-001..048` | **42 Open / 6 Partial / 0 Closed** | Partial 为 `AUD-P1-018,020,026,033,035,036`；018 因路由源码候选与测试存在新增 Partial，其余五项状态不变 |
| Runtime139 `AUD-P2-001..012` | **12 Open / 0 Partial / 0 Closed** | 没有新的工程关闭证据 |
| Runtime139 `G01..G32` | **27 Fail / 5 Partial / 0 Pass** | Partial 为 `G09,G11,G15,G19,G30`；G09 从 Fail 提升为 Partial |
| Runtime08B P1 | **12 Open / 8 Partial / 0 Closed** | 路由已包含在原 Partial 聚合项中，数量不变 |
| Plugins11 / NSND P1 | **42 Open / 6 Partial / 0 Closed** | `NSND-P1-018` 同步映射为 Partial，不新增编号 |
| Plugins11 / NSND P2 | **12 Open** | 不变 |
| Editor17 | **P0 5 Open；P1 60 Open；P2 12 Open** | Editor228 六门全 Fail，未发现可关闭证据 |

六个 Partial 的准确含义：

- `AUD-P1-018`：expanded route/gain 候选与三项 frame-capture test 已存在；fresh current-source/broad/product 资格未执行，failure 仍 Open。
- `AUD-P1-020`：graph snapshot/COW/锁外 compile 减少部分控制锁持有；仍无显式 render command/observation 架构。
- `AUD-P1-026`：importer 保留 layout、复用 scratch；全 PCM、双重 resident、mono/stereo 限制与无 streaming 未关闭。
- `AUD-P1-033`：handler 预排序/索引与 reserve 有进展；pending unbounded/drain-all/无 budget 未关闭。
- `AUD-P1-035`：automation 减少 clone；active Kira 仍 Unsupported。
- `AUD-P1-036`：timeline 有预分配和 lookup 优化；仍依赖 caller delta、无 sample clock 和产品 pump。

### 6.2 32 门当前状态

| 分组 | Fail | Partial | Pass |
|---|---|---|---|
| G01-G08 产品/实例/实时/设备/能力矩阵 | G01-G08 | - | - |
| G09-G11 routing/graph/preset | G10 | G09、G11 | - |
| G12-G18 World/spatial/source/artifact/voice | G12-G14、G16-G18 | G15 | - |
| G19-G25 event/automation/acoustics/artifact identity | G20-G25 | G19 | - |
| G26-G32 Editor/evidence/reliability/benchmark | G26-G29、G31-G32 | G30 | - |
| **合计** | **27** | **5** | **0** |

`G09` 的 pass 条件保持不变：三项 direct/send/master/parent-gain frame capture 必须在 fresh current source 全部通过，并补齐 broad/product、独立 review 与 failure closeout。源码候选不改变门定义。

## 7. 参考引擎差异

### 7.1 Unreal：主要工程基线

- `AudioMixerSourceManager` 在每个 render block 明确泵 MPSC command、处理 deferred release、scheduled render、source/bus、plugin stage 与 render cost；还使用双 command buffer 和 pending release fence。Zircon 当前没有等价的 audio-thread protocol 与分阶段 budget。
- `IAudioExtensionPlugin` 定义 factory/instance 生命周期及 `Initialize`、`OnInitSource`、`ProcessAudio`、`OnAllSourcesProcessed`、`OnReleaseSource`、shutdown。Zircon optional feature 目前只是 descriptor，不具备 source lifetime 或 render-stage 插件合同。
- `SoundWave` 管理 streamed chunks、seek offset、cooked/derived data、异步加载与 compressed-data retain/release。Zircon 的 `SoundAsset` 是 full PCM，不能作为长音频工程基线。
- `AudioComponent` 将 attenuation、concurrency、occlusion、virtualization、submix/bus send 与 SceneComponent 生命周期绑定。Zircon 没有 AudioWorldSystem/scene component consumer。

### 7.2 Godot、Fyrox、Bevy

- Godot `AudioServer` 有明确 playback 状态、audio driver mix 回调、bus/effect/meter 路径，并把非实时安全的释放推回主线程。它证明了 server/bus/playback 生命周期闭合。
- Fyrox `SoundEngine` 的真实 output callback 调用 context render，source 经 bus、default/HRTF renderer 与 effect graph 到输出；`StreamingBuffer` 具有 decoder、seek/rewind。其 callback 内锁整个 state 不是 Zircon 应复制的实时性能上限，但其纵向可达性明显完整于当前 Zircon。
- Bevy `AudioPlayer` 由 ECS system 在 asset 就绪后创建 sink，更新 emitter/listener transform，并在完成时 cleanup/despawn。它不是 AAA 音频性能基线，但证明 ECS/World 生命周期不能由裸 manager API 替代。
- 本地 Unity `dev/Graphics` 是渲染包，不拥有 Audio。此范围标记为 N/A，不用渲染概念虚构音频完成度。

## 8. 目标架构

```text
Product / Export / Editor / NativeDynamic
  -> SoundActivationPlan + validated EffectiveSoundConfig
  -> SoundRuntimeSupervisor
       |- DeviceSupervisor (stable device identity, negotiate, LKG, recovery)
       |- ProviderRegistry (generation, lease, drain/unload)
       `- AudioWorldSlot[qualified world/preview generation]
            |- AudioWorldSystem (scene delta -> source/listener/volume truth)
            |- VoiceAllocator (priority, concurrency, virtualize/steal)
            |- MixerCompiler (document -> validated IR -> render generation)
            |- SampleClockTimelineCompiler
            `- bounded ControlCommandRing
                    -> Audio Render Instance / Kira backend
                    -> immutable graph + voice + spatial/plugin stages
                    -> bounded ObservationRing

Asset source
  -> deterministic AudioClipArtifact cook
  -> resident pages / stream chunks / decoder pool / residency authority
  -> voice data source

Editor document + transaction
  -> same compiler/artifact/provider generation
  -> isolated PreviewWorldSlot
  -> typed observation / diagnostics / terminal receipt
```

所有 public ID 必须至少包含 instance/world owner 与 generation；device、graph、voice、artifact、provider、timeline 不能继续只用普通 `u64` 或名称字符串。当前 Kira 可以作为后端继续使用，但后端不得拥有 Zircon 产品配置、World 生命周期、artifact identity 或 Editor 真相。

## 9. 依赖有序重构里程碑

| 顺序 | 里程碑 | 必须交付 | 禁止提前声称完成 |
|---:|---|---|---|
| A0 | 产品真相与 fail-close | `SoundActivationPlan`、effective config、provider/catalog closure、source/native capability parity、ordinary product reachability test | 仅 module unit test 或 manifest descriptor |
| A1 | 实例/World owner | `SoundRuntimeSupervisor`、qualified `AudioWorldSlot`、lifecycle/generation/shutdown、Scene delta consumer | 单个全局 manager 手工调用 demo |
| A2 | 实时与设备 | bounded command/observation、device negotiation、two-phase switch/LKG/recovery、callback telemetry、deadline policy | 只记录估算 latency 或后台字段 |
| A3 | artifact/stream/residency/voice | deterministic cook identity、resident/stream 分层、decoder pool、single-flight/cancel/evict、VoiceAllocator | full PCM preload 或仅提升 max capacity |
| A4 | mixer/effect/sample clock | executable matrix、immutable graph generation、block swap/tail drain、effects、automation/timeline compile | DTO 可序列化、inactive graph test |
| A5 | spatial/acoustics | AudioWorldSystem、真实 render parameter、HRTF、partitioned convolution、acoustic provider generation/budget | 隔离函数数值测试或手工提交整段 IR |
| A6 | Editor/dist/feature/Voice Chat | Sound catalog provider、document/transaction/preview、33 command factory、truthful ZUI、feature state/bridge；Voice Chat 独立安全里程碑 | descriptor、Space、stateless shell 或借用 playback API |
| A7 | 资格与竞争证据 | offline golden、device/hotplug/fault/OOM/stream/voice/soak matrix、callback percentile、内存与同负载 Unreal 对照 | ignored microbenchmark 或不同功能负载比较 |

执行原则：A0/A1 未关闭前，不接受新增高级 DSP/声学功能作为“引擎进展”；A2/A3 未关闭前，不接受以短音频 demo 推导实时/长音频能力；A4/A5 未关闭前，Editor 不得公开无法执行的 preset/effect；A7 必须 correctness-first，不能为了“优于 Unreal”选择不等价负载。

## 10. 分层验收门

1. **产品门**：普通 Client、Editor Host、generated export、NativeDynamic 生成同一 activation digest；非法 config 在 instance/device 创建前拒绝；shutdown 后无 late callback/handle。
2. **实时门**：callback 路径无普通 mutex wait、动态分配、I/O、decoder block、foreign callback；command/observation overflow 有确定 disposition；记录 p50/p95/p99.9/max deadline 与 XRUN。
3. **设备门**：重名设备、default change、hotplug、format mismatch、stall/loss、LKG rollback 全部有状态机与机器可读 receipt。
4. **资源门**：短/长 clip、seek、loop、layout/remix、stream starvation、decoder error、cancel、evict、reload 在明确 bytes/time/concurrency budget 下通过。
5. **声音门**：direct/send/master、effect order、pre/post send、automation、source update、voice steal/virtualize、spatial/HRTF/convolution 用真实 block 和 offline oracle 验证。
6. **World 门**：多 World/preview 同时存在，entity generation、transform、pause、unload 与实例 teardown 不串音、不泄漏、不复活 stale voice。
7. **Editor 门**：公开 command 全部具备 schema decoder、factory、transaction、undo/redo、terminal receipt；ZUI 消费 document/observation，preview 与 runtime 使用同 artifact/compiler/provider generation。
8. **竞争门**：在相同设备、backend、sample rate、block、channel、voice、effect、spatial、stream 与输出正确性下比较 CPU、deadline、latency、memory、recovery；否则不得写“优于 Unreal”。

## 11. 本轮不关闭的事项

- 不关闭 `docs/plans/zircon_plugins/02/failure-2026-07-19-kira-send-frame-capture-routing.md`。
- 不把 routing source candidate 或 test 文件存在写成 fresh GREEN。
- 不把 Kira 拥有 callback 写成 Zircon 已拥有 realtime architecture。
- 不把 importer scratch/preallocation 优化写成 streaming/residency 完成。
- 不把 isolated DSP/spatial algorithm 写成 audible product feature。
- 不把 Editor descriptor、command path、ZUI `Space` 或 feature metadata 写成 Editor 产品。
- 不为 Voice Chat 创建空接口或占位 capability；其 capture/privacy/network/DSP 生命周期必须作为独立完整系统设计。
- 不运行或修改 Tooling，不查询协调器状态；后续实现从 A0 开始，逐门提供 current-source 证据。
