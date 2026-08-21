---
related_code:
  - zircon_runtime/src/asset/assets/scene/post_process.rs
  - zircon_runtime/src/scene/components/scene/post_process.rs
  - zircon_runtime/src/scene/components/scene/physics.rs
  - zircon_runtime/src/scene/world/render_post_process.rs
  - zircon_runtime/src/scene/world/project_io/scene_asset.rs
  - zircon_runtime/src/scene/world/property_access
  - zircon_runtime/src/core/framework/render/post_process
  - zircon_runtime/src/core/framework/physics
  - zircon_runtime/src/core/framework/sound/components.rs
  - zircon_plugins/physics/runtime/src/backend/builtin/trigger
  - zircon_plugins/physics/runtime/src/backend/jolt/runtime.rs
  - zircon_plugins/physics/runtime/src/manager.rs
  - zircon_plugins/sound/runtime/src/components.rs
  - zircon_plugins/sound/runtime/src/service_types/acoustics.rs
  - zircon_plugins/sound/runtime/src/engine/source_environment
  - zircon_plugins/sound/editor
  - zircon_runtime/src/core/framework/navigation/modifier.rs
  - zircon_plugins/navigation/runtime/src/manager/bake
  - zircon_plugins/navigation/editor
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world/workbench_extension_volume_editor_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/rendering/workbench_extension_post_process_workspace.zui
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench
  - zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/08a-physics-runtime-review.md
  - docs/plans/optimize/zircon_runtime/08b-audio-runtime-review.md
  - docs/plans/optimize/zircon_runtime/08d-navigation-runtime-review.md
  - docs/plans/optimize/zircon_runtime/09h2-exposure-color-bloom-dof-motion-blur-ssr-terminal-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md
  - docs/plans/optimize/zircon_editor/16-terrain-landscape-foliage-scatter-world-partition-level-streaming-authoring-review.md
  - docs/plans/optimize/zircon_editor/17-sound-audio-clip-mixer-routing-effect-spatial-acoustic-timeline-authoring-review.md
  - docs/plans/optimize/zircon_editor/18-physics-material-rigidbody-collider-joint-collision-profile-cook-ragdoll-debug-authoring-review.md
  - docs/plans/optimize/zircon_editor/19-navigation-navmesh-settings-agent-area-surface-modifier-obstacle-off-mesh-link-bake-query-debug-authoring-review.md
  - docs/plans/optimize/zircon_editor/21-gameplay-ability-effect-attribute-tag-cue-debug-authoring-review.md
  - docs/plans/optimize/zircon_editor/22-render-pipeline-frame-capture-lighting-bake-reflection-probe-post-process-debug-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/GameFramework/Volume.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/TriggerVolume.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/GameFramework/PhysicsVolume.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Sound/AudioVolume.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/PostProcessVolume.h
  - dev/UnrealEngine/Engine/Source/Runtime/NavigationSystem/Public/NavModifierVolume.h
  - dev/godot/scene/3d/physics/area_3d.h
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Volume/Volume.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Volume/VolumeManager.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Volume/VolumeProfile.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Volume/VolumeStack.cs
  - dev/Fyrox/fyrox-impl/src/scene/collider.rs
  - dev/bevy/crates/bevy_ecs/src/event/trigger.rs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 37 · Volume / Zone / Trigger / Region / Gameplay / Audio / Post Process Environment Authoring 工程化差距

## 1. 结论

Zircon当前不是没有空间区域能力，而是已有四套真实、局部可用、彼此不兼容的区域语义，并在其上放置了一套虚假的通用Volume Editor。后处理、物理触发、音频环境和导航修改器分别有自己的shape、priority、filter、lifecycle与evaluation规则；`VOL_DamageZone`、`VOL_Checkpoint`和`VOL_StreamingGate`却只存在于静态ZUI、route和固定反馈中。当前产品面把“多个底层原语”和“一个可用的工程级区域系统”混为一谈。

值得保留的基础很多。Scene可以持久化`PostProcessVolumeComponent`和`ColliderComponent`；后处理有typed component registry、global/local Box/Sphere extract、camera mask、priority order和连续blend evaluator。Collider有local transform、sensor、layer、group/mask及多种shape。Builtin与Jolt路径共享确定性的Enter/Stay/Exit计算合同，并将事件送入LevelSystem快照。Sound manager会验证并保存`SoundVolumeDescriptor`，按priority/weight/stable ID选出影响并执行gain、low-pass、reverb与convolution。Navigation bake也能解析`NavMeshModifierDescriptor`并改变area。这些能力不应被另造一套“通用Volume runtime”替代。

真正的断层在边界。局部后处理必须同实体存在Collider，且只接受Box/Sphere；缺Collider或使用Capsule/Cylinder/ConvexHull/TriangleMesh/HeightField/Compound时，源码和测试都明确把体积从extract中排除，但没有Scene/Inspector/cook诊断。Sound的Box/Sphere把world center/extents直接嵌入descriptor，Box没有rotation；全仓production consumer只见服务API和内部storage，没有Scene entity/component到Sound manager的创建、更新、销毁桥。Navigation所谓area volume把Empty节点的scale当轴对齐half extents，忽略rotation，并只以待烘焙源节点的position判定area，而不是裁切或标记实际三角形覆盖。

Physics trigger是最可靠的离散重叠基础，但事件只含world、kind、trigger entity、other entity和一个point。它没有pair generation、shape/subshape identity、event sequence、退出原因、filter decision、current-overlap snapshot或bounded gameplay dispatch。更重要的是，生产代码没有DamageZone、CheckpointVolume、StreamingGate、GameplayVolume或等价typed behavior；因此收到`PhysicsTriggerEvent`不等于能执行伤害、检查点、关卡流送或玩法状态变更。

Editor的通用Volume workspace是第二套假authority。页面固定显示`12 x 8 x 6`、`Pawn capsule Ready`、`25 DPS Priority 10`、`24 volumes / 12 overlaps / 1 warning`，下拉框固定列出Damage/Reverb/Checkpoint/Streaming；Inspect与Validate只返回固定“queued”文本。Post Process workspace也只操作模板字段。内建Scene property access只覆盖Collider等Physics字段，未找到`PostProcessVolumeComponent`的读写入口；Sound AudioVolume和Navigation Modifier drawer主体仍是`Space`占位。页面无法绑定真实Scene document、selection、transaction、runtime generation、overlap stream、subsystem contributor或validation receipt。

目标不能是一个塞入Damage、Audio、PostProcess、Navigation、Streaming所有字段的万能property bag。正确收敛是共享稳定的空间身份与几何合同，同时保留typed domain adapter：`SpatialRegionSource + stable region/component IDs -> compiled geometry artifact -> generation-qualified RegionInstance/Index -> PostProcess continuous evaluator | Audio source/listener evaluator | Physics overlap stream | Navigation bake adapter | Gameplay/Streaming typed consumers -> Editor document/Inspector/gizmo/diagnostics`。共享的是identity、shape、transform、filter、revision、artifact与diagnostic；各域仍可使用最合适的查询后端、采样时机和组合规则。

Unreal的本地源码支持这一裁决：`AVolume`提供共享brush/bounds/encompass基础，Trigger、Physics、Audio、PostProcess和NavModifier在其上保留域语义、priority和subsystem proxy，而不是共享一个通用效果表。Godot `Area3D`进一步证明monitoring/monitorable、shape-pair reference count、body/area overlap map和override combination mode属于可复用区域合同。Unity Graphics的Volume/Profile/Stack/Manager展示camera-scoped连续混合、layer registration、priority dirty tracking和collider cache，但它只覆盖渲染环境，不能替代Gameplay/Audio/Navigation设计。Fyrox证明sensor与active intersection应由物理世界拥有；Bevy本地`event::trigger`不是空间Volume产品，不能作为降低基线的理由。

本报告记录5个P0、60个P1、12个P2、M0-M9重构路线和32个验收门。它只做review，不修改production代码或tests。

## 2. 审查边界与证据

### 2.1 当前工作树物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| Scene、Post Process与共享shape候选 | 16 / 6,412 / 239,567 | E3逐字段/调用/测试：Scene持久化、Collider property access、volume registry/extract/evaluator、unsupported shape与camera mask；35个test attributes |
| Physics trigger与LevelSystem事件 | 18 / 2,938 / 103,915 | E3逐pair/event/lifecycle：Builtin/Jolt、BTree pair diff、Enter/Stay/Exit、manager/runtime event surface；24个test attributes |
| Sound Volume runtime与Editor贡献 | 18 / 1,254 / 48,126 | E3逐descriptor/service/effect/drawer：validation、storage、source-position influence、strongest resolver、空drawer和无Scene bridge；0个test attributes |
| Navigation modifier/area volume | 10 / 1,466 / 46,840 | E3逐bake path：descriptor、hierarchy inheritance、Empty-node AABB、point classification、diagnostics与overlay；16个test attributes |
| Volume/Post Process Editor产品面 | 9 / 4,319 / 172,309 | E3逐模板/route/feedback/Inspector projection：固定业务数据、固定queued、字段绑定和dynamic component fallback；6个test attributes |
| selected combined scope | 71 / 16,389 / 610,757 | 当前工作树fingerprint `ba7dfd9dd6d5a8f6c88a9fb3f415c2c26032a3ee2d32bc655c975207d2d0a80c`；81个test attributes、0个ignored、7个在途文件 |

7个在途文件为`volume_component.rs`、`volume_component/params.rs`、`volume_component/tests.rs`、`volume_extract.rs`、`volume_registry.rs`以及两个Workbench template binding文件，均非本轮产生。本报告按读取时当前工作树事实编写；实施前必须重导71文件manifest、重算fingerprint，并复核这些文件的最终registry、override和route合同。

### 2.2 Post Process静态事实

1. `ScenePostProcessVolumeAsset`和`PostProcessVolumeComponent`都表达active/global/priority/weight/blend distance/profile。
2. Scene project load/save会保存Post Process Volume和Collider，两者可在同一entity共存。
3. Global volume不需要shape；local volume必须从同entity的`ColliderComponent`提取shape。
4. local Box会组合entity world transform与collider local transform，保留rotation和非均匀scale。
5. local Sphere会把非均匀scale收敛为最大轴，形成保守球体。
6. extract合同只接受Box与Sphere，其余Collider shape返回`None`。
7. 缺Collider同样返回`None`；测试明确断言component仍存在而render extract为空。
8. unsupported shape测试把这种行为称为“not silently downgraded”，但产品层没有对应拒绝诊断，最终用户仍只看到效果消失。
9. camera volume mask与render culling mask是两套入口；local fog另受render layers影响。
10. Volume evaluator过滤active/mask/influence，按priority升序应用，entity/index提供相同priority的确定性顺序。
11. registry有typed parameter interpolation与unknown component/apply error，而非裸字符串覆盖。
12. Scene每次收集会遍历PostProcessVolume archetype并排序，没有共享空间index、dirty set或大世界cell接口。
13. Editor production搜索只找到Post Process Workbench模板/route，未找到Scene property access对该typed component的读写实现。

### 2.3 Physics Trigger静态事实

1. Collider提供sensor、layer、collision group/mask、material和local transform，Scene load/save与property access均可往返sensor。
2. Builtin trigger扫描使用Physics interaction filter与实际overlap结果，不是Editor自行重算。
3. pair key只有`trigger_entity + other_entity`，使用`BTreeMap`获得稳定遍历顺序。
4. 两个对象都为sensor时，会生成方向相反的两个trigger pair。
5. current存在且previous不存在生成Enter，两者都存在生成每step一次Stay，previous消失生成Exit。
6. Exit point沿用previous map中的旧point，没有退出时新几何、原因或时间戳。
7. Jolt runtime当前也调用共享`compute_trigger_events()`，因此事件合同主要来自同步后的Collider集合，而非Jolt原生shape-pair callback。
8. `PhysicsTriggerEvent`只有world/kind/trigger/other/point，没有step index、sequence、pair generation、shape index、normal或filter trace。
9. LevelSystem公开当前物理帧的`Arc<[PhysicsTriggerEvent]>`，这是可复用的immutable snapshot基础。
10. production精确consumer只到manager/runtime event registration与LevelSystem surface，未找到Damage/Checkpoint/Streaming执行器。

### 2.4 Sound Volume静态事实

1. `sound.Component.AudioVolume`注册shape、priority、interior/exterior gain、low-pass、reverb、convolution和crossfade属性。
2. `SoundVolumeDescriptor`是Sound service DTO，ID稳定到Sound manager范围，支持Sphere和Box。
3. Sphere/Box直接保存world center；Box只保存extents，没有rotation或local transform。
4. validation会拒绝非finite/负半径、负extents、非法cutoff和负crossfade，这是值得保留的输入门。
5. manager的update/remove只在HashMap按SoundVolumeId插入/删除，没有Scene/entity revision或world generation。
6. influence按source position计算，Box是axis-aligned distance；没有listener-position与source/listener relative interior transition状态。
7. overlap只选择一个strongest volume，顺序为priority、weight、ID；没有Add/Blend/Override/Min/Max等域组合模式。
8. selected volume会执行gain、low-pass和convolution send；这是真实DSP路径，不是空DTO。
9. 全仓production精确搜索未找到Scene/runtime system调用`update_volume()`或`remove_volume()`。
10. AudioVolume drawer只有4个`Space`，authoring binding注册Apply/Set Shape/Set IR字符串但没有Scene command factory。

### 2.5 Navigation Modifier静态事实

1. `NavMeshModifierDescriptor`支持Add/Modify/Remove、agent filter、children inheritance、area override和link generation override。
2. direct modifier来自dynamic component JSON，父级modifier可按`apply_to_children`向下继承。
3. area volume只选择`NodeKind::Empty`且`override_area=true`、mode非Remove的节点。
4. center取world translation，half extents取`world scale.abs() * 0.5`；world rotation没有进入contains。
5. contains只测试待处理Scene node的world position，不测试生成的triangle vertices、centroid coverage或polygon clipping。
6. 多个area volume命中时使用collection顺序的第一个，没有priority、stable conflict record或blend/replace policy。
7. 结果可修改`triangle_areas`，但判定粒度来自源node position，复杂mesh跨越边界时整批三角形可能一起被标记或完全漏标。
8. Modifier drawer只有一个`Space`，Editor19已记录其shape/affected tile可视化、transaction和bake产品差距。

### 2.6 通用Volume Workbench静态事实

1. workspace固定列出`VOL_DamageZone`、`VOL_AudioReverb`、`VOL_Checkpoint`、`VOL_StreamingGate`。
2. profile dropdown固定为Damage/Reverb/Checkpoint/Streaming，没有对应asset type、component schema或plugin owner。
3. bounds、overlap、damage rule、OnEnter和log都在ZUI中硬编码。
4. Inspect/Validate route只返回固定queued文本，没有job ID、world/document revision、runtime generation或receipt。
5. field edit/commit只是模板binding和navigation action，没有Scene transaction、property write或save acknowledgment。
6. workspace将连续PostProcess/Audio influence、离散Physics overlap、bake-time Navigation和Gameplay effect压成同一模糊“Volume profile”。
7. 这套UI既不是现有PostProcessVolume的Inspector，也不是AudioVolume/NavModifier的统一投影，不能作为任何域的真实authoring入口。

### 2.7 动态证据边界

本轮是review-only，没有修改Runtime、Editor、interface、plugin、App生产代码或tests，也没有运行新的动态测试。固定ZUI/feedback、PostProcess unsupported shape排除、Sound Scene bridge缺失、Navigation AABB/point判定和Gameplay类型缺失均可由当前源码直接证明，不需要用一个无法触达产品工作流的Cargo lane重复确认。

此前`zircon_editor --lib`测试编译在617.2秒后被239个既有错误和122个warning阻断，当前源代码没有解除该阻断条件，本轮没有重复同一lane。后续实施必须先恢复可编译基线，再运行跨Builtin/Jolt、Sound、Render、Navigation bake、Scene roundtrip和真实Editor交互的组合验收。

## 3. 目标架构

### 3.1 共享边界，不共享万能效果表

```text
Scene entity / Prefab instance
  -> SpatialRegionSource(region_id, shape_source, local transform, filters, source revision)
  -> RegionCompiler / validator
  -> CompiledRegionGeometry(artifact key, exact shape, bounds, query capabilities)
  -> RegionInstance(world, entity, component, generation, active state)
  -> SpatialRegionIndex / immutable query snapshot
       -> PostProcess adapter: camera point + continuous influence + typed stack
       -> Audio adapter: source/listener state + acoustic combination policy
       -> Physics adapter: authoritative overlap pairs + Enter/Stay/Exit stream
       -> Navigation adapter: bake-time triangle classification/clipping
       -> Gameplay adapter: typed trigger conditions/effects
       -> Streaming adapter: world-partition request/lease policy
  -> Editor document / Inspector / gizmo / contributor debugger / diagnostics
```

共享合同只应拥有：

1. stable region、component和shape IDs；
2. source revision、artifact digest、world/Play generation；
3. local transform、world transform、bounds和shape capability；
4. domain/layer/filter metadata与unknown-field roundtrip；
5. activation、load/unload、plugin ownership和diagnostic identity；
6. broadphase/index handle与只读query snapshot；
7. schema/version/migration/cook/pack合同。

以下内容必须保持typed domain ownership：Post Process参数插值，Audio interior/exterior与submix/reverb策略，Physics pair/event时序，Navigation area/agent/bake语义，Gameplay authority/cooldown/effect执行，Streaming cell lease与内存预算。共享geometry不意味着所有域必须使用同一个update loop或同一种精确查询算法。

### 3.2 建议核心类型

| 类型 | 必须字段/职责 | 明确禁止 |
|---|---|---|
| `SpatialRegionId` | project-stable UUID或等价ID、instance lineage | 用entity index或显示名作为持久身份 |
| `SpatialRegionSource` | shape source、transform、domain mask、filters、revision | 内嵌所有域效果字段 |
| `RegionShapeSource` | Box/Sphere/Capsule/Convex/asset-backed/composite引用 | 每个子系统复制一份center/extents |
| `CompiledRegionGeometry` | schema/backend/version/source digest、bounds、exact query data、capabilities | 直接保存Editor live pointers |
| `RegionInstanceKey` | world/entity/component/region/generation | 只用SoundVolumeId或EntityId跨世界寻址 |
| `RegionQuerySnapshot` | immutable records、broadphase generation、budget/overflow | UI读取可变runtime HashMap |
| `RegionDiagnostic` | stable code、owner path、domain、shape/property、revision/generation、fix action | 只返回toast字符串 |
| `RegionEffectComponent` | typed plugin-owned payload与schema version | Damage/Reverb/Checkpoint共用字符串profile |
| `RegionOverlapEvent` | pair/shape identity、sequence、step/time、cause/filter、generation | 仅包装PhysicsTriggerEvent后声称工程完成 |

## 4. P0 阻断项

### P0-1：通用Volume Editor是静态第二authority

固定`VOL_DamageZone`、24 volumes、12 overlaps和queued反馈必须先撤销可工作暗示。没有真实Scene controller、selection、transaction、runtime snapshot和validation receipt时，该workspace只能标记Unavailable，或从production构建删除。

### P0-2：四套区域shape/identity/filter/lifecycle合同互不兼容

PostProcess借Collider提取旋转Box/Sphere，Sound自带world-space AABB/Sphere，Physics拥有完整Collider/sensor pair，Navigation用Empty node scale构造无旋转AABB。一个用户可见边界无法可靠驱动多个域，也无法证明它们看见同一个区域。必须先建立共享identity/geometry artifact与domain adapter边界。

### P0-3：局部Post Process配置可合法保存却在extract中无诊断消失

缺Collider或使用六类不支持shape时，Scene component仍存在但render extract为空。必须在Inspector、Scene validation、cook和runtime diagnostic同时拒绝或清晰标记unsupported；不能依赖测试知道这项隐藏前置条件。

### P0-4：AudioVolume存在服务DTO和DSP，却没有Scene到Sound manager的生命周期桥

component descriptor、drawer operation和Sound service三层没有生产执行者连接。必须按world/entity/component revision创建、更新和删除SoundVolume，处理transform、scene unload、Play generation、plugin disable与device restart；Box rotation和source/listener semantics也必须在桥接前定案。

### P0-5：Damage/Checkpoint/Streaming等Gameplay Volume只有UI名字，没有生产类型与执行链

Physics Enter/Stay/Exit不能替代typed gameplay behavior。必须定义condition/filter、authority、cooldown/once/rearm、effect command、save/replay、network prediction/replication和failure contract；在此之前不得展示DPS、Checkpoint或Streaming成功状态。

## 5. P1 工程化重构项

### 5.1 Identity、Shape、Scene 与 Artifact（P1-01 至 P1-12）

### P1-01：缺project-stable Region identity

定义跨save/reimport/prefab instance/world reload稳定的`SpatialRegionId`，并区分source identity、instance identity和runtime generation。

### P1-02：缺source shape与compiled geometry分层

Scene保存可编辑shape source；cook/runtime安装不可变geometry artifact。artifact key必须包含schema、source revision、transform policy、backend和platform，而不是直接共享live Collider。

### P1-03：缺跨域shape capability matrix

为Box/Sphere/Capsule/Cylinder/Convex/Mesh/HeightField/Compound声明各域支持、approximation和拒绝原因。任何降格都必须显式、可测试、可在Editor预览。

### P1-04：local/world transform与非均匀scale规则不统一

统一handedness、unit、parent transform、collider local transform、rotation、negative/nonuniform scale和large-world origin shift规则；域adapter不得自行猜测。

### P1-05：render layer、physics mask、agent filter和gameplay filter互不映射

建立typed domain filter集合及Editor投影。允许有意不同，但必须显示差异并生成可解释的filter decision，而不是复用一个裸`u32 mask`。

### P1-06：区域与效果是一对一模糊profile

允许一个Region geometry挂接多个typed effect component，每个effect有plugin owner、schema version和组合规则；禁止用Damage/Reverb/Checkpoint字符串切换整套语义。

### P1-07：Scene schema缺Region版本与迁移

定义source/effect schema version、default evolution、unknown field roundtrip、prefab override和hard-cut migration。旧Collider+PostProcess组合迁移需保留行为和诊断。

### P1-08：缺cook-time Region validation/artifact

在pack前验证finite bounds、zero/negative extent、unsupported shape、missing effect dependency、filter capacity和domain capability，并输出可追踪artifact manifest。

### P1-09：缺共享Region registry与incremental index

按world generation安装只读records，使用dirty set更新bounds/filter/effect revision；查询侧读取snapshot，避免每camera或每source遍历全部Scene component。

### P1-10：activation与world/plugin lifecycle合同不完整

active hierarchy、component enabled、scene unload、world destroy、Play exit、plugin disable和hot reload必须原子撤销所有domain proxy，并可证明无stale record。

### P1-11：plugin-owned Region effect缺unknown roundtrip

插件缺失时保留payload、schema和owner identity，Editor显示read-only原因；不得丢字段、自动删除或将未知effect当成功加载。

### P1-12：numeric precision与bounds validation不统一

统一NaN/Inf、极大坐标、退化shape、epsilon、boundary inclusion和touching语义。CPU、Physics backend、Render和Editor gizmo必须在声明容差内一致。

### 5.2 Overlap、Trigger 与 Gameplay（P1-13 至 P1-24）

### P1-13：Trigger event缺pair/shape generation与sequence

增加world step、event sequence、pair generation、trigger/other shape或subshape ID，使destroy/recreate、compound shape和同帧Enter/Exit可确定区分。

### P1-14：Trigger没有typed target filter

支持entity class/component/tag/team/ability state/player index和custom predicate provider，并输出filter命中/拒绝原因；Physics collision mask只负责几何候选。

### P1-15：Exit原因与lifecycle不可解释

区分Separated、TriggerDisabled、OtherDisabled、EntityDestroyed、WorldUnloaded、FilterChanged和BackendReset，避免Gameplay将销毁误判为正常离开。

### P1-16：Stay每physics step无cadence与budget

定义EveryStep、FixedInterval、OnMeaningfulChange和Disabled策略，设每world/event count/bytes预算与overflow telemetry，避免高频区域放大主线程负载。

### P1-17：没有current overlap query/snapshot

提供generation-bound当前重叠集合、pair count和since-enter时间；Editor和Gameplay恢复状态不得重算另一套几何或只依赖瞬时事件。

### P1-18：once/cooldown/rearm状态无owner

typed Gameplay Region保存每target或global激活状态、cooldown clock、rearm policy和authority；状态更新必须可重放、可保存、可迁移。

### P1-19：没有typed event binding与command receipt

OnEnter/Stay/Exit绑定stable operation/effect ID、versioned payload和target selection，执行返回accepted/completed/failed receipt，不以“Generate overlap warning”字符串代替。

### P1-20：网络authority、prediction与replication缺失

声明ServerOnly、OwnerPredicted、ClientCosmetic等模式；pair identity、activation sequence、rollback correction和dedup必须进入network/replay合同。

### P1-21：save/load/replay无法恢复区域状态

Checkpoint、once trigger、cooldown、occupancy和streaming lease必须选择持久化级别，并在load后通过generation reconcile恢复，而不是重复触发Enter。

### P1-22：DamageZone没有damage model

需要damage type/source/instigator、rate vs pulse、delta integration、team/immunity/tag filter、stacking、invulnerability和ability/effect integration；DPS不能只是UI数字。

### P1-23：Checkpoint没有spawn/ownership/commit语义

定义player scope、activation order、spawn transform、save transaction、respawn dependency、多人竞争与rollback。触碰区域不应直接等同于durable checkpoint成功。

### P1-24：StreamingGate没有cell request/lease/budget合同

与Editor16/Runtime streaming owner对接prefetch radius、priority、dependency、memory budget、activation fence、failure fallback和release delay；Trigger只产生意图，不直接加载关卡。

### 5.3 Post Process、Audio、Navigation Domain Adapter（P1-25 至 P1-36）

### P1-25：Post Process local shape前置条件没有结构化诊断

为MissingCollider、UnsupportedColliderShape、InvalidTransform、DegenerateBounds和MaskMismatch建立stable code，在Scene/Inspector/cook/runtime同时展示并可定位字段。

### P1-26：Post Process shape支持与approximation policy未产品化

决定Capsule/Convex/Compound是原生支持、编译为距离场/凸集合还是明确拒绝；禁止静默AABB降格，也不能长期只靠Box/Sphere测试锁死产品上限。

### P1-27：Post Process收集没有spatial/dirty acceleration

建立per-world region index、priority buckets和dirty bounds；多camera/stack/large-world场景只查询相关records，并记录candidate/evaluated/contributor成本。

### P1-28：Post Process profile shared/instance语义不完整

明确shared asset、per-entity override、runtime clone、prefab override和unknown component roundtrip，提供stable component/property ID和migration。

### P1-29：Post Process缺contributor stack调试

对选定camera显示每个volume的mask、priority、distance、shape weight、global weight、override property和最终值，绑定frame/world generation。

### P1-30：Sound Scene bridge缺失

建立`SceneAudioRegionAdapter`按entity/component revision diff调用update/remove，并将SoundVolumeId映射到RegionInstanceKey；world unload与Play切换必须drain。

### P1-31：Sound shape丢失rotation与hierarchy语义

Box应消费共享oriented geometry或明确编译为Sound支持形态；world center/extents不得由Editor手填并与Scene transform分叉。

### P1-32：Sound只选strongest volume，缺typed combination mode

定义Override、Blend、Additive Send、Min/Max Filter等按属性组合规则，处理priority tie、nested room/portal和crossfade；保留stable ID tie-break。

### P1-33：Sound只按source position判断inside/outside

明确listener/source各自区域状态、interior/exterior transition、不同volume之间的穿越和transition time。当前source-only gain不能冒充完整AudioVolume语义。

### P1-34：Sound proxy更新没有generation与batch receipt

批量安装region diff，返回accepted generation、invalid record和removed proxy；audio thread只读取不可变snapshot，避免UI或Scene直接争用HashMap。

### P1-35：Navigation area volume忽略rotation、priority和exact shape

改为消费compiled region geometry与stable conflict policy；至少支持oriented Box并在unsupported shape时阻断bake，不能从Empty scale暗推AABB。

### P1-36：Navigation以source node position代替triangle coverage

按triangle centroid/coverage、polygon clipping或Recast convex area marking执行，并为跨界mesh、重叠modifier、hierarchy继承输出deterministic结果和affected tile统计。

### 5.4 Editor Document、Inspector、Gizmo 与 Preview（P1-37 至 P1-48）

### P1-37：Volume workspace固定业务数据

删除固定名称、数量、DPS和warning；列表必须来自Scene projection，空项目显示真实empty state，Runtime未连接显示Unavailable原因。

### P1-38：Profile dropdown混淆geometry与domain effect

改为Region geometry Inspector加typed effect component列表。Damage、Audio、PostProcess、Nav、Streaming由各自schema/drawer贡献，不共享枚举字符串。

### P1-39：没有Region authoring document/controller

建立selection/world/document revision、draft、transaction、dirty/save、validation、preview session和runtime acknowledgment的唯一controller。

### P1-40：PostProcessVolume缺Scene property access

补齐active/global/priority/weight/blend/profile component的读写、validation、undo和prefab override；不允许只能通过静态Post Process页面编辑字符串。

### P1-41：AudioVolume与NavModifier drawer为空

从runtime schema生成typed fields，并提供自定义shape/filter/area/effect drawer。operation metadata不能替代可执行command factory。

### P1-42：缺Region viewport gizmo

支持pick、move/rotate/scale、Box face、Sphere radius、Capsule handles、local/world mode、snap、cancel和numeric entry；visual shape必须来自同一compiled preview。

### P1-43：缺multi-selection与batch edit

显示mixed values，按component/effect兼容集批量编辑，生成单一可撤销transaction，并对部分失败给出逐entity receipt。

### P1-44：field edit/commit没有transaction/save闭环

Change只更新draft/preview，Submit执行validated command；undo/redo、autosave、recovery、source control conflict和project close沿用Editor02合同。

### P1-45：shared profile与instance override UX缺失

明确Edit Shared、Make Unique、Reset Override、Promote to Asset和引用计数影响，避免修改一个profile意外改变全场景。

### P1-46：Overlap Inspector不消费runtime snapshot

显示真实pair、Enter时间、last event、filter、shape identity、backend/world generation和overflow；支持freeze/filter/select entity，不在Editor重算结果。

### P1-47：Validate按钮是假任务

接shared job admission/cancel/progress，输入绑定source revision，输出typed diagnostics与artifact receipt；stale结果不得覆盖新编辑。

### P1-48：缺跨域contributor/debug视图

选择一个位置、camera、source、listener或entity后，展示PostProcess/Audio/Nav/Gameplay/Streaming各自命中、权重、优先级、拒绝原因和实际consumer generation。

### 5.5 Ownership、Performance、Test 与 Release（P1-49 至 P1-60）

### P1-49：Region effect plugin ownership未定义

manifest声明effect type、schema、compiler、runtime adapter、Editor drawer、diagnostics和maturity；缺任一required owner时功能不可标stable。

### P1-50：domain Scene adapter没有统一注册与依赖排序

按Region geometry -> domain effect -> subsystem proxy顺序激活，声明Physics/Sound/Render/Navigation/Streaming依赖；禁止靠初始化偶然顺序。

### P1-51：world/Play/plugin generation fencing不足

所有query/event/proxy/preview/receipt携带generation，旧world、旧Play、旧plugin和旧artifact结果在安装点被拒绝。

### P1-52：thread ownership与snapshot合同缺失

Scene主线程生成diff，Physics/Audio/Render/Nav各自owner线程安装，不共享可变component引用；UI只读immutable diagnostic/query snapshot。

### P1-53：event与diagnostic队列没有统一预算

定义per-world count/bytes/time上限、drop/coalesce policy、high-watermark、overflow code和shutdown drain，防止Stay或大规模区域淹没Editor/runtime。

### P1-54：缺large scene/multi-camera/multi-listener规模设计

建立broadphase、cell partition、priority bucket和incremental dirty update；Render camera、Audio source/listener和Gameplay query不得各自全表扫描。

### P1-55：跨域priority与tie-break不可解释

priority允许域内含义不同，但排序方向、stable tie、NaN处理、nested resolution和contributor order必须由schema声明并可调试。

### P1-56：headless/server/cook边界未定义

Dedicated server可保留Gameplay/Physics/Streaming而裁掉Render/Audio；cook必须验证被裁域的引用和fallback，不依赖Editor-only组件。

### P1-57：缺schema/parser/malformed/fuzz矩阵

覆盖dynamic component JSON、Scene region、shape asset、profile、artifact、event和plugin payload；malformed输入不得panic、无界分配或覆盖旧artifact。

### P1-58：Builtin/Jolt及域adapter缺共享parity suite

同一shape/filter/motion/lifecycle fixture比较pair序列、boundary tolerance、destroy/disable Exit和generation；明确允许差异并记录backend capability。

### P1-59：缺Region规模benchmark与预算

至少覆盖1/1k/100k regions、1M overlap candidates、多camera/listener、moving regions、large coordinates和plugin unload，记录p50/p95/p99、RSS和每帧allocation。

### P1-60：maturity、migration、telemetry与rollback缺失

release gate必须绑定schema/artifact/runtime/editor版本、qualification evidence和known limits；升级失败可回滚，旧Scene不会被静默重写。

## 6. P2 高阶能力

### P2-01：Convex、mesh与SDF Region

支持asset-backed convex decomposition、triangle/voxel/SDF距离查询，并为各域声明精确度、GPU/CPU residency和fallback。

### P2-02：Boolean与Composite Region

Union/Intersection/Difference/child transform拥有stable node IDs、compiler、bounds和debug visualization，不在运行时递归解释任意Editor graph。

### P2-03：Moving与Deforming Region

为高速移动、骨骼绑定、动画shape和continuous overlap定义swept update、dirty budget与temporal aliasing政策。

### P2-04：Large World分区区域

Region source/artifact按cell切分，跨cell identity与overlap连续，origin rebasing和stream in/out不制造重复Enter/Exit。

### P2-05：GPU resident Region query

为大量视觉/粒子/环境查询提供GPU index与batched influence，同时保留CPU authority、readback预算和可验证reference path。

### P2-06：Weather、Fluid、Wind与Environment Region

在共享geometry之上添加typed climate/fluid/wind/domain adapter，避免下一轮Weather再发明一套region center/extents。

### P2-07：Rollback/predicted overlap

保存pair history、shape generation和authority timeline，为网络rollback、replay scrub和deterministic simulation提供可重建事件。

### P2-08：Data-oriented batch query API

提供point/shape/ray批量查询、SoA结果、query plan与arena预算，支持Gameplay/AI/Audio一次读取共享candidate snapshot。

### P2-09：Authoring conversion与lint automation

可审查地把Collider/Trigger/PostProcess/Audio/Nav旧配置转换为Region source/effects，输出behavior diff与人工确认，不静默改项目。

### P2-10：Multi-user Region editing

stable subobject IDs、field-level conflict、lease/merge和runtime preview ownership支持协作编辑，geometry drag不会覆盖他人effect修改。

### P2-11：Influence/occupancy heatmap与成本分析

以真实runtime snapshots生成camera/audio/gameplay/nav覆盖热图、overdraw/candidate cost和priority conflict建议。

### P2-12：Adaptive hierarchical region index

按world density、motion和query workload选择BVH/grid/cell/hybrid结构，用可重复benchmark证明相对Unreal等参考目标的性能优势。

## 7. 参考引擎差异矩阵

| 参考 | 已验证边界 | Zircon当前差距 | 采用原则 |
|---|---|---|---|
| Unreal `AVolume` | 共享brush/bounds/`EncompassesPoint`，Trigger/Physics/Audio/PostProcess/NavModifier保留各域priority、effect和proxy/lifecycle | Zircon有四套shape/identity，无共享region source/artifact，通用UI却先宣称统一 | 学共享geometry基类与域专用adapter，不复制Actor/UObject布局 |
| Unreal Audio/PostProcess | Audio有listener/source interior/exterior、reverb/submix/proxy；PostProcess有priority/blend radius/weight/unbound和stable tie identity | Sound仅source-position strongest AABB/Sphere且无Scene bridge；PostProcess局部shape无产品诊断 | 分别保留域组合语义，共享region identity/geometry |
| Godot `Area3D` | monitoring/monitorable、body/area map、shape-pair reference count、priority与override combination mode，并含audio bus/reverb | Physics event无shape pair/current overlap/override mode，Gameplay consumer缺失 | 参考pair lifecycle和combination contract，不把Area3D所有职责塞进单类 |
| Unity Graphics Volume | Volume/Profile/Stack/Manager、layer registration、priority dirty、collider cache、per-camera continuous blend与default stack | ZirconRender evaluator真实，但无Editor profile/stack contributor、spatial cache和unsupported shape诊断 | 仅作为渲染Volume参考，不外推到Gameplay/Audio |
| Fyrox Collider | sensor、interaction/solver groups、active intersection由PhysicsWorld拥有 | ZirconPhysics基础接近，但上层pair identity/filter/state/Editor消费不足 | 保持Physics为离散overlap authority，Region层不重算 |
| Bevy ECS trigger | 展示typed event/observer边界 | 本地文件不是空间Volume/Zone产品，也不覆盖authoring | 只参考typed dispatch ownership，不作为功能完成基线 |

目标“优于Unreal”必须由同任务证据证明：更低query/dirty-update成本、更清晰的typed plugin boundary、更强的generation safety、更可组合的domain adapter、更完整的Editor diagnostics和更可重复的benchmark。文件数量、一个万能Region类或静态UI不能构成优势。

## 8. 分层实施路线

| 里程碑 | 内容 | 前置 | 退出条件 |
|---|---|---|---|
| M0 Truthfulness与基线 | Volume workspace降级/移除固定成功；恢复Editor可编译基线；冻结现有四域行为fixture | 无 | UI不再宣称Damage/Checkpoint/Streaming可用；现有PostProcess/Physics/Sound/Nav行为有baseline |
| M1 Shared contract | SpatialRegionId/Source/Shape/Instance/Diagnostic、domain capability matrix、generation与plugin ownership | M0 | schema/API review通过，未引入万能effect bag |
| M2 Scene与Artifact | Scene persistence、migration、compiled geometry、validation、registry/index、unknown roundtrip | M1、Editor02/03/05 | old/new Scene roundtrip，unsupported在cook前拒绝，snapshot可增量安装 |
| M3 Physics/Gameplay | pair identity、current overlap、filter/cadence/budget、typed effect、authority/save/replay | M2、Runtime08A/08G | Damage/Checkpoint fixture由真实event到effect receipt闭环，Builtin/Jolt parity通过 |
| M4 Post Process | local shape diagnostics、shared geometry adapter、profile instance、contributor stack、spatial acceleration | M2、Runtime09H2、Editor22 | camera结果与baseline一致，unsupported不再无诊断消失，大volume stack预算通过 |
| M5 Audio | SceneAudioRegionAdapter、oriented shape、listener/source状态、combination mode、audio snapshot | M2、Runtime08B、Editor17 | Scene edit到可听结果与proxy generation闭环，unload/device restart无stale volume |
| M6 Navigation/Streaming | exact modifier geometry、triangle marking、conflict policy、streaming gate adapter | M2、Runtime08E、Editor16/19 | rotated/cross-boundary modifier corpus正确，stream lease/failure可解释 |
| M7 Editor产品 | Region document、typed inspectors、gizmos、multi-edit、transaction、live overlap/contributor/validation | M3-M6、Editor02/03/05/09/11 | 所有显示来自真实Scene/runtime/job receipt，undo/save/recovery与preview通过 |
| M8 Scale与资格 | benchmarks、fault/fuzz、cross-platform、headless、plugin unload、large world | M7、Tooling07/10 | 预算与required lanes通过，0 ignored required test，qualification artifact可重现 |
| M9 Hard cutover | 迁移旧配置，删除静态workspace/旧shape DTO旁路，生成maturity与rollback证据 | M8 | 全仓无旧authority消费者，旧项目迁移/回滚通过，stable状态由证据生成 |

实施不得跳过M0-M2直接补Damage组件或绘制新Region面板。没有共享identity/geometry与generation时，新增功能只会形成第五套临时区域系统。

## 9. 验收门 G01-G32

### G01：Product truth

新建空项目时Volume页面不显示固定实体、DPS、overlap或warning；Unavailable/empty/ready均来自真实controller状态。

### G02：Schema roundtrip/migration

Region source、typed effects、unknown plugin payload、prefab overrides和旧PostProcess/Collider组合可往返；迁移有version、diff与rollback。

### G03：Shared geometry identity

同一Region在Render/Audio/Physics/Nav/Gameplay diagnostics中报告相同region/source revision与world generation，domain proxy ID可回跳。

### G04：Shape capability truth

所有Collider/Region shape在每个domain得到Supported/CompiledApproximation/Unsupported结果；unsupported在编辑或cook时结构化阻断。

### G05：Transform parity

parent/local rotation、negative/nonuniform scale、large coordinate与origin shift corpus中，gizmo、artifact、Physics、Audio、Render bounds在容差内一致。

### G06：Activation/lifecycle

disable、reparent、destroy、scene unload、world destroy、Play exit和plugin unload会撤销全部proxy；旧generation query/event/receipt被拒。

### G07：Physics event determinism

Builtin/Jolt对Enter/Stay/Exit顺序、双sensor方向、destroy/disable和filter change通过共享fixture，差异有明确capability记录。

### G08：Pair/current-overlap identity

compound shapes、同entity多shape、destroy/recreate和同帧切换可由pair generation/subshape/sequence区分；current snapshot与event stream一致。

### G09：Filter/cadence/budget

tag/team/component/custom filter与Stay cadence按声明执行；event count/bytes超限产生可观测overflow且无无界队列。

### G10：Typed gameplay effect

Damage/Checkpoint fixture从overlap到validated command/effect receipt，once/cooldown/rearm和失败路径可重放，不依赖UI字符串。

### G11：Authority/save/replay

server/predicted/cosmetic模式、save/load和rollback不会重复执行或丢失activation；sequence与correction可诊断。

### G12：Post Process blend correctness

global/local、nested priority、equal priority、mask、weight、blend band和camera stack与CPU reference一致，property interpolation使用typed registry。

### G13：Post Process unsupported shape

缺Collider及Capsule/Cylinder/Convex/Mesh/HeightField/Compound不再静默消失；诊断绑定entity/component/field并提供可执行修复。

### G14：Post Process contributor view

Editor显示真实camera/world/frame generation和逐volume property contribution；最终值与render evaluator snapshot一致。

### G15：Audio Scene bridge

Scene create/edit/delete、undo、unload、Play切换与plugin/device restart按generation update/remove SoundVolume，无orphan proxy。

### G16：Audio transform/shape

rotated Box、parent transform、Sphere非均匀scale与crossfade边界通过听觉数值reference和gizmo overlay，不再使用分叉world extents。

### G17：Audio combination semantics

nested/overlap volume对source/listener、priority、tie、blend/additive send和interior/exterior transition输出符合声明且可调试。

### G18：Navigation triangle coverage

大mesh跨越modifier边界时只标记实际覆盖三角形或裁切后的polygon，不能按source node position整批误判。

### G19：Navigation rotated/conflict corpus

rotated Box、nested modifiers、agent filters、hierarchy、equal priority和overlap conflict得到deterministic area/tile结果与诊断。

### G20：Inspector roundtrip

PostProcess、Audio、Nav、Trigger和Gameplay effect字段由同一schema读写，invalid输入不到runtime，unknown字段保留。

### G21：Gizmo parity

pick/drag/snap/numeric/cancel/multi-select生成同一transaction；可见boundary与compiled/runtime boundary在像素/数值容差内一致。

### G22：Undo/save/recovery

geometry、effect、profile shared/unique和multi-edit支持undo/redo、dirty/save/autosave/recovery/conflict，stale preview不提交。

### G23：Multi-selection

mixed values、兼容effect集合、partial failure和batch receipt正确；一次操作形成一个可解释transaction。

### G24：Live overlap inspector

Inspector消费generation-bound Physics snapshot，能freeze/filter/select并显示pair/filter/cadence/overflow，不另算几何。

### G25：Diagnostics/fix actions

所有失败有stable code、domain、owner path、source revision、runtime generation和fix action；toast只是投影，不是事实源。

### G26：Plugin lifecycle

effect/compiler/adapter/drawer依赖与版本由manifest装配；missing/disable/reload保留数据、撤销proxy并给出确定终态。

### G27：Bounded queues/snapshots

event、diagnostic、preview和proxy diff均有count/bytes/time预算、overflow政策与shutdown drain；压力下无死锁或无界增长。

### G28：Scale/performance

1/1k/100k regions、多camera/listener、1M candidates和moving dirty storm满足声明CPU/内存/allocation预算，性能报告可重复。

### G29：Malformed/fuzz/fault

Scene JSON、shape/profile/artifact/plugin payload、NaN/Inf、corrupt version、job cancel、device/backend reset和disk failure均不panic或破坏旧状态。

### G30：Headless/package

Dedicated server正确保留Physics/Gameplay/Streaming并裁剪Render/Audio；packaged build不依赖Editor path或未声明plugin。

### G31：Cross-platform/backend

Windows/Linux目标、Builtin/Jolt、GPU backend和audio device矩阵证明capability/fallback truth，未支持组合在激活前失败。

### G32：Release/maturity/rollback

Region schema/artifact/plugin版本升级有migration、canary、qualification和rollback；删除旧authority后源码扫描与产品测试均无旁路。

## 10. 跨计划边界

- Runtime06拥有World/ECS/hierarchy/entity lifecycle，本篇只定义Region source/instance如何消费其generation。
- Runtime08A与Editor18拥有Collider、Physics backend、query和debug，本篇拥有从authoritative pair到Region overlap/gameplay consumer的上层合同。
- Runtime08B与Editor17拥有Sound engine、DSP、SceneAudioBridge总目标和Audio authoring，本篇新增共享region geometry、AudioVolume transform/combination和跨域一致性要求。
- Runtime08E与Editor19拥有Navigation bake/query/toolkit，本篇只新增modifier geometry必须接共享Region artifact及triangle coverage，不重复NavMesh总审查。
- Runtime09H2与Editor22拥有Post Process registry/evaluator/profile authoring，本篇新增local shape拒绝诊断、共享identity/index和跨域contributor。
- Editor16拥有World Partition/Level Streaming，本篇的StreamingGate只产生typed request/lease，不复制streaming manager。
- Editor21/Runtime08G拥有Ability/Effect/Attribute/Tag/Cue，本篇的DamageZone等Gameplay effect应调用其typed authority，不另造伤害框架。
- Editor02/03/05/09/11拥有transaction、gizmo、Inspector、job和diagnostic基础，本篇只能接入，不能复制私有实现。

## 11. 禁止的临时修补

1. 禁止只新增`VolumeComponent { kind: String, properties: Json }`。
2. 禁止把Damage、Audio、PostProcess、Navigation、Streaming字段塞进同一万能struct。
3. 禁止继续显示`VOL_DamageZone`、固定DPS/overlap/warning并称为示例数据。
4. 禁止让Validate按钮只返回queued toast或固定计数。
5. 禁止用entity display name或当前ECS index作为持久Region ID。
6. 禁止让每个domain复制center/extents/priority却没有source revision和geometry artifact。
7. 禁止把所有Collider shape无条件转AABB来“支持”PostProcess/Audio/Nav。
8. 禁止缺Collider或unsupported shape时只返回`None`而没有产品诊断。
9. 禁止在Editor手填Sound world center/extents并绕过Scene transform。
10. 禁止把`update_volume()` API存在解释为AudioVolume Scene集成已完成。
11. 禁止用strongest-volume算法冒充所有音频叠加与interior/exterior语义。
12. 禁止按Navigation source node position给整mesh分配area。
13. 禁止让Region层自行重算Physics overlap并与Builtin/Jolt事件并存。
14. 禁止没有pair generation/sequence时直接驱动可持久或网络Gameplay effect。
15. 禁止用每step Stay无预算执行DPS、脚本或Editor日志。
16. 禁止Trigger直接同步加载World Partition cell并绕过lease/budget/fence。
17. 禁止UI线程读取Sound/Physics/Render可变HashMap。
18. 禁止无界event/diagnostic/proxy diff channel。
19. 禁止只补drawer字段而没有transaction/save/runtime acknowledgment。
20. 禁止以Unreal共享`AVolume`为理由复制其Actor层级或制造一个全域owner。
21. 禁止把Unity Graphics Volume外推成Gameplay/Audio/Navigation参考。
22. 禁止用Bevy typed event文件证明空间Region产品已覆盖。
23. 禁止在旧authority迁移和rollback未通过前删除项目数据。
24. 禁止在规模、fault、backend和headless门未通过前把Region/Volume Editor标记stable。

## 12. 本轮产出边界

本轮只新增审查与重构计划，没有修改production Runtime/Editor/interface/plugin/App代码或tests。静态证据覆盖71个显式文件、16,389行、610,757 bytes、81个test attributes和0个ignored，读取时fingerprint为`ba7dfd9dd6d5a8f6c88a9fb3f415c2c26032a3ee2d32bc655c975207d2d0a80c`。

7个在途文件均非本轮产生，实施前必须重算物理范围并复核其终态。本轮没有运行动态测试；此前Editor lib测试编译仍被239个既有错误/122个warning阻断。后续实现必须从M0 truthfulness和可编译基线开始，先建立共享identity/geometry/generation，再逐域接入，不得新增第五套临时Volume系统。
