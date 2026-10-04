---
related_code:
  - zircon_runtime/src/scene/reflect
  - zircon_runtime/src/scene/world/dynamic_components.rs
  - zircon_runtime/src/script/vm/reflection
  - zircon_editor/src/core/editing/command.rs
  - zircon_editor/src/ui/workbench
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/13-scripting-and-reflection.md
  - docs/plans/zircon_plugins/08-scripting-runtime-and-zr-vm.md
reference_sources:
  - dev/bevy/crates/bevy_reflect/src/type_registry.rs
  - dev/godot/core/object/object.cpp
  - dev/godot/core/object/class_db.cpp
tests:
  - zircon_runtime/src/scene/reflect/derived/tests.rs
  - zircon_runtime/src/scene/reflect/vm_type_backing.rs
  - zircon_runtime/src/scene/tests/ecs_reflect
  - current-source Windows zircon_runtime reflection tests pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime scene reflect逐文件性能静态审查（2026-07-22）

## 范围与覆盖

`zircon_runtime/src/scene/reflect/**`当前源 **26/26** 个Rust文件、**2,713** 行、**5** 个就地tests已逐文件阅读，并追到World动态组件、VM catalog、editor command/workbench Inspector的生产consumer。范围包含root注册表/转换/动态组件/World反射，`builtin_reflection/**`、`derived/**`、`json_document/**`，没有把测试存在或静态阅读冒充动态验收。

## 已直接修复

- `derived_component_registration`原先先构造一次`ReflectTypeRegistration`取type path，再进入公开custom-adapter构造器重新生成整份字段/metadata。现两个入口各只构造一次并把owned registration交给统一finish helper，内置组件启动注册不再重复生成descriptor。
- 动态组件`read_fields`已经持有注册表与字段descriptor，旧实现却对每个字段再次进入`read_field`，重复查注册表并线性找字段，形成`O(F²)` schema probes。现直接复用当前`ReflectFieldInfo`调用`read_declared_field`，保留字段顺序、错误与值转换语义。
- VM type upsert/remove的短路径索引重建原先先clone全部full/short path到临时Vec，再次clone进入BTreeMap/BTreeSet。现从registrations借用流式构建replacement indexes，只为最终索引所有权分配一次，不保留中间全表。
- 三组源码守卫均先观察RED、修改后GREEN；scoped `rustfmt --check`与`git diff --check`通过。本轮归档PERF-MVP-457。受管Cargo test lane被`runtime10-runtime03-animation-frame-demand-producer-20260722`精确预约，本轮没有运行raw Cargo。

## 仍需责任计划完成的热路径

- World反射的字符串字段查找、动态property path String/Vec构造、derived component单字段写入前整组件clone，以及editor/VM高频read/write边界，应由PERF-MVP-331/443的interned call ABI、dense field slot与typed access scope收口，不在这里新增兼容快路。
- VM catalog prepare/commit、`apply_to_world`与revision snapshot仍会复制registration、重建registry并跨World验证；本轮仅降低单次重建常数，generation-owned immutable artifact/delta仍按PERF-MVP-446交接Runtime13/Plugins08。
- `list_reflect_types`和Inspector `reflect_fields`必须输出owned公共DTO；是否缓存/共享须以registry/world generation为失效依据，不能用全局永久缓存隐藏reload或字段值变化。

## 参考引擎对照

Bevy `TypeRegistry`在registration写入时直接更新`TypeId`、full path、short path与ambiguity索引，并让index helper借用registration；这支持Zircon删除重建前的owned staging Vec，但Zircon动态插件卸载仍需要generation/delta合同。Godot的`ClassDB`在读锁下沿继承链追加已注册`PropertyInfo`，对象层再组合script/extension属性；其优势是类型元数据由注册owner持有，而不是每字段重新解析schema。Zircon editor需要同样让编译后的registry/field identity成为owner-owned artifact，同时保持VM reload的原子代际语义。

## 动态验收

1. current-source reflection Cargo：derived registration、type registry ambiguity/upsert/remove、dynamic/fixed component read-fields/read-write、resource与VM backing全部测试。
2. types/fields/entities为1/100/10k，cold register、upsert/remove、Inspector stable/read/write记录registration builds、registry/field probes、path/String clone bytes、component clone bytes、index rebuild entries与p95。
3. PERF-MVP-457要求derived默认注册build=1/type、dynamic bulk read registry lookup=1/request且field probes近F、short-index rebuild无staging full-path clones；PERF-MVP-331/443/446完成后再要求stable generation registry build/clone=0、dense field access无字符串scan。

动态Cargo、规模counter与F4 Inspector产品trace未完成，因此该目录继续保留在`pending.md`，不进入`review.md`。
