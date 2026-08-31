---
doc_type: implementation-design-and-performance-plan
status: proposed
implementation_status: pending
validation_status: static-research-complete; coordinator-baseline-pending
source_recheck_required: true
owners:
  - Runtime22: random authority, stream-state contract, replay identity
  - Runtime08F: behavior-tree compiled selector table and AI consumer migration
related_code:
  - zircon_runtime/src/core/framework/random
  - zircon_runtime/src/core/runtime/random
  - zircon_runtime/src/core/runtime/handle/random.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/compile.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/executor.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/executor/support.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/executor/selector.rs
  - zircon_plugins/ai/runtime/src/manager/tick.rs
  - zircon_runtime/src/core/framework/ai/tick.rs
  - zircon_runtime/src/core/framework/ai/snapshot.rs
references:
  - docs/plans/optimize/zircon_runtime/22-time-clock-domain-fixed-step-determinism-rng-replay-scheduling-review.md
  - docs/plans/optimize/zircon_runtime/08f-ai-behavior-tree-blackboard-perception-runtime-review.md
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Math/RandomStream.h
---

# Runtime22 / Runtime08F · Random Authority 到行为树消费端的硬切迁移计划

## 状态

| 项目 | 状态 | 当前证据 |
|---|---|---|
| Runtime 随机权威 | 已存在，尚未端到端接入 | `RandomService` 使用版本化 PCG32、master-seed generation、稳定 key 派生、`RandomState` snapshot 与 draw index。 |
| AI `RandomSelector` 消费 | 未迁移 | `weighted_random_child` 仍以 `DefaultHasher(tree_id, node_id, tick)` 生成样本；它不读取 `RandomService`，不保存状态。 |
| AI 随机 owner identity | 缺失 | 执行上下文只有 `WorldHandle` 和裸 `EntityId`；`EntityId` 当前是 `u64`，没有 generation。 |
| 回放/存档状态 | 不完整 | `AiRuntimeSnapshot` 是运行观测快照，未携带每个随机 selector 的 `RandomState`。 |
| 性能基线 | 未执行 | 现有 ignored benchmark 只比较参数 key 字符串分配；不能量化完整 selector 的算法复杂度、分配或功耗。 |

本文件是调研后的实施合同，不是完成记录。运行时随机底座、AI 选择器和当前相关源码均有在途修改，实施前必须由协调器重新确认源码快照和基线结果。

## 已核实的问题

`RandomService::stream(RandomStreamKey)` 已把 algorithm ID、master seed、seed generation、world/entity/system/purpose 和 authoring seed 纳入 BLAKE3 派生；`RandomStream` 已保存 PCG state、increment 和 draw index，并用 rejection sampling 提供无偏的有界 `u32`。

AI 的 `weighted_random_child` 绕开这条链：

1. 对 tree ID、node ID 和本地 `tick` 使用标准库 `DefaultHasher`。该算法既不是跨 BuildSet 格式合同，也不包含 world、entity、master seed 或 seed generation。
2. 同 tree/node/tick 的不同 agent 使用同一个哈希输入，因此会作出同一选择；world 重建和 entity ID 重用也没有隔离边界。
3. 每次选择都会建立 `Vec<f32>`，并且对每个 child 用 `borrowed_child_weight` 线性扫描 node parameter slice。权重参数与 child 数量同阶时，查权重的渐进复杂度是 `O(children * parameters)`，之后还有一次求和和一次线性选择。
4. `selected_child` 只保存当前分支，不保存随机流。分支终止后重新选择依赖递增 `tick`，而 `tick` 使用 wrapping add，既不是 snapshot state，也不是 simulation tick identity。

这不是局部 hash 替换问题。直接将 `DefaultHasher` 换成 PCG，仍会保留身份碰撞、无法恢复的选择历史、线性热点和未版本化权重语义。

## 目标架构

```
CoreRuntime RandomServiceState
        |
        | typed seed-authority / world-identity input
        v
AiBehaviorRandomContextV1
        |
        +--> compiled RandomSelectorTableV1
        |       owner_id, total_weight, prefix_weights
        |
        v
BehaviorNodeRuntimeState.random_state
        |
        v
RandomStream::try_next_bounded_u32(total_weight)
        |
        v
binary search prefix_weights -> selected child
```

### 1. 权威与身份

Runtime 继续唯一拥有 master seed 和 reseed generation。AI manager 不保存可独立 reseed 的随机服务，也不以 wall clock、frame order、指针或 `DefaultHasher` 作为 seed 来源。

`AiBehaviorRandomContextV1` 必须由 runtime tick adapter 传入，并至少携带：

- `RandomServiceState`，或一个只允许派生而不允许 reseed 的 runtime-issued issuer；
- `RandomWorldKey(id, generation)`；
- `RandomEntityKey(id, generation)`；
- 与 `SimulationTickId` 关联的诊断/replay identity，但它不得成为 stream key 的替代品。

现有 `WorldHandle(u64)` 和 `EntityId = u64` 不满足 generation 合同。因此 Runtime22 先定义稳定 generation 的来源和生命周期，再由 Runtime08F 消费；不得把 `0` 伪装成 entity generation。

每个 selector 的 owner identity 使用显式版本化的 `AiBehaviorRandomOwnerIdV1`。它必须由 canonical tree asset identity、node identity 和 authoring seed 产生；若采用 digest，domain tag、输入字节序、输出截断和算法版本都进入源代码与 golden vector。不得重新使用无版本标准库 hash。该 owner identity 进入 `RandomStreamKey` 的 stable purpose/authoring-seed 分量，从而使一个 agent 的多个 selector 不共享流。

### 2. 状态与回放

首次运行 selector 时，AI 从 runtime-issued context 派生 `RandomStream`；随后把 `RandomState` 放进对应 `BehaviorNodeRuntimeState`。每一次实际选择先推进流、再记录 `selected_child`，并在分支终止后保留流状态以供下一次选择。tree asset、world/entity generation 或 seed generation 改变时，该 node 的随机状态必须作为显式 lifecycle transition 清除，并输出可观测 receipt。

AI durable/replay snapshot 需要增加 versioned selector-state payload，至少含 `RandomState`、owner identity 和 compiled-table version。debug `AiRuntimeSnapshot` 不得被误用为恢复格式。restore 对不支持的 algorithm、table version、world/entity generation 或 asset identity 要 fail closed，而不是静默重新播种。

### 3. 权重编译与运行期复杂度

`RandomSelectorTableV1` 属于 `CompiledBehaviorNode`，而不是 executor scratch：

- 编译期单次扫描 parameters 与 children，解析 `weight.<child-id>` 和 canonical `weight_<index>`，并保留现有 ID-key 优先于 position-key 的语义；
- 验证 weight 为有限、非负值，定义 V1 的显式量化策略和总量上限；非法值、量化溢出与 total=0 都返回 typed compile error；
- 生成不含浮点累计的 `u32` cumulative prefix table 与 total weight。运行期通过既有无偏 `try_next_bounded_u32(total)` 取得 sample，再二分查找 prefix table；
- `total=0` 的 fallback 必须是表格式的显式 policy，而不是依赖浮点 epsilon。若保留“第一个 child”兼容语义，必须写入 V1 policy 与 golden cases。

这样编译成本为 `O(children + parameters)`，稳态选择为一次随机抽样加 `O(log children)` 查表，且不分配 `Vec`、不格式化字符串、不扫描 parameter slice。选择权重的量化会改变历史 float 边界选择，故它是 BuildSet/asset schema 的硬切，不保留双算法隐式 fallback。

### 4. Runtime 与 Plugin 边界

`RandomAlgorithmId`、`RandomState`、stable key value objects 和 replay wire schema 保持在 `core::framework::random`；PCG execution 和 master-seed authority 保持在 `core::runtime::random`。AI 只依赖 framework contract 与 runtime-issued context，不持有 `CoreRuntime` 或通过 plugin registration 偷取长期引用。

AI-specific owner derivation、compiled table、weight validation和 selector-state lifecycle 保持在 `zircon_plugins/ai/runtime`。AI API 版本改变时通过 framework AI contract 显式硬切，更新所有 request builders、plugin adapters、tests、debug schema 和 asset compiler；不得留下 `Option<AiBehaviorRandomContextV1>` 的“旧路径仍可运行”兼容分支。

## 实施顺序

1. **Runtime22 identity 前置条件**：提供 world/entity generation authority、`AiBehaviorRandomContextV1` 输入合同、wire version 和 fixed-step/replay tick association。先完成 authority contract 的 unit/golden tests。
2. **基线采集**：在锁定 revision 上运行完整 selector benchmark，而不是现有 key-allocation microbenchmark；记录 allocation、p50/p95、CPU samples、功耗/频率与输入规模。
3. **Runtime08F 编译表**：实现 `RandomSelectorTableV1`、固定量化、error surface和 deterministic owner vectors；先保持 executor 不消费它，比较新旧 table 的参数解释与明确版本差异。
4. **状态硬切**：把 `RandomState` 纳入 node runtime durable state，tick adapter 强制提供随机 context；删除 `tick` hash path、`DefaultHasher` imports 及其旧 golden helpers。
5. **回放与恢复**：实现保存、restore、world/entity reuse、asset reload、reseed generation transition、selector failure/retry 的 end-to-end traces。
6. **性能与功耗复测**：用相同固定输入、CPU affinity/power profile和 release artifact重跑；仅当 baseline 和复测均由协调器留档后，才将本计划状态改为 completed 或 accepted。

## 基线与验收矩阵

| 场景 | 必须证明 | 指标 |
|---|---|---|
| 1, 8, 64, 1,024 children；每 child 一项 weight | 编译后查表等价于 V1 权重合同 | compile time、table bytes、typed reject cases |
| 1,024 children / 1,024 parameters，反复重选 | 热路径不分配、不格式化、不扫描参数 | allocations/selection = 0；复杂度 trace 为 `O(log N)` |
| 两个 world、相同 entity ID、多个 selector | identity 不碰撞，world/entity generation 可隔离 | stream snapshot and selection vectors |
| snapshot -> restore -> next selection | 恢复产生完全相同的下一次选择和 draw index | state/selection golden digest |
| reseed、asset reload、world/entity reuse | 明确 reset/reject receipt，无静默重播种 | lifecycle trace |
| 60 Hz 和 fixed-step replay、失败后重试 | 无执行顺序依赖，effect trace 一致 | tick/selection/effect digest |

性能测试必须使用 release artifact、固定 CPU governor/电源计划和隔离负载。报告 p50、p95、allocation、CPU sample 和 package power；未获得同机器的前后数据时，不得声称功耗接近 Unreal 或其他引擎。现阶段 Cargo baseline 由协调器执行，本会话不产生未验证的性能数字。

## 非目标与禁止项

- 不将 render/particle 的 frame-random 或 GPU hash 临时塞入 AI stream；它们是 Runtime22 的独立 consumer migration。
- 不以 `tick`、wall time 或 frame number作为随机 stream 的身份。
- 不新增全局 mutable RNG、thread-local RNG 或跨插件共享的无 owner stream。
- 不把 `RandomService` 复制成每个 AI manager 都能自行 reseed 的服务。
- 不用 benchmark 的字符串分配改进替代完整 selector 的算法基线。

## 完成判定

本计划只有在 Runtime22 的身份/authority contract 与 Runtime08F 的编译表、状态持久化、消费者硬切、回放测试和受控性能复测全部通过后才能标记 completed。当前仅完成静态调研与实施设计；没有 Cargo 验证、协调器验收、提交或企微通知。
