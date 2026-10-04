---
related_code:
  - zircon_runtime/src/scene/mod.rs
  - zircon_runtime/src/scene/navigation.rs
  - zircon_runtime/src/scene/prelude.rs
  - zircon_runtime/src/scene/runtime_extension
  - zircon_runtime/src/scene/runtime_extension
  - zircon_runtime/src/scene/runtime_level_traits.rs
  - zircon_runtime/src/scene/semantics
  - zircon_runtime/src/scene/serializer
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_runtime/src/scene/module/level_manager_lifecycle.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md
reference_sources:
  - dev/bevy/crates/bevy_app/src/plugin_group.rs
  - dev/godot/core/extension/gdextension_manager.cpp
tests:
  - zircon_runtime/src/scene/runtime_extension/mod.rs
  - zircon_runtime/src/scene/tests/ecs_scheduled_native_systems.rs
  - current-source Windows zircon_runtime scene tests pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime scene root/runtime hook逐文件性能静态审查（2026-07-22）

## 范围与覆盖

`zircon_runtime/src/scene/{mod.rs,navigation.rs,prelude.rs,runtime_level_traits.rs,runtime_extension/**,semantics/**,serializer/**}`当前源 **9/9** 个Rust文件、**688** 行已逐文件阅读；并为调用频率与锁边界继续核对`module/{world_driver,level_manager_lifecycle}.rs`及既有stage-plan源码守卫。`scene/mod.rs`已有共享工作区改动，本切片只读保护，没有覆盖。

这些文件中root/prelude/semantics/runtime-level traits只是零运行期开销的类型或导出边界；serializer把同步scene I/O/投影委托给既有World owner，不在facade重复构造。`SceneNavigationRuntime`内置实现覆盖single-agent入口，剩余全World收集与邻域问题继续归PERF-MVP-437，不重复立项。

## 已直接修复

- `SceneRuntimeHookSet::try_merge`原先先深clone全部既有registration descriptor，再对每个incoming hook线性扫描combined Vec查重；增量安装/热重载最坏`O((N+M)M)`，失败路径也先复制全部既有String。现先收集incoming，以借用`&str`的预分配`HashSet`一次验证existing+incoming identity，成功后才预分配并clone/move canonical ordered Vec；重复检测期望`O(N+M)`，失败只分配错误id。
- `WorldRuntimeExtensionPlan::append_unique`原先把existing与incoming的每个key都clone进`BTreeSet<String>`。现以`BTreeSet<&str>`借用两侧稳定String，验证完成后显式drop借用集合再move incoming，成功路径不再为唯一性校验复制key。
- 两项均先加入源码守卫并观察RED，再修改生产代码得到GREEN；scoped `rustfmt`与`git diff --check`通过。current-source受管Cargo复试时CPU lane被`frameworks03-audio-channel-layout-private-reexport-closeout-r3-20260722`精确预约，本轮未启动Rust测试，因此PERF-MVP-450仍为code-fixed/dynamic-pending。

## 仍需架构修复

`WorldDriver::apply_world_runtime_extensions`从`runtime_extensions: Mutex<WorldRuntimeExtensionPlan>`取得guard后，在guard生命周期内依次执行所有type-erased extension callback。该入口在每次level/world创建调用：慢插件把所有并发World初始化串行在同一锁下，callback若重入安装/应用边界还有自锁风险，且主线程world bootstrap会把不可控callback wall time算入F0/F2。

正确修复不是每次apply深cloneplan：Runtime06/Plugins01应发布`Arc`持有的immutable registration generation，WorldDriver只在短锁内取得generation handle，锁外按稳定顺序执行callback；安装用copy-on-write构造候选并原子发布，in-flight snapshot保持旧generation存活。问题登记为PERF-MVP-451并交接Runtime06。

## 参考引擎对照

Bevy plugin group同样在配置期以map维护identity、以Vec维护canonical order，支持把查重复杂度留在注册期而不进入frame scheduler；Zircon本轮沿用该边界，只清除无消费者的二次扫描/字符串复制。Godot GDExtension manager的frame/lifecycle遍历稳定extension引用并执行callback，没有把不受信任callback包进一个可见的manager全局mutex；这里迁移的是“短锁快照、锁外调用”原则，而不是复制容器实现。

## 动态验收

1. current-source scene focused Cargo：extension duplicate/failed merge preservation、hook order/duplicate/stage-plan reuse以及完整scene library tests。
2. 1/100/1000 existing+incoming hooks/extensions记录identity probes、String clone bytes、merge wall/alloc；修复后successful uniqueness key clone=0，hook probes近线性，duplicate failure不clone existing descriptors。
3. 1/8/64并发World创建 × 0/10/1000ms callback、callback重入install fixture，记录mutex wait/hold、callback-in-lock wall和F0/F2 p95；PERF-MVP-451完成后callback-in-lock必须为0且无死锁，顺序/失败原子性等价。

动态与产品验收完成前本切片继续保留在`pending.md`，不进入`review.md`。
