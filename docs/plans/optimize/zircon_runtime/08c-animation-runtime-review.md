---
related_code:
  - zircon_runtime/src/animation
  - zircon_runtime/src/core/framework/animation
  - zircon_runtime/src/scene/level_system/animation_runtime.rs
  - zircon_runtime/src/asset/importer/ingest/gltf_animation_subassets.rs
  - zircon_plugins/animation/plugin.toml
  - zircon_plugins/animation/runtime
  - zircon_plugins/animation/editor
  - zircon_plugins/animation/dist
  - zircon_plugins/animation_graph
  - zircon_plugins/gltf_importer/runtime/src/subassets.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/zircon_plugins/04-animation.md
  - docs/plans/performance/01/2026-07-22-runtime-animation-static-review.md
  - docs/plans/zircon_plugins/04/failure-2026-07-22-runtime-animation-fallback-evaluator-divergence.md
  - docs/plans/zircon_plugins/04/failure-2026-07-29-animation-frame-diagnostics-hardcut-omission.md
  - docs/plans/zircon_runtime/runtime/11/failure-2026-07-29-dynamic-runtime-animation-module-duplication.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Animation/AnimInstance.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Animation/AnimNodeBase.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Animation/AnimSequence.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/Animation/AnimSync.h
  - dev/godot/scene/animation/animation_mixer.h
  - dev/godot/scene/animation/animation_player.h
  - dev/godot/scene/animation/animation_tree.h
  - dev/godot/scene/3d/skeleton_3d.h
  - dev/Fyrox/fyrox-animation/src/track.rs
  - dev/Fyrox/fyrox-animation/src/pose.rs
  - dev/Fyrox/fyrox-animation/src/machine/layer.rs
  - dev/bevy/crates/bevy_animation/src/lib.rs
  - dev/bevy/crates/bevy_animation/src/graph.rs
  - dev/bevy/crates/bevy_animation/src/transition.rs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 08C · Animation Runtime 工程化差距

## 1. 结论

Zircon Animation 不是纯占位实现。当前插件已经有真正的 `animation.evaluate` scene system、typed ECS projection、asset revision 检查、compiled skeleton target table、compiled clip/graph/state-machine cache、PosePool、四个 direct-clip worker shard、clip/graph/state-machine/sequence 求值、layer/mask/additive blend、two-bone/look-at IK，以及 animation 到 physics 的 `SkeletalPoseTargets` 和 simulated pose 回灌。最近的 current-source 修改还补入 world replacement epoch、事件 admission/defer、可续采样游标、生产批次原子性和 deferred entity 的时间/姿态/状态回滚。事件入口按范围数、事件数、字节和单次时间跨度设有硬预算。这些都是值得保留的正确基础，不能在重构时退回全场景字符串扫描、无界事件或无 replacement generation 的旧 hook。

但从“导入一个带骨骼动画的 glTF，在 App/Editor 中稳定播放并由 GPU 蒙皮显示”这条最短产品链看，当前仍存在数个阻断级断点。高优先级 first-party `gltf_importer` 把动画导成写有“not implemented yet”的 `DataAsset`，而低优先级 Runtime builtin importer 才会产生 `AnimationClipAsset`；builtin 又把非根骨骼的 leaf name 写进要求完整 canonical path 的 `target_id`，导致 compiled target table 无法解析。inverse bind matrices 被装进无人消费的通用 JSON data asset，GPU skinning readiness 永远是默认 disabled，所谓 `SkinningPaletteDoubleBuffer` 只在 CPU 上 clone `Vec`，没有 renderer upload consumer。因此公开 asset、runtime、render surface 并没有形成 skinned mesh 产品闭环。

运行时还有一个直接影响产品正确性的缺口：`animation_frame_demand` 依赖 `LevelSystem::animation_requires_continuous_frame()`，但全仓生产代码没有任何调用点在动画播放时写入 `animation_requires_continuous_frame`；只有测试和 failed-tick reset 访问该位。事件 backlog 会单独保持帧循环，但没有事件的普通 clip/graph/state machine 在反应式宿主中可能只推进一次。这个问题说明现有 pipeline tests 不能替代 App/session cadence 验收。

性能上，typed projection 和 cache 只解决了第一层问题。每帧仍扫描所有 animation candidates、逐实例取得 asset revision、构造多组 `BTreeMap/BTreeSet/Vec`、clone graph/state-machine parameter map、同步 load owned asset、为每个 clip 输出重新分配带 `String` bone name 的 AoS pose，并深比较完整 pose/playback map决定是否发布。graph/state-machine/IK/pose apply 主要串行；四个 worker 只覆盖 direct clip，且 main thread 每帧建立 channel、提交最多四个任务并阻塞接收。最后又把每根骨骼写回通用 scene node transform。这套路径无法以 1,000 个角色、数十万骨骼和严格主线程预算扩展，也没有证据能支持“性能优于 Unreal”。

本轮登记 20 项 P1、5 项 P2，没有新增 P0。优先级不是按“功能听起来高级”排序，而是先恢复真实产品闭环与唯一 authority，再建立 prepared artifact、instance runtime、dense pose、任务图、renderer/physics bridge和产品级 authoring。Retarget、root motion、sync/montage、motion matching和 Control Rig 必须进入架构，但不能在基础资产身份、帧调度和 pose ownership仍错误时通过更多 DTO 堆叠实现。

## 2. 审查边界与覆盖

### 2.1 已读范围

| 范围 | Rust 文件 | Rust 行数 | `#[test]` | 证据等级 |
|---|---:|---:|---:|---|
| `zircon_runtime/src/animation` production | 16 | 1,998 | 10 | E3：manager/module、clip event sampler、compiled sequence与scene apply |
| `core/framework/animation` | 38 | 3,210 | 5 | E3：asset、player、graph/state machine、event、IK、GPU/readiness与manager contract |
| animation runtime `src` | 134 | 12,448 | 26 | E3：pipeline、compiled evaluator、cache、worker、IK、manager、plugin/system |
| animation runtime integration tests | 26 | 5,330 | 99 | E2/E3：算法和world行为较强，产品/render/cook/scale证据不足 |
| animation editor/dist | 6 | 293 | 4 | E2：package、descriptor、inspector registration |
| animation graph package | 6 | 923 | 11 | E2：validation/registration；无真实graph document/editor/runtime artifact compile |

物理文件统计以 2026-08-15 当前工作区为准。production 与 test 代码中共发现 163 个 `#[test]`；上述子树合计 11 次 `include_str!`、88 次 `.contains(...)`。未发现 Criterion/`#[bench]`、property test、Loom、sanitizer、真实 GPU skinning capture、1k-character product benchmark、跨平台 soak 或 App/Editor reactive cadence evidence。现有测试对 channel interpolation、compiled target、graph/state machine、event budget、IK 和 world replacement 提供了有价值的行为保护，但也有大量 source-shape/registration assertion，不能把测试数量直接解释为产品完成度。

沿产品链额外复核了 dynamic session construction/frame demand、LevelSystem animation state、scene transform apply、project asset manager、core/plugin glTF importer、physics pose bridge、renderer skinning binding和已有 animation/performance/failure记录。`zircon_plugins/animation/runtime` 与 core animation manager/IK 当前有其他 Session 的大量修改；本报告按 current source 记录其新增保护，但实现前必须重新读取 overlapping diff、取 fingerprint 并运行新鲜门禁，因此标记 `source_recheck_required`。

### 2.2 参考边界

- Unreal 用 `Initialize/CacheBones/Update/Evaluate` 分相、AnyThread contract、required-bone/LOD、parallel evaluation、compressed platform data、sync group/marker、montage、root motion和retargeting建立完整执行与内容链。Zircon不应复制它的UObject层次，但必须吸收prepared data、明确phase、实例状态和产品证据边界。
- Godot 把 AnimationMixer/Player/Tree 与 Skeleton3D modifier、skin/server update 分开，说明动画逻辑、骨架姿态和渲染提交不应通过通用scene node逐骨骼写回混成一个owner。
- Fyrox 的track/pose/machine layer保留稳定binding、fetch/cursor、pose map、mask和transition owner；它可作为Rust语义与asset/runtime分层参考。
- Bevy 的AnimationTarget、graph/evaluator/transition展示了ECS target identity、active graph和transition component数据流；它不是Zircon规模性能结论的替代品。
- Unity Graphics参考树属于renderer/deformation owner。本篇只把GPU skinning桥登记为跨域合同，buffer layout、upload、compute deformation、culling和render graph实现由09/10图形审查展开，避免用graphics源码反推状态机语义。

### 2.3 明确未做

- 没有改production code，没有运行Cargo、App、Editor、GPU capture、跨平台、fault、soak或规模性能测试。本篇是静态current-source审查和重构计划，不是实现验收。
- 没有把缺少Unreal某个类或节点直接判为缺陷。P1只包含产品闭环、正确性、owner/lifecycle、可扩展性和已公开能力真实性；高级工作流进入P2。
- 没有否定最近加入的replacement epoch、事件背压和revision projection。它们是后续generation-safe pipeline的起点，但尚不足以证明完整事务或规模性能。

## 3. 当前闭环与必须保留的能力

### 3.1 插件 scene system 已经是真实 production evaluator

Animation plugin 在 `PostUpdate` 注册 `animation.evaluate`，显式排在 world transform 之后。tick 会扫描typed animation component query，解析manager/settings/asset manager，评估clip、sequence、graph、state machine、layer、physics blend与IK，写回pose并发布immutable snapshot。与Audio 08B不同，它不是只有manager API而没有scene caller。重构必须保持scene schedule identity与单一production system，不得恢复已经退役的Runtime `scene_hook`。

### 3.2 compiled target、revision projection和有界事件是正确方向

`SkeletonTargetTable` 把canonical path映射成dense slot，并对cycle、duplicate和ambiguous leaf进行typed error；clip/graph/state machine有revision-keyed cache。projection保留QueryState和component change tick，paused且revision未变化时可跳过采样。clip event queue按producer batch原子admit，最多保留256个range，单次drain最多32个sample、64个event、64 KiB和1秒playback span，支持cursor继续；world replacement会清空backlog。后续应把相同generation/backpressure原则推广到pose、task、renderer和Editor，而不是删除这些合同。

### 3.3 state machine、layer、IK和physics bridge已有可演进骨架

当前state machine覆盖clip、graph、1D/2D blend space、nested machine、layer、transition、exit time与interruption；IK支持two-bone和look-at，并有按world有界队列；`SkeletalPoseTargets` 与 `SimulatedPoseFeed` 形成animation/physics双向边界。这些能力还不等于完整AnimGraph/Control Rig/ragdoll系统，但已经足以作为MVP runtime instance和phase拆分的输入，不能另起第二套API。

## 4. P1 差距清单

### P1-1：core与plugin仍拥有同名module/manager实现，唯一authority没有物理收敛

`zircon_runtime/src/animation/module.rs` 与 `zircon_plugins/animation/runtime/src/module.rs` 都声明 `animation.runtime`，两边manager目录也维护近似相同的配置、IK queue和service实现。dynamic session目前按是否链接animation plugin选择其一，避免同一session重复注册，这是近期正确修复；但fallback仍是一套可独立演进的manager实现，而production evaluation只在plugin。相同公开identity横跨两个build boundary，当前IK replacement-epoch修改也必须在两边同步，已经证明漂移成本。

目标硬切为一个真实implementation owner。`zircon_runtime`只保留稳定contract、asset/scene absorption和明确Unsupported capability route；animation plugin拥有manager、runtime instance、scene system与诊断。未链接plugin时不得悄悄启动第二套manager/evaluator；应发布typed unavailable reason。删除重复实现前需要linked/unlinked动态会话、native unload、manager handle staleness和world replacement回归。已有open failure不能仅因注册分支修过就关闭，必须以物理owner收敛和current-source gate为准。

### P1-2：播放状态没有驱动reactive frame demand，普通动画可能只推进一帧

session frame demand仅在 `LevelSystem::animation_requires_continuous_frame()` 为真时返回Immediate。生产animation pipeline没有调用 `record_animation_requires_continuous_frame(true/false)`；唯一非测试caller是failed tick时写false。event queue的独立backlog位能继续唤醒有待排事件的帧，但无event clip、graph或state machine没有此保护。

目标由animation runtime在同一replacement epoch中发布 `AnimationFrameDemandSnapshot`，内容至少包含active advancing instances、pending tasks、event backlog、transition/sequence、paused-but-dirty和next scheduled wake。session只消费immutable generation，不推断pose map是否非空。空world、pause、speed=0、once-at-end、asset unavailable、world replacement和failed tick都要有exact transition测试；App与Editor在reactive cadence下实测持续播放、最终回idle和无busy loop。

### P1-3：glTF importer authority反转，高优先级first-party插件把动画降级成placeholder

Runtime builtin `zircon.builtin.model.gltf` priority 10 能生成 `AnimationSkeletonAsset` 和 `AnimationClipAsset`。first-party `gltf_importer.gltf` priority 120却为每个animation输出通用 `DataAsset`，文字明确写着channel import未实现；它只导skin与inverse bind data。安装正常插件后，产品更可能选择更高优先级但更不完整的实现，形成“fallback比正式插件完整”的反向能力。

目标选择一个canonical glTF decode/cook owner，插件和builtin不得各维护一套subasset逻辑。若插件是产品owner，应迁入已验证的skeleton/clip逻辑后删除builtin重复路径；若Runtime owner被保留，插件只能注册该owner而不能复制/占位。importer descriptor必须列出真实additional outputs和feature status。门禁从真实多骨、skin、多个animation、external buffer glTF经过import、asset resolve、runtime sample、render capture，不能只断言placeholder/JSON shape。

### P1-4：builtin glTF对非根骨骼生成错误target_id，导入资产可在compiled evaluation时失配

`GltfTrackBuilder::new` 把 `target_id` 设置为单个 `bone_name`，例如 `Node1:Hand`。runtime `resolve_track_target` 只要看到target_id就把它作为完整canonical `EntityPath` hash；`SkeletonTargetTable` 对该骨骼的id却来自 `Node0:Root/Node1:Hand`。非根骨骼因此找不到slot，并且不会回退到unique bone name。现有import test与compiled target test分别验证各自局部输入，没有覆盖“importer输出直接进入evaluator”的组合链。

目标在import hierarchy阶段生成与skeleton table同源的stable path/target ID，或让cook阶段从统一skeleton identity重写track slot；禁止同时保存互相可能冲突的leaf/path真相。增加duplicate leaf、multi-root、skin joint subset、animation-only hierarchy、rename/reimport和round-trip测试。非法/失配track必须让asset cook失败并定位animation/channel/node，不得在frame tick中silent skip。

### P1-5：Skeleton、Skin、inverse bind、mesh与clip没有形成一个版本化资产关系

core importer把inverse bind matrices单独序列化成 `kind=gltf_inverse_bind_matrices` 的JSON `DataAsset`；全仓没有animation/render consumer。plugin importer的skin也没有生成AnimationSkeleton。runtime pose以skeleton asset为准，renderer skinning binding没有获得joint remap、inverse bind、mesh skin index或asset generation。skeleton target path、scene descendant name和mesh joint顺序分别存在，缺少同一identity。

目标建立 `SkeletalRigCookArtifact`：stable rig id/generation、parent table、bind local/model pose、inverse bind、joint-to-dense-slot、mesh skin remap、retarget metadata、bounds和format version。clip artifact依赖rig signature；skinned mesh依赖同一rig/skin signature。reimport先prepare所有artifact与dependency graph，成功后原子publish generation；旧instances/renderer lease继续使用旧generation直到retire。JSON debug view可以派生，但不能是runtime真相。

### P1-6：原始String/AoS asset直接进入帧循环，缺少压缩与平台prepared artifact

`AnimationClipAsset`保留完整typed key `Vec`、每track bone name/target string和f32值。runtime首次使用时才clone channel并compile；没有cook-time sorted/finite validation receipt、quantization、constant track elimination、segment index、streaming chunk、platform/profile key、compressed bytes budget或derived data cache。Unreal `AnimSequence`明确区分raw/editor和platform compressed data并有derived-data key；Zircon目前无法控制大项目导入时间、包体、residency和sample bandwidth。

目标建立versioned `AnimationClipCookArtifact`：rig signature、dense track slots、constant/default masks、quantized transforms、segment/page table、event index、root-motion/morph curve、compression error report和target profile hash。Editor保留source/editable data，runtime只消费prepared artifact。短clip常驻，长clip按segment lease/预取；missing page有hold/fallback/diagnostic而不是同步load。质量门记录per-bone position/angle/scale error、root trajectory error、decode ns/bone和bytes/bone-second。

### P1-7：帧内同步asset load与owned clone破坏residency和延迟边界

clip sample会调用 `load_animation_skeleton_asset`/`load_animation_clip_asset`，graph timing为算signature会遍历并load所有贡献clip，state machine与sequence也在frame中load owned asset。IK每command再次load skeleton。Resource/Asset 04已确认这些load接口返回owned clone且residency同步；animation cache并没有消除进入cache前的load/clone和revision查询。失败路径多用 `.ok()?`/`continue`，graph/state/sequence缺失可能只表现为不动。

目标在scene admission/asset change阶段解析 `AnimationPreparedSetLease`，包含rig、clip/graph/state-machine/sequence artifact generation及所需stream pages。frame evaluation只接受ready lease和stable handle；异步prepare有single-flight、cancel、world epoch、last-good与错误负缓存。asset unavailable产生typed instance state和一次性diagnostic，Editor可追踪dependency；不得在每帧重试同步load，也不得把空pose当作成功。

### P1-8：pose公共形态以bone String为中心，PosePool后仍每sample分配和clone

PosePool复用SoA临时缓冲，但 `sample_compiled_pose` 最终仍创建新的 `Vec<AnimationPoseBone>`，每根骨骼clone name。graph为每个贡献clip生成完整pose，构造base/additive Vec，再在layer间往返PoseBuffer与public output。presentation snapshot和physics target又把pose转成带bone name的对象。缓存只减少channel定位，未建立steady-state dense pose page。

目标让runtime内部只使用rig-scoped dense `PoseBuffer`/`PosePage`，SoA transforms、valid/additive masks、generation和scratch lease；bone name只在diagnostic/Editor边界按需解析。instance evaluation写入双/三缓冲page，graph/layer/IK/physics在同一dense layout工作，renderer直接消费palette/deformation input。public inspection API返回Arc snapshot或按选中实体materialize，不能迫使所有角色每帧构造String AoS。

### P1-9：每帧仍扫描全部animation candidates并重建多组容器

projection避免扫描完全无关node，但每帧仍遍历所有skeleton/player candidates，逐实体取得asset revision，并新建scan结果、revision map、pose source set、update Vec、event batch和playback map。paused unchanged只是在完成candidate/revision工作后跳过sample。graph/state-machine parameters以 `BTreeMap` clone进入求值；playback times在LevelSystem外部map中重建。

目标建立增量 `AnimationInstanceRegistry`：component add/remove/change、asset generation、visibility/LOD、playback command和world replacement通过dirty queue更新slot；active advancing instances在dense active list，paused clean不进入frame scan。参数存typed slot page并以change mask提交。frame phase预留scratch arena，按instance count复用；统计candidate/dirty/active/evaluated/applied数量和allocated bytes。1/100/1k/10k entities及paused/culled分布必须有scale curve。

### P1-10：direct clip worker是假局部并行，任务生命周期与cache locality不稳定

当前最多四个direct evaluator，各自持skeleton/clip cache；round-robin会让同一资源在多个shard重复compile/resident。每帧创建sync channel、向scheduler提交最多四个task，然后main thread阻塞 `recv`；schedule返回值被忽略，channel failure会panic。graph/state machine、sequence、IK、blend、event和pose apply仍在owner thread，工作多时Amdahl瓶颈明显。

目标用Runtime11 job system表达明确phase DAG：Update/transition/event intent在owner lane；pure sample/decompress/graph evaluation按instance batch并行；blend/IK可按独立rig batch并行；commit/apply按确定顺序回owner。prepared artifact cache是共享immutable generation，worker scratch按thread/arena复用。每个task带world epoch、deadline/cancel和result slot；submit/reject/panic/stale结果都有typed处理。禁止每frame新建channel或靠阻塞recv组织join。

### P1-11：graph masked blend按全局权重归一，可能使未被其他输入覆盖的骨骼衰减

graph先计算所有base clip总权重，再对每个clip按这个全局总量归一。blend时target mask会跳过不属于该输入的bone，但不会按bone重新归一剩余贡献。两个各0.5的base clip中，第二个只mask手臂时，腿只收到第一clip的0.5权重，translation/scale/quaternion可能被错误缩小，而不是完整继承第一clip。现有graph功能测试没有覆盖这种per-bone normalization oracle。

目标在graph compile时生成每node的dense bone mask和每bone contributor plan，明确base/reference/additive缺失输入语义；evaluate按bone归一或以reference pose补足，quaternion采用稳定hemisphere/normalized blend。加入disjoint/overlap/nested masks、zero/negative/NaN weights、duplicate target、additive base和order determinism金样。compiled plan复用，不在每instance递归创建weighted pose Vec。

### P1-12：graph cache与求值仍按clone参数/线性项管理，不是可扩展实例程序

graph每次求值会解析节点、构造贡献pose和字符串/Vec；per-frame evaluation cache最多256项，以包含clone parameter map的key做线性比较，满时 `remove(0)`。graph timing cache在命中前仍load所有clip计算signature。graph player本身缺少完整的speed/loop/weight/local time owner，时间存于LevelSystem外部BTreeMap。

目标把authoring graph编译成immutable `AnimationProgramArtifact`：topological instruction stream、dense parameter layout、clip/pose slots、mask pages、sync metadata、scratch size和dependency generations。每instance只保存program handle、parameter page、node state、time和output page；共享相同program的实例按batch执行。cache用generation key和bounded policy，不在frame中clone parameter map或O(n)逐项比较。

### P1-13：state machine trigger不是一次性语义，运行时实例状态分散

`AnimationParameterValue::Trigger` 只要仍存于player parameter map就持续满足 `Triggered` condition。全仓没有production consume/reset/remove路径，因此一次trigger可在后续任何满足from-state条件的帧重复触发。active state写回scene component，graph/state-machine time和transition却存在LevelSystem map，interrupted pose/runtime state在pipeline cache；同一实例真相分散在三个owner，事件defer时需要深clonecheckpoint回滚。

目标建立单一 `AnimationMachineInstance` slot，持active/nested/layer state、transition stack、node clocks、one-shot trigger bits、parameter generation和rollback journal。authoring component只保存asset/config/default parameter，不每帧写runtime active state。trigger在确定transition arbitration后消费，并定义同帧多transition、deferred event、interruption和rollback语义。状态更新、事件生成与pose sample共享同一transaction generation。

### P1-14：state machine缺少生产级arbitration、同步和诊断合同

当前已有exit time、condition、duration、nested/layer和interruption基础，但没有明确transition priority/ordered arbitration、any-state/conduit、sync group/marker、state alias、cached pose、inertialization或per-node update rate。compile/asset failure在部分runtime路径中可能silent continue。`AnimationRuntimeStatus`、player/rig status与tick report主要是默认合同和测试，没有pipeline producer或Editor consumer。

目标先完成MVP deterministic machine contract：ordered priority、one transition decision per phase、any-state policy、exit/loop边界、sync marker、interruption source/destination、typed compile/runtime diagnostic和instance trace。高级node不必一次全做，但未支持能力必须在asset compile与Editor palette禁用。runtime status由真实pipeline counters/generation产生，不能保留永远default的“完整”API。

### P1-15：event入口有背压，但clip内采样与下游发布仍不完整

有界range queue与cursor值得保留；然而 `ProjectAnimationClipEventSampler` 每个range同步load owned clip，收集candidate后反复 `min_by` 选择下一event，最坏可接近O(E²)。event track没有在cook/clip compile阶段排序索引。publish同时构造clone payload的 `AnimationEventRecord` 并发送原始 `AnimationClipEvent`，而仓外没有确认到真实gameplay consumer；ECS单帧event容器的总预算也不由animation queue保证。

目标把event track编译为按time排序的dense table与loop segment index，cursor直接二分/顺序推进。定义一个canonical gameplay event ABI、world/entity/clip/instance generation、delivery phase、late/stale策略和consumer cursor；需要Editor trace时从同一record投影，禁止双份字符串payload。下游EventBus/ECS ingestion也要按count/bytes/time有界，critical event和cosmetic event可采用不同policy。产品测试必须有脚步声/gameplay consumer，而非只检查world里出现event。

### P1-16：sequence/timeline仍逐帧load和解释通用scene property mutation

pipeline每帧load active `AnimationSequenceAsset`，compiled sequence虽存在revision cache，但最终通过通用scene property writer应用。它与skeletal graph拥有独立time/update路径，缺少统一clock、scrub、section、binding generation和event transaction。大量property track可能在animation owner thread逐条修改World，并与Editor timeline/undo或其他system写同一属性发生顺序冲突。

目标把sequence作为独立但同clock-domain的compiled program：binding在scene generation变化时重编译，track按typed property slot分组，evaluate生成bounded mutation buffer，commit phase统一处理conflict/priority。skeletal sequence引用应提交animation instance command，而不是绕过graph直接写bone node。Editor scrub使用preview world和明确restore transaction；runtime loop/seek/event共享clock与replacement epoch。

### P1-17：IK command执行仍同步load、重复建model pose且缺少rig phase合同

two-bone/look-at有typed command和4096/world有界队列，近期加入replacement epoch，这是正确基础。但每个command会load/clone skeleton，构造 `Vec<Option<ModelBone>>` 与visiting状态；two-bone求解中多次重建model pose。`drain_ik_commands_excluding`把deferred set转Vec再对每command线性contains。多个IK命令没有显式priority/group/order或冲突语义，也缺少joint limit、effector orientation、stretch/twist和contact owner。

目标把IK作为compiled rig program phase，使用同一dense rig/model-pose scratch与prepared artifact；commands按instance slot、phase、priority稳定排序，deferred lookup用dense bitset/set。基础two-bone/look-at先补joint limits、orientation、weight、invalid-chain diagnostic和multi-command determinism。Control Rig/full-body/foot locking进入P2，但MVP必须证明IK不触发asset load、每rig只构建一次model pose并能与physics blend顺序稳定组合。

### P1-18：姿态写回通用scene node，无法支撑大规模骨架与清晰ownership

`apply_pose_transforms_to_scene_nodes`按skeleton entity后代name建立binding，然后每active frame逐bone调用scene transform mutation。runtime target table使用full path，pose output和scene binding又退回bone name；duplicate leaf或层级变化可能产生歧义。更重要的是每根骨骼成为通用scene node脏变更，会触发world generation、hierarchy/derived transform路径和其他observer，10k角色无法承担这种写放大。

目标让Skeleton Pose成为专用component/storage owner，不把所有bone暴露为普通entity。animation输出local pose page；skeleton transform system可按LOD计算需要的model bones；renderer消费palette/deformation buffer；physics消费指定body bones；attachment/socket系统只把选定bone transform投影到scene entity。Editor骨架树通过inspection adapter显示虚拟bone，不要求每bone实体化。需要兼容已有scene skeleton时做一次性hard migration，不保留双写。

### P1-19：GPU skinning、morph与render bridge是false-green surface

`AnimationGpuSkinningReadiness`默认disabled，DefaultAnimationManager没有override；`SkinningPalette`/`DoubleBuffer`只在自身测试使用，CPU通过 `posed * bind.inverse()`生成矩阵并clone到所谓uploaded buffer，没有GPU resource、fence、ring、generation或renderer caller。renderer仅有 `SkinningPaletteStorage` binding枚举。morph readiness/status同样无producer，glTF morph channel在import match中直接忽略。

目标与Graphics 09/10定义明确handoff：per-visible-instance rig/mesh generation、palette offset/count、current/previous pose、motion-vector epoch、upload allocation/fence、compute/vertex skinning capability、bounds/culling和device-loss rebuild。inverse bind来自rig/skin artifact，不在frame中反推。morph curve进入clip artifact并与mesh target table绑定。Animation owner负责dense pose与deformation request，renderer负责GPU lifetime；真实skinned mesh capture、CPU oracle和GPU timing通过前readiness保持Unsupported。

### P1-20：Editor与animation_graph只是descriptor/validation脚手架，能力声明超过产品实现

animation editor注册authoring view/drawer与inspector descriptor，animation_graph注册palette、toolkit、command和operation id；声明的 `authoring.zui`、blend-space、avatar-mask、graph-player和state-machine-player视图文件并不存在。operation id在生产没有handler。`compile_animation_graph`只验证后返回output source字符串，state-machine compile只返回entry/state/transition计数，不生成runtime artifact。没有graph document canvas、selection、undo/redo、copy/paste、debug preview、live instance trace、asset save/reimport或compile error navigation。

目标先把capability标成Experimental/Unavailable，未接handler的command不进入ready产品菜单。随后由Animation-owned Editor extension提供真实asset document与transaction：node/edge model、schema-driven property panel、typed parameter/condition editor、blend-space viewport、mask bone tree、preview scene、timeline、compile artifact、diagnostic定位和runtime instance debug。所有编辑通过authoring source -> compile -> last-good artifact事务；Play runtime不直接执行未验证document。用真实用户流测试create/open/edit/undo/save/reload/compile/preview/Play，而不是descriptor source assertion。

## 5. P2 扩展差距

### P2-1：Retarget/IK Rig与跨骨架动画复用没有系统

当前所谓target identity只是同一skeleton内的path hash，没有source/target rig、chain mapping、retarget pose、translation/scale policy、orientation correction或runtime/offline retarget artifact。基础rig artifact稳定后，建立Retarget Profile与IK Rig authoring，优先cook-time生成target clip，必要时才runtime retarget；误差、missing chain和比例极端情况必须可诊断。

### P2-2：Root Motion没有提取、消费、角色控制和网络语义

clip/graph/state machine没有root trajectory artifact，也没有accumulate/consume API、character controller integration、collision result correction或network prediction/reconciliation。应从prepared clip提取root curve，graph按相同blend权重合成，owner phase在pose commit前发布deterministic delta；gameplay/physics决定实际movement并把修正反馈给animation。不能通过让root bone直接移动scene node临时实现。

### P2-3：Montage、slot、sync group、marker与inertialization缺失

生产角色需要locomotion与一次性action叠加、section jump、branching notify、marker同步、leader/follower、slot mask、blend profile和interruption。Unreal `AnimSync`/Montage表明这些是实例调度与事件语义，不是多加几个graph node即可。应在MVP instance/program完成后增加统一sync clock与action stack，复用同一prepared artifact和event transaction。

### P2-4：Motion Matching/pose search没有数据、索引与预算架构

当前无pose feature extraction、trajectory query、database cook/index、search budget、continuity cost、streaming或debug visualization。实现应以offline deterministic feature database、SIMD/accelerated search、per-character budget与fallback graph为核心，并记录quality/performance曲线；不能在frame中遍历所有clip/key做最近邻。

### P2-5：Control Rig、Full-Body IK、facial/morph/cloth与crowd animation仍未建立平台

two-bone/look-at不足以覆盖procedural rig、constraints、foot contact、facial curves、correctives、cloth耦合和大规模crowd。它们应建立在统一rig program、dense pose/deformation buffer和task DAG之上，按feature/plugin分层；facial/morph/cloth的最终GPU实现与Graphics owner协同。不得在通用scene transform或字符串dynamic event上继续堆叠。

## 6. 参考引擎差距裁决

| 工程问题 | Zircon当前 | 参考源码给出的边界 | Zircon裁决 |
|---|---|---|---|
| 帧生命周期 | 一个scene tick，active frame demand未接 | Unreal initialize/cache/update/evaluate/post与parallel data | typed phase DAG + instance demand snapshot |
| 内容数据 | raw keys/runtime compile，无compression/cook | Unreal platform compressed/derived data；Fyrox稳定track | prepared clip/rig/program artifact + residency |
| target/pose | path/string混用，AoS String pose | Bevy AnimationTarget；Fyrox pose binding；Unreal compact pose | rig-scoped dense slot与SoA pose page |
| graph/machine | 有基础功能，instance state分散 | Godot AnimationTree；Fyrox machine layer；Unreal graph/sync | compiled program + single instance owner |
| 骨架/render | 每bone写scene node，GPU surface无consumer | Godot Skeleton modifier/server；Unreal skeletal mesh pose | dedicated skeleton storage + renderer handoff |
| event/action | bounded range queue，但采样/consumer未闭环 | Unreal notify/montage queue与sync；Fyrox signal/event | cooked event index + canonical bounded ABI |
| authoring | descriptor与validator，缺document/product | 参考引擎均以真实graph/state/preview工具驱动asset | last-good compile transaction + live trace |
| 性能证据 | 单元测试多，无1k角色/GPU产品曲线 | Unreal AnyThread/LOD/required bones是明确预算机制 | 相同质量workload下测main/worker/GPU/memory |

参考源码不是要求Zircon复制Unreal所有feature，也不能用Bevy/Fyrox较小实现替代目标。可迁移的共同原则是：source data与runtime artifact分开；实例状态有唯一owner；update/evaluate/commit分相；目标与pose使用稳定dense identity；事件和异步结果带generation；渲染、物理、Editor通过明确bridge消费，不共享可变字符串对象图。

## 7. 目标架构

### 7.1 Owner与数据流

| Owner | 主要职责 | 禁止承担 |
|---|---|---|
| Runtime animation contract | asset/component/command/status ABI，capability truth | 第二套manager/evaluator |
| Animation plugin service | world slot、instance registry、frame demand、task DAG、diagnostics | renderer GPU资源、通用scene hierarchy |
| Animation cook | rig/clip/program/sequence/event artifact、compression、dependency generation | frame内同步load |
| Animation instance | parameters/triggers/state/time/transition/event transaction | authoring source document |
| Pose/Skeleton runtime | dense local/model pose pages、LOD、IK/physics composition | 每bone普通scene entity双写 |
| Renderer deformation | palette/morph upload、GPU lifetime、culling、motion vectors | graph/state-machine逻辑 |
| Physics bridge | body mapping、ragdoll/feed generation、ownership mode | 直接改animation cache |
| Animation Editor | source document、undo、preview、compile、trace | 绕过artifact执行未验证数据 |

数据流固定为：`source/import -> rig/clip/program cook -> versioned artifact residency -> world instance command/update -> parallel evaluate -> deterministic event/state commit -> dense pose publish -> physics/attachment/render consumers`。任何stage都携带project/world/replacement/artifact generation；旧异步结果只能丢弃或完成旧lease，不能提交到新world。

### 7.2 关键运行时数据

- `AnimationWorldSlot`：world handle、replacement epoch、instance slots、active/dirty lists、frame demand和telemetry。
- `AnimationPreparedSetLease`：rig、clip/program/sequence artifact generation与stream residency。
- `AnimationMachineInstance`：typed parameter page、trigger bits、node clocks、transition/layer/nested state和rollback journal。
- `PosePageSet`：current/previous local/model pose、valid mask、rig generation、writer/reader fence和scratch arena。
- `AnimationEventJournal`：canonical record、sequence cursor、count/bytes/age budget与consumer cursors。
- `DeformationRequestPage`：mesh/rig generation、pose offsets、morph weights、previous frame和render visibility/LOD。

### 7.3 Phase contract

1. **Admission**：消费component/command/asset/visibility变化，解析ready artifact，更新instance registry与frame demand。
2. **Update**：owner lane推进clock、transition、trigger与event intent，生成immutable evaluation jobs。
3. **Evaluate**：worker纯函数sample/decompress/blend/IK，写预分配result pages，不碰World和manager locks。
4. **Commit**：核对world/artifact generation，按稳定instance顺序提交state/event/pose；stale result丢弃并计数。
5. **Project**：attachment/physics/render分别消费sealed pose generation；只有必要socket投影到scene transform。
6. **Observe**：collector发布有界counter/trace，Editor按cursor读取；disabled/unavailable明确记录零工作或reason。

## 8. 必须硬切的旧路径

1. 删除core/plugin重复的animation module/manager implementation，只保留一个production owner和Runtime contract。
2. 删除高优先级glTF animation placeholder路径；canonical importer必须生成同一rig/clip artifact。
3. 禁止runtime frame调用owned `load_animation_*_asset`；改为prepared lease admission。
4. 禁止internal hot path生成带bone String的AoS pose；只在inspection边界materialize。
5. 禁止每bone写普通scene node作为skinning主路径；socket/attachment使用选择性projection。
6. 删除无production consumer的CPU“GPU upload”假实现；真实renderer bridge就绪前capability保持Unsupported。
7. 删除未接handler/view/artifact compile的Ready Editor command和capability声明。
8. 禁止silent `.ok()?`/`continue`吞掉asset/program failure；进入typed instance fault与diagnostic。

硬切不允许alias、compatibility shim、双写、hidden fallback、test-only bypass或调用方特判。迁移需要一次性资产version升级和清晰错误，不长期维护两套target/pose/manager真相。

## 9. 重构里程碑

### M0：能力真相与回归冻结

- 冻结current-source fingerprint，归并open animation failures与当前修改owner。
- 增加真实glTF import-to-evaluate RED、reactive cadence RED、masked blend RED和GPU readiness false-surface测试。
- capability按Unavailable/Experimental/Partial/Ready重标，移除无handler的产品入口。

### M1：唯一owner与world/session生命周期

- 硬切core/plugin duplicate manager/module。
- 建立 `AnimationWorldSlot`、replacement generation、shutdown/unload和frame demand snapshot。
- linked/unlinked plugin、App/Editor/standalone/headless的availability与teardown一致。

### M2：统一glTF、Rig/Skin/Clip资产与cook

- 收敛importer authority，修正full target path，建立rig/skin/clip依赖。
- 生成prepared compressed clip、event index、inverse bind与mesh remap artifact。
- 实现single-flight、last-good reimport、version migration和residency budget。

### M3：Instance registry与compiled program

- active/dirty dense registry替换per-frame candidate/map rebuild。
- graph/state machine编译成program，parameter/trigger/time/transition进入单一instance slot。
- 修复masked blend、trigger consume、deterministic arbitration和typed fault。

### M4：Dense pose与并行phase DAG

- hot path切到SoA pose pages、scratch arena、shared immutable artifact cache。
- Runtime11 task DAG覆盖sample/blend/IK，owner lane只做update/commit。
- task rejection/panic/cancel/stale generation与fairness可诊断。

### M5：Event、Sequence、IK与Physics事务

- cooked event cursor + canonical bounded journal +真实gameplay consumer。
- sequence编译typed mutation buffer并与clock/generation统一。
- IK复用model pose，physics/ragdoll bridge按明确ownership mode和generation组合。

### M6：Skeleton projection与Renderer deformation闭环

- 删除per-bone scene write主路径，建立socket/attachment选择性projection。
- Graphics owner实现palette/morph buffer、fence、device-loss、culling与motion vector bridge。
- 多骨glTF skinned mesh在App/Editor/Export产生CPU oracle一致的可视capture。

### M7：真实Animation Editor

- 实现graph/state/blend-space/mask document、undo/redo、save/reload、compile diagnostics和preview world。
- runtime live trace只读sealed instance generation，不直接持有mutable pipeline。
- asset reimport/compile失败保留last-good，Play/Stop/world replacement不泄漏instance或worker。

### M8：规模、质量、平台与高级系统入口

- 建立1/100/1k/10k实例、30/100/300骨、LOD/visibility分布、长clip streaming和事件突发矩阵。
- 对同等画质/骨数/节点/事件/worker配置比较Unreal/Fyrox/Bevy可复现实验；不以少功能取胜。
- 基础门稳定后再开启retarget/root motion/sync/montage/motion matching/Control Rig子计划。

## 10. 验收与性能门

### 10.1 正确性门

- glTF多骨、duplicate leaf、多个skin/animation、cubic、morph、inverse bind从import到render全链金样。
- clip/graph/state/layer/mask/additive/trigger/interruption/event在30/60/120Hz、fixed/variable、pause/seek/loop下结果确定。
- world replacement、asset hot reload、plugin unload、task迟到、event defer不提交旧generation。
- reactive host中active动画持续唤醒，pause/end/unload精确回Idle且无busy loop。
- CPU reference pose与GPU skinning结果在定义误差内，previous pose/motion vector无generation串帧。

### 10.2 性能与内存门

- steady-state hot path记录alloc count/bytes、asset loads、String clones、candidate/dirty/evaluated/applied数量，目标active batch内无按bone heap/String分配，paused-clean为零sample/apply。
- 记录main Update/Commit、worker Evaluate、renderer upload/GPU deformation的p50/p95/p99，而非只报总frame。
- cache/residency记录raw/compressed/pose/scratch/GPU bytes、hit/miss/evict/stall和stream page starvation。
- 1k角色基准明确骨数、graph nodes、IK、morph、events、LOD与可见比例；比较必须同语义、同质量、同硬件、同线程和warmup。
- 1小时loop/transition/event/asset reload soak无queue增长、generation leak、pose page leak或worker残留。

### 10.3 产品与平台门

- Windows先按repository policy完成App、Editor Play/preview、standalone与export capture；Linux-specific需求再进入WSL证据。
- Debug/Release、headless/server、GPU feature fallback和device loss分别声明capability，不用空图或disabled default冒充成功。
- Editor真实用户流覆盖create/open/edit/undo/save/reload/reimport/compile/preview/Play/debug/close。
- 每个Ready capability保留source fingerprint、artifact version、product capture、性能报告和expiry；历史记录由新receipt明确supersede。

## 11. 实施前重新核对

当前animation pipeline、IK queue和manager文件存在其他Session未提交修改，其中replacement epoch、event admission和deferred rollback改变了本报告所依据的transaction边界。开始任何M0-M8实现前必须：

1. 重新读取focused `git diff`，确认外部修改已稳定且owner明确。
2. 对本报告每个P1重新跑production caller搜索，尤其是frame demand、GPU readiness、Editor handler与glTF importer priority。
3. 复核所有open failure的current-source事实，不能沿用2026-07的pass/fail计数。
4. 先运行support/baseline门，再运行animation package与产品门；本轮静态review没有Cargo通过声明。

本报告完成的是08C首轮E3静态审查。它不表示Animation已实现、已修复或达到Unreal级别；实现状态保持 `pending`。
