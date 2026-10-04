---
related_code:
  - zircon_runtime/src/animation
consumer_code_read_only:
  - zircon_runtime/src/core/framework/animation
  - zircon_plugins/animation/runtime/src/evaluation/pipeline
  - zircon_plugins/animation/runtime/src/manager.rs
  - zircon_plugins/animation/runtime/src/module.rs
base_reports:
  - docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md
  - docs/plans/performance/01/2026-07-22-runtime-animation-static-review.md
owner_plans:
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - docs/plans/zircon_runtime/runtime/14-runtime-module-family-closeout.md
  - docs/plans/zircon_plugins/04-animation.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Animation/AnimInstance.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Animation/AnimNodeBase.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Animation/AnimSequence.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/Animation/AnimSync.h
doc_type: currentness-revalidation
status: static_current_revalidated_structural_and_dynamic_pending
---

# Runtime animation core当前性重验（2026-08-23）

## 冻结边界

| 模块 | 已逐文件复读 | physical lines | bytes | inline tests |
|---|---:|---:|---:|---:|
| `zircon_runtime/src/animation`根文件 | 4/4 | 818 | 28,409 | 7 |
| `zircon_runtime/src/animation/manager` | 6/6 | 852 | 29,173 | 3 |
| `zircon_runtime/src/animation/sequence` | 7/7 | 789 | 28,655 | 7 |
| 合计 | 17/17 | 2,459 | 86,237 | 17 |

ordered relative path + NUL + raw bytes + NUL SHA256为
`74f429d2a613ae1c0cfc4861d2e40a9b147b67432ca175cab0ba2fdc945df82d`。
`clip_event.rs`与`manager/mod.rs`有其他Session的纯格式改动，本轮只读保留。

调用链复读到
`RuntimeDynamicSession::build -> linked animation或builtin fallback module ->
animation.evaluate PostUpdate system -> scan -> sample/graph/state-machine/layer/IK -> scene/physics publication`。
插件pipeline、framework asset/trait是consumer证据，不在本报告17/17验收分母内；尤其
`zircon_plugins/animation/runtime/src/**`的完整currentness仍未验收。

## 已纠正的旧结论

- clip event已经用`BinaryHeap`合并track candidate，当前batch选择复杂度是`O(T + E log T)`，不再是旧版
  quadratic selection；events/bytes/playback span也已有边界。
- sequence channel已经用`partition_point`找区间，但每次sample之前仍全量检查key time有限性，所以当前
  总复杂度仍是`O(K)`，不能只按二分定位宣称`O(log K)`。
- compiled sequence已经在compile边界解析entity/property writer，帧路径不再解析property文本；剩余问题
  是raw channel解释、通用World property mutation和每帧asset load，不是“完全没有compiled binding”。
- linked animation与builtin fallback在单个dynamic session中互斥，不是同帧双重执行；问题是两套同名
  module/manager及graph/pose/state-machine实现独立演化，缺少唯一implementation owner。

## 当前结构瓶颈

### A-P1-1：框架接口把source asset求值暴露为manager同步方法

`AnimationManager`直接接收`AnimationGraphAsset`、`AnimationStateMachineAsset`、
`AnimationSkeletonAsset`和`AnimationClipAsset`执行求值。`DefaultAnimationManager::sample_clip_pose`每次先从
skeleton重建带`String`名称的`Vec<AnimationPoseBone>`，再为每条track按name/path找bone。target path分支会
对bone候选反复构建ancestor `Vec<String>`和joined path；最坏规模接近`O(T * B * D)`并伴随字符串分配。

graph evaluator对每次递归都线性扫描node表，clone参数、node id和target id；一般tree为`O(V^2)`，共享
subgraph还会按到达路径重复展开。state machine同样以字符串线性扫描state/transition并clone完整参数表。
这些都属于缺少导入期compiled program、dense slot和instance state，不应靠单个HashMap补丁固化错误接口。

### A-P1-2：raw editable数据直接进入帧循环

`AnimationClipAsset`保存bone/target字符串、完整key `Vec`和未压缩f32通道。有限性、target解析、constant
track、区间索引与误差预算没有形成versioned cook receipt。channel sample每次扫描全部key验证时间，随后才
二分定位；pose sample每次重建bind pose。event sampler虽已有有界heap，仍在请求时同步
`load_animation_clip_asset`并为发出的target/event/payload clone字符串。

目标不是删除一次检查，而是让Editor/source asset只存在于authoring/import边界，runtime只消费带rig
signature、dense track slot、constant/default mask、quantized segment/page table、event index、profile hash和
compression error receipt的`AnimationClipCookArtifact`。

### A-P1-3：局部worker后立即阻塞，完整phase仍以owner线程串行组织

真实插件只把direct clip分成最多4个round-robin shard；每帧新建`sync_channel`，忽略schedule返回值，然后
owner线程逐个`recv()`。graph、state-machine、layer、IK、sequence和scene apply随后串行执行。同步join本身
不是错误，但当前没有prepared shared generation、stable instance batching、deadline/cancel/stale result和
typed submit failure，worker cache还会按round-robin重复驻留资源。

目标使用Runtime11唯一task runtime表达`Update -> Sample/Decompress -> Blend/IK -> Commit` DAG：owner lane
只推进实例时钟、transition/trigger/event intent和最终确定性提交；纯求值按rig/program generation批处理，
scratch按worker复用。禁止再建animation私有线程池或每帧channel。

### A-P1-4：pose仍是String/AoS并逐骨骼写回通用scene

插件最终遍历每个pose/bone，通过name binding生成`Vec<(EntityId, Transform)>`，再逐项
`world.update_transform`；physics publication又clone bone name生成另一份targets。pipeline为了判断是否发布，
还会clone/deep compare完整`BTreeMap<EntityId, AnimationPoseOutput>`。

目标是rig-scoped dense SoA pose page、required-bone/LOD mask与generation receipt。renderer消费palette/
deformation request，physics只消费body subset，socket/attachment只投影选定bone；Editor用inspection adapter
展示虚拟骨架。不能保留“dense pose + 每bone entity写回”的长期双写兼容路径。

### A-P1-5：sequence、event、IK仍未进入统一实例事务

sequence每帧同步load active asset，逐track sample并写通用World property；IK命令虽有4096/world容量和
replacement epoch，consumer仍同步load skeleton并构建model pose；event有ingress预算但字符串ABI和下游
publication仍未cook。它们需要共享instance clock、prepared lease、defer/admission receipt和commit顺序，
否则局部优化只会增加多个状态owner。

### A-P1-6：builtin fallback与插件production owner尚未硬切

dynamic session在未链接animation插件时装载`zircon_runtime::animation::AnimationModule`，链接时使用插件。
两边module除路径和constructor形态外近乎相同，pose/state-machine等manager实现主要只有crate import差异；
但真正scene evaluator只在插件。fallback因此既不是完整production evaluator，也不是明确Unavailable contract。

目标按Optimize08c硬切：framework保留稳定asset/command/status contract；animation插件拥有唯一manager、
instance registry、runtime system和diagnostic。未链接插件时发布typed unavailable reason，不启动第二套raw
evaluator。迁移必须一次性删除重复module/manager与旧use site，不留re-export或兼容facade。

## Unreal源码裁决

- `AnimInstance.h:1414-1429,1713-1722`明确区分parallel update/evaluate、game-thread pre/post和proxy生命周期；
  Zircon应吸收phase与proxy/snapshot边界，不复制UObject层次。
- `AnimNodeBase.h:729-754`把initialize、cache bones、update、evaluate定义为AnyThread节点合同；这支持
  compiled program、required bones和immutable worker input，不支持manager同步解释raw asset。
- `AnimSequence.h:276-278,380-383,427,506-508,766-818`区分raw/editor与cooked/platform compressed data，
  并管理platform cache和read scope；Zircon必须有自己的versioned prepared artifact、residency和error receipt。
- `AnimSync.h:16-32,42-51,63-113`把tick record、sync group/marker和双缓冲read/write状态放到实例同步owner；
  这说明sequence/state-machine/event不能各自维护互不关联的time map。

参考源码给出的是结构证据，不是声称Unreal的具体耗时阈值适用于Zircon。没有相同角色数、骨数、压缩质量、
LOD、worker和画质的同机数据前，不写“接近Unreal功耗/耗时”。

## 实施顺序

1. **M0能力真相与唯一owner**：冻结linked/unlinked行为；定义typed unavailable；硬切重复module/manager，
   插件成为唯一production owner，framework trait移除raw求值方法。
2. **M1 Rig/Clip prepared artifact**：import/cook完成validation、dense binding、constant elimination、压缩、
   event index、profile hash和quality receipt；frame禁止owned `load_animation_*_asset`。
3. **M2 Instance registry与compiled program**：一个world generation内由唯一slot拥有clock、parameters、
   transition、trigger、event cursor和prepared lease；graph/state-machine编译为dense program。
4. **M3 Dense pose与任务DAG**：required-bone/LOD SoA pose page；Runtime11执行sample/blend/IK DAG；owner
   lane只commit，renderer/physics/socket各消费正式generation bridge。
5. **M4统一sequence/event/IK事务**：同一clock/admission/defer/commit receipt；通用property mutation只保留
   明确的non-skeletal timeline owner。
6. **M5动态验收**：当前源码可执行产物上用WPR/xperf采集main/worker CPU、ready/queue age、context switch、
   allocation、RSS和功耗；RenderDoc只验skinning/palette/draw/pixel/GPU pass，不代替CPU profiler。

## 当前验收状态

本轮完成17/17 current-source静态复读、产品caller和Unreal源码核对，没有生产代码修改。原因不是放弃简单
优化，而是当前确定性热点都依赖prepared artifact、instance owner和phase contract；直接删validation、缓存
字符串或扩大worker数会掩盖结构错误。

可执行的静态门结果：`test_animation_runtime_helpers_arc_import`与
`test_frameworks_01_scene_animation_boundary`合计11/11通过（46.220s）；17/17 animation Rust文件的
`rustfmt +1.94.1 --edition 2021 --check`与scoped diff check通过。docs convention扫描3,152份文档、
83,772条路径，复现全仓既有801项违规，本轮两份文档owned violation为0。

managed Cargo执行身份已归档，Rust test执行数为0；当前源码也没有可启动的产品二进制，因此WPR、功耗和
RenderDoc采样数为0。后续只可在current-source launchable artifact出现后做动态验收。本模块保持
`static_current_revalidated_structural_and_dynamic_pending`，不得写入`review.md`。
