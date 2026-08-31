---
title: Runtime Random Authority、Stream、Checkpoint、Replay、Consumer 与 Performance 当前工作树工程化差距
category: zircon_runtime
report_id: Runtime210
review_date: 2026-08-31
baseline_head: working-tree
observed_head: f31fd06f69fdaedb70a0a56fe6d0268de1af83a6
doc_type: current-working-tree-review-and-refactor-plan
review_status: review_complete
implementation_status: compile_blocked_product_incomplete
source_recheck_required: true
tooling_scope: excluded_by_user_request
coordination_tracking: skipped_by_user_request
supersedes_currentness_of:
  - docs/plans/optimize/zircon_runtime/102-runtime-random-authority-stream-checkpoint-replay-consumer-performance-current-source-review.md
related_reports:
  - docs/plans/optimize/zircon_runtime/209-runtime-support-crates-contracts-math-resource-rhi-wgpu-workspace-boundary-device-lifecycle-product-integration-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/22-time-clock-domain-fixed-step-determinism-rng-replay-scheduling-review.md
  - docs/plans/optimize/zircon_runtime/08f-ai-behavior-tree-blackboard-perception-runtime-review.md
  - docs/plans/optimize/zircon_runtime/26-particle-vfx-system-emitter-cpu-gpu-simulation-rendering-scalability-determinism-product-integration-review.md
related_code:
  - Cargo.toml
  - zircon_runtime/Cargo.toml
  - zircon_runtime/crates/zr_contracts/src/random
  - zircon_runtime/src/core/framework/random
  - zircon_runtime/src/core/runtime/random
  - zircon_runtime/src/core/runtime/handle/random.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_plugins/ai/runtime/src/behavior_tree
  - zircon_plugins/particles/runtime/src
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Math/RandomStream.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/DemoNetDriver.h
  - dev/godot/core/math/random_pcg.h
  - dev/godot/core/math/random_number_generator.h
  - dev/godot/tests/core/math/test_random_number_generator.cpp
  - dev/bevy/crates/bevy_math/src/sampling/shape_sampling.rs
  - dev/bevy/crates/bevy_math/src/sampling/mesh_sampling.rs
  - dev/Fyrox/fyrox-impl/src/scene/particle_system/mod.rs
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Runtime/Utilities/Playables/VisualEffectControl/VisualEffectControlTrackMixerBehaviour.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/ShaderLibrary/Random.hlsl
---

# Runtime Random Authority、Stream、Checkpoint、Replay、Consumer 与 Performance 当前工作树工程化差距

## 1. 结论

当前 random slice 的设计方向比临时 `rand()` 封装严格：`RandomAlgorithmId::Pcg32XshRrV1`、BLAKE3 stable-key 派生、world/entity generation、system/purpose/authoring seed、PCG state/increment/draw index、无偏 bounded draw、同 key 唯一 mutable lease、65,536 retained-stream 上限、canonical key order、reseed generation，以及 registry/seed 单一锁序都是真实源码。2026-08-29 已关闭的 mixed-era checkpoint 竞态不重新打开。

但 2026-08-31 当前工作树不是可接纳的工程级实现：

- checkpoint format 已从 v1 提升到 v2，并为每个 stream 增加 `master_seed_generation`；然而 `service_checkpoint.rs` 返回 `RandomServiceCheckpointError::StreamAuthorityGenerationMismatch`，`checkpoint_error.rs` 并未声明该 variant。这是当前源码级编译断口，不是缺测试或集成不完整。
- 103 个聚焦输入虽已全部进入 index，但仅 63 个存在于 HEAD，70 个处于 dirty 状态；根 workspace/runtime manifest 仍没有 `zr_contracts` member/dependency。clean checkout 无法重建当前 random slice。
- `RandomStreamLease::Drop` 仍无条件提交 draw progress，完全不知道 fixed-step begin/commit/abort；失败后重试同一 tick 会从下一随机数继续。
- `RandomServiceCheckpoint` 仍只是 seed authority + parked stream vector。`CoreRuntime` 可用它直接创建带默认 clock 的新 Runtime，却不校验 ProjectIdentity、BuildSet、World、clock epoch、schedule graph、tick、input/effect journal 或 replay manifest。
- serde 仍先构造无界 `Vec<RandomStreamCheckpoint>`，随后才在 Runtime registry 检查 65,536 上限；count/bytes/depth/digest/cancellation admission 不存在。
- App、Runtime Host、Runtime Interface、Editor production Rust 对 random service/seed/checkpoint 的复核命中均为 0。AI 仍用 `DefaultHasher(tree,node,tick)`；Particle CPU 仍用私有 64-bit LCG，GPU 则把 64-bit seed 折成 32-bit 后结合 slot/浮点 age hash，二者没有共同 profile、checkpoint 或 parity 合同。

本报告新增 **1 项 P0**：`RT-RANDOM-P0-001`。Runtime153 已有的 source/Cargo 两项 P0 继续由 Runtime209 记账，状态保持 `1 Open / 1 Partial / 0 Closed`，本报告不重复计数。Runtime154 的 9 项 P1 重判为 **9 Open / 0 Partial / 0 Closed**，4 项 P2 重判为 **4 Open / 0 Partial / 0 Closed**；24 道资格门为 **17 Fail / 7 Partial / 0 Pass**。此前 4 个 Pass 降为 Partial，是因为当前候选存在确定的源码编译断口，静态局部性质不能再冒充可执行资格证据。

## 2. 冻结范围与 currentness

统计覆盖所列目录中的全部 Rust 文件，而不是只取 symbol 命中文件。fingerprint 使用 lower-case repo-relative path、文件 SHA-256、路径排序、`path<TAB>hash` 与 LF 拼接后再次 SHA-256。

| 范围 | files | lines | non-empty | bytes | tests | ignored | unsafe tokens |
|---|---:|---:|---:|---:|---:|---:|---:|
| `zr_contracts/src/random` | 13 | 923 | 808 | 28,297 | 10 | 0 | 0 |
| Runtime random framework/kernel/handle + `CoreRuntime` owner | 20 | 2,332 | 2,055 | 79,260 | 27 | 1 | 0 |
| AI behavior-tree consumer | 26 | 6,231 | 5,690 | 223,860 | 52 | 18 | 0 |
| Particle runtime CPU/GPU consumer | 44 | 8,847 | 8,058 | 310,053 | 54 | 3 | 5 |
| Zircon union | **103** | **18,333** | **16,611** | **641,470** | **143** | **22** | **5** |
| Unreal/Godot/Bevy/Fyrox/Unity Graphics references | **10** | **4,102** | **3,473** | **150,286** | **2** | n/a | n/a |

Zircon union fingerprint 为 `8d9d4518edf2d74519c23a543d6819cd680296949b1beb489f8a97b2c6b6ddf9`；参考输入 fingerprint 为 `e8c2fb9446b5917fbc5279c094f98777158ba69a397db99f5716a9f323c3e2db`。Zircon union 的 currentness 为 **HEAD 63 / index 103 / dirty 70 / untracked 0**。

当前 HEAD 为 `f31fd06f69fdaedb70a0a56fe6d0268de1af83a6`。工作树存在大量其他任务的在途变化；本报告只冻结上述输入，不归因、不回退、不吸收其余变更。Tooling 按用户要求排除，也未查询、轮询、等待或实时跟踪协调器。

本轮逐文件阅读 contracts、kernel、tests、CoreRuntime owner/handle、AI behavior tree 与 Particle CPU/GPU/runtime snapshot；对 App、Host、Interface、Editor 执行 production symbol 负扫描；再对照本地五套参考源码。没有运行 Cargo、Miri、fuzz、跨平台 replay、GPU parity、fault/soak 或产品 benchmark，因为当前任务是 review-only，且 source/Cargo graph 已有确定阻断。

Runtime154 引用的 65,536-stream 历史 probe（p50 22.925 ms、p95 41.934 ms）只作为旧实现成本证据。本轮未重跑，不能把该数字当作当前性能结果，更不能据此声称优于 Unreal。

## 3. 新增 P0

### RT-RANDOM-P0-001：checkpoint v2 校验返回不存在的 error variant

**状态：Open。**

`zircon_runtime/crates/zr_contracts/src/random/service_checkpoint.rs` 当前在 stream generation 与 service generation 不一致时构造：

```rust
RandomServiceCheckpointError::StreamAuthorityGenerationMismatch {
    index,
    service_generation,
    stream_generation,
}
```

但 `zircon_runtime/crates/zr_contracts/src/random/checkpoint_error.rs` 的 enum 只有 `UnsupportedFormatVersion`、`NonCanonicalStreamOrder` 与 `StreamAlgorithmMismatch`。因此当前 checkpoint v2 变更在契约 crate 内部就无法完成名称解析。`tests/checkpoint.rs` 又已经按新 generation 行为编写，说明这不是废弃分支，而是未完成的 cross-file migration。

**影响：** random contracts、Runtime restore、checkpoint tests 及任何未来 replay artifact 都无法形成可编译、可版本化的基础。由于 `zr_contracts` 尚未进入 workspace graph，这类断口不会被默认 workspace check 自动发现。

**重构要求：**

1. 在同一 atomic change 中补齐 typed variant、字段、Display、contract tests 和所有 exhaustive matches。
2. 把 `zr_contracts` 正式接入 workspace/runtime dependency graph，建立 clean-checkout compile gate；不能通过删除 generation 校验回退到 mixed-era 风险。
3. 增加 v1 artifact 的明确迁移或 fail-closed policy。直接把 `FORMAT_VERSION` 改回 1、给 generation 默认为 0、或在 serde fallback 中猜测旧数据都不可接纳。
4. 合并前必须在当前源码冻结点重新审查，因为该断口位于高频变化的 dirty 文件中。

## 4. 可保留的底座及其边界

| 能力 | 当前证据 | 只能在以下条件下保留 |
|---|---|---|
| 算法身份 | `RandomAlgorithmId` 使用稳定 `u16`，未知值 fail closed | generator、derivation、key schema、distribution policy 一起进入 BuildSet/Replay profile |
| stable key framing | world/entity generation、system、purpose、authoring seed进入 BLAKE3 framing | key/capability 由 Runtime authority 签发，调用者不能伪造 raw IDs |
| stream progress | PCG state、odd increment、draw index 可序列化 | progress 必须绑定 session/world/tick，并由 transaction commit 发布 |
| unique mutable lease | 同 key 同时只有一个 lease；draw path不持 registry lock | normal Drop 默认 abort，只有 step commit 能提交 |
| retention cap | 默认 65,536 retained streams | wire decode、memory bytes、owner quota、checkpoint deadline 同样有界 |
| canonical ordering | BTreeMap order 与 strict-increasing validation | 完整 artifact 还要有 digest、profile、BuildSet 和 bounded codec |
| reseed generation | checked successor receipt，active lease 时拒绝 reseed | receipt 还需 principal、reason、session、profile 与 durable operation ID |
| mixed-era lock order | checkpoint/reseed 使用 `registry -> seed` 顺序 | 扩展成 world/time/schedule/random 的共同 quiescence barrier |

这些基础不应被 thread-local/global RNG、复制 stream value、私有 consumer seed 或兼容 raw-key acquire 取代。

## 5. 当前 owner 与断路图

```text
CoreRuntime
  `- RandomService
       `- Arc<RandomAuthority>
            |- Mutex<master seed + generation>
            `- Mutex<BTreeMap<RandomStreamKey, Available | Leased>>
                 `- RandomStreamLease -> local PCG32 draws -> Drop commits

CoreRuntime public restore
  |- with_random_seed
  |- with_random_service_state
  `- with_random_service_checkpoint
       (default clock possible; no Project/BuildSet/World/Schedule/Tick admission)

AI RandomSelector -------> DefaultHasher(tree id, node id, tick)
Particle CPU ------------> private 64-bit LCG state
Particle GPU ------------> folded 32-bit seed + hash(slot, float age)
App/Host/Interface/Editor -> no seed/checkpoint/session product contract
```

随机 draw 不需要中央全局锁；工程目标应是 Runtime 在 tick admission 时签发不可伪造的 scoped capability，consumer 本地无锁执行，step commit 原子发布 progress，abort 丢弃 staged state，完整 checkpoint 再由共同 snapshot barrier 组合 world/time/schedule/random/input/effect 状态。

## 6. Runtime154 P1 重判

| ID | 状态 | 当前工作树证据 | 必须重构为 |
|---|---|---|---|
| Runtime154-P1-001 | Open | `CoreRuntime::with_random_service_checkpoint*` 仍可从 partial checkpoint + default/new clock 建立 Runtime | 仅内部 assembly primitive；公开入口只接收验证过的 `SimulationCheckpointManifest` |
| Runtime154-P1-002 | Open | world/entity/system/purpose/key 构造器仍公开接收裸 `u64` | session-bound `RandomStreamCapability`，由 World/System authority 签发并校验 generation/BuildSet |
| Runtime154-P1-003 | Open | `RandomStreamLease::Drop` 仍无条件调用 `commit`；无 tick/abort/rollback token | tick-scoped staged lease；normal Drop abort，step transaction 唯一 commit authority |
| Runtime154-P1-004 | Open | checkpoint 只检查 `active_leases == 0`；没有关闭 schedule/task/world admission | `SimulationSnapshotCoordinator` 的共同 quiescence barrier |
| Runtime154-P1-005 | Open | serde wire 先创建无界 `Vec`，Runtime 后置检查 entry count | bounded sequence visitor + count/bytes/depth/digest/cancel budget |
| Runtime154-P1-006 | Open | 只有 generator ID；`zircon.random.stream.v1` derivation domain 仍是 runtime 私有常量 | versioned `RandomProfileId`，显式包含 generator/derivation/key/distribution/float policy |
| Runtime154-P1-007 | Open | default master seed 仍为 0；四个产品层 production scan 均 0 命中 | launch/session manifest 中的 seed provenance、profile、permission、replay binding 与 receipt |
| Runtime154-P1-008 | Open | checkpoint 仍在单 registry mutex 内复制全部 stream；无 deadline/cancel/page/swap | immutable generation swap 或有预算的短 barrier，后台 canonicalize/encode |
| Runtime154-P1-009 | Open | 公开观测仍只有 registered/active counts | bounded diagnostics：contention、reject、draw volume、capacity、eviction、checkpoint bytes/time、restore/reseed reason |

generation 字段是必要修复，但没有关闭其中任何一项：它只验证 parked stream 与 seed era 相同，不能证明 world、tick、schedule、profile 或 BuildSet 相同。

## 7. Runtime154 P2 重判

| ID | 状态 | 当前差异 | 长期目标 |
|---|---|---|---|
| Runtime154-P2-001 | Open | 公共采样面仍只有 u32、bounded u32、unit f32；Particle 自写 range/vector/shape | versioned distribution API：range、bool、shuffle、weighted choice、normal、vector/sphere/cone/mesh sampling |
| Runtime154-P2-002 | Open | deterministic simulation RNG 与 secure/nondeterministic entropy 未在类型上隔离 | 独立 `EntropyService` capability；禁止 PCG 用于 token/session secret |
| Runtime154-P2-003 | Open | acquire/release/checkpoint/reseed/evict 共用一把 BTreeMap mutex；scope eviction O(N) | deterministic shards + world/entity lifecycle index，保持 canonical output 与 single owner |
| Runtime154-P2-004 | Open | 只有 stateful PCG32；Particle GPU 自有 stateless hash ABI | CPU/GPU/counter-based/SIMD provider IDs、dimension allocation、overflow 与 parity/migration policy |

## 8. 消费者复核

### 8.1 AI behavior tree

`weighted_random_child` 每次构建 `Vec<f32>`，再用 `std::collections::hash_map::DefaultHasher` 对 tree id、node id、tick hash，最后把 `u64` 映射到权重区间。它没有 world/entity generation、RandomProfile、draw index、checkpoint progress 或 Runtime-issued capability。当前测试主要证明新旧函数结果一致和减少 key string 分配，这会把旧的私有随机语义固化为优化基线，而不是迁移到统一 authority。

应按 Runtime08F/Runtime22 owner plan hard-cut：compiled behavior node 获得稳定 purpose ID；behavior instance 在 fixed-step context 中领取 scoped lease；weighted choice 由公共 versioned distribution 实现；abort 不提交 draw；tree checkpoint 保存统一 stream identity/progress，而不是从 tick 重新 hash。

### 8.2 Particle CPU

`ParticleRng` 是私有 64-bit LCG，seed 由 asset seed、handle raw value 与 emitter index 常量异或得到。`ParticleRuntimeSnapshot` 只暴露 emitter counts、sprites、diagnostics 和 GPU feedback；没有 CPU RNG state、spawn accumulator、burst cursor、pool state 的 durable restore contract。产品 snapshot 名称不能被误认为 replay checkpoint。

应由 Runtime26 owner plan定义 Particle simulation checkpoint，并把 CPU sampling 接到统一 `RandomProfile`/distribution surface。handle 必须是稳定 generation identity，不能把 transient raw handle 当作长期 seed ABI。

### 8.3 Particle GPU

GPU planner 把 64-bit system seed 折成 32-bit；shader 再通过 `hash_u32(emitter.seed ^ slot ^ bitcast<u32>(emitter.sim.z))` 生成 spawn seed，其中 `sim.z` 是浮点 age。它与 CPU LCG 的 draw order、range conversion、shape sampling都不等价，也没有声明“有意不等价”的 provider/profile ID。GPU owner 虽有 prepared-state/commit 局部事务，这是可保留进展，但没有与 world fixed-step transaction 或 durable checkpoint 组合。

应使用明确的 counter tuple，例如 `(session/world, emitter generation, spawn sequence, particle slot, dimension)`，固定 hash/provider ID 和 float conversion；CPU/GPU 要么提供 golden parity，要么在 profile 中明确标记非等价并阻止 replay backend 漂移。

### 8.4 产品接线

App、Runtime Host、Runtime Interface、Editor production Rust 对 `RandomService`、`RandomStreamKey`、`RandomServiceCheckpoint`、`RandomServiceState`、master/random seed、reseed 与常见 `rand` RNG 名称均为 0 命中。当前 `zircon_plugins/ai/runtime` 只依赖 `zircon_runtime` feature 集，Particle 只依赖 graphics feature；两者都没有显式 random contract dependency/capability。

这意味着 seed policy、restore、reseed permission、profile negotiation、diagnostics 与 replay correlation 尚未成为可由真实产品启动和操作的功能。

## 9. 参考引擎对照

| 参考 | 已验证的可借鉴边界 | Zircon 当前缺口 | 不照搬的部分 |
|---|---|---|---|
| Unreal `FRandomStream` + Demo replay | initial/current seed、Initialize/Reset、range/vector API；Demo driver 把 checkpoint、scrub、fast-forward 放在 replay owner 下 | Zircon低层 state 更严格，但 partial random checkpoint 越权成为 Runtime restore 入口 | 不采用 name/time seed、任意 value-copy stream或旧低位质量算法 |
| Godot `RandomPCG`/RNG | seed/state/increment、randomize、range/normal/weighted sampling与 restore tests | Zircon缺 entropy taxonomy、丰富 distribution、明确 restore product surface | 不把独立 RNG object 当作完整引擎 replay authority |
| Bevy sampling | shape/mesh distribution 接收 caller-owned RNG，算法与 owner 解耦 | Zircon公共 sampler 过窄，Particle 自写 shape/range | Bevy external RNG 默认算法不是持久格式或全引擎 authority |
| Fyrox Particle | particle owner持有 RNG，可 reset，并接入 Reflect/Visitor 生命周期 | Zircon CPU RNG progress未进入 durable snapshot，GPU另有算法 | 未固定版本的默认 RNG 不能直接成为长期 wire ABI |
| Unity Graphics/VFX | start seed、resetSeedOnPlay、Reinit、prewarm、timeline scrub；shader random helper显式分层 | Zircon没有产品 seed/reinit/scrub 控制，也没有 CPU/GPU profile 协商 | VFX局部控制面不能替代 World/replay transaction |

Zircon 的目标应比这些局部接口更严格：参考引擎用于证明必要能力和 owner 边界，不用于证明任何单一参考实现已经解决了跨 World、任务图、GPU 与 replay 的全部确定性问题。

## 10. 目标架构

```text
Launch / Replay Admission
  ProjectIdentity + BuildSet + RandomProfile + SeedProvenance
                         |
                         v
SimulationSessionAuthority
  |- RandomCapabilityIssuer
  |- FixedStepTransactionCoordinator
  |- SimulationSnapshotCoordinator
  `- DeterminismDiagnostics
                         |
          +--------------+---------------+
          |                              |
          v                              v
RandomStreamRegistry                CounterRandomProvider
  deterministic shards               CPU/GPU profile
  immutable parked generations        tuple/dimension contract
          |                              |
          +---------------+--------------+
                          v
                 Consumer adapters
       AI / Particle / Physics / Gameplay / Scripting
                          |
                          v
Step commit -> publish progress; abort -> discard staged draws

Snapshot barrier
  -> World + Clock + Schedule + Random + Input + Effect journals
  -> bounded immutable artifact -> digest/sign -> async persistence
```

核心合同必须包括：

1. `RandomProfileId`：generator、derivation、key schema、distribution、float policy、CPU/GPU provider 都是版本化身份。
2. `RandomStreamCapability`：绑定 session、BuildSet、world/entity generation、compiled system、purpose；serialized key 只是数据，不是权限。
3. `RandomStepLease`：保存 base/staged state 和 tick token；normal Drop abort；只有 coordinator 的 commit receipt 能发布。
4. `SimulationCheckpointManifest`：绑定完整复合状态、各部分 digest、quiescence fence 和 artifact budget；不再公开 partial Runtime restore。
5. `RandomServiceDiagnosticsSnapshot`：有界、无 seed/state 泄漏，能关联 session/world/tick/profile。

## 11. 分层重构里程碑

### M0：先恢复可编译、可复现的 source graph

修复 `RT-RANDOM-P0-001`；把 `zr_contracts` 纳入 workspace/runtime dependency；补齐 v1/v2 migration/rejection test；确保所有 random 输入存在于 HEAD 的 clean checkout。退出门是 clean checkout 可 compile/test contracts 与 Runtime random kernel，default workspace graph不能遗漏该 crate。

### M1：冻结 profile 与 identity authority

引入 `RandomProfileId`、BuildSet binding、seed provenance；由 World/System registry 签发 capability；删除 public raw-key acquire 权限路径。退出门是任意 plugin不能伪造另一 world/system stream，未知 profile fail closed，跨进程/架构 golden vectors固定全部语义。

### M2：与 fixed-step transaction 原子化

实现 staged lease、tick token、explicit commit/abort；把 random progress纳入 Runtime22 fixed-step transaction；为 immediate authoring/diagnostic draw提供独立命名 API。退出门是 error/cancel/retry/rollback均不前移 committed RNG，draw hot path仍无 registry lock/allocation。

### M3：完整 checkpoint 与 bounded codec

建立共同 quiescence barrier；组合 World/Clock/Schedule/Random/Input/Effect；使用 bounded visitor、count/bytes/depth/digest budget；immutable capture 后异步 encode/persist。退出门是 partial checkpoint不能公开创建 Runtime，malformed/oversize/mixed-build/mixed-world artifact在分配和 mutation 前拒绝。

### M4：consumer hard-cut

AI weighted choice迁移到公共 distribution；Particle CPU迁移到统一 profile；Particle GPU引入counter/provider identity及 parity 或显式 non-parity policy；移除私有 DefaultHasher/LCG/隐式 shader salt ABI。退出门是 production source不再出现私有 simulation RNG owner。

### M5：产品控制与可观测性

App/Host/Interface launch/session contract携带 profile/seed provenance；Editor提供受 capability 约束的 inspect/reseed/reinit/replay 操作；输出 bounded diagnostics、typed receipt、audit correlation。默认 seed也必须进入manifest digest，UI不能直接改Runtime内存或伪造成功。

### M6：性能结构

建立 deterministic shards/lifecycle index、immutable generation swap、checkpoint budget/cancel、consumer local batching和CPU/GPU bulk generation。65,536-stream capture必须同时满足冻结预算、峰值内存、cancellation与正确性，不能只优化均值。

### M7：资格矩阵与竞品基线

覆盖 Windows/Linux、x86_64/arm64、debug/release、toolchain matrix、dual-run digest、save/load/fault/fuzz、并发 schedule、CPU/GPU parity与长时soak；在相同内容/硬件/线程/质量下与Unreal建立原始trace基线。没有这些证据时不得声称“优于 Unreal”。

## 12. 资格门

| Gate | 状态 | 当前证据/缺口 |
|---|---|---|
| G01 clean checkout含当前 contracts/kernel/handle/consumers | Partial | 103/103在index，只有63/103在HEAD，70 dirty |
| G02 workspace/runtime Cargo图可构建 `zr_contracts` | Fail | root members/dependencies与Runtime dependencies均缺owner |
| G03 PCG vectors与next-draw restore稳定 | Partial | 源码/tests存在；当前契约编译断口阻止当前候选资格 |
| G04 generator/derivation/key/distribution profile显式 | Fail | 只有generator ID，derivation domain仍是私有常量 |
| G05 same-key唯一 mutable lease | Partial | registry源码满足；未在当前可编译候选验证 |
| G06 draw path无 registry lock/hash/allocation | Partial | lease本地draw满足；未在当前可编译候选验证 |
| G07 retained stream有界 | Partial | Runtime有65,536 entry cap，wire/memory/owner quota仍缺 |
| G08 checkpoint canonical且校验algorithm/generation | Partial | ordering/algorithm/generation源码存在，但generation error variant缺失 |
| G09 wire decode有count/bytes/depth/digest预算 | Fail | serde先构造无界Vec |
| G10 checkpoint绑定Project/BuildSet/world/clock/schedule | Fail | random artifact没有这些字段 |
| G11 snapshot与schedule/task/world共享quiescence | Fail | 只检查active lease |
| G12 fixed-step abort/retry不推进RNG | Fail | Drop无条件commit |
| G13 AI不再使用DefaultHasher/tick seed bypass | Fail | production仍直接使用 |
| G14 Particle CPU使用共享profile/distribution | Fail | production仍使用私有LCG |
| G15 Particle GPU有provider ID与parity合同 | Fail | 32-bit fold + age/slot hash无profile |
| G16 App/Host/Interface/Editor有seed provenance/ABI | Fail | production scan均0命中 |
| G17 reseed/checkpoint/restore有typed operation receipt | Fail | 只有局部seed generation receipt |
| G18 BuildSet包含random profile/schema | Fail | 无相关manifest binding |
| G19 replay只接受完整manifest restore | Fail | CoreRuntime公开接受partial checkpoint |
| G20 diagnostics覆盖contention/capacity/checkpoint/reject | Fail | 只有两个即时count getter |
| G21 65,536 capture满足hitch/memory/cancel预算 | Fail | 历史p95 41.934 ms，当前无budget/cancel证据 |
| G22 跨平台/toolchain/fault/fuzz/dual-run资格 | Fail | 未建立矩阵 |
| G23 range/distribution/shape统一合同 | Partial | kernel有3个primitive，Particle仍私有实现 |
| G24 deterministic RNG与secure entropy类型隔离 | Fail | 没有EntropyService/capability taxonomy |

## 13. 禁止的临时修补

- 不得删除 generation 校验、降回 checkpoint v1 或让缺失 generation 默认为 0 来绕过 P0。
- 不得保留 public raw-key acquire 再额外提供“推荐”capability；权限入口必须 hard-cut。
- 不得让 `Drop` 根据线程是否 panic、返回值或隐式标志猜测 commit；commit必须来自 fixed-step authority。
- 不得把 random-only checkpoint 改名为 world/replay checkpoint后继续公开恢复。
- 不得让 AI/Particle 继续私有 RNG，只在外层记录一个 seed 当作统一 authority。
- 不得只提高 `Vec` 上限或在反序列化完成后检查 bytes；admission 必须发生在大分配之前。
- 不得用 `unsafe` bulk copy、nondeterministic hash/shard iteration 或 silent fallback 换取 benchmark数字。
- 不得用 ignored microbenchmark、单机均值或不同内容/画质配置宣称性能优于 Unreal。

## 14. 首个实现切片

后续开始改代码时，第一个切片只应做 M0：

1. RED：以 `zr_contracts` 独立 manifest/managed validator证明当前缺失 variant 的编译失败，并增加 v2 generation mismatch契约测试。
2. GREEN：补齐 error variant及所有match；把 crate接入workspace/runtime graph；修正必要的manifest/source wiring。
3. REFACTOR：冻结v1/v2迁移/拒绝策略，禁止默认generation与删除校验。
4. 验证：clean checkout compile/test contracts + Runtime random；再复核HEAD/index/dirty与Cargo graph。

M0关闭后，第二个切片才进入 M2 的 staged lease RED test：同一 fixed tick draw 后 abort/retry，必须得到相同 draw且 committed checkpoint不变化。不要同时迁移 AI/Particle、重写 codec 和做 sharding；这些依赖 transaction/profile合同先稳定。

## 15. 审查收口

当前 random kernel 有值得保留的低层工程基础，但源码编译边界、source graph、transaction、完整 snapshot、bounded wire、consumer统一与产品接线都未闭合。最危险的误判是把“有可序列化 PCG stream”和“有引擎级 deterministic replay”视为同一件事。它们之间还缺 session identity、BuildSet/profile、world/tick transaction、共同 quiescence、durable artifact、产品 ABI 与跨后端资格证据。

本报告只完成 review 与重构规划，没有修改 production Rust/Cargo/ABI/tests/UI，没有运行 Cargo 或动态产品验证。下一轮实现必须从 `RT-RANDOM-P0-001` 和 Runtime209 的 source/Cargo gate 开始，再按 M1-M7 依赖顺序推进。
