---
title: Editor Sound / Audio Clip / Mixer / Routing / Effect / Spatial / Acoustic / Timeline / Audition 与 Product Integration 当前源码复审
category: zircon_editor
report_id: Editor93
review_date: 2026-08-25
baseline_head: 8ee9411db24b7b4bdaf3fe028194642a7557c0b6
verification_head: 8ee9411db24b7b4bdaf3fe028194642a7557c0b6
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/17-sound-audio-clip-mixer-routing-effect-spatial-acoustic-timeline-authoring-review.md
related_runtime_owner:
  - docs/plans/optimize/zircon_runtime/99zn-runtime-audio-sound-clip-streaming-device-mixer-bus-effect-spatial-occlusion-reverb-timeline-event-voice-chat-editor-product-integration-current-source-review.md
related_editor_owners:
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md
  - docs/plans/optimize/zircon_editor/07-play-session-process-pie-game-view-live-edit-recovery-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/14-animation-sequence-graph-state-machine-timeline-curve-preview-compiler-authoring-review.md
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
  - dev/Graphics
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Editor93 · Sound / Audio Clip / Mixer / Spatial / Timeline 当前源码复审

## 1. 结论

当前 Zircon Editor 没有工程级 Sound authoring 产品。仓库已有宽而类型化的 Runtime Audio/Sound 合同、Kira/CPAL backend、严格 WAV 校验、Symphonia 解码、Mixer/Automation/Timeline DTO、三类 Scene component descriptor，以及可保留的 `SoundEditorLiveOutputController`。Editor 侧也确实注册了 Sound Mixer、Acoustic Debug、AudioSource/AudioListener/AudioVolume inspector customization 和 33 条 operation descriptor。问题不是“完全没写”，而是这些底座没有形成默认产品中可打开、可执行、可撤销、可保存、可试听、可观察和可确定关闭的纵向闭环。

当前产品断点是确定的：`ResourceKind::Sound`只有显示元数据和 placeholder thumbnail，没有 asset toolkit；first-party Editor catalog 仍只链接 Navigation 与 Neural，没有 Sound 或两个 Sound optional feature。Sound 插件的 33 条业务 operation 只调用`register_command`，全仓没有对应 operation factory，dispatch 会进入 typed `MissingFactory`。五份 ZUI 共 **301 行、43 个 node、29 个 `Space`、3 个 Button**；只有 Mixer 的 Refresh/Start/Stop 三个按钮声明 route，其余业务区域全为空，三个 route 也没有 factory/controller binding。

`SoundEditorLiveOutputController`本身是正确的薄边界雏形：它依赖中立 manager trait，能枚举设备、投影状态并 configure/start/stop，失败时也返回 best-effort snapshot。但产品代码对它只有 `lib.rs` 重导出，没有 host factory、document/view-model owner、generation、disconnect/reconnect 或 shutdown 接点；当前三个行为测试全部依赖 fake manager。Sound 资产双击不能打开 Audio Clip editor，也没有 waveform、transport、scrub、loop/marker、import settings、reimport diff、audition session 或只读降级面。

Runtime139 之后的当前工作树增加了真实的局部优化：WAV 常见多声道 layout 走直接映射；Symphonia 初始预分配被限制且复用 packet scratch；automation graph 避免部分全图 clone；timeline 容量与 binding retention、volume low-pass in-place 路径有所改善。这些改动应保留，但没有关闭任何 Editor17 条目：最终音频仍完整累积到 resident `Vec<f32>`，active Kira automation 仍显式返回 Unsupported，timeline 仍由 `delta_seconds` 手工推进，Editor plugin、ZUI、catalog 与 operation factory 自 Runtime139 baseline 后没有变化。

本轮重判 **Editor17 的 5 项 P0 全部 Open、60 项 P1 全部 Open、12 项 P2 全部 Open；32 项 Editor 资格门全部 Fail**。没有动态证据支持 Zircon Sound 的 authoring 完成度、稳定性、性能或听觉表现优于 Unreal；在同资产、同设备、同格式、同负载、同失败条件的可复现 benchmark 与质量 oracle 建立前，禁止作此声明。

## 2. 审查边界、统计与 currentness

### 2.1 冻结范围

统计口径为当前 working tree 的物理文件、物理行、非空行、bytes、Rust `#[test]` 与 `#[ignore]` 声明。fingerprint 对 repository-relative lowercase path 排序，为每个文件拼接 `path + NUL + lowercase(file SHA-256) + LF` 后再取 SHA-256。

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | fingerprint |
|---|---:|---|
| Sound Editor 与两个 optional feature editor | **25 / 1,607 / 1,436 / 60,362 / 10 / 0** | `dede832f3ce7753ab3e08666ffe0bbed9f95ad294a2941441f9e7b7b772eba13` |
| Editor shared/product boundary | **142 / 24,467 / 22,144 / 820,361 / 188 / 8** | `fdda2aa9a77f90284f2d11f9e6427d87a4c88a013b4005cdd7b865076a4fec6e` |
| Runtime、asset、importer 与 provider downstream selected | **313 / 22,325 / 20,371 / 772,170 / 147 / 11** | `798db94f99a3897564eb294eb3afb6c0e1279fe2b808ee8af4a224080b483f11` |
| Zircon selected union | **480 / 48,399 / 43,951 / 1,652,893 / 345 / 19** | `6df17124f34673629cd7783f7f9d5a0d857de0ae692555338c76a47da09f4584` |
| Unreal selected | **14 / 18,470 / 15,738 / 657,765 / 0 / 0** | `9595f180feae352aee782dffe10921b7277dcf53410be7b30a36f683372ead37` |
| Godot selected | **8 / 3,809 / 3,210 / 148,049 / 0 / 0** | `c9c8458feb71458484964ccffe4bca1b599d6cf788968f0bd37a3ec5769a9100` |
| Fyrox selected | **7 / 2,590 / 2,319 / 99,657 / 4 / 0** | `4bbc95806f46be87909c52e280859cf851a77cd4628d8d84ed6f9b99ac4cbc9b` |
| Bevy selected | **5 / 1,314 / 1,152 / 44,490 / 1 / 0** | `2e9963df0cc7a0c57b5d9ff5581186fc6b4dfa0419a545b6ff816308097dac85` |
| Unity Graphics package-boundary selected | **4 / 228 / 228 / 12,832 / 0 / 0** | `5394c50fa2df2200959e7f019234a5b341f52f278f5e65d7df456085503d1db6` |
| 五引擎 reference selected union | **38 / 26,411 / 22,647 / 962,793 / 5 / 0** | `550f2fc72ed08ed1c558d568e8e91eea62081e2d4b0f3c3384c9aa4a9e124027` |

### 2.2 currentness 与限制

- baseline 与 verification HEAD 均为 `8ee9411db24b7b4bdaf3fe028194642a7557c0b6`，commit time 为 `2026-08-25T17:37:22+08:00`。
- 480 个 Zircon selected 文件中有 **64 个**包含用户或其他 Session 的 working-tree 修改/新增；本报告读取物理现状，不回退、不覆盖，也不把在途改动写成已集成。
- 25 个 Sound Editor/optional editor 文件相对 Runtime139 baseline 没有源码 diff；Editor 的产品断点不是旧结论漏掉的新实现。
- 按用户要求未查询、轮询或等待协调器；Tooling 也不在本轮范围。
- 本轮只做源码 review 与文档记录，未运行 Cargo、Editor、真实声卡、software-null audition、asset import/cook、save/reopen、PIE、hotplug、fault、scale、soak、profiling 或竞争 benchmark。

### 2.3 Owner 边界

- Editor93 唯一负责 Audio Clip/Mixer/Acoustic/Timeline authoring document、operation、transaction、audition、toolkit、view model 与 truthful product surface。
- [Runtime139](../zircon_runtime/99zn-runtime-audio-sound-clip-streaming-device-mixer-bus-effect-spatial-occlusion-reverb-timeline-event-voice-chat-editor-product-integration-current-source-review.md) 负责 device/callback、mixer execution、voice、stream/residency、World bridge、DSP/spatial/acoustics、event/timeline clock 与 shutdown；本报告不重复登记其 Runtime P1。
- [Editor02](02-document-transaction-save-autosave-recovery-review.md)、[Editor04](04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md)、[Editor05](05-inspector-reflection-property-authoring-customization-review.md)、[Editor09](09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md) 分别持有共享 document、asset/import、typed inspector 与 job authority。
- [Editor14](14-animation-sequence-graph-state-machine-timeline-curve-preview-compiler-authoring-review.md)持有通用 Timeline/Curve authoring；Sound track provider只能注册类型化扩展，不得另造平行时间系统。

## 3. 当前产品链事实

| 链路 | 当前源码事实 | 判定 |
|---|---|---|
| App/Catalog | `target-editor-host`只带 Sound contracts；first-party Editor catalog feature/依赖/分支只有 Navigation、Neural | Open |
| Asset open | builtin registry显示 Sound/SND/asset-sound；`builtin_toolkit`只覆盖 UI 与 Animation 三类资产 | Open |
| Plugin surface | 注册 Mixer、Acoustic Debug、一个 generic drawer、三类 inspector customization | Partial foundation |
| Operation | 20 Mixer/output/debug + 7 AudioSource + 3 AudioListener + 3 AudioVolume = 33 descriptor | Metadata only |
| Dispatch | Sound helper只`register_command`；33条路径无 factory，调用进入`MissingFactory` | Open |
| ZUI | 5文件/301行/43 nodes/29 Spaces/3 Buttons；三条 inline route 无 executor/controller | Open |
| Live output | controller/DTO 真实存在；生产 caller 只有重导出，行为覆盖只用 fake manager | Open |
| Audio Clip | 无 toolkit/document/waveform/transport/import-settings/reimport/audition | Open |
| Mixer | 无 source asset/document/strip/meter/send/effect/sidechain/automation UI 或 compiler receipt | Open |
| Scene audio | inspector 都是 Space；无 Editor camera/listener policy、gizmo、overlay 或 Runtime World bridge receipt | Open |
| Optional feature | Timeline 与 ray-convolution editor只发布 descriptor/capability，无 extension behavior | Open |
| Runtime progress | layout/scratch/COW/capacity/in-place filter有局部优化；active automation与产品时钟仍未闭合 | Partial foundation |
| Evidence | 10项Editor tests验证registration/route/fake manager，不启动默认产品、不打开资产、不执行33条命令 | Open |

## 4. 必须保留的真实底座

1. 保留 Runtime-owned neutral Audio/Sound contracts、typed IDs/errors、Kira/CPAL 唯一 backend owner；Editor 不得创建第二套 audio engine。
2. 保留严格 WAV RIFF/fmt/data/extensible mask 校验、多声道 layout、Symphonia codec path、bounded initial reservation 与 scratch reuse。
3. 保留 Mixer track/send/effect、source/listener/volume、automation/timeline、device/status 与 acoustic DTO；未执行的合同必须 fail-close。
4. 保留 `SoundEditorLiveOutputController` 和 serializable snapshot/action report，将其升级为 generation-bound product controller，而不是改为按钮直接调用 backend。
5. 保留 Editor operation factory/transaction/document/job/diagnostic 基础；Sound 必须成为这些共享 authority 的 consumer。
6. 保留五份 ZUI 的稳定 asset/control ID 作为迁移输入，但在 provider 未就绪时必须 disabled/Unavailable，不能用 `Space` 宣称功能。
7. 保留当前 active automation 的明确 Unsupported 错误和 Opus DiagnosticOnly 行为，直到真正实现；禁止 silent no-op。
8. 保留 Sound `Beta/Partial` 成熟度，G01-G32 通过前不得提升为 shipping-ready。

## 5. P0：产品虚假可达与纵向闭环断裂

| ID | 状态 | 当前问题 | 必须重构为 |
|---|---|---|---|
| SND2-P0-01 | Open | 默认 Editor catalog 无 Sound，Sound asset 无 toolkit，manifest 的 editor module 不等于产品可达 | profile-qualified Sound activation plan 原子绑定 runtime/editor provider、resources、toolkit、controller 与 capability receipt |
| SND2-P0-02 | Open | 33条公开 operation 只有 descriptor，没有 factory/handler | 每条命令绑定 typed payload、permission、document/history、prepare/apply/revert/cancel 与 terminal receipt；否则从可见 catalog 删除 |
| SND2-P0-03 | Open | live-output controller 没有产品 owner，device/callback telemetry 也未成为可信 observation | Editor runtime bridge 持有 instance/device generation、bounded observation、disconnect/recovery 与 shutdown fence |
| SND2-P0-04 | Open | 五份 visible surface 以29个`Space`承载 Mixer、component 与 acoustics 能力 | 真实 toolkit/pane/provider 到位；未到位的区域隐藏或显示 typed unavailable reason |
| SND2-P0-05 | Open | 无 transactional authoring、audition 与 scene-to-sound revision 闭环 | `source revision -> document transaction -> compiled artifact -> runtime generation -> audition/scene receipt -> truthful projection` |

## 6. P1：工程级完整性差距

### 6.1 产品装配、能力与生命周期

| ID | 状态 | 当前差距 | 重构要求 |
|---|---|---|---|
| SND2-P1-01 | Open | Sound asset type只有presentation与placeholder，无toolkit/read-only fallback | 注册 Audio Clip toolkit；provider缺失时给可诊断只读/repair面 |
| SND2-P1-02 | Open | Runtime base catalog可选择Sound，但Editor catalog无Sound，Editor/target provider closure不对称 | 同一 package selection 解析runtime/editor/feature closure及generation |
| SND2-P1-03 | Open | plugin registration成功只证明descriptor存在，不验证五份ZUI materialize与controller依赖 | publication前解析资源/schema/provider/factory，任一失败整批回滚 |
| SND2-P1-04 | Open | Mixer与Acoustic Debug复用generic authoring surface，无typed pane payload | 定义专用 pane descriptor、document key、controller factory与restore state |
| SND2-P1-05 | Open | generic drawer 被命名为Sound Mixer，drawer/inspector/toolkit ownership混杂 | 拆分 asset toolkit、global mixer pane、scene inspector与debug overlay owner |
| SND2-P1-06 | Open | capability只控制可见性，不证明executor/backend readiness | readiness receipt必须覆盖registration/resource/factory/backend/dependency |
| SND2-P1-07 | Open | 33个schema ID只是字符串，无schema registry、size limit或migration | 版本化typed schema，decode前执行bytes/depth/items budget与migration |
| SND2-P1-08 | Open | tests只断言ID、route与注册集合 | 增加默认bootstrap、asset-open、dispatch、save/reopen、disable/reload产品测试 |
| SND2-P1-09 | Open | plugin disable/unload无audition/device/telemetry drain gate | revoke admission后取消job/voice，drain callback/observation，再撤销UI资源 |
| SND2-P1-10 | Open | capability名不含实现层级与backend支持矩阵 | 发布可执行matrix及degraded reason，Editor控件只显示真实支持项 |

### 6.2 Audio Clip、导入、waveform 与试听

| ID | 状态 | 当前差距 | 重构要求 |
|---|---|---|---|
| SND2-P1-11 | Open | `SoundAsset`仍把完整interleaved PCM放入可序列化`Vec<f32>` | 拆`AudioClipSource`、derived artifact与runtime pages/residency |
| SND2-P1-12 | Open | 缺source URI/hash、importer/settings/version、tool revision等provenance | provenance进入asset、artifact key、reimport diff与diagnostic |
| SND2-P1-13 | Open | 缺codec/quality/sample-rate/channel/remix/downmix policy | versioned ImportSettings + target capability validation |
| SND2-P1-14 | Open | 缺trim、normalize、loudness target、true peak与clipping policy | bounded analysis/build job输出可解释报告与artifact identity |
| SND2-P1-15 | Open | 缺loop region、cue marker、beat/BPM、tempo与sample-accurate metadata | stable marker IDs与合法区间/迁移/roundtrip合同 |
| SND2-P1-16 | Open | 无waveform/peak envelope derived artifact与后台generation | Editor09 job + DDC key + cancel/generation fence + multiresolution cache |
| SND2-P1-17 | Open | 无play/pause/stop/seek/scrub/loop audition session | 独立audition owner消费同一runtime artifact/mixer/provider generation |
| SND2-P1-18 | Open | `audio_importer`、`asset_importers/audio`、`opus_importer`存在重叠authority | 明确唯一选择规则、优先级冲突诊断与format capability truth |
| SND2-P1-19 | Open | bounded initial reserve与scratch reuse后仍完整decode并无界增长，无stream/cancel | decoder pages、backpressure、deadline/cancel、seek table与memory budget |
| SND2-P1-20 | Open | import完成不等待waveform/audition/catalog revision acknowledgement | import receipt绑定source/settings/artifact/catalog generations与last-good |

### 6.3 Mixer graph、routing、effect 与 automation

| ID | 状态 | 当前差距 | 重构要求 |
|---|---|---|---|
| SND2-P1-21 | Open | Mixer没有versioned source asset或document owner | `SoundMixerSource` + `SoundMixerDocument` + canonical serialization/migration |
| SND2-P1-22 | Open | track create/update/delete没有stable identity与引用修复 | generational TrackId，删除/重命名采用prepare/reject/fixup transaction |
| SND2-P1-23 | Open | send graph的cycle/feedback/channel规则没有Editor定位诊断 | shared compiler返回对象/pin/path定位和建议修复 |
| SND2-P1-24 | Open | effect操作无typed node schema、版本、参数范围与backend支持检查 | versioned effect registry与executable capability matrix |
| SND2-P1-25 | Open | Mixer无callback事实驱动的peak/RMS/clip meter通道 | RT写入有界lock-free observation；UI节流读取且显示generation/staleness |
| SND2-P1-26 | Open | mute/solo/bypass/gain/pan等strip没有可交互合同 | typed control state、gesture merge、undo、automation touch/latch策略 |
| SND2-P1-27 | Open | send matrix/sidechain/effect rack仍为`Space` | virtualized真实控件消费document snapshot和compiler diagnostics |
| SND2-P1-28 | Open | preset list/apply无资产类型、diff、partial apply或unsupported策略 | versioned preset asset，preview diff后transactional apply |
| SND2-P1-29 | Open | binding类型存在，但Editor无parameter registry；active automation仍Unsupported，clock仍非sample authority | shared parameter registry + sample clock/epoch/seek/loop语义 + backend ack |
| SND2-P1-30 | Open | 无source revision到compiled graph/device generation的可见ack | prepare immutable graph、block-boundary swap、stale reject与last-good |

### 6.4 Scene component、spatial audio 与 acoustics

| ID | 状态 | 当前差距 | 重构要求 |
|---|---|---|---|
| SND2-P1-31 | Open | AudioSource drawer七个`Space`，Apply/field操作均无factory | typed multi-selection field editor、validation、transaction、dirty/save |
| SND2-P1-32 | Open | AudioListener drawer无active唯一性与viewport/PIE camera ownership | 明确Editor camera、Play camera、多listener策略和切换receipt |
| SND2-P1-33 | Open | AudioVolume drawer无shape/priority/overlap/crossfade语义 | typed volume model、gizmo、overlap resolver preview与runtime parity |
| SND2-P1-34 | Open | component property仍依赖字符串schema，customization无typed address/version | 接共享reflection/property schema与migration，拒绝unknown/stale field |
| SND2-P1-35 | Open | 无产品 `AudioWorldSystem/SceneAudioBridge`消费三类component | per-World owner按entity/component generation create/update/remove |
| SND2-P1-36 | Open | spatial transform缺parent world transform、handedness、meters-per-unit与velocity timestep合同 | canonical transform projection与固定场景oracle |
| SND2-P1-37 | Open | attenuation/cone/doppler无viewport gizmo和听觉/视觉一致性 | Editor gizmo与Runtime函数共享参数/曲线并做容差测试 |
| SND2-P1-38 | Open | occlusion只有描述/原型，无query schedule、budget、smoothing与fallback | physics query broker + generation + cost/latency/drop observation |
| SND2-P1-39 | Open | IR/convolution字段存在但无asset/probe/bake/residency产品链 | versioned IR asset、bounded bake、partition/tail budget、last-good fallback |
| SND2-P1-40 | Open | Acoustic Debug五个`Space`，无真实overlay provider | overlay只显示真实listener/source/volume/ray/IR generation及成本 |

### 6.5 Live output、preview runtime 与遥测

| ID | 状态 | 当前差距 | 重构要求 |
|---|---|---|---|
| SND2-P1-41 | Open | controller无ProductHost构造、实例解析或owner lease | SoundEditorRuntimeBridge由activation receipt构造并持有generation lease |
| SND2-P1-42 | Open | 三个output route没有桥到controller action | typed operation factory调用bridge，结果进入operation receipt/diagnostic |
| SND2-P1-43 | Open | device picker/status是`Space`，没有snapshot projection | 显示default/selected/available/format/latency/state/error/stale |
| SND2-P1-44 | Open | configure/start/stop没有并发状态机、idempotency与deadline | serialized device supervisor，重复/冲突请求有typed terminal result |
| SND2-P1-45 | Open | rendered/callback/xrun字段存在，但未证明由真实callback持续写入 | callback单调计数、sequence/gap/xrun/last-error来源可追踪 |
| SND2-P1-46 | Open | 无hotplug/default-device change订阅、LKG与recovery | stable device ID、format renegotiation、fallback/retry/backoff状态机 |
| SND2-P1-47 | Open | audition、Editor ambient preview、PIE/runtime output所有权未定义 | session/voice/device owner矩阵，停止一个session不误杀其他voice |
| SND2-P1-48 | Open | graph readiness与device start顺序无Editor gate | start前等待artifact/provider/graph/device generation一致 |
| SND2-P1-49 | Open | controller错误停留在字符串report，未统一到Console/Notification/operation journal | typed error code/context/action，接Editor10/11共享渠道 |
| SND2-P1-50 | Open | close project/plugin shutdown无voice/callback/worker/device fence | admission close、cancel、drain、retire、release顺序可测试且有timeout policy |

### 6.6 Timeline、动态事件、测试与性能治理

| ID | 状态 | 当前差距 | 重构要求 |
|---|---|---|---|
| SND2-P1-51 | Open | Timeline optional editor只有descriptor/capability，无track contribution | 注册stable track/section/clip schema、toolkit、factory、compiler与preview |
| SND2-P1-52 | Open | 没有Sound timeline document/runtime observation产品投影 | UI只读取document/playback snapshot，不显示固定Ready/sample文本 |
| SND2-P1-53 | Open | Runtime仍按`delta_seconds`手工推进，缺sample/frame/time转换、seek epoch | audio sample clock为authority，显式rate/rounding/loop/discontinuity合同 |
| SND2-P1-54 | Open | dynamic event registry是Mixer空节点，无schema browser/handler truth | versioned event/payload catalog、handler readiness、preview和bounded result |
| SND2-P1-55 | Open | ray-traced convolution editor/runtime feature仍是manifest/descriptor壳 | provider state、geometry generation、bake job、IR artifact、overlay与unload |
| SND2-P1-56 | Open | fake manager、source-shape与registration tests被当作能力证据 | 分层产品测试启动真实catalog/host/operation/software-null/real device |
| SND2-P1-57 | Open | 缺codec恶意输入、allocation bomb、超长文件与cancel corpus | bytes/time/memory/fd/thread预算和残留artifact断言 |
| SND2-P1-58 | Open | 缺software-null、默认设备、指定设备、hotplug与失效矩阵 | 可重复offline lane + lab real-device lane + fault injection |
| SND2-P1-59 | Open | 缺万clip/千track/长waveform/高频meter的Editor预算 | canonical workloads记录CPU、memory、I/O、UI latency、deadline miss |
| SND2-P1-60 | Open | 当前ignored microbench只证明局部allocation优化，无法与参考引擎同任务比较 | build/source/device/workload绑定的raw evidence与correctness/quality前置门 |

## 7. P2：工程级能力扩展

| ID | 状态 | 扩展差距 | 目标 |
|---|---|---|---|
| SND2-P2-01 | Open | 无MetaSound级可编程audio graph | versioned node registry、compiler、audition、diagnostic与sandbox |
| SND2-P2-02 | Open | 无interactive music segment/transition系统 | tempo map、quantization、condition、transition/fade与预听 |
| SND2-P2-03 | Open | 无Sound Cue/random/container/playlist资产家族 | typed composite asset、deterministic random与voice policy |
| SND2-P2-04 | Open | 无响度、频谱、相位、多声道分析工具 | offline/real-time analyzers、标准化单位与exportable report |
| SND2-P2-05 | Open | 录音、麦克风、voice chat、AEC无owner | permission/device/session/network/privacy与diagnostic闭环 |
| SND2-P2-06 | Open | 高级空间格式/对象音频无能力协商 | platform/device format negotiation与fallback |
| SND2-P2-07 | Open | 无offline render/bounce/stem export | deterministic render graph、artifact identity与cancel/retry |
| SND2-P2-08 | Open | 无audio golden、diff与感知回归 | waveform/spectrum/loudness/latency oracle和可审计阈值 |
| SND2-P2-09 | Open | 字幕、视觉提示与无障碍metadata未进资产工作流 | stable cue linkage、localization、validation与preview |
| SND2-P2-10 | Open | 第三方DSP/plugin缺sandbox与兼容治理 | ABI/version/thread/budget/crash isolation和签名策略 |
| SND2-P2-11 | Open | 分布式cook/waveform/loudness cache未接共享DDC | content-addressed remote execution、provenance与poison recovery |
| SND2-P2-12 | Open | 无协作编辑、merge与reviewable audio graph diff | stable IDs、semantic diff、conflict model与review artifact |

## 8. 历史台账重判

| 历史报告 | 当前重判 | 说明 |
|---|---:|---|
| Editor17 P0 | **5 Open / 0 Partial / 0 Closed** | 默认产品/asset toolkit、33 commands、live output owner、29 Spaces、transaction/audition/scene loop均未闭合 |
| Editor17 P1 | **60 Open / 0 Partial / 0 Closed** | 当前Runtime局部优化不满足任何Editor端到端完成条件 |
| Editor17 P2 | **12 Open / 0 Partial / 0 Closed** | 高级graph/music/analyzer/voice/spatial/offline/golden/accessibility/DSP/DDC/collaboration均无产品owner |
| Editor17 Gates | **32 Fail / 0 Partial / 0 Pass** | 每个Gate要求可执行产品闭环；类型、descriptor、fake test或microbench不单独构成Partial |

Runtime 的细粒度状态继续由 Runtime139 持有。唯一相关 Sound failure handoff [Kira send frame-capture routing](../../zircon_plugins/02/failure-2026-07-19-kira-send-frame-capture-routing.md) 仍不得由 Editor 文档宣告关闭。

## 9. 参考引擎差异裁决

### 9.1 Unreal Engine

- `AudioEditorModule.cpp`注册/注销 property customization、graph node/pin/connection factory，并提供 SoundCue/SoundSubmix 专用 editor 与 menu/toolbar extensibility manager；Zircon 只有静态 descriptor。
- `AssetDefinition_SoundWave.cpp`把 SoundWave 作为可打开资产，支持自定义 editor 或 SimpleAssetEditor fallback；Zircon Sound asset没有toolkit。
- `SoundSubmixEditor.cpp`有专用 graph/details tab、selection、create/delete、`FScopedTransaction`、Undo/Redo、dirty/save和多资产编辑；Zircon Mixer没有document/graph/transaction。
- `ReimportSoundFactory.cpp`保存source path、处理格式变化、执行atomic reimport、失效compressed data、更新platform data/thumbnail并mark dirty；Zircon缺source provenance与reimport acknowledgement。
- Audio Track Editor 与 MetaSound Editor说明 timeline、可编程graph、validation、clipboard、toolkit与runtime audition是独立工程子系统，不是一组operation字符串。

### 9.2 Godot

- AudioStream inspector生成后台waveform，支持play/pause/stop、seek、duration/position与accessible controls。
- Audio Bus Editor读取真实bus/channel/VU/peak，编辑volume/solo/mute/bypass/send/effect并通过`EditorUndoRedoManager`记录do/undo，支持layout保存。
- Import settings与Interactive Music editor提供preview、trim/loop/quality/transition等可操作工作流；Zircon没有等价Audio Clip或music document。

### 9.3 Fyrox

- Rust Editor已有 AudioPanel、bus tree、parent routing、effect list、renderer/distance model/HRTF resource与command execute/revert。
- Audio preview保存并恢复scene sound state，Play/Pause/Stop/Rewind/Seek可真实驱动Sound node，普通编辑命令会退出preview避免状态污染。
- `StreamingBuffer`明确为长音频提供decoder-backed block refill/seek/rewind；Zircon仍把导入结果完整驻留到PCM DTO。

### 9.4 Bevy

- Bevy用`AudioPlayer + PlaybackSettings + AudioSink/SpatialAudioSink`把asset readiness、entity lifecycle、play/pause/seek/volume和cleanup接到真实ECS system。
- 它明确空间音频只是stereo pan，且没有Editor authoring；可借鉴的是“能力较少但产品路径和限制真实”，不能作为Zircon空Editor能力的完成基准。

### 9.5 Unity Graphics 边界

本地 `dev/Graphics` 选择集是 Render Pipelines Core/HDRP/URP/ShaderGraph package manifest，不含 Unity Audio Editor/Mixer源码。它只能证明该checkout不是Sound参考owner；本报告不从渲染仓库推断Unity Audio能力，也不声称已完成对Unity Audio的源码比较。

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
  -> bounded observations/meter/acoustic overlay

TimelineDocument + SoundTrackProvider
  -> sample-clock-qualified artifact
  -> runtime playback/event receipts
```

关键约束：source/document/artifact/runtime/editor observation必须使用不同类型；每个异步结果携带source revision、artifact key、runtime/device generation和cancel token；UI只能显示document或observation事实，不能显示固定Ready/零telemetry。

## 11. 分层重构里程碑

| 里程碑 | 依赖 | 交付物 | 退出条件 |
|---|---|---|---|
| M0 Capability truth | MVP F0-F2 | Sound activation plan、Editor catalog/provider/resource/factory preflight | 缺一项整能力fail-close，无残余菜单/toolkit |
| M1 Audio Clip source | M0 + asset/import owner | source/settings/provenance、bounded decoder、artifact/pages/seek/waveform | 长音频不全PCM驻留；reimport可解释且可取消 |
| M2 Clip toolkit/audition | M1 + document/job | toolkit、transport、scrub、loop/marker、audition session | save/reopen与software-null/真实设备基本矩阵通过 |
| M3 Mixer document | M0 + transaction | stable track/effect/send IDs、transaction、save/recovery | 33命令中Mixer相关路径可执行或删除 |
| M4 Mixer compiler/runtime ack | M3 + Runtime139 M1-M4 | validator、immutable graph、generation swap、meter observation | active graph变更、stale reject、RT安全通过 |
| M5 Live output | M0 + Runtime device supervisor | controller factory、device picker、telemetry、hotplug/LKG | configure/start/stop/failure/reconnect/shutdown矩阵通过 |
| M6 Scene audio authoring | M2-M5 + Scene owner | typed inspectors、gizmos、camera/listener policy、World bridge | create/update/remove/unload无泄漏且视觉/听觉一致 |
| M7 Acoustics | M6 + physics/resource | occlusion broker、IR/probe/bake/residency、truthful overlay | budget/fallback/generation/oracle通过 |
| M8 Timeline/events | M2-M4 + Editor14 | track provider、sample clock、event schema/browser/preview | seek/loop/scrub/save/compile/runtime parity通过 |
| M9 Qualification | M1-M8 | corpus、fault、scale、soak、quality与competitive raw evidence | G01-G32全Pass后才允许升级成熟度 |

M0 不得绕过当前 MVP 主链。高级 Sound 实施必须在`.codex/plans/MVP重构方案/00-index.md`的 F0-F5 依赖顺序允许后进入；本轮仅记录差异，不提前实现旁路。

## 12. 资格门

| Gate | 状态 | 必须证明的结果 |
|---|---|---|
| G01 | Fail | Sound启用时默认Editor加载runtime/editor/feature provider、resources、toolkit与controller；缺失fail-close |
| G02 | Fail | Sound asset打开Audio Clip toolkit；provider缺失时安全只读fallback给typed原因 |
| G03 | Fail | 33条operation逐项有factory/handler/terminal receipt或从catalog删除 |
| G04 | Fail | 五份ZUI不再以`Space`承载业务；每个可见控件有data/state/action/error projection |
| G05 | Fail | ImportSettings可序列化/迁移/hash并进入artifact key，reimport diff可解释 |
| G06 | Fail | WAV/MP3/OGG/FLAC/AIFF/Opus声明与实际backend一致，不支持项预先拒绝 |
| G07 | Fail | 畸形/截断/超大音频满足bytes/time/memory预算，取消后无临时残留 |
| G08 | Fail | 长音频通过page/stream/seek/residency，不因完整PCM DTO无界驻留 |
| G09 | Fail | waveform按source/settings generation异步生成、可取消/复用，stale结果被拒绝 |
| G10 | Fail | play/pause/stop/seek/loop在offline/software-null/真实设备有terminal result与确定shutdown |
| G11 | Fail | Mixer操作支持execute/revert、dirty、save/reload、autosave/recovery |
| G12 | Fail | track删除/重命名transactionally修复或拒绝所有引用 |
| G13 | Fail | compiler定位cycle/missing/unsupported/invalid/channel incompatibility |
| G14 | Fail | graph ack绑定source revision、artifact key、runtime/device generation，stale commit拒绝 |
| G15 | Fail | meter来自callback事实；RT线程无Editor锁、allocation、I/O或同步UI调用 |
| G16 | Fail | device picker显示真实default/selected/format/latency/state并处理hotplug |
| G17 | Fail | rendered/callback/sequence/xrun/error由backend更新，不支持字段显示Unavailable |
| G18 | Fail | configure/start/stop并发、重复、timeout、失败回滚状态机通过 |
| G19 | Fail | Audition与PIE/runtime voice/device ownership隔离且停止互不误杀 |
| G20 | Fail | AudioSource字段typed投影、mixed value、validation、undo/save与runtime schema一致 |
| G21 | Fail | AudioListener active/pose/HRTF与Editor/PIE camera切换规则可测试 |
| G22 | Fail | AudioVolume shape/priority/overlap/crossfade与gizmo/runtime resolver一致 |
| G23 | Fail | AudioWorldSystem按entity/component revision create/update/remove且unload无voice泄漏 |
| G24 | Fail | transform、单位、handedness、velocity timestep通过固定3D场景oracle |
| G25 | Fail | attenuation/cone/volume gizmo与runtime曲线/边界在容差内一致 |
| G26 | Fail | occlusion受budget/smoothing/fallback控制，overlay显示真实query generation/cost |
| G27 | Fail | IR/convolution有验证/cook/residency/tail budget/fallback，空feature不可启用 |
| G28 | Fail | Timeline track有stable IDs、sample mapping、scrub/loop/fade、undo/save/compile/playback |
| G29 | Fail | Dynamic event schema/version/handler/editor/preview闭环，缺失给typed diagnostic |
| G30 | Fail | close/disable/device loss/shutdown取消job并drain voice/callback/late completion |
| G31 | Fail | 万clip、千track/event、长waveform、高频meter满足CPU/memory/I/O/UI latency预算 |
| G32 | Fail | 与Unreal/Fyrox/Godot同内容同设备同格式报告记录质量、原始数据和失败结果 |

## 13. 禁止继续采用的临时实现

1. 禁止用operation字符串、capability、manifest module或菜单存在证明功能完成。
2. 禁止给33条路径补no-op factory、固定success、固定Ready或只改control property的executor。
3. 禁止把29个`Space`换成静态Label/Table后就关闭条目；所有数据必须有真实owner与generation。
4. 禁止让Editor直接持有Kira/CPAL、音频线程、decoder线程池或第二份mixer graph。
5. 禁止让Audio callback等待普通`Mutex`、访问Editor/document/scene、分配无界容器或调用foreign/UI callback。
6. 禁止用fake manager、software-null、source-shape test或ignored microbenchmark代替真实产品/设备/质量门。
7. 禁止继续把完整PCM、waveform、IR和graph都塞进source DTO；source、derived artifact与runtime residency必须分离。
8. 禁止以`delta_seconds`手工推进作为shipping timeline authority；必须接sample clock/epoch。
9. 禁止optional feature只有descriptor/capability却在UI宣称可用。
10. 禁止在没有同任务correctness与质量前宣称性能/表现优于Unreal。

## 14. 完成定义

Editor93 只有在以下条件同时满足时才可关闭：M0-M9按依赖顺序完成；Editor17的5项P0、60项P1、12项P2逐项有current-source证据；G01-G32全部Pass；Sound Editor在默认产品中可打开Audio Clip/Mixer/Acoustic/Timeline并完成transaction/save/reopen/audition；Runtime139提供唯一可执行backend与generation receipt；真实设备、fault、scale、soak和竞争证据可复现；旧descriptor-only、Space、无factory、全PCM与手工clock产品路径已hard cutover且无兼容壳。

本轮没有修改production代码，也没有宣告整体Sound、Editor或Engine目标完成。
