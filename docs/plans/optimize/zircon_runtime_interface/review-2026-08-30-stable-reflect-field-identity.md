# RuntimeInterface 稳定反射字段身份重审（2026-08-30）

## 状态

`field_identity_and_slot_index_source_foundation_implemented /
public_single_and_bulk_dto_dense_route_implemented /
vm_editor_journal_dynamic_scene_v3_stable_id_implemented /
descriptor_only_plugin_explicit_identity_blocked_by_runtime42 /
type_and_variant_identity_pending /
managed_and_product_validation_pending`

对应父计划：`02-serialization-reflection-resource-project-world-sync-public-dto-contract-review.md`
的 P1-43。

## 实现前基线结论

当前 reflection schema、实例读写、dynamic scene 和 VM state 都把 `String field_name` 当身份。
`ZrReflect`/adapter 同时暴露字符串读写与 `u32 field_slot` 读写；Runtime 写入先按名称线性扫描字段，
再把位置转换为 slot。这个 slot 只表示本次 registration 的 vector 位置，字段插入或重排就会改变，
不能作为持久 ID。VM 的 `VmStateFieldRename { from, to }` 仍是局部字符串补丁，scene、script 和
editor state 没有共同 migration authority。

必须分离三种概念：

- stable field identity：进入 scene/script/editor state 和跨进程 DTO 的 128-bit ID；
- current schema name/display name/aliases：只用于 authoring、诊断和显式旧格式迁移；
- dense execution slot：registration admission 后生成的本地连续索引，只在 adapter 热路径使用。

不采用 `hash(current type_path + current field_name)` 作为隐藏 fallback。schema codegen 可以从显式、
可保留的 identity key 生成初始 ID，但字段重命名时必须保留该 key；current name、display name 和
identity key 不是同一 authority。也不把当前 vector position 重新命名成 stable ID。

## Unreal 参考与结构决策

本地 Unreal `dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/UObject/PropertyTag.h`
的 `FPropertyTag` 同时保存 `Name` 和可选 `PropertyGuid`。`Private/UObject/Class.cpp` 的 tagged load
先在支持 GUID 的 Blueprint schema 上调用 `FindPropertyNameFromGuid` 回定位 current name，再调用
`FProperty::FindRedirectedPropertyName` 处理显式 property redirect。顺序一致时沿 `PropertyLink`
线性前进以保持 O(n) bulk load；字段重排时才退回查找。保存路径也只在支持该 identity authority
且非 cooking 时附加 property GUID。

Zircon 不复制 Unreal 的条件编译历史包袱，但采用相同分层：ID 负责持久身份，redirect/alias 是迁移
metadata，dense slot 是 admitted schema 的执行投影。RuntimeInterface `ReflectSchemaCatalog` 是唯一
identity/index owner，Runtime `TypeRegistry` 只保留 adapter projection；derive/build codegen 生成 ID，
adapter 不自行哈希名称或维护第二份映射。

## Current-source implementation

本轮已完成字段身份的公共 DTO、执行投影和主要持久化 consumer 硬切，不把 descriptor/type/variant
身份与未执行的 managed/product gate 误报为闭环：

- RuntimeInterface 新增非 nil、canonical UUID serde 的 `ReflectFieldId`；`ReflectFieldInfo` 强制携带
  `id`，并把 current `name`、`display_name` 与显式 `aliases` 保持为独立 metadata；
- `ZrReflect` 与 `ZirconScriptType` derive 支持 type/field `identity` 和重复 `alias`，显式 identity key
  必须非空且已经 trim；默认 key 只发生在 codegen，运行时没有用 current name 重算 ID 的 fallback；
- neutral catalog 在 publication 前分两阶段验证全部 current name/ID，再验证 aliases，拒绝 current
  name 与 alias 的任意碰撞、跨类型 ID collision，并限制单字段/单类型 alias 数量；
- catalog 是唯一 `ReflectFieldId -> dense u32 slot` owner。字段数 `<=512` 使用按 ID 排序的紧凑数组
  二分，`>512` 使用一次构建的 `HashMap`；register、VM schema replacement、remove 与 clear 同步维护；
- `ReflectReadRequest`/`ReflectWriteRequest` 直接携带 stable ID；`ReflectFieldValue` 携带 ID、当前诊断名
  和值。旧 wire 的 `field_name` request 被 `deny_unknown_fields` 拒绝，不存在名称兼容成功路径；
- `WorldReflection` 单字段 read/write 直接用 catalog ID index 解析 dense slot；`reflect_fields` 按 admitted
  schema 顺序遍历 slot，component/resource 都不再进入 name adapter 或 adapter-owned `read_fields`；
  `ReflectResource` 的 slot read 是强制契约，现有构造点均显式接入；
- VM reflected state 硬切为 `VmStateFieldValue { field_id, value }`，默认 producer schema version 升为
  `VM_STATE_SCHEMA_VERSION_V3`；object/schema 使用 `deny_unknown_fields` 拒绝旧 `field_name` 与 `renames`
  payload，迁移按 `ReflectFieldId` 匹配，current name 改名后只要保留同一 stable key 就无需 rename map；
- editor `SetReflectedSceneFieldCommand` 在 capture 边界把当前 authoring name 解析成 stable ID，journal、
  apply 和 undo 只保存/消费 ID；
- dynamic scene 升为 v3；capture 按 schema 顺序读取 dense slot，v2 importer 从历史 type/name 生成初始
  ID，spawn/resource/plugin JSON preflight 将 ID 编译成 dense slot，并拒绝 unknown/duplicate stable ID；
- final owner 位于 RuntimeInterface `schema_catalog/admission.rs`、`field_index.rs`；Runtime 的旧两文件已
  hard-cut。`type_registry.rs` 为 689 行，catalog owner 为 336/70 行，均低于 800 行 review warning。

尚未完成的硬切包括：descriptor-only plugin 的 `ComponentPropertyDescriptor` 没有显式 field ID，当前
只能从 `(descriptor.type_id, property.name)` 生成临时 ID；该 contract 文件由 Runtime42 活跃拥有，
本轮不越权修改。generated type ID、enum variant ID 与依赖 IR 也仍开放。

本次公共 DTO 硬切已移除单字段请求上的线性 current-name scan；public request 直接进入 stable ID
index，bulk capture/load 按 schema 顺序线性处理 dense slots。下述 production-index lookup 数据可以
代表字段定位内核，但仍不是包含 serde、world access、editor、scene I/O、allocation/RSS/power 的产品
端到端数据，因此不单独宣称完整产品性能闭环。

## 候选查找算法与基线

当前单字段写路径的名称查找是 `O(field_count * compared_name_bytes)`。在 F 盘 release harness 中，
固定同一命中序列、预先构造所有 name/ID/index，并对三种 lookup 交错采样 15 次；下表只计 lookup，
不计输入生成、字符串分配或 registration build：

| fields | probes/sample | linear string P50/P95 | `HashMap<u128, slot>` P50/P95 | sorted ID binary P50/P95 |
| ---: | ---: | ---: | ---: | ---: |
| 16 | 2,000,000 | 69.7 / 103.6 ns | 22.7 / 57.7 ns | 11.2 / 33.6 ns |
| 128 | 500,000 | 549.0 / 1505.4 ns | 22.9 / 31.4 ns | 13.7 / 17.4 ns |
| 256 | 300,000 | 1050.7 / 3142.0 ns | 22.3 / 157.9 ns | 16.2 / 115.9 ns |
| 512 | 200,000 | 2050.7 / 2834.1 ns | 20.5 / 32.1 ns | 17.5 / 27.4 ns |
| 1024 | 100,000 | 4032.0 / 5522.4 ns | 22.0 / 32.1 ns | 23.1 / 288.4 ns |
| 4096 | 20,000 | 17003.4 / 23167.2 ns | 60.6 / 95.8 ns | 100.7 / 3024.6 ns |

完整未裁剪 sample 位于
`F:/codex-targets/019ffe-runtime-interface-field-identity-20260830/field_lookup_candidate_bench.rs`
及其输出。本轮保留 1024/4096 binary P95 的调度离群值，不用删样本美化结论。

固定只用 hash 会为常见的小 schema 支付额外 bucket/control-byte 内存与 hashing；固定只用 binary
在最大 4096 字段 schema 上又落后 hash。实现采用 immutable adaptive index：小 schema 使用按 ID
排序的紧凑数组和 binary search，大 schema 使用一次性构建的 hash index；两者都只返回 dense
slot。阈值先取 512（该点 binary P50/P95 仍优于 hash），必须在生产 `ReflectFieldId`/index 完成后
重新 profile，不能把该候选 harness 当产品性能结论。

复杂度为：registration build `O(n log n)`（小）或 expected `O(n)`（大），单字段 lookup
`O(log n)` 或 expected `O(1)`，adapter access `O(1)`。bulk capture/load 按 schema 顺序直接遍历
dense slots，避免对每个字段重复 lookup。完整 ID 集和 alias 集在 publication 前一次性碰撞检查。

## 分层实现范围

1. RuntimeInterface 增加非 nil、canonical serde 的 `ReflectFieldId`；`ReflectFieldInfo` 强制携带 ID，
   current name/display name/aliases 保持独立 metadata。
2. derive/codegen 从显式 stable identity key 生成 ID，并允许 rename 后保留 key；不提供按 current name
   的运行时 fallback。
3. Runtime registration 原子验证 ID/alias collision，并构建 immutable ID-to-slot index。
4. VM state、public/remote read/write、`ReflectFieldValue`、editor command journal 和 dynamic scene v3
   已 hard-cut 到 field ID；按名称 migration 只存在于显式 v2 importer。
5. adapter 热路径只接 dense slot。schema/current name 只用于 inspection label、诊断和类型校验上下文。

P1-43 的 type ID 与 enum variant ID 不能用 field 局部 hash 冒充：它们要进入同一 generated schema
IR，并与 P1-45 registry fingerprint/collision gate 一起完成。字段切片完成时会明确记录它们仍开放，
不把 P1-43 整体误标完成。

## 验收门

- 非 Cargo：proc-macro token/static gates、scoped rustfmt/diff check、F 盘 production-source identity/index
  harness、至少 21 个独立 lookup/build 样本；
- managed：RuntimeInterface reflect contracts、derive compile/runtime tests、Runtime registry/world/
  dynamic-scene/VM focused tests；
- product：editor inspector、scene save/load、VM hot reload 的 matched workload timing、allocation、RSS
  与 power；必须证明线性名称 scan 从采样热点消失，且 rename round-trip 保持同一 ID。

候选 microbenchmark 不是产品 latency、功耗、Unreal 横向数据，也不能关闭 managed/product gate。

2026-08-30 current-source non-Cargo gate：scoped `rustfmt --check --config skip_children=true` 与
`git diff --check` 均通过；结构门确认 5/5 `ReflectResource` 构造点具有强制 `read_field_by_slot`，并确认
`read_reflected_field` 的 component/resource 两个分支均只调用 dense slot adapter。资源 facade 回归增加
named/slot route counter，预期公开单字段读取为 `(named=0, slot=1)`；该测试因 managed Cargo lane
不可用仍处 pending，不能标记为执行通过。

2026-08-31 VM persistence current-source gate：`VmStateFieldRename`、旧 V2 常量、`renames:` 构造和
`field.field_name` 消费在 Runtime script 与 ZrVM runtime 范围内均为零；13 个 VM source/doc 路径的
scoped rustfmt 与 `git diff --check` 通过。新增回归覆盖 stable ID 跨 current-name rename、旧
`field_name` payload 拒绝、旧 `renames` schema 拒绝、重复 stable ID 拒绝，以及 hot-reload rollback
保持相同 field ID。Runtime 与 ZrVM 两个 managed Windows release gate 已提交前置快照时才可记录为
通过；当前仍处 pending。

2026-08-31 public DTO/editor/dynamic-scene current-source gate：scoped rustfmt、`git diff --check` 和
production consumer scan 通过；源码扫描确认 production `.read_fields(` / `read_fields:` 为零，所有
public read/write 构造均使用 `ReflectFieldId`，dynamic-scene v3 migration 回归覆盖 component/resource
初始 ID，spawn 回归覆盖重复 ID 拒绝。协调器快照 `2422`，source manifest
`fc579de87232fdc876330d0e31e270ba304bb7763bdd61931a960d7ef03aedb8`；四组 Windows release
managed tickets 已排队：RuntimeInterface `3701010c374d4a6281ec42715ea4488a`、Runtime ECS reflection
`b28e2941ebad4b03b46254834cdbaa15`、Runtime dynamic scene
`c2ab992d9af84cfc8f016f3c41322630`、Editor reflected command
`efe405cfaae046b680b8cfb15e41cf1f`。排队不等于通过，managed/product 状态保持 pending。

## Production-index source evidence

F 盘 `field_slot_index_source_gate.rs` 直接 `#[path]` 引入当前生产
`zircon_runtime_interface/src/reflect/schema_catalog/field_index.rs`，以同尺寸 128-bit mock contract 隔离 Cargo
阻塞；0/1/16/512/513/4096 字段的全量已登记 lookup、未知 ID 与阈值切换均通过。该 gate 验证的是
生产索引算法源码，不是完整 RuntimeInterface ABI。

同目录 `field_slot_index_production_source_bench.rs` 对生产索引源码和旧线性字符串 scan 做 15 轮
交错 release 采样，构建、输入生成和分配均在计时区外。单位为 ns/probe：

| fields | probes/sample | linear P50/P95 | production index P50/P95 | speedup P50/P95 |
| ---: | ---: | ---: | ---: | ---: |
| 16 | 2,000,000 | 83.61 / 196.40 | 13.19 / 24.63 | 6.3x / 8.0x |
| 128 | 500,000 | 600.83 / 1365.23 | 18.55 / 34.50 | 32.4x / 39.6x |
| 256 | 300,000 | 1134.21 / 2647.55 | 20.73 / 33.81 | 54.7x / 78.3x |
| 512 | 200,000 | 3034.09 / 5290.51 | 23.60 / 39.58 | 128.6x / 133.7x |
| 1024 | 100,000 | 5536.63 / 13217.62 | 38.52 / 76.11 | 143.7x / 173.7x |
| 4096 | 20,000 | 20721.64 / 24393.92 | 72.44 / 195.53 | 286.0x / 124.8x |

完整未裁剪样本由 harness 输出保留；表中的 P95 与调度离群没有从样本中删除。原始六行输出保存在同目录
`field_slot_index_production_source_bench-output.txt`。
由于 harness 用等宽 mock ID 隔离接口依赖，它仍不是完整 `ReflectFieldId` 产品 profile。managed ignored
benchmark、editor/scene/VM matched workload、allocation/RSS/power 与 Unreal 经验值比较继续 pending。
