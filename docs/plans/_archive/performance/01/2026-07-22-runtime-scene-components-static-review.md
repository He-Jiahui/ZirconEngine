---
related_code:
  - zircon_runtime/src/scene/components
  - zircon_runtime/src/scene/world/query.rs
  - zircon_runtime/src/scene/inspection/snapshot.rs
  - zircon_runtime/src/navigation/runtime.rs
  - zircon_runtime/src/animation
  - zircon_runtime/src/script/vm
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_plugins/03-physics.md
  - docs/plans/zircon_plugins/04-animation.md
  - docs/plans/zircon_plugins/05-navigation.md
  - docs/plans/zircon_runtime/runtime/13-script-binding-and-reflection.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
reference_sources:
  - dev/bevy/crates/bevy_ecs/src/query
  - dev/godot/scene/main/node.cpp
tests:
  - current-source Windows zircon_runtime scene component/reflection tests pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime scene components逐文件性能静态审查（2026-07-22）

## 范围与覆盖

`zircon_runtime/src/scene/components/**`当前源 **20/20** 个Rust文件、**1,523** 行已逐文件阅读：root 1、`render2d` 3、scene owner 12及reflection adapters 4。范围覆盖identity/hierarchy/activation/transform、camera/lights/render2d/mesh/post-process、physics/animation、SceneNode/NodeRecord与LocalTransform/MeshRenderer/RigidBody reflection。

共享工作区正在把tracked `scene/components/scene.rs`硬切为folder-backed `scene/{activation,animation,camera,hierarchy,identity,mesh_renderer,node,physics,transform}.rs`等owner，`components/mod.rs`也有外部改动。本切片完整读取current source但不覆盖这些迁移文件，不把外部结构变化归为本轮实现。

## 性能结论

多数文件只是Copy/small owned component schema、serde/default与reflection metadata声明，不自行进入frame循环。camera/light/post-process defaults为固定规模；render2d/resource default只在component构造执行。没有证据支持为这些冷构造新增缓存或scheduler。

真正的跨子系统放大器是几乎重复的wide `SceneNode`/`NodeRecord`：每行同时携带name、camera、mesh（含morph/primitives/LODs/material overrides）、2D、五类light、physics shape/joint、五类animation player。`World::node_records()`按全部entity重新project、深clone并排序该宽DTO，而不同consumer通常只需要其中一两类component。

该根因已经由明确consumer计划治理，不再新增泛化重复任务：

- Physics的shape/material/joint全量clone归PERF-MVP-335/Plugins03；
- Navigation agent/obstacle JSON投影归PERF-MVP-437/Plugins05；
- Animation player/skeleton/pose apply归PERF-MVP-439/440/Plugins04；
- Script binding/gameplay query归PERF-MVP-442/443/Runtime13；
- Editor inspection/hierarchy/stats归PERF-MVP-456/Editor05。

这些任务共同要求consumer硬切到typed component query/change generation与query-specific dense projection；不得通过缓存完整NodeRecord形成新的wide authority。Runtime07/08最终应限制`node_records()`为显式兼容/序列化边界，frame/tick/host/editor consumer不得使用。

Reflection adapter中MeshRenderer的morph/primitives/LODs会构造完整`ReflectedValue::List/Map`并为static map key/resource id分配String，RigidBody enum write会normalize String；它们当前只由选中实体inspection/显式编辑触发。steady F4重建问题已并入PERF-MVP-456，单个用户写操作的短String不抢占MVP frame热点，也不在没有interned reflection schema前增加局部cache。

## 参考引擎对照

Bevy ECS以typed query/filter只访问声明组件，不要求系统先生成包含所有可选组件的统一Node DTO；Godot Node以具体type/property与变更信号供编辑器/运行期consumer读取。Zircon保留SceneNode/NodeRecord用于序列化/兼容可以成立，但热consumer应沿typed storage与change generation读取。

## 动态验收

current-source受管Cargo取测试lane时被`runtime10-runtime03-animation-frame-demand-producer-20260722`精确预约，本轮未启动Rust测试。后续需：

1. scene component serde/default/reflection完整Cargo，验证folder owner hard cut没有漏字段、旧path或ABI/schema漂移。
2. nodes 1/1k/100k、各component稀疏度0/1/50/100%，分别运行Physics/Navigation/Animation/Script/Inspection产品路径，记录NodeRecord builds、wide fields cloned、payload bytes与consumer p95。
3. 对应PERF任务完成后所有stable hot consumer的`node_records()`调用为0；changed工作与目标typed component/dirty entity数相关。兼容序列化的单次wide snapshot保持字段/顺序/serde parity。

动态验收未完成，因此该目录继续保留在`pending.md`，不进入`review.md`。
