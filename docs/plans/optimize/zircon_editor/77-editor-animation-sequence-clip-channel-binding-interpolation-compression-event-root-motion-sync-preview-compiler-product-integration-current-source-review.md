---
title: Editor Animation Sequence、Clip、Channel Binding、Interpolation、Compression、Event、Root Motion、Sync、Preview、Compiler 与 Product Integration 当前源码工程化差距
category: zircon_editor
report_id: Editor77
review_date: 2026-08-23
baseline_head: f1614c5e601d0879cfa3ac1e5d4886f0d8734d97
baseline_epoch: 355
related_code:
  - zircon_runtime/src/core/framework/animation/asset
  - zircon_runtime/src/core/framework/animation/timeline.rs
  - zircon_runtime/src/animation/clip_event.rs
  - zircon_runtime/src/animation/sequence
  - zircon_plugins/animation/runtime/src/channel_sampling
  - zircon_plugins/animation/runtime/src/evaluation/clip_evaluator
  - zircon_plugins/animation/runtime/src/evaluation/compiled_animation_clip
  - zircon_plugins/animation/runtime/src/evaluation/pipeline
  - zircon_editor/src/ui/animation_editor/session/sequence.rs
  - zircon_editor/src/ui/host/animation_editor_sessions
  - zircon_editor/src/ui/retained_host/app/assets/workspace.rs
  - zircon_runtime/src/asset/importer/ingest/gltf_animation_subassets.rs
tests:
  - zircon_runtime/src/animation/sequence/tests.rs
  - zircon_runtime/src/animation/clip_event.rs
  - zircon_plugins/animation/runtime/src/tests.rs
  - zircon_editor/src/ui/animation_editor/session/tests.rs
  - zircon_editor/src/tests/editor_event/animation_runtime/sequence.rs
  - zircon_editor/src/tests/host/animation_editor.rs
  - zircon_editor/src/tests/host/binding_dispatch/animation.rs
  - zircon_editor/src/ui/retained_host/app/helpers/animation_assets/tests.rs
plan_sources:
  - docs/plans/mvp/index.md
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md
  - docs/plans/optimize/zircon_editor/14-animation-sequence-graph-state-machine-timeline-curve-preview-compiler-authoring-review.md
  - docs/plans/optimize/zircon_editor/32-model-mesh-skeleton-geometry-import-lod-collision-retarget-preview-authoring-review.md
  - docs/plans/optimize/zircon_editor/75-editor-animation-timeline-dope-sheet-curve-editor-track-key-selection-transport-scrub-snap-clipboard-transaction-virtualization-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/76-editor-animation-graph-state-machine-node-edge-parameter-condition-compiler-runtime-transition-blend-preview-transaction-persistence-product-integration-current-source-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Animation/AnimSequenceBase.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Animation/AnimSequence.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Animation/AnimMontage.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimSequence.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimSequenceBase.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimMontage.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimationCompressionDerivedData.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/Animation/AnimSync.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/Animation/AnimNotifyQueue.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Animation/AnimData/IAnimationDataModel.h
  - dev/godot/scene/resources/animation.h
  - dev/godot/scene/resources/animation.cpp
  - dev/godot/scene/animation/animation_mixer.cpp
  - dev/godot/scene/animation/animation_player.cpp
  - dev/godot/editor/animation/animation_track_editor.cpp
  - dev/Fyrox/fyrox-animation/src/track.rs
  - dev/Fyrox/fyrox-animation/src/signal.rs
  - dev/Fyrox/fyrox-animation/src/lib.rs
  - dev/Fyrox/fyrox-impl/src/resource/gltf/animation.rs
  - dev/Fyrox/editor/src/plugins/inspector/editors/animation.rs
  - dev/bevy/crates/bevy_animation/src/animation_curves.rs
  - dev/bevy/crates/bevy_animation/src/animation_event.rs
  - dev/bevy/crates/bevy_animation/src/lib.rs
  - dev/bevy/crates/bevy_gltf/src/loader/gltf_ext/scene.rs
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Editor/Expressions/VFXExpressionBakeCurve.cs
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Editor/Expressions/VFXExpressionSampleCurve.cs
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Editor/Models/Slots/Implementations/VFXSlotAnimationCurve.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Editor/Tools/Converters/AnimationClipConverter/EditorCurveBindingUtils.cs
doc_type: current_source_review
review_status: complete
implementation_status: not_started
source_recheck_required: true
---

# Editor Animation Sequence、Clip、Channel Binding、Interpolation、Compression、Event、Root Motion、Sync、Preview、Compiler 与 Product Integration 当前源码工程化差距

## 1. 结论

当前Zircon已经有两条真实动画数据路径，而不是完全空白。Runtime内置`AnimationSequenceAsset`可以把property path编译为world writer并逐帧采样；Animation Runtime插件可以验证Clip的骨骼T/R/S通道、建立target table、复用pose buffer并发布clip event。Editor也能打开、修改和保存Sequence资产。这些底座应保留，并收敛为同一份source schema、semantic compiler和prepared runtime artifact。

但“资产可序列化、运行时能得到一个值”目前被当成了“动画序列完成”。资产层同时存在`AnimationSequenceAsset`、`AnimationClipAsset`和无人消费的`AnimationTimelineDescriptor`三套不等价模型；Sequence编译允许缺失binding后部分成功，artifact又只保存指向外部source Vec的下标；Clip与Sequence各自复制采样器，甚至把Step在精确关键帧的错误行为写进测试。`Hermite`旋转实际只是忽略切线的slerp，类型或切线错误则在一条路径静默退回left/zero、在另一条路径被拒绝。

播放提交同样没有工程级合同。非循环player到末尾后继续累加时间并反复采样末帧；负speed会把时间向后推进，但event collector直接拒绝`to <= from`，因此反向播放永远没有事件。事件cursor以“播放时间 + event字符串 + Vec下标”续传，发布时又抹掉clip identity并把同一事件以两个Rust类型各发一次。多个Sequence写同一属性时，没有priority、blend、ownership或冲突诊断，最终值只取决于容器与查询遍历顺序。

本轮没有新增P0。Editor14、Editor32、Runtime08C与Editor76已有的缺toolkit、无共享compiler、无runtime preview、glTF动画导入语义分裂、无prepared/compressed artifact、事件/根运动/同步/montage平台和双Runtime authority仍是实施阻断，本报告不重复计数。新增 **13项P1、6项P2与48个资格门**，目标是建立versioned stable-ID source document、唯一`AnimationSemanticCompiler`、自包含`PreparedAnimationClip / CompiledPropertySequence / CookedEventIndex`以及带明确时间、冲突、事件、root motion和完成态的`AnimationPlaybackTransaction`。

本轮只做current-source review与文档建账，不修改生产源码。未运行Cargo、真实Editor、GUI/GPU、save/reopen、reimport、cook、fault/soak/profile或同语义跨引擎benchmark；因此不能宣称当前正确性、性能或表现达到、更不能宣称超过Unreal。

## 2. 审查边界、currentness与冻结语料

### 2.1 本轮唯一owner

本报告只拥有“Sequence/Clip持久语义如何经过validation/compiler/preparation，进入sampling/playback/event/root-motion提交并由Editor预览同一artifact”的纵向边界。

以下内容必须回链既有owner：

- Editor32唯一拥有Skeleton、Skin、retarget、glTF/import/reimport authority；本轮会证明当前可达glTF sibling animation转换失真，但不重复登记该导入P0/P1。
- Editor75唯一拥有Dope Sheet、Curve Editor、track/key selection、scrub/snap/clipboard、timeline virtualization与交互状态。
- Editor76唯一拥有Animation Graph、State Machine、layer、blend space、transition和共享semantic compiler总authority。
- Runtime08C唯一拥有通用animation scheduling、pose、IK、skinning、prepared/compressed artifact平台、root motion与montage/sync大类能力。
- Editor14唯一拥有默认toolkit可达性、通用transaction、无效save、runtime-backed preview与Animation authoring总产品面。

### 2.2 Currentness

- 审查HEAD：`f1614c5e601d0879cfa3ac1e5d4886f0d8734d97`。
- 协作baseline epoch：`355`；session：`optimize-editor77-animation-sequence-clip-review-r1-20260823`。
- working tree中`zircon_runtime/src/animation/clip_event.rs`、`zircon_runtime/src/animation/manager/mod.rs`及部分插件evaluation文件存在非本轮修改；逐项diff复核为import/formatting变化，没有改变本报告列出的语义事实。
- 这是静态current-source证据，不把既有测试名称、忽略benchmark或文档声明当作动态资格。

### 2.3 冻结语料与可复算fingerprint

统计口径：规范化为小写相对路径并排序；每个文件取SHA-256，再拼接`path + NUL + lowercase file hash + LF`计算集合fingerprint。test declaration按Rust/C++/C#声明正则统计。

| 范围 | 文件 / 行 / 非空行 / bytes / test declarations | fingerprint |
|---|---:|---|
| Zircon asset/schema/import | **46 / 5,493 / 5,060 / 193,714 / 0** | `38eb90e4cc87e3c8c26bfd802214cd9d08d766b71230aba393ba37708d6de329` |
| Zircon runtime compiler/sampling | **46 / 6,607 / 6,092 / 235,060 / 29** | `249be1c21cc10ceef9a7b1c6d2dcf867c2f541410cfc01bec8016127c847d2ce` |
| Zircon editor/product | **34 / 4,409 / 4,178 / 166,846 / 4** | `0dc59afc1a887e1062138ca8482299104672e135393025565c5b99df48bbadc3` |
| Zircon focused tests | **18 / 4,958 / 4,572 / 180,825 / 88** | `371af1fbd1c21ba6588155c558f3b14414cc11584c8948eb17fbc862c715632f` |
| Zircon deduplicated focused set | **144 / 21,467 / 19,902 / 776,445 / 121** | `527103e50a10013104c542e8c29367381a92f65e582deb661c4ef5186cea2d6b` |
| Unreal selected set | **21 / 22,039 / 18,603 / 810,219 / 0** | `65f8233319268fc16ed675c384661db5de7f60f25e87dcdf4b4db1fffd9b0b30` |
| Godot selected set | **7 / 21,262 / 18,135 / 784,235 / 0** | `b2d3e7077150ea902c604cb8eb40bb319e695fcd06f111550f67860bfe6e05e0` |
| Fyrox selected set | **7 / 4,291 / 3,866 / 158,775 / 1** | `32df26a05f8bb07d4633287992625aae19cda300866585c5d844335fe641995e` |
| Bevy selected set | **5 / 3,286 / 2,974 / 119,546 / 11** | `62754cf427d1a2f8687fe17bc4b1fb3da60146046db798f7b6ecdbe492fd6630` |
| Unity Graphics selected set | **6 / 740 / 588 / 26,930 / 1** | `c3211b52dbc721612c3c15020d148b128baa31c7f170912b75736e0d5ffd501d` |
| Five-engine deduplicated set | **46 / 51,618 / 44,166 / 1,899,705 / 13** | `1fda455740e1f5ba56caf94889bac089147c089fa33b6cb2af7254b44426a066` |

## 3. 当前真实产品链与已存在底座

### 3.1 持久资产

`asset/channel.rs`提供Step/Hermite/Linear与Bool/Integer/Scalar/Vec2/Vec3/Vec4/Quaternion；key有time/value和可选切线。`asset/clip.rs`按bone name存T/R/S channel，并能存time/event/payload。`asset/sequence.rs`按binding path、可选target string和property path存任意属性channel。`asset/binary.rs`已有`ZRANIM01` envelope与旧版本试读入口。

这些是可以迁移的数据起点，但不是完整的source model：没有stable binding/track/key/event ID、tangent mode/weight、root motion policy、additive base、curve/morph track、notify state、sync marker、section/slot/montage或compression recipe。

### 3.2 编译、采样与提交

- Runtime内置Sequence compiler把property path解析为world writer，并缓存asset revision与world currentness。
- 插件Clip compiler会拒绝重复bone channel、非有限值、非严格递增time、错误T/R/S value type和零长度rotation。
- Clip evaluator有revision cache、target table、pose pool和diagnostic入口；Graph/State Machine pipeline可继续消费采样pose。
- `clip_event.rs`已有有界record数量/字符串bytes、cursor与batch result，不需要退回一次性扫描API。

问题在于这些能力并未共享schema validator、time/interpolation kernel、artifact identity或playback transaction。Pipeline在compile/apply失败处多次使用`continue`、`.ok()?`或忽略返回值，把“本帧没有输出”同时用于资产缺失、编译失败、binding stale、预算耗尽和合法空结果。

### 3.3 Editor产品链

Editor session可打开Sequence，执行Add/Remove Key、Create/Remove/Rebind Track、scrub/range/select/playback并保存资产。真实Sequence body仍主要是空slot与固定frame控制，typed value/interpolation/tangent/event/root-motion/additive/compression编辑能力由Editor14/75拥有并仍未闭合。

可达的glTF sibling转换位于`zircon_editor/src/ui/retained_host/app/assets/workspace.rs`：它把CubicSpline映射成Hermite，却要求`times.len() == values.len()`；glTF cubic output本来是三元组，因此会被拒绝。Linear也被映射成Hermite且没有切线，随后产生缓入缓出而不是线性。该错误证明当前preview/source并不可信，但canonical修复owner仍是Editor32。

## 4. 父报告校正、开放阻断与不重复计数

| 现象 | 唯一owner | 本轮处理 |
|---|---|---|
| Animation asset没有默认完整toolkit、缺typed field/event/root/additive/compression编辑、Save不经shared compiler、preview不执行runtime artifact | Editor14 P0-1/P0-4/P0-5、P1-13至P1-27 | 保持Open，不重复计数 |
| Editor sibling glTF转换与Runtime importer并存；Linear/CubicSpline、morph、target binding语义分裂 | Editor32 P0-2、P1-32及P1-25至P1-36 | 本轮补充可达证据，修复回Editor32 |
| timeline transient/durable状态、selection、scrub、clipboard、virtualization/cache/render | Editor75 | 只消费其`AnimationTimeDomain`和preview session合同 |
| Graph/State Machine/Layer/BlendSpace与三套compiler/runtime authority | Editor76 P0-1/P0-2及P1账 | 本轮artifact必须接入其唯一semantic compiler |
| raw String/AoS clip、同步load/clone、pose allocation、事件平台、root motion、montage/slot/sync/marker/inertialization | Runtime08C P1-6/P1-7/P1-8/P1-15/P1-16、P2-2/P2-3 | 保持Runtime平台owner，本轮只定义Sequence/Clip消费合同 |
| 同一Clip的插件与Runtime manager重复authority | Runtime08C、Editor76 | 不另记P0；hard cut前不得新增第三套evaluator |

因此本报告的0项P0不表示Animation Sequence安全可实施；它表示阻断已被更上游owner登记。Editor77的里程碑必须显式依赖这些owner，而不是借“本轮无P0”绕过它们。

## 5. P1：本轮新增的工程差距

### ED77-P1-01：三份持久动画模型互不等价，`AnimationTimelineDescriptor`还是无人消费的死schema

证据：`AnimationSequenceAsset`表示property binding，`AnimationClipAsset`表示bone T/R/S与event，`AnimationTimelineDescriptor`又表示clip/track/event。全仓引用显示Timeline descriptor只有定义与re-export，没有compiler、Editor或Runtime consumer。它还会静默修正非法duration/fps并把speed夹到非负，与Editor接受负speed矛盾。

风险：同一“动画序列”无法稳定回答source of truth、迁移版本、事件归属和时间政策；继续给任一结构加字段只会扩大转换丢失。

要求：建立一个versioned `AnimationSourceDocument`家族，显式区分property sequence、skeletal clip与action/montage，但共享stable identity、time base、channel、event、metadata、dependency和migration协议；删除无人消费的平行schema或把它迁移为唯一owner。

验收：仓内只有一个资产注册/迁移authority；任意旧版本都经确定性upgrade到当前source schema；round-trip不会静默改写speed/duration/interpolation。

### ED77-P1-02：Sequence编译允许partial success，pipeline又丢弃compile/apply outcome

证据：`sequence/compiled.rs`会保留找不到target/property的track并产出artifact；apply把sample缺失、writer失败归为missing。`evaluation/pipeline/sequences.rs`遇到compile error会remove/continue，调用apply后不消费完整结果。

风险：所谓“compiled”和“played”可能只写一部分属性甚至零属性；Editor preview、游戏帧和日志都无法区分合法空动画与数据损坏。

要求：compiler必须返回typed diagnostic set和明确admission policy；strict product默认任一required binding失败即不替换last-good artifact。运行时提交返回`AnimationApplyReceipt`，逐track disposition可聚合但不能消失。

验收：missing optional、missing required、stale target、type mismatch、writer rejected、budget exhausted分别有稳定code；失败artifact不会进入active generation。

### ED77-P1-03：`CompiledAnimationSequence`不是自包含artifact，靠外部Vec下标和caller revision约定保持正确

证据：compiled binding只保存binding/track index，采样时重新索引外部`AnimationSequenceAsset`；artifact没有source content hash、schema/compiler version、dependency stamp或内嵌prepared channel。

风险：任何绕过当前cache wrapper的调用、资产就地mutation或跨线程publication都可能让artifact引用不同语义；它也无法独立cook、序列化、DDC命中或故障回放。

要求：`CompiledPropertySequence`必须拥有canonical prepared channels、resolved writer program、source/artifact identity、compiler recipe与dependency generations；source asset只在compile边界读取。

验收：artifact离开source owner后仍可独立validate/sample；source mutation不改变已发布generation；cache key由完整语义输入决定。

### ED77-P1-04：Step在精确关键帧仍返回前一关键值

证据：两套channel sampler都用`partition_point(|key| key.time < sample_time)`再选择前一项；在精确命中内部key时，Step取到更早的key。Focused test把该结果写成现状预期。

风险：离散状态、visibility、enum、method/event-like property会延迟一个采样瞬间；不同帧率、seek与压缩采样会产生可见不确定性。

要求：定义统一interval truth table；Step在key time必须切换到该key，边界、duplicate rejection、loop wrap和reverse seek均有同一kernel。

验收：内置Sequence与插件Clip对首/中/末key、key前后ULP、loop边界、正反seek的golden结果逐项一致。

### ED77-P1-05：Quaternion `Hermite`忽略切线，公开标签与实际数学语义不一致

证据：`sequence/interpolation.rs`和插件采样路径对Quaternion Hermite最终只做slerp；输入/输出tangent没有进入计算。其他类型在缺切线时又用zero tangent替代。

风险：导入器、Editor curve UI和runtime对同一interpolation枚举产生不同预期；旋转曲线无法复现glTF cubic spline，也无法评估角速度误差或compression parity。

要求：若支持cubic rotation，使用有明确定义和归一化策略的quaternion cubic算法并保存切线语义；若不支持则compiler fail-close，不得以Hermite名义执行slerp。

验收：reference quaternion cubic fixtures在每个key及区间sample通过误差门；输出归一化，退化输入产生typed diagnostic。

### ED77-P1-06：同一Channel schema存在两套冲突的validation与fallback语义

证据：内置Sequence sampler在value type不匹配时返回left，在缺/错tangent时使用zero；插件Clip compiler对部分T/R/S类型和值拒绝，但仍复制另一套sampling/hermite实现。资产schema本身未声明某用途允许的value/interpolation组合。

风险：一份资产作为Sequence可“播放”，作为Clip却compile失败；修复其中一份采样器不会自动修复另一份，Editor preview也无法说明真实runtime路径。

要求：建立共享`AnimationChannelSemanticValidator`和一个sampling kernel；用途特定约束由typed channel role扩展，禁止运行时猜测或默认纠错。

验收：同一fixture经asset load、Editor compile、cook和runtime load得到相同diagnostic code、location与outcome。

### ED77-P1-07：Clip compiler未建立完整的duration、key range、interpolation、rotation与event合同

证据：当前验证覆盖finite、strict time、T/R/S value类型与非零rotation，但没有统一拒绝非正duration、key超出duration、tangent/interpolation不兼容、非单位rotation、event越界/重复/非法payload等；后续runtime再分别clamp或skip。

风险：错误资产能进入cache，到特定采样时刻才产生静默结果；source、prepared和event索引可能基于不同duration理解。

要求：compiler一次完成结构、数值、时间、binding、event和policy验证；所有repair只能是显式import migration，产出可审计receipt，不能藏在runtime sample。

验收：malformed corpus逐项映射稳定diagnostic；strict cook零silent repair；permissive migration保留before/after与误差证据。

### ED77-P1-08：事件遍历没有reverse、seek、loop boundary和direction change政策

证据：`clip_event.rs`在`to <= from`时直接返回空；`parameter_apply.rs`允许负speed并把time向零回退。结果是反向播放、反向seek或方向切换均不发事件，且零点/clip duration事件在loop处没有统一一次性归属。

风险：gameplay notify、footstep、damage window和audio cue在reverse/loop时丢失或重复；调用者只能自行补丁并再次分裂语义。

要求：`AnimationTraversal`显式携带from/to、direction、loop count、seek policy、boundary inclusion与event class policy；预编译event index按同一truth table正反遍历。

验收：Forward、Reverse、ForwardLooping、ReverseLooping、multi-loop、seek suppress/fire和direction flip均有边界golden tests。

### ED77-P1-09：事件续传身份依赖event字符串与Vec下标，reimport/reorder后不可稳定恢复

证据：cursor保存playback time、last event string和track Vec index；资产event没有stable ID。相同时间同名事件、改名、插入、排序或reimport会改变cursor所指语义。

风险：budget分页、暂停恢复、hot reload和网络回放可能重复或跳过事件；无法对单个notify做ack、撤销、迁移或diagnostic定位。

要求：source event使用稳定`AnimationEventId`，compiled index生成generation-qualified ordinal；cursor绑定artifact generation和traversal ID，stale cursor必须显式拒绝或resync。

验收：same-time duplicate、rename、insert、reimport与artifact replacement都有确定性resume结果，不靠payload文本识别身份。

### ED77-P1-10：中立事件发布抹掉Clip身份，并把同一逻辑事件以两个类型重复派发

证据：pipeline把Clip event转换成`AnimationEventRecord`时写`clip: None`，随后同时发布record与raw event。Focused test把`clip: None`称为neutral，而不是验证可追溯delivery。

风险：consumer不能按clip instance、player、graph state或generation去重；订阅两种类型的系统会执行两次副作用，失败/拒绝也没有统一receipt。

要求：只发布一个typed `AnimationEventDelivery`，携带source clip、artifact/player instance、event ID、direction、loop ordinal、target、payload schema和delivery sequence；compat raw event必须hard cut或显式adapter去重。

验收：一个逻辑事件只产生一次authoritative delivery；bus rejection、consumer fault和retry均保留同一identity与terminal disposition。

### ED77-P1-11：多个Sequence写同一属性时没有确定性冲突、优先级或混合政策

证据：compiled sequence逐binding/track直接调用writer；多个player或多个track可解析到同一target/property，甚至测试允许把`AnimationPlayer.weight`写为`2.0`。没有property claim、priority、blend operator、range validation或conflict diagnostic。

风险：最终值由asset Vec顺序、world query顺序与system装配顺序决定；动画、Editor preview、live edit和游戏逻辑可互相覆盖。

要求：compiler生成`AnimationPropertyClaim`，playback transaction先收集sample contribution，再按schema定义的exclusive/override/additive/blend operator和priority解析，最后一次性commit。

验收：相同输入在不同容器/query顺序下结果一致；非法多writer、无operator类型和超出property domain值在commit前被拒绝。

### ED77-P1-12：非循环播放没有完成态与terminal contract，只是永久clamp到末帧

证据：player time持续执行`time + dt * speed`，下界只`max(0)`；采样器在duration外clamp端点。没有Completed、StoppedAtEnd、hold/freeze/reset、completion event或一次性terminal transition。

风险：末帧属性被每帧重复写入，事件/graph切换/资源释放无法依赖明确完成时刻，negative/zero speed与duration change更难解释。

要求：`AnimationPlaybackState`定义Playing/Paused/Completed/Stopped/Faulted与end behavior；time advancement返回crossing receipt，terminal副作用恰好一次。

验收：正向末尾、反向起点、零duration、speed change、loop关闭和artifact replacement都有确定性状态转换与一次性completion delivery。

### ED77-P1-13：Editor、Timeline descriptor、Sequence与Clip没有共享时间域和播放提交阶段

证据：Timeline descriptor把speed夹到非负，Editor接受任意finite speed，Sequence time helper做自己的clamp，Clip event又拒绝反向区间；pose、property、event分别在pipeline不同位置提交。

风险：scrub preview、游戏运行和cook verification对同一time给出不同pose/property/event；无法定义event suppression、root motion、pre-animated restore或失败回滚顺序。

要求：消费Editor75的`AnimationTimeDomain`，建立Runtime唯一`AnimationPlaybackTransaction`：advance time -> sample -> resolve pose/property -> collect notify -> extract root motion -> validate currentness -> atomic commit。

验收：Editor preview与game runtime使用同一artifact/kernel/traversal policy；任一阶段失败不会留下半帧property、pose、event或root-motion副作用。

## 6. P2：质量、可观测性与维护性债务

### ED77-P2-01：Binary channel携带`arity`却在decode时不校验

错误arity仍可按value variant进入内存，削弱格式自描述和corruption detection。Decode必须校验value kind、arity、key payload长度与limit，并给出字段位置。

### ED77-P2-02：资产仍以`Option<String>`保存target ID，未使用已有typed `AnimationTargetId`

字符串身份导致反复parse/hash、大小写/格式漂移与弱迁移定位。持久schema应保存稳定typed ID并保留可读source locator，二者职责分离；Editor14/32仍拥有binding UI/import owner。

### ED77-P2-03：Diagnostic去重依赖格式化错误文本

Clip evaluator把`error.to_string()`纳入dedupe identity；文案、浮点格式或localization变化会改变同一故障的identity。应使用stable diagnostic code、artifact generation、source address和bounded arguments。

### ED77-P2-04：Event bytes预算只统计字符串，且允许单条超预算事件穿透

预算没有计入record、Vec、Arc/queue envelope等成本；为了保证进度，oversized first event仍会被发出。应区分hard admission limit与page budget，并用真实owned bytes或保守上界计量。

### ED77-P2-05：Event分页每批重建heap并clone字符串，性能测试还被ignored

这不足以证明大规模同时间notify、长payload、多player场景的稳定帧预算。Cooked index应允许无分配遍历或受控arena，benchmark必须进入可执行性能资格而非长期ignore。

### ED77-P2-06：测试覆盖集中于happy path，且部分测试固化错误语义

内置Sequence只有少量property/stale/topology测试；缺Step精确key、Hermite parity、reverse/loop event、same-time cursor、conflict、completion、source/artifact replacement和Editor-runtime golden。现有Step测试必须先转为RED expected semantics，不能用“已有测试”证明正确。

## 7. 五套参考源码的可执行差距

### 7.1 Unreal：source model、compression、notify、sync与montage是分层系统

Unreal的`IAnimationDataModel`把bone/curve/attribute/timing source数据、controller mutation、modification notification/bracket、evaluation lock和GUID generation分开；`AnimSequence`再承载raw/source到compressed runtime data、additive、retarget和root motion政策。Compression DDC以版本、recipe和依赖构建key，而不是让runtime直接解释source key。

`AnimSequenceBase`、`AnimSync`、`AnimNotifyQueue`和`AnimMontage`还明确处理sorted notify、正反/循环区间、sync marker leader/follower、section/slot/branching point/blend和root motion extraction。Zircon缺的不是几个枚举，而是source/controller/compiler/artifact/playback五层合同。

### 7.2 Godot：typed track与loop/update语义进入资产和Editor命令

Godot Animation区分value、position、rotation、scale、blend shape、method、Bezier、audio与animation track，拥有稳定track API、interpolation/update mode、loop none/linear/pingpong和压缩页结构。Mixer/Player处理root motion、discrete/capture、method/audio/reset等不同副作用；Editor通过UndoRedo修改同一typed model。

这说明generic value channel不能替代method/audio/discrete语义，也不能在采样后统一当作property writer。

### 7.3 Fyrox：UUID identity、双向signal与可恢复preview

Fyrox track、signal均使用UUID；signal passage显式考虑正反方向。Player拥有speed/time/loop/root motion/event queue，Editor command按UUID execute/revert/finalize。Preview进入时快照目标节点，退出或执行命令前恢复，避免预览污染authoring world。

Zircon现有Vec index cursor、字符串target和直接world writer距离该可恢复authoring语义仍有结构差距。

### 7.4 Bevy：typed curve/evaluator与四类event traversal truth table

Bevy用`AnimationTargetId`、typed `AnimationCurve`与evaluator ID绑定目标，event traversal明确区分Forward、Reverse、ForwardLooping、ReverseLooping，并用边界测试固定语义。glTF loader保留cubic triplet/tangent、rotation normalization和wide morph curve。

Bevy适合作为Rust typed runtime与遍历算法参考，但它不提供完整Editor、DDC/compression或montage产品，不能被误写为全功能对标。

### 7.5 Unity Graphics：只作为curve bake/sample与binding迁移的有界证据

本地Graphics仓不是完整Mecanim源码。可用证据是VFX把curve bake与sample expression分开、typed curve slot持有源数据；URP converter对`EditorCurveBinding`到shader property语义有明确转换与测试；motion vector保留多帧history并处理初始化。

本报告不从这些文件推断Unity完整clip/notify/montage能力，只采用“authoring source、baked artifact、typed binding、history currentness分层”这一有限原则。

## 8. 目标架构与唯一authority

### 8.1 Source层

```text
AnimationSourceDocument
  identity: AssetId + SourceRevision + SchemaVersion
  time_domain: rational tick resolution + display rate + duration policy
  documents:
    PropertySequenceSource
    SkeletalClipSource
    AnimationActionSource
  stable ids:
    BindingId / TrackId / ChannelId / KeyId / EventId / MarkerId / SectionId
  typed data:
    ChannelRole / ValueSchema / Interpolation / Tangent / EventSchema
  policies:
    additive / root_motion / sync / completion / compression_recipe
```

Source只表达作者意图和可迁移数据。Import repair必须生成receipt；Editor mutation必须通过transaction/controller，禁止运行时在sample时静默修复。

### 8.2 Compiler与Artifact层

`AnimationSemanticCompiler`是Editor76与本报告共享的唯一入口，分阶段产出：

1. `AnimationValidationReport`：结构、数值、time、binding、dependency、event、policy。
2. `CompiledPropertySequence`：prepared channel blocks、resolved writer program、property claims。
3. `PreparedAnimationClip`：target slots、compressed/streamable channel pages、curve/morph/attribute data。
4. `CookedEventIndex`：stable event ID、direction/loop检索、payload schema与预算元数据。
5. `RootMotionTrack`与`SyncMarkerTable`：独立于普通bone pose，具有明确提取/消费政策。
6. `AnimationActionArtifact`：section/slot/branching point/montage语义，依赖Runtime08C平台里程碑。

每个artifact必须包含source hash、schema/compiler/recipe version、dependency generations、platform variant、error metrics和content hash；publication只原子替换last-good generation。

### 8.3 Playback层

```text
AnimationPlaybackTransaction
  1. validate player/artifact/world generation
  2. advance AnimationTimeDomain and emit TraversalReceipt
  3. sample prepared pose/property/curve contributions
  4. resolve blend, additive and property claims deterministically
  5. collect typed event/notify deliveries
  6. extract root motion and sync result
  7. revalidate currentness
  8. commit pose + property + event + root motion atomically
  9. publish PlaybackReceipt / terminal state
```

Editor preview、PIE和game runtime必须调用相同artifact与kernel，只通过policy决定是否commit authoring world、是否suppress gameplay event、是否restore pre-animated state。

### 8.4 Binding与冲突

`AnimationBindingSchemaResolver`把source locator解析为typed target/property program，并返回domain、blend operator、ownership和range。Compiler建立claim table；Runtime先收集contribution再按exclusive/override/additive/blend规则解析，禁止遍历顺序决定结果。

## 9. 重构里程碑

### ED77-M0：Owner、truth table与RED证据冻结

- 与Editor14/32/75/76、Runtime08C冻结字段和owner，禁止新建第四份schema或第三份sampler。
- 写出time/interpolation/event/completion/conflict truth table。
- 先添加Step精确key、glTF Linear/CubicSpline、reverse/loop event与partial compile RED fixtures。

### ED77-M1：Versioned source schema与stable identity

- 建立`AnimationSourceDocument`、stable IDs、typed channel/event/time policy。
- 为三份旧schema提供确定性migration与round-trip receipt。
- 旧`AnimationTimelineDescriptor`硬切或迁为唯一source投影，不留并行写authority。

### ED77-M2：统一validation、compiler与prepared artifact

- 合并Sequence/Clip channel validator和sampler。
- 产出自包含`CompiledPropertySequence`、`PreparedAnimationClip`、diagnostic与last-good publication。
- Pipeline不再用`None/continue`吞掉编译和提交失败。

### ED77-M3：Interpolation conformance与glTF golden

- 修正Step exact-key和interval边界。
- 明确Linear、cubic scalar/vector/quaternion与tangent normalization。
- 与Editor32一起用glTF linear/step/cubic、rotation、morph fixtures完成import -> artifact -> sample golden。

### ED77-M4：Event/Notify与双向遍历

- stable EventId、cooked index、generation-qualified cursor。
- Forward/Reverse/Looping/seek truth table和typed single delivery。
- 建立payload schema、budget admission、fault/retry receipt。

### ED77-M5：Playback transaction、冲突与完成态

- 引入Traversal/Playback/Apply receipt、terminal state和end behavior。
- property claim/arbitration、range validation和atomic commit。
- 失败不留下半帧pose/property/event/root motion。

### ED77-M6：Compression、cook、DDC与source/prepared分离

- 与Runtime08C和Asset owner建立recipe/version/platform key。
- 加入压缩误差、内存、decode吞吐、streaming page与determinism receipt。
- Runtime shipping路径不得读取source key Vec或同步解析文本/通用序列化。

### ED77-M7：Root Motion、Additive、Sync与Action/Montage集成

- 依赖Runtime08C平台，建立root motion extraction/consume policy、additive base、sync group/marker。
- 再引入section/slot/branching point/action artifact；不得把event字符串临时改名为montage。

### ED77-M8：Editor Sequence/Clip真实产品与runtime preview

- 依赖Editor14/75完成typed track/key/event/curve产品和transaction。
- Preview使用同一artifact/playback transaction，进入/退出恢复目标状态。
- Compile/save/cook diagnostic映射stable source address，禁止静态success。

### ED77-M9：性能、故障与跨引擎资格

- 大骨骼/长clip/多player/密集event/多property writer场景做allocation、cache、decode与提交profile。
- 覆盖hot reload、stale cursor、asset replacement、bus rejection、consumer fault与shutdown。
- 只在同场景、同质量、同硬件、同构建配置下比较Unreal/Fyrox/Bevy/Godot/Unity可比子能力。

## 10. 48个资格门

当前静态状态：**ED77-G01至ED77-G48全部Fail**。已有局部unit test不等于整条source -> compiler -> artifact -> playback -> Editor product通过。

| Gate | 资格 | 当前 |
|---|---|---|
| ED77-G01 | 三份旧动画schema都有唯一owner与字段映射表 | Fail |
| ED77-G02 | `AnimationTimelineDescriptor`不再是无人消费的并行authority | Fail |
| ED77-G03 | Binding/Track/Channel/Key/Event具有持久stable ID | Fail |
| ED77-G04 | time base使用确定性tick/rational contract | Fail |
| ED77-G05 | binary decode校验version、limit、arity与payload shape | Fail |
| ED77-G06 | source round-trip不静默clamp或改写语义 | Fail |
| ED77-G07 | Sequence与Clip共享一个semantic validator | Fail |
| ED77-G08 | required binding失败不能partial publish | Fail |
| ED77-G09 | optional binding disposition具有stable diagnostic | Fail |
| ED77-G10 | artifact拥有source/compiler/recipe/dependency identity | Fail |
| ED77-G11 | artifact离开source asset后可独立sample | Fail |
| ED77-G12 | last-good artifact按generation原子替换 | Fail |
| ED77-G13 | 内置Sequence与插件Clip只使用一个sampling kernel | Fail |
| ED77-G14 | Step在精确key及ULP边界符合truth table | Fail |
| ED77-G15 | Linear scalar/vector/quaternion golden通过 | Fail |
| ED77-G16 | Cubic scalar/vector tangent golden通过 | Fail |
| ED77-G17 | Cubic quaternion有真实数学语义或被compiler拒绝 | Fail |
| ED77-G18 | type/tangent mismatch不再静默left/zero fallback | Fail |
| ED77-G19 | glTF Step/Linear/CubicSpline导入到采样golden通过 | Fail |
| ED77-G20 | glTF morph/rotation normalization与wide curve通过 | Fail |
| ED77-G21 | key duration/range/finite/order合同一次编译完成 | Fail |
| ED77-G22 | event duration/range/payload schema编译验证完成 | Fail |
| ED77-G23 | Forward event边界矩阵通过 | Fail |
| ED77-G24 | Reverse event边界矩阵通过 | Fail |
| ED77-G25 | ForwardLooping与multi-loop矩阵通过 | Fail |
| ED77-G26 | ReverseLooping与direction flip矩阵通过 | Fail |
| ED77-G27 | seek suppress/fire policy可配置且确定性 | Fail |
| ED77-G28 | event cursor绑定stable ID与artifact generation | Fail |
| ED77-G29 | same-time/reorder/reimport分页不重不漏 | Fail |
| ED77-G30 | 一个逻辑事件只有一个typed authoritative delivery | Fail |
| ED77-G31 | delivery保留clip/player/event/direction/loop identity | Fail |
| ED77-G32 | event hard admission与page budget均有证据 | Fail |
| ED77-G33 | player具有Playing/Paused/Completed/Stopped/Faulted合同 | Fail |
| ED77-G34 | 非循环末尾/起点只产生一次terminal receipt | Fail |
| ED77-G35 | loop、speed change、duration change结果确定性 | Fail |
| ED77-G36 | 多writer property claim在compile期可见 | Fail |
| ED77-G37 | property conflict与blend不依赖遍历顺序 | Fail |
| ED77-G38 | property domain/range在commit前校验 | Fail |
| ED77-G39 | pose/property/event/root motion按事务原子提交 | Fail |
| ED77-G40 | currentness失败不留下半帧副作用 | Fail |
| ED77-G41 | root motion提取与消费policy可验证 | Fail |
| ED77-G42 | additive base、sync group/marker合同可验证 | Fail |
| ED77-G43 | source与compressed/prepared shipping数据物理分离 | Fail |
| ED77-G44 | compression error/memory/decode/determinism receipt通过 | Fail |
| ED77-G45 | Editor preview与game runtime使用同一artifact/kernel | Fail |
| ED77-G46 | preview退出、切asset、undo与fault可恢复authoring state | Fail |
| ED77-G47 | 大clip/多player/密集event hot path达到既定零分配/预算门 | Fail |
| ED77-G48 | 同质量同场景跨引擎正确性与性能资格通过 | Fail |

## 11. 实现顺序、依赖与停止条件

1. 先执行ED77-M0，不允许直接给当前字符串target、Vec index cursor或双sampler继续加功能。
2. ED77-M1依赖Editor32确认import owner，ED77-M2依赖Editor76确认唯一compiler/runtime authority。
3. ED77-M3/M4/M5先闭合可执行语义，再做M6 compression；不能用压缩掩盖错误sampling。
4. ED77-M7必须等待Runtime08C的root motion/sync/montage平台合同，M8必须等待Editor14/75的产品与transaction合同。
5. 任一里程碑若仍用`.ok()?`、`continue`、字符串identity、source Vec下标或遍历顺序代替typed receipt/currentness，停止进入下一里程碑。

这一路径受`docs/plans/mvp/index.md`的F0-F5依赖链约束。Review完成不改变MVP实施状态；没有产品E2E、fault、profile和同语义benchmark evidence时，不得宣称Animation Sequence/Clip production-ready或性能超过参考引擎。

## 12. 本轮验证边界

- 已完成：当前工作树静态源码逐文件复核、focused test意图核对、五类本地参考源码对照、owner去重、冻结语料fingerprint与48门建账。
- 未执行：Cargo check/test、Editor启动、save/reopen/reimport、runtime preview、cook/DDC、GUI/GPU、native input、fault/soak/profile、跨引擎benchmark。
- Review结论：`review_status: complete`只表示Editor77所声明物理范围已完成本轮静态审查；`implementation_status: not_started`与48门全Fail是当前真实交付状态。
