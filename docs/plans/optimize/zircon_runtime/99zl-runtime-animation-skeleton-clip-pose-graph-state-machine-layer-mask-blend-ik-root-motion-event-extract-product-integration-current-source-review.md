---
title: Runtime Animation / Skeleton / Clip / Pose / Graph / State Machine / Layer / IK / Root Motion / Extract 当前源码复审
category: zircon_runtime
report_id: Runtime137
review_date: 2026-08-24
baseline_head: 79f64878f3b9526517644c055ad3bf5cadfccd0f
baseline_epoch: 421
verification_head: ed543173cbd825fe3b7e1f6c81d52c9ca3391095
verification_epoch: 422
supersedes_currentness_of:
  - docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md
  - docs/plans/optimize/zircon_plugins/13-first-party-animation-source-runtime-editor-dist-catalog-skeleton-clip-pose-graph-state-machine-ik-skinning-product-integration-review.md
related_code:
  - zircon_runtime/src/animation
  - zircon_runtime/src/core/framework/animation
  - zircon_plugins/animation
  - zircon_plugins/animation_graph
  - zircon_runtime/src/asset/importer/ingest/gltf_animation_subassets.rs
  - zircon_plugins/gltf_importer/runtime/src/subassets.rs
  - zircon_runtime/src/scene/level_system/animation_runtime.rs
  - zircon_runtime/src/scene/level_system_render_extract.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh
  - zircon_plugins/physics/runtime/src/skeletal
  - zircon_editor/src/core/editing/animation_document
  - zircon_editor/src/ui/animation_editor
  - zircon_editor/src/ui/graph
  - zircon_editor/src/ui/timeline
  - zircon_editor/src/ui/curve
  - zircon_app/Cargo.toml
plan_sources:
  - docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md
  - docs/plans/optimize/zircon_plugins/13-first-party-animation-source-runtime-editor-dist-catalog-skeleton-clip-pose-graph-state-machine-ik-skinning-product-integration-review.md
  - docs/plans/zircon_plugins/04/failure-2026-07-22-runtime-animation-fallback-evaluator-divergence.md
  - docs/plans/zircon_plugins/04/failure-2026-07-29-animation-frame-diagnostics-hardcut-omission.md
  - docs/plans/zircon_runtime/runtime/11/failure-2026-07-29-dynamic-runtime-animation-module-duplication.md
  - docs/plans/zircon_plugins/04/failure-2026-07-29-animation-sequence-caller-root-drift.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Animation/AnimInstance.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Animation/AnimNodeBase.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Animation/AnimSequence.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/Animation/AnimInstanceProxy.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/BoneContainer.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/GPUSkinVertexFactory.h
  - dev/bevy/crates/bevy_animation/src/lib.rs
  - dev/bevy/crates/bevy_animation/src/graph.rs
  - dev/bevy/crates/bevy_animation/src/transition.rs
  - dev/bevy/crates/bevy_gltf/src/loader/mod.rs
  - dev/Fyrox/fyrox-animation/src/pose.rs
  - dev/Fyrox/fyrox-animation/src/track.rs
  - dev/Fyrox/fyrox-animation/src/machine
  - dev/godot/scene/animation/animation_mixer.cpp
  - dev/godot/scene/animation/animation_tree.cpp
  - dev/godot/scene/3d/skeleton_3d.h
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Runtime137 · Animation 当前源码复审

## 1. 结论

当前Animation不是“完全没有实现”，但仍是多个局部原型叠加出来的产品假象，距离可与Unreal、Godot、Bevy、Fyrox同维度比较的工程级动画系统很远。可以保留的真实底座有五类：builtin glTF importer已经能生成Clip/Skeleton并构造canonical target path；事件区间采样已经从重复线性选最小值改为`BinaryHeap`和可恢复cursor；插件内已有dense target table、compiled clip/graph/state evaluator、mask和有限trigger消费；Scene extract已经把pose交给renderer，renderer也有current/previous palette与morph/motion history；Editor当前工作树新增了document、revision、transaction、undo/redo、atomic save、source compiler和last-good product。

这些进展仍未形成单一Animation产品合同。Runtime fallback和Animation plugin继续以相同module/driver/manager名称维护两套求值器；新Runtime compiler只是source-only validator，又与插件compiled evaluator和Animation Graph validator形成第三、第四套语义；帧循环继续同步加载、按字符串绑定、克隆player/parameter/pose并由owner线程等待临时channel；IK生产集成已经删除，只剩孤立solver；root motion、retarget、montage/sync marker/inertialization、motion matching、Control Rig、morph curve和server cook policy仍不存在。

渲染链必须特别纠正旧结论：pose现在确实能到达GPU skinning路径，因此不再是“完全断路”。但palette在`skinning.rs:30`使用`posed_world * bind_world.inverse()`，而不是消费glTF真实inverse-bind matrix；同时没有mesh-joint到skeleton-slot的versioned remap，且GPU分支前仍准备整份CPU-skinned primitive。这是**已接通但变形正确性、resident data和性能仍未合格**，只能判Partial。

Editor同样只能判Partial。document/undo/save/compiler是实质进展，但两个核心ZUI仍只是header加空hybrid slot，pane只投影字符串列表；Graph/Timeline/Curve foundation没有产品consumer，compiler product没有runtime preview consumer，首方Editor catalog也没有Animation provider。插件声明的八份`plugins://...zui`仍全部缺失。

旧Runtime08C的20项P1重判为 **14 Open、4 Partial、2 Closed**，5项P2全部Open。旧Plugins13的48项P1重判为 **39 Open、8 Partial、1 Closed**，12项P2全部Open。四份Animation failure handoff仍为Open。本轮不修改production/tests，也不把shared working tree中的未提交实现当作已集成资格证据。

## 2. 审查边界、方法与currentness

### 2.1 冻结物理范围

统计口径为当前工作树物理行、非空行、文件bytes、test declaration和ignored declaration。fingerprint按normalized lowercase path排序，对每个文件拼接`path + NUL + lowercase(file SHA-256) + LF`后再取SHA-256。

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | fingerprint |
|---|---:|---|
| Runtime Animation与Framework Animation完整目录 | **69 / 9,059 / 8,236 / 298,550 / 39 / 2** | `8e8964978c714348c26cad0304b5715636b04afbe12111867ebd40cd57e24ee9` |
| Animation与Animation Graph插件完整目录 | **176 / 19,626 / 18,070 / 691,741 / 153 / 2** | `16eaaa3f6dcc880b588f9beb39260a9fb7b8a8d757b37bb775e433a265b8e933` |
| Editor document/session/graph/timeline/curve/host/template纵切面 | **61 / 6,731 / 6,169 / 230,956 / 33 / 0** | `7273daf0f248a1f6ca7b10a8fd302a96f7976c75f1987af1b613fea215a58c35` |
| glTF、Scene、Render、Physics、App/catalog产品边界 | **28 / 9,222 / 8,522 / 334,587 / 41 / 3** | `cf72578202a82f9a89accc85da376b04203a0088c02c4a32bd4e2150a2139d3f` |
| Zircon selected union | **334 / 44,638 / 40,997 / 1,555,834 / 266 / 7** | `43fc053661dee039865a580adfd1ed7397fb14a74bbbffcddce96c31d282f5e6` |
| 五引擎参考选择集 | **32 / 26,143 / 22,206 / 1,084,243 / 20 / 0** | `25711706f628d09c8955b681c4435048f6d6bb78de2e9d73493309892ad2a874` |
| selected combined scope | **366 / 70,781 / 63,203 / 2,640,077 / 286 / 7** | `1c8c4cdca30752a5f654e3165c9026a80dd055b8d3dd5a78ecb3f480eb23588b` |

参考revision：Bevy `fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`、Godot `8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、Fyrox `8d815db36494f1badb347547dfc7094bf4fbbdf8`、Unity Graphics `a7e4c051d256a781ab362c64316b125a1e104694`。`dev/UnrealEngine`没有独立Git元数据，由10个选择文件和参考集合fingerprint冻结。

### 2.2 检查方法

1. 逐文件读取Runtime fallback、Framework schema/compiler/asset、Animation plugin evaluator/state/layer/mask/IK/GPU DTO和Animation Graph插件。
2. 沿`glTF -> source asset -> compiler/cache -> frame request -> pose publication -> Scene extract -> renderer/physics`追踪真实生产调用链，而不是根据类型名或单元测试推断功能完成。
3. 沿`asset type -> Editor route -> document -> mutation/history -> compile/last-good -> pane payload -> retained template -> save/reimport`追踪authoring链。
4. 反查App target feature、first-party runtime/editor catalog、source/dist manifest和missing resource URI，核对普通Client与Editor Host是否真的可达。
5. 对Runtime08C与Plugins13原编号逐项重判。局部类型、测试或断开的foundation存在时最多Partial；只有原失败条件和产品旁路都消失才允许Closed。
6. 对照Unreal的instance/proxy/node lifecycle、compressed data/required bones/sync/notifies/root motion，Bevy的stable target/graph mask/transition与typed glTF skin，Fyrox的pose/layer/mask/state/transition，Godot的typed track cache/tree/filter/root motion/skeleton skin version，以及Unity Graphics的resident instance/current-previous transform consumer边界。

### 2.3 动态证据边界

- Session基线为`79f64878f3b9526517644c055ad3bf5cadfccd0f` / epoch 421；冻结时共享主线为`ed543173cbd825fe3b7e1f6c81d52c9ca3391095` / epoch 422。
- Animation、Editor与compiler范围包含其他Session/用户的tracked与untracked改动。本文审查其当前物理内容，不覆盖、不回退，也不把它们标记为已合并功能。
- 本轮是review-only，不运行Cargo、Editor、PIE、GPU capture、真实glTF corpus、save/reopen、server cook、fault/scale/soak/profile或跨引擎同语义benchmark。
- 静态调用图足以证明的零consumer、重复authority、错误inverse-bind、同步load和missing URI不因本轮未运行Cargo而改变；所有动态资格门仍按缺证据Fail/Partial。
- Tooling按用户要求排除，未来单独迁移到Rust，不进入本报告优先级和统计。

## 3. 当前真实链路

```text
glTF source
  +-- builtin importer ----------> Clip + Skeleton + generic Skin/IBM Data
  |                                  |
  |                                  +--> raw bincode v1 source assets
  |
  +-- first-party glTF plugin ---> placeholder Clip + generic Skin/IBM Data

Runtime composition
  +-- zircon_runtime fallback AnimationModule/DefaultAnimationManager
  +-- zircon_plugins/animation AnimationModule/DefaultAnimationManager
      (same names, different evaluators)

Frame
  Scene scan -> sync asset loads -> compiled/legacy evaluation
             -> temporary worker channels -> merge -> String pose
             -> generic LocalTransform writes (after world transform)
             -> Level pose snapshot
                  +--> Render extract -> current/previous palette -> GPU scene
                  +--> Physics full String-keyed pose copy

Editor
  route -> AnimationAuthoringDocument -> mutation/history -> source-only compile
        -> last-good product (no runtime consumer)
        -> string pane payload -> header + empty hybrid slot
        -> explicit atomic save/reimport
```

这张图的核心问题不是缺少更多class，而是每条边都没有共享同一代artifact、stable identity、residency lease、phase fence和qualification receipt。

## 4. 必须保留的基础

1. 保留builtin glTF canonical target path构建及cycle/duplicate/missing-node拒绝，但把它升级为stable joint/target ID和typed artifact graph。
2. 保留Clip event的heap/cursor/budget结构，补齐resident clip、forward/reverse/loop边界、observer overflow和downstream receipt。
3. 保留插件dense target table、PoseBuffer、compiled condition/layer/mask和one-shot trigger消费，迁入唯一Runtime evaluator authority。
4. 保留Scene extract到renderer的pose snapshot、current/previous palette、motion history和GPU scene槽位；替换其错误binding输入与CPU fallback热路径。
5. 保留Editor document revision、CAS mutation、transaction、undo/redo、last-good和atomic write；让产品UI与runtime preview真正消费它们。
6. 保留Runtime compiler的typed diagnostic/schema registry作为统一compiler前端起点，但删除“只验证source就叫installable artifact”的语义。
7. 保留Animation Graph和Animation插件的公开descriptor意图，先对齐schema和provider ownership，再删除重复validator/palette。

## 5. 当前最高风险差异

### 5.1 Authority、artifact与identity

| ID | 状态 | 当前证据 | 工程级目标 |
|---|---|---|---|
| AR-P1-001 | Open | Runtime与插件`module.rs:13`都声明`animation.runtime`，并各自构造`DefaultAnimationManager` | 一个Runtime-owned `AnimationService`；plugin只提供注册/扩展输入，禁止第二实现 |
| AR-P1-002 | Open | fallback manager仍公开graph/state/clip三条source求值入口；插件内部又有legacy manager、compiled graph和state pipeline | source统一编译为`AnimationProgramArtifact`，frame只执行一个evaluator |
| AR-P1-003 | Open | Runtime compiler product在`product.rs:28`自称source-only artifact，只有Editor document消费 | compiler输出versioned dependency-closed runtime artifact、diagnostic和install receipt |
| AR-P1-004 | Open | Skeleton只有name/parent/local TRS；Clip/Graph/State/Sequence继续携带String和raw key Vec | Skeleton/Rig/Clip/Graph/State/Sequence/SkinBinding分别source/cooked分型，stable ID与schema version闭合 |
| AR-P1-005 | Open | binary envelope仍为version 1，却尝试多个历史Rust layout；decode允许同版本结构漂移 | 显式reader/writer matrix、migration、canonical hash、strict trailing policy和fuzz corpus |
| AR-P1-006 | Open | builtin importer产出真实Clip/Skeleton，高优先级glTF plugin在`subassets.rs:514`仍产出placeholder | 单一import authority；placeholder禁止进入Ready，provider选择必须可观测 |
| AR-P1-007 | Open | Skin/IBM仍是generic Data；Skeleton只收skin.joints且可丢失中间transform节点 | typed SkinBinding含joint target、IBM、mesh primitive/LOD remap、rig signature和dependency digest |
| AR-P1-008 | Open | Graph插件palette含blend-space node，Runtime graph schema含additive/mask且pin合同不同 | 一个schema registry生成Editor palette、validator、compiler和runtime opcode |

### 5.2 Frame execution、scheduling与状态

| ID | 状态 | 当前证据 | 工程级目标 |
|---|---|---|---|
| AR-P1-009 | Open | graph/state/sequence/clip仍在frame路径同步load；错误多处折叠为空pose | prepare/residency/install phase；frame只有bounded handle lookup和typed disposition |
| AR-P1-010 | Open | direct clip每shard新建`sync_channel`，忽略schedule result，owner阻塞`recv`且worker丢失时panic | generation-qualified job DAG、deadline/cancel/fault isolation、last-good publication |
| AR-P1-011 | Open | parameter/player/active-state/String pose反复clone，cache以完整parameter map和线性Vec作为key | compiled slots、interned IDs、SoA instance state、scratch arena与bytes-budget cache |
| AR-P1-012 | Open | system在`zircon.scene.world_transform`之后运行却写LocalTransform | PreTransform evaluate/apply、PostTransform dependent consumers和明确previous/current fence |
| AR-P1-013 | Partial | plugin transition记录并移除`consumed_triggers`，fallback没有同语义 | trigger slot在唯一program中原子消费，transition arbitration有priority/sync/interruption receipt |
| AR-P1-014 | Open | state source-order首命中；无sync group/marker、inertialization、montage slot或state debug trace | deterministic arbitration、sync domain、marker phase、inertialization和可追踪transition history |
| AR-P1-015 | Open | `record_animation_requires_continuous_frame(true)`无生产caller，仅失败/reset写false | player/activity/root-motion/event需求驱动frame demand并带reason/generation |
| AR-P1-016 | Open | IK只剩`TwoBoneIkJob`/`LookAtJob`导出与unit tests，原postprocess生产链已删除 | rig phase DAG、component-space cache、constraint graph、Control Rig/FBIK扩展合同 |

### 5.3 Deformation、render、physics与extract

| ID | 状态 | 当前证据 | 工程级目标 |
|---|---|---|---|
| AR-P1-017 | Partial | pose经`level_system_render_extract.rs:36`进入renderer，current/previous palette存在 | pose publication绑定skeleton/skin/artifact generation和render-frame fence |
| AR-P1-018 | Open | palette以重建bind world求逆，不消费imported IBM，也无mesh joint remap | `mesh_to_skeleton_slot[] + inverse_bind[] + rig_signature`是唯一变形输入 |
| AR-P1-019 | Open | GPU选择前仍clone并准备完整CPU-skinned primitive；每帧同步Skeleton load | resident prepared mesh/deformation instance，GPU路径零CPU vertex skinning和零冷load |
| AR-P1-020 | Open | importer拒绝morph weight animation，evaluator无morph output，renderer只有独立morph payload | compiled curve/morph channels进入同一pose/deformation result及previous-frame history |
| AR-P1-021 | Open | Physics使用`BTreeMap<String, SkeletalPoseTarget>`并复制整姿态 | generation-qualified pose slot view、bone/constraint dense index和按需physical-bone subset |
| AR-P1-022 | Open | generic scene node transform既是动画写入边界又与render pose快照并行 | canonical pose owner；scene projection、render、physics、gameplay各消费明确view，不双写truth |

### 5.4 Editor、catalog与产品可达性

| ID | 状态 | 当前证据 | 工程级目标 |
|---|---|---|---|
| AR-P1-023 | Partial | Editor有真实document/history/save/source compile；last-good无runtime/UI diagnostic consumer | authoring document、compiler artifact、preview instance和save/cook共享revision/digest |
| AR-P1-024 | Open | capability表明确clip/additive/mask/semantic compiler/runtime preview unavailable | capability来自artifact-bound qualification receipt，不由静态表自报 |
| AR-P1-025 | Open | Graph只可加Output/Blend且无layout；Sequence新key复制last value，state frame换算硬编码30fps | 完整graph/state/timeline/curve/notify/root-motion/rig编辑模型与typed timebase |
| AR-P1-026 | Open | generic Graph/Timeline/Curve foundation只有定义/测试，无Animation产品consumer | retained product control直接消费document projection，规模化virtualization和selection闭合 |
| AR-P1-027 | Partial | 两个core ZUI存在，但仍是header+空hybrid slot；pane只发`node_items`/`track_items`字符串 | 可交互graph/timeline/curve/viewport控件，截图、输入、locale/theme和large-asset资格 |
| AR-P1-028 | Open | Animation声明4个、Graph/Timeline声明4个不存在的plugin URI；asset/content roots为空 | contribution bundle必须带mount/hash/resource closure，missing resource fail-close |
| AR-P1-029 | Open | Runtime catalog有Animation branch，但App默认Client/Editor Host不启用first-party runtime plugins | target composition显式选择Animation provider并输出activation receipt |
| AR-P1-030 | Open | first-party Editor catalog没有Animation/Graph/Timeline；native dist为metadata shell | source/native同语义provider或明确Unsupported，不允许Loaded/Enabled伪象 |

## 6. Runtime08C逐项重判

| 原ID | 状态 | 当前判定 |
|---|---|---|
| P1-1 duplicate core/plugin manager/module | Open | 同名module/driver/manager及独立实现仍在 |
| P1-2 frame demand not driven | Open | production没有写true的caller |
| P1-3 high-priority glTF plugin placeholder | Open | placeholder文本仍在 |
| P1-4 builtin wrong non-root target_id | **Closed** | canonical full parent path与异常拒绝已落地 |
| P1-5 skeleton/skin/IBM/mesh/clip relation | Open | typed SkinBinding和versioned relation仍缺失 |
| P1-6 raw String/AoS, no prepared compressed artifact | Open | source-only/raw assets仍是runtime输入 |
| P1-7 sync loads/owned clones | Open | 多条frame路径仍同步load/clone |
| P1-8 String pose/public allocations | Open | public pose仍为String/Vec |
| P1-9 per-frame candidate/container rebuild | Open | instance/cache/hot path仍重建 |
| P1-10 pseudo parallel workers | Open | 临时channel、blocking recv、无handle/deadline |
| P1-11 global mask normalization | **Closed** | 当前实现按bone对有效层归一化 |
| P1-12 graph cache/clone/linear interpreter | Open | 尚未成为resident compiled program |
| P1-13 trigger not one-shot/state scattered | Partial | plugin消费selected trigger，fallback与整体authority未收敛 |
| P1-14 arbitration/sync/diagnostics thin | Open | source-order、无sync marker/inertialization |
| P1-15 event sampling/downstream incomplete | Partial | heap/cursor改进；residency、字符串clone与observer closure仍缺 |
| P1-16 sequence/timeline per-frame load/general mutation | Open | 同步load和generic scene mutation仍在 |
| P1-17 IK sync load/repeated model pose/no rig phase | Open | 生产IK链已删除，问题未解决 |
| P1-18 pose writes generic scene nodes | Open | generic LocalTransform写入仍是主路径之一 |
| P1-19 GPU skinning/morph/render bridge | Partial | render bridge真实存在；IBM/remap/residency/morph animation不正确 |
| P1-20 editor/animation_graph scaffolding | Partial | document/history/save出现；UI、preview、catalog仍断开 |
| P2-1 retarget | Open | 无rig compatibility/retarget profile/solver |
| P2-2 root motion | Open | 无extract/consume/authority/network policy |
| P2-3 montage/sync/inertialization | Open | 无对应program与Editor产品 |
| P2-4 motion matching | Open | 无pose search/database/cook/runtime budget |
| P2-5 Control Rig/FBIK/facial/morph/cloth/crowd | Open | 仍无统一rig/deformation扩展面 |

## 7. Plugins13逐项重判

### 7.1 Packaging、catalog、artifact与runtime

| 原ID | 状态 | 当前判定 |
|---|---|---|
| NANI-P1-001 | Open | ordinary Client仍未启用first-party runtime catalog |
| NANI-P1-002 | Open | Editor Host仍未选择Animation provider |
| NANI-P1-003 | Open | first-party Editor catalog无Animation分支 |
| NANI-P1-004 | Open | Graph/Timeline无catalog closure |
| NANI-P1-005 | Open | builtin advertisement/effective provider仍可能错位 |
| NANI-P1-006 | Open | fallback/plugin双authority仍在 |
| NANI-P1-007 | Open | server strip/evaluation policy仍缺 |
| NANI-P1-008 | Open | Animation四份plugin ZUI仍缺失 |
| NANI-P1-009 | Open | Graph/Timeline四份plugin ZUI仍缺失 |
| NANI-P1-010 | Open | native dist仍是metadata shell |
| NANI-P1-011 | Open | source/native scenario parity仍无证据 |
| NANI-P1-012 | Open | capability仍不表达真实支持矩阵 |
| NANI-P1-013 | Open | 无Animation activation receipt |
| NANI-P1-014 | Open | plugin glTF仍产placeholder |
| NANI-P1-015 | Open | Skin/IBM仍非typed artifact |
| NANI-P1-016 | Open | source/cook/runtime/editor artifact graph仍未建立 |
| NANI-P1-017 | Open | legacy manager与compiled evaluator仍重复 |
| NANI-P1-018 | Open | graph/state/Runtime compiler继续增加语义入口 |
| NANI-P1-019 | Open | clip/skeleton热路径同步load |
| NANI-P1-020 | Open | graph/state/sequence同步load |
| NANI-P1-021 | Open | 多种失败仍静默为空pose/成功tick |
| NANI-P1-022 | Open | worker仍不消费scheduler提交结果 |
| NANI-P1-023 | Open | channel断开仍panic |
| NANI-P1-024 | Open | owner线程仍同步等待 |
| NANI-P1-025 | Open | parameter/player/string clone仍在 |
| NANI-P1-026 | Open | public dense pose仍退化String/Vec |
| NANI-P1-027 | Open | cache/eviction无规模资格 |
| NANI-P1-028 | Open | raw channel仍不是prepared artifact |
| NANI-P1-029 | Open | graph仍构造临时Vec/parameter snapshot |
| NANI-P1-030 | **Closed** | mask已按bone做局部归一化 |
| NANI-P1-031 | Open | pose写入仍依赖不稳定字符串解析 |
| NANI-P1-032 | Open | animation仍排在world transform之后 |
| NANI-P1-033 | Partial | plugin已消费选中的trigger；fallback和统一状态权威未闭合 |
| NANI-P1-034 | Open | state machine仍缺priority/sync/history/trace |
| NANI-P1-035 | Open | layer/entity PoseBuffer与publication仍无稳定arena合同 |
| NANI-P1-036 | Open | BlendSpace2D仍不是cook-time triangulated artifact |
| NANI-P1-037 | Partial | event heap/cursor改进；observer/backpressure/resident clip不完整 |
| NANI-P1-038 | Open | IK production integration已删除，只剩local solver |
| NANI-P1-039 | Open | Physics仍String-keyed full-pose copy |
| NANI-P1-040 | Partial | GPU render路径存在，但IBM/remap/CPU热路径未合格 |
| NANI-P1-041 | Partial | current/previous palette和morph renderer存在；typed skin/morph animation relation缺失 |
| NANI-P1-042 | Open | root motion仍无产品authority |
| NANI-P1-043 | Partial | builtin Editor route/document可达；plugin catalog/provider仍不可达 |
| NANI-P1-044 | Partial | 两份core模板存在但为空slot；八份plugin resource仍缺 |
| NANI-P1-045 | Partial | builtin Editor event/document compile可执行；plugin operation/dist仍无executor |
| NANI-P1-046 | Partial | undo/save/last-good已出现；runtime preview/同artifact闭环仍缺 |
| NANI-P1-047 | Open | 无跨carrier同场景parity corpus |
| NANI-P1-048 | Open | 无scale/stability/performance qualification |

### 7.2 P2保持Open

Plugins13的12项P2全部保持Open：retarget/profile；root motion policy；montage/slot/sync marker；inertialization；motion matching/pose search；Control Rig/FBIK；facial/morph curve；cloth/deformation graph；crowd sharing/LOD；network/replay determinism；server cook/strip；Animation Insights/debugger/profiler。新增局部compiler、Editor document或renderer palette都没有关闭这些系统级合同。

## 8. 参考引擎差异

| 参考 | 本轮读取的可验证模式 | Zircon差异与采用边界 |
|---|---|---|
| Unreal | `AnimNodeBase`的Initialize/CacheBones/Update/Evaluate生命周期；`AnimInstanceProxy`的pre/update/evaluate/post与required bones；AnimSequence source/compressed data、root motion、sync marker/notifies；GPU skin current/previous bone buffer | 采用phase lifecycle、compact bone container、compressed derived artifact、sync/notifies/root-motion和current/previous deformation合同；不复制UObject/宏架构 |
| Bevy | stable `AnimationTargetId`、player active map、event range；graph threaded acceleration与dense mask；transition；glTF joints/IBM和target component | 采用stable target identity、typed glTF skin关系和预计算graph/mask；避免把ECS component布局直接当Zircon产品架构 |
| Fyrox | pose/root motion、binding fetch hint、pool handle、layer/mask/state/transition/action/event与machine evaluate | 采用pool/generation handle、layered machine与root-motion result；结合Zircon Runtime service和artifact边界 |
| Godot | Mixer typed track cache、blendshape/value/method/audio/animation分型；Tree filter/sync/thread state；Skeleton skin refs/version/dirty propagation/physical bones | 采用typed track cache、skeleton/skin version、filter与physical-bone桥；不复制NodePath字符串作为runtime hot-path identity |
| Unity Graphics | resident instance data、current/previous transform更新、component-mask selective update、batch/cull/LOD/occlusion | 仅用作renderer消费侧resident deformation/previous-frame/batching参考；该仓库不是Animation authoring/evaluator参考，不能据此补写其不存在的能力 |

没有任何一个参考引擎支持“用更多临时class补齐功能清单”的方向。共同点都是：source与runtime data分离、stable identity、预计算/缓存、明确phase、bounded publication、typed failure和可观测lifecycle。Zircon应先补齐这些基础合同，再扩展高级节点。

## 9. 目标架构与hard cutover

```text
AnimationSourceDocument
  SkeletonSource / RigSource / ClipSource / GraphSource / StateSource / SequenceSource
          |
          v
AnimationCompiler (single semantic authority)
  +-- diagnostics + dependency graph + schema migration
  +-- SkeletonArtifact { compact bones, target IDs, rig signature }
  +-- SkinBindingArtifact { joint remap, IBM, mesh/LOD compatibility }
  +-- ClipArtifact { compressed tracks, curves, events, root motion, markers }
  +-- ProgramArtifact { opcodes, slots, masks, transitions, sync domains }
          |
          v
AnimationArtifactInstaller / Residency
  generation + lease + last-good + retirement + server strip receipt
          |
          v
AnimationInstanceStore
  SoA state + parameter slots + pose slots + scratch arenas
          |
          v
Prepare -> Parallel Evaluate -> Rig/IK -> Merge -> Publish
          |
          +--> Scene transform projection
          +--> Render deformation view (current/previous)
          +--> Physics physical-bone view
          +--> Gameplay events/root-motion view
          +--> Editor preview/debug trace
```

Hard cutover规则：

1. 先让Runtime成为唯一compiler/evaluator/service owner，再删除fallback/plugin重复manager；禁止re-export shim长期共存。
2. source asset与cooked artifact使用不同类型和版本；不在同一个`ZRANIM01/v1`下猜测Rust历史layout。
3. 删除runtime bone name/target path解析，外部名称只在compile/import/diagnostic边界出现。
4. 删除frame内`ProjectAssetManager::load`、临时channel和owner blocking wait；未resident返回typed disposition并保留last-good。
5. 删除renderer“重建bind pose”的近似路径，SkinBinding artifact是唯一joint palette输入。
6. 删除Graph插件独立schema/validator和dead GPU skin DTO；Editor palette从Runtime schema生成。
7. 删除空dist和missing-resource的Loaded表象；provider缺行为或资源时必须Unsupported/Rejected。

## 10. 依赖顺序与实施里程碑

| Milestone | 内容 | 退出条件 |
|---|---|---|
| M0 truth freeze | 加入fallback/plugin/compiler/schema/import/render调用图与负例characterization | 双authority、placeholder shadow、wrong IBM、frame-demand false-ready都有稳定RED |
| M1 artifact identity | 定义Skeleton/Rig/SkinBinding/Clip/Program artifact及schema/version/dependency digest | glTF corpus能证明joint/remap/IBM/中间transform/多skin关系，旧v1有显式migration |
| M2 compiler convergence | Runtime唯一semantic compiler；Graph/State/Sequence/Mask/BlendSpace编译到同一program | Editor/plugin不再维护独立validator/palette；invalid source不替换last-good |
| M3 residency/install | prepared artifact cache、lease、generation、retirement、server strip | frame路径零同步load；stale/evicted/missing有typed disposition |
| M4 instance/evaluate DAG | SoA instance、slot parameter、pose arena、Prepare/Evaluate/Rig/Merge/Publish | worker失败不panic，deadline/cancel/late result可测，steady frame零规模相关heap分配 |
| M5 state/sequence semantics | priority、sync marker/group、interruption、inertialization、events、root motion | deterministic trace、loop/reverse/event边界、replay/network/server policy闭合 |
| M6 deformation consumers | typed render/physics/gameplay views、IBM/remap、morph/curve、previous frame | GPU路径无CPU skinning；large crowd/LOD/motion-vector correctness与budget合格 |
| M7 Editor product | graph/timeline/curve/viewport/diagnostics/preview、save/reopen/cook | 同revision artifact在Editor preview和Runtime执行；八个missing URI被删除或真实打包 |
| M8 advanced animation | retarget、montage、Control Rig/FBIK、motion matching、facial/cloth/crowd | 每项独立artifact、budget、debug、fallback和qualification，不以demo替代产品门禁 |

M0-M4是MVP基础链，必须先于高级节点。M5-M8不能通过在现有字符串manager上继续叠加feature来推进。

## 11. 资格门

### 11.1 Correctness与artifact

- `Gate-A01`：单一compiler/evaluator/service authority；fallback和plugin不再有平行行为。
- `Gate-A02`：Skeleton、SkinBinding、Clip、Program有独立schema/version/hash和reader/writer matrix。
- `Gate-A03`：glTF translation/rotation/scale/cubic-spline、joint hierarchy、IBM、多skin、morph curve有oracle corpus。
- `Gate-A04`：invalid/stale/missing artifact不能发布Ready或覆盖last-good。
- `Gate-A05`：stable target/joint/parameter/state/track identity跨save/reimport/cook保持可迁移。
- `Gate-A06`：root motion、event、notify、marker在forward/reverse/loop/seek下有确定语义。
- `Gate-A07`：state transition priority/interruption/sync/inertialization有deterministic trace。
- `Gate-A08`：retarget/rig compatibility不依赖名称猜测。

### 11.2 Performance与scheduling

- `Gate-P01`：steady evaluate无随bone/key/player数量增长的heap allocation和String clone。
- `Gate-P02`：frame线程零同步asset I/O、零临时channel、零无deadline阻塞。
- `Gate-P03`：1/10/100/1,000 player、30/100/500 bone和layer/graph规模矩阵有CPU、RSS、tail latency预算。
- `Gate-P04`：worker submit failure、panic、cancel、deadline、shutdown、late result不会终止host或发布半帧。
- `Gate-P05`：cache按bytes与lease淘汰，有hit/miss/compile/evict/stale telemetry。
- `Gate-P06`：GPU skin path不执行CPU vertex skin，palette/update按dirty subset上传。
- `Gate-P07`：current/previous pose、root motion和motion vector使用同一generation/frame fence。
- `Gate-P08`：crowd共享、LOD、visibility/culling下Animation工作量有可证明上界。

### 11.3 Product、Editor与carrier

- `Gate-E01`：普通Client、Editor Host、Server的provider选择/strip policy/activation receipt可观察。
- `Gate-E02`：source与NativeDynamic运行同一scenario corpus，或Native明确Unsupported。
- `Gate-E03`：Editor graph/timeline/curve/viewport有真实控件、input、selection、undo/redo和large asset virtualization。
- `Gate-E04`：compiler diagnostics、last-good、runtime preview和debug trace可见且绑定document revision。
- `Gate-E05`：save/reopen/reimport/cook/install/preview/runtime执行同一语义artifact。
- `Gate-E06`：missing resource、provider withdrawal、reload和cancel fail-close，不显示Loaded/Ready伪象。
- `Gate-E07`：renderer、physics、gameplay和Editor不复制/篡改pose authority。
- `Gate-E08`：locale/theme/DPI/reduced-motion/accessibility与截图回归覆盖动画编辑器产品面。

### 11.4 Reliability与高级系统

- `Gate-R01`：corrupt/truncated/oversized/unknown-version资产通过fuzz与故障注入，不panic、不OOM。
- `Gate-R02`：hot reload/reimport在活动player、transition、IK和render in-flight时有generation fence与retirement。
- `Gate-R03`：save/replay/network/server对gameplay curve/event/root motion保持确定性，并剥离render-only数据。
- `Gate-R04`：设备丢失、renderer fallback和headless不会改变gameplay animation truth。
- `Gate-R05`：IK/Control Rig/physics feedback有cycle detection、iteration/error budget和debug view。
- `Gate-R06`：motion matching database cook/query有memory、latency、quality和fallback receipt。
- `Gate-R07`：facial/morph/cloth/deformation组合有统一curve/pose generation和previous-frame policy。
- `Gate-R08`：长时运行、频繁切换、reload storm、crowd soak无泄漏、无无界cache、无event backlog漂移。

当前32项Gate均未Pass；`A03/A06/P05/P07/E03/E04/E05`最多Partial，其余Fail。局部unit test不替代产品资格。

## 12. Owner边界与非重复计数

- Runtime137拥有Animation source/compiler/artifact/instance/evaluator/pose publication与consumer contract。
- Plugins13继续拥有package/source-dist/provider/catalog/resource closure；本报告只刷新其Animation专项currentness。
- Plugins07拥有glTF provider遮蔽与通用import artifact transaction；Runtime137拥有Animation typed artifact oracle。
- Runtime09D/64拥有通用resource residency/lease；Runtime137定义Animation prepared artifact与frame不可同步load的专项要求。
- Runtime09A/09B拥有renderer/GPU scene；Runtime137定义SkinBinding、pose generation、morph/root-motion deformation输入。
- Runtime60/62拥有ECS、hierarchy和transform phase；Runtime137要求Animation不在错误phase双写派生truth。
- Editor14/50及Animation Editor专项拥有通用toolkit/document/retained UI；Runtime137定义同artifact preview和animation-specific product surface。
- Runtime59拥有通用scheduler；Runtime137定义Animation DAG、deadline、last-good与pose publication语义。

## 13. 首个实施切片

实现阶段不要从增加新Graph node开始。首个切片应同时建立四个characterization RED：

1. 同一fixture分别经fallback与plugin evaluator产生可观察差异，证明双authority不可保留。
2. glTF Skin fixture带非单位IBM和中间transform节点，当前renderer palette与oracle不等，证明近似bind重建错误。
3. scheduler拒绝direct clip shard时当前路径会等待/失败，证明必须引入submit receipt与last-good。
4. 普通`target-client`和`target-editor-host`无法给出Animation provider activation receipt，证明产品不可达。

然后按M1建立`SkeletonArtifact + SkinBindingArtifact`最小闭环，并让builtin importer、renderer和一个Editor preview fixture共享同一generation。只有这条纵切通过后，才开始删除duplicate manager和迁移graph/state compiler。

## 14. 本轮未做事项

本轮只新增当前源码复审与索引记录，没有修改Rust、Cargo、manifest、ZUI、测试或旧报告，没有运行Cargo和产品动态验证。所有实现、性能优于参考引擎、画面质量优于Unreal、平台资格与release结论仍为pending；实现前必须在最新working tree重取334个Zircon selected文件的统计和fingerprint。
