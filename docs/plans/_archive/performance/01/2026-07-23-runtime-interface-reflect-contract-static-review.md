---
related_code:
  - zircon_runtime_interface/src/reflect
  - zircon_runtime_interface/src/tests/reflect_contracts.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/13-script-binding-and-reflection.md
  - docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
reference_sources:
  - dev/bevy/crates/bevy_reflect/src/type_registry.rs
  - dev/godot/editor/inspector/editor_inspector.cpp
tests:
  - zircon_runtime_interface/src/tests/reflect_contracts.rs
  - current-source Windows reflection contract tests pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime interface reflect 性能静态审查（2026-07-23）

## 范围与覆盖

`zircon_runtime_interface/src/reflect/**` 当前源 **15/15** 个 Rust 文件、**1,122** 行已逐文件阅读；同时完整读取 `src/tests/reflect_contracts.rs` **1/1** 个文件、442 行、8 个合同测试。生产反查覆盖 Runtime `WorldReflection`/`TypeRegistry`、Editor workbench snapshot/字段更新/选择恢复与 VM backing。该目录和合同测试当前无工作区改动，本轮未修改源码。

## 性能结论

- `ReflectSchemaResponse` 必须拥有完整 `Vec<ReflectTypeRegistration>`；每个 registration 又递归拥有 type path、display/documentation、字段名/显示名/类型名、枚举选项与默认值。合同没有 schema generation、`NotModified`、cursor/page 或 byte budget，因此同一 registry 的稳定查询仍只能深拷贝完整元数据。Runtime `reflect_schema` 当前直接 `.clone()` registration，列表接口也逐项 clone。
- `ReflectFieldsResponse` 必须拥有全部 `Vec<ReflectFieldValue>`，`ReflectedValue::{String,Enum,Resource,List,Map,Json}` 继续递归拥有正文；请求没有 object generation、field slots、最大字段数、最大字节或最大深度。当前合同测试只验证 JSON shape/roundtrip，没有 stable-generation fast path 或资源上限断言。
- `ZrReflect` 已提供 dense `u32` slot 的读写入口，但公共 `ReflectReadRequest`/`ReflectWriteRequest` 仍只携带 `field_name: String`。进程内 Inspector 因而重复构造 type path/field name，并在每次请求重新解析 schema/name；slot 能力没有贯穿 DTO/Editor consumer。
- Editor `snapshot_with_component_drawers` 对每个选中实体的动态组件先 `reflect_schema()` 深拷贝整份 registration，再 `reflect_fields()` 物化全部 values；随后对每个 schema field 在线性 fields Vec 中 `.find(...)`，形成每组件 **O(F²)** join。`can_edit_dynamic_component_field` 又 clone schema并单字段 read，选择恢复还按 draft field 逐项 read；稳定 snapshot 没有 registry/object generation 门。新增 **PERF-MVP-567**，并与全世界 inspection artifact 的 PERF-MVP-456、脚本 compiled ABI 的331/457分工。
- `ZrReflectValue` 的标量/向量转换为常数成本，浮点有限性检查正确；`Vec<T>` 两向转换均按输入长度预分配。String 转换的 clone 是当前 owned value 合同决定的成本，不应在此文件局部加缓存。
- `ReflectTypeRegistration::with_plugin_id` 在注册冷路径复制一次 plugin id以保持内外字段同步；错误路径才分配上下文 String/`format!`。没有证据表明这些构造器本身是帧级热点，不单独立项。

## PERF-MVP-567 设计

1. Runtime13 发布唯一 `ReflectCatalogGeneration`：稳定 type/field 文本由 interned IDs/共享不可变表拥有，`TypeSlot`/`FieldSlot` 为进程内调用主键；registration change 一次构建有序字段表和 name→slot index。Bevy 的 `TypeRegistry` 以 `TypeId`/静态 `TypeInfo` 借用注册元数据可作为索引与生命周期参考，但 Zircon 仍保留可序列化、插件可变的 generation owner。
2. Editor02/05 保存 `catalog_generation + object/component_generation + selection_generation`，只在任一 generation 变化时取得 affected schema/field delta。workbench snapshot/drawer 编译一次 slot→row 映射，按 schema order 单遍 zip/slot lookup；稳定 snapshot 的 schema clone、全字段 read、String join 与 O(F²) find 均为 0。
3. 跨进程/远程边界才物化版本化 schema/value page，request 显式带 generation hint、cursor、`max_fields`、`max_bytes`、`max_depth`；response 为 `NotModified` 或有界 page，并返回 next cursor/effective limits。旧 owned JSON shape 在一次 hard cut 中迁移所有 Rust caller，不得与 retained generation artifact并存为第二权威。
4. write/readback 仍保持当前 typed error、changed 标志、schema order和有限浮点语义；恶意/过大 List/Map/Json 在分配整树前按统一 transport budget失败或截页，不能只在序列化完成后统计字节。

## 参考引擎对照

- Bevy `TypeRegistry` 用 `TypeIdMap` 和静态 type path/type info建立注册表，查询返回 borrowed registration/type info；这证明进程内反射不必为每个 consumer重新拥有整份 schema。
- Godot Inspector 监听 `property_list_changed` 再刷新 property list，而非把类型元数据无条件绑到每次 UI snapshot。Zircon应采用显式 registry/object generation，不能照搬 Godot动态 List分配，也不能保留当前稳定帧深 clone。

## 动态验收

1. current-source interface/runtime/editor reflection合同、JSON migration、unknown/ambiguous path、component/resource、read/write/readback、finite value与插件 hot-reload测试。
2. types/components/fields 各 1/100/10k，stable 60/120/240 Hz、selection-only、单字段 edit、1% schema reload：记录 registration/value/String/JSON clone bytes、registry/name probes、field comparisons、snapshot builds、lock和 p95。stable schema/full-field build=0；changed join近 O(affected fields)，不再 O(F²)。
3. remote payload 1 B/1 MiB/1 GiB、List/Map depth 1/64/1k、page 1/100/10k fields：记录 decoded/retained/encoded bytes、alloc、drop/reject、cursor和 RSS；所有 hard limits在整树物化前生效，内存有界。
4. F4 Inspector产品 trace验证 dynamic drawer、plugin unload/reload、draft restore、undo/writeback和 pixel/row order parity；Bevy/Godot仅作架构参照，不作为通过证据。

动态门禁、规模 counter、hard-cut API 迁移与 F4产品 trace未完成，因此该目录和合同测试继续保留在 `pending.md`，不进入 `review.md`。
