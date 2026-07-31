---
related_code:
  - zircon_runtime/src/scene/inspection
  - zircon_runtime/src/scene/world/query.rs
  - zircon_editor/src/scene/viewport/edit_mode_projection
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
  - docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
reference_sources:
  - dev/godot/editor/scene/scene_tree_editor.cpp
  - dev/bevy/crates/bevy_ecs/src/query/filter.rs
tests:
  - zircon_runtime/src/scene/inspection/tests.rs
  - zircon_runtime/src/scene/inspection/subscription/tests.rs
  - zircon_runtime/src/scene/tests/inspection.rs
  - current-source Windows zircon_runtime inspection tests pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime scene inspection逐文件性能静态审查（2026-07-22）

## 范围与覆盖

`zircon_runtime/src/scene/inspection/**`当前源 **7/7** 个Rust文件、**812** 行、**11** 个tests已逐文件阅读。原5/5审查后共享工作区新增`subscription.rs`及其测试，本次已增量阅读；继续核对`World::node_records/active_in_hierarchy`与editor `edit_mode_projection/build.rs`真实consumer：editor consumer当前仍受`cfg(test)`门控，生产F4尚未调用，但Runtime inspection API本身是production且是未来Hierarchy/Inspector直接边界。

## 已直接修复

- `WorldInspection::from_world`原先先完整构造hierarchy rows，再为focused entity第二次线性遍历全部rows。现把focus作为builder输入，在row首次写入时直接设置，删除无消费者的第二遍O(N) pass。
- parent→children只做按key lookup、从不按parent key迭代；原`BTreeMap`的O(logN)插入/查询改为预分配`HashMap`，child Vec仍按已排序node输入保持确定顺序。rows与visited按node count预分配。
- reflected component已返回owned field Vec，旧代码又clone每个`field_name`构造临时HashMap key。现保留owner Vec并用`HashMap<&str, &ReflectFieldValue>`借用identity；最终inspection DTO所需metadata/value clone不变。
- 源码守卫先观察RED，修改后GREEN；scoped rustfmt/diff check通过。本轮局部止损并入PERF-MVP-456。current-source受管Cargo取得测试lane时被`runtime13-plugin08-script-call-table-atomic-hardcut-20260722`精确预约，本轮未启动Rust测试。

## F4生产启用前的架构门

每次`inspect_hierarchy`仍调用`node_records()`：按全部entities重新project owned `SceneNode`、深clone名字/组件、排序，再建立node/parent/visited/row-index/subtree-hash/edge多份容器并为每row复制display/kind。`inspect_fields`对选中entity遍历整个TypeRegistry、逐component contains/read、复制所有editor-visible metadata/value并全量sort。

editor test-only projection随后把runtime hierarchy/fields再投影成第二套owned DTO，并调用`build_stats`第二次`node_records()`全场景扫描。若直接解除`cfg(test)`，idle editor帧会把全hierarchy/inspector/stats工作绑定到投影调用频率，`WorldInspection.generation/subtree_hash`只被输出、不用于producer cache或delta。

PERF-MVP-456交接Editor05（Editor02/Runtime07共同验收）：按world hierarchy/name/active/type/component generation发布共享immutable inspection projection与added/changed/removed rows；selection变化只重建选中entity field table，viewport/hierarchy/stats消费同一artifact。editor不得再复制第二套完整runtime DTO；stable generation producer visits/build/clone必须为0。

## 新增subscription table的invalidate放大

新增`SubscriptionTable`已提供nonzero token、by-key/by-token authority、dirty token去重排序和frame flush，这些是PERF-MVP-456按generation驱动的必要基础。但`invalidate_subtree`当前遍历全部WatchKey，并对每个subtree watch重新分配`BTreeSet`、从同一entity走一遍ancestor chain；watches×depth放大直接落在mutation throat。`invalidate_all_assets`同样扫描全部key并先collect临时Vec；component type invalidation为一次lookup构造owned String；pending facts在flush前没有count/bytes预算。

该代码是其他会话当前未跟踪实现，本轮只读保留，没有覆盖其TDD切片。PERF-MVP-468已交接Editor02：按WatchKey variant建立typed direct maps，单fact只走一次祖先链并以root direct lookup tokens；component使用interned/borrowed identity，asset reload不扫描非asset key，fact storm有coalesce和显式budget。

## 参考引擎对照

Godot `SceneTreeEditor`接收`tree_changed`/`node_renamed`信号，按节点/子树更新并把完整tree update延迟合并，不把Hierarchy UI定义为每idle frame无条件重扫；Bevy change detection提供component change tick/filter供派生投影按变更消费，其centralized observer storage还按event/component/entity分别建立direct map，而不是触发时扫描异构watch key。Zircon已有world generation与subtree hash，下一步应把它们变成producer-owned invalidation/delta合同，并让subscription lookup按variant/root直接命中。

## 动态验收

1. current-source inspection Cargo：composed snapshot、focus、rename/reparent/cycle、5k deep hierarchy与reflection field tests。
2. nodes 1/1k/10k/100k、depth 1/64/5k、components/types 1/100/10k，stable/rename/reparent/active/selection/field edit：记录node_records/project/sort、map entries、row/field String/value clone、type scans、subtree visits、build count与p95。
3. PERF-MVP-456完成后stable generation hierarchy/stats producer build/visits/clone=0；selection-only hierarchy rebuild=0、field work只与selected components相关；rename/reparent只更新受影响row/ancestor hash，viewport/hierarchy/stats共享artifact。解除`cfg(test)`前须通过F4产品trace与current-source editor Cargo。
4. watches/depth/facts 1/1k/100k的spawn/reparent/reload storm记录ancestor walks、visited allocations、key scans、String/Vec alloc、pending count/bytes/age；PERF-MVP-468要求ancestor walk≤1/fact、工作近depth+matched tokens且队列/RSS有界。

动态验收未完成，因此该目录继续保留在`pending.md`，不进入`review.md`。
