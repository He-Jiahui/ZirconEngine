---
related_code:
  - zircon_editor/src/ui/animation_editor
  - zircon_editor/src/ui/binding/animation
  - zircon_editor/src/ui/binding_dispatch/animation
  - zircon_editor/src/ui/host/animation_editor_sessions
  - zircon_editor/src/ui/host/editor_event_execution/animation_event.rs
  - zircon_editor/src/ui/host/editor_manager_animation_editor.rs
  - zircon_editor/src/ui/host/builtin_views/activity_windows/animation_graph_view_descriptor.rs
  - zircon_editor/src/ui/host/builtin_views/activity_windows/animation_sequence_view_descriptor.rs
  - zircon_editor/src/ui/layouts/views/animation_editor.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/animation_graph.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/animation_sequence.rs
  - zircon_editor/src/ui/retained_host/app/helpers/animation_assets
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench
  - zircon_editor/src/ui/template_runtime/builtin/template_bindings.rs
  - zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/gameplay_animation.rs
  - zircon_editor/src/ui/timeline_strip
  - zircon_editor/src/ui/weight_heatmap
  - zircon_editor/assets/ui/editor/animation_editor.zui
  - zircon_editor/assets/ui/editor/host/animation_graph_body.zui
  - zircon_editor/assets/ui/editor/host/animation_sequence_body.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation
  - zircon_runtime/src/core/framework/animation/asset
  - zircon_editor/src/tests/editor_event/animation_runtime
  - zircon_editor/src/tests/editor_event/runtime/animation_assets.rs
  - zircon_editor/src/tests/host/animation_editor.rs
  - zircon_editor/src/tests/host/binding_dispatch/animation.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md
  - docs/plans/optimize/zircon_editor/07-play-session-process-pie-game-view-live-edit-recovery-review.md
  - docs/plans/optimize/zircon_editor/08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/11-logging-diagnostic-journal-output-console-status-routing-retention-export-review.md
  - docs/plans/optimize/zircon_editor/12-settings-preferences-scope-persistence-locale-i18n-appearance-plugin-extensibility-review.md
  - docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/AnimationEditor
  - dev/UnrealEngine/Engine/Source/Editor/Persona
  - dev/UnrealEngine/Engine/Source/Editor/AnimationBlueprintEditor
  - dev/godot/editor/animation
  - dev/Fyrox/editor/src/plugins/animation
  - dev/Fyrox/editor/src/plugins/absm
  - dev/bevy/crates/bevy_animation
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 14 · Animation Sequence、Graph、State Machine、Timeline/Curve、Preview 与 Compiler Authoring 工程化差距

## 1. 结论

Zircon Editor已经具备动画创作的若干可保留基础：Sequence/Graph/State Machine三类document session、typed runtime asset、`ZRANIM01` magic/version/kind envelope、旧payload fallback、document dirty/autosave接点、部分编辑event、glTF skeleton/clip派生，以及可复用的`TimelineStrip`和`WeightHeatmap`绘制primitive。运行时Graph还支持Clip/Blend/Additive/Mask/Output，State Machine支持Clip、1D/2D Blend Space、SubMachine、GraphRef、layer、exit time和interruption。结论不能写成“动画资产或版本机制完全不存在”。

但当前产品尚不能称为可用的动画编辑器，更不能作为面向大型项目的工程级authoring系统。最严重的五个断点是：

1. 生产asset type registry只为UI资源注册toolkit。Animation Skeleton/Clip/Sequence/Graph/State Machine虽进入resource kind registry，`OpenAsset`却会因没有toolkit直接显示“No toolkit”；现有打开测试在执行前手工注入动画toolkit，掩盖了真实产品入口断裂。
2. 可见Sequence/Graph界面主要是空slot和字符串列表。`Scrub Timeline`固定发送frame 0，`Add Node`固定发送`State`节点，而session只接受`output`或`blend`，所以按钮稳定无效。Sequencer、Montage、Blend Space、Pose Library、Retarget、Control Rig、Motion Matching和Compression工作区又多数只改静态control/status字符串，却使用“Preview queued”“Apply queued”等成功措辞，形成能力过度声明。
3. `EditorEvent::Animation`被标为`DelegatedToTransactionEngine`，实际路径却直接原地修改session，没有transaction ID、undo/redo或command inverse。session先被改动，随后`ensure_document_external_effect`若失败，内存资产已经变化但dirty/history/metadata合同没有完成。
4. binary loader只校验envelope，不提供Graph/State Machine/Sequence语义compiler与validator。Editor可保存空/重复ID、悬空引用、端口不匹配、cycle、错误条件类型、无效entry/layer/reference或属性类型不匹配的资产；序列化成功不等于可执行资产成立。
5. scrub/play/preview没有连接preview world或runtime evaluator。Sequence的`playing`只参与标签显示，Blend Space transport只切control状态并把时间跳到固定0或3秒。创作者无法在保存前看到与运行时一致的pose、transition、root motion、notify或debug结果。

本报告记录5个P0、60个P1、12个P2，给出M0-M7重构路线与32个验收门。目标是建立`AnimationAuthoringDocument + Semantic Compiler + Transaction Command + Preview Session + Typed Projection`，而不是继续增加只能修改字符串或裸asset vector的按钮。本轮没有修改production动画代码。上一轮同一工作树的`zircon_editor --lib`测试编译在617.2秒后被239个既有test-build错误和122个warning阻断，本轮没有重复相同Cargo lane；58个focused test attributes只做静态inventory，不得表述为动态通过。

## 2. 审查边界与可复验证据

### 2.1 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| session/event/binding/host authoring核心 | 31 / 5,147 / 188,207 | E3：open/restore/mutate/dirty/save以及Sequence/Graph/State Machine逐分支闭环；fingerprint `197af8bc...e4fae73d` |
| 产品projection、ZUI、showcase与preview control | 52 / 8,814 / 380,881 | E3代表链、E2全量inventory：pane template、slot、reflection、transport/search和高级animation workspace；fingerprint `99d88d06...01fe5b06` |
| runtime asset schema与glTF导入接点 | 20 / 2,098 / 67,952 | E3：binary envelope/fallback、channel/graph/state/sequence schema及skeleton/clip derivation；fingerprint `ca9c0a02...68d188b4` |
| focused tests | 17 / 3,909 / 141,474 | E3静态阅读：58个test attributes、0 ignored；fingerprint `ee4b2275...7fc8693f` |
| selected combined scope | 118 / 19,681 / 767,992 | 本轮取证时去重集合，83个总test attributes、0 ignored；fingerprint `6e5d6e57...16b2175` |

fingerprint按相对路径排序，将`path + NUL + per-file SHA-256 + LF`拼接后计算SHA-256。它标识本轮证据集合，不是animation asset schema、compiler key或DDC key。

focused tests覆盖基本session load/save、Sequence key/track/range/rebind、Graph node/connect/parameter、State Machine state/transition/condition、synthetic open route、pane metadata、binding codec和template slot anchor。它们没有覆盖生产registry下真实打开、真实按钮产生有效authoring、transaction/undo、preview tick/evaluation、semantic compiler、Graph cycle/port/type、typed property binding、curve/notify/root motion、atomic/CAS保存、malformed/large input、插件节点或大资产性能。

### 2.2 在途文件与验证隔离

成文时有13个animation/timeline/weight相关source/test文件处于并行在途状态，包括Graph/Sequence session、animation codec、gameplay animation template binding、reflection route、timeline strip、weight heatmap及多份focused tests。本报告没有回退、格式化、暂存或提交这些修改。实施前必须重取source、fingerprint、production registry和动态测试结果，因此`source_recheck_required=true`。

证据等级：

- E3：`Asset Browser OpenAsset -> type definition toolkit -> view open route -> restore session -> document toolkit`逐函数闭环。
- E3：`binding/event -> apply_animation_event -> session mutator -> dirty external effect -> metadata`逐分支闭环。
- E3：Sequence、Graph和State Machine每个公开mutator与runtime asset schema逐字段对照。
- E3：pane presentation、ZUI slot、template binding、Blend Space search/transport和extension feedback逐action对照。
- E3：Unreal Persona/Animation Editor/Animation Blueprint、Godot Animation Track Editor、Fyrox Animation/ABSM command源码对照。
- E2：Bevy用于runtime graph asset、event和编译后执行结构参考，不是Editor UX基准；仓内Unity Graphics checkout不含Unity核心Animation Editor，不能据此推断其完整authoring实现。
- 未覆盖：真实GPU skinning preview、音频同步、motion capture、多人协作、超大骨骼/十万key、坏盘/断电、外部DCC往返和所有平台窗口交互。它们进入验收门或后续专项，不冒充已验证能力。

### 2.3 本轮追踪的生产链

1. builtin registry声明五类animation resource kind，但`builtin_toolkit()`只为UiLayout/UiWidget/UiStyle返回descriptor。
2. `OpenAsset`只有在definition带toolkit时才创建open operation；否则只更新status并返回unchanged。
3. focused test helper先调用`register_animation_asset_toolkits()`，再写fixture、索引和打开，所以测试证明的是“手工补齐后的route”，不是默认产品可达性。
4. route恢复按locator解析source path，使用`AnimationEditorSession::from_path`读取整个文件，并依赖`.sequence.zranim`、`.graph.zranim`或`.state_machine.zranim`后缀选择decoder。
5. restore先把entry插入session map，再sync metadata和register document toolkit；后两步失败时没有移除刚插入的entry。
6. animation event直接调用host mutator；mutator先原地改asset，再注册dirty external effect和更新view metadata。
7. `undo_policy_for_event`声称animation交给transaction engine，但dispatch trace只为Inspector和Operation取得transaction ID，animation记录为None。
8. save直接`fs::write`source，session立刻清dirty；随后读取metadata、best-effort调用import并忽略结果，最后sync instance。
9. Sequence scrub只更新current frame，play只更新三个transient字段；仓内未发现消费`playing`并推进animation editor preview的tick owner。
10. Graph visible Add Node发送unsupported `State`；Graph session只创建Output或Blend。连接只检查source存在且非self，不检查target/port/type/cycle。
11. State Machine editor只创建GraphRef state，transition用固定30 FPS把frame转seconds；exit time、interruption、layers和其他state kind不可编辑。
12. pane presentation把track/parameter/node/state/transition压成`Vec<String>`；host body留出空timeline/canvas slot，独立Animation Editor layout又主要由Space节点构成。
13. Blend Space search只过滤三个硬编码名字；transport只改checked/text/current_time，previous/next固定为0/3秒，不调用runtime evaluator。
14. 高级animation workspace action统一进入extension feedback表，结果是预设output/status文本；没有对应asset transaction、job、compiler或preview output ownership。
15. glTF导入能派生translation/rotation/scale track，这是正确基础；MorphTargetWeights被忽略，event tracks固定为空，Editor也没有retarget/compression authoring owner。

## 3. 已有工程基础，重构时必须保留

### 3.1 Versioned runtime asset基础

- binary envelope已有`ZRANIM01` magic、version 1和asset kind检查，并为部分schema提供V1/V2/V3 payload转换。重构应把它接入migration/validation链，而不是另造不兼容格式。
- channel已表达Bool/Integer/Scalar/Vec2/Vec3/Vec4/Quaternion、Step/Linear/Hermite和tangent；Editor当前只消费其中很小一部分。
- Graph schema已有Clip/Blend/Additive/Mask/Output，State Machine已有多种state kind、layer、blend mode、mask weight、exit time和interruption。
- runtime animation专项已经审查执行、缓存、pose、IK、GPU skinning和状态机边界；Editor应消费同一compiler/evaluator合同，不建立第二套近似语义。

### 3.2 Document与产品接点

- animation session能持有三类document，dirty和autosave payload已接入通用document toolkit框架。
- typed `AnimationTrackPath`、asset locator route和view descriptor给稳定identity奠定基础。
- pane metadata同步、close/save/autosave入口已有骨架，适合替换成transactional document authority。
- `TimelineStrip`和`WeightHeatmap`已有bounded/cached painting基础，可作为typed timeline/blend-space projection primitive继续使用。

### 3.3 Import与运行时参考

- glTF skeleton与clip derivation已覆盖joint hierarchy以及translation/rotation/scale sample转换。
- runtime asset能枚举direct references，为dependency graph、reimport invalidation和compiler key提供起点。
- runtime evaluator和compiled graph已有实现基础；Editor preview的正确方向是建立受控gateway，不是在UI层再写简化采样器。

## 4. 目标架构

### 4.1 Authoring document与编译边界

| 层 | 应持有内容 | 不得持有内容 |
|---|---|---|
| `AnimationAuthoringDocument` | stable element ID、typed track/node/state、layout metadata、selection-independent asset data、revision | live preview entity、raw control string、UI widget pointer |
| `AnimationSchemaRegistry` | node/state/track/parameter descriptor、pin/property type、owner/version、migration、capability | 当前document内容 |
| `AnimationSemanticCompiler` | name/reference/type/cycle/time/domain校验，canonical IR和stable diagnostics | UI status string、磁盘副作用 |
| `CompiledAnimationArtifact` | source revision、dependency revisions、platform/profile key、runtime-ready IR | mutable Editor selection |
| `AnimationPreviewSession` | isolated world、subject/skeleton、compiled artifact、parameter state、clock、debug snapshot | authoritative source asset |
| `AnimationProjectionModel` | typed rows/nodes/pins/keys/curves/diagnostics、stable IDs和incremental diff | 用display string反向解析命令 |

Canonical pipeline必须是：

```text
bounded bytes / imported source
  -> version dispatch + migration
  -> structural validation
  -> schema/reference/type resolution
  -> semantic compile + deterministic diagnostics
  -> immutable compiled artifact keyed by source/dependencies/options
  -> isolated preview runtime
  -> typed debug/profiling snapshot
```

任何阶段失败都保留上一个可预览artifact，但UI必须明确显示“source invalid / previewing last good revision”，不得把旧preview伪装成新资产成功。

### 4.2 Transaction command

所有Sequence key/track、Graph node/edge/parameter、State/transition/layer和import setting修改都必须形成`AnimationEditCommand`：包含document ID、base revision、stable target IDs、typed before/after或可逆delta、merge key、diagnostic provenance和dirty external effect。执行流程为prepare/validate/apply/compile/publish，失败不改变document/history/dirty/projection；连续scrub和drag可以coalesce，但mouse-up/commit必须生成可撤销transaction。

保存流程必须采用validated revision snapshot、同目录temp、flush、atomic replace、目录durability、source identity/CAS与import acknowledgement。import或sync失败时不能把document宣称为clean；autosave必须记录schema、source revision、dependency snapshot和恢复诊断。

### 4.3 Preview与runtime debug

Editor preview应通过明确gateway创建隔离world和preview subject，绑定真实skeleton/mesh/material，使用runtime同一compiler/evaluator并由Editor clock驱动。scrub必须确定性seek；play、pause、step、loop、speed和range必须消费统一timebase。debug snapshot至少包含active state/transition、node weights、blend samples、parameters、events、root motion、pose errors、cache hit与evaluation timing，并以source stable ID映射回Graph/State Machine UI。

preview不得直接污染当前项目world；Play/PIE可选择attach runtime instance，但需要显式session identity、read-only/record模式、disconnect和stale-generation防护。

### 4.4 Typed产品面

Animation toolkit至少提供Document、Viewport、Skeleton/Hierarchy、Asset Browser、Details、Timeline/Dope Sheet、Curve Editor、Notifies/Events、Graph/State Canvas、Parameters和Compiler Results。pane projection传递typed key/node/pin/diagnostic model及stable row identity；Canvas/Timeline处理selection、marquee、zoom/pan、snap、drag/drop、keyboard、accessibility和virtualization。尚未接真实asset/compiler的showcase必须移出生产导航或显式标记prototype，不能继续发布成功反馈。

## 5. P0：产品不可达、数据合同失真或无法验证authoring结果的问题

### P0-1 · 默认产品无法从Asset Browser打开动画资源

Animation五类resource kind虽然已注册，默认definition没有toolkit。生产`OpenAsset`因此不会创建Animation Sequence/Graph view；打开测试预先注入toolkit才通过route断言。必须由animation editor plugin或builtin owner在同一registry transaction中注册resource definition、toolkit、capability、descriptor和lifecycle lease，并增加不使用test helper的真实bootstrap测试。Skeleton/Clip若暂不支持编辑，也必须有只读inspector/preview或明确capability，而不是静默无toolkit。

### P0-2 · 可见控件与高级工作区声明了不存在的能力

`Scrub Timeline`固定frame 0，`Add Node`发送session不支持的`State`并返回unchanged；timeline/canvas是空slot。Sequencer/Montage/Blend Space/Pose Library/Retarget/Control Rig/Motion Matching/Compression action多数只写“queued”字符串。用户看到的按钮、状态和完整工作区形态会使其相信资产已修改、预览、应用、重建或压缩。必须先建立capability truth table：未实现功能隐藏/disabled并提供typed reason；已显示command必须产生可观察的transaction、job或preview result，不能以字符串模拟成功。

### P0-3 · Animation mutation绕过承诺的transaction/history合同

undo policy标记为delegated，但event直接原地修改session；无transaction ID、inverse、undo/redo、coalescing或history context。更严重的是mutator完成后才注册dirty external effect，后置失败会留下已改asset但history/dirty/metadata未提交。必须把document mutation放入Editor02的transaction authority，先prepare external effect与validation，再原子commit revision；任何失败断言document bytes、dirty、history和projection不变。

### P0-4 · 没有共享语义compiler，Editor可保存不可执行资产

runtime binary envelope能拒绝magic/version/kind错误，但不保证ID唯一、Graph有且仅有合法output、引用存在、pin类型兼容、无非法cycle、parameter与condition匹配、entry/layer有效、sequence path可绑定或数值有限。Editor mutator也只做局部字符串检查。必须建立Editor/runtime共享的semantic compiler和stable diagnostic code；保存、preview、cook与runtime load应消费同一规则和canonical IR，不能各自容忍不同坏资产。

### P0-5 · Preview不执行runtime动画，authoring没有真实性闭环

Sequence scrub/play仅改变session字段和标签；Blend Space transport仅改变control状态/文字与固定时间。没有preview subject、pose evaluation、active node/state、event/root motion、runtime error或performance feedback。创作者无法判断Graph/State Machine/Curve修改是否正确。必须以runtime compiler/evaluator构建隔离preview session；在此之前所有“playing/preview/apply”状态必须降级为明确不可用，而不是伪造成功。

## 6. P1：工程级完整性差距

### 6.1 Toolkit、document lifecycle、save与诊断

1. Animation Skeleton和Clip被识别为resource kind，但没有明确的只读/可编辑toolkit、preview和details能力合同。
2. Sequence、Graph和State Machine route依赖operation string与共享layout/tab常量，没有按asset kind声明toolkit strategy、supported commands和pane composition。
3. State Machine复用`editor.animation_graph` descriptor；mode只藏在loaded payload，command enablement和UI composition无法由descriptor capability可靠区分。
4. `from_path`按filename suffix选择decoder，而不是使用registry已解析的resource kind和validated route；扩展名、metadata和payload kind冲突没有typed diagnostic。
5. `fs::read`在任何size budget前读完整文件，bincode decode也未配置反序列化limit；恶意或损坏资源可先消耗大量内存。
6. restore先插入session map，再执行metadata sync和toolkit registration；后置步骤失败会留下orphan或半注册session。
7. source save使用raw `fs::write`，没有同目录temp、flush、atomic replace、directory sync、backup/LKG或crash recovery。
8. session在文件写成功后立即清dirty；后续metadata、import或sync失败时，Editor可能显示clean但catalog/runtime派生物仍是旧revision。
9. `import_asset`结果被显式忽略，失败不进入notification、job、diagnostic或retry owner。
10. autosave payload只有source path和bytes，没有asset kind/schema、document revision、source identity、dependency revision、checksum或恢复兼容策略。
11. 没有外部文件编辑检测、CAS、three-way merge、read-only/source-control checkout和保存冲突workflow。
12. invalid parameter、unsupported node、missing state等多种原因都折叠为`Ok(false)`和通用“Ignored”；missing focus又靠英文字符串精确匹配改写为success，缺少typed error、diagnostic code和actionable location。

### 6.2 Sequence、Timeline、Dope Sheet与Curve

13. `create_track`只解析`EntityPath/ComponentPropertyPath`字符串，不解析preview/project scene、component schema、property existence、writability或channel type。
14. 新track无论目标属性类型都创建Scalar(0)+Step key，不能正确初始化Bool/Integer/Vec/Quaternion/Transform或离散属性。
15. 新binding的`target_id`固定None；没有stable entity/bone/component binding、rename/move remap、missing target placeholder或rebind diagnostic。
16. `add_key`复制vector末尾的key而不是在插入时间采样当前曲线或当前preview property；在中间插key会得到与时间邻域无关的值。
17. key identity用seconds差值`f32::EPSILON`比较；帧率变化、长时间线和浮点转换下不能形成稳定key ID或quantization语义。
18. Editor没有typed key value修改、batch numeric edit、quaternion/euler显示策略、color/unit metadata或Inspector integration。
19. 没有move/scale/duplicate/copy/paste/cut key、ripple edit、time stretch、reverse、retime或transaction coalescing。
20. runtime channel支持Linear/Hermite和tangent，但Editor没有interpolation mode、break/unify tangent、weighted tangent或curve handle editing。
21. selection只有单个`(track_path,start,end)`span；没有multi-key/multi-track selection、stable key identity、marquee、selection preservation或跨pane同步。
22. timeline range是transient session字段，可超出asset duration；修改range不修改duration，添加超界key也不定义duration extension policy。
23. scrub只修改frame，不采样channel、不更新preview pose/property、不触发event suppression策略，也没有subframe/timecode/drop-frame语义。
24. playback没有tick owner；speed只拒绝非finite，仍接受0或负值而没有reverse/paused语义、loop boundary和event traversal规则。
25. 没有track hierarchy、expand/collapse、filter、lock、mute/solo、visibility、color、group、layer或虚拟化大列表。
26. 没有event/notify/audio/montage section轨道、payload schema、duration notify、sync marker或触发预览；glTF importer也把event_tracks固定为空。
27. 没有真正Curve Editor、dope sheet模式切换、root-motion extraction/trajectory、additive base、compression error overlay或source-vs-compressed比较。

### 6.3 Graph、Parameter、Pin与Compiler

28. Graph mutator只能新增Output或Blend，而runtime schema已有Clip/Additive/Mask；节点palette与asset能力不一致。
29. 可见Add Node固定发送`State` kind、placeholder locator和`new_state` ID，当前实现必然返回no-change，且测试没有从真实button断言document mutation。
30. node ID只在新增Blend时做exact duplicate检查；空ID、reserved `output`、大小写/Unicode策略和loaded duplicate都没有统一validator。
31. connect只检查source存在和非self，不验证target存在、目标pin、source output type、target input type、cardinality或duplicate edge identity。
32. Graph没有cycle检测、topological compile、unreachable node、dead parameter或multiple/missing output诊断。
33. Additive节点连接永远写`base`，没有选择`additive` pin；Mask/Output也只按target ID猜唯一输入。
34. disconnect以from/to字符串移除所有匹配，没有edge/pin stable ID，无法只断开某个多输入slot或保持edge metadata。
35. remove node把引用清为空字符串并继续保存，而不是通过transaction删除edge、插placeholder或阻止产生悬空必需pin。
36. parameter不存在时根据文本`trigger -> bool -> int -> scalar -> vectors`顺序猜类型；`"1"`与`"1.0"`会创建不同schema，输入形式意外决定公共资产合同。
37. 没有parameter rename/remove/type change、default reset、range/unit/category/tooltip、expose-to-runtime、reference update和迁移。
38. Graph document不保存节点位置、size、comment/group、selection、zoom/pan、reroute、collapsed state或user layout profile。
39. 没有owner-versioned node descriptor/pin registry、plugin node lifecycle、unknown node placeholder、node upgrade/migration和serialization compatibility。
40. 没有incremental semantic compile、last-good artifact、dependency key、source-to-runtime debug map、node weight/profiling overlay或compiler result pane。

### 6.4 State Machine、Blend Space、Layer与高级动画工具

41. runtime state支持Clip、BlendSpace1D、BlendSpace2D、SubMachine和GraphRef，Editor只能创建GraphRef state。
42. state name只做exact duplicate检查；空名、reserved name、大小写/Unicode规范、rename与引用更新没有合同。
43. 删除entry state后静默选择vector第一个state；没有显式用户决策、deterministic stable order或invalid-entry diagnostic。
44. transition按`from/to`唯一，创建同pair会覆盖duration；无法表达同一状态对之间基于不同condition/priority的多条transition。
45. transition duration用硬编码30 FPS把frame换算seconds，与sequence FPS、project timebase、display rate和源asset无关。
46. runtime字段`exit_time`和`interruption`不可编辑；也没有priority、ordered interruption、self transition、blend profile或sync policy。
47. condition不确认parameter存在、value type与operator兼容；Triggered可携带错误value，数值比较可指向Bool/Trigger。
48. 每个transition每个parameter只能有一条condition，且没有remove/reorder、AND/OR group、threshold hysteresis、trigger consume或condition debug。
49. runtime layer、Override/Additive blend mode、weight、mask weights完全没有Editor authoring、validation或preview。
50. 没有真实1D/2D Blend Space sample增删、轴范围、triangulation、duplicate point、outside-hull policy、sample analysis和weight preview。
51. 可见Blend Space search只过滤三个硬编码名字；transport只切control属性和0/3秒时间，`WeightHeatmap`没有绑定runtime sample weights。
52. Sequencer、Montage、Pose Library、Retarget、Control Rig、Motion Matching和Compression是静态workspace/feedback演示；没有对应asset schema transaction、job artifact、compiler或runtime preview，必须逐项实现或退出生产能力面。

### 6.5 Product projection、import、性能与测试

53. `AnimationEditorPanePresentation`把节点、参数、轨道、状态和transition压为字符串，丢失stable ID、类型、pin、diagnostic、selection和incremental diff。
54. `animation_editor.zui`主要使用Space占位，host Sequence/Graph body只有空Container/CanvasBox slot；focused tests只证明anchor projection存在，不证明可交互timeline/canvas存在。
55. 没有preview viewport、preview scene settings、skeleton tree、asset details、curve/notifies pane、mesh/LOD/material切换、camera/bookmark和floor/light控制。
56. selection、canvas layout、timeline zoom/scroll、expanded tracks、preview subject和per-asset/user workspace没有持久化与migration。
57. 没有animation compile/import/compression background job、cancellation acknowledgement、progress、resource budget、DDC artifact或large-document virtualization基准。
58. glTF clip derivation忽略MorphTargetWeights并固定空event tracks；asset reimport后也没有Editor session依赖失效、selection remap、last-good preview或冲突解决闭环。
59. 测试依赖注入toolkit，集中于happy path/no-op/string/anchor；没有生产bootstrap、undo/redo、Nth-step failure、compiler corpus、preview parity、large/malformed和plugin lifecycle测试。
60. 可见文本和状态大量硬编码英文，图/时间线缺少keyboard/focus/screen-reader语义、reduced motion/high contrast和本地化revision接入。

## 7. P2：不阻断正确性但影响成熟度的差距

1. 缺少per-user timeline/grid颜色、key shape、curve density、node compact mode和preview背景偏好。
2. 缺少可命名的viewport camera、timeline range、Graph location和state debug bookmark。
3. 缺少近期asset、最近编辑element和跨document navigation history。
4. 缺少drag/drop ghost、snap guide、edge auto-pan、minimap、overview ruler和selection breadcrumb。
5. 缺少key/curve/node/state的批量标签、颜色、comment和review note。
6. 缺少两个animation revision的结构diff、curve overlay和compiled artifact比较。
7. 缺少将compiler/debug snapshot导出为可附加issue的bounded support bundle。
8. 缺少authoring操作耗时、compile cache hit和preview frame budget的opt-in telemetry摘要。
9. 缺少批量rebind、批量compression、批量retarget和命令行dry-run报告。
10. 缺少DCC round-trip preset、source take/clip range管理和import recipe复用。
11. 缺少Editor scripting/remote automation的typed animation command与query surface。
12. 缺少多人协作下的asset lock、元素级冲突提示和review-only模式。

## 8. 参考引擎对照

| 参考 | 逐源码确认的原则 | Zircon当前差距 | 采用边界 |
|---|---|---|---|
| Unreal Animation Editor / Persona | toolkit组合Viewport、Skeleton Tree、Details、Document、Curve Editor、Asset Browser等tab；Animation Data Controller配合transaction/bracket编辑；Animation Blueprint把runtime debug data映射回node | Zircon缺真实toolkit注册、typed panes、transaction controller、curve authoring、preview scene和runtime debug map | 吸收toolkit分层、controller transaction、preview/debug映射原则，不复制UObject/Slate细节 |
| Godot Animation Track Editor | 大量action通过UndoRedo创建；track/key、Bezier、library、blend tree、blend space、state machine各有专用authoring面 | Zircon事件直接改vector，Sequence/Graph/State Machine只覆盖极小操作集 | 吸收所有用户修改可撤销、typed track/key与专用surface原则 |
| Fyrox Animation / ABSM | command实现`execute/revert`；state/node/transition/layer/parameter和animation signal/curve有对应command/editor | Zircon undo policy与实际执行分裂，layer/curve/signal/parameter lifecycle缺失 | 吸收command inverse、ABSM分层与property editor integration原则 |
| Bevy Animation | graph asset与compiled/threaded graph结构分离，asset变化驱动执行结构重建；runtime event是明确领域对象 | Zircon Editor source直接承担展示，缺compiler artifact和event authoring | 只作为runtime data/compile分层参考；Bevy不是本轮Editor UX基准 |
| Unity Graphics checkout | 仅能观察Graphics package内局部Editor/provider惯例 | checkout不含Unity核心Animation Editor，无法做完整对照 | 不用缺失源码证明Unity动画能力，也不据此设定Zircon上限 |

共同结论不是“界面要像某一引擎”，而是：authoring source、transaction command、semantic compile、runtime artifact、preview instance和UI projection必须分层；所有可见能力必须有真实数据与执行闭环。

## 9. 重构里程碑

### M0 · Capability Truth 与产品入口硬切

- 为Animation resource kind注册生产toolkit/owner lease和真实bootstrap测试。
- 建立capability truth table；隐藏或disable所有无真实command/job/preview的workspace/action。
- 修正`Add Node`/`Scrub`等最小可见command，使unsupported输入返回typed diagnostic。
- 定义Animation document ID、revision、asset kind、route和stable diagnostic code。

### M1 · Schema Registry、Validator 与 Compiler

- 建立typed track/node/pin/state/parameter descriptor registry及plugin owner/version。
- 实现bounded parse、migration、structural/semantic validation和canonical IR。
- Graph检查ID/reference/pin/type/cardinality/cycle/output；State Machine检查state/transition/condition/layer；Sequence检查binding/type/time/value。
- runtime load、Editor preview、save/cook共享规则与diagnostic corpus。

### M2 · Transactional Document 与 Durable Save

- 将全部animation mutator迁移为可逆`AnimationEditCommand`和history context。
- prepare dirty external effect、compile和projection，单revision原子commit。
- 实现atomic replace、CAS/external edit、LKG、autosave schema/revision和import acknowledgement。
- restore使用staged entry，任何失败不污染session map/toolkit/metadata。

### M3 · Sequence / Timeline / Curve 完整authoring

- typed binding/track creation、property schema resolution和stable key ID。
- 完成Dope Sheet、Curve Editor、multi-selection、move/scale/copy/paste、interpolation/tangent、snap/timebase。
- 加入events/notifies、root motion、sync marker、track hierarchy、mute/solo/lock和大列表virtualization。
- 让`TimelineStrip`成为真实typed projection，而非独立展示primitive。

### M4 · Graph / State Machine / Blend Space

- 完成全runtime node/state kind、typed pins、node palette、canvas layout和plugin node placeholder/migration。
- 完成parameter lifecycle、transition priority/group/exit/interruption和layer/mask authoring。
- 完成1D/2D Blend Space sample/axis/triangulation/weight visualization。
- compiler result与每个source element stable ID关联。

### M5 · Preview Session 与 Runtime Debug

- 建立隔离preview world/subject/clock和runtime compiler/evaluator gateway。
- 完成seek/play/pause/step/loop/reverse、event/root-motion规则和last-good artifact提示。
- 映射active node/state/transition、weights、parameters、events、cache和timing到UI。
- 支持受控attach Play/PIE instance，不允许跨session或stale generation污染。

### M6 · Import、Retarget、Compression 与高级工具

- 完善morph/event/import recipe、reimport remap和dependency invalidation。
- 分别定义Montage/Sequencer/Pose Library/Retarget/Control Rig/Motion Matching/Compression的真实asset、job、artifact与验收；未达到则保持非生产。
- compile/compression/retarget进入Editor09 background job和DDC。
- 建立source-vs-runtime/compiled质量与性能比较。

### M7 · 性能、兼容、可访问性与发布门

- 建立大骨骼、十万key、大Graph/State Machine、批量asset和malformed corpus基准。
- 完成schema/plugin node迁移、unknown placeholder、cross-version golden和crash recovery。
- 完成keyboard/focus/screen reader/i18n/high contrast/reduced motion。
- 通过production bootstrap、transaction、compiler parity、preview parity、failure injection和平台lane后再公开完整Animation Editor能力。

## 10. 验收门

- [ ] 1. 默认production bootstrap不注入test helper即可从Asset Browser打开Sequence、Graph和State Machine，并为Skeleton/Clip给出声明一致的toolkit行为。
- [ ] 2. 任一route/resolve/sync/toolkit registration失败都不会留下orphan session、view metadata或dirty owner。
- [ ] 3. filename suffix、catalog resource kind与binary kind冲突返回stable typed diagnostic，live document保持不变。
- [ ] 4. oversized、深层、截断、未知版本和malformed binary在明确budget内失败，无panic/OOM/无界分配。
- [ ] 5. save使用validated revision snapshot和durable atomic replace；每个注入失败点保持旧source或完整新source。
- [ ] 6. import/sync失败不会把document标clean，并进入可重试job/notification/diagnostic。
- [ ] 7. external edit/CAS冲突和autosave恢复不会静默覆盖source，能显示revision与merge/另存决策。
- [ ] 8. 每个animation mutation拥有非空transaction ID、history context和typed command provenance。
- [ ] 9. Sequence key/track/value/range编辑的undo/redo恢复完全相同的document bytes、selection和dirty状态。
- [ ] 10. Graph node/edge/parameter/layout编辑的undo/redo恢复完全相同的source与projection。
- [ ] 11. State/transition/condition/layer编辑的undo/redo恢复完全相同的source与preview compile generation。
- [ ] 12. validation、dirty registration、compile或projection在第N步失败时，source/history/dirty/preview均不变。
- [ ] 13. shared compiler拒绝空/重复ID、悬空reference、missing/multiple output、非法numeric和unsupported schema。
- [ ] 14. Graph compiler验证target/pin/type/cardinality/cycle/reachability，并把diagnostic定位到stable node/edge ID。
- [ ] 15. State Machine compiler验证entry、state kind、transition、condition operator/type、exit/interruption、layer/mask和nested cycle。
- [ ] 16. dependency revision变化会确定性失效compiled artifact和preview，并保留明确last-good状态。
- [ ] 17. track创建解析真实component/property schema并选择正确channel value/interpolation，不再默认所有属性为Scalar。
- [ ] 18. key multi-select、move/scale/copy/paste/duplicate/delete和snap在不同FPS/长timeline下保持stable identity与确定结果。
- [ ] 19. Linear/Hermite/tangent编辑与runtime采样golden一致，Quaternion显示/编辑策略有明确测试。
- [ ] 20. timeline display rate、tick resolution、asset duration、range和subframe规则一致，negative/zero speed语义明确。
- [ ] 21. scrub/play/pause/step/loop/reverse驱动真实preview pose，固定输入下与runtime evaluator逐帧一致。
- [ ] 22. event/notify/root motion/sync marker在seek、loop、reverse和边界跨越时有确定且不重复的触发规则。
- [ ] 23. Graph canvas支持typed palette/pins/edges、zoom/pan/marquee/keyboard并在大图下满足frame与内存预算。
- [ ] 24. State Machine可编辑所有runtime state kind、transition/layer字段，并显示active state/transition debug。
- [ ] 25. 1D/2D Blend Space可编辑sample/axis/triangulation，weight heatmap与runtime实际weights一致。
- [ ] 26. 所有高级animation workspace要么产生真实asset/job/artifact/preview，要么从生产导航移除并明确prototype状态。
- [ ] 27. plugin node/state descriptor支持owner lease、unknown placeholder、migration和卸载/重载，不丢opaque source数据。
- [ ] 28. 10万key、1万node/transition、超大骨骼和批量compile有声明budget、可取消job与可复现benchmark。
- [ ] 29. glTF morph/track/reimport变化能更新dependency、保留可映射selection并处理本地未保存编辑冲突。
- [ ] 30. Animation toolkit完成keyboard/focus/screen-reader/i18n/high-contrast/reduced-motion验收。
- [ ] 31. 测试同时覆盖默认production registry和test fixture；禁止用注入toolkit掩盖bootstrap断路。
- [ ] 32. crash、断电、磁盘满、import失败、plugin卸载、preview runtime失败和跨版本恢复均有failure-injection/golden证据。

## 11. 非目标与边界

- 本报告不要求一次重写runtime animation evaluator；其执行/性能问题由Runtime 08C拥有，Editor只要求消费统一合同并暴露真实诊断。
- 本报告不把制作完整DCC建模/骨骼绑定工具作为M0-M5前置；但import/retarget边界必须明确，不能以静态workspace代替能力。
- 本报告不要求复制Unreal Persona布局或Godot/Fyrox控件；要求复制的是transaction、compiler、preview和capability truth等工程原则。
- 本报告不把现有58个focused test attributes当成动态通过，也不因当前test-build基线失败而降低验收标准。
- 本轮只做review与计划，不修改production代码、测试、asset schema或插件manifest。

## 12. 完成定义

Animation Editor只有在以下事实同时成立时才能从“原型/基础设施”提升为“工程级authoring”：生产入口可达且能力声明真实；所有修改可撤销并durable；source能由共享compiler给出确定语义；preview与runtime一致；Sequence/Graph/State Machine及其typed UI能覆盖runtime模型；导入、依赖、插件、升级、崩溃和大资产均有失败边界与证据。在此之前，应把当前实现定位为asset/session/event骨架与UI prototype，而不是完整动画制作套件。

## 产出记录与时间

| 日期 | 里程碑 | 状态 | 完成项目与证据 |
| --- | --- | --- | --- |
| 2026-08-24 | M0 Capability Truth：内建动画资产生产入口 | `进行中 / 生产代码与静态门通过 / Cargo 行为门未执行` | `zircon_editor/src/core/asset/type_registry/builtin.rs` 为 `AnimationSequence`、`AnimationGraph`、`AnimationStateMachine` 注册唯一 built-in toolkit route；`zircon_editor/src/core/commands/defaults.rs` 注册三条对应 open-operation；`zircon_editor/src/tests/editor_event/{support.rs,runtime/animation_assets.rs,animation_runtime/support.rs}` 与 `tests/workbench/reflection/action_dispatch.rs` 删除测试注入路径，真实打开用例改为默认 bootstrap。production owner 与更新的支持测试 `rustfmt --check` 通过，route-operation 静态映射为 3/3、旧注入符号为 0，scoped `git diff --check` 通过；`runtime/animation_assets.rs` 存在本切片前的全文件 rustfmt 漂移，未为删除一行重排无关断言；未执行 Cargo。该切片不关闭 M0：capability truth table、typed diagnostic、document revision 与 unsupported visible commands 仍待实现；M1-M7 不受此记录影响。 |
| 2026-08-24 | M0 Capability Truth：移除无效 header action | `进行中 / 生产代码与静态门通过 / Cargo 行为门未执行` | 删除 `assets/ui/editor/host/animation_graph_body.zui` 的固定 `AddNode` action、`assets/ui/editor/host/animation_sequence_body.zui` 的固定 frame-0 `ScrubTimeline` action，以及 `template_bindings.rs` 中两条伪命令投影；保留 graph canvas 与 timeline 原生 slot。删除无 builtin-template、binding、descriptor 或文档消费方的旧 `assets/ui/editor/animation_editor.zui` 空壳资产，不保留兼容入口。静态检查确认 5/5 placeholder route 均不存在、2/2 原生 slot 仍在、全仓旧 asset 引用为 0；相关 Rust owners `rustfmt --check` 与 `git diff --check` 通过，Cargo 未执行。 |
| 2026-08-24 | M0 Route Kind：canonical operation 解码与恢复回滚 | `进行中 / 生产代码与静态门通过 / Cargo 行为门未执行` | `AnimationEditorDocumentKind` 与 `from_path_for_document_kind` 让宿主按 canonical open-operation 选择 decoder，不再以文件 suffix 决定恢复文档；host 从内建 asset type registry 反查三条动画 toolkit，并拒绝 operation/view descriptor 不一致的 route。旧 `AnimationEditorSession::from_path` 降为 `#[cfg(test)]` 的 fixture helper，不再存在于 production API surface。动画 restore 在 sync 或 toolkit registration 失败时删除刚写入的 session，并在注册失败时恢复原 view metadata，避免遗留半注册 animation session。新增 `route_loading_tests.rs` 覆盖 graph bytes 使用 sequence suffix 时仍按 route 解码；lifecycle 内联测试覆盖 3 个 registered route、错误 descriptor 与未知 operation。原 `session/tests.rs` 保持 797 行，新测试 owner 51 行，避免越过结构规范 800 行预算。scoped `rustfmt --check --config skip_children=true` 与 `git diff --check` 通过；未执行 Cargo。typed diagnostic、document revision、完整 staged restore/transaction 与 UI asset 对称修复仍待后续切片。 |
| 2026-08-24 | M2 Durable Save：动画 toolkit 写入边界收束 | `进行中 / 生产代码与静态门通过 / Cargo 行为门未执行` | `animation_editor_sessions/save.rs` 现在在 route locator 解析后从 session 取得不可变 document bytes，以 runtime `atomic_write` 发布 source，并将 `import_asset` 失败转换为返回错误；删除原先的直接 `fs::write`、先清会话 dirty、忽略导入错误和保存中 metadata sync。`sync_animation_editor_instance` 改为只投影 `DirtyDocumentRegistry`，不再从 session dirty 反向登记外部 effect；正式保存 target 仅由 canonical route locator 决定，session 的 `asset_path` 仅作为 title fallback 和 pane presentation 的展示标签，直接 `save()` 仅保留为 `#[cfg(test)]` fixture 且同样使用 `atomic_write`。宿主现有保存回归覆盖 edit -> durable source bytes -> workbench dirty 清除。静态结果：atomic write `1/1`，direct write、session save、swallowed import、save-time sync 均为 `0`；4 个 owner 的 `rustfmt --check --config skip_children=true` 与 scoped `git diff --check` 通过。未执行 Cargo；validated revision snapshot、CAS/external edit、import failure injection、LKG/autosave revision 与 mutation transaction 仍待 M2 后续切片。 |
| 2026-08-24 | M0 Capability Truth：动画目标稳定诊断 | `进行中 / 生产代码与静态门通过 / Cargo 行为门未执行` | 新增 `AnimationEditorTargetDiagnostic`，以 Sequence/Graph x no-focus/missing-focus/wrong-focus-kind 的 6 个 typed 状态映射 `ZR-ANIM-TARGET-001..006`。`focused_animation_sequence_instance` 与 `resolve_animation_graph_instance` 的 6 条目标错误路径不再构造 `UiAsset(String)`；`editor_event_execution/animation_event.rs` 改为仅依据 typed accessor 决定可忽略警告，裸字符串即使文案相同也不再改变控制流。新增 error/执行器单元测试锁定 code、display 和 typed-only tolerate 合同。静态结果：typed target error `6`、旧 target `UiAsset` 路径 `0`、生产执行器旧文案匹配 `0`、稳定 code `6`；`editing.rs`、`animation_event.rs` rustfmt 和四文件 scoped `git diff --check` 通过。`editor_error.rs` 与 `host/mod.rs` 存在本切片前的全文件 rustfmt 漂移，未为新增类型重排无关导出/旧断言；未执行 Cargo。route/binary kind diagnostic、document revision 与 transaction provenance 仍待后续 M0/M2。 |
| 2026-08-24 | M0 Route/Binary Kind：fallback-safe typed load diagnostic | `进行中 / 生产代码与静态门通过 / Cargo 行为门未执行` | Runtime `AnimationAssetError::binary_kind_mismatch()` 递归穿透 document/stream 与 V1/V2/V3 fallback 链，保留 envelope 的 `expected/actual` kind；`animation_editor/session/error.rs` 取代 tuple-string session error，只有真实 kind mismatch 才生成结构化 `AnimationEditorBinaryKindMismatch`。Host restore 将该结构化状态投影为 `EditorError::AnimationDocumentLoad` 和稳定代码 `ZR-ANIM-LOAD-001`，普通 I/O/格式错误仍保留原始 `UiAsset` 路径，不以错误文本决定分支。新增 runtime fallback、graph bytes 被 sequence route 打开、editor diagnostic 三个测试 owner 覆盖。静态结果：裸 `AnimationEditorSessionError(...)` 构造 `0`、结构化 accessor `3`、route loader 使用 `4`、独立 route test `86` 行且原 `session/tests.rs` 仍为 `797` 行；10 个新增/变更 owner 的 scoped `rustfmt --check --config skip_children=true` 与 scoped `git diff --check` 通过。`editor_error.rs` 的无关 import/旧断言存在本切片前的全文件 rustfmt 漂移，未重排；未执行 Cargo。M0 capability truth table、M2 authoritative document/revision/transaction 仍待实现。 |
| 2026-08-24 | M0 Capability Truth：Graph schema action hard cut | `进行中 / 生产代码与静态门通过 / Cargo 行为门未执行` | 参考 `dev/UnrealEngine` 的 graph schema 将 node creation 作为显式 capability 判定：新增 `animation_editor/capabilities.rs` 的 10 项真值表，明确 document open、sequence timeline、Output/Blend、basic state-machine edit 为当前可用，Clip/Additive/Mask、semantic compiler 与 runtime preview 为不可用。该表直接驱动 `resolve_animation_graph_node_kind`；session `add_graph_node` 硬切为仅接收 typed `AnimationGraphNodeKind`，删除 raw string kind 与未知 kind 静默 `Ok(false)` 路径。Host 对 `AddGraphNode` 映射 unknown kind 到 `ZR-ANIM-CMD-001`，对 runtime 已声明但 editor 未实现的 Clip/Additive/Mask 映射到 `ZR-ANIM-CMD-002` 和 `EditorError::AnimationCommandUnavailable`，不再伪装为成功/无变更。新增 capability owner 测试和更新的 event runtime 回归锁定完整 typed dispatcher error chain 与 console 文案。静态结果：capability rows `10`、明确 unavailable graph kinds `3`、raw string session API `0`、silent unknown no-op `0`、host resolver `1`、typed error regression `1`；5 个相关 owner rustfmt 与 scoped `git diff --check` 通过，`capabilities.rs` `206` 行、`session/tests.rs` `797` 行、graph event test `316` 行。`editor_error.rs` 的无关 import/旧断言仍有本切片前的 rustfmt 漂移，未为新增 variant 重排；未执行 Cargo。M0 的完整 visible capability projection、document ID/revision 与 M2 transaction 仍待实现。 |
| 2026-08-24 | M2 Architecture：authoritative document 迁移前调研 | `进行中 / 架构调研完成 / 尚未开始该切片生产迁移` | 审查 `CoreEditContext`、`EditorTransactionEngine::TransactionScope::commit_after_apply`、`DocumentToolkitRegistry`、`DirtyRegistry` 与当前 animation session。现有基础已提供唯一 `DocumentId`、`HistoryContextId::Document`、history save token、dirty document/external-effect generation 和在 history publish 前回滚的 after-apply hook；animation session 却仍同时拥有 Sequence/Graph/StateMachine 持久 asset 与 timeline/playback/selection 瞬态 UI state，host 直接锁 session mutation 后再标 dirty。定稿迁移顺序：1) 在 core edit context 建立 `AnimationAuthoringDocumentStore`，以 document ID、canonical asset locator、asset kind、monotonic revision 持有唯一持久 source；2) session 仅保留 sequence timeline/playback/selection 与 projection；3) 用 reversible `AnimationEditCommand` 以 expected revision apply/revert，所有 mutate 在 `HistoryContextId::Document` transaction 内执行；4) `commit_after_apply` 同时完成 dirty external effect、compile/projection，失败则 command rollback；5) restore 先 staged core document，再 session projection/toolkit，任何环节失败全量回退。未直接在 UI session 加 ID/revision，也未创建未接线 core store skeleton，避免新旧双权威。该调研非性能优化，未运行 profiler/Cargo；M2 production 迁移、CAS/external edit、LKG/autosave 与 failure injection 仍待后续切片。 |
| 2026-08-24 | M0 Route Kind：同 view 的 Graph/State Machine 文档隔离 | `进行中 / 生产代码与静态门通过 / Cargo 行为门未执行` | `editor.animation_graph` 同时承载 Graph 与 State Machine；此前 resolver 仅检查 view descriptor，Graph event 可能进入 State Machine session 后退化为 raw `UiAsset`。新增 session `document_kind()`，将 resolver 硬切为 expected asset kind + target kind 双校验：Graph operation 仅接受 Graph document，6 个 State Machine mutator 全部走 StateMachine resolver。稳定诊断保留既有 Sequence `001..003`、Graph `004..006`，为同-view/wrong-document 增加 Graph `ZR-ANIM-TARGET-008`、StateMachine `009..012`（含 `012` wrong-document）；不改既有 code。Graph node kind capability 判定位于 document resolve 后，避免错误 target 被错误归类为 unavailable node。新增 event runtime 回归覆盖 state-machine asset 上的 Graph AddNode，锁定完整 typed dispatcher chain 和 `ZR-ANIM-TARGET-008`。静态结果：StateMachine resolver mutator calls `6`、Graph/State resolver and document gate 均存在、旧 Graph wrong-view `006` 保留、wrong-document code `008/012` 和两条 typed regression 均存在；14 个相关 owner 的 scoped rustfmt 与 scoped `git diff --check` 通过。`editor_error.rs` 的无关 import/旧断言存在本切片前 rustfmt 漂移，未重排；未执行 Cargo。document ID/revision 与 transactional core store 继续由 M2 接管。 |
