---
related_code:
  - zircon_runtime/src/animation
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_plugins/04-animation.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
reference_sources:
  - dev/bevy/crates/bevy_animation/src/lib.rs
  - dev/bevy/crates/bevy_animation/src/graph.rs
  - dev/Fyrox/fyrox-animation/src/track.rs
  - dev/Fyrox/fyrox-animation/src/pose.rs
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimInstance.cpp
tests:
  - zircon_runtime/src/animation/sequence/tests.rs
  - zircon_runtime/src/animation/manager/graph.rs::performance_contract_tests
  - zircon_runtime/src/animation/scene_hook/graph.rs::performance_contract_tests
  - current-source Windows Cargo and animation-scale product traces pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime animation逐文件性能静态审查（2026-07-22）

## 范围与覆盖

`zircon_runtime/src/animation/**`当前源 **28/28** 个Rust文件、**2,849** 行、**7** 条测试已逐文件阅读，覆盖clip event、manager graph/parameter/pose/state-machine、scene scan/tick/pose/graph/events、sequence apply/channel/interpolation/target及全部测试。现有测试只覆盖基础sequence、channel、node pose与lock恢复，没有active/paused规模、分配、asset load、事件预算或主线程时间门。

## 关键瓶颈

- **PERF-MVP-439 / Plugins04**：scene hook每frame clone playback runtime maps、收集所有entity并探测/clone player；clip/sequence暂停态仍采样/应用，PoseApply再按pose×bone多轮全node扫描。disabled/empty仍replace maps并写10条diagnostic series。
- **PERF-MVP-440 / Plugins04/Runtime11**：built-in manager保留Plugins04 M1 hard-cut前的字符串/owned算法。graph递归逐节点线性scan、参数与clip Vec重建；state event与pose重复evaluate同一graph；channel每sample全量finite校验+线性window；track-to-bone构造path String并多次线性找bone；blend/additive按bone name嵌套到`O(poses*B²)`，纯求值全部串行。
- **PERF-MVP-441 / Plugins04/Runtime11**：loop event按时间跨度逐occurrence while生成、clone并sort，无count/time/bytes预算；大seek、恢复或极短clip可制造无界主线程循环、event Vec和RSS。
- render消费侧的pose BTreeMap deep clone复用PERF-MVP-027，skinning palette/CPU skin/morph与history复用386/405，不重复创建根因。

## 本轮直接止损

1. graph Blend节点直接按input index计算首权重与共享trailing weight，删除每次节点求值的临时`Vec<Real>`。
2. base pose blend把`Vec<GraphWeightedPose>`转owned iterator并取首项，删除`first()?.clone()`造成的完整bone Vec/name/transform深拷贝；后续仍按原顺序消费其余姿态。

两项完成RED→GREEN源码契约、`rustfmt --edition 2021`与scoped `git diff --check`。语义保留zero/one/many input权重、base/additive顺序和active state输出。

## 参考约束与动态验收

Bevy使用稳定`AnimationTargetId`、indexed graph和threaded graph artifact，并以parallel query更新player/target；Fyrox track保留fetch hints，pose按稳定key映射混合；Unreal把Pre/ParallelUpdate/ParallelEvaluate/Post分相并用`NeedsUpdate`避免无条件工作。仓内Plugins04也已实现revision-aware compiled evaluator、dense target table与PosePool，因此正确方向是删除后备分叉并复用同一artifact，而不是继续微调第二套字符串求值器。

需要1/100/10k entities/graphs/nodes/keys/bones、1/100/1k instances及16ms/1h loop event跨度，记录visits、graph evaluations、asset loads、String/Vec/pose bytes、channel/bone comparisons、diagnostic writes、event queue age/depth、worker/main-thread time和p95。当前managed Cargo CPU lane由Plugins01预约，未运行raw Cargo；规模counter和产品trace完成前留在`pending.md`，不进入`review.md`。
