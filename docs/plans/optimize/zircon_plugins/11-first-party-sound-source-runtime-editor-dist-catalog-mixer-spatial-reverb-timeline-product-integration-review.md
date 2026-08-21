---
title: First-Party Sound Source、Runtime、Editor、Dist、Catalog、Mixer、Spatial、Reverb、Timeline 与 Product Integration 工程化差距
category: zircon_plugins
report_id: Plugins11
review_date: 2026-08-19
baseline_head: 25e09a23178000f2e783ce2143cf70a8b118d404
baseline_epoch: 333
related_code:
  - zircon_plugins/sound/plugin.toml
  - zircon_plugins/sound/runtime/Cargo.toml
  - zircon_plugins/sound/runtime/src
  - zircon_plugins/sound/editor/Cargo.toml
  - zircon_plugins/sound/editor/src
  - zircon_plugins/sound/editor/acoustic_debug.zui
  - zircon_plugins/sound/editor/audio_listener.drawer.zui
  - zircon_plugins/sound/editor/audio_source.drawer.zui
  - zircon_plugins/sound/editor/audio_volume.drawer.zui
  - zircon_plugins/sound/editor/mixer_console.zui
  - zircon_plugins/sound/dist/Cargo.toml
  - zircon_plugins/sound/dist/src
  - zircon_plugins/sound/features/ray_traced_convolution_reverb
  - zircon_plugins/sound/features/timeline_animation_track
  - zircon_plugins/first_party_runtime_catalog/Cargo.toml
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_plugins/first_party_editor_catalog/Cargo.toml
  - zircon_plugins/first_party_editor_catalog/src
  - zircon_app/Cargo.toml
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_app/src/entry/first_party_editor_plugins.rs
  - zircon_app/src/entry/export_bootstrap.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/sound_features.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/sound_features/manifest.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/sound_features/rows.rs
tests:
  - zircon_plugins/sound/runtime/src/tests
  - zircon_plugins/sound/editor/src/tests.rs
  - zircon_plugins/sound/dist/src/lib.rs
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/runtime/src/tests.rs
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/editor/src/tests.rs
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/dist/src/lib.rs
  - zircon_plugins/sound/features/timeline_animation_track/runtime/src/tests.rs
  - zircon_plugins/sound/features/timeline_animation_track/editor/src/tests.rs
  - zircon_plugins/sound/features/timeline_animation_track/dist/src/lib.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md
  - docs/plans/optimize/zircon_plugins/07-first-party-asset-importer-source-dependency-subasset-artifact-determinism-sandbox-product-integration-review.md
  - docs/plans/optimize/zircon_plugins/08-first-party-editor-authoring-extension-document-operation-toolkit-runtime-contract-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/08b-audio-runtime-review.md
  - docs/plans/optimize/zircon_runtime/22-time-clock-domain-fixed-step-determinism-rng-replay-scheduling-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/42-builtin-runtime-module-catalog-profile-target-feature-selection-extension-registration-capability-load-report-product-integration-review.md
  - docs/plans/optimize/zircon_editor/17-sound-audio-clip-mixer-routing-effect-spatial-acoustic-timeline-authoring-review.md
  - docs/plans/optimize/zircon_editor/25-runtime-diagnostics-performance-timeline-console-telemetry-observability-authoring-review.md
  - docs/plans/optimize/zircon_editor/50-editor-extension-contribution-store-registry-toolkit-provider-snapshot-reload-lifecycle-product-integration-review.md
  - docs/plans/zircon_runtime/runtime/01/failure-2026-07-31-kira-sound-owner-inventory-drift.md
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
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 11 · First-Party Sound Source、Runtime、Editor、Dist、Catalog、Mixer、Spatial、Reverb、Timeline 与 Product Integration 工程化差距

## 1. 结论

zircon_plugins/sound 不是纯接口壳。它已经有 Kira 0.12.2 输出桥、typed source/listener/volume/mixer/effect/automation/timeline/acoustic 合同、图校验、PCM playback、output lifecycle、动态事件 ABI、若干空间/遮挡/HRTF/卷积算法、三份 mixer preset，以及大量局部测试。全包 1,307 个 tracked 文件中，1,292 个是 Rust 文件，1,038 个文件位于测试路径；362 项 test attribute 对图校验、source validation、Kira lifecycle、动态事件和 manifest 结构形成了细粒度回归底座。

问题不是“没有代码”，而是这些代码没有形成普通 Zircon 产品中的可听音频系统。默认 App target 不启用 first-party runtime plugin 集合，target-editor-host 也不链接 base runtime catalog；first-party editor catalog只链接 Navigation 与 Neural。因此普通 Client 和 Editor Host 都可以编译到 Sound 合同，却没有安装 Sound provider。generated export 能走另一条显式 registration 路径，但它和普通 host 不是同一个组合事实源，不能证明预览、编辑器、开发运行与导出产品等价。

即使显式构造 Sound manager，真实 Kira render path 也只闭合了有限 PCM 播放。source_environment 下的 attenuation、cone、doppler、volume、occlusion、HRTF 和 convolution 没有生产调用者；engine/dsp 与 engine/filter 更只在测试配置下编译。mixer graph 对外接受 effect、advanced control 和 pre-effect send，compile 阶段却统一拒绝这些语义；内建 music_sfx 与 spatial_room preset 正好使用当前不可执行的 effect/send，所以 catalog 中三份 preset 实际只有 default 可应用。Kira send routing 仍有三项已登记红测，不能宣称 submix/send 路由正确。

Editor 与 optional feature 同样主要停留在声明层。Sound Editor 有 33 个 command descriptor，但没有 operation factory；五份 ZUI 中除 Mixer 的 Refresh/Start/Stop 外，业务区域均为 Space 占位。live-output controller 只有测试 fake 构造，没有产品 factory。Ray-Traced Convolution Reverb 和 Timeline Animation Track 的 runtime/editor registration 都只发布空 ModuleDescriptor 或 capability，dist 仍只导出 metadata。它们的 beta capability 没有对应可执行 provider。

音频 importer 与 SoundClip artifact 的唯一 owner 已由 Plugins07 管理；音频运行时本体由 Runtime08B 管理；Sound authoring 由 Editor17 管理；catalog/native 通用缺陷由 Plugins01/06 管理。本篇不重复计最高优先级问题，登记 **0 项新增 P0、48 项 P1、12 项 P2**。本篇只拥有 Sound 单包从 manifest、source runtime、optional feature、editor、dist、catalog 到普通产品 consumer 的纵向交付合同。

## 2. 审查边界、规模与 currentness

### 2.1 物理冻结

| 范围 | 文件 / 行 / bytes | 冻结事实 |
|---|---:|---|
| zircon_plugins/sound 全包 | 1,307 / 23,903 / 907,000 | 9 个 Cargo manifest、1,292 个 Rust 文件、plugin.toml 与 5 个 ZUI |
| production / exact test-path files | 269 / 1,038 | runtime 生产 232 个文件；测试路径共约 12,411 行 |
| package fingerprint | 0a1d327265a1b5c134aa52f4f5b7e3b9a12bbb79adbf14e5f38b1f4ba86b624b | tracked path 排序，以小写 path 加文件 SHA-256 的 LF 串、无末尾 LF 再计算 SHA-256 |
| base runtime | 1,265 / 21,314 / 799,224 | production 232、test 1,033；Kira bridge、service types、engine、automation、timeline、dynamic event、output 与 package |
| base editor / dist | 15 / 1,289 / 54,854；2 / 100 / 4,047 | Editor 33 commands、5 ZUI、live controller；dist metadata projection |
| ray feature runtime/editor/dist | 5 / 246 / 11,054；5 / 75 / 2,797；2 / 157 / 6,477 | 三层均只有 descriptor/capability/registration 壳 |
| timeline feature runtime/editor/dist | 5 / 237 / 10,551；5 / 72 / 2,711；2 / 152 / 6,261 | 三层均只有 descriptor/capability/registration 壳 |
| 产品装配 | ordinary runtime 0、editor 0；catalog root 1、optional feature 0 | runtime catalog在 feature gate 下链接 Sound root；默认 App/Editor target 不启用该闭包 |

源 revision 为 25e09a23178000f2e783ce2143cf70a8b118d404。冻结时 Sound 包无 tracked working-tree 差异；共享 catalog、App、Runtime 与计划存在其他会话或用户改动，因此本文按当前工作树读取并保留 source_recheck_required。实施前必须重算 Sound、catalog、profile、App entry、Runtime builtin row 与 audio importer consumer 的同一 generation。

### 2.2 测试库存不等于产品资格

测试路径中的 1,038 个文件不是 1,038 条独立行为门。其中 optional_feature_manifest 占 582 个文件、约 4,397 行和 151 项 test，manifest support 又有 238 个文件。optional feature 的 147 项 structure test 大量使用 include_str 与 contains，主要证明源码被拆成期望文件、转发方法仍存在；它们不能证明 provider 已装入产品、音频线程实际执行、结果可听或卸载安全。

较有价值的行为测试集中在 Kira bridge、dynamic event、playback、graph validation、source input、output lifecycle 与 poison recovery。它们仍主要依赖 MockBackend、单进程状态或显式 manager 调用。仓内没有真实设备矩阵、callback deadline/glitch、长时 soak、voice pressure、streaming residency、device hotplug、Editor audition 或 preview/export parity 资格。

docs/plans/zircon_plugins/02/failure-2026-07-19-kira-send-frame-capture-routing.md 仍记录精确门：4 项中 1 通过、3 失败，分别涉及 post-effect send contribution、master gain 只应用一次、活动图 parent gain 变化后的 send resync。本篇不得把 Kira routing 写成已修复。

### 2.3 本轮纵向追踪

1. plugin.toml 的 maturity、capability、module、option、event 与两个 optional feature。
2. base runtime 的 ModuleDescriptor、manager factory、service types、Kira device/playback/graph、output lifecycle、动态事件、automation、timeline、ray/acoustics 与 presets。
3. Ray-Traced Convolution Reverb 与 Timeline Animation Track 的 runtime/editor/dist 三层 registration。
4. Sound Editor provider、commands、component drawer、mixer/acoustic surface、live-output controller 与测试 fake。
5. first-party runtime/editor catalog、App target feature、generated export bootstrap 与 Runtime builtin feature rows。
6. Runtime08B、Editor17、Plugins01/06/07 与两个 open failure owner 的边界。
7. Unreal AudioMixer/AudioExtensions/AudioEditor、Godot AudioServer、Fyrox sound、Bevy audio，以及 Unity Graphics package topology。

本轮为 E3 静态源码审查，没有运行 Cargo、真实声卡、Editor、NativeDynamic、音频 importer、空间场景、soak 或性能测试。测试数量是源码库存，不是本轮通过数。

## 3. 当前真实产品链与断点

~~~text
ordinary zircon_app client
  -> target-client
  -> does not enable first-party-runtime-plugins
  -> Sound contracts can exist, Sound provider is absent

ordinary zircon_app editor host
  -> target-editor-host
  -> links selected advanced plugins, not base Sound runtime
  -> first_party_editor_catalog omits Sound and both Sound feature editors

explicit first-party runtime catalog
  -> can link Sound root under base-runtime-plugins
  -> does not collect ray/timeline optional providers

generated export bootstrap
  -> can explicitly link selected feature registrations
  -> uses a stronger composition path than ordinary App/Editor host

Sound root registration
  -> SoundDriver + lazy DefaultSoundManager/Kira manager
  -> factory always uses SoundConfig::default()
  -> plugin.toml options do not become effective runtime config
  -> no World/Scene AudioSource/Listener/Volume system

Kira output
  -> Clip and Silence sources can play
  -> graph effects/advanced controls/pre-effect sends rejected
  -> spatial/HRTF/occlusion/volume/convolution algorithms not called
  -> completion, timeline and dynamic events advance only when callers poll

Sound Editor
  -> provider is not in editor catalog
  -> 33 descriptors have no factories
  -> five surfaces are placeholders
  -> live controller has test fake only

NativeDynamic dist and optional features
  -> descriptor/capability metadata
  -> no equivalent manager, render, editor, state, lifecycle or bridge behavior
~~~

目标不是把所有职责塞进一个全局 SoundManager。目标是由 host 的 resolved activation plan 创建明确的 SoundRuntimeInstance，由 device supervisor 与 audio render service 拥有 callback/thread，World audio system 拥有 scene projection，asset service 拥有 clip/stream residency，mixer generation 拥有不可变 compiled graph；Editor、ray acoustics 与 timeline 通过 typed generation-bound contract 接入同一实例。

## 4. 可保留基础

| 基础 | 当前价值 | 重构约束 |
|---|---|---|
| Kira 0.12.2 bridge | 已能驱动真实 CPAL playback，且 Kira 依赖已收敛到 Sound runtime owner | 保持唯一 owner；不得在其他 crate 再建平行 Kira authority |
| Typed service contracts | source/listener/volume/mixer/effect/timeline/device/error 均有较完整词汇 | 合同必须绑定实际执行、generation、thread domain 与 capability receipt |
| Validation 分层 | descriptor 和 graph 有大量负向校验 | admission 与 executable compiler 必须同一能力矩阵，不能先接受后在 compile 拒绝 |
| Kira lifecycle transaction | start/stop、graph install、source sync 和失败回退已有结构 | 扩为 device supervisor、recovery、drain、callback fence 与 generation retirement |
| Dynamic event ABI | request/status/callback/executor 已有 typed ABI | 加 bounded queue、pump owner、deadline/cancel、unload quiescence 和 dispatch receipt |
| Mixer preset catalog | default/music_sfx/spatial_room 表达了产品意图 | 只有 compiled graph 真能执行时才可见；不可执行 preset 必须 fail-close |
| Spatial/acoustic algorithms | attenuation、cone、doppler、occlusion、HRTF preview、volume influence 和 convolution 有原型 | 必须接入真实 render block，建立跨 block state、预算和 oracle，不保留不可达“完成外观” |
| Editor contribution vocabulary | view、drawer、command、live-output model 已有边界词汇 | 接入真实 document/factory/controller/telemetry，不让 Space 或 descriptor 作为完成证据 |
| Beta/partial 声明 | 没有把 Sound 标成 stable/complete | 所有 G01-G32 通过前保持 beta/partial；optional feature 默认关闭 |

## 5. 参考实现给出的工程边界

### 5.1 Unreal AudioMixer、AudioExtensions 与 AudioEditor

AudioMixerDevice 和 AudioMixerSourceManager 显式区分 game、audio 与 render thread，并用命令队列把控制变更送入 callback。render callback 负责泵命令、更新时间、更新 source、执行 submix graph 和生成最终 block；source release、pending command 和 effect chain 均有清楚的线程归属。Zircon 可以采用 Kira 或自己的 renderer，但不能让一个普通 Mutex 同时承担 device、graph、playback、timeline 与 event 的跨线程所有权。

Unreal 的 device owner 处理 hardware stall、device swap、default device 变化和多阶段恢复；submix 具有 main/default/reverb/EQ、dynamic effect chain、send、wet/dry、meter/envelope/spectrum、recording 与 listener。SoundWave 又拥有 compressed/cooked platform data、streaming chunk、cache、seek、first chunk、loading behavior 和 hot-reload proxy。它证明工程级音频不是“解码完整 PCM 后播放”一条路径。

IAudioExtensionPlugin 为 spatialization、occlusion、reverb、modulation 与 source-data override 定义 factory、device 生命周期、per-source init/release 和 process callback。Ray acoustics 不应只在 manifest 中声明 dependency，也不能把 IR bytes 存入 manager 后称为集成。

AudioEditorModule、SoundSubmixEditor 和 SoundWave asset definition 注册 asset action、preview/import/reimport、graph/node/connection factory、property customization、component broker 与对称 teardown。一个只含 33 个 descriptor 和五份 Space surface 的插件不构成 Audio Editor。

### 5.2 Godot AudioServer

AudioServer 显式拥有 driver、speaker mode、device、mix count、bus/channel/effect/send、meter 与 playback list。playback state 用 fade-to-pause/delete 避免 render thread 直接释放 owner，并把主线程 deallocation 与 audio mixing 分开。AudioStreamPlayer3D 会持续提交位置、bus volume、attenuation、area 与 doppler。Editor 侧同时有 stream waveform/preview 和可 undo/redo、save、add/duplicate/move/effect 的 bus editor。

这些事实要求 Zircon 的 source/listener/volume DTO 必须由 World system 每帧投影到 audio generation，不能只存进 manager map；Mixer Console 也必须编辑真实 bus graph 和 transaction，而不是展示静态 surface。

### 5.3 Fyrox Sound

Fyrox SoundEngine 直接拥有 output 或 headless manual render；多个 SoundContext 作为 sound scene，分别持有 source、listener、renderer、bus graph 与 distance model。render loop 会真正把 source 混入 bus/effect graph。StreamingBuffer 有增量 decoder、固定 block、seek/rewind，HRTF renderer 和 effects 也进入实际输出路径。

Fyrox 规模小于 Unreal，但它提供了很重要的下界：即使不追求全部高级功能，scene、streaming、spatial、bus/effect 与 output 仍需端到端连通。Zircon 当前“合同更多、真实路径更少”不能视为架构领先。

### 5.4 Bevy Audio

Bevy AudioPlugin 把播放、sink 创建、finished cleanup 与 spatial position update 安装到 ECS schedule；asset 未加载时保留 queued handle，加载后才附加 sink。它明确只实现简单 stereo panning、没有 HRTF，因此能力声明与行为范围一致。

Zircon 已经暴露更宽的 HRTF、occlusion、convolution、timeline 与 graph contract，却没有产品 scheduler 和 render integration。Bevy 的适用结论是“较小范围也必须真实接线并诚实声明”，不是降低 Zircon 的目标。

### 5.5 Unity Graphics 参考边界

本地 Unity Graphics 镜像不包含 Unity Audio。SRP Core package.json 与 assembly definition 只可用于核对 runtime/editor/optional package 的依赖和编译边界；不能从图形 package 推断 Unity 音频实现，也不能把缺少参考源码当作 Zircon 完成证据。

## 6. P0 归属：本文不新增最高优先级 finding

| 已证实现象 | Canonical owner | 本篇责任 |
|---|---|---|
| 音频运行时、Kira graph/routing、spatial、streaming、device 与 telemetry 本体 | Runtime08B；两个 open failure handoff | 记录它们如何阻断 Sound package 的产品闭环，不复制 P0 |
| Sound Clip/Mixer/Spatial/Timeline authoring 与 audition | Editor17、Editor25/50 | 记录本包 command/resource/controller/catalog 断点 |
| 音频 importer/source snapshot/artifact/cook | Plugins07 | 只定义 Sound consumer 必须消费的 artifact contract |
| source/dist/native ABI 与 lifecycle parity | Plugins01 | 定义 Sound parity gate，不重造通用 loader/ABI P0 |
| first-party catalog/profile/capability closure | Plugins06、Runtime42、App01 | 记录 ordinary App 0 provider、catalog root 1/feature 0/editor 0 的单包影响 |
| clock、stable handle、operation、shutdown、evidence | Runtime22/24 及 O02/O07/O11/O14/O15/O16 | 要求 Sound 消费共享 owner，不建立音频私有替代品 |

只要 Sound 保持 beta/partial、optional feature default-off，且普通产品不把不可达能力显示为 ready，本篇不因功能量差距新增 P0。任何 profile、Editor 或 release 将 Sound、ray acoustics 或 timeline 标为 stable/complete/required/default-enabled 前，必须先关闭父 owner 的 P0 并通过本篇 G01-G32。

## 7. P1：Package、Catalog、Capability、Editor 与 Distribution 闭环

| ID | 当前差距 | 需要重构 |
|---|---|---|
| NSND-P1-001 | 默认 target-client 不启用 first-party runtime plugins，普通 App 有 Sound 合同却没有 provider | 由 project/profile 生成 resolved provider closure；required Sound 缺 linked provider 时构建或启动 fail-close |
| NSND-P1-002 | target-editor-host 未链接 base Sound runtime，Editor preview 与游戏运行可落在不同能力集合 | Editor Host 与导出产品消费同一 SoundActivationPlan，仅允许 target policy 有显式差异 |
| NSND-P1-003 | first-party runtime catalog 只收集 Sound root，不收集 ray/timeline optional provider | 生成 root-feature provider graph，逐 feature 记录 requested、linked、admitted、activated 与 degraded receipt |
| NSND-P1-004 | first-party editor catalog 不链接 Sound Editor 或两个 feature editor | editor closure 由同一 package selection 解析；缺 provider 时隐藏 capability 并给 typed 原因 |
| NSND-P1-005 | generated export 能显式链接 feature，普通 host 使用另一条 composition authority | source、generated、library、native 全部复用 ProviderResolver 与 activation receipt |
| NSND-P1-006 | manager factory固定使用 SoundConfig::default()，manifest options 没有成为 effective config | 建立 validated EffectiveSoundConfig，保留 source、override、range、target 与 generation |
| NSND-P1-007 | native projection module 名称/行为与 source package 不等价，dist 只输出 metadata | 实现完整 source/native Sound provider bridge、state/lifecycle/quiesce，或撤销 NativeDynamic 可用声明 |
| NSND-P1-008 | base dist command/event/state/save/restore/unload/host-ready 均为空 | 对 source runtime 可观察行为定义 ABI projection；无法表达的能力必须显式 unsupported |
| NSND-P1-009 | ray runtime 和 timeline runtime 只注册空 ModuleDescriptor，却发布 beta capability | capability admission 要求 executable provider、dependency handle 和同代 receipt；否则保持 unavailable |
| NSND-P1-010 | 两个 feature editor 只注册 descriptor/capability，且产品 catalog 不链接 | 只有存在真实 controller/document/preview 时注册 Editor extension；空壳不发布可见能力 |
| NSND-P1-011 | Sound Editor 33 个 command descriptor 无 factory/handler | 每项绑定 typed payload、permission、transaction/job、cancel/deadline 和 terminal receipt |
| NSND-P1-012 | 五份 ZUI 除三个 output 按钮外均以 Space 表示 Mixer、Source、Listener、Volume 与 Acoustic UI | 用真实 data provider/controller/view model 替换占位；无后端区域保持隐藏或明确 unavailable |
| NSND-P1-013 | 没有 Audio Clip toolkit、waveform/import settings、Mixer document、undo/redo、audition 或安全 preview | 接入 Editor document/transaction/artifact 基础，形成 open-edit-undo-save-reopen-preview 闭环 |
| NSND-P1-014 | live-output controller 只在测试 fake 中构造，产品没有 factory 或 owner | 由 Editor runtime bridge 创建 generation-bound controller，处理 disconnect、stale、timeout 与重连 |
| NSND-P1-015 | catalog 暴露 default/music_sfx/spatial_room，但后两者含不可执行 effect/send | preset 可见性绑定 compiled graph qualification；不可执行 preset 在 catalog admission 阶段拒绝 |
| NSND-P1-016 | 插件声明 Sound、ray 和 timeline 能力，没有“已链接但降级/已声明但不可执行”状态 | 统一 CapabilityTruthReceipt，UI、profile、export 和 telemetry 只消费 receipt |

## 8. P1：Audio Render、Mixer、Scene 与 Resource 闭环

| ID | 当前差距 | 需要重构 |
|---|---|---|
| NSND-P1-017 | graph validator 接受 effect、advanced control 和 pre-effect send，Kira compiler 随后统一拒绝 | 生成单一 executable capability matrix；validation、compile、preset 与 Editor palette 使用同一数据 |
| NSND-P1-018 | Kira send track 未形成完整目标 submix processing，三项路由门仍为红色 | 重建拓扑与 gain law，证明 direct/send/master/parent update 在真实 frame capture 下准确 |
| NSND-P1-019 | 结构性 graph mutation 在任何 active playback 存在时被禁止 | 编译不可变 graph generation，在 audio block 边界原子交换，旧 generation 按 callback fence 退役 |
| NSND-P1-020 | 一个全局 Mutex 串行 device、playback、graph、timeline、dynamic event 与状态查询 | 分离 game-thread control、audio command queue、render-owned state 与 observation snapshot；禁止 callback 等普通锁 |
| NSND-P1-021 | Sound 没有 Runtime Scene/World system，AudioSource/Listener/Volume component 不会自动实例化、更新或销毁 | 建立 per-world AudioWorldSystem，消费 scene delta、transform、pause/time scale 与 entity generation |
| NSND-P1-022 | source_environment 的 attenuation、cone、doppler、volume、occlusion、HRTF、convolution 无生产调用点 | 将环境评估编译为 per-source render parameters，在真实 block 路径执行并具备 budget/oracle |
| NSND-P1-023 | engine/dsp 与 engine/filter 只在 cfg(test) 编译，容易形成“已实现 effect”的假外观 | 将真实 DSP 接入 production renderer并资格化，或迁移为明确 prototype/testkit，不得支撑 capability |
| NSND-P1-024 | source/listener/volume update 多数只改变 manager storage，没有 audible generation receipt | 每次 mutation 返回 accepted generation，audio callback 发布 applied generation 与 sample clock |
| NSND-P1-025 | External 与 Synth input 被公共类型接受，但 Kira binding 拒绝 | 为每种 source kind 提供真实 decoder/generator contract、backpressure 与 lifecycle；未实现类型不进入 public admission |
| NSND-P1-026 | Clip 完整解码为静态 PCM，mono 复制为 stereo，超过双声道被拒绝 | 建立 compressed/cooked artifact、streaming blocks、channel layout、decoder pool、seek 与 residency budget |
| NSND-P1-027 | 没有 voice allocation、priority、virtualization、steal、concurrency group 或 inaudible policy | 建立 bounded VoiceAllocator，按 priority/audibility/cost 决策并发布 steal/virtualize reason |
| NSND-P1-028 | completion 只在 API 调用时轮询，update_source 通过 stop/restart 生效 | audio callback 发布 bounded lifecycle events；参数更新走 sample/block-aligned command，不重启 voice |
| NSND-P1-029 | 创建 source 即可发 gameplay perception，即使 output 未启动或声音不可听 | 由 audible/virtualized policy 生成 perception receipt，区分 intended、started、audible 与 completed |
| NSND-P1-030 | device ID 基于 display name，后端只接受 mono/stereo，设备能力和 identity 不稳定 | 建立 backend-qualified stable device identity、layout/rate/buffer negotiation 与 migration policy |
| NSND-P1-031 | device config 会先停 Kira 再改状态，失败时没有自动 restart、LKG 或 supervisor | DeviceSupervisor 以 prepare-swap-commit-retire 处理 hotplug、default change、stall、loss 与 fallback |
| NSND-P1-032 | backend 在 output 停止时仍可报告 Ready，callback 计数器没有生产写入 | 状态机区分 unavailable/ready/stopped/starting/running/recovering/failed，并由 callback 产生 telemetry |

## 9. P1：Event、Automation、Acoustics、Lifecycle 与 Qualification 闭环

| ID | 当前差距 | 需要重构 |
|---|---|---|
| NSND-P1-033 | dynamic event pending Vec 无界、每次 drain all，handler clone/sort，且没有产品 pump | 采用 bounded MPSC/priority queue、frame budget、stable handler snapshot、drop reason 与唯一 scheduler |
| NSND-P1-034 | dynamic event callback ABI 没有 deadline/cancel/unload quiescence 或 generation fence | 绑定 plugin generation、in-flight lease、deadline/cancel 和 terminal disposition 后才允许卸载 |
| NSND-P1-035 | automation 在 output inactive 时只改 metadata，active Kira path明确返回 M5 unsupported | automation compiler 产出 sample-clock curve commands；active/inactive 使用相同语义和 receipt |
| NSND-P1-036 | timeline 依赖 caller 手工传 delta，没有 audio sample clock、seek/scrub/catch-up policy | TimelinePlayer 绑定 clock domain/epoch，支持 sample-accurate schedule、seek、pause、loop 与 missed-event policy |
| NSND-P1-037 | ray provider 只存 IR descriptor/sample/status，没有 scene geometry、ray scheduler、bake/update 或 source binding | optional provider消费 acoustic scene generation，生成 versioned IR field 并原子挂入 mixer |
| NSND-P1-038 | convolution 是朴素 frame×tap 循环且跨 block history/partition/latency contract不完整 | 实现 partitioned convolution、tail state、IR transition、CPU budget、fallback 与误差/延迟 oracle |
| NSND-P1-039 | HRTF preview 使用简化 delay/gain，未形成 dataset validation、interpolation、tail 与 multi-listener policy | 建立 cooked HRTF profile、bounded per-source state、interpolation和切换/退化资格 |
| NSND-P1-040 | occlusion/ray query 与 geometry owner、thread handoff、stale result policy 未接 | acoustic query 按 world/scene generation异步批处理，过期结果丢弃并发布 latency/quality |
| NSND-P1-041 | source asset pipeline 没有消费统一 cooked audio artifact、dependency/provenance 和 last-good | 与 Plugins07 对接 AudioClipArtifact；runtime 只消费已 admission 的 platform artifact和stream index |
| NSND-P1-042 | source、manager、device、graph、feature 没有统一 shutdown/drain/reload 顺序或 generation handle | 建立 SoundRuntimeInstance lifecycle：stop admission、drain command/callback、retire voice/graph、release device、unload provider |
| NSND-P1-043 | source/export/native/editor 没有相同行为与错误矩阵 | 建立同一 scenario corpus 的 registration、playback、graph、device、feature、failure parity |
| NSND-P1-044 | 1,038 个测试文件高度结构碎片化，include_str/contains 对源码形状的约束超过产品行为 | 合并 support parser 与 source-shape test，保留少量结构门，把预算迁移到 executable contract tests |
| NSND-P1-045 | 没有真实设备、不同采样率/layout/buffer、hotplug、callback overrun、OOM 和长时 soak | 建立硬件/虚拟后端矩阵、fault injection、deterministic offline oracle 与机器可读结果 |
| NSND-P1-046 | 没有 callback CPU、alloc/lock、voice、stream miss、XRUN、latency、memory 的统一观测 | 音频线程写入无分配 ring telemetry，由 Runtime/Editor diagnostics 异步读取 |
| NSND-P1-047 | beta/partial 升级没有产品 workload、correctness、failure、memory 或 latency 阈值 | 将 maturity 绑定 SoundQualificationRecord 与 BuildSet/source/device/workload identity |
| NSND-P1-048 | 当前没有证据支持“表现和性能优于当前 Unreal”的结论 | 先完成同内容、同设备、同声道/采样率、同 DSP/voice/streaming 负载的统计基准，再允许比较 |

## 10. P2：工程级能力扩展

| ID | 能力差距 | 目标方向 |
|---|---|---|
| NSND-P2-001 | 没有 MetaSound 类 procedural audio graph、typed node compiler 与 runtime graph VM | 在基础 mixer/voice 稳定后建立确定性 audio graph IR、compiler、artifact 和 bounded renderer |
| NSND-P2-002 | 没有 ambisonics、soundfield、object audio 或平台 spatial endpoint | 建立 channel/object/soundfield format negotiation 与可插拔 spatial backend |
| NSND-P2-003 | 没有 microphone/capture/voice-chat 与 echo/noise processing | 由独立 capture/communications owner接入权限、AEC/NS、jitter、privacy 与 network |
| NSND-P2-004 | 没有 portals、diffraction、多路径传播和可扩展 acoustic LOD | 在 acoustic scene 与 ray budget 成熟后增加分级传播模型 |
| NSND-P2-005 | 没有 GPU/硬件 ray acoustics 或异步 bake farm | 以同一 IR artifact contract接入可选 backend，不把 GPU 存在当作结果正确 |
| NSND-P2-006 | 没有平台 codec、hardware DSP、console/mobile 低功耗策略 | 建立 target codec/cook/profile 和平台资格，不在 runtime 动态猜测 |
| NSND-P2-007 | 没有 loudness、dialogue normalization、hearing accessibility 与 localization variation | 建立内容元数据、bus policy、QA meter 与用户设置链 |
| NSND-P2-008 | 没有 offline render、stem bounce、deterministic mix export | 复用 compiled graph 和 sample clock提供 headless render 与 artifact receipt |
| NSND-P2-009 | 没有 remote audio profiler、trace capture 和 block replay | 以有界 trace、隐私过滤、BuildSet和设备身份支持离线分析 |
| NSND-P2-010 | 没有第三方 spatial/reverb/codec 插件认证与故障隔离 | 定义 audio extension ABI、实时安全规则、sandbox/timeout、certification corpus |
| NSND-P2-011 | 没有 adaptive music state、transition、stem quantization 与 cinematic sequencer 集成 | 建立 music clock、tempo map、quantized transition和Timeline共享artifact |
| NSND-P2-012 | 没有 large-world audio partition、区域流送、voice cluster 与跨 world travel | 在 per-world owner、stable identity 和 asset streaming 完成后实现分区加载与平滑 handoff |

## 11. 目标架构与所有权

### 11.1 产品组合

~~~text
ProjectManifest + TargetProfile + LinkedProviders
  -> SoundActivationPlan
       package/version/features/options
       provider/build/schema generations
       target/device policy
       required/degraded admission
  -> SoundActivationReceipt
       requested -> linked -> admitted -> activated -> running
       or typed unavailable/degraded reason
~~~

ordinary App、Editor Host、generated export 与 NativeDynamic 必须消费同一 resolver。source/runtime、feature/editor 与 dist 可以有不同实现载体，但不能有不同能力事实。

### 11.2 Runtime owner

| Owner | 唯一职责 | 不拥有 |
|---|---|---|
| SoundRuntimeInstance | host/world Sound 生命周期、service handle 与 generation publication | 不直接执行 audio callback |
| AudioDeviceSupervisor | device identity、negotiation、start/stop、hotplug、loss/recovery | 不拥有 World component |
| AudioRenderService | callback、sample clock、bounded command queue、voice、compiled mixer、final block | 不读 Editor 或 project source |
| AudioWorldSystem | AudioSource/Listener/Volume component projection、transform、pause、world teardown | 不解码或持有设备 |
| AudioResourceService | cooked clip/stream/HRTF/IR artifact、decoder、residency 与 last-good | 不拥有 gameplay entity |
| MixerCompiler | authored graph 到 immutable executable generation | 不在 callback 中解析动态 schema |
| AcousticProvider | acoustic scene 到 occlusion/IR parameter generation | 不绕过 mixer/source generation |
| TimelineAudioProvider | timeline artifact 到 sample-clock commands | 不建立第二时间 authority |
| AudioObservationStream | callback-safe telemetry 到 Runtime/Editor diagnostics | 不反向控制 renderer |

控制线程只写 bounded commands 或 immutable snapshots；audio render thread只读取其拥有的 generation，在 block 边界应用更新。旧 voice、graph、artifact 与 extension 只有在 callback fence 和 in-flight lease 清零后退役。

### 11.3 Editor owner

Sound Editor 应以 AudioClipDocument、MixerDocument、AcousticPreviewSession 和 TimelineAudioTrackDocument 为真实 owner。所有 command 进入 transaction/job，所有 preview 使用当前 source revision编译出的同一 artifact 和 mixer generation；live view只读 AudioObservationStream。未安装 runtime provider时，Editor仍可做无损 source 编辑，但 audition、device control 与 live mixer必须 fail-close。

### 11.4 Artifact 与 identity

AudioClipArtifact 至少绑定 source hash、import settings、decoder/cook version、target codec、channel layout、sample rate、seek/stream table、dependency与content hash。MixerGraphArtifact、HrtfProfileArtifact、ImpulseResponseArtifact 和 TimelineAudioArtifact分别有独立 schema/version，但都通过 BuildSet、artifact generation 和 retirement receipt连接。禁止用 display name、Vec index 或裸 Arc 作为跨线程/跨重载身份。

## 12. 分层重构里程碑

### M0 · Truth Freeze 与红门保留

- 保持 Sound、ray、timeline 为 beta/partial/default-off；
- 把 ordinary App 0 provider、catalog root 1/feature 0/editor 0 变成机器可读 activation test；
- 保留 Kira routing 的 3 项红测，不以 mock 或禁用测试关闭；
- 将不可执行 preset、effect、source kind 和 feature capability fail-close。

### M1 · Composition、Config 与 Lifecycle

- 建立 SoundActivationPlan/Receipt 和同一 ProviderResolver；
- 让 manifest options 生成 EffectiveSoundConfig；
- 收敛 source/export/native/editor registration parity；
- 建立 SoundRuntimeInstance、generation handle、shutdown/drain/unload 顺序。

### M2 · Audio Thread、Device 与 Mixer Generation

- 分离控制线程、bounded command queue、audio-owned state 和 observation；
- 建立 DeviceSupervisor 与 prepare/swap/recover/LKG；
- 编译 immutable mixer graph并在 block 边界原子切换；
- 关闭三项 Kira routing 红门，证明 effect/send/master gain。

### M3 · World、Voice 与 Artifact Streaming

- AudioWorldSystem 接入 source/listener/volume scene lifecycle；
- AudioResourceService 消费 Plugins07 cooked artifact；
- 实现 streaming decoder、residency、channel layout、seek、voice allocator 与 completion queue；
- gameplay perception绑定 audible receipt。

### M4 · Spatial、Acoustic、Automation 与 Timeline

- 将 attenuation/cone/doppler/volume/HRTF/occlusion/convolution接入真实 render block；
- 建立 acoustic scene generation、bounded query 与 partitioned convolution；
- automation/timeline统一到 sample clock；
- optional provider通过 typed extension接入并可安全卸载。

### M5 · Editor 产品闭环

- 链接 Sound Editor 与 feature editor；
- 实现 Audio Clip/Mixer/Acoustic/Timeline document、transaction、compiler、audition；
- 33 个 command 全部具有 executable factory或从公开 catalog 移除；
- live controller消费真实 observation并处理断连/重连/stale。

### M6 · Qualification 与竞争基准

- 建立 offline oracle、真实设备矩阵、hotplug/fault/OOM/soak/callback deadline；
- 记录 voice/stream/XRUN/alloc/lock/latency/memory；
- source/editor/export/native在同一 corpus上做行为 parity；
- 只有 correctness 与可靠性通过后才运行对 Unreal/Fyrox/Godot/Bevy 适用范围的同条件比较。

## 13. 资格门

| Gate | 验收内容 |
|---|---|
| G01 | 默认 Client、Editor Host、generated export 与 NativeDynamic 都生成 SoundActivationPlan/Receipt；required provider缺失时 fail-close |
| G02 | root、ray、timeline runtime/editor provider closure可重建，requested/linked/admitted/activated 状态一致 |
| G03 | plugin.toml options 全部进入 EffectiveSoundConfig，非法值在 manager/device创建前拒绝 |
| G04 | source/dist/native 对 registration、state、lifecycle、error、unload 和 capability 行为 parity |
| G05 | SoundRuntimeInstance shutdown 会停止 admission、drain command/callback、退役 graph/voice/artifact 后释放 device/provider |
| G06 | audio callback 无普通 Mutex 等待、无动态分配、无文件/decoder阻塞；超时和 XRUN可观测 |
| G07 | device stable identity、layout/rate/buffer协商、default change、hotplug、stall、loss和LKG恢复通过 |
| G08 | graph validator、compiler、preset、Editor palette 使用同一 executable capability matrix |
| G09 | direct/send/master/parent gain 在真实 frame capture 中通过已登记 4 项 routing gate |
| G10 | active playback期间可在 block boundary 原子切换 graph generation，旧 generation 安全退役 |
| G11 | default、music_sfx、spatial_room所有可见 preset均能真实执行，或不可执行项不进入 catalog |
| G12 | AudioWorldSystem对source/listener/volume create/update/remove/world unload和entity generation有产品测试 |
| G13 | attenuation、cone、doppler、volume、occlusion、HRTF 与 convolution 都能改变真实输出 block并有数值 oracle |
| G14 | External/Synth未实现时 admission拒绝；实现后具备 backpressure、cancel、seek与lifecycle |
| G15 | cooked clip支持 stream/residency/seek/channel layout；不再要求完整 PCM 常驻和mono复制stereo |
| G16 | VoiceAllocator在超额 source下按priority/audibility/cost虚拟化或steal，并发布原因 |
| G17 | playback completion由callback有界发布，不依赖外部API轮询；普通参数更新不stop/restart |
| G18 | gameplay perception区分 intended/started/audible/virtualized/completed，不为未输出声音伪报 audible |
| G19 | dynamic event queue有bytes/items/time预算、公平性、drop reason、deadline/cancel与唯一pump |
| G20 | plugin dynamic event callback绑定generation/in-flight lease，卸载前quiesce且late callback失效 |
| G21 | automation和timeline在active/inactive输出下语义一致，绑定sample clock、epoch、seek/pause/loop |
| G22 | ray provider消费真实 acoustic scene generation并产生可安装 IR，不只保存descriptor/status |
| G23 | convolution具备跨block tail、partition、IR transition、latency/CPU budget和fallback |
| G24 | HRTF profile有cook/validation/interpolation/tail/multi-listener policy与退化状态 |
| G25 | AudioClipArtifact绑定source/settings/compiler/target/dependency/content hash并支持last-good/retirement |
| G26 | Sound Editor catalog真实链接；Audio Clip、Mixer、Acoustic、Timeline有document/transaction/save/reopen/compiler |
| G27 | 33 个公开 command 均有 executable factory和terminal receipt；无执行体command不再可见 |
| G28 | 五份 surface 无业务 Space 占位，数据来自document/runtime observation，断连时typed degraded |
| G29 | audition与游戏运行消费相同 artifact、mixer和provider generation；preview/export有golden parity |
| G30 | 测试报告绑定source/build/device/workload identity，structure test不能替代device/behavior qualification |
| G31 | offline oracle、真实设备矩阵、hotplug/fault/OOM、voice pressure、stream miss、长时 soak 全部机器可读 |
| G32 | 同内容同设备同采样率/layout/DSP/voice/streaming条件下记录CPU、callback deadline、latency、RSS与音频误差后才允许竞争结论 |

## 14. 禁止的临时修补

1. 不得仅在默认 App Cargo feature中硬塞 Sound，而继续保留 ordinary/generated/native 三套 resolver。
2. 不得把 feature ModuleDescriptor 非空、capability 字符串存在或 source file 存在当作可执行证据。
3. 不得为通过测试而把不可执行 effect/preset/source kind静默降级成 no-op。
4. 不得在 audio callback 中获取全局 manager Mutex、分配 Vec、解码文件、排序 handler 或调用 Editor/plugin 用户代码。
5. 不得通过每帧 stop/restart source 来实现参数变化。
6. 不得继续用完整 PCM 常驻、mono复制stereo作为 streaming 与 channel-layout 的最终方案。
7. 不得让 ray/timeline feature建立第二 SoundManager、第二 mixer、第二 clock 或私有 observation authority。
8. 不得用 mock backend、source-shape test或固定 telemetry数字证明真实设备、空间音频或 Editor 完成。
9. 不得关闭或绕过 docs/plans/zircon_plugins/02 中三项 Kira routing 红门。
10. 不得在同条件 correctness、failure、soak 和统计基准完成前宣称优于 Unreal。

## 15. 状态与产出边界

| 项目 | 状态 |
|---|---|
| Sound 全包、普通产品装配、optional feature、Editor、dist 与 tests E3 静态审查 | review_complete |
| Unreal、Godot、Fyrox、Bevy、Unity Graphics适用参考核对 | review_complete |
| 新增 finding | 0 P0 / 48 P1 / 12 P2 |
| 资格门 | 32 |
| Production / tests 修改 | 无 |
| Cargo、真实设备、Editor、NativeDynamic、soak、性能验证 | 本轮未运行 |
| 实施状态 | pending |

本篇完成的是证据冻结、owner边界、重构顺序与验收合同，不是 Sound 实现完成证明。后续实施必须从 M0 的产品 truth 与既有红门开始，按 M1-M6推进；任何 source drift 都要求重算 fingerprint、provider closure、测试库存与 capability truth。
