# RuntimeInterface Reflection schema catalog 与 redirect authority 重审（2026-08-30）

## 状态

`catalog_and_runtime_projection_source_foundation_implemented / generated_dependency_pending /
persistence_cutover_pending /
managed_and_product_validation_pending`

对应父计划
`02-serialization-reflection-resource-project-world-sync-public-dto-contract-review.md` 的 P1-45，
也是 P1-43 dynamic-scene、editor state 与 VM state 从字段名硬切到稳定字段 ID 的结构前置。

## 当前结构性问题

`TypeRegistry` 已经拥有 runtime adapter、full/short type-path lookup、ambiguous short-name 集合、
field ID 到 dense slot 的索引和 catalog generation；RuntimeInterface 的 `ReflectSchemaResponse` 仅返回
一组 registration。dynamic-scene v0/v1/v2 migration 是不接收 registry 的 `serde_json::Value` 版本链，
VM state v2 又独立维护 `VmStateFieldRename { from, to }`。因此现在至少有三条 schema/migration
authority：

- Runtime registration admission 能识别当前 field ID/name/alias，但没有可发布的 catalog snapshot；
- dynamic-scene importer 只能看旧 JSON 名称，无法合法得到当前 stable field ID；
- VM plugin migration 按字符串构造独立 rename map，不能证明它与 runtime/editor schema 相同。

直接把 `ReflectFieldValue.field_name` 改成 ID 会使旧 scene/VM state 无法迁移；在 importer 中用
`hash(current type_path + current field_name)` 补 ID 则把可变名称重新变成身份，违反 P1-43。保留两套
长期字符串兼容路径也会让 editor、scene、script 的行为继续分叉。

## Unreal 参考与采用的边界

本地 Unreal
`dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/UObject/PropertyTag.h` 的
`FPropertyTag` 同时携带 property name 与可选 `PropertyGuid`。CoreUObject
`Private/UObject/Class.cpp` 的 tagged load 在 GUID 可用时先调用 `FindPropertyNameFromGuid` 回定位
当前 property，再对旧名称调用 `FProperty::FindRedirectedPropertyName`，最后才查找并反序列化字段。
保存路径使用同一 GUID authority，而不是从当前显示名临时推导身份。

`Private/UObject/CoreRedirects.cpp` 对 redirect chain 设置最大迭代次数，检测冲突 value redirect，
并把 property redirect 限定在 owner/type context；这说明 redirect 是受 admission 和循环上限约束的
迁移数据，不是普通查询的无限 fallback。

Zircon 采用相同分层但保持 Rust crate 边界：

1. RuntimeInterface 拥有中立 schema catalog contract、稳定 field identity index、legacy alias admission、
   dependency closure 和 registration-set fingerprint；
2. Runtime `TypeRegistry` 继续唯一拥有 ECS/resource/VM adapter，只消费 catalog 的 dense declaration slot；
3. editor/script/scene 只消费同一已发布 snapshot，不重建局部 name map；
4. current name 只用于 authoring/diagnostic，alias 只允许 legacy importer 使用，正常读写只接受 stable ID；
5. redirect 不允许 wildcard、隐式 current-name hash、跨 owner 搜索或无限链。

## Catalog IR 与 admission 方案

catalog entry 由一个 `ReflectTypeRegistration` 和显式 dependent full type paths 组成。catalog publication
按 full type path 排序并一次性建立：

- full path -> entry；
- short path -> unique full path，以及显式 ambiguous short-name 集；
- `(type path, field ID)` -> declaration slot；
- `(type path, legacy current name/alias)` -> field ID，仅供 importer；
- global field ID owner，用于拒绝跨类型 ID collision；
- dependency graph，用于拒绝 duplicate/self/missing dependency 并形成确定性闭包顺序；
- domain-separated BLAKE3 registration-set fingerprint。

fingerprint 输入必须使用带长度前缀的 canonical fields，而不是 `Debug`、map iteration order 或当前内存
布局；至少包含 catalog algorithm version、sorted full type paths、role/serialization/visibility、sorted
dependency paths、field ID/current name/aliases/value type 和会改变序列化形状的 metadata。display-only
文本是否进入 fingerprint 要显式版本化，不能随 serde field 顺序隐式变化。

第一阶段只建立 neutral catalog、identity/alias/dependency admission、snapshot 与 runtime registry 消费点；
第二阶段才升级 dynamic-scene/VM wire version，将旧名称通过该 catalog 的 scoped legacy resolver 一次性
转换为 ID，并删除 `VmStateFieldRename`。插件动态类型必须先发布 plugin schema entry，随后才允许加载其
持久对象；缺失 dependency 要 fail closed，不能退回全局短名称。

## Current-source implementation

本轮已完成第一阶段源码基础，但不把 wire/persistence migration 误报为完成：

- RuntimeInterface 新增 `ReflectSchemaCatalogEntry`、`ReflectSchemaCatalog`、immutable snapshot 与
  canonical hex `ReflectSchemaFingerprint`。batch publication 与 incremental insert 都拒绝 duplicate
  full path、全局 field ID collision、name/alias collision、缺失/self/duplicate dependency 和 cycle；
- field name/alias、type/dependency 数量均有 admission 上限。alias 和 dependency set canonical sort；
  short path 保留 unique map 与显式 ambiguous set，不选择注册先后 winner；
- fingerprint 使用 domain-separated BLAKE3、显式 algorithm version、长度前缀和逐字段 enum tag。
  `ReflectedValue`/JSON 默认值使用显式 work stack 哈希，JSON object key 排序，不依赖递归调用、
  `Debug` 或 map iteration order；snapshot decode 必须重新 admission 并比对 fingerprint、ambiguous
  projection 与 dependency order；
- fingerprint 与 dependency topological order 只在 snapshot/查询时 lazy materialize；incremental
  registration 不重复哈希全 catalog。replace 先构造完整 candidate 再原子替换，remove 拒绝 live
  dependents；
- Runtime `TypeRegistry` 已删除本地 `field_identity_admission.rs` 和 `field_slot_index.rs`，不再保存第二份
  short/ambiguous/field-slot map。catalog 负责 metadata、解析与 stable-ID slot；Runtime map 只保留
  adapter projection，并在 publication 后使用 catalog canonical registration；
- `WorldReflection` schema listing 从 catalog snapshot 投影，`ReflectSchemaResponse` 现在携带 catalog
  algorithm version 与全 registration-set fingerprint；筛选分页可以判断是否属于同一 catalog；
- dynamic-scene descriptor transaction 用当前 catalog clone 做 staging admission，因此既有类型与批内
  stable ID collision 都在不可失败 publication 之前返回 typed error。

源码被拆为 `mod.rs` 400 行、`admission.rs` 336 行、`fingerprint.rs` 369 行、`field_index.rs`
70 行和 `entry.rs` 26 行；Runtime `type_registry.rs` 为 689 行，均低于 800 行 review warning。

F 盘 `schema_catalog_admission_source_gate.rs` 直接 include production `admission.rs` 与
`field_index.rs`，canonical alias、ambiguous short path、dependency order、全局 ID collision、缺失
dependency 和 dependency cycle 均通过。第一次 source compile 捕获了 alias 引用集合存活期间排序的
借用错误；修复后 gate 通过。该 gate 使用最小 DTO shell 隔离 Cargo 阻塞，不是完整 serde/ABI gate。

Frameworks01 受管 Resource profile job `28eb6b1ee6a649e79a8cac8c19dc5c21` / run
`071e0c99214e4abd965e52a0ebf9bfda` 编译的是修复前 snapshot，在 `admission.rs:188,199`
命中同一个 E0502（stderr SHA-256
`8ebdbf17fdc749490e2ce382d75d4d6c3dfaa5e38671a58bccf9e4fad545e0c4`）。当前 blob SHA-256
`18d866b7ecbad235a8c83d34fea59d6a28ccc10f3275e0f5e462c90e0abb2ba7` 已改成第二次 immutable
alias validation、显式 `drop(field_names)` 后才排序，并被上述 direct-source gate 编译运行通过。
因此旧 run 是有定位价值的 RED，不是 current-source GREEN；受管 current-source focused gate 仍 pending。

尚未完成：derive/script codegen 还没有生成 catalog dependency edges；`ReflectFieldValue`、read/write
request、dynamic-scene 与 VM state 仍使用 field name；旧 scene/VM importer 尚未接 scoped alias
resolver；Runtime adapter wrapper 仍保留 catalog registration 的 canonical copy。完整 generated IR、
persistence hard cut、managed Windows 和 matched product profile 继续 pending。

## 算法与性能边界

设类型数为 `T`、字段总数为 `F`、alias 总数为 `A`、dependency edge 为 `D`：

- publication admission：`O(T log T + (F + A) log F + D log T)`，只在 startup/plugin generation
  切换等冷路径执行；
- cycle/closure validation：三色 DFS 或 Kahn traversal，`O(T + D)`，使用显式 work stack 避免深递归；
- stable ID lookup：沿 P1-43 adaptive index，小 schema `O(log fields)`，大 schema expected `O(1)`；
- legacy alias lookup：只在旧资产 migration，使用 per-type immutable map，expected `O(1)`；
- fingerprint：snapshot 发布时 `O(canonical schema bytes)`，普通 read/write 不重新计算；
- bulk serialize/load：按已发布 declaration slot 顺序 `O(F)`，不为每个字段重复做 name/alias lookup。

catalog registration 是冷路径，当前主要性能风险不是单次 map lookup，而是多 owner 重复建表、每次请求
重复 canonicalize、以及 migration 在对象循环内做字符串 rename。实现后 profile 必须分别统计 catalog
build、snapshot/fingerprint、stable lookup、legacy migration 和 bulk load；microbenchmark 不能替代
editor/scene/VM matched workload 的 allocation、RSS、CPU time 与 power 数据。

## 不实施的局部修补

- 不在 dynamic-scene v2 importer 中按 current name 生成稳定 ID；
- 不给 `ReflectFieldValue` 同时长期保留 `field_name` 与 `field_id` 双 authority；
- 不让 VM schema、editor inspector 或 plugin host 各自复制 alias/redirect map；
- 不把 Runtime adapter、ECS storage 或 function pointer 放进 RuntimeInterface catalog；
- 不用 wildcard match、静默丢字段或未知 dependency 容错来伪造旧资产兼容。

## 验收门

- source/static：catalog owner 唯一性、current name 不参与 stable ID fallback、所有 redirect 有 owner scope、
  duplicate/collision/ambiguous/dependency-cycle fixtures；
- managed Windows：RuntimeInterface catalog contract、Runtime registry publication/replacement/remove、
  derive/script registration 和 dynamic-scene/VM focused tests，全部 `--locked`；
- product：至少 15 组交错样本，报告 catalog build/fingerprint、scene save/load、VM hot reload 的 P50/P95、
  allocation/RSS/CPU，并验证名称线性扫描与对象内 rename map 构建从热点消失；
- migration：current scene/VM wire 只写 stable ID，旧 fixture 通过同一 catalog alias resolver 升级，未知、
  冲突、循环或缺失 plugin schema 均返回 typed failure。

在 managed/product gate 完成前，本记录和 P1-45 均不得标记 accepted；候选 source harness 也不构成
功耗或 Unreal 横向性能结论。
