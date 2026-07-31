---
related_code:
  - zircon_runtime/src/scene/world/dynamic_components.rs
  - zircon_runtime/src/scene/reflect/dynamic_component.rs
  - zircon_runtime/src/script/vm/reflection
  - zircon_plugins/zr_vm_language/runtime/src/reflection_host
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/13-script-binding-and-reflection.md
  - docs/plans/zircon_plugins/08-zr-vm.md
reference_sources:
  - dev/bevy/crates/bevy_reflect/src/type_registry.rs
  - dev/godot/core/object/class_db.cpp
tests:
  - zircon_runtime/src/scene/tests/ecs_dynamic_components_structure.rs
  - zircon_runtime/src/scene/tests/ecs_reflect/dynamic_components.rs
  - zircon_runtime/src/script/vm/reflection/tests.rs
  - current-source Windows zircon_runtime dynamic-component tests pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime scene world dynamic components逐文件性能静态审查（2026-07-22）

## 范围与覆盖

`zircon_runtime/src/scene/world/dynamic_components.rs`当前源 **1/1** 个Rust文件、**642** 行、**0** 个就地tests已逐行阅读，并追到reflection adapter、VM catalog/reflection host与结构/行为测试。该文件同时拥有plugin component descriptor、JSON payload、presence、property conversion和VM schema同步，是Runtime13/Plugins08共享控制面与数据面边界。

## 已直接修复

`validate_retained_vm_payloads`原先对每个incoming registration重新扫描所有entity dynamic-component maps并查type path，成本为registrations×entities，即使绝大多数entity没有目标type也照样访问。现先单遍把现存payload按type path分组，再保持registration输入顺序直接验证匹配payload；空registration直接返回。源码守卫先RED后GREEN，rustfmt/diff通过，本轮记PERF-MVP-461。

current-source受管Cargo最近一次申请仍被`runtime10-runtime03-animation-frame-demand-producer-20260722`精确预约，本轮未运行raw Cargo。

## 仍待既有架构任务

- VM单字段JSON写入先深clone整份component、插入一个field，再全对象schema验证；验证对每个JSON field线性find reflected field并第二遍检查missing fields，最坏payload clone + O(F²)。PERF-MVP-443/331须用typed/dense accessor与事务delta校验，不把JSON作为稳定游戏热路径。
- `prepare_vm_type_sync` clone完整ComponentTypeRegistry/TypeRegistry/sets，逐registration upsert与short-path rebuild，随后验证全部retained payload；`register_vm_type`也复制两张registry以事务提交。PERF-MVP-446发布一次prepare/validate的immutable registry generation和World delta。
- `dynamic_components_for_entity`为公开snapshot深clone全部component id、JSON value与descriptor并排序；仅允许在明确inspection/export边界调用，脚本/动画/反射单字段不能借此取值。
- property descriptor write仍线性扫描properties；compiled schema field index与scene property PERF-MVP-329/331/443共享同一identity owner。

## 参考引擎对照

Bevy TypeRegistry在注册时直接维护TypeId/full/short path索引，运行查询借用registration；Godot ClassDB用StringName直接查`property_setget` getter/index。两者都把schema identity编译到注册owner，而不是在每次字段写入时重建/遍历整份动态schema。Zircon需要保留VM catalog原子事务，但prepare一次、publish generation、数据面按dense field accessor验证delta。

## 动态验收

1. current-source Cargo：VM register/sync/remove/collision、retained payload rollback、dynamic property read/write、plugin unload/presence与reflection parity。
2. registrations/entities/types/fields/payload bytes为1/100/10k与1B/1MiB，prepare/sync/write记录entity-map probes、registry builds/clones、JSON clone bytes、field probes、validation count与p95。
3. PERF-MVP-461要求retained scan近existing dynamic components + matched payloads，不再registrations×entities；PERF-MVP-446要求prepare build/validate≤1、commit 0，PERF-MVP-443要求single-field write不clone/validate无关payload字段。

动态Cargo、规模counter与F2/F4插件/脚本产品trace未完成，因此本文件继续保留在`pending.md`，不进入`review.md`。
