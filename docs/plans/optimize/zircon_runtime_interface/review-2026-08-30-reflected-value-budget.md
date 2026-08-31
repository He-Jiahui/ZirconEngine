# RuntimeInterface `ReflectedValue` 预算与准入重审（2026-08-30）

## 状态

`source_implemented / static_and_f_drive_verified / managed_and_product_validation_pending`

对应父计划：`02-serialization-reflection-resource-project-world-sync-public-dto-contract-review.md`
的 P1-42。

## 实现前基线结论

`ReflectedValue` 同时包含递归 `List`、`Map` 和可再次递归的任意 `serde_json::Value`，当前只有
`type_name()`，没有节点、深度、字符串、单容器或 finite 准入。`ReflectFieldInfo::default_value`、
editor/remote read/write DTO、dynamic scene 和 VM state 都能携带该树。

准入不能散落到 adapter：

- schema default 由 Runtime `TypeRegistry` 在 publication 前原子准入；
- editor/remote 实例读写由 `WorldReflection` 在返回或 mutation 前准入；
- world-query inspection 直接读取 adapter，必须在自己的 wire/time preflight 前复用同一准入；
- dynamic scene capture/spawn 绕过 `WorldReflection`，在 capture/compiled preflight 准入；
- VM reflected snapshot 绕过 World，在 `validate_reflected_objects` 准入。

RuntimeHost foreign JSON decode 已先做 encoded-byte、JSON value count 和 nesting-depth preflight；本项
不改写该 foreign owner。VM lifecycle JSON 仍需独立的 envelope byte admission，但它不是放宽
`ReflectedValue` 预算的理由。

## 参考与结构决策

本地 Unreal 源码 `dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Serialization/Archive.h`
让 archive/caller 持有 `ArMaxSerializeSize` 和错误状态；
`dev/UnrealEngine/Engine/Source/Runtime/Json/Public/Serialization/JsonSerializer.h` 的读取使用显式
`ScopeStack`，写入使用显式 `ElementStack`。可复用的是“中立表示与调用方 policy 分离、遍历不依赖
无界调用栈”，不是复制某个 Unreal 阈值。

仓库内 binary value 与 `UiBindingValueBudget` 已采用 depth/node/string/container/finiteness 五个单位。
反射值沿用相同单位，避免第三套计数语义；具体 Runtime 上限由 Runtime owner 定义，接口 DTO 只提供
`ReflectValueBudget` 和纯验证器。

统一计数语义：

- 根 `ReflectedValue` 深度为 1；嵌入 JSON root 是 `Json` wrapper 的子节点；
- 每个 `ReflectedValue` 和每个 JSON value 各计一个 node；
- `String`、`Enum`、`Resource`、Map key、JSON string 和 JSON object key 都计入累计 UTF-8 bytes；
- 每个 List/Map/JSON array/object 分别执行单容器 entry 限制；
- Scalar/Vec2/Vec3/Vec4/Quaternion 的每个分量必须 finite；
- 所有累计与 depth increment 使用 checked arithmetic，失败关闭，不用 saturation 掩盖输入。

## 算法评估

验证所有标量和 key 的时间下界是 `O(nodes + string bytes)`。候选算法在 F 盘 release `rustc`
独立进程中用 1,365-node、4-wide、5-deep 树做 21 组交错样本，每组 10,000 次：

| candidate | P50 | P95 | auxiliary memory |
| --- | ---: | ---: | --- |
| checked recursion | 7.30 us | 16.40 us | call stack `O(depth)` |
| iterator-frame stack | 14.62 us | 24.47 us | heap `O(depth)` |
| flat work stack | 7.13 us | 16.92 us | heap `O(pending width)` |

iterator-frame 在该布局约慢 2 倍，故不采用。递归与 flat work stack 的分位数相近，但 public caller
可以构造不同 depth policy；为使验证器自身不依赖调用栈，选择 flat work stack。它先拒绝超大
container，再把 child reference 入栈，因此辅助内存受单容器和 node budget 共同约束；遍历一次、
不 clone value/string，复杂度为 `O(nodes + string bytes)` time、`O(min(nodes, pending width))` memory。

这些是候选遍历微基准，不是 editor/remote product latency、RSS、功耗或 Unreal 横向数据；不得据此
声称产品瓶颈或功耗已闭环。实现后必须用生产 validator 重跑 F 盘至少 21 个样本，并保留 managed
Cargo、产品 workload、RSS/power 作为独立验收项。

## MVP 实现范围

1. 在 RuntimeInterface 增加 caller-owned `ReflectValueBudget`、结构化
   `ReflectValueValidationError` 和 `ReflectedValue::validate_with_budget`。
2. Runtime 定义一个明确的 reflection boundary policy；所有 owner 复用它，不各自复制数字。
3. schema default、World read/fields/write、dynamic scene capture/spawn 和 VM reflected object 在
   mutation/publication/serialization 前 fail closed。
4. 增加 budget/finiteness、读取 adapter 输出、写入 mutation 前拒绝、schema 原子拒绝和 VM bypass
   回归。

本里程碑只拒绝超大 inline value。paged/bulk handle 需要明确 transport owner、lifetime、revision、
cancel/backpressure 与权限模型，不能在 P1-42 中临时伪造第二套 blob identity；该能力保留为后续
独立架构项。

## 当前实现与量化

- RuntimeInterface 新增 `value_budget.rs` 与 `value_validation.rs`；flat work stack 同时遍历 tagged
  value 和 embedded JSON，没有 value/string clone，真实累计使用 checked arithmetic。
- Runtime 单一 policy 为 depth 128、nodes 16,384、累计 string bytes 1 MiB、单容器 4,096。
- `TypeRegistry` default、`WorldReflection` read/fields/write、world-query inspection、dynamic JSON
  component admission、dynamic-scene capture/spawn、reflected JSON read/write、VM reflected object 与
  schema default 已接入；旧递归 `reflected_value_is_finite` owner 删除。
- F 盘生产文件 include harness 通过 3/3（混合图、129-depth 拒绝、NaN component 定位）。
- 生产 validator 用 128-entry mixed fixture（1,153 nodes、6,784 string bytes、depth 6）运行 21 个
  独立 release 进程，每进程 10,000 次：P50 `37.3 us/validation`、P95 `54.2 us/validation`，范围
  `25.7..113.9 us`，P50 约 `32.3 ns/node`。范围保留单个明显调度离群值，不裁剪样本。
- scoped rustfmt 与 diff-check 通过；managed Cargo 尚未运行。

单值 policy 不能限制一个含数千字段的完整 DTO 总量；RuntimeHost world-query 已有 outer wire
byte/item/depth/time owner，VM lifecycle envelope 与其它直接 DTO 仍需各自 outer admission。paged/bulk
handle 同样未在本项伪造。

## 验收门

- 非 Cargo：scoped rustfmt、diff check、legacy/source gates、F 盘生产源码 harness 与 21 样本；
- managed：RuntimeInterface reflect contracts、Runtime ECS reflection/dynamic-scene/VM focused tests；
- 产品：editor inspector、world query、scene load、VM migration 的 matched workload timing、allocation、
  RSS 与 power；没有这些数据不得宣称达到其它引擎经验值。
