---
title: Editor Sound / Audio Clip / Mixer / Routing / Effect / Spatial / Acoustic / Timeline / Audition 当前源码复审
category: zircon_editor
report_id: Editor139
review_date: 2026-08-26
baseline_head: 8e56165c4c789416c328898d3d8937d934b52efa
verification_head: 8e56165c4c789416c328898d3d8937d934b52efa
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/17-sound-audio-clip-mixer-routing-effect-spatial-acoustic-timeline-authoring-review.md
  - docs/plans/optimize/zircon_editor/93-editor-sound-audio-clip-mixer-routing-effect-spatial-acoustic-timeline-audition-product-integration-current-source-review.md
related_runtime_owner:
  - docs/plans/optimize/zircon_runtime/08b-audio-runtime-review.md
  - docs/plans/optimize/zircon_runtime/99zn-runtime-audio-sound-clip-streaming-device-mixer-bus-effect-spatial-occlusion-reverb-timeline-event-voice-chat-editor-product-integration-current-source-review.md
related_editor_owners:
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md
  - docs/plans/optimize/zircon_editor/07-play-session-process-pie-game-view-live-edit-recovery-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/14-animation-sequence-graph-state-machine-timeline-curve-preview-compiler-authoring-review.md
related_plugin_owner:
  - docs/plans/optimize/zircon_plugins/11-first-party-sound-source-runtime-editor-dist-catalog-mixer-spatial-reverb-timeline-product-integration-review.md
  - docs/plans/zircon_plugins/02-sound.md
  - docs/plans/zircon_plugins/02/failure-2026-07-19-kira-send-frame-capture-routing.md
related_code:
  - zircon_plugins/sound/editor
  - zircon_plugins/sound/features/timeline_animation_track/editor
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/editor
  - zircon_plugins/first_party_editor_catalog
  - zircon_editor/src/core/asset/type_registry
  - zircon_editor/src/core/document
  - zircon_editor/src/core/editing
  - zircon_editor/src/core/jobs
  - zircon_editor/src/ui/host/editor_operation_dispatch.rs
  - zircon_runtime/src/asset/assets/sound.rs
  - zircon_plugins/audio_importer
  - zircon_plugins/asset_importers/audio
  - zircon_plugins/opus_importer
  - zircon_plugins/sound/runtime
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/AudioEditor
  - dev/UnrealEngine/Engine/Source/Editor/MovieSceneTools/Private/TrackEditors/AudioTrackEditor.cpp
  - dev/UnrealEngine/Engine/Plugins/Runtime/Metasound/Source/MetasoundEditor
  - dev/Fyrox/editor/src/audio
  - dev/Fyrox/editor/src/scene/commands/sound_context.rs
  - dev/Fyrox/fyrox-sound/src/buffer/streaming.rs
  - dev/godot/editor/audio
  - dev/godot/editor/import/audio_stream_import_settings.cpp
  - dev/godot/modules/interactive_music/editor
  - dev/bevy/crates/bevy_audio
  - dev/Graphics/Packages
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Editor139 · Sound / Audio Clip / Mixer / Spatial / Timeline 当前源码复审

## 1. 结论

当前 Zircon Editor 仍没有工程级 Sound authoring 产品。仓库已经具有宽而类型化的 Audio/Sound runtime contract、Kira/CPAL backend、严格 WAV 校验、Symphonia 解码、Mixer/Automation/Timeline DTO、三类 Scene component descriptor，以及可保留的 `SoundEditorLiveOutputController`。问题不是“没有任何音频代码”，而是这些底座没有形成默认产品中可打开、可执行、可撤销、可保存、可试听、可观察、可恢复和可确定关闭的纵向闭环。

产品断点在当前源码中仍然确定：`ResourceKind::Sound`只有显示元数据和 placeholder thumbnail，没有 Audio Clip toolkit；first-party Editor catalog 仍只装配 Navigation 与 Neural，没有 Sound 或两个 Sound optional feature。Sound 插件注册了 33 条业务 operation descriptor，却没有对应 operation factory，公开 dispatch 仍会进入 typed `MissingFactory`。五份 ZUI 仍为 **301 行、43 个 node、29 个 `Space`、3 个 Button、3 条 route、0 个 provider binding**；除 Mixer 的 Refresh/Start/Stop 外，其余业务区域没有可执行产品，三个 route 也没有生产 controller/factory。

`SoundEditorLiveOutputController`已经从单文件拆为 `live_output/{controller,model}.rs`，但产品行为没有闭合：全仓生产引用仍只有重导出，没有 host factory、activation owner、document/view-model binding、instance/device generation、disconnect/reconnect 或 shutdown 接点；三个行为测试仍全部依赖 fake manager。Sound 资产双击不能打开 Audio Clip editor，也没有 waveform、transport、scrub、loop/marker、import settings、reimport diff、audition session 或 provider 缺失时的只读降级面。

当前 working tree 的 Runtime 音频改动包含值得保留的局部工程优化：graph compile 缓存 `TrackHierarchyIndex`，automation mutation 使用定向 `Arc` COW，timeline/automation 增加容量与预校验，low-pass 改为 in-place，WAV 常见 channel layout 直接映射，Symphonia 限制初始 reserve 并复用 scratch。这些变化没有关闭任何 Editor17/93 产品条目：最终 PCM 仍完整累积到 `SoundAsset.samples: Vec<f32>`，clip 仍以 resident `LoadedClip`/Kira `StaticSoundData`为主，没有 page/stream/residency/voice allocator；active Kira automation 仍返回 `UnsupportedAdvancedFeature`；timeline 仍由调用者传入 `delta_seconds`；Runtime telemetry 字段没有 callback writer；Sound runtime 中也没有生产 `AudioWorldSystem`/`SceneAudioBridge` 注册与调用链。

本轮重判 **5 项 P0 全部 Open、60 项 P1 全部 Open、12 项 P2 全部 Open；32 项 Editor 资格门全部 Fail**。现有 descriptor、DTO、fake-manager test、ignored microbenchmark 与局部优化不能证明产品完成。没有同资产、同设备、同格式、同负载、同失败条件的 correctness、听觉质量与原始性能证据，因此禁止声称 Zircon Sound 的性能或表现优于 Unreal。

## 2. 审查边界、统计与 currentness

### 2.1 冻结范围

统计口径为 `2026-08-26T16:07:55+08:00` 当前 working tree 的去重物理文件、物理行、非空行、bytes、Rust `#[test]` 与 `#[ignore...]` 声明。Sound Runtime 为递归全量，不以文件名抽样；reference 只选择与本报告判断直接相关的实现文件。

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 本轮证据 |
|---|---:|---|
| Sound Editor 与两个 optional feature editor | **25 / 1,607 / 1,436 / 60,362 / 10 / 0** | plugin、33 operation、live-output model/controller、五份 ZUI、registration/fake-manager tests 全量 |
| Editor shared/product boundary | **160 / 29,694 / 26,937 / 1,000,596 / 229 / 16** | asset type/toolkit、document、editing/history、job、extension/store、operation dispatch、catalog 与 App |
| Runtime、asset、importer 与 provider downstream | **1,336 / 32,560 / 29,596 / 1,122,993 / 420 / 12** | Sound runtime 递归全量，加 Audio/Sound framework、SoundAsset、三 importer、optional runtime 与 catalog |
| Zircon selected union | **1,521 / 63,861 / 57,969 / 2,183,951 / 659 / 28** | 上述三组去重集合；Runtime 文件数大不等于 Editor 产品可达 |
| Unreal selected | **9 / 11,316 / 9,498 / 388,813 / 0 / 0** | AudioEditor、SoundWave open/reimport、SoundSubmix editor、MovieScene AudioTrack、MetaSound editor/validation |
| Godot selected | **10 / 4,181 / 3,508 / 160,751 / 0 / 0** | AudioStream preview、Audio Bus、import settings、interactive music editor 的 cpp/header |
| Fyrox selected | **7 / 2,105 / 1,915 / 80,468 / 1 / 0** | AudioPanel/bus/preview/commands 与 generic/streaming sound buffer |
| Bevy selected | **4 / 1,196 / 1,045 / 39,695 / 1 / 0** | ECS AudioPlayer/PlaybackSettings/output/sink 生命周期 |
| Unity Graphics package boundary | **4 / 228 / 228 / 12,832 / 0 / 0** | Core/HDRP/URP/ShaderGraph manifests，只用于限定非 Audio owner |
| 五引擎 reference selected union | **34 / 19,026 / 16,194 / 682,559 / 2 / 0** | 当前本地参考实现去重集合 |
| Plan/docs evidence | **11 / 3,751 / 2,786 / 367,388 / 0 / 0** | engine-wide、Editor17/93、Runtime08b/139、Plugins11/02 与两个 open handoff |
| 全部证据 union | **1,566 / 86,638 / 76,949 / 3,233,898 / 661 / 28** | Zircon、reference 与 owner docs 的当前物理集合 |

### 2.2 currentness 与限制

- baseline 与 verification HEAD 均为 `8e56165c4c789416c328898d3d8937d934b52efa`；报告读取当前 working tree，而不是只读取 HEAD blob。
- selected Zircon 边界内有 **88 条** modified/deleted/untracked status。它们属于用户或其他 Session；本报告不回退、不覆盖，也不把在途改动写成已集成。
- 25 个 Sound Editor/optional editor 文件的产品形态与 Editor93 一致；新增实质集中于 Runtime 性能/校验与共享 Editor owner，不存在新 Sound toolkit、factory、provider 或 production controller owner。
- Kira send frame-capture routing handoff仍为 Open；当前源码优化没有提供新的 current-source production frame capture 证据，本报告不越权宣告关闭。
- 按用户要求未查询、轮询、等待或实时跟踪协调器；Tooling 不在本轮范围。
- 本轮只做源码 review 与文档记录，未运行 Cargo、真实 Editor、声卡、software-null audition、import/cook、save/reopen、PIE、hotplug、fault、scale、soak、profiling或竞争 benchmark。

### 2.3 Owner 边界

- Editor139 负责 Audio Clip/Mixer/Acoustic/Timeline 的 authoring document、operation、transaction、audition、toolkit、view model 与 truthful product surface。
- Runtime139 负责 device/callback、mixer execution、voice、stream/residency、World bridge、DSP/spatial/acoustics、event/timeline clock 与 shutdown；本报告只记录下游阻断，不重复登记 Runtime P1。
- Editor02/04/05/09 分别持有共享 document、asset/import、typed inspector 与 job authority；Sound 必须消费这些 authority，不能复制一套局部框架。
- Editor14 持有通用 Timeline/Curve authoring；Sound track provider只能注册类型化扩展，不得另造平行时间系统。
- Plugins11/02 持有 runtime/editor/dist/catalog/package activation 闭包；manifest 或 capability 字符串本身不构成产品 readiness。

## 3. 当前产品链事实

| 链路 | 当前源码事实 | 判定 |
|---|---|---|
| App/Catalog | `target-editor-host`只有 Sound contracts；first-party Editor catalog只有 Navigation/Neural | Open |
| Asset open | builtin registry显示 Sound/SND/asset-sound；`builtin_toolkit`不覆盖 Sound | Open |
| Plugin surface | 注册 Mixer、Acoustic Debug、generic drawer 和三类 inspector customization | Partial foundation |
| Operation | 20 Mixer/output/debug + 7 AudioSource + 3 AudioListener + 3 AudioVolume = 33 descriptor | Metadata only |
| Dispatch | Sound helper只`register_command`；无 factory，公开调用进入`MissingFactory` | Open |
| ZUI | 5文件/301行/43 nodes/29 Spaces/3 Buttons/3 routes/0 provider | Open |
| Live output | controller/DTO存在；生产 caller只有重导出，覆盖只使用 fake manager | Open |
| Audio Clip | 无 toolkit/document/waveform/transport/import settings/reimport/audition | Open |
| Mixer | 无 source asset/document/strip/meter/send/effect/sidechain/automation UI/compiler receipt | Open |
| Scene audio | inspector业务位都是Space；无camera/listener policy、gizmo、overlay、World bridge receipt | Open |
| Optional feature | Timeline 与 ray-convolution editor/runtime只发布 descriptor/capability/empty module | Open |
| Streaming/residency | `SoundAsset`持有全PCM；decode完整累积；clip cache无page/unload/residency/voice allocator | Open |
| Automation/clock | Kira active automation显式Unsupported；timeline仍由`delta_seconds`推进 | Open |
| Telemetry/World | 字段只初始化/reset/project；callback不写，World system无生产注册/caller | Open |
| Runtime progress | hierarchy cache、COW、capacity、in-place filter、layout/scratch是局部真实优化 | Partial foundation |
| Evidence | Editor tests验证registration/route/fake manager，不启动默认产品、不执行33命令 | Open |

## 4. 必须保留的真实底座

1. 保留 Runtime-owned neutral Audio/Sound contracts、typed IDs/errors、Kira/CPAL 唯一 backend owner；Editor 不得创建第二套 audio engine。
2. 保留严格 WAV RIFF/fmt/data/extensible-mask 校验、多声道 layout、Symphonia codec path、bounded initial reservation 与 scratch reuse。
3. 保留 Mixer track/send/effect、source/listener/volume、automation/timeline、device/status 与 acoustic DTO；未执行的合同继续 fail-close。
4. 保留 `SoundEditorLiveOutputController` 与 serializable snapshot/action report，将其升级为 generation-bound product controller。
5. 保留 Editor operation factory/transaction/document/job/diagnostic 基础；Sound 必须成为共享 authority 的 consumer。
6. 保留五份 ZUI 的稳定 asset/control ID 作为迁移输入；provider 未就绪时必须 disabled/Unavailable，不能用 `Space` 宣称功能。
7. 保留 active automation 的明确 Unsupported 和 Opus DiagnosticOnly 行为，直到真实实现；禁止 silent no-op。
8. 保留 graph hierarchy cache、targeted COW、in-place filter 等已测性能优化，但 correctness/产品闭环仍是其前置资格门。
9. 保留 Sound `Beta/Partial` 成熟度；G01-G32 通过前不得提升为 shipping-ready。

## 5. P0：产品虚假可达与纵向闭环断裂

| ID | 状态 | 当前问题 | 必须重构为 |
|---|---|---|---|
| SND2-P0-01 | Open | 默认 Editor catalog 无 Sound，Sound asset 无 toolkit，manifest editor module 不等于产品可达 | profile-qualified activation plan原子绑定runtime/editor provider、resources、toolkit、controller与receipt |
| SND2-P0-02 | Open | 33条公开 operation 只有 descriptor，没有 factory/handler | 绑定typed payload、permission、document/history、prepare/apply/revert/cancel与terminal receipt；否则删除 |
| SND2-P0-03 | Open | live-output controller无产品owner，device/callback telemetry不是真实observation | Editor runtime bridge持有instance/device generation、bounded observation、recovery与shutdown fence |
| SND2-P0-04 | Open | 五份 visible surface 以29个`Space`承载 Mixer/component/acoustics 能力 | 真实toolkit/pane/provider；未到位区域隐藏或给typed unavailable reason |
| SND2-P0-05 | Open | 无transactional authoring、audition与scene-to-sound revision闭环 | `source revision -> document transaction -> artifact -> runtime generation -> receipt -> projection` |

## 6. P1：工程级完整性差距

### 6.1 产品装配、能力与生命周期

| ID | 状态 | 当前差距 | 重构要求 |
|---|---|---|---|
| SND2-P1-01 | Open | Sound asset只有presentation/placeholder，无toolkit/read-only fallback | 注册Audio Clip toolkit；provider缺失时给可诊断只读/repair面 |
| SND2-P1-02 | Open | Runtime catalog可选Sound，Editor catalog无Sound，provider closure不对称 | 同一selection解析runtime/editor/feature closure及generation |
| SND2-P1-03 | Open | registration成功只证明descriptor存在，不验证ZUI/controller依赖 | publication前解析resource/schema/provider/factory，失败整批回滚 |
| SND2-P1-04 | Open | Mixer/Acoustic复用generic surface，无typed pane payload | 专用pane descriptor、document key、controller factory与restore state |
| SND2-P1-05 | Open | generic drawer命名为Mixer，drawer/inspector/toolkit ownership混杂 | 拆asset toolkit、global mixer pane、scene inspector、debug overlay owner |
| SND2-P1-06 | Open | capability只控制可见性，不证明backend executor readiness | readiness receipt覆盖registration/resource/factory/backend/dependency |
| SND2-P1-07 | Open | 33个schema ID只是字符串，无schema registry/budget/migration | 版本化typed schema，decode前执行bytes/depth/items budget与migration |
| SND2-P1-08 | Open | tests只断言ID、route和注册集合 | 增加bootstrap、asset-open、dispatch、save/reopen、disable/reload产品测试 |
| SND2-P1-09 | Open | disable/unload无audition/device/telemetry drain gate | revoke admission后取消job/voice，drain callback/observation，再撤UI |
| SND2-P1-10 | Open | capability名不含实现层级/backend支持矩阵 | 发布可执行matrix与degraded reason，控件只显示真实支持项 |

### 6.2 Audio Clip、导入、waveform 与试听

| ID | 状态 | 当前差距 | 重构要求 |
|---|---|---|---|
| SND2-P1-11 | Open | `SoundAsset`仍序列化完整interleaved PCM `Vec<f32>` | 拆`AudioClipSource`、derived artifact与runtime pages/residency |
| SND2-P1-12 | Open | 缺source URI/hash、importer/settings/version/tool revision provenance | provenance进入asset、artifact key、reimport diff与diagnostic |
| SND2-P1-13 | Open | 缺codec/quality/sample-rate/channel/remix/downmix policy | versioned ImportSettings + target capability validation |
| SND2-P1-14 | Open | 缺trim、normalize、loudness target、true peak/clipping policy | bounded analysis/build job输出解释报告与artifact identity |
| SND2-P1-15 | Open | 缺loop region、cue marker、beat/BPM/tempo/sample metadata | stable marker IDs与合法区间/迁移/roundtrip合同 |
| SND2-P1-16 | Open | 无waveform/peak-envelope artifact与后台generation | Editor09 job + DDC key + cancel/generation fence + multires cache |
| SND2-P1-17 | Open | 无play/pause/stop/seek/scrub/loop audition session | 独立audition owner消费同一runtime artifact/mixer/provider generation |
| SND2-P1-18 | Open | 三个audio/opus importer存在重叠authority | 唯一选择规则、优先级冲突诊断与format capability truth |
| SND2-P1-19 | Open | bounded reserve/scratch后仍完整decode增长，无stream/cancel | decoder pages、backpressure、deadline/cancel、seek table、memory budget |
| SND2-P1-20 | Open | import不等待waveform/audition/catalog revision acknowledgement | receipt绑定source/settings/artifact/catalog generation与last-good |

### 6.3 Mixer graph、routing、effect 与 automation

| ID | 状态 | 当前差距 | 重构要求 |
|---|---|---|---|
| SND2-P1-21 | Open | Mixer没有versioned source asset或document owner | `SoundMixerSource` + document + canonical serialization/migration |
| SND2-P1-22 | Open | track CRUD没有stable identity与引用修复 | generational TrackId；删除/重命名prepare/reject/fixup transaction |
| SND2-P1-23 | Open | send cycle/feedback/channel规则无Editor定位诊断 | shared compiler返回object/pin/path定位和修复建议 |
| SND2-P1-24 | Open | effect操作无typed schema/version/range/backend检查 | versioned effect registry与executable capability matrix |
| SND2-P1-25 | Open | Mixer无callback事实驱动peak/RMS/clip meter | RT写有界lock-free observation；UI节流并显示generation/staleness |
| SND2-P1-26 | Open | mute/solo/bypass/gain/pan没有交互合同 | typed control、gesture merge、undo、automation touch/latch策略 |
| SND2-P1-27 | Open | send matrix/sidechain/effect rack仍为`Space` | virtualized控件消费document snapshot与compiler diagnostic |
| SND2-P1-28 | Open | preset无资产类型、diff、partial apply/unsupported策略 | versioned preset asset，preview diff后transactional apply |
| SND2-P1-29 | Open | Editor无parameter registry；Kira automation拒绝；clock非sample authority | shared registry + sample clock/epoch/seek/loop + backend ack |
| SND2-P1-30 | Open | 无source revision到compiled graph/device generation可见ack | prepare immutable graph、block-boundary swap、stale reject、LKG |

### 6.4 Scene component、spatial audio 与 acoustics

| ID | 状态 | 当前差距 | 重构要求 |
|---|---|---|---|
| SND2-P1-31 | Open | AudioSource七个`Space`，Apply/field operation均无factory | typed multi-selection editor、validation、transaction、dirty/save |
| SND2-P1-32 | Open | AudioListener无active唯一性和viewport/PIE camera ownership | 明确Editor/Play camera、多listener策略和切换receipt |
| SND2-P1-33 | Open | AudioVolume无shape/priority/overlap/crossfade语义 | typed volume model、gizmo、resolver preview与runtime parity |
| SND2-P1-34 | Open | component依赖字符串schema，customization无typed address/version | 接共享reflection/property schema与migration，拒绝unknown/stale field |
| SND2-P1-35 | Open | 无生产`AudioWorldSystem/SceneAudioBridge`消费三类component | per-World owner按entity/component generation create/update/remove |
| SND2-P1-36 | Open | spatial transform缺world transform/handedness/unit/velocity合同 | canonical transform projection与固定scene oracle |
| SND2-P1-37 | Open | attenuation/cone/doppler无gizmo与听觉/视觉一致性 | Editor gizmo与Runtime共享参数/曲线并做容差测试 |
| SND2-P1-38 | Open | occlusion无query schedule/budget/smoothing/fallback | physics query broker + generation + cost/latency/drop observation |
| SND2-P1-39 | Open | IR/convolution字段无asset/probe/bake/residency链 | versioned IR asset、bounded bake、partition/tail budget、LKG fallback |
| SND2-P1-40 | Open | Acoustic Debug五个`Space`，无真实overlay provider | 只显示真实listener/source/volume/ray/IR generation及成本 |

### 6.5 Live output、preview runtime 与遥测

| ID | 状态 | 当前差距 | 重构要求 |
|---|---|---|---|
| SND2-P1-41 | Open | controller无ProductHost构造、instance解析或owner lease | bridge由activation receipt构造并持有generation lease |
| SND2-P1-42 | Open | 三个output route没有桥到controller action | typed factory调用bridge，结果进入operation receipt/diagnostic |
| SND2-P1-43 | Open | device picker/status是`Space`，没有snapshot projection | 显示default/selected/available/format/latency/state/error/stale |
| SND2-P1-44 | Open | configure/start/stop没有并发状态机/idempotency/deadline | serialized supervisor，重复/冲突请求有typed terminal result |
| SND2-P1-45 | Open | rendered/callback/xrun字段没有真实callback writer | callback单调计数、sequence/gap/xrun/last-error来源可追踪 |
| SND2-P1-46 | Open | 无hotplug/default-device订阅、LKG与recovery | stable device ID、renegotiation、fallback/retry/backoff状态机 |
| SND2-P1-47 | Open | audition/ambient preview/PIE output所有权未定义 | session/voice/device owner矩阵，停止session不误杀其他voice |
| SND2-P1-48 | Open | graph readiness与device start顺序无Editor gate | start前等待artifact/provider/graph/device generation一致 |
| SND2-P1-49 | Open | controller错误停留在字符串report | typed error code/context/action接Console/Notification/journal |
| SND2-P1-50 | Open | close/disable无voice/callback/worker/device fence | admission close、cancel、drain、retire、release顺序与timeout policy |

### 6.6 Timeline、动态事件、测试与性能治理

| ID | 状态 | 当前差距 | 重构要求 |
|---|---|---|---|
| SND2-P1-51 | Open | Timeline optional editor只有descriptor/capability | 注册stable track/section/clip schema、toolkit、factory、compiler、preview |
| SND2-P1-52 | Open | 无Sound timeline document/runtime observation投影 | UI只读document/playback snapshot，不显示固定Ready/sample |
| SND2-P1-53 | Open | Runtime按`delta_seconds`推进，缺sample/frame/seek epoch | audio sample clock authority + rate/rounding/loop/discontinuity合同 |
| SND2-P1-54 | Open | dynamic event registry是Mixer空节点，无handler truth | versioned event/payload catalog、readiness、preview、bounded result |
| SND2-P1-55 | Open | ray-convolution feature仍是manifest/descriptor壳 | provider、geometry generation、bake job、IR artifact、overlay、unload |
| SND2-P1-56 | Open | fake manager/source-shape/registration test被当能力证据 | 启动真实catalog/host/operation/software-null/real device的分层测试 |
| SND2-P1-57 | Open | 缺codec恶意输入/allocation bomb/超长文件/cancel corpus | bytes/time/memory/fd/thread预算与残留artifact断言 |
| SND2-P1-58 | Open | 缺software-null/default/specified/hotplug/device-loss矩阵 | 可重复offline lane + lab real-device lane + fault injection |
| SND2-P1-59 | Open | 缺万clip/千track/长waveform/高频meter预算 | canonical workload记录CPU/memory/I/O/UI latency/deadline miss |
| SND2-P1-60 | Open | ignored microbench只证明局部优化，无法跨引擎比较 | build/source/device/workload绑定raw evidence，correctness/quality前置 |

## 7. P2：工程级能力扩展

| ID | 状态 | 扩展差距 | 目标 |
|---|---|---|---|
| SND2-P2-01 | Open | 无MetaSound级可编程audio graph | versioned node registry、compiler、audition、diagnostic、sandbox |
| SND2-P2-02 | Open | 无interactive music segment/transition系统 | tempo map、quantization、condition、transition/fade、preview |
| SND2-P2-03 | Open | 无Sound Cue/random/container/playlist资产家族 | typed composite asset、deterministic random、voice policy |
| SND2-P2-04 | Open | 无响度/频谱/相位/多声道分析 | offline/realtime analyzer、标准单位、exportable report |
| SND2-P2-05 | Open | 录音/麦克风/voice chat/AEC无owner | permission/device/session/network/privacy/diagnostic闭环 |
| SND2-P2-06 | Open | 高级空间格式/对象音频无协商 | platform/device format negotiation与fallback |
| SND2-P2-07 | Open | 无offline render/bounce/stem export | deterministic render graph、artifact identity、cancel/retry |
| SND2-P2-08 | Open | 无audio golden/diff/感知回归 | waveform/spectrum/loudness/latency oracle与阈值 |
| SND2-P2-09 | Open | 字幕/视觉提示/无障碍metadata未进工作流 | stable cue linkage、localization、validation、preview |
| SND2-P2-10 | Open | 第三方DSP/plugin缺sandbox与兼容治理 | ABI/version/thread/budget/crash isolation/signature |
| SND2-P2-11 | Open | 分布式cook/waveform/loudness cache未接DDC | content-addressed execution、provenance、poison recovery |
| SND2-P2-12 | Open | 无协作编辑/merge/reviewable audio graph diff | stable IDs、semantic diff、conflict model、review artifact |

## 8. 历史台账重判

| 历史台账 | 当前重判 | 说明 |
|---|---:|---|
| Editor17/93 P0 | **5 Open / 0 Partial / 0 Closed** | catalog/toolkit、33 commands、live owner、29 Spaces、transaction/audition/scene loop均未闭合 |
| Editor17/93 P1 | **60 Open / 0 Partial / 0 Closed** | Runtime局部优化没有满足任何Editor端到端退出条件 |
| Editor17/93 P2 | **12 Open / 0 Partial / 0 Closed** | graph/music/analyzer/voice/spatial/offline/golden/accessibility/DSP/DDC/collaboration均无产品owner |
| Editor17/93 Gates | **32 Fail / 0 Partial / 0 Pass** | 类型、descriptor、fake test或microbench不单独构成可执行产品门的Partial |

Runtime细粒度状态继续由 Runtime139 持有。Kira send frame-capture routing 与 Sound owner inventory drift 两份 handoff 仍由各自 owner 管理；Editor139不阻塞等待，也不越权关闭。

## 9. 参考引擎差异裁决

### 9.1 Unreal Engine

- `AudioEditorModule.cpp`装配Audio editor/customization/factory并维护注册生命周期；Zircon只有静态descriptor。
- `AssetDefinition_SoundWave.cpp`通过`OpenAssets`打开SoundWave，并可退到`FSimpleAssetEditor`；Zircon Sound无toolkit。
- `SoundSubmixEditor.cpp`有graph/details tab、selection、create/delete、`FScopedTransaction`和dirty；Zircon Mixer无document/graph/transaction。
- `ReimportSoundFactory.cpp`保存source path、处理格式变化、执行atomic reimport、失效compressed data并`MarkPackageDirty`；Zircon缺provenance与reimport acknowledgement。
- MovieScene AudioTrack 多处用transaction新增音频轨/section并可选择audio clock；MetaSound Editor有graph transaction、validation和live audition。它们是独立工程子系统，不是一组operation字符串。

### 9.2 Godot

- AudioStream editor生成后台waveform，支持play/pause/stop、鼠标seek、duration/position和accessible controls。
- Audio Bus Editor读取真实bus/channel/VU/peak，编辑volume/solo/mute/bypass/send/effect，并通过`EditorUndoRedoManager`记录do/undo。
- Import Settings 与 Interactive Music editor承载trim/loop/quality/transition等工作流；Zircon没有等价Audio Clip/music document。

### 9.3 Fyrox

- Rust Editor已有AudioPanel、bus tree、parent routing、effect list、renderer/distance model/HRTF resource与command execute/revert。
- Audio preview真实驱动Play/Pause/Stop/Rewind/Seek；scene sound command提供execute/revert而非固定成功反馈。
- `StreamingBuffer`通过decoder-backed block refill支持seek/rewind；Zircon仍把导入结果完整驻留在PCM DTO/静态clip数据。

### 9.4 Bevy

- Bevy用`AudioPlayer + PlaybackSettings + AudioSink/SpatialAudioSink`把asset readiness、entity lifecycle、pause、volume和cleanup接到ECS system。
- emitter/listener的`GlobalTransform`变化会更新空间sink，完成时按mode remove/despawn。其Editor能力很少，但运行路径和限制真实，不能为Zircon空authoring产品背书。

### 9.5 Unity Graphics 边界

本地`dev/Graphics`选择集只有SRP Core/HDRP/URP/ShaderGraph package manifest，不含Unity Audio Editor/Mixer源码。它只能证明该checkout不是Sound owner；本报告不从渲染仓库推断Unity Audio能力，也不声称已完成Unity Audio源码比较。

## 10. 目标架构

```text
Project/Profile SoundSelection
  -> SoundActivationPlan(runtime + editor + optional providers + resources)
  -> SoundEditorRuntimeBridge(instance/device generation lease)

AudioClipSource + ImportSettings
  -> validate/decode/analyze jobs
  -> AudioClipArtifact(pages + seek + waveform + metadata)
  -> AudioClipDocument/Toolkit
  -> AuditionSession

SoundMixerSource
  -> SoundMixerDocument + reversible transactions
  -> SoundMixerCompiler
  -> ImmutableMixerGraphArtifact
  -> block-boundary runtime apply receipt

SceneAuthoringDocument(AudioSource/Listener/Volume)
  -> typed component transactions + gizmos
  -> Runtime AudioWorldSystem generation
  -> bounded meter/acoustic observations

TimelineDocument + SoundTrackProvider
  -> sample-clock-qualified artifact
  -> runtime playback/event receipts
```

关键约束：source/document/artifact/runtime/editor observation必须使用不同类型；每个异步结果携带source revision、artifact key、runtime/device generation和cancel token；UI只能显示document或observation事实，不能显示固定Ready或零telemetry。

## 11. 分层重构里程碑

| 里程碑 | 依赖 | 交付物 | 退出条件 |
|---|---|---|---|
| M0 Capability truth | MVP F0-F2 | activation plan、catalog/provider/resource/factory preflight | 缺一项整能力fail-close，无残余菜单/toolkit |
| M1 Audio Clip source | M0 + asset/import | source/settings/provenance、bounded decoder、pages/seek/waveform | 长音频不全PCM驻留；reimport可解释可取消 |
| M2 Clip toolkit/audition | M1 + document/job | toolkit、transport、scrub、loop/marker、audition | save/reopen与software-null/真实设备基本矩阵通过 |
| M3 Mixer document | M0 + transaction | stable track/effect/send IDs、transaction、save/recovery | 33命令中Mixer路径可执行或删除 |
| M4 Compiler/runtime ack | M3 + Runtime | validator、immutable graph、generation swap、meter | active graph、stale reject、RT安全通过 |
| M5 Live output | M0 + device supervisor | controller factory、picker、telemetry、hotplug/LKG | start/stop/failure/reconnect/shutdown矩阵通过 |
| M6 Scene authoring | M2-M5 + Scene | typed inspectors、gizmos、listener policy、World bridge | create/update/remove/unload无泄漏且视听一致 |
| M7 Acoustics | M6 + physics/resource | occlusion broker、IR/probe/bake/residency、overlay | budget/fallback/generation/oracle通过 |
| M8 Timeline/events | M2-M4 + Editor14 | track provider、sample clock、event browser/preview | seek/loop/save/compile/playback parity通过 |
| M9 Qualification | M1-M8 | corpus、fault、scale、soak、quality、competitive raw evidence | G01-G32全Pass后升级成熟度 |

M0不得绕过当前MVP主链。高级Sound实现必须按`.codex/plans/MVP重构方案/00-index.md`允许的F0-F5依赖顺序进入；本轮只记录差异，不实现旁路。

## 12. 资格门

| Gate | 状态 | 必须证明的结果 |
|---|---|---|
| G01 | Fail | Sound启用时默认Editor加载runtime/editor/feature provider、resource、toolkit、controller；缺失fail-close |
| G02 | Fail | Sound asset打开Audio Clip toolkit；provider缺失时只读fallback给typed原因 |
| G03 | Fail | 33条operation逐项有factory/handler/terminal receipt或从catalog删除 |
| G04 | Fail | 五份ZUI不再以`Space`承载业务；控件有data/state/action/error projection |
| G05 | Fail | ImportSettings可序列化/迁移/hash并进入artifact key，reimport diff可解释 |
| G06 | Fail | WAV/MP3/OGG/FLAC/AIFF/Opus声明与backend一致，不支持项预拒绝 |
| G07 | Fail | 畸形/截断/超大音频满足bytes/time/memory预算，取消无残留 |
| G08 | Fail | 长音频通过page/stream/seek/residency，不因完整PCM无界驻留 |
| G09 | Fail | waveform按source/settings generation生成、可取消复用，拒绝stale |
| G10 | Fail | play/pause/stop/seek/loop在offline/software-null/真实设备有terminal result |
| G11 | Fail | Mixer操作支持execute/revert、dirty、save/reload、autosave/recovery |
| G12 | Fail | track删除/重命名transactionally修复或拒绝所有引用 |
| G13 | Fail | compiler定位cycle/missing/unsupported/invalid/channel incompatibility |
| G14 | Fail | graph ack绑定source/artifact/runtime/device generation，拒绝stale commit |
| G15 | Fail | meter来自callback事实；RT无线程锁、allocation、I/O、同步UI调用 |
| G16 | Fail | picker显示真实default/selected/format/latency/state并处理hotplug |
| G17 | Fail | rendered/callback/sequence/xrun/error由backend更新，不支持项Unavailable |
| G18 | Fail | configure/start/stop并发、重复、timeout、失败回滚状态机通过 |
| G19 | Fail | Audition与PIE/runtime voice/device ownership隔离且互不误杀 |
| G20 | Fail | AudioSource typed投影、mixed value、validation、undo/save与runtime一致 |
| G21 | Fail | AudioListener active/pose/HRTF与Editor/PIE camera规则可测试 |
| G22 | Fail | AudioVolume shape/priority/overlap/crossfade与gizmo/runtime一致 |
| G23 | Fail | AudioWorldSystem按entity/component revision create/update/remove，unload无泄漏 |
| G24 | Fail | transform、unit、handedness、velocity timestep通过固定3D oracle |
| G25 | Fail | attenuation/cone/volume gizmo与runtime曲线边界在容差内一致 |
| G26 | Fail | occlusion受budget/smoothing/fallback控制，overlay显示真实成本/generation |
| G27 | Fail | IR/convolution有validation/cook/residency/tail budget/fallback |
| G28 | Fail | Timeline track有stable IDs、sample mapping、scrub/loop/fade、undo/save/playback |
| G29 | Fail | Dynamic event schema/version/handler/editor/preview闭环 |
| G30 | Fail | close/disable/device loss/shutdown取消job并drain late completion |
| G31 | Fail | 万clip、千track/event、长waveform、高频meter满足资源/UI预算 |
| G32 | Fail | 与Unreal/Fyrox/Godot同内容同设备同格式记录质量、原始数据、失败结果 |

## 13. 禁止继续采用的临时实现

1. 禁止用operation字符串、capability、manifest module、菜单或测试注册集合证明功能完成。
2. 禁止给33条路径补no-op factory、固定success、固定Ready或只改control property的executor。
3. 禁止把29个`Space`换成静态Label/Table后关闭条目；数据必须来自真实owner并带generation。
4. 禁止让Editor直接持有Kira/CPAL、audio thread、decoder pool或第二份mixer graph。
5. 禁止让audio callback等待普通`Mutex`、访问Editor/document/scene、无界分配或同步调用UI。
6. 禁止用fake manager、software-null、source-shape test或ignored microbenchmark替代真实产品/设备/质量门。
7. 禁止继续把完整PCM、waveform、IR和graph塞进source DTO；source、artifact、runtime residency必须分离。
8. 禁止以`delta_seconds`手工推进作为shipping timeline authority；必须接sample clock/epoch。
9. 禁止optional feature只有descriptor/capability却在UI宣称可用。
10. 禁止在同任务correctness与质量前宣称性能/表现优于Unreal。

## 14. 完成定义

Editor139只有在以下条件同时满足时才可关闭：M0-M9按依赖完成；5项P0、60项P1、12项P2逐项有current-source证据；G01-G32全部Pass；默认Editor可打开Audio Clip/Mixer/Acoustic/Timeline并完成transaction/save/reopen/audition；Runtime提供唯一可执行backend与generation receipt；真实设备、fault、scale、soak和竞争证据可复现；旧descriptor-only、Space、无factory、全PCM与手工clock产品路径hard cutover且无兼容壳。

本轮没有修改production代码，也没有宣告整体Sound、Editor或Engine目标完成。
