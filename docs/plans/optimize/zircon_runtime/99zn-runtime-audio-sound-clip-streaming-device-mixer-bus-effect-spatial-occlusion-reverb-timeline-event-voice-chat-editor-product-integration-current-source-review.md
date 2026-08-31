---
title: Runtime Audio / Sound / Clip / Streaming / Device / Mixer / Bus / Effect / Spatial / Occlusion / Reverb / Timeline / Event / Voice Chat / Editor 当前源码复审
category: zircon_runtime
report_id: Runtime139
review_date: 2026-08-24
baseline_head: ed543173cbd825fe3b7e1f6c81d52c9ca3391095
baseline_epoch: 422
verification_head: ed543173cbd825fe3b7e1f6c81d52c9ca3391095
verification_epoch: 422
supersedes_currentness_of:
  - docs/plans/optimize/zircon_runtime/08b-audio-runtime-review.md
  - docs/plans/optimize/zircon_plugins/11-first-party-sound-source-runtime-editor-dist-catalog-mixer-spatial-reverb-timeline-product-integration-review.md
  - docs/plans/optimize/zircon_editor/17-sound-audio-clip-mixer-routing-effect-spatial-acoustic-timeline-authoring-review.md
related_code:
  - zircon_runtime/src/core/framework/audio
  - zircon_runtime/src/core/framework/sound
  - zircon_runtime/src/asset/assets/sound.rs
  - zircon_runtime/src/asset/importer/ingest/import_sound.rs
  - zircon_plugins/sound
  - zircon_plugins/audio_importer
  - zircon_plugins/asset_importers/audio
  - zircon_plugins/opus_importer
  - zircon_plugins/first_party_runtime_catalog
  - zircon_plugins/first_party_editor_catalog
  - zircon_app/src/entry
plan_sources:
  - docs/plans/optimize/zircon_runtime/08b-audio-runtime-review.md
  - docs/plans/optimize/zircon_plugins/11-first-party-sound-source-runtime-editor-dist-catalog-mixer-spatial-reverb-timeline-product-integration-review.md
  - docs/plans/optimize/zircon_editor/17-sound-audio-clip-mixer-routing-effect-spatial-acoustic-timeline-authoring-review.md
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
  - dev/UnrealEngine/Engine/Source/Editor/AudioEditor/Private/AudioEditorModule.cpp
  - dev/UnrealEngine/Engine/Source/Editor/AudioEditor/Private/SoundSubmixEditor.cpp
  - dev/UnrealEngine/Engine/Source/Editor/AudioEditor/Private/AssetTypeActions/AssetDefinition_SoundWave.cpp
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
---

# Runtime139 · Audio / Sound 当前源码复审

## 1. 结论

当前Audio不是工程级声音系统，而是“宽合同、有限Kira静态播放、不可达空间/DSP原型、手工推进的事件与时间线、断开的产品装配和Editor声明壳”的组合。它已有可保留基础：Runtime提供中立的channel layout、source/listener/volume、mixer/effect、playback、device、automation、timeline和acoustic合同；Sound插件是Kira 0.12.2的唯一owner，能通过CPAL启动设备、播放mono/stereo静态PCM、建立部分track/send图并做局部参数同步；importer已能保留常见多声道布局，Symphonia包解码也加入scratch复用和有界预分配；动态事件已有预排序索引，graph/timeline/automation的一些瞬时复制和分配已减少。

这些进展没有形成普通Zircon产品中的可听、可编辑、可资格化系统。默认`target-client`只启用`sound-contracts`，不启用`first-party-runtime-plugins`；`target-editor-host`也不链接base Sound runtime或Sound Editor。first-party runtime catalog只有在`base-runtime-plugins` feature下才可返回Sound root，first-party editor catalog只包含Navigation和Neural，两个Sound optional feature也没有进入普通产品provider closure。Sound module factory仍使用`SoundConfig::default()`，manifest options没有成为运行时effective config；全仓生产调用检索找不到App/World对`start_output_device`、`create_source`、`update_listener`或`update_volume`的调用。

即使显式取得manager，执行链仍只闭合静态PCM。`SoundAsset`和Symphonia importer会把完整音频解码到`Vec<f32>`，`LoadedClip`再常驻一份Kira `StaticSoundData`；播放阶段把mono复制为stereo并拒绝超过双声道。没有stream index、decoder pool、residency、eviction、single-flight或unload。一个`Mutex<SoundEngineState>`仍同时拥有device、Kira handle、clip、voice、graph、listener、volume、timeline、dynamic event和telemetry；Kira拥有真正callback，但Zircon没有audio-thread command/observation contract。结构性mixer变更在活动播放存在时直接拒绝，effect、advanced control和pre-effect send仍由M1 compiler拒绝，内建`music_sfx`和`spatial_room`却继续暴露这些不可执行内容。

空间与声学代码的“存在”尤其容易制造完成假象。attenuation、cone、doppler、volume、occlusion、HRTF和convolution函数没有生产render调用者；`engine/dsp`和`engine/filter`仍只在`cfg(test)`下编译。source/listener/volume更新主要写manager map，`update_source`会stop/restart voice；completion只在其他manager调用时轮询Kira handle。automation在Kira active时明确返回M5 unsupported，timeline依赖caller传入`delta_seconds`逐track解释，dynamic event仍使用无界`Vec`并同步执行外部callback。设备ID由display name拼接，只有mono/stereo，configure先停设备，缺hotplug/default-change/recovery/LKG和真实callback telemetry。

Editor与distribution同样没有产品闭环。Sound Editor登记33个command descriptor，但全仓没有这些路径的operation factory；五份ZUI共有29个`Space`占位，仅Mixer有Refresh/Start/Stop三个Button。live-output controller只有公开构造器和fake测试，没有产品owner。Ray-Traced Convolution Reverb与Timeline Animation Track的runtime/editor只发布descriptor/capability，dist为stateless metadata entry，没有provider行为、state、command、event、unload或bridge。唯一Sound failure handoff仍是Open：Kira send frame-capture四项门中历史证据为1 pass / 3 fail。

因此当前没有证据支持“性能和表现优于当前Unreal”。局部ignored release benchmark只比较旧/新容器分配和查找，不证明callback deadline、XRUN、stream miss、voice pressure、设备恢复、空间误差、长时稳定性或同功能竞争表现。本报告只刷新当前事实、目标架构和资格门，不修改生产代码。

旧Runtime08B的20项P1重判为 **12 Open、8 Partial、0 Closed**，5项P2全部Open；旧Plugins11的48项P1重判为 **43 Open、5 Partial、0 Closed**，12项P2全部Open；Editor17的5项P0、60项P1和12项P2全部Open。唯一相关failure handoff仍为Open。

## 2. 审查边界、方法与currentness

### 2.1 冻结Audio范围

统计口径为当前工作树物理行、非空行、文件bytes、Rust `#[test]`和`#[ignore`声明。fingerprint按normalized lowercase path排序，对每个文件拼接`path + NUL + lowercase(file SHA-256) + LF`后再取SHA-256。产品consumer集合只包含限定App/catalog/AI/Animation/builtin目录中实际含sound/audio的Rust/TOML/ZUI文件，不代表这些crate的总规模。

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | fingerprint |
|---|---:|---|
| Runtime Audio合同、Sound asset/import和resource loading纵切面 | **42 / 4,524 / 4,129 / 155,530 / 11 / 1** | `030f45a4e1abfee76aaa761f141622f98cf127852d3e0d41d7bb2fa389d5b0bd` |
| Sound插件source/editor/runtime/dist/features/tests全量 | **1,307 / 27,985 / 25,355 / 970,262 / 383 / 7** | `ef6acecccb900516f1f3ee6abb6cccce4433aa30299f25f591b39d9a9337eea3` |
| Audio Importer、legacy audio family和Opus importer全量 | **21 / 2,392 / 2,153 / 86,856 / 29 / 3** | `5f557631e22f9116b7c3366deb0a6e6ffe96d20320fd18204a607e74f7461659` |
| App/catalog/AI/Animation/builtin含Sound consumer | **29 / 5,419 / 5,053 / 196,542 / 68 / 4** | `a9300bc65cf43b0dc9f2d4a4fc8635bb094d1db4550aa764c0df8f4c512a1665` |
| Zircon selected union | **1,399 / 40,320 / 36,690 / 1,409,190 / 491 / 15** | `b4f700c269922e459fed15c2caf8f120cc8de01823cb0ae669f18febf2ca35cd` |
| 五引擎参考选择集 | **30 / 33,809 / 28,257 / 1,232,600 / 4 / 0** | `7ab056bbe6fdf8db5529d0f3c2bdfc8131176c59fcdf733b3d0d2899459fc862` |

Sound runtime生产部分为229文件、12,611行、21项test和7项ignored；专用测试路径为1,035文件、12,464行和344项test。测试目录中1,032个子文件里，823个文件名属于manifest相关分组，132个文件使用`include_str!`，185个文件包含源码形状式`.contains(...)`检查。该规模说明结构约束很密，不等于产品行为资格。

参考revision：Bevy `fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`、Fyrox `8d815db36494f1badb347547dfc7094bf4fbbdf8`、Godot `8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、Unity Graphics `a7e4c051d256a781ab362c64316b125a1e104694`。`dev/UnrealEngine`没有独立Git元数据，由11个选择文件及参考集合fingerprint冻结。

### 2.2 检查方法

1. 逐文件读取Runtime audio/sound合同、Sound asset/WAV解析、artifact/load路径，以及Sound插件production 229文件、测试树、Editor、dist和两个optional feature。
2. 沿`App target/profile -> first-party catalog -> package/provider -> Module factory -> SoundManager -> output/device -> Kira graph/playback -> completion/telemetry`追踪普通产品链，不从类型、capability或测试名推断可达性。
3. 沿`source bytes -> importer selection -> full decode -> SoundAsset -> resource lease -> LoadedClip -> StaticSoundData -> Kira voice`核对资源生命周期；沿`Scene component -> World system -> source/listener/volume -> render parameter`核对空间链。
4. 沿`route -> command descriptor -> operation factory -> document/transaction -> ZUI controller -> runtime observation -> save/cook/preview`核对Editor闭环；核对source、NativeDynamic和optional feature是否行为等价。
5. 对Runtime08B、Plugins11、Editor17和Sound failure逐项重判。减少临时分配但未改变产品失败条件的条目最多Partial，不允许Closed。
6. 参考Unreal的AudioMixer线程/device/submix/voice/SoundWave streaming/AudioExtensions/AudioEditor，Godot的AudioServer/3D player/bus editor，Fyrox的SoundContext/streaming/HRTF/bus/effect，Bevy的ECS schedule/queued asset/sink cleanup；Unity Graphics只用于确认本地镜像不是Audio owner。

### 2.3 动态证据边界

- Session基线、冻结HEAD为`ed543173cbd825fe3b7e1f6c81d52c9ca3391095` / epoch 422。
- Audio相关文件含其他Session/用户的working-tree修改；本报告读取当前内容，不覆盖、不回退，也不把未提交的ignored性能测试写成已集成资格。
- 本轮为review-only，没有运行Cargo、真实声卡、Client/Editor、PIE、NativeDynamic、import/cook/reload、hotplug、callback capture、fault/scale/soak/profile或竞争benchmark。
- 静态调用图足以证明的零产品caller、零Editor provider、无World system、full PCM、M1 compiler拒绝、active automation拒绝、无界event和Space占位不因未跑Cargo而改变；动态门保守判定。
- Tooling按用户要求排除，未来迁移Rust时单独审查。

## 3. 当前真实产品链路

```text
ordinary zircon_app Client
  target-client -> sound-contracts
  x no first-party-runtime-plugins -> no Sound provider

ordinary zircon_app Editor Host
  target-editor-host -> advanced render/navigation/neural providers
  x no base Sound runtime -> no SoundManager/output
  x first-party editor catalog has Navigation/Neural only -> no Sound Editor

explicit base runtime catalog
  RuntimePluginId::Sound -> Sound registration
  Sound module factory -> DefaultSoundManager::from_weak_core
  x always SoundConfig::default(), manifest options not consumed
  x no RuntimeSceneSystemRegistration / AudioWorldSystem

explicit manager path
  load_clip -> full resident SoundAsset + resident Kira StaticSoundData
  start_output_device -> Kira/CPAL mono or stereo
  play/create source -> static clip voice
  x External/Synth adapter
  x streaming/residency/voice allocator
  x spatial/HRTF/occlusion/convolution production render call
  x callback observation/completion queue

Sound Editor / optional features / dist
  descriptors + capabilities + 33 command paths + 29 Space placeholders
  x product factory/controller/document/transaction/preview
  x ray/timeline executable provider
  x NativeDynamic behavior parity
```

目标不是继续扩大`DefaultSoundManager`。目标是App只提交project/profile选择，Runtime创建generation-bound `SoundRuntimeInstance`，device supervisor和audio render service拥有callback与sample clock，World system拥有component投影，resource service拥有stream/residency，mixer compiler拥有不可变执行图，Editor和optional provider通过typed lease接入同一实例。

## 4. 必须保留的基础

1. 保留Runtime-owned neutral Audio/Sound合同，但为mutation、playback、device、graph、timeline和event补generation、clock domain、admission/apply receipt与thread domain。
2. 保留Kira 0.12.2作为Sound plugin唯一执行后端owner；不得在Runtime、App、Editor或feature crate创建第二Kira manager。
3. 保留当前channel layout校验、WAV extensible mask和Symphonia speaker projection，继续收敛为artifact format contract而非仅内存DTO。
4. 保留Symphonia scratch复用、有界预分配、locator缓存、graph snapshot/COW和timeline/event索引优化；这些只是局部foundation，必须进入统一budget和动态资格。
5. 保留Kira graph validator、incremental diff、prepare/apply/rollback意图；升级为immutable graph generation和callback fence，删除active-playback结构变更禁令。
6. 保留typed source/listener/volume、effect、automation、timeline、dynamic event和acoustic词汇；未进入真实render的类型必须fail-close，不能支撑capability。
7. 保留Editor contribution ID和live-output model边界；接入真实document/operation/runtime observation后再公开surface。
8. 保留beta/partial maturity；G01-G32通过前不得升级为shipping-ready，也不得用ignored microbenchmark宣称优于Unreal。

## 5. 当前最高风险差异

### 5.1 Composition、capability、config、lifecycle与Editor reachability

| ID | 状态 | 当前证据 | 工程级目标 |
|---|---|---|---|
| AUD-P1-001 | Open | 默认Client只有`sound-contracts`，不链接first-party Sound provider | project/profile产生selected provider closure和activation receipt；required缺失时fail-close |
| AUD-P1-002 | Open | Editor Host同样不链接base Sound runtime，preview与export可落在不同provider集 | Editor Host与目标产品消费同一SoundActivationPlan，只允许显式target policy差异 |
| AUD-P1-003 | Open | runtime catalog仅在base feature下收集Sound root，不收集ray/timeline provider | root-feature provider graph记录requested/linked/admitted/activated/degraded及generation |
| AUD-P1-004 | Open | first-party editor catalog无Sound及两个feature editor | 同一package selection解析Editor closure；缺provider时隐藏能力并给typed原因 |
| AUD-P1-005 | Open | generated/native显式路径强于ordinary host，composition并非单一事实源 | source/library/generated/native统一消费ProviderResolver和activation receipt |
| AUD-P1-006 | Open | module factory调用`SoundConfig::default()`；manifest options只注册为metadata | validated EffectiveSoundConfig绑定source/override/target/generation并进入实例创建 |
| AUD-P1-007 | Open | base dist是ABI v3 stateless metadata shell，source行为没有native projection | 实现完整provider bridge/lifecycle/quiescence，或明确NativeDynamic Unsupported |
| AUD-P1-008 | Open | dist command/event/state/save/restore/unload/host-ready均为空 | source可观察行为必须在ABI中等价表达；无法表达的能力不得Loaded/Enabled |
| AUD-P1-009 | Open | ray与timeline runtime注册空ModuleDescriptor却发布beta capability | admission要求可执行provider、依赖lease和同代receipt；否则Unavailable |
| AUD-P1-010 | Open | 两个feature editor只有descriptor/capability，无document/controller/preview | 只有真实Editor extension可注册可见能力；删除空壳contribution |
| AUD-P1-011 | Open | Sound Editor 33个command descriptor无operation factory/handler | 每项绑定typed payload、permission、transaction/job、cancel/deadline和terminal receipt |
| AUD-P1-012 | Open | 五份ZUI有29个`Space`，除三个output按钮外无业务控件 | 使用真实view model/controller替换占位；无后端区域隐藏或typed unavailable |
| AUD-P1-013 | Open | 无Audio Clip toolkit、waveform/import settings、Mixer document、undo/redo、audition | 建立open-edit-undo-save-reopen-compile-preview闭环并复用运行时artifact |
| AUD-P1-014 | Open | live-output controller无产品factory/owner，只有构造器和fake测试 | Editor runtime bridge创建generation-bound controller并处理stale/disconnect/reconnect |
| AUD-P1-015 | Open | catalog公开default/music_sfx/spatial_room，后两者含compiler拒绝的effect | preset可见性绑定executable qualification；不可执行项在catalog admission拒绝 |
| AUD-P1-016 | Open | capability无法区分declared/linked/degraded/executable/running | 建立CapabilityTruthReceipt，UI/profile/export/telemetry只消费同一receipt |

### 5.2 Audio thread、device、mixer、World与resource闭环

| ID | 状态 | 当前证据 | 工程级目标 |
|---|---|---|---|
| AUD-P1-017 | Open | graph validator接受effect/advanced control/pre-effect send，M1 compiler随后拒绝 | validator/compiler/preset/Editor palette由单一executable capability matrix生成 |
| AUD-P1-018 | Open | canonical failure仍记录三项send/master/parent-gain frame-capture失败 | 重建direct/send/master拓扑和gain law，以真实frame capture关闭红门 |
| AUD-P1-019 | Open | 活动playback存在时结构图修改直接Unsupported | 编译immutable graph generation，在block边界原子切换并按callback fence退役 |
| AUD-P1-020 | Partial | graph snapshot、Arc COW及锁外compile减少部分锁持有；全局state mutex仍串行所有服务 | 分离control queue、render-owned state与lock-free/bounded observation，callback禁普通锁 |
| AUD-P1-021 | Open | Sound无Runtime Scene/World system，component不会自动create/update/remove | per-world AudioWorldSystem消费scene delta、world transform、pause/time scale和teardown |
| AUD-P1-022 | Open | source_environment attenuation/cone/doppler/volume/occlusion/HRTF/convolution无生产caller | 编译per-source render parameters并在真实block执行，提供budget和数值oracle |
| AUD-P1-023 | Open | `engine/dsp`、`engine/filter`仍只在`cfg(test)`编译 | 接入production renderer并资格化，或迁移到明确testkit/prototype目录 |
| AUD-P1-024 | Open | source/listener/volume mutation只返回`Result<()>`并主要更新map | 返回accepted generation，callback发布applied generation、sample clock和disposition |
| AUD-P1-025 | Open | public接受External/Synth，playing admission和Kira binding明确拒绝 | 每种source kind有decoder/generator、backpressure、cancel/seek/lifecycle；否则不公开 |
| AUD-P1-026 | Partial | importer保留多声道layout并复用scratch，但SoundAsset/LoadedClip仍全PCM常驻，Kira只收mono/stereo | cooked/compressed artifact、stream blocks、decoder pool、layout/remix、seek和residency budget |
| AUD-P1-027 | Open | `max_voices`仅作为容量错误，无priority/concurrency/steal/virtualization | bounded VoiceAllocator按priority/audibility/cost决策并发布原因 |
| AUD-P1-028 | Open | completion靠manager调用轮询；`update_source`先stop再重新sync | callback写入bounded lifecycle queue；参数走sample/block-aligned command不重启voice |
| AUD-P1-029 | Open | source create可立即发gameplay emission，即使output未启动或无可听voice | perception区分intended/started/audible/virtualized/completed并绑定audibility receipt |
| AUD-P1-030 | Open | device identity由backend+display name组成，backend只支持mono/stereo | stable device identity、layout/rate/buffer能力枚举、协商和迁移policy |
| AUD-P1-031 | Open | configure先deactivate；失败无prepare-swap-commit-retire、restart、LKG或supervisor | DeviceSupervisor处理hotplug/default change/stall/loss/fallback和last-good |
| AUD-P1-032 | Open | stopped backend仍报告Ready；rendered/callback/underrun计数没有生产写入 | 状态机区分ready/stopped/starting/running/recovering/failed，由callback产生telemetry |

### 5.3 Event、automation、timeline、acoustics、qualification与性能

| ID | 状态 | 当前证据 | 工程级目标 |
|---|---|---|---|
| AUD-P1-033 | Partial | handler已预排序并按event建索引、delivery预留容量；pending仍为无界Vec且每次drain all | bounded MPSC/priority queue、bytes/items/time budget、公平性、drop reason和唯一pump |
| AUD-P1-034 | Open | callback ABI无deadline/cancel/generation/in-flight lease或unload quiescence | callback绑定provider generation和lease，terminal disposition后才允许卸载 |
| AUD-P1-035 | Partial | automation减少整图clone和binding clone；Kira active时仍明确返回M5 unsupported | compiler生成sample-clock command，active/inactive语义一致且有apply receipt |
| AUD-P1-036 | Partial | timeline做容量预分配、直接binding lookup和少量clone；仍由caller传delta逐track解释 | 绑定audio sample clock/epoch，支持seek/scrub/pause/loop/catch-up和missed-event policy |
| AUD-P1-037 | Open | ray provider只保存IR descriptor/sample/status，无scene geometry、ray scheduler或source binding | provider消费acoustic scene generation，生成versioned IR field并原子安装 |
| AUD-P1-038 | Open | convolution为直接frame×tap原型，无partition/tail/transition/latency contract | partitioned convolution、跨block state、IR crossfade、CPU budget、fallback和oracle |
| AUD-P1-039 | Open | HRTF preview为简化delay/gain，缺dataset cook/interpolation/切换和multi-listener policy | cooked HRTF profile、bounded per-source state、interpolation/tail与typed degradation |
| AUD-P1-040 | Open | occlusion/ray query无geometry owner、thread handoff、stale result和budget | 按world/acoustic generation异步batch，丢弃stale并发布latency/quality |
| AUD-P1-041 | Open | runtime没有统一AudioClipArtifact、stream index、dependency/provenance/last-good | 与asset owner对接platform artifact；runtime只消费admitted artifact和resident pages |
| AUD-P1-042 | Open | source/manager/device/graph/feature无统一shutdown/drain/reload/generation顺序 | SoundRuntimeInstance停止admission、drain command/callback、退役voice/graph/artifact再释放 |
| AUD-P1-043 | Open | source/export/native/editor没有同一行为与错误矩阵 | 同一scenario corpus覆盖registration/playback/graph/device/feature/failure parity |
| AUD-P1-044 | Open | 1,035个测试路径文件高度碎片化，结构测试预算超过产品行为资格 | 合并support/source-shape门，将预算迁移到executable contract/device/product tests |
| AUD-P1-045 | Open | 无真实设备、rate/layout/buffer、hotplug、overrun、OOM、long-soak矩阵 | hardware/software-null矩阵、fault injection、offline oracle和机器可读结果 |
| AUD-P1-046 | Open | 无callback CPU/alloc/lock/voice/stream miss/XRUN/latency/memory统一观测 | audio thread写无分配ring telemetry，Runtime/Editor diagnostics异步读取 |
| AUD-P1-047 | Open | beta/partial升级不绑定产品workload、correctness、failure、memory或latency阈值 | SoundQualificationRecord绑定BuildSet/source/device/workload和原始样本 |
| AUD-P1-048 | Open | 没有同功能、同设备、同采样率/layout/DSP/voice/streaming竞争证据 | correctness、可靠性和统计基准全部通过后才允许与Unreal比较 |

## 6. P2工程级能力扩展

| ID | 状态 | 当前差距 | 目标方向 |
|---|---|---|---|
| AUD-P2-001 | Open | 无MetaSound类procedural graph、typed node compiler和audio VM | 基础renderer稳定后建立确定性graph IR/compiler/artifact/bounded VM |
| AUD-P2-002 | Open | 无ambisonics、soundfield、object audio或平台spatial endpoint | channel/object/soundfield协商和可插拔空间backend |
| AUD-P2-003 | Open | 无microphone/capture/voice chat/AEC/NS/jitter/privacy | 独立communications owner接入permission、DSP、network和privacy |
| AUD-P2-004 | Open | 无portal、diffraction、多路径传播和acoustic LOD | acoustic scene与query budget成熟后增加分级传播模型 |
| AUD-P2-005 | Open | 无GPU/硬件ray acoustics或异步bake farm | 通过同一IR artifact接入可选backend，不把GPU存在当正确性 |
| AUD-P2-006 | Open | 无平台codec、hardware DSP、console/mobile低功耗策略 | target codec/cook/profile和平台资格，禁止runtime猜测 |
| AUD-P2-007 | Open | 无loudness/dialogue normalization/hearing accessibility/localization variation | 内容元数据、bus policy、subtitle/dialogue link、meter和用户设置链 |
| AUD-P2-008 | Open | 无offline render、stem bounce和deterministic mix export | 复用compiled graph/sample clock提供headless render与artifact receipt |
| AUD-P2-009 | Open | 无remote profiler、trace capture和block replay | bounded trace、privacy filter、BuildSet/device identity和离线分析 |
| AUD-P2-010 | Open | 无第三方spatial/reverb/codec认证与故障隔离 | audio extension ABI、实时安全规则、sandbox/timeout和certification corpus |
| AUD-P2-011 | Open | 无adaptive music state/transition/stem quantization/cinematic integration | music clock、tempo map、quantized transition和Timeline共享artifact |
| AUD-P2-012 | Open | 无large-world audio partition、区域流送、voice cluster和跨world travel | per-world owner和artifact streaming完成后实现partition与平滑handoff |

## 7. 历史台账重判

### 7.1 Runtime08B

| 原ID | 状态 | 当前判定 |
|---|---|---|
| P1-1、2、3 | Open | product bootstrap/World system仍缺；仍是单global state；factory仍用default config |
| P1-4 | Partial | Kira已有静态PCM和增量graph diff，但effect、advanced control、active structural swap仍缺 |
| P1-5、6 | Open | spatial/environment未进Kira，两个optional feature仍是空provider |
| P1-7、8 | Partial | importer layout/scratch和locator fast-cache有进展；prepared stream、single-flight、unload/eviction/residency仍缺 |
| P1-9、10 | Open | 无voice allocator；completion仍轮询且whole-source update重启voice |
| P1-11、12 | Partial | timeline/event减少分配与排序；sample clock、bounded queue、budget和唯一pump仍缺 |
| P1-13、14 | Open | 无device supervisor；callback/XRUN/meter生产观测仍缺 |
| P1-15 | Partial | graph compile可在锁外做，Arc/COW减少复制；全服务单mutex仍在 |
| P1-16、17 | Open | External/Synth无adapter；Editor仍是descriptor/placeholder壳 |
| P1-18、19 | Partial | importer热路和multichannel metadata改善；owner重复、Opus diagnostic-only和Kira双声道限制仍在 |
| P1-20 | Open | source-shape和ignored microbenchmark仍不能替代产品资格 |

Runtime08B的5项P2均Open：capture/voice chat；ray acoustics/convolution；interactive music/sample-accurate graph；offline/deterministic/quality regression；advanced platform/accessibility/pro production。

### 7.2 Plugins11

Plugins11的48项P1保持原编号，当前只有5项Partial：

| 原ID | 状态 | 当前判定 |
|---|---|---|
| NSND-P1-020 | Partial | graph snapshot/COW/锁外compile降低局部control cost，但统一state mutex与callback observation边界未解决 |
| NSND-P1-026 | Partial | channel layout和decode scratch进展真实；full PCM、StaticSoundData复制、stereo-only与无stream仍在 |
| NSND-P1-033 | Partial | handler registry预排序/索引和delivery capacity已优化；pending/budget/pump/deadline仍缺 |
| NSND-P1-035 | Partial | inactive automation COW/target projection更轻；active Kira仍Unsupported |
| NSND-P1-036 | Partial | timeline lookup/capacity/clone减少；sample clock、seek/scrub/catch-up仍缺 |
| 其余43项P1 | Open | composition、dist、feature、Editor、preset、routing、World、device、voice、artifact、acoustic、lifecycle与qualification失败条件均仍存在 |

Plugins11的12项P2全部Open，与AUD-P2-001..012一一对应。

### 7.3 Editor17

Editor17的5项P0全部Open：默认产品无法打开Sound资产；33项operation无factory；live-output无产品owner；五份surface以29个Space占位；没有transactional authoring/audition/scene loop。

Editor17的60项P1按原编号全部Open。当前源码没有AudioClip document/waveform/transport，Mixer document/strip/send/effect/automation compiler，Scene Audio bridge/gizmo/acoustic overlay，真实device picker/telemetry/lifecycle，Timeline sample conversion，也没有save/reopen、real-device、malicious codec、large-authoring或竞品资格。12项P2也全部Open。

## 8. Failure handoff状态

| Handoff | 状态 | 当前判定 |
|---|---|---|
| `docs/plans/zircon_plugins/02/failure-2026-07-19-kira-send-frame-capture-routing.md` | Open | canonical记录仍为4项中1 pass / 3 fail；当前source虽有send route增量修改，但没有fresh current-source focused/broad/product GREEN、独立review和协调器提交证据 |

不得以source diff、测试名存在或局部unit pass推断该failure关闭。修复owner仍是Plugins02的Kira graph compile/route installation边界。

## 9. 参考引擎差异

### 9.1 Unreal Engine

- `FMixerDevice`与`FMixerSourceManager`显式区分game/audio/render thread，MPSC command queue由render thread pump；source finish、effect tail、scheduled render step、relative render cost和command queue fill都有明确owner。
- device层覆盖hardware init/teardown/timing/stall、device list/default change和恢复；submix具备dry/wet、dynamic effect chain、recording、meter/envelope/spectrum、bus和soundfield路径。
- `SoundWave`拥有platform compressed data、stream chunk、seek table、DDC、loading behavior、first chunk和proxy/hot-reload边界。Zircon的full PCM `SoundAsset -> StaticSoundData`不在同一工程等级。
- `IAudioExtensionPlugin`把spatialization、occlusion、reverb等provider的factory/device/per-source init-release/process callback分开。Zircon optional feature只有capability/descriptor，尚不构成扩展系统。
- AudioEditor有SoundWave preview/reimport和Submix graph/node/connection/undo/save/effect authoring。33个descriptor和29个Space不构成同级Editor。

### 9.2 Godot

- AudioServer直接拥有driver、speaker mode、device、mix count、bus/channel/effect/send、peak和playback list；mix与主线程释放有清晰隔离。
- AudioStreamPlayer3D持续提交position、attenuation、area、doppler和bus volume；空间属性不是只存DTO。
- Audio bus editor提供add/duplicate/move/effect、undo/redo和save；stream editor提供waveform/preview。Zircon必须把World和Editor链接到真实执行图。

### 9.3 Fyrox

- SoundEngine可拥有真实output，也可headless manual render；多个SoundContext分别拥有source、listener、renderer、bus graph和distance model。
- context render会将source真正混入bus/effect graph；HRTF renderer进入实际输出路径，而非不可达preview helper。
- StreamingBuffer有增量decoder、固定block、rewind/seek和独占语义。即使规模小于Unreal，它也给出了streaming/spatial/bus/output端到端的最低下界。

### 9.4 Bevy

- AudioPlugin把queued asset playback、sink创建、finished cleanup和spatial position update安装到ECS schedule；asset未加载时保持queued，加载后才创建sink。
- Bevy明确声明空间音频只是stereo pan且不支持HRTF。它的适用结论是“能力较少但产品路径真实且声明诚实”，不是Zircon可以停在更宽的空合同。

### 9.5 Unity Graphics边界

本地`dev/Graphics`是SRP Core/URP/HDRP/ShaderGraph/VFX Graphics镜像，package说明也只声明render pipeline基础，不包含Unity Audio实现。因此本报告不伪造Unity Audio对比；Unity Graphics只用于确认Audio到VFX/Render的consumer应通过稳定跨域artifact/observation边界，不能替代Unreal/Godot/Fyrox/Bevy的Audio参考。

## 10. 目标架构与hard cutover

### 10.1 Owner与数据流

```text
Project manifest + target profile + linked providers
  -> SoundActivationPlan / Receipt
      -> SoundRuntimeInstance { host/world/provider/config generations }
          +-- AudioDeviceSupervisor
          +-- AudioRenderService { callback + sample clock + bounded commands }
          +-- AudioWorldSystem { source/listener/volume projection }
          +-- AudioResourceService { clip/stream/HRTF/IR residency }
          +-- MixerCompiler -> immutable MixerGraphGeneration
          +-- AcousticProvider / TimelineAudioProvider leases
          +-- AudioObservationStream
                 +--> gameplay/AI/VFX
                 +--> Editor mixer/audition/acoustic/timeline tools
```

| 类型 | Owner | 必须包含 |
|---|---|---|
| `SoundProviderDescriptor` | Runtime catalog + provider | build/platform/backend/features/limits、realtime rules和qualification receipt |
| `SoundActivationPlan/Receipt` | Runtime composition | selected package/features/options/provider generations和typed degraded/failure |
| `AudioClipArtifact` | Asset/compiler | source/settings/compiler/target hashes、codec/layout/rate、stream/seek table、loudness/markers/dependencies |
| `MixerGraphArtifact` | Mixer compiler | stable bus/effect/send/parameter IDs、topology、latency、resource budget和executable matrix |
| `SoundRuntimeInstance` | Runtime | lifecycle、config/provider/device/world generations、service leases和fault state |
| `AudioWorldInstance` | Runtime World | component/entity generations、listener policy、voice intents、pause/time authority和teardown |
| `AudioRenderCommand` / `Receipt` | Render service | sequence、target generation、sample/block deadline、apply/reject/drop disposition |
| `VoiceHandle` | Render service | instance/generation/slot、priority/concurrency/virtual state和completion cursor |
| `AudioObservationBatch` | Observation stream | sample clock、graph/device/world generation、meters/XRUN/voice/stream/event journals和gap |
| `AudioPreviewSession` | Editor | source revision、artifact/mixer/provider generation、device lease、transaction和audition state |

### 10.2 必须删除或替换的旧路径

| 旧路径 | hard cutover |
|---|---|
| target只启用`sound-contracts`却可表现为Sound能力存在 | 改为selected provider + activation receipt；无provider明确Unavailable |
| module factory固定`SoundConfig::default()` | 删除；只接受validated EffectiveSoundConfig artifact |
| 单一`Mutex<SoundEngineState>`拥有全部控制与状态 | 拆为instance/control/render/resource/world/observation owner和有界通信 |
| full PCM `SoundAsset` + resident `StaticSoundData` | 替换为platform cooked clip/stream artifact与page residency；小clip可显式static policy |
| mono复制stereo、multichannel播放拒绝 | 替换为layout-aware remix/spatial/output negotiation，不静默改变内容 |
| active playback时禁止结构图变更 | 替换为immutable generation block-boundary swap和retirement fence |
| completion依赖API轮询、whole-source update stop/restart | 替换为callback lifecycle queue和sample/block command |
| manual `advance_timeline_sequences(delta)`作为产品clock | 删除产品authority；timeline绑定audio sample clock和epoch |
| unbounded pending event + drain-all +同步foreign callback | 替换为bounded queue/budget/deadline/lease/quiescence |
| 不可达source_environment、test-only DSP/filter支撑capability | 接入真实render或移入prototype/testkit并从capability删除 |
| 空feature ModuleDescriptor和stateless metadata dist | 实现provider行为或明确Unsupported，不能Loaded/Enabled |
| 33个无factory command和29个Space | 删除可见入口或接入真实document/operation/controller/observation |
| display-name device ID和configure先停 | 替换为stable identity、prepare/swap/commit/retire、LKG和recovery |
| source-shape test/ignored microbenchmark作为产品完成证据 | 只保留少量结构门；资格必须绑定current source/build/device/workload原始样本 |

### 10.3 性能证据规则

“优于Unreal”只能在相同hardware/OS/compiler、device/backend、sample rate/layout/block size、clip codec/streaming、voice count、bus/effect/spatial/acoustic workload和正确性容差下比较。至少报告median、p95、p99、max、callback deadline miss/XRUN、alloc/lock、stream miss、voice steal/virtualization、latency、resident/peak memory、startup/recovery时间和输出误差。只比较容器查找、descriptor registration、空图或不同功能集平均耗时一律不计。

## 11. 依赖顺序与实施里程碑

| 里程碑 | 依赖 | 交付物 | 完成条件 |
|---|---|---|---|
| M0 Sound truth freeze | 全局MVP 00 composition/owner前置 | reachability、false capability、default-config、invalid preset、routing、zero-callback RED门 | ordinary Client/Editor/source/native的失败可稳定复现且不新增shim |
| M1 Composition/config/lifecycle | Runtime composition compiler | SoundActivationPlan/Receipt、EffectiveSoundConfig、SoundRuntimeInstance、quiescence | provider/feature/options/target事实唯一；shutdown无late callback/handle |
| M2 Device/audio thread/mixer generation | M1 | DeviceSupervisor、bounded command/observation、sample clock、immutable graph swap | callback无普通锁/分配/阻塞；routing红门和device recovery通过 |
| M3 World/voice/resource streaming | M1-M2 + asset/resource基础 | AudioWorldSystem、AudioClipArtifact、decoder/pages/residency、VoiceAllocator、completion journal | create/update/remove/stream/seek/pressure/teardown在产品路径闭合 |
| M4 Spatial/acoustic/automation/timeline | M2-M3 + Physics/Time generation | render-integrated spatial/HRTF/occlusion/convolution、sample-clock automation/timeline | 所有声明能力改变真实输出并有budget/oracle/degraded状态 |
| M5 Editor产品闭环 | M1-M4 + Editor operation/document | Clip/Mixer/Acoustic/Timeline documents、transactions、compiler、audition、live observation | 33项command可执行或删除；29个Space消失；save/reopen/preview/runtime parity |
| M6 Distribution与qualification | M1-M5 | source/native parity、offline oracle、device/fault/soak/cross-platform/benchmark corpus | 全部门通过后才允许shipping readiness和竞争结论 |

全局MVP 00尚未允许高级Sound实现时，本报告只作为review/实施依赖记录。实施必须底层优先：composition、asset、time、world、operation或plugin lifecycle失败，应由对应owner修复，不得在Sound建立本地兼容层。

## 12. 资格门

### 12.1 Product、lifecycle、device与render

| Gate | 状态 | 通过要求 |
|---|---|---|
| G01 product provider reachability | Fail | ordinary Client、Editor Host、generated export和NativeDynamic生成同一SoundActivationPlan/Receipt |
| G02 feature closure | Fail | root/ray/timeline runtime/editor provider closure可重建，状态与generation一致 |
| G03 effective config | Fail | plugin options全部进入validated config，非法值在实例/device创建前拒绝 |
| G04 source/native parity | Fail | registration、state、lifecycle、error、unload和capability行为等价 |
| G05 instance shutdown | Fail | 停admission、drain command/callback、退役graph/voice/artifact后释放device/provider |
| G06 realtime safety | Fail | callback无普通Mutex等待、动态分配、文件/decoder阻塞或foreign callback |
| G07 device supervision | Fail | stable ID、format negotiation、hotplug/default change/stall/loss/LKG恢复 |
| G08 executable matrix | Fail | validator/compiler/preset/Editor palette消费同一可执行能力矩阵 |
| G09 Kira routing | Fail | direct/send/master/parent gain在fresh current-source frame capture全部通过 |
| G10 graph generation swap | Fail | active playback期间block-boundary原子切换，旧generation安全退役 |
| G11 preset truth | **Partial** | default可执行；music_sfx/spatial_room须执行或从可见catalog移除 |

### 12.2 World、resource、voice与spatial

| Gate | 状态 | 通过要求 |
|---|---|---|
| G12 AudioWorldSystem | Fail | source/listener/volume create/update/remove、transform、world unload和entity generation产品测试 |
| G13 spatial output | Fail | attenuation/cone/doppler/volume/occlusion/HRTF/convolution改变真实block并有oracle |
| G14 source-kind admission | Fail | External/Synth未实现时拒绝；实现后有backpressure/cancel/seek/lifecycle |
| G15 clip artifact/stream | **Partial** | 已有layout基础；须有cook/stream/residency/seek/remix且不再全PCM+mono复制 |
| G16 voice allocation | Fail | 超额source按priority/audibility/cost virtualize/steal并发布原因 |
| G17 completion/update | Fail | callback有界发布completion，普通参数更新不stop/restart |
| G18 gameplay audibility | Fail | intended/started/audible/virtualized/completed语义和generation准确 |

### 12.3 Event、timeline、acoustics、Editor与qualification

| Gate | 状态 | 通过要求 |
|---|---|---|
| G19 dynamic event budget | **Partial** | 已有预排序索引；须有bytes/items/time budget、公平性、drop/deadline/cancel和唯一pump |
| G20 callback unload safety | Fail | callback绑定generation/in-flight lease，卸载前quiesce，late callback失效 |
| G21 sample-clock automation | Fail | active/inactive语义一致，绑定sample clock/epoch/seek/pause/loop |
| G22 acoustic scene provider | Fail | ray provider消费真实geometry generation并生成可安装IR |
| G23 convolution qualification | Fail | partition/tail/IR transition/latency/CPU budget/fallback/oracle |
| G24 HRTF qualification | Fail | cook/validation/interpolation/tail/multi-listener/degraded policy |
| G25 artifact identity | Fail | clip/graph/HRTF/IR/timeline绑定source/settings/compiler/target/dependency/content hash和last-good |
| G26 Sound Editor closure | Fail | catalog真实链接，Clip/Mixer/Acoustic/Timeline有document/transaction/save/reopen/compiler |
| G27 executable commands | Fail | 33个公开command均有factory与terminal receipt，或从catalog删除 |
| G28 truthful surfaces | Fail | 29个Space不再承载业务；数据来自document/observation且断连typed degraded |
| G29 audition parity | Fail | audition与runtime消费同artifact/mixer/provider generation，preview/export golden parity |
| G30 evidence identity | **Partial** | 已有若干raw-sample ignored perf marker；须绑定current source/build/device/workload并由受管门执行 |
| G31 reliability matrix | Fail | offline oracle、真实设备、hotplug/fault/OOM/voice pressure/stream miss/soak机器可读 |
| G32 competitive benchmark | Fail | 同功能同设备同格式同负载记录correctness、CPU、deadline、latency、memory后才比较 |

合计：32项Gate为 **28 Fail、4 Partial、0 Pass**。Partial只代表局部source foundation，不允许发布shipping-ready能力。

## 13. Owner边界与非重复计数

1. App只拥有进程、target/profile选择与composition提交，不拥有SoundManager、device或mixer truth。
2. Runtime拥有neutral contracts、activation/artifact header、World/audio clock连接、generation handles和observation contract；Kira实现留在Sound plugin。
3. Asset/Resource拥有source/import/cook/cache/dependency/last-good；本报告记录Sound消费缺口，不另建私有asset authority。
4. Sound plugin拥有device backend、audio render、voice、mixer compiler和provider实现，不拥有Scene authoring或App selection。
5. Editor拥有Clip/Mixer/Acoustic/Timeline source document、transaction、preview和诊断投影，不拥有runtime callback truth。
6. Physics只提供generation-bound acoustic query；AI/VFX只消费audibility/observation，不可从source create推断已可听。
7. Runtime08B、Plugins11、Editor17保留历史编号；本报告刷新currentness，不把相同失败重复计为新增finding。

## 14. 首个实施切片

在全局MVP 00和composition前置允许Sound实施后，首个切片只做truth gate，不直接扩充DSP或Editor面板：

1. 添加ordinary Client、Editor Host、runtime/editor catalog、generated/native的provider reachability RED测试。
2. 添加manifest option未进入factory、空feature/dist却可声明capability的fail-closed RED测试。
3. 保留并fresh执行三项Kira send frame-capture红门，不弱化断言或改成mock。
4. 添加`music_sfx`/`spatial_room`可见但compiler拒绝的catalog truth RED测试。
5. 添加full PCM双常驻、mono复制stereo、多声道拒绝和无unload/residency的资源RED证据。
6. 添加无World system、source update重启、completion轮询、active automation拒绝、manual timeline clock和unbounded event的RED测试。
7. 添加callback telemetry恒零、device configure失败无LKG及Editor command无factory/Space占位的产品RED门。
8. 冻结删除矩阵后，先交付`SoundActivationReceipt + SoundRuntimeInstance generation + EffectiveSoundConfig`最小vertical slice；不保留旧default factory兼容壳。

## 15. 本轮未做事项

- 未修改production、tests、Cargo、manifest或ZUI；未实现任何Audio/Sound修复。
- 未运行Cargo check/test、真实Client/Editor、Kira/CPAL设备、PIE、NativeDynamic、asset cook/reload或audition。
- 未运行callback capture、XRUN、hotplug/fault/OOM、voice pressure、stream miss、scale/soak、cross-platform或性能profile。
- 未执行与Unreal/Godot/Fyrox/Bevy的同语义benchmark，因此不作性能优越声明。
- 未覆盖用户明确排除的Tooling优化；Tooling未来Rust迁移另立计划。
- 当前selected文件含其他Session/用户改动；实施前必须重取HEAD、epoch、文件集和fingerprint，再按测试驱动及受管验证推进。
