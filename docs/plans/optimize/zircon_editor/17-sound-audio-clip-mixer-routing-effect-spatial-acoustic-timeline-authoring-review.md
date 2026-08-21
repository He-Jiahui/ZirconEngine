---
related_code:
  - zircon_plugins/sound/editor
  - zircon_plugins/sound/runtime
  - zircon_plugins/sound/features/timeline_animation_track
  - zircon_plugins/sound/features/ray_traced_convolution_reverb
  - zircon_plugins/audio_importer
  - zircon_plugins/asset_importers/audio
  - zircon_plugins/opus_importer
  - zircon_runtime/src/asset/assets/sound.rs
  - zircon_runtime/src/core/framework/sound
  - zircon_plugins/editor_support/src/lib.rs
  - zircon_plugins/first_party_editor_catalog/src/catalog.rs
  - zircon_editor/src/core/asset/type_registry/builtin.rs
  - zircon_editor/src/ui/host/editor_operation_dispatch.rs
  - zircon_editor/src/ui/host/editor_event_execution/asset_event.rs
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_sequencer_workspace.zui
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/08b-audio-runtime-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md
  - docs/plans/optimize/zircon_editor/07-play-session-process-pie-game-view-live-edit-recovery-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/10-notification-center-toast-decision-history-actions-retention-accessibility-diagnostic-integration-review.md
  - docs/plans/optimize/zircon_editor/11-logging-diagnostic-journal-output-console-status-routing-retention-export-review.md
  - docs/plans/optimize/zircon_editor/14-animation-sequence-graph-state-machine-timeline-curve-preview-compiler-authoring-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/AudioEditor
  - dev/UnrealEngine/Engine/Plugins/Runtime/Metasound/Source/MetasoundEditor
  - dev/Fyrox/editor/src/audio
  - dev/godot/editor/audio
  - dev/godot/editor/import/audio_stream_import_settings.cpp
  - dev/godot/modules/interactive_music/editor
  - dev/bevy/crates/bevy_audio
  - dev/Graphics
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: false
---

# 17 · Sound/Audio Clip、Mixer、Routing/Effect、Spatial/Acoustic 与 Timeline Authoring 工程化差距

## 1. 结论

Zircon的Sound域不是空壳。Runtime已有较完整的typed service contracts、Kira/CPAL backend、output-device lifecycle、mixer graph描述、playback/source/control API、动态事件与automation DTO；`SoundAsset`能严格解析WAV，Audio Importer还能通过Symphonia解码MP3/OGG/FLAC/AIFF等格式。Editor也有真实的`SoundEditorLiveOutputController`和serializable DTO，能枚举设备、读取状态并执行configure/start/stop。这些基础必须保留。

但当前产品仍不能称为工程级Sound authoring。最严重的五个断点是：

1. builtin registry只给Sound显示信息，不给asset toolkit；default linked first-party editor catalog只装配Navigation和Neural。Sound package可被外部动态发现不等于默认产品能打开Sound资产，仓内也没有Audio Clip editor、waveform、preview或import settings入口。
2. Sound插件注册33个authoring operation，但共享batch只调用`register_command`，没有operation factory；五份ZUI合计29个`Space`占位，只有Mixer的Refresh/Start/Stop三个按钮带route。所有component Apply和Mixer编辑命令在产品中都没有transactional executor。
3. `SoundEditorLiveOutputController`是可保留的真实边界，却只在自身单元测试中构造；Mixer没有controller/data binding。Runtime output status的rendered blocks/frames、callback count、sequence与underrun字段只被初始化、清零和投影，没有音频callback写入，因此即使接上UI也会显示伪遥测。
4. Mixer、AudioSource、AudioListener、AudioVolume与Acoustic Debug都发布了完整能力名和可见surface，但内容只是空布局；没有bus/track tree、meter、effect rack、spatial gizmo、occlusion ray、volume influence或IR cache provider，属于能力过度声明。
5. Editor没有`SoundAuthoringDocument`、可逆mixer/component transaction、试听session、source-revision绑定和save/reimport acknowledgement；Runtime插件又只注册component descriptor/options/event catalog，没有`RuntimeSceneSystem`把AudioSource/Listener/Volume同步到Sound manager。Editor无法证明“当前资产/场景修订 -> 当前图 -> 当前输出设备 -> 可听结果”。

本报告记录5个P0、60个P1、12个P2，给出M0-M8重构路线与32个验收门。目标是建立`SoundAuthoringDocument + AudioClipSource/DerivedArtifact + Versioned Mixer Graph + Reversible Audio Edit + Audition Session + Scene Audio Bridge + Truthful Telemetry`，不是继续增加operation字符串、capability或空ZUI。本轮只做静态review，没有修改production代码。[Runtime Audio 08B](../zircon_runtime/08b-audio-runtime-review.md)已拥有backend、scene bridge、streaming/residency与DSP执行差距，本报告只在Editor闭环需要时交叉引用，不重复冒充第二份Runtime总审查。

上一轮同一工作树的`zircon_editor --lib`测试编译在617.2秒后被239个既有test-build错误和122个warning阻断，本轮没有重复相同Cargo lane；86个test attributes只作静态inventory，不得表述为动态通过。

## 2. 审查边界与可复验证据

### 2.1 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| Sound editor/package | 16 / 1,755 / 63,878 | E3逐文件：plugin、33 operations、五份ZUI、live-output controller/DTO及6个test attributes；fingerprint `0e7b89f7...44549db` |
| Timeline audio与ray-traced convolution可选feature | 24 / 1,064 / 39,851 | E3逐文件：editor/runtime/dist/capability/manifest及10个test attributes；fingerprint `d0b20b8c...1b0f7f4` |
| Sound runtime生产实现，不含`src/tests` | 232 / 11,754 / 389,282 | E2交叉复核08B结论，E3复核scene registration、output telemetry与Editor依赖接口；fingerprint `0dd69693...ab3e2be` |
| Sound asset与三组audio importer | 22 / 2,400 / 84,995 | E3读取asset/decode/import authority，23个test attributes；fingerprint `720cd9b2...7862814` |
| Runtime core Sound公共合同 | 28 / 2,142 / 65,840 | E2完整inventory，E3复核三类scene component DTO与Editor依赖；fingerprint `008a130b...e3c238` |
| Editor共享catalog/toolkit/operation/asset-open/Sequencer接点 | 6 / 1,418 / 60,272 | E3默认装配、No-toolkit、MissingFactory和Audio Theme静态行；fingerprint `60572a82...16e0b2` |
| selected combined scope | 328 / 20,533 / 704,118 | 当前工作树去重fingerprint `c9a49221...8ace0e`；86个test attributes、0 ignored、范围内无在途文件 |

fingerprint按相对路径排序，将`path + NUL + per-file SHA-256 + LF`拼接后计算SHA-256。它只标识本轮证据集合，不是audio cook key、waveform key、mixer revision或audition generation。取证时所选328个文件没有工作树修改，因此`source_recheck_required=false`；这不免除实施时对相关源码、依赖版本、产品构造与动态结果的重验，也不得回滚范围外的并行改动。

86个静态test attributes主要覆盖WAV/Symphonia解析、service contract、mixer/backend内部行为、manifest/dist ABI、Editor注册与fake manager controller。它们没有覆盖默认Editor bootstrap、Sound资产Open、五份ZUI数据投影、33个operation的factory dispatch、真实设备callback telemetry、import settings/reimport、waveform与seek试听、document undo/save/recovery、scene component同步、viewport acoustics overlay、Timeline audio执行或设备热插拔。

### 2.2 证据等级与未覆盖边界

- E3：Sound editor/package每个文件逐项阅读，五份ZUI物理存在性、节点类型、event route和注册关系闭环。
- E3：33个operation从descriptor到authoring batch、command registry和`MissingFactory`分支闭环；全仓排除descriptor/test后复核无业务consumer。
- E3：`SoundEditorLiveOutputController`到manager traits、DTO、测试与生产caller搜索闭环；确认没有产品构造和ZUI binding。
- E3：builtin Sound asset type、toolkit选择、default linked editor catalog及`OpenAsset`路径闭环。
- E3：`SoundAsset`、WAV/Symphonia importer、Opus diagnostic importer及clip metadata/streaming/preview字段缺失逐项复核。
- E3：Sound runtime plugin registration、component descriptors、可选feature的空module registration及output telemetry写入点复核。
- E2：Runtime sound生产实现沿08B证据作Editor所需的service/graph/playback/output复核；DSP数值、real-time safety和codec完整正确性仍归08B实施验收。
- E2：Unreal AudioEditor 129个C++/头文件及MetaSoundEditor 90个文件按factory、asset editor、graph schema、audition与diagnostic职责抽样对照。
- E2：Fyrox editor audio 3个Rust文件完整阅读；Godot editor audio 8个文件、import dialog和interactive music editor按preview/bus/undo职责对照。
- E1：Bevy `bevy_audio`只提供runtime ECS/audio边界，没有可比较的Editor authoring产品；Unity Graphics checkout没有Audio Editor源码，不作为完成度基准。
- 未覆盖：真实声卡、ASIO/WASAPI/CoreAudio/PipeWire设备、callback线程、长音频stream、codec corpus、响度/峰值正确性、HRTF/IR声学、跨平台延迟、耳机/多声道硬件和主观听音。它们全部进入验收门。

### 2.3 本轮追踪的生产链

1. Runtime builtin catalog与Sound package manifest声明`runtime.plugin.sound`、Editor module、丰富options及两个optional feature，说明产品意图不是纯metadata示例。
2. Sound editor plugin注册Mixer与Acoustic Debug两个view、一个drawer、主Mixer template和三类component inspector customization。
3. 五份`plugins://sound/editor/...`资源都真实存在，区别于Terrain缺文件；问题是资源内容与可执行能力不匹配。
4. `mixer_console.zui`有9个`Space`占位和3个按钮；preset/device/status/track meter/send/effect/sidechain/automation/dynamic event均没有组件实现。
5. `acoustic_debug.zui`有5个`Space`占位、0 event；toolbar、listener/source cone、volume influence、occlusion ray和IR probe cache均无provider。
6. AudioSource drawer有7个`Space`占位、0 event；input/output/gain/playback/spatial/cone/doppler/send/parameter binding都不可编辑。
7. AudioListener和AudioVolume drawer各有4个`Space`占位、0 event，合计29个空节点。
8. `authoring_bindings.rs`注册20个Mixer/output/debug操作、7个AudioSource操作、3个AudioListener操作和3个AudioVolume操作，共33个。
9. 每个operation有schema ID与capability metadata，但`EditorAuthoringContributionBatch`没有factory字段，publication只调用`registry.register_command`。
10. operation dispatch在descriptor无normalized event且registry无factory时返回`OperationCommandFactoryError::MissingFactory`。
11. 只有output refresh/start/stop三个operation被ZUI route引用；它们同样没有normalized event或factory，按钮点击不能到达live-output controller。
12. 其余30个operation只存在于descriptor/tests，没有typed payload decoder、document target、history context或business executor。
13. `SoundEditorLiveOutputController`真实持有`Arc<dyn SoundEditorLiveOutputManager>`，能snapshot、enumerate、configure、start和stop，并在失败后尽力返回新snapshot。
14. controller DTO包含设备选中/default/availability、backend state、latency、rendered blocks/frames、callback count、sequence、underrun、last error及diagnostics。
15. controller只在自身三项fake-manager测试中构造；全仓生产路径没有owner、lifecycle、view model或shutdown接点。
16. Sound editor module文档还明确说明该slice不添加native pane payload，也不添加`zircon_editor`专用operation dispatch分支。
17. Runtime output telemetry的rendered blocks/frames、callback count、last sequence与underrun只在storage初始化、configure清零和status投影出现，没有callback更新。
18. builtin asset registry将`ResourceKind::Sound`投影为Sound/SND/asset-sound，但`builtin_toolkit`只给三类UI asset返回toolkit。
19. default linked first-party editor catalog只有Navigation和Neural registration；Sound动态package可能被外部发现，但仓内默认发行路径没有Sound editor保证。
20. 因此Sound asset的production `OpenAsset`会进入No-toolkit路径，Editor没有Audio Clip文档或安全只读fallback。
21. `SoundAsset`只保存URI、sample rate、channel count/layout和完整interleaved `Vec<f32>`；没有source provenance、codec、loop/marker、loudness、streaming/cook policy或derived waveform。
22. WAV parser对RIFF/chunk/format/channel/rate/block align/extensible channel mask做了真实验证，应保留。
23. Audio Importer通过Symphonia完整解码到内存，支持多种常见codec；另一组asset importer与Opus importer形成重复authority，Opus仍为diagnostic-only。
24. 导入链没有Editor settings schema、trim/normalize/quality/streaming/loop设置、reimport diff或试听acknowledgement。
25. Runtime Sound plugin注册三类component descriptor、options和event catalogs，没有注册scene system、manager owner或AudioSource/Listener/Volume同步执行器。
26. Runtime08B已确认默认manager inactive，当前唯一生产形态的`start_output_device`调用点在Editor controller，而controller本身没有生产构造。
27. Runtime mixer/service API远比Editor丰富，但Kira graph目前只执行受限子集；unsupported effects/advanced parameters不能在Editor中被预验证或清晰禁用。
28. `sound.timeline_animation_track`的Editor feature只发布descriptor/capability，Runtime feature只注册空`ModuleDescriptor`，没有track schema、clip、compiler或player hook。
29. `sound.ray_traced_convolution_reverb`同样只有descriptor/capability/manifest与空module，没有probe/IR asset、bake job、physics ray consumer或Editor overlay。
30. Sequencer workspace的`Audio Theme / 0000-1460 / Ready`是固定展示行，没有对应Sound timeline document或runtime snapshot。

## 3. 已有工程基础，重构时必须保留

### 3.1 Runtime service与backend基础

- 保留`SoundManager`/trait分层、Kira/CPAL backend、output device descriptor/lifecycle、typed error、graph validation与revision recheck；不要为Editor另造第二套audio engine。
- 保留mixer track/send/effect/automation/dynamic-event DTO与manager API，把Editor document/compiler适配到这些合同，并明确当前backend支持矩阵。
- 保留`SoundEditorLiveOutputController`及其serializable snapshot/action report，它已经是正确的thin controller雏形；需要补产品owner、binding和generation，而不是改回直接按钮调用CPAL。
- 保留WAV严格校验、Symphonia codec覆盖与channel layout建模；重构重点是source/derived/runtime分层、streaming和Editor import policy。

### 3.2 Editor共享基础

- 复用[Editor02](02-document-transaction-save-autosave-recovery-review.md)的document/transaction/save/recovery、[Editor04](04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md)的import/reimport/job/catalog和[Editor05](05-inspector-reflection-property-authoring-customization-review.md)的typed property/custom surface合同。
- 复用[Editor07](07-play-session-process-pie-game-view-live-edit-recovery-review.md)的Play ownership，但Audio audition必须有独立session/generation，不得借全局Play假装preview。
- operation dispatch已经能明确报告`MissingFactory`，应把33个operation的可执行性纳入bootstrap invariant，禁止以no-op factory消除错误。
- 五份ZUI已有稳定asset ID/control ID和布局位置，可在真实component/controller到位后演进；完成前应disabled并显示typed unavailable reason。

### 3.3 参考实现给出的最低工程基线

- Unreal AudioEditor把SoundWave/reimport、AudioBus、SoundCue、SoundClass、SoundMix、Submix、effect、attenuation和concurrency拆成factory、asset definition、graph schema和专用editor；MetaSound进一步提供versioned node registry、compile/audition和diagnostics。
- Fyrox以Rust实现了AudioPanel、bus tree、parent route、effect属性、selection、command execute/revert及Sound preview，证明这些不是语言限制。
- Godot提供后台waveform preview、play/pause/stop/seek、import preview/zoom/trim/loop设置、Audio Bus的meter/send/effect/drag与UndoRedo，以及3D listener/source gizmo。
- Bevy只证明runtime ECS sound source/sink/volume边界，不能替代Editor产品基线；本地Unity Graphics checkout没有音频模块，不能据此宣称对Unity Audio的比较完成。

## 4. 目标架构

### 4.1 Authority分层

| 层 | 应持有内容 | 不得持有内容 |
|---|---|---|
| `AudioClipSource` | source URI/hash、import settings/version、codec metadata、loop/marker、channel policy、provenance | 全量runtime PCM、UI selection、设备handle |
| `AudioClipDerivedArtifact` | target codec/chunks、seek table、waveform min/max、loudness/peak、dependency/cook key、diagnostics | mutable Editor draft、live voice state |
| `SoundAuthoringDocument` | stable object ID、source revision、mixer/component/timeline draft、selection、dirty/history、validation | CPAL/Kira handle、ZUI control ID |
| `MixerGraphSource` | versioned track/send/effect/sidechain/automation topology与stable IDs | backend-specific handle、meter sample |
| `MixerGraphArtifact` | validated ordering、backend capability lowering、parameter layout、source key与typed diagnostics | live output state |
| `AudioAuditionSession` | document/artifact generation、transport、voice ownership、device generation、cancel/drain fence | project save authority |
| `SceneAudioBridge` | entity/component revision、source/listener/volume diff、voice mapping、world/Play generation | Inspector draft |
| `SoundTelemetrySnapshot` | callback-owned counters、meter ring snapshot、latency/xrun/device generation、timestamp与validity | 静态默认零值冒充事实 |

权威链必须是：

```text
bounded source bytes + versioned import settings
  -> codec probe/decode or target streaming cook
  -> immutable clip + waveform/loudness/seek artifact
  -> transactional SoundAuthoringDocument mutation
  -> mixer/component/timeline semantic validation
  -> generation-bound backend graph + audition or scene diff
  -> audio callback-owned output + telemetry snapshot
  -> Editor projection with revision, readiness and typed diagnostics
```

### 4.2 关键合同

1. `SoundOperationFactory`：33个operation逐个拥有typed payload、target document、history context、precondition与invertible command。
2. `AudioClipImportSettingsV1`：codec/quality/sample-rate/channel/streaming/loop/trim/normalize/loudness policy可序列化、可迁移、可hash。
3. `AudioClipPreviewProvider`：异步waveform与audition，带source/artifact/device generation、cancel、seek和completion reason。
4. `MixerGraphCompiler`：source graph验证为backend capability-aware artifact；unsupported node必须在提交前给typed diagnostic。
5. `SoundEditorProductController`：持有live output、mixer document、audition session与telemetry subscription，随view/project/plugin生命周期确定性关闭。
6. `SceneAudioBridge`：把AudioSource/Listener/Volume的增删改按generation同步到唯一Sound manager，并在Play切换与project close时drain。
7. `AcousticDebugProvider`：从physics/sound runtime snapshot生成可筛选overlay，不从ZUI固定文案生成。
8. `SoundTimelineTrack`：stable clip/event IDs、sample/time mapping、scrub/audition语义、compiler/runtime hook和undo/save合同。

## 5. P0 阻断项

### P0-1：默认产品不能打开Sound资产，也不能保证装载Sound authoring插件

builtin registry识别Sound但不给toolkit，default linked catalog不含Sound。必须先建立Audio Clip toolkit、只读fallback与Sound editor装配硬门，动态外部package发现只能作为扩展路径，不能替代默认产品合同。

### P0-2：33个Sound authoring operation全部缺少可执行factory

共享authoring batch只注册command descriptor，产品dispatch最终命中`MissingFactory`；三个可见output按钮也不例外。必须为每个命令建立typed factory/handler或删除能力宣称，禁止用统一no-op executor伪造成功。

### P0-3：真实live-output controller没有产品owner，遥测字段又没有callback写入

controller仅在测试构造，Mixer没有数据绑定；rendered frames/callback/xrun等字段从未被生产callback更新。必须先闭合manager construction、device generation、callback telemetry和view lifecycle，UI才能显示或控制真实输出。

### P0-4：五份可见Sound surface以29个`Space`占位宣称完整能力

Mixer、三类Inspector和Acoustic Debug没有业务组件、数据源或交互，只保留了能力名称和布局。未实现surface必须在capability/readiness层不可见或显式disabled，不能以空白面板作为工程功能交付。

### P0-5：没有transactional authoring、audition与scene-to-sound闭环

Editor修改、保存、试听、Play和Runtime scene component之间不存在generation-bound authority；Runtime也没有scene system。必须证明同一source revision经compile、backend graph和当前设备产生可听输出，并支持undo、失败回滚、project/plugin close drain。

## 6. P1 核心重构差距

### 6.1 产品入口、装配与能力真实性

### P1-1：Sound asset type只有presentation，没有toolkit/read-only preview fallback

`OpenAsset`应能稳定选择Audio Clip toolkit；插件不可用时至少显示metadata、waveform缓存状态与typed unavailable reason，而不是No-toolkit。

### P1-2：default linked editor catalog与runtime builtin catalog的Sound能力不对称

需要一个可测试的product assembly contract，证明manifest启用Sound时editor registration、resources、controller factory和runtime service成套可用。

### P1-3：Editor插件注册成功不验证五份ZUI可materialize并具备所需controller

bootstrap必须验证resource存在、版本可解析、required control IDs和controller/data provider可解析，失败时拒绝发布capability。

### P1-4：Mixer和Acoustic Debug共享generic authoring surface，缺专用pane payload

专用payload需要携带project/document/device/runtime generation及readiness，而不是只有view ID和template URI。

### P1-5：一个generic drawer被命名为Sound Mixer，所有权与实际inspector drawer混淆

应区分Mixer document pane、Audio Clip toolkit、scene component custom inspector和acoustic overlay owner，避免生命周期互相污染。

### P1-6：33个operation的capability只控制可见性，不证明executor readiness

Command projection必须同时检查factory、target document、backend feature和write authority；readiness变化要有generation并触发UI更新。

### P1-7：operation schema ID只有字符串，没有schema注册、版本迁移或payload上限

每个schema必须有typed decoder、unknown-field policy、size/depth budget、version upgrade和actionable validation diagnostic。

### P1-8：menu/view/drawer注册测试只验证ID存在

应增加default product bootstrap、template materialization、controller construction、route dispatch和project close的集成测试。

### P1-9：插件disable/unload没有Sound-specific audition/device/telemetry drain gate

capability撤销前必须停止audition voice、解绑callback subscription、关闭view/document并等待backend generation fence。

### P1-10：Sound能力名称没有实现级别或backend支持矩阵

M1 Kira子集、未实现effect、ray convolution和timeline feature必须以structured capability status表达，不能都显示为可用。

### 6.2 Audio Clip、导入、waveform与试听

### P1-11：`SoundAsset`把所有PCM样本直接存入source/runtime DTO

长音频会带来完整decode、序列化和内存驻留成本；应拆分source metadata、streaming/cooked chunks和小clip resident policy。

### P1-12：Audio Clip没有source provenance与import settings版本

无法解释产物来自哪个源hash、codec参数、引擎版本和目标平台，也无法做可靠reimport diff或DDC key。

### P1-13：缺codec/quality/sample-rate/channel/remix policy

Editor必须提供可验证设置并把channel layout/downmix/upmix规则写入artifact，禁止平台backend临时猜测。

### P1-14：缺trim、normalize、loudness target与peak policy

这些变换必须非破坏性记录、可预览、可撤销，并输出峰值/响度diagnostic，不能修改源文件后失去provenance。

### P1-15：缺loop region、cue marker、beat/BPM与sample-accurate metadata

Timeline、interactive music和游戏事件需要stable marker ID及明确frame/time换算，当前`SoundAsset`无法表达。

### P1-16：没有waveform derived artifact与后台generation

应按多分辨率min/max、channel、source hash和settings生成可缓存artifact，并通过Editor Job系统取消、进度和失败重试。

### P1-17：没有Audio Clip play/pause/stop/seek与scrub session

参考Godot/Fyrox建立独立audition transport，具备device generation、single/multi voice policy、focus和project close drain。

### P1-18：多个audio importer形成重叠authority，Opus仍为diagnostic-only

必须建立单一format dispatch与priority/ownership规则，重复extension不能依赖注册顺序，unsupported codec应在导入前给明确诊断。

### P1-19：Symphonia路径完整decode到`Vec<f32>`，没有bounded streaming/cancel

大文件需要metadata probe、decode budget、chunked cook、cancellation和temporary artifact cleanup，避免导入线程占满内存。

### P1-20：导入成功没有与waveform、audition和catalog revision绑定的acknowledgement

UI只能在source revision、artifact key、catalog entry和preview generation一致时显示Ready；旧任务晚到必须被丢弃。

### 6.3 Mixer graph、routing、effect与automation

### P1-21：Mixer没有versioned source asset或document owner

track/send/effect/preset修改需要稳定asset identity、dirty/history/save/recovery，而不是直接调用global manager。

### P1-22：track create/update/delete operation没有stable ID与引用修复语义

删除track必须处理send、source output、sidechain、automation和preset引用，并以一笔可撤销transaction提交。

### P1-23：send graph没有cycle detection与feedback policy的Editor诊断

Runtime validator能力应前移到authoring compiler，定位到具体track/send和修复建议，禁止点击Apply后才失败。

### P1-24：effect add/update/delete/reorder没有typed node schema与backend支持检查

effect参数、单位、范围、channel policy、latency和tail必须来自注册schema；Kira不支持的effect不能作为可选项发布。

### P1-25：Mixer没有meter数据通道与实时线程隔离

需要callback写入bounded lock-free/RT-safe snapshot，UI按节流频率读取peak/RMS/clip，绝不能在audio callback锁Editor对象。

### P1-26：mute/solo/bypass/gain/pan等基础strip合同缺失

这些控制需要区分authoring default与ephemeral monitoring state，明确哪些进入asset、哪些只属于session。

### P1-27：send matrix与sidechain picker只是空节点

应投影真实graph、过滤非法target、显示cycle/latency并支持keyboard/drag操作和transactional undo。

### P1-28：Mixer preset list/apply没有资产类型、diff或partial apply语义

Preset应有versioned schema、target compatibility、preview diff、dependency与冲突处理，不能仅是manager内部名字。

### P1-29：automation bind/unbind没有parameter registry和time-domain authority

需要stable parameter ID、unit/range、curve interpolation、sample/block scheduling及Timeline/animation桥接，不使用display name作为身份。

### P1-30：Mixer graph编译没有source revision到backend graph revision的可见ack

Editor必须显示编译中、已应用、stale、rejected和backend generation；旧graph commit不得覆盖新document。

### 6.4 Scene component、spatial audio与acoustic debug

### P1-31：AudioSource drawer没有真实field editor或Apply factory

input/output/gain/playback/spatial/cone/doppler/send/parameter必须投影typed property、mixed value和validation，并复用Editor transaction。

### P1-32：AudioListener drawer没有active listener唯一性与viewport ownership

需要定义多个listener、Editor camera preview、Play camera切换和inactive listener语义，避免任意最后写入者获胜。

### P1-33：AudioVolume drawer没有shape/priority/overlap/crossfade编辑语义

shape应接Scene gizmo与collision/query authority，priority/overlap必须有deterministic resolver及可视化。

### P1-34：component descriptor用字符串property type，Editor缺共享typed schema

`sound_source_input`、`sound_attenuation`、track/IR ID等必须可反射、可验证、可迁移，不能在drawer中另造解析器。

### P1-35：Runtime没有SceneAudioBridge消费三类component

需要按entity/component revision diff创建、更新、销毁voice/listener/volume，并处理scene unload、Play generation和hot reload。

### P1-36：spatial transform没有父子world transform与坐标/单位合同

position/forward/up/velocity、meters-per-unit、handedness和doppler timestep必须由scene authority提供，不能直接信任local fields。

### P1-37：attenuation/cone/doppler缺viewport gizmo和数值一致性测试

gizmo必须取同一runtime函数/参数生成边界，支持selection、handle edit、undo和非均匀scale策略。

### P1-38：occlusion只有布尔字段，没有query schedule、budget与smoothing

应定义physics snapshot、ray budget、update cadence、material transmission、hysteresis和失效回退，并可在debug overlay观察。

### P1-39：convolution/IR字段存在但没有asset、probe、bake与residency链

Impulse response需要source/artifact、channel/rate/length验证、streaming/residency、tail budget和fallback reverb。

### P1-40：Acoustic Debug没有真实overlay provider

listener/source cone、volume、occlusion ray、probe/cache必须来自generation-bound runtime snapshot，支持filter、freeze、selection和cost显示。

### 6.5 Live output、preview runtime与遥测

### P1-41：live-output controller没有ProductHost构造路径

应由Sound plugin/editor service factory从唯一runtime manager创建，随project/plugin/device生命周期持有并可诊断构造失败。

### P1-42：Mixer三个output route没有桥到controller action

Refresh/Start/Stop及Configure必须经typed operation factory或pane controller执行，返回action report并更新同generation snapshot。

### P1-43：device picker/status panel没有data projection

需要稳定device ID、default/selected/available、format、latency、state、error与hotplug变化，禁止只显示label。

### P1-44：configure/start/stop没有并发状态机与重复操作策略

Starting/Stopping/Reconfiguring期间应合并、拒绝或取消请求，所有transition带operation ID、deadline和terminal result。

### P1-45：真实callback不更新rendered/callback/xrun telemetry

计数必须由callback或backend事实写入，快照标注timestamp、generation和validity；unsupported字段显示Unavailable而不是0。

### P1-46：output device热插拔和default-device变化没有订阅/recovery

设备丢失应停止旧generation、保存选择策略、尝试可控fallback并给用户可操作诊断，不能静默回软件null。

### P1-47：Editor audition与Play/runtime输出的所有权未定义

需要明确共享manager时的mixer namespace、priority、mute/solo、device reconfigure和session teardown，避免互相截断。

### P1-48：backend graph应用与device start顺序没有Editor readiness gate

只有clip artifact、mixer artifact、device format和manager state全部ready时才能试听；部分失败需保留上一已知可用generation。

### P1-49：错误只进入action report/diagnostic strings，未统一到Console与Notification

应发布typed sound diagnostic，包含source、device、graph revision、operation、severity与remediation，并遵循Editor10/11 retention。

### P1-50：close project/plugin shutdown没有voice、callback、worker与device fence验收

必须可证明停止新提交、取消decode/waveform、drain voice、解绑callback、关闭device并拒绝late completion。

### 6.6 Timeline、动态事件、测试与性能治理

### P1-51：Timeline audio optional feature只有capability和空module

应提供track/clip schema、Editor registration、operation factory、compiler与runtime player hook；否则必须标记Unavailable。

### P1-52：Sequencer的Audio Theme/Ready是固定数据

删除静态成功行，改为真实track projection与artifact/runtime acknowledgement，未启用Sound feature时给disabled reason。

### P1-53：Timeline缺sample/time/frame rate换算与seek语义

需要定义clip start/offset/duration/loop/fade、scrub audition、pre-roll和seek granularity，避免帧时间与采样时间漂移。

### P1-54：动态事件registry只是Mixer空节点

事件定义、payload schema、handler/executor ownership、preview invocation、versioning和引用搜索必须成为可编辑资产或document section。

### P1-55：ray-traced convolution feature只有manifest依赖与空module

必须建立physics ray budget、probe/IR bake job、runtime consumer和Editor debug；在此之前不能发布已实现capability。

### P1-56：测试把注册ID和fake manager成功当作产品证据

新增真实bootstrap、template/controller binding、operation transaction、asset open/import/preview、scene bridge和shutdown集成测试。

### P1-57：缺codec与恶意输入corpus、fuzz和资源预算测试

覆盖截断chunk、畸形metadata、超大duration/channel/sample rate、decode bomb、NaN/Inf与取消清理，所有上限可配置且可诊断。

### P1-58：缺真实设备与software-null分层测试矩阵

software-null验证确定性控制面，真实设备lane验证enumeration/start/stop/callback/hotplug/xrun；两者结果不能互相替代。

### P1-59：缺大项目音频authoring性能预算

需要对万级clip、千track/event、长音频waveform、频繁meter和批量reimport建立CPU、内存、I/O、UI latency与shutdown基线。

### P1-60：没有与Unreal/Fyrox/Godot同任务、同内容质量的对比证据

“性能和表现优于Unreal”必须按import time、preview latency、voice count、callback budget、xrun、memory、cook size和authoring操作延迟逐项证明。

## 7. P2 扩展差距

### P2-1：MetaSound级可编程audio graph尚无versioned node registry与compiler

在Mixer M1闭环后再规划DSP graph、node compatibility、hot swap与audition，不能用当前33个字符串operation替代。

### P2-2：交互音乐缺segment/transition/quantization/tempo map系统

参考Godot interactive music建立可逆transition graph与beat/bar调度，但必须依赖已完成的sample-accurate timeline。

### P2-3：Sound Cue/random/container/playlist资产家族未建立

需要deterministic random、shuffle history、branch、concurrency与预加载策略，不应全部塞进AudioSource字段。

### P2-4：专业响度、频谱、相位与多声道分析工具未规划

分析产物应异步、可缓存、带算法版本，实时meter与离线分析使用不同预算。

### P2-5：录音、麦克风、voice chat与回声消除没有Editor/Runtime owner

涉及权限、隐私、设备路由、AEC/NS/AGC与网络时序，应作为独立系统而非复用试听按钮。

### P2-6：高级空间音频格式与平台对象音频缺能力协商

Ambisonics、Atmos/object bed、平台spatializer与耳机HRTF需要channel/object metadata和fallback策略。

### P2-7：离线render/bounce/stem export没有确定性管线

应固定graph/source revision、sample rate、seed与plugin版本，生成可比对的音频artifact及诊断。

### P2-8：音频差异、golden与感知质量回归工具缺失

需要区分bit-exact、容差频谱和感知指标，并保留输入、build、device/backend和算法版本。

### P2-9：字幕、视觉提示与音频可访问性metadata未进入资产工作流

Dialogue/event应可关联caption、speaker、locale、haptic/visual cue和loudness policy。

### P2-10：第三方DSP/plugin sandbox与兼容治理未规划

插件崩溃、阻塞、denormal、allocation、版本迁移和版权/部署必须有隔离与admission policy。

### P2-11：分布式audio cook、waveform/loudness cache与远程执行未接共享DDC

完成artifact key后接入Tooling08的内容寻址cache、租约、失败隔离和跨平台复用。

### P2-12：协作编辑、merge与reviewable audio graph diff尚无模型

需要stable object IDs、semantic diff、conflict policy、auditionable change set和版本兼容，不能依赖整文件文本覆盖。

## 8. 参考引擎差距裁决

| 参考 | 本轮确认的最低工程职责 | Zircon当前差距 | 采用边界 |
|---|---|---|---|
| Unreal AudioEditor | SoundWave import/reimport、asset definitions、Cue/Class/Submix graph、effect/attenuation/concurrency factories与专用editors | 缺Audio Clip toolkit、graph document/factory/compiler、preview与产品装配 | 学职责/authority/验证，不复制宏/UI框架 |
| Unreal MetaSoundEditor | versioned node registry、graph compile、audition、diagnostics与asset lifecycle | 可选feature仅descriptor/空module，暂无DSP graph产品 | 作为M8以后上限，不阻塞M0-M5基础Mixer |
| Fyrox editor audio | Rust中的bus tree、route/effect editing、command undo、selection与Sound preview | Zircon只有descriptor与空ZUI，真实controller未接产品 | 直接证明Rust可实现的MVP下限 |
| Godot editor audio/import | async waveform、transport/seek、import zoom/trim/loop、bus meter/send/effect/drag/UndoRedo与3D gizmo | waveform/import settings/meter/gizmo/undo均缺 | 学紧凑工作流与后台preview，不复制server API |
| Bevy audio | ECS AudioPlayer/Sink/Volume等runtime边界 | Zircon已有更丰富runtime合同，但scene bridge未形成 | 只参考ECS ownership，不作为Editor UX基准 |
| Unity Graphics checkout | 本地仅Graphics/render pipeline源码，无Audio Editor | 无法提供音频authoring证据 | 明确不比较，不从路径名推断Unity Audio能力 |

工程裁决是：先达到Fyrox/Godot可执行的asset preview、bus editing、undo与scene gizmo下限，再达到Unreal的资产家族、graph compiler、audition和诊断分层；MetaSound级可编程DSP属于后续上限。任何“优于Unreal”的结论必须来自同任务动态证据，而不是文件数量或capability清单。

## 9. 分层重构里程碑

### M0：能力真实性与默认装配

- 将Sound editor纳入可验证的product assembly，或在未装配时隐藏/禁用全部Sound authoring入口。
- 为五份ZUI建立resource/controller/readiness硬门；删除静态Audio Theme Ready。
- 给33个operation生成executor coverage表，未实现者不发布可用状态。

### M1：Audio Clip source/artifact/toolkit

- 建立versioned import settings、source provenance、clip cook/stream policy、waveform/loudness/seek artifact。
- 实现Audio Clip toolkit、async waveform、transport/seek、reimport diff、typed diagnostics与job cancellation。
- 收敛三组importer的格式authority与Opus状态。

### M2：SoundAuthoringDocument与transaction

- 建立stable IDs、dirty/history/save/recovery与typed operation factories。
- 先完成Mixer基础track/send/effect操作和三类component Apply的execute/revert。
- 让所有visible command具备目标document、payload schema和失败回滚。

### M3：Mixer graph compiler与真实UI

- 实现track strip、routing/send matrix、effect rack、preset、sidechain、automation与backend support matrix。
- 编译versioned MixerGraphArtifact，发布applied/stale/rejected generation。
- 接入RT-safe meter snapshot与UI节流。

### M4：Live output与audition session

- 产品构造`SoundEditorLiveOutputController`，绑定device picker/status/actions。
- 补callback-owned telemetry、hotplug/recovery、concurrent state machine和LKG graph/device generation。
- 建立独立AuditionSession及与Play输出的明确所有权。

### M5：Scene audio bridge与spatial authoring

- 实现AudioSource/Listener/Volume scene diff、voice mapping、world transform与lifecycle。
- 完成typed inspectors、attenuation/cone/volume gizmo、selection与undo。
- Project close、scene unload、Play切换和plugin disable均有drain fence。

### M6：Acoustics与debug

- 建立occlusion budget/smoothing、IR source/artifact/residency及fallback。
- 让Acoustic Debug消费真实snapshot，提供cone/volume/ray/probe/cache filter与cost。
- ray-traced convolution保持disabled，直到physics ray、probe bake、IR和runtime consumer闭环。

### M7：Timeline、动态事件与interactive music

- 完成Sound timeline track、sample-time mapping、scrub/audition、compiler/runtime hook。
- 建立dynamic event asset/schema/registry/editor和versioned handler compatibility。
- 再实现segment/transition/tempo/quantization的interactive music。

### M8：规模、质量与竞品验收

- 建立codec fuzz、真实设备矩阵、长音频stream、graph stress、xrun和shutdown tests。
- 建立offline render、golden/perceptual audio regression及内容寻址缓存。
- 对Unreal/Fyrox/Godot执行同任务基线，只有证据满足预算后才允许性能/表现声明。

依赖顺序：`M0 -> M1 -> M2 -> M3 -> M4 -> M5 -> M6 -> M7 -> M8`。M1的artifact key与M2 document可并行设计，但M3不得绕过M2直接操作global manager；M6/M7不得在M4/M5没有generation/lifecycle前发布capability。

## 10. 验收矩阵

| Gate | 必须证明的结果 |
|---|---|
| G1 | 默认发行物在Sound启用时加载editor registration、五份resources、controller factory与runtime service；任一缺失则capability不可用。 |
| G2 | Sound asset双击稳定打开Audio Clip toolkit；插件不可用时显示安全只读fallback和typed reason。 |
| G3 | 33个operation逐项有factory/handler coverage；无event/factory的descriptor在bootstrap失败，而非点击后才失败。 |
| G4 | 五份ZUI不再有承载业务能力的`Space`占位；每个visible control有data source、state、action和error projection。 |
| G5 | Audio Clip import settings可序列化、迁移、hash并进入artifact key；reimport diff可解释变化。 |
| G6 | WAV/MP3/OGG/FLAC/AIFF/Opus支持矩阵与实际backend一致；unsupported格式在导入前给typed diagnostic。 |
| G7 | 畸形/截断/超大音频corpus满足byte/time/memory预算，取消后不残留临时artifact。 |
| G8 | 长音频不会因完整PCM DTO/序列化导致无界驻留；streaming chunk、seek table和residency可观测。 |
| G9 | waveform按source/settings key异步生成、可取消、可复用；旧generation不会覆盖新资产。 |
| G10 | Audio Clip play/pause/stop/seek/loop在software-null和真实设备lane有terminal result与确定性shutdown。 |
| G11 | Mixer track/send/effect/preset/sidechain/automation操作均支持execute/revert、dirty、save、reload与crash recovery。 |
| G12 | 删除/重命名track能transactionally修复或拒绝所有source/send/sidechain/automation引用。 |
| G13 | Mixer compiler检测cycle、missing reference、unsupported effect、非法参数与channel incompatibility，并定位对象。 |
| G14 | backend graph acknowledgement绑定source revision、artifact key和device generation；stale commit被拒绝。 |
| G15 | Meter peak/RMS/clip来自callback事实，RT线程无Editor锁、分配或阻塞，UI读取有界且节流。 |
| G16 | device picker显示真实default/selected/available/format/latency/state；hotplug可恢复或给明确terminal failure。 |
| G17 | rendered frames/blocks、callback count、sequence、xrun和last error由backend更新；unsupported值显示Unavailable。 |
| G18 | configure/start/stop并发、重复、timeout和失败回滚有状态机测试，不出现双owner或悬挂Starting。 |
| G19 | Audition与Play共享/隔离策略有测试；一个session停止不会误杀另一session voice。 |
| G20 | AudioSource inspector所有字段typed投影、mixed value、validation、undo/save，并与runtime component schema一致。 |
| G21 | AudioListener active/pose/HRTF/Doppler与Editor camera/Play camera切换规则确定且可测试。 |
| G22 | AudioVolume shape/priority/overlap/crossfade与gizmo、undo及runtime resolver输出一致。 |
| G23 | SceneAudioBridge按entity/component revision创建、更新、销毁voice；scene unload/Play exit后无泄漏voice。 |
| G24 | parent transform、meters-per-unit、velocity timestep与handedness通过固定3D scene验证。 |
| G25 | attenuation/cone/volume gizmo与runtime函数共享参数，视觉边界和听觉衰减在容差内一致。 |
| G26 | occlusion query受budget控制、有smoothing/fallback；debug overlay显示真实ray generation与cost。 |
| G27 | IR/convolution asset具备验证、cook、residency、tail budget与fallback；未闭环feature不能启用。 |
| G28 | Timeline audio track支持stable clip IDs、sample/time mapping、scrub、loop/fade、undo/save、compile和runtime playback。 |
| G29 | Dynamic event schema/version/handler/editor/preview闭环；缺handler或payload不兼容时给typed diagnostic。 |
| G30 | project close、plugin disable、device loss和process shutdown会取消job、drain voice/callback并拒绝late completion。 |
| G31 | 万级clip、千track/event、长waveform和高频meter满足明确CPU/内存/I/O/UI latency预算。 |
| G32 | 与Unreal/Fyrox/Godot的同内容任务报告记录版本、硬件、设置、质量、原始数据和失败结果，才允许性能/表现比较。 |

## 11. 实施约束

1. 不得把Runtime service API复制成Editor私有audio engine；Editor只能通过versioned document/compiler/controller/bridge消费唯一Sound authority。
2. 不得用no-op factory、固定Ready、默认零telemetry、软件null或测试fake证明真实设备和可听输出。
3. Audio callback不得锁Editor/scene/document、分配无界容器、做文件I/O或发布同步UI事件。
4. 所有异步decode/waveform/cook/graph apply必须携带source revision、artifact key、device/runtime generation与cancel token。
5. 所有authoring mutation必须进入Editor02的transaction/history/save/recovery合同；组件drawer不得直接改runtime manager。
6. 能力发布必须同时满足registration、resource、factory/controller、backend和dependency readiness；可选feature的manifest存在不等于实现完成。
7. 复用Editor09 job、Editor10/11 notification/diagnostic与Tooling08 cache，不建立Sound私有线程池、toast或缓存目录。
8. 实施前重算本报告selected scope fingerprint，并复核依赖版本、默认catalog、产品构造和08B runtime结论；任何变化都要更新差距裁决。

## 12. 本轮状态

- Review：完成，范围为Sound/Audio Clip、Mixer、Routing/Effect、三类scene component、Spatial/Acoustic Debug、live output、Timeline audio与可选convolution的Editor纵向闭环。
- Production code：未修改。
- Dynamic validation：未执行；同一工作树既有`zircon_editor --lib`test-build阻断未变化，不重复消耗相同lane。
- Static validation：完成；5/60/12/32编号连续，37个frontmatter路径与69个报告/索引相对链接存在，selected scope fingerprint复核一致，`git diff --check`无whitespace错误（仅既有CRLF转换提示）。
- Implementation：pending，按M0-M8依赖顺序推进；优先封闭默认入口、factory/controller、Audio Clip toolkit与transaction/audition，不先扩展MetaSound级功能。
