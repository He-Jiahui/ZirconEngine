---
title: Editor Motion Matching、Pose Search、Database、Feature Schema、Trajectory Query、Runtime Selection、Preview、Debugger 与 Product Integration 当前源码工程化差距
category: zircon_editor
report_id: Editor79
review_date: 2026-08-23
baseline_head: f1614c5e601d0879cfa3ac1e5d4886f0d8734d97
baseline_epoch: 355
related_code:
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/gameplay/workbench_ability_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_motion_matching_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/index/workbench_extension_module_workspaces.zui
  - zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/gameplay_animation.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/gameplay_animation.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/reference_menu_actions.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/componentized_window.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback.rs
  - zircon_editor/src/ui/retained_host/workbench_preview_actions.rs
  - zircon_editor/src/ui/retained_host/workbench_preview_actions/extensions.rs
tests:
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation.rs
  - zircon_editor/src/ui/retained_host/workbench_preview_actions.rs
plan_sources:
  - docs/plans/mvp/index.md
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/14-animation-sequence-graph-state-machine-timeline-curve-preview-compiler-authoring-review.md
  - docs/plans/optimize/zircon_editor/32-model-mesh-skeleton-geometry-import-lod-collision-retarget-preview-authoring-review.md
  - docs/plans/optimize/zircon_editor/63-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/69-editor-scene-viewport-realtime-update-preview-simulation-time-domain-pause-step-animation-particle-physics-audio-visibility-throttling-invalidation-performance-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/75-editor-animation-timeline-dope-sheet-curve-editor-track-key-selection-transport-scrub-snap-clipboard-transaction-virtualization-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/76-editor-animation-graph-state-machine-node-edge-parameter-condition-compiler-runtime-transition-blend-preview-transaction-persistence-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/77-editor-animation-sequence-clip-channel-binding-interpolation-compression-event-root-motion-sync-preview-compiler-product-integration-current-source-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Plugins/Animation/PoseSearch/Source/Runtime
  - dev/UnrealEngine/Engine/Plugins/Animation/PoseSearch/Source/Editor
  - dev/godot/scene/animation
  - dev/Fyrox/fyrox-animation/src
  - dev/Fyrox/editor/src/plugins/absm
  - dev/bevy/crates/bevy_animation/src
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Data/Nodes/MeshDeformation
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Editor/Models
doc_type: current_source_review
review_status: complete
implementation_status: not_started
source_recheck_required: true
---

# Editor Motion Matching、Pose Search、Database、Feature Schema、Trajectory Query、Runtime Selection、Preview、Debugger 与 Product Integration 当前源码工程化差距

## 1. 结论

当前Zircon没有Motion Matching产品，也没有Pose Search运行时平台。仓内唯一专用产品表面是230行`workbench_extension_motion_matching_workspace.zui`：`MM_Locomotion / MM_Combat / MM_Traversal`、`Idle_Breath`、`184 clips`、`1 warning`、三个固定trajectory option和`Cost Bias: 0.42`全部写死；Preview与Rebuild只选择按钮并写入固定的`queued`反馈。专用action最终进入通用workspace/tab/row/command/field路由，字段`commit`没有database mutation，所谓cost、warning与selected pose也没有数据来源。

更关键的是，`zircon_runtime`、`zircon_plugins`和`zircon_app`中没有Motion Matching、Pose Search、pose feature database或trajectory query的生产consumer。没有source asset、feature schema、offline extractor、database build key、prepared search index、pose history query、continuing-pose policy、search result、runtime instance、animation graph node、selection receipt、trace或cook。当前工作区不能通过补一个按钮或在frame内扫描clip升级为工程级实现。

本轮不新增P0。Editor14 P0-2继续唯一拥有“静态成功工作区声明不存在能力”，其P1-52继续拥有高级动画工作区没有asset/transaction/job/compiler/runtime preview的总账；Runtime08C P2-4继续唯一拥有Motion Matching/pose search平台整体缺失及不得逐帧全量扫描clip的总账。本报告只登记尚未被逐项建账的 **15项P1、5项P2和48个资格门**，把产品纵向合同展开为`MotionMatchingDatabaseSource -> PoseFeatureSchema -> PoseSearchBuildPlan -> PreparedPoseSearchDatabase -> MotionQuerySnapshot -> MotionMatchingSelectionReceipt -> AnimationPlaybackTransaction`。

本轮只做current-source review和文档建账，不修改生产源码。未运行Cargo、真实Editor、GUI/GPU、database build、cook、runtime query、preview、trace、fault/soak/profile或同语义跨引擎benchmark；因此不能宣称当前功能正确、可用、性能达标，更不能宣称性能或表现超过Unreal。

## 2. 审查边界、currentness与冻结语料

### 2.1 本轮唯一owner

本报告只拥有“Motion Matching数据库source如何定义feature schema与动画entry，经唯一offline compiler生成immutable prepared search artifact，由Runtime在qualified pose/trajectory history上进行有预算选择，并由Editor对同一artifact generation进行authoring、preview与debug”的纵向边界。

- Editor14继续拥有高级动画工作区可达性、静态成功措辞、通用asset/toolkit/preview/compile真实性P0；本轮引用但不重复计数。
- Editor09继续拥有background job admission、cancel、progress、shutdown和durable artifact总合同；Pose Search build只能作为其typed job接入。
- Editor32继续拥有Skeleton、Skin、import/reimport、mirror/retarget identity；数据库只能引用其稳定artifact。
- Editor63继续拥有transaction/history/savepoint/document scope和async operation总合同。
- Editor69继续拥有PreviewWorld、time domain、pause/step和可见性调度。
- Editor75继续拥有Timeline/Dope Sheet/Curve/transport交互；数据库preview只消费其时间合同。
- Editor76继续拥有Animation Graph唯一compiler/runtime authority；Motion Matching必须成为该program的typed node/operation，而不是第二套evaluator。
- Editor77继续拥有Clip/Event/Root Motion/Sync/prepared animation artifact/playback transaction；选择结果必须引用其稳定clip artifact并交给其播放链。
- Runtime08C P2-2继续拥有root trajectory artifact与gameplay movement桥，P2-4继续拥有Motion Matching平台大类、搜索预算和fallback graph；本轮展开Editor/product contract但不把同一平台缺席换名重报。

### 2.2 Currentness

- 审查HEAD：`f1614c5e601d0879cfa3ac1e5d4886f0d8734d97`。
- 协作baseline epoch：`355`；session：`optimize-editor79-motion-matching-pose-search-review-r1-20260823`。
- 本轮focused paths在审查前没有working-tree diff；两级`optimize`索引已有本会话前的在途文档变更，本轮只追加Editor79并按物理文件重算统计，不回退任何既有内容。
- `git grep`确认Motion Matching/Pose Search专用词只落在Editor静态surface/binding/feedback/action inventory；`zircon_runtime`、`zircon_plugins`、`zircon_app`返回零生产命中。
- 静态action白名单、`LazyLock<HashMap>`路由测试、固定feedback和ZUI可加载都不等于database、search、preview或runtime资格。

### 2.3 冻结语料与可复算fingerprint

统计口径：路径转为小写正斜杠并排序；每个文件取SHA-256，再拼接`path + NUL + lowercase file hash + LF`计算集合fingerprint。declarations使用Rust/C++/C#的`fn/class/struct/enum/trait`行首声明正则，仅用于规模定位。

| 范围 | 文件 / 行 / 非空行 / bytes / declarations | fingerprint |
|---|---:|---|
| Zircon editor/product | **12 / 5,739 / 5,479 / 271,986 / 62** | `edeef349ff8c0736065e0e473bc0925ca47530b15c4f45c63cb9dc7fb397ddff` |
| Unreal selected set | **24 / 16,237 / 13,655 / 659,990 / 144** | `e60927a19dfe28deba821146481d4c07f5142f954c3bdfdf606e9e41d1d5c5a7` |
| Godot selected set | **8 / 7,988 / 6,792 / 303,944 / 55** | `099b10b876b4c965499ba1856e035cb9ccf11a2e8f8fa96225325763e5a56a22` |
| Fyrox selected set | **5 / 1,828 / 1,633 / 67,123 / 122** | `1b2c8ba330ad2ef0e05e41aaf0a5a7e0acd8b4d5402a2f96a86ea0298f611dab` |
| Bevy selected set | **3 / 2,979 / 2,692 / 113,433 / 168** | `f8835c768eb054f0c2d1d8fbebd4c80e93c23a365f579e02cb636246bec9ee1a` |
| Unity Graphics selected set | **4 / 458 / 414 / 21,159 / 7** | `f087df51937d0fb6a582d70655f297e60c15d36fecffda8b84d08132b7726bde` |
| Five-engine deduplicated set | **44 / 29,490 / 25,186 / 1,165,649 / 496** | `a59dac6147d7b8f07b60cc14f004e810bcfa29d3b0a70fffa5af58036339433e` |

### 2.4 集合成员

Zircon集合为frontmatter列出的12个product文件：Ability入口、Motion Matching ZUI、extension workspace host、template binding、navigation index/spec、通用action执行、componentized field处理、固定feedback和preview action inventory。没有可加入集合的Motion Matching runtime/compiler/database source；这不是抽样遗漏，而是本轮全仓专用词与候选类型检索的结果。

Unreal集合覆盖PoseSearch Runtime的Schema、FeatureChannel、TrajectoryChannel、Database、Index、Context、Result、History、MotionMatching AnimNode、DerivedData/Key、AssetIndexer、search实现和Trace，以及Editor的Database toolkit/tree/view model/debugger database view。Godot集合覆盖AnimationTree/StateMachine、AnimationMixer root motion与对应Editor；Fyrox覆盖pose、ABSM machine/node和Editor command；Bevy覆盖AnimationGraph asset/loader/threaded graph、player和transition；Unity Graphics集合覆盖ShaderGraph deformation/skinning与VFX skinned renderer consumer。

## 3. 当前真实产品链与可保留底座

### 3.1 可见workspace是固定演示数据

Motion Matching ZUI第49-95行固定三个tab和三个database名字；第138-184行固定四条pose、cost、stance、trajectory match、turn bias、foot lock及`184 clips / 1 warning`；第199-230行固定database、trajectory options与`Cost Bias: 0.42`。这里没有asset ID、document ID、schema、source revision、build generation、query frame、selected pose address或operation receipt。

Ability workspace只增加一个`Motion Matching`按钮；extension workspace host只把ZUI挂入隐藏容器。它们证明surface可达，不证明对应产品存在。

### 3.2 Action链只做UI选择与固定反馈

template binding把22个ZUI event转换成`workbench.extension.motion_matching.*` action。navigation spec第623-724行只列出tab/row/command control ID与field action；通用`ExtensionActionRoute`只有workspace/tab/row/command和`field_action: bool`。`apply_workbench_extension_action`第196-219行只切workspace、exclusive selected state、dropdown popup，再调用feedback。

feedback第302-329行把Open、Preview、Rebuild、Idle与Pivot映射为固定字符串，包括`Preview queued MM_Locomotion trajectory 0.8s`与`Rebuild queued 184 clips 1 warning`。preview action测试只断言open/rebuild/cost action存在于静态集合；没有数据库变更、job、artifact、runtime query、preview frame或错误路径断言。

### 3.3 Runtime、compiler与cook链为零

仓内没有Pose Search resource kind、database source schema、feature channel registry、offline indexer、derived-data key、prepared artifact或runtime search service。Animation插件没有Motion Matching node/instance；Runtime没有pose history与trajectory query ABI；App没有cook/install路径。当前所谓database、trajectory与cost彼此无数据关系，也没有任何代码能产生ZUI显示的pose cost。

### 3.4 可保留的只是通用UI基础

单进程`LazyLock<HashMap>` action index、exclusive selection、popup与retained template基础可以保留，但只能作为产品projection。未来Motion Matching controller必须把typed document snapshot与operation state投影到这些控件；不能让control ID、显示文本或当前focus继续充当domain identity。

## 4. 父报告校正、开放阻断与不重复计数

| 既有owner | 本轮确认仍Open的事实 | 本轮处理 |
|---|---|---|
| Editor14 P0-2 | 可见Motion Matching workspace以`queued`和固定成功数据声明不存在能力 | 保持原P0，不重复登记假UI |
| Editor14 P1-52 | 高级动画workspace没有asset schema、transaction、job artifact、compiler或runtime preview | 保持总账，本轮只展开专属合同 |
| Editor09 | Rebuild必须是有准入、取消、进度、关闭与artifact receipt的background job | 作为ED79 build依赖，不重复通用job finding |
| Editor32 | Skeleton/Skin/import/mirror/retarget identity尚未完全闭合 | 数据库引用其artifact，不另造Skeleton owner |
| Editor63 | transaction/history/savepoint/async operation总合同未闭合 | 数据库编辑与publication复用，不重复通用undo finding |
| Editor69 | PreviewWorld/time domain/pause-step总合同未闭合 | runtime-backed preview复用，不另造preview world |
| Editor75/76/77 | Timeline、Graph compiler/runtime、Clip/Event/Root Motion/Sync各有唯一owner | ED79只增加Motion Matching专属source/query/selection bridge |
| Runtime08C P2-2 | root trajectory与movement bridge缺失 | trajectory query依赖，不重复计数 |
| Runtime08C P2-4 | feature extraction、trajectory query、database index、budget、continuity、streaming、debug平台缺失 | 保持大类owner；ED79登记可验收的产品纵向分解 |

没有新证据可以关闭任何父P0/P1/P2。`Rebuild`按钮、`Cost: 0.04`、`Ready`或action binding都不是动态产品证据。

## 5. 新增P1工程差距

### ED79-P1-01：没有canonical `MotionMatchingDatabaseSource`、稳定身份、版本、revision与依赖闭包

当前database身份只是`MM_Locomotion`等显示字符串。没有`DatabaseAssetId / DocumentId / SourceRevision / SkeletonArtifactId / FeatureSchemaId / EntryId`，也没有load/save/migration/reimport关系。

目标建立versioned source document，分离durable source、editor session、build generation与runtime instance；所有mutation按qualified document target和expected revision提交，显示名不得参与寻址。

### ED79-P1-02：没有typed `PoseFeatureSchema`、channel registry、维度布局与兼容性规则

ZUI只展示一个无类型cost bias。不存在pose/trajectory/curve/phase/event channel、bone/role/reference frame、sample offsets、cardinality、layout offset、channel version或schema compatibility。

目标建立可扩展typed feature channel registry；每个channel必须声明输入、坐标系、时间偏移、维度、单位、缺值政策、build/query实现、debug label与版本升级，Schema finalize后生成稳定layout descriptor。

### ED79-P1-03：weight、normalization与cost不是可审计数学合同

当前没有per-channel/per-dimension weight、normalization group、mean/deviation、零方差政策、distance metric、bias单位或cost decomposition。单个`Cost Bias: 0.42`无法解释0.04/0.18/0.31等固定值。

目标把raw feature、normalization stats、sqrt weight、metric与bias顺序写入prepared artifact和query receipt；finite、nonnegative、zero-weight、cross-database normalization与schema drift必须fail-close。

### ED79-P1-04：数据库entry没有源动画语义、采样区间、镜像、loop、reselection与provenance

没有stable entry ID、clip artifact generation、sampling interval、exclude head/tail、loop/mirror option、blend-space sample/permutation、tags、enablement、block-transition或source event关联。

目标以typed entry引用Editor77 prepared clip与Editor32 Skeleton/Mirror artifact；一个source可展开多个indexed segment，但每个pose必须可逆映射到entry、source asset、time、mirror/permutation与build generation。

### ED79-P1-05：没有唯一、确定性的offline feature extractor与build/query等价规则

仓内既无asset sampler/indexer，也无“同一channel在offline IndexAsset与runtime BuildQuery使用同一数学定义”的机制。直接临时采样clip会制造Editor/Runtime、CPU/backend和版本漂移。

目标建立唯一`PoseFeatureCompiler`和共享channel kernel；固定时间量化、边界/extrapolation、root/reference transform、mirror、curve与floating-point policy，并用golden feature vector验证跨build重复性。

### ED79-P1-06：Rebuild没有build key、dependency invalidation、cancel、last-good、原子publication与cook

按钮只写`queued`。没有source/schema/clip/skeleton/mirror/compiler version组成的key，没有外部依赖变更监听，没有旧任务取消、并发代际、失败保留last-good、原子安装或target-platform cook。

目标通过Editor09提交`PoseSearchBuildJob`，生成dependency manifest、content key、diagnostics和immutable artifact；只有完整验证通过的generation可CAS publication，取消/失败不得破坏last-good。

### ED79-P1-07：没有`PreparedPoseSearchDatabase`的自包含数据布局、索引元数据与结构校验

不存在dense feature table、pose metadata、pose-to-entry/time mapping、normalization stats、search backend payload、alignment、endianness、version、checksum、memory estimate或load validator。

目标artifact自包含runtime搜索所需数据，不借用Editor Vec下标或source对象地址；load/cook必须验证维度、offset、pose range、finite值、排序、树结构、dependency generation和预算。

### ED79-P1-08：没有qualified pose history与trajectory query ABI

`Forward 0.8s`只是下拉字符串。没有history/current/future samples、sample timestamp、time domain、coordinate frame、position/facing/velocity、source controller、confidence、reset/teleport/cut epoch或missing-data政策。

目标建立`MotionQuerySnapshot { world, entity, frame, time_domain, pose_history, trajectory, source_generation }`；movement/controller只发布typed trajectory，Pose Search不得读取当前UI、场景节点或隐式全局时钟。

### ED79-P1-09：runtime query builder没有与offline schema一致的channel取样、缓存与失败语义

没有按schema构建query vector、continuing pose复用、channel cache、骨架兼容、历史插值、trajectory sample或normalization。未来若各node自行拼Vec，会重演多compiler authority。

目标由prepared schema驱动唯一query builder，复用有界scratch与channel cache；输出携带schema/generation，缺骨、历史不足、非finite、teleport和过期artifact必须产生typed disposition，不能静默填零。

### ED79-P1-10：没有搜索政策、continuity/filter/tie/budget/fallback与可解释结果

当前不存在candidate filter、continuing pose、jump threshold、reselect history、block transition、cost addend、early-out、deterministic tie-break、deadline、shortlist、fallback graph或failure receipt。

目标先建立可作为oracle的精确有界搜索，再接加速backend；`MotionMatchingSelectionReceipt`必须记录query/database generation、候选/过滤数、选中pose address/time、cost decomposition、continuing/new、elapsed/budget、fallback原因与deterministic tie key。

### ED79-P1-11：没有Motion Matching runtime instance、Animation Graph接入与原子播放handoff

没有per-character state、elapsed search time、selected asset/time、play rate、blend/interrupt、reset-on-relevance、selection history、root motion/event/sync handoff或completion。即使将来能搜到pose，也没有正确播放路径。

目标把Motion Matching作为Editor76 compiled animation program中的typed operation，由单一Animation Runtime持有instance state；新选择通过Editor77/Runtime08C的atomic playback transaction接入blend、sync、events和root motion。

### ED79-P1-12：多数据库搜索没有schema兼容、共同归一化、共享query与结果可比性

Locomotion/Combat/Traversal只是三个静态名字。没有数据库集合、tag/asset filter、schema compatibility、共同normalization set、query cache共享、跨库bias、priority或结果合并合同。

目标显式编译search set；不可比较的schema必须拒绝或经声明的projection转换，跨库cost必须证明同尺度，结果合并使用稳定tie key并记录每库shortlist与skip原因。

### ED79-P1-13：Editor没有真实Database toolkit、transactional asset tree、详情、统计与诊断

当前workspace只有固定三行database、四行pose和三个字段。没有asset create/open/save、entry add/remove/reorder、drag/drop、sampling range、schema/channel editor、dependency browser、statistics、build status、invalid pose定位或undo/redo。

目标建立per-document toolkit和view model，所有编辑进入Editor63 transaction；tree/详情/统计由document/build snapshot投影，selection使用stable entry/pose address，build diagnostics可导航回source element。

### ED79-P1-14：Preview与Debugger没有runtime-backed generation、query向量、候选拒绝和可回放trace

Preview只写固定文本，Debug tab没有内容。没有preview actor/world、artifact generation、query vs pose feature draw、trajectory/history、cost列、candidate flags、continuing pose、search timing、frame trace或rewind。

目标Editor69 PreviewWorld与真实runtime evaluator消费同一prepared artifact；trace以bounded、versioned event记录query、候选、选择和playback handoff，Debugger按frame/node/database展示cost breakdown与拒绝原因，并可从trace定位source pose。

### ED79-P1-15：没有确定性、正确性、故障、质量与性能资格体系

现有测试只验证action inventory和通用HashMap复用。没有schema migration、feature golden、index determinism、exact-search oracle、continuity sequence、teleport/reset、cancel/publication、corrupt artifact、cross-platform float drift、1/100/1k角色或quality/performance曲线。

目标建立unit/property/golden/integration/fault/soak/profile矩阵；任何加速backend必须与exact oracle比较召回、cost regret和选择稳定性，性能报告同时固定角色、骨数、数据库pose数、query维度、搜索频率、画质与硬件。

## 6. 新增P2扩展差距

### ED79-P2-01：大数据库shard、streaming、residency、prefetch与hot-swap尚未建立

在基础artifact与budget闭合后，再按gameplay tag/region/locomotion set分片，建立resident generation、prefetch hint、memory pressure、eviction、miss fallback与无停顿hot-swap；不能让IO出现在animation update热路径。

### ED79-P2-02：多角色interaction、role assignment、同步selection与warping尚未建立

双人/多人交互需要role-qualified skeleton/history、共同candidate、原子多实例commit、availability、root alignment、failure rollback与网络authority。它不是把多个单人结果并排播放。

### ED79-P2-03：SIMD、PCA/KD/VPTree、ANN、GPU或异构加速缺少quality-bounded策略

加速后端必须是prepared artifact的可替换实现，保留exact oracle、deterministic mode、recall/regret预算、fallback与backend-specific telemetry。禁止为了benchmark数字减少feature、pose或搜索频率。

### ED79-P2-04：大规模authoring的virtualization、批量规则、统计分布、outlier与diff/merge缺失

数万clip/百万pose数据库需要分页tree、批量tag/range/mirror规则、feature分布、异常值、coverage、duplicate/prune解释、artifact diff和multi-user冲突；不能把全部pose物化成ZUI row。

### ED79-P2-05：超过Unreal所需的自动调参与同语义质量/性能实验室缺失

目标不是复制Unreal默认参数，而是以固定输入录制、ground-truth事件/轨迹、foot sliding、trajectory error、transition pop、recall/regret、CPU/memory/latency共同评价，允许离线搜索权重、schema与backend Pareto前沿。没有同语义证据时不得宣传“更快”或“表现更好”。

## 7. 五套参考源码裁决

### 7.1 Unreal：Pose Search/Motion Matching主架构参考

Unreal PoseSearch插件把Runtime与Editor明确分层。`PoseSearchSchema.h:71-129`定义sample rate、roled skeleton/mirror、递归channels、normalization、cardinality、padding与debug channel；`PoseSearchFeatureChannel.h:84-123`要求每个channel提供Finalize、BuildQuery、FillWeights和IndexAsset；Trajectory channel把time offset、position/velocity/facing flags、weight与normalization group结构化。

`PoseSearchDatabase.h:509-620`把Schema、continuing/base/loop bias、entry、normalization set、PCA/KD/VPTree参数和prepared index关联；第688-745行提供continuing与多backend search。DerivedData manager定义New/Continue/Wait、InProgress/Success/Failed并监听object/property/transaction/package/delete；key builder遍历依赖、加入版本与animation compression version，避免非确定Name hash。

`AnimNode_MotionMatching.h:109-180`和cpp把database set、interrupt、pose jump、reselect history、search throttle、play rate、continuing pose、blend与runtime state接入Animation Graph；Trace记录query vector、candidate flags/cost、pose history与interrupt。Editor另有Database toolkit、asset tree、details/statistics、preview transport/query draw、transaction与rewind debugger。Zircon应学习这种source/build/runtime/editor责任分离，不应复制UE类型名、实验API或默认参数。

### 7.2 Godot：AnimationTree、StateMachine与root-motion边界参考

Godot当前源码没有一等Motion Matching/Pose Search命中。它提供AnimationTree/StateMachine的typed播放与travel/process，以及AnimationMixer中明确的root motion track、local政策、position/rotation/scale delta与accumulator，Editor用专用property选择root track。

因此Godot只证明选择结果必须进入真实animation/runtime/root-motion边界，不能证明pose database或search architecture；本报告不从Godot虚构feature schema。

### 7.3 Fyrox：pose、ABSM runtime与transactional authoring参考

Fyrox当前源码也没有一等Motion Matching/Pose Search。`pose.rs`提供typed `AnimationPose`与blend，Machine对pose graph求值；ABSM Editor的add/remove/state/blend-space操作实现`execute/revert`命令。

它适合交叉校验Zircon的selection handoff、pose ownership和Editor transaction，不适合替代Unreal的offline feature database与search budget设计。

### 7.4 Bevy：serialized graph、asset loader与prepared traversal参考

Bevy没有Motion Matching/Pose Search产品，但`graph.rs`分离`SerializedAnimationGraph`、`AnimationGraphAssetLoader`与`ThreadedAnimationGraph`，在asset event后重建prepared traversal/mask；transition component明确主动画与fade-out集合。

这支持“持久source不可直接充当每帧执行结构”和“选择结果必须进入正式player/transition”的裁决，不提供pose-search算法证据。

### 7.5 Unity Graphics：只作为deformation与frame-history consumer边界

Unity Graphics选取范围没有Motion Matching平台；唯一同词命中来自STP图像重建注释，与角色动作匹配无关。ShaderGraph的Compute Deform/LBS节点只消费position/normal/tangent、bone matrix/weight，VFX只消费SkinnedMeshRenderer/root-bone transform。

因此Graphics只能约束最终pose generation必须稳定交给skinning/deformation/motion-vector消费者，并保留previous/current frame history；不能成为database、trajectory或search设计主参考。

## 8. 目标架构与唯一authority

### 8.1 Source、build、runtime与product分层

```text
MotionMatchingDatabaseSource + PoseFeatureSchema
        | stable ids / source revision / dependency manifest
        v
PoseSearchBuildPlan -- Editor09 job --> PoseFeatureCompiler
        | exact sampler / normalization / diagnostics / build key
        v
PreparedPoseSearchDatabase (immutable, versioned, cooked, self-contained)
        | generation-qualified install
        v
MotionQuerySnapshot --> PoseSearchRuntime --> MotionMatchingSelectionReceipt
        | pose history + trajectory      | exact/accelerated + budget/fallback
        v                                v
CompiledAnimationProgram --> AnimationPlaybackTransaction --> Pose/RootMotion/Event
        ^
        |
Editor Toolkit / PreviewWorld / Trace Debugger consume the same artifact generation
```

### 8.2 核心合同

`MotionMatchingDatabaseSource`拥有database/entry/schema稳定ID、source revision、Skeleton/Clip/Mirror依赖与authoring字段。`PoseFeatureSchema` finalize为不可变`PoseFeatureLayout`，它同时驱动offline index与runtime query；不允许Editor和Runtime各自实现一份channel数学。

`PoseSearchBuildPlan`冻结source revision、dependency generations、compiler/backend version与target platform。`PreparedPoseSearchDatabase`包含header、layout、normalization、pose metadata、feature/index payload、source address map、diagnostics summary、checksum和memory budget。publication是CAS，Runtime只读取last-good immutable generation。

`MotionQuerySnapshot`必须带World/Entity/Frame/TimeDomain/teleport epoch、pose history、trajectory和source generation。`PoseSearchRuntime`执行有界search并输出typed receipt；receipt不是日志文本，而是Runtime选择、Editor debugger、telemetry和回放的共同事实。

### 8.3 Crate与owner责任

- `zircon_editor`拥有toolkit、document session、transaction projection、build orchestration、preview/debug UI，不拥有运行时搜索数学副本。
- animation runtime owner拥有feature channel ABI、compiler共享kernel、prepared artifact validator、query builder、search backend与Motion Matching instance；具体落点必须服从`zircon_runtime`固定spine和现有animation plugin收敛，不新增平行root package。
- `zircon_runtime`资源/任务基础设施拥有artifact identity、install generation、bounded job/runtime scheduling；Animation Graph/Clip/Root Motion继续由Editor76/77与Runtime08C canonical owner提供。
- renderer/deformation只消费发布后的pose/current-previous generation，不反向参与pose search。

### 8.4 必须硬切的旧路径

- 在真实capability成立前隐藏或disabled当前Motion Matching workspace；删除固定`queued / 184 clips / 1 warning / cost`成功反馈。
- 禁止以control ID、显示名、当前focus或dropdown字符串定位database、entry、query或pose。
- 禁止Runtime逐帧遍历全部clip/key、同步构建index、同步load source asset或读取Editor source对象。
- 禁止offline extractor与runtime query builder拥有两份feature数学；禁止多个compiler或backend自行解释schema。
- 禁止build失败覆盖last-good，禁止未校验artifact安装，禁止过期query/database generation产生选择。
- 禁止只报告平均搜索时间而隐去召回、cost regret、transition质量、feature数量或搜索频率。

## 9. 重构里程碑

### ED79-M0：Capability truth、owner、RED corpus与benchmark protocol

先将现有workspace降为Unavailable/Prototype；固定父owner、source corpus、reference revision、quality metrics和hardware/workload protocol，加入“无真实artifact不得显示Rebuild成功”的RED测试。

### ED79-M1：Stable source identity、entry与Feature Schema

建立database/schema/entry IDs、schema version/migration、typed channels、layout finalize、Skeleton/Clip/Mirror dependency和transactional source document。

### ED79-M2：Deterministic sampler与offline feature compiler

实现统一time/space/mirror/extrapolation kernel、entry expansion、feature vector golden、normalization与pose provenance；先以精确数据表为oracle。

### ED79-M3：Build key、job、prepared artifact与cook

接入Editor09 job，完成dependency key、cancel/coalesce、diagnostics、last-good CAS、artifact validator、target-platform cook与atomic install。

### ED79-M4：Pose history、trajectory与query ABI

建立qualified history/trajectory snapshot、teleport/reset/cut epoch、schema-driven query builder、scratch/cache与typed missing/invalid disposition。

### ED79-M5：Exact search、continuity、budget、fallback与receipt

完成candidate filter、continuing pose、jump/reselect、deterministic tie、deadline/early-out、fallback graph和完整cost/result receipt；精确搜索成为长期测试oracle。

### ED79-M6：Animation Runtime instance与播放事务

把Motion Matching编译为Animation Program operation，闭合instance state、search throttle、interrupt、play rate、blend、sync/event/root-motion handoff与atomic selection/playback。

### ED79-M7：真实Editor toolkit、preview与trace debugger

实现asset tree、schema/details/statistics、transaction、diagnostic navigation、PreviewWorld transport、query/pose/trajectory draw、candidate cost table和rewind trace。

### ED79-M8：Scale、streaming与加速backend

在exact oracle通过后加入SIMD/PCA/KD/VPTree/ANN等可替换backend、shard/residency/prefetch/hot-swap、virtualized Editor与quality-bounded telemetry。

### ED79-M9：Fault、soak、profile与同语义跨引擎资格

完成corrupt/stale/cancel/reload/teleport/stream miss故障矩阵，1/100/1k角色、不同pose数/维度/频率规模曲线，并以固定内容和质量指标对比Unreal；未通过不得宣称超越。

## 10. 48个资格门

当前48项全部为`Fail`。静态代码存在、action可点击、测试函数存在或参考引擎有实现都不能改为Pass。

| Gate | 资格 | 当前 |
|---|---|---|
| MM-01 | Database/Schema/Entry均有stable ID、version与migration | Fail |
| MM-02 | source revision与qualified document target可验证 | Fail |
| MM-03 | Skeleton/Clip/Mirror dependency使用稳定artifact identity | Fail |
| MM-04 | feature channel registry typed、可扩展且有owner/version | Fail |
| MM-05 | channel声明time/space/unit/cardinality/missing policy | Fail |
| MM-06 | schema finalize产生唯一稳定layout与compatibility result | Fail |
| MM-07 | weight/normalization/metric/bias数学可审计 | Fail |
| MM-08 | source save/reopen/migration/reimport无损且transactional | Fail |
| MM-09 | entry sampling interval/loop/mirror/permutation语义完整 | Fail |
| MM-10 | pose可逆映射到entry/source/time/mirror/generation | Fail |
| MM-11 | offline sampler的time/space/extrapolation政策唯一 | Fail |
| MM-12 | offline IndexAsset与runtime BuildQuery共享channel kernel | Fail |
| MM-13 | feature vector跨重复build得到golden一致结果 | Fail |
| MM-14 | build key覆盖source及全部transitive dependencies/version | Fail |
| MM-15 | build job支持admission/cancel/coalesce/progress/shutdown | Fail |
| MM-16 | failed/cancelled build保留last-good并原子CAS publication | Fail |
| MM-17 | prepared artifact自包含、versioned、checksummed、cooked | Fail |
| MM-18 | artifact load验证layout/range/finite/tree/dependency | Fail |
| MM-19 | artifact给出memory/pose/dimension/backend统计 | Fail |
| MM-20 | source变更只失效正确database且无stale install | Fail |
| MM-21 | PoseHistory定义采样频率、容量、空间与插值政策 | Fail |
| MM-22 | Trajectory有历史/当前/未来typed samples与时间戳 | Fail |
| MM-23 | query携带world/entity/frame/time-domain/source generation | Fail |
| MM-24 | teleport/cut/reset/缺历史/非finite均有typed disposition | Fail |
| MM-25 | query builder由prepared schema驱动且scratch有界复用 | Fail |
| MM-26 | continuing pose与current character pose选择政策明确 | Fail |
| MM-27 | 多database schema/normalization可比性被验证 | Fail |
| MM-28 | query/cache不读Editor source、当前focus或隐式全局状态 | Fail |
| MM-29 | exact search可作为正确性oracle并有确定tie-break | Fail |
| MM-30 | filters/continuity/jump/reselect/block-transition正确 | Fail |
| MM-31 | 每实例deadline、candidate/shortlist与early-out有预算 | Fail |
| MM-32 | budget miss/index unavailable有typed fallback graph | Fail |
| MM-33 | result记录database/pose/source/time/generation | Fail |
| MM-34 | result记录分channel cost、bias与candidate reject reason | Fail |
| MM-35 | stale query/artifact/result在playback前fail-close | Fail |
| MM-36 | selection receipt可用于Runtime、debugger、trace与回放 | Fail |
| MM-37 | Motion Matching是唯一Animation Program中的typed operation | Fail |
| MM-38 | instance state处理relevance/reset/throttle/interrupt/history | Fail |
| MM-39 | selected pose以原子事务进入blend/play rate/sync/event | Fail |
| MM-40 | root motion与movement trajectory之间无双writer/反馈环 | Fail |
| MM-41 | Editor toolkit可真实create/open/save/add/remove/reorder | Fail |
| MM-42 | schema/entry/details/statistics/build diagnostics来自真实snapshot | Fail |
| MM-43 | preview消费同一prepared artifact与runtime evaluator | Fail |
| MM-44 | debugger显示query/pose/trajectory/cost/candidate/timing | Fail |
| MM-45 | trace bounded、versioned、frame-qualified且可rewind | Fail |
| MM-46 | determinism/property/golden/fault/soak矩阵动态通过 | Fail |
| MM-47 | 1/100/1k角色与pose/dimension/frequency规模预算通过 | Fail |
| MM-48 | 同质量、同内容、同硬件跨引擎证据支持性能/表现声明 | Fail |

## 11. 实现顺序、依赖与停止条件

实现顺序必须是M0至M7的source/build/query/runtime/product闭环，再进入M8/M9加速与超越目标。Runtime08C的基础asset identity、prepared animation、pose ownership、task scheduling和fallback graph若仍未闭合，ED79可以先完成source/schema/compiler/golden，但不得把静态workspace重新标记为Available。

任一里程碑出现第二份feature schema、第二份extractor/query math、以显示文本寻址、frame内全clip扫描、同步source load/build、失败覆盖last-good、未qualified跨线程结果或只以低画质赢benchmark时必须停止并回到owner裁决。

P2能力不能反向阻塞P1工程闭环；multi-role、streaming、ANN/GPU和auto-tuning只在exact oracle、artifact generation、runtime receipt与质量指标成立后开放。

## 12. 验证边界与实施前重检

本报告的`review_status: complete`只表示本轮静态取证、参考裁决、差距建账和重构路线完成。`implementation_status: not_started`与48个`Fail`是当前产品事实。

实施前必须重取HEAD、focused diff、父报告状态、Motion Matching专用词全仓命中、animation plugin/runtime装配、Clip/Skeleton/Root Motion/PreviewWorld/Background Job最新合同及五套参考源码revision，并重算本报告语料fingerprint。若期间已有他会话实现真实source/artifact/runtime链，应按动态证据关闭或改写finding，不能沿用本报告的“零实现”断言。

本轮未运行Cargo是有意的review-only边界，不是动态通过。后续至少需要schema/build/query/search/runtime/editor六层测试、故障注入、规模profile与同语义跨引擎实验，才可逐项把Gate从Fail改为Pass。
