---
title: Editor Audio / Sound 当前工作树 Mixer、Source/Listener/Volume、设备试听与声学调试复审
category: zircon_editor
report_id: Editor228
review_date: 2026-08-30
baseline_head: working-tree
baseline_epoch: 2026-08-30
verification_head: working-tree
verification_epoch: 2026-08-30
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/139-editor-sound-audio-clip-mixer-routing-effect-spatial-acoustic-timeline-audition-current-source-review.md
  - docs/plans/optimize/zircon_editor/93-editor-sound-audio-clip-mixer-routing-effect-spatial-acoustic-timeline-audition-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/17-sound-audio-clip-mixer-routing-effect-spatial-acoustic-timeline-authoring-review.md
related_code:
  - zircon_plugins/sound/editor
  - zircon_plugins/sound/features/*/editor
  - zircon_editor/src/core/editor_extension
  - zircon_editor/src/scene/viewport
  - zircon_runtime/src/core/framework/sound
  - zircon_plugins/sound/runtime/src/service_types
  - zircon_plugins/first_party_editor_catalog
  - zircon_plugins/first_party_runtime_catalog
plan_sources:
  - docs/plans/optimize/zircon_editor/139-editor-sound-audio-clip-mixer-routing-effect-spatial-acoustic-timeline-audition-current-source-review.md
  - docs/plans/optimize/zircon_editor/93-editor-sound-audio-clip-mixer-routing-effect-spatial-acoustic-timeline-audition-product-integration-current-source-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/AudioEditor/Private/AudioEditorModule.cpp
  - dev/UnrealEngine/Engine/Source/Editor/AudioEditor/Private/SoundSubmixEditor.cpp
  - dev/UnrealEngine/Engine/Source/Editor/AudioEditor/Private/AssetTypeActions/AssetDefinition_SoundWave.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/AudioComponent.h
  - dev/godot/editor/audio/audio_stream_editor_plugin.cpp
  - dev/godot/editor/audio/editor_audio_buses.cpp
  - dev/godot/scene/audio/audio_stream_player.cpp
  - dev/godot/scene/3d/audio_stream_player_3d.cpp
  - dev/Fyrox/fyrox-sound/src/engine.rs
  - dev/Fyrox/fyrox-sound/src/bus.rs
  - dev/bevy/crates/bevy_audio/src/audio.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/package.json
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
tooling_scope: excluded_by_user_request
---

# Editor228 · Audio / Sound 当前工作树复审

## 1. 结论

Sound Editor 已登记 authoring drawer、mixer view、acoustic debug view、inspector customization、29 条 operation path 和 live-output controller；但这些是 extension metadata 与模型投影，不是可审计的编辑器产品。当前没有声音 document/controller/job/preview-world/cook artifact 的完整 ownership，也没有把操作 path 绑定到真实 operation factory、事务、revision conflict 或终态 receipt。

## 2. 复审边界

Sound editor 与五份 ZUI、feature editor 共 14 个文件、1,386 行、52,892 bytes、8 个测试、0 个 ignored；fingerprint 为 `b4c69c33a0f66c16be3ab4644cae82c206aecda449d23f7aeea7f04d72f3aa03`。Sound runtime 依赖及 editor core 只作为集成证据检查，不改变本报告的编辑器统计口径。

## 3. 发现

### 3.1 注册与 owner 缺口

1. `SoundEditorPlugin::register_editor_extensions` 只调用 `register_authoring_extensions`、binding 和 inspector customization（`zircon_plugins/sound/editor/src/plugin.rs:34-72`）；没有 document type、asset editor、preview session、operation handler、viewport overlay provider 或 shutdown owner。
2. `first_party_editor_catalog` 仍只 materialize Navigation/Neural（`zircon_plugins/first_party_editor_catalog/src/catalog.rs:41-54`）；Sound editor 即使 crate 存在，也没有普通 editor host 的 provider closure 证据。runtime catalog 也依赖可选 feature 才返回 Sound plugin，测试 manifest 不等于产品装配。
3. `SoundRayTracedConvolutionEditorFeature` 与 `SoundTimelineAnimationEditorFeature` 只构造 `EditorPluginDescriptor`/capability（各自 `.../editor/src/plugin.rs:8-25`），没有 UI、provider、artifact、runtime bridge 或可回收状态。

### 3.2 ZUI 是占位布局，不是可用工具

1. `mixer_console.zui` 只有 device picker、Refresh/Start/Stop 三个 Button；track strip、send matrix、effect chain、sidechain、automation lane、dynamic event registry 全是 `Space`（`zircon_plugins/sound/editor/mixer_console.zui:13-110`）。按钮 route 也只是 `sound.output.device.*`，没有 graph transaction 或 selection binding。
2. `acoustic_debug.zui` 的 toolbar、listener/source layer、volume panel、occlusion ray panel、IR cache panel 全是 `Space`（`.../acoustic_debug.zui:13-48`），没有 toggle、filter、world/scene revision、query budget、stale/error 表达。
3. `audio_source.drawer.zui`、`audio_listener.drawer.zui`、`audio_volume.drawer.zui` 只提供布局容器，未声明 input/slider/menu/curve editor 与 operation payload。表面上有 inspector customization，用户不能可靠地修改组件并获得事务结果。
4. Sound editor operation descriptors 以字符串 schema 注册（`zircon_plugins/sound/editor/src/authoring_bindings.rs:23-85,128-202`），但仓库没有同等规模的 handler registry、schema decoder、selection authority、undo/redo 或 capability failure UI。注册数量不能作为实现数量。

### 3.3 Live output 只有局部模型

1. `SoundEditorLiveOutputController` 能枚举设备、聚合 diagnostics、发出 configure/start/stop（`zircon_plugins/sound/editor/src/live_output/controller.rs:12-79`），但其 trait 直接依赖 runtime manager，没有 editor operation/job/receipt、后台任务和 stale snapshot 处理。
2. controller 的测试 fake 对 `render_output_device_block`、`available_output_backends`、`pull_output_backend_callback` 使用 `unimplemented!`（`.../controller.rs:303-314`）；这说明测试只覆盖行模型，不覆盖真实输出协议。
3. UI 没有把 output status 的 callback sequence、underrun、actual latency、backend unavailable 和 recovery state 展示为可操作状态。Runtime 端 latency 仍是估算，editor 不能声称“试听质量已验证”。

### 3.4 编辑器与 runtime 数据边界

1. runtime `SoundMixerGraph` 是可序列化 DTO，Sound manager 的 graph revision 与 Kira handle 只在 runtime 内部维护；editor 没有 document revision、preview graph、diff、compile receipt 或 failed apply 的 rollback UI。
2. mixer graph structural edit 在 active playback 时被拒绝（`zircon_plugins/sound/runtime/src/kira_bridge/manager/graph.rs:55-63`）。编辑器没有“pending graph / preview / apply at block boundary”状态，只能让用户停止声音或收到通用错误。
3. 声学 debug 没有正式 overlay provider。Editor core 已有 provider registry/toggle/retirement 机制，但 Sound plugin 没有登记 provider；因此 debug ZUI 的 `Space` 不会从 scene/query snapshot 得到稳定绘制数据。
4. 试听 source/listener/volume 没有 preview world 和隔离资源；直接绑定 live manager 会让编辑器改动污染运行时世界，且无法在未启动设备时预览 stream、automation、HRTF 或 IR。

## 4. 编辑器重构路线

### E0：产品 owner 与文档模型

1. 建立 `SoundAssetDocument`、`MixerGraphDocument`、`SoundComponentDocument` 与 `PreviewWorld`，每个带 source revision、schema version、dirty state、diagnostic 和 transaction id。
2. 建立 editor operation registry：字符串 path 只作为稳定标识，真正执行需 typed payload、selection/world scope、expected revision、undo record、terminal receipt。没有 handler 的 path 不得注册为 available。
3. 将 Sound、Ray-Traced Reverb、Timeline Track 的 feature editor 接入同一 owner，定义 provider registration、capability closure、unload/retire 和 failure handoff。

### E1：可用 authoring surface

1. 用真实 controls 替换 `Space`：track tree/drag reorder、send matrix、effect chain、parameter curve、device picker、source clip/range、listener selection、volume shape/priority、automation timeline。
2. 每个 control 绑定 typed command，并显示 pending/accepted/failed/stale/conflict；图结构编辑走 preview compile 和可回滚 apply，不因 active playback 直接丢操作。
3. 声学 debug 使用正式 `ViewportOverlayProvider`，消费 scene revision + acoustic snapshot，支持 layer toggle、world filter、ray budget、cache age、fallback reason。

### E2：试听与验证

1. preview world 拥有独立 SoundManager、stream/cache budget、virtual device 和 deterministic block clock；live output 仅在用户明确选择后通过 command bridge 连接。
2. 设备面板展示实际 backend、sample rate、block、latency、callback sequence、XRUN、reconnect；Start/Stop/Configure 变成可观察 job，而不是同步按钮副作用。
3. 资源 editor 显示 codec、duration、channels/layout、resident/streaming policy、cook variants、IR/HRTF readiness 和 import receipt。

## 5. 编辑器资格门（当前均 Fail）

| 门 | 通过条件 |
|---|---|
| ED-1 catalog | 普通 editor host 能 materialize Sound + feature provider，关闭/重载可回收 |
| ED-2 operation | 每个可见 command 有 typed handler、transaction、revision conflict、undo/redo 和 terminal receipt |
| ED-3 mixer authoring | graph/send/effect/automation 在 active preview/live playback 下可预览、编译、回滚 |
| ED-4 component authoring | source/listener/volume drawer 可编辑真实字段并同步 scene document，不污染 live world |
| ED-5 acoustic overlay | overlay provider 从稳定 snapshot 绘制 listener/cone/volume/ray/IR，并标注 stale/error |
| ED-6 audition | preview/live output 的设备恢复、stream seek、长音频、HRTF/IR 和 callback diagnostics 有自动化证据 |

本报告只做 review 和计划记录，没有修改 Sound 生产代码；Tooling 仍按用户要求排除。
