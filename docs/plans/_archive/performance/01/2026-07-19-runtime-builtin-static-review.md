---
related_code:
  - zircon_runtime/src/builtin
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
reference_sources:
  - dev/bevy/crates/bevy_app/src/plugin.rs
  - dev/godot/core/string/string_name.cpp
  - dev/godot/core/string/string_name.h
tests:
  - zircon_runtime/src/builtin/runtime_modules/tests
  - zircon_runtime/src/builtin/runtime_modules/assembly/registration_inputs/tests.rs
  - current-source Windows Cargo and plugin-scale startup traces pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime builtin逐文件性能静态审查（2026-07-19）

## 范围与覆盖

`zircon_runtime/src/builtin/**`当前源 **27/27** 个Rust文件、**2,532** 行、**23** 条测试已逐文件阅读，覆盖target/profile manifest、availability、plugin/feature registration、extension inputs、core/plugin module assembly、activation sort、load diagnostics与RuntimePluginId。11个文件含其他会话当前改动，本轮保留并只修改成功取得租约的两处独立切片。

## 性能结论

- feature/profile路径会在相邻owner间深clone完整registration/feature reports；report含package manifest和宽extension registry。本轮删除profile wrapper的首次deep clone，target assembly仍保留一次必要owned catalog输入。
- `extension_inputs_from_extension_registries`先collect registry refs，再为asset importer和8类graphics extension多轮扫描并clone descriptors；active feature refs还用available IDs×registrations嵌套查找。它们是bootstrap/editor plugin generation成本，继续回链PERF-MVP-038与Plugins01 `runtime-plugin-catalog-derived-projection-rebuild`，应共享其ordered generation projection。
- `runtime_plugin_descriptors()`每assembly构造builtin catalog；外部当前改动已新增single-availability-projection门禁，继续回链Plugins01 `runtime-profile-availability-rebuild`。module descriptor/order多owner复用PERF-MVP-322，editor重复startup projection复用427。
- **PERF-MVP-436**：未知`RuntimePluginId`通过全局mutex interner永久`Box::leak`，dynamic discovery/reload key churn没有owner或容量。

## 本轮直接止损

1. plugin key normalization改为`Cow<str>`：trim后已规范小写的builtin/alias/external key借用输入，只有ASCII大写才分配lowercase String；interner查询接受`&str`，重复dynamic key不再先建临时String。
2. profile feature assembly收集borrowed registration refs并传给target owner，删除wrapper层对两类宽report的整份深clone。

两项均完成RED→GREEN源码守卫、`rustfmt`与`git diff --check`。受管Cargo仍被validator非JSON入口故障阻止。

## 参考与验收

Bevy plugin build/finish是显式启动生命周期；Godot `StringName`区分static_count与refcount，并在`unref/cleanup`释放动态项。Zircon不复制API，但dynamic plugin symbol必须随catalog generation释放，不能用永久地址充当identity。

对1/100/1k plugins/features/extensions及1/1k/100k reload key churn记录catalog/availability builds、report/descriptor clone bytes、registry passes、feature comparisons、interner lock/entries/string bytes、startup wall与RSS；规范key alloc=0，最终projection每generation≤1且interner entries≤static+active budget。Cargo与F0/F4 plugin toggle/reload trace完成前留在`pending.md`。
