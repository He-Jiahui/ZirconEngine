---
title: Runtime Random Authority、Stream、Checkpoint、Replay、Consumer 与 Performance 当前源码工程化差距
category: zircon_runtime
report_id: Runtime154
review_date: 2026-08-29
baseline_head: 8aabbee3e99dc919f6da4611e3a44e8463a7fe7f
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: partial_foundation_product_incomplete
source_recheck_required: true
related_code:
  - Cargo.toml
  - zircon_runtime/Cargo.toml
  - zircon_runtime/crates/zr_contracts/src/random
  - zircon_runtime/src/core/runtime/random
  - zircon_runtime/src/core/runtime/handle/random.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_plugins/ai/runtime/src/behavior_tree
  - zircon_plugins/particles/runtime/src
plan_sources:
  - docs/plans/optimize/zircon_runtime/22-time-clock-domain-fixed-step-determinism-rng-replay-scheduling-review.md
  - docs/plans/optimize/zircon_runtime/22/2026-08-29-random-stream-registry-architecture-and-performance-plan.md
  - docs/plans/optimize/zircon_runtime/22/2026-08-25-random-authority-ai-consumer-migration-plan.md
  - docs/plans/optimize/zircon_runtime/22/2026-08-24-fixed-step-transaction-architecture-and-performance-plan.md
  - docs/plans/optimize/zircon_runtime/101-runtime-support-crates-contracts-math-resource-rhi-wgpu-workspace-boundary-device-lifecycle-product-integration-current-source-review.md
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

# Runtime Random Authority、Stream、Checkpoint、Replay、Consumer 与 Performance 当前源码工程化差距

## 1. 结论

当前随机内核已经不是临时 `rand()` 包装。它有明确的 `RandomAlgorithmId::Pcg32XshRrV1`、BLAKE3 stable-key 派生、world/entity generation、system/purpose/authoring seed、PCG state/increment/draw index、无偏 bounded draw、单 key 唯一 mutable lease、65,536 retained-stream 上限、canonical checkpoint、reseed generation，以及 checkpoint/restore 后 next-draw identity。2026-08-29 的 mixed-era checkpoint 竞态也已通过保持 `registry -> seed` 单一锁序修复；该 failure 保持 `fixed`，本报告不重新打开。

但这仍只是一个质量较好的 **random kernel slice**，不是完整引擎级 determinism/replay 产品：

- `zr_contracts/random`、runtime random kernel、framework projection 和 handle 接线仍有 33 个 untracked 文件；根 workspace/runtime Cargo 图也仍由 Runtime153-P0-001/P0-002 阻断。
- `CoreRuntime` 可以只凭 `RandomServiceCheckpoint` 构造新 Runtime，而 checkpoint 不绑定 Project/BuildSet、World snapshot、clock、schedule graph、simulation tick 或 replay manifest，允许恢复出格式合法但仿真组合不可能的状态。
- `RandomStreamLease::Drop` 无条件提交 draw progress，不参与 fixed-step begin/commit/abort；系统失败或重试时，world/clock 可以 abort，而 RNG 已经前移。
- checkpoint 只证明当前没有 active lease，没有阻止 schedule/task 在检查后立刻 acquire，也没有与 World capture 建立同一 quiescence barrier。
- `RandomServiceCheckpointWire` 先通过 serde 构造无界 `Vec<RandomStreamCheckpoint>`，之后 Runtime 才检查 65,536 entry 上限；不可信存档/网络输入可在 admission 前消耗无界内存。
- App、Runtime Host、Runtime Interface 的 production Rust 对 `RandomService`、checkpoint、master seed 或 random seed 都是 0 命中。AI 仍用 `DefaultHasher(tree,node,tick)`；Particle CPU/GPU 仍维护两套不兼容算法。

本报告新增 **0 项 P0、9 项 P1、4 项 P2**，24 项资格门为 **18 Fail / 2 Partial / 4 Pass**。Runtime153 的两项 source/Cargo P0、Runtime22 的 TIME-P1-021..040、Runtime08F 的 AI consumer、Runtime26 的 Particle/VFX consumer 继续作为 canonical owner；本报告只对当前新源码暴露出的具体合同缺口新增计数，不重复已有 owner 条目。

## 2. 冻结范围、currentness 与方法

### 2.1 物理范围

fingerprint 口径为 lower-case repo-relative path、文件 SHA-256，按路径排序后以 `path<TAB>hash` 和 LF 拼接，再计算 SHA-256。

| 范围 | files / lines / nonempty / bytes / tests | tracked / modified / untracked / dirty | fingerprint |
|---|---:|---:|---|
| Zircon random contracts/kernel/runtime wiring + AI/Particle consumers | **52 / 7,937 / 7,187 / 263,718 / 32** | **19 / 13 / 33 / 46** | `1b9fa8d3abf2a483ffff9b542056bde51d8308566c33ff0a981e64d3b8e1df72` |
| Unreal/Godot/Bevy/Fyrox/Unity Graphics reference selection | **10 / 4,102 / 3,473 / 150,286 / 2** | n/a | `e8c2fb9446b5917fbc5279c094f98777158ba69a397db99f5716a9f323c3e2db` |

本轮逐文件阅读了 `zr_contracts::random` 与 runtime random production/tests，全量核对 CoreRuntime owner/handle 构造链，并深读 AI weighted selector、Particle CPU RNG、GPU seed/hash/shader、Particle reset/snapshot。对 App、Runtime Host、Runtime Interface 和 Plugins 做 production symbol 负扫描：前三者目标命中均为 0；Plugins 的 4 个命中都只是 Particle 私有 `rng_seed`。

当前 HEAD 为 `8aabbee3e99dc919f6da4611e3a44e8463a7fe7f`。工作树有其他 session 的大量在途修改；本报告只冻结所列内容，不吸收、不回退，也不将 dirty 归因于本轮。Tooling 按用户要求排除；未查询、轮询、等待或实时跟踪协调器。

### 2.2 动态证据边界

本轮是 review-only，没有运行 Cargo、Miri、fuzz、跨平台 replay、fault/soak、GPU parity 或跨引擎 benchmark。`random-stream-registry` 子计划记录的 isolated test 和 checkpoint probe 作为历史证据引用，不能替代当前 dirty workspace 的 managed validation。特别是 65,536 stream checkpoint 的 p50 22.925 ms、p95 41.934 ms只证明当前实现的成本，不证明产品帧可接受，也不证明优于 Unreal。

## 3. 当前可保留的工程基础

| 能力 | 当前证据 | 保留条件 |
|---|---|---|
| 算法身份 | `RandomAlgorithmId` 使用稳定 `u16`，未知值 fail closed | generator 与 derivation/schema identity 必须共同进入 BuildSet |
| 稳定 owner key | world/entity generation、system、purpose、authoring seed进入 BLAKE3 framing | key 必须由 Runtime issuer 产生，不能由任意调用者伪造 raw IDs |
| stream state | PCG state、odd increment、draw index 可序列化与恢复 | state 必须绑定 owner/build/tick，不可作为孤立 blob 接纳 |
| 单 mutable lease | 同 key 并发 acquire 只有一个成功；draw path 无 registry lock/hash | lease 必须加入 simulation transaction，失败不能隐式 commit |
| retention bound | 默认最多 65,536 retained keys，scope eviction 显式 | wire decode 也必须有 count/bytes/depth 预预算 |
| canonical checkpoint | BTreeMap order、strict increasing key、algorithm match | 与完整 World/clock/schedule snapshot 原子组合并带 digest |
| reseed generation | checked successor generation、active lease 时拒绝 | seed 来源、principal、reason、session identity 与 receipt 必须进入产品合同 |
| mixed-era atomicity | checkpoint 在 registry guard 内捕获一次 seed state | 扩展为全 simulation quiescence，不只保证 random 内部两把锁 |

这八项基础应该保留，不应退回 thread-local/global RNG、每消费者私有 seed 或复制 stream value 的兼容路径。

## 4. 当前 owner 与断路图

```text
CoreRuntime
  `- RandomService
       `- Arc<RandomAuthority>
            |- Mutex<master seed + generation>
            `- Mutex<BTreeMap<RandomStreamKey, Available | Leased>>
                 `- RandomStreamLease -> local PCG32 draws -> Drop commits

CoreRuntime public constructors
  |- with_random_seed
  |- with_random_service_state
  `- with_random_service_checkpoint
       (no Project/BuildSet/World/Clock/Schedule/Tick admission)

AI weighted selector --------> DefaultHasher(tree,node,tick)
Particle CPU ----------------> private 64-bit LCG
Particle GPU ----------------> 32-bit seed fold + hash(seed,slot,float age)
App / Host / Interface ------> no random seed/checkpoint/session contract
```

产品目标不是让所有子系统每次 draw 都锁住中央服务。目标是：Runtime/World 在 tick admission 时签发不可伪造的 scoped stream capability；consumer 在本地无锁执行；step commit 原子发布 RNG progress，abort 丢弃；完整 checkpoint 由同一个 snapshot barrier 组合 world/time/schedule/random/input/effect 状态。

## 5. Runtime22 既有 finding 当前状态

| Existing ID | 当前状态 | current-source 复核 |
|---|---|---|
| TIME-P1-021 | Partial | RandomService、algorithm ID、master seed、key hierarchy 已存在；BuildSet/产品 seed provenance/clean source graph 未闭合 |
| TIME-P1-022 | Open | AI 仍缺 world/entity generation 和 runtime-issued random context |
| TIME-P1-023 | Open | `weighted_random_child` 仍在 `support.rs:58` 使用 `DefaultHasher` |
| TIME-P1-024 | Open | Particle CPU 私有 LCG 与 GPU hash/frame-age 算法仍分裂 |
| TIME-P1-025 | Not re-evaluated | WOC parity 属于本轮排除的 Tooling；沿用原 owner 状态，不新增计数 |
| TIME-P1-026 | Partial | draw index/state/checkpoint 已实现；fork/counter portfolio、consumer durable state 和 transaction integration 未完成 |
| TIME-P1-027..030 | Open | BuildSet schedule binding、effect journal、canonical encoding、determinism profile均未由 random slice关闭 |
| TIME-P1-031..040 | Open | random checkpoint 不等价于 replay bundle/world checkpoint/clock/seek/digest/qualification matrix |

AI consumer 的完整 hard-cut 设计已在 `22/2026-08-25-random-authority-ai-consumer-migration-plan.md` 中定义，本报告不复制其 finding。Particle CPU/GPU parity 与 authoring/product controls 继续由 Runtime26 和对应 Editor 报告拥有。

## 6. P1：必须在工程级 random/replay 前关闭

### Runtime154-P1-001：partial random checkpoint 被暴露为 Runtime 构造入口，允许不可能的复合状态

`CoreRuntime::with_random_service_checkpoint`（`runtime.rs:84`）和带 clock source 的同类入口可以直接建立一个全新 Runtime。`RandomServiceCheckpoint` 只含 service state 与 stream vector；random contracts 中 `BuildSet`、`SimulationTick`、`WorldCheckpoint`、`ReplayManifest` 均为 0 命中。

这会允许调用者把旧世界的随机进度与新世界、默认 clock、新 schedule graph 和不同 asset/schema 组合。序列化格式合法不代表仿真语义合法。应将该入口降为内部 assembly primitive；公开恢复必须接收 `SimulationCheckpointManifest`，先验证 ProjectIdentity、BuildSet、world/schema、clock epoch、schedule digest、random algorithm/derivation、input/effect journal fence，再原子创建 session。

### Runtime154-P1-002：RandomStreamKey 可由任意调用者伪造，authority 只管状态、不管身份签发

`key.rs:11/32/50/64/87/102` 的 world/entity/system/purpose/key 构造器均公开接收裸 `u64`。任何 plugin 都能伪造另一个 system/purpose、复用 entity generation 0，或让两个功能碰撞到同一个 logical stream。当前 single-lease invariant只保证同一 value 同时不能有两个 owner，不证明 value 来自真实 World/System authority。

应增加 Runtime-issued `RandomStreamCapability`：由 frozen system graph、World/Entity generation authority、asset/node identity 和 authoring seed 生成；public contract保留可序列化 identity，但 acquire 只接受由本 session issuer 签发且绑定 BuildSet/world epoch 的 capability。反序列化 key 是 candidate，不是权限。

### Runtime154-P1-003：lease 在 Drop 时无条件提交，RNG progress 与 fixed-step abort 不原子

`lease.rs:79-90` 在显式 release 或 Drop 时都将 progressed stream 放回 registry。API 不接收 `SimulationTickId`、step token、commit receipt 或 abort reason；random production files 对 `SimulationTick`、`rollback`、`abort` 都是 0 命中。

若一个 fixed system draw 后返回错误，Runtime22 fixed transaction 可以保留 clock/debt并 abort step，但 lease drop 已提交随机状态。重试同一 tick 会得到下一随机数，形成不可复现 failure path。应把 lease 改为 tick-scoped staged progress：normal Drop 默认 abort/discard，只有 step commit authority 能原子发布；非事务性工具/authoring调用必须使用明确命名的 immediate scope，不可共享 simulation API。

### Runtime154-P1-004：checkpoint 的“零 active lease”不是 schedule/world quiescence barrier

`registry.rs:100-123` 在 registry mutex 内检查 `active_leases == 0` 并复制 parked streams，这正确关闭了 random 内部 mixed-era race，但没有阻止另一个 task 在 checkpoint 结束后、World capture 前 acquire/draw，也没有等待 tick-scoped task graph、deferred command、input/effect journal 或 World mutation quiesce。

应由 `SimulationSnapshotCoordinator` 关闭新 step/task admission，等待当前 step terminal，冻结 World generation 与 journal fence，在同一 barrier 内捕获 clock/schedule/random/world，再快速释放 simulation；压缩、编码、digest 和持久化在 immutable snapshot 上异步执行。random service不应自行宣称 engine checkpoint。

### Runtime154-P1-005：checkpoint wire 在 admission 前构造无界 Vec，缺少 bytes/depth/digest 预算

`service_checkpoint.rs:14-26` 通过 derive serde 先构造 `Vec<RandomStreamCheckpoint>`，`validate` 只检查 version、algorithm 和 strict order；65,536 cap 直到 `RandomStreamRegistry::from_checkpoints` 才检查。没有 max bytes、decode depth、checksum/digest、owner/build identity或 cancellation。

应使用 bounded decoder/sequence visitor，在分配前读取 envelope 的 count/encoded length 并验证 profile budget；每 entry 增量验证 strict order、algorithm 和 key grammar。完整 artifact 还需要 content digest、schema/derivation versions、BuildSet与 atomic file commit。未知版本 fail closed；支持迁移时由显式 migrator 产生新 artifact，不在 serde fallback 中猜测。

### Runtime154-P1-006：generator ID 没有完整表达 stream derivation/schema identity

`RandomAlgorithmId::Pcg32XshRrV1` 标识 PCG generator；真正决定初始 state/sequence 的 BLAKE3 framing 位于 runtime 私有 `derivation.rs:7-49`，domain 字符串是 `zircon.random.stream.v1`。contracts 没有独立 `RandomDerivationId`/key-schema version，BuildSet也不记录该实现。

如果以后调整 BLAKE3 input framing、key字段、byte order或截断，同一 service checkpoint 对尚未出现的 key 会派生不同 stream，而已 parked key 仍延续旧 state，形成同一 artifact 内的双语义。应将 generator、derivation、key schema、float/distribution policy作为一个 versioned `RandomProfileId` 或明确分离的 IDs，全部进入 checkpoint/replay/BuildSet，并提供跨语言/跨架构 golden vectors和迁移/拒绝策略。

### Runtime154-P1-007：产品启动没有 seed provenance、policy 或 ABI，默认 seed 0 被静默采用

`RandomService::default` 固定 master seed 0，CoreRuntime 暴露 `with_random_seed`，但 App、Runtime Host、Runtime Interface production Rust 对 RandomService/checkpoint/master seed/random seed 均为 0 命中。实际 dynamic/project session不能声明 deterministic seed、恢复 checkpoint、获得 seed receipt，也不能区分 user-authored seed、server seed、recorded replay seed或 nondeterministic launch seed。

应由 versioned launch/session manifest携带 `RandomSessionPolicy`：seed source、seed value或 sealed provenance、generation、profile ID、determinism level、reseed permission和 replay binding。Runtime Interface capability/version协商后传入；默认值只能是明确的 profile default并进入 manifest digest，不能只是 CoreRuntime 内部常量。

### Runtime154-P1-008：最大 checkpoint 在全局 registry mutex 内造成 22-42 ms 停顿，没有帧预算或取消合同

`checkpoint_with_authority_snapshot` 持有 registry mutex，分配并复制所有 entry，再取得 seed mutex。历史 optimized probe 对 65,536 streams测得 p50 22.925 ms、p95 41.934 ms，已经超过典型 60 Hz/30 Hz单帧预算。当前没有 checkpoint deadline、max capture time、incremental page、immutable generation swap、cancel或 telemetry。

工程修复不是在持锁循环里加 yield。应在 simulation barrier 中通过 generation swap/immutable parked-state snapshot快速冻结 owner，随后异步 canonicalize/encode；若保留 stop-the-world capture，必须有 profile budget、estimated bytes、admission reject和 hitch telemetry。任何优化都要同时证明 next-draw identity、单 owner、无 mixed-era和峰值内存，不能用 unsafe copy破坏合同。

### Runtime154-P1-009：只有即时计数，没有可诊断的 stream/checkpoint/reseed telemetry 与资格证据

公开观测只有 `registered_stream_count` 和 `active_lease_count`。random 目录对 telemetry/metric/cancel 均为 0 命中；没有 acquire contention、same-key rejection、draw volume、stream age/owner、capacity pressure、eviction、checkpoint entries/bytes/time、reseed block、restore reject和algorithm/profile标签。

应输出 bounded、无 seed/state 泄漏的 `RandomServiceDiagnosticsSnapshot` 与 trace events，绑定 Runtime/session/world/tick。shipping默认只暴露聚合指标；Editor/debug可通过 capability读取 owner-key摘要和 draw ranges。资格必须覆盖 Windows/Linux、x86_64/arm64、debug/release、不同 toolchain、save/load/fault、concurrent schedule和 deterministic dual-run，而不是只保留本机 18 个 unit tests。

## 7. P2：长期能力与性能上限

### Runtime154-P2-001：公共采样面只有 u32、bounded u32 和 unit f32

成熟消费方还需要稳定的 u64/i32/range、f64、bool、shuffle、weighted choice、unit vector、sphere/cone/box和可插拔 distribution。Bevy 的 shape/mesh sampling 把算法写成接收 caller-owned RNG，Godot/Unreal提供多种 range/vector分布。Zircon应定义 versioned distribution policy与generic sampler adapter；不能让每个 plugin自行写 float conversion和 modulo。

### Runtime154-P2-002：没有 deterministic simulation RNG 与 nondeterministic/secure entropy 的类型隔离

Godot区分显式 seed/state 与 randomize。Zircon当前只有 deterministic service，没有 OS entropy/CSPRNG owner、用途标签或禁止将 PCG用于 token/session secret 的编译/审查边界。应建立独立 `EntropyService` capability，绝不进入 replay state；simulation code只能拿 deterministic capability，security/session code只能拿 secure entropy capability。

### Runtime154-P2-003：scope eviction 和所有 admission 共用单 BTreeMap mutex

draw hot path是无锁的，这是正确基础；但 acquire/release/checkpoint/reseed/evict 全部竞争一把 mutex，`evict_world/evict_entity` 在锁内 O(N) 扫描全部 key。长期大世界/流式分区需要按 world/entity generation 的二级生命周期索引或 deterministic shards。优化必须维持 canonical checkpoint和同 key唯一 lease，不能换成 nondeterministic LRU。

### Runtime154-P2-004：缺少面向 GPU/counter-based/SIMD 的算法 portfolio 与能力协商

当前 profile只有 stateful PCG32。GPU粒子使用 stateless hash，说明产品确实需要按 slot/dimension 可随机访问的 counter-based stream；大批量生成又需要 SIMD/vector lane语义。应把这些作为明确 algorithm/provider IDs，定义 CPU/GPU parity或显式不等价，给出 dimension allocation、counter overflow、shader golden和backend migration policy，而不是让 shader salt成为隐含 ABI。

## 8. 参考引擎给出的边界

| 参考 | 可借鉴边界 | Zircon 当前差异 | 不应照搬 |
|---|---|---|---|
| Unreal `FRandomStream` / replay | initial/current seed、reset、range/vector API；Demo replay有checkpoint/scrub/fast-forward owner | Zircon低层state更严格，但random checkpoint未并入完整 replay operation | 不采用 name/time seed、value-copy stream和旧低位质量算法 |
| Godot `RandomPCG` / RNG | seed、state、increment、randomize、bounded/weighted/normal分布和restore测试 | Zircon缺randomize/entropy taxonomy与丰富distribution | 不把独立RNG对象直接当全引擎唯一authority |
| Bevy sampling | distribution接收显式 caller RNG，shape/mesh算法与RNG owner解耦 | Zircon各plugin仍复制采样实现，公共surface过窄 | Bevy没有提供完整引擎级replay authority，不能把外部rand默认算法当持久格式 |
| Fyrox Particle RNG | particle owner保存seed、可reset、可Reflect/Visitor集成 | Zircon Particle私有CPU RNG state未进入durable snapshot且GPU另算 | `StdRng`默认算法不应在未固定版本时成为长期wire合同 |
| Unity Graphics/VFX | start seed、resetSeedOnPlay、Reinit、Timeline scrub/prewarm；shader random helpers显式分层 | Zircon没有产品seed/reseed/scrub控制，CPU/GPU随机语义未协商 | Unity VFX局部控制面不能替代World/replay transaction |

本报告追求的是比参考更严格的可复现与事务边界，不是复制它们的具体算法。性能“优于 Unreal”只能在相同内容、画质、硬件、线程和 replay/checkpoint workload 下，用原始 trace建立。

## 9. 目标架构

```text
Launch / Replay Admission
  ProjectIdentity + BuildSet + RandomProfile + SeedProvenance
                         |
                         v
SimulationSnapshotCoordinator
  close admission -> await step/task terminal -> freeze generation
       |                  |                   |
       v                  v                   v
 World snapshot     Clock/Schedule       Random generation
 Input/Effect fence                        snapshot
       `---------------------+------------------'
                             v
                 SimulationCheckpointArtifact
                    digest + bounded encoding

FixedStepTransaction
  `- RandomScopeIssuer(system/world/entity/tick)
       `- StagedRandomLease (local lock-free draws)
            |- commit -> publish progress with step
            `- abort/drop -> discard progress

Consumer adapters
  AI compiled selector | Particle CPU | GPU counter profile | Script API
  all consume versioned profiles/capabilities, no private authority
```

## 10. 重构里程碑

### M0：关闭继承的 source/Cargo P0

- 原子纳入 `zr_contracts`、runtime random、framework/handle/tests与 manifest/lockfile接线。
- clean checkout 运行 metadata、contracts/kernel tests和source fingerprint guard。
- 在 M0 关闭前，不继续把更多 product consumer 接到不可复现源码。

### M1：封印 profile 与 identity issuer

- 定义 `RandomProfileId`（generator/derivation/key schema/distribution policy）并绑定 BuildSet。
- 建立 World/System/Entity authority 到 `RandomStreamCapability` 的不可伪造签发链。
- 删除 production consumer 构造 raw key 后直接 acquire 的能力。

### M2：加入 fixed-step transaction

- 引入 staged lease；Drop 默认 abort，step commit 批量发布 progress。
- 明确定义 immediate/non-simulation scope并隔离 API。
- 以失败、panic、retry、parallel winner、world replacement证明 clock/world/RNG一致。

### M3：完整 checkpoint admission 与 bounded codec

- 建立 `SimulationCheckpointManifest` 和 snapshot coordinator。
- random decode 在分配前执行 count/bytes/depth budget，增量校验 order和profile。
- 移除公开 partial Runtime restore；只保留内部 assembly。

### M4：消费者硬切

- AI 使用 compiled weighted table + runtime-issued scoped stream + durable node random state。
- Particle CPU 接 shared distribution/profile；GPU 使用明确 counter/hash profile并给出parity policy。
- Script/Plugin ABI只能消费 capability，不能持有/reseed RandomService。

### M5：产品控制与观测

- App/Host/Interface接入 seed provenance、replay checkpoint和capability negotiation。
- Editor/VFX提供 seed/reseed-on-play/reset/reinit/scrub/prewarm，但调用同一 Runtime operation。
- 增加 bounded diagnostics、trace、reject receipts和隐私策略。

### M6：性能结构

- 将 snapshot freeze 与 canonical encode分离；评估 generation swap/immutable snapshot。
- 为 lifecycle eviction增加 deterministic secondary index或shard。
- 对 acquire/release/draw/checkpoint/restore在 1/64/1,024/65,536 streams 下记录 p50/p95/p99、lock wait、allocations、bytes和峰值内存。

### M7：资格矩阵

- deterministic dual-run、save/load/seek、failure/retry、hitch/suspend、cross-platform/toolchain、CPU/GPU shader vectors、fuzz/malformed/oversize、fault/soak。
- 同场景对比参考引擎，发布 raw trace和配置；没有这些证据时不声称性能或表现更优。

## 11. 资格门

| Gate | 当前 | 关闭条件 |
|---|---|---|
| G01 clean checkout 包含当前 contracts/kernel/handle能力 | Fail | 原子 source integration + clean clone |
| G02 workspace/runtime Cargo graph解析 `zr_contracts` | Fail | metadata/check/test通过 |
| G03 algorithm/sequence/state基本vector稳定 | Partial | 当前unit vector存在；补cross-language/platform/BuildSet |
| G04 derivation/key/distribution profile显式版本化 | Fail | profile ID + golden + migration/reject |
| G05 同key唯一mutable lease | Pass | 保持并用并发test守护 |
| G06 lease内draw无registry lock/hash/allocation | Pass | source guard + profile持续通过 |
| G07 retained stream数量有上限 | Pass | 保持capacity fail-close |
| G08 checkpoint canonical order/algorithm guard | Pass | 保持contracts tests |
| G09 wire decode在分配前有count/bytes/depth预算 | Fail | bounded visitor/admission |
| G10 checkpoint绑定Project/BuildSet/world/clock/schedule | Fail | composite manifest admission |
| G11 snapshot与schedule/task/world具有同一quiescence barrier | Fail | coordinator E2E |
| G12 fixed abort/retry不推进random state | Fail | staged lease transaction tests |
| G13 AI无DefaultHasher/tick seed旁路 | Fail | Runtime08F hard cut |
| G14 Particle CPU消费shared profile | Fail | private LCG删除 |
| G15 Particle GPU有算法ID/parity或显式不等价 | Fail | shader profile/golden |
| G16 App/Host/Interface携带seed provenance | Fail | launch/session ABI E2E |
| G17 random/reseed/checkpoint有typed operation/receipt | Fail | product operation owner |
| G18 BuildSet记录random profile与consumer schema | Fail | artifact digest admission |
| G19 replay restore只能从完整manifest进入 | Fail | partial public constructors移除 |
| G20 diagnostics覆盖contention/capacity/checkpoint/reject | Fail | bounded snapshot/trace |
| G21 65,536 stream capture满足产品hitch/memory budget | Fail | 当前p95 41.934 ms，无budget |
| G22 cross-platform/toolchain/fault/fuzz矩阵 | Fail | required CI/managed evidence |
| G23 range/distribution/shape sampling使用统一合同 | Partial | 当前仅3种primitive；补versioned adapters |
| G24 deterministic RNG 与 secure entropy 类型隔离 | Fail | capability/DAG/source guards |

## 12. 首个允许实施的切片

第一切片只能按依赖顺序完成 Runtime153 的 M0，而不是先迁移 AI 或优化 mutex：

1. 冻结 52 个文件的 current source manifest与本报告 fingerprint。
2. 原子纳入 `zr_contracts` manifest/source/tests、Runtime dependency、random kernel、framework/handle和lockfile。
3. 从 clean checkout验证 metadata、contracts、random kernel和 CoreRuntime handle tests。
4. 复算 source fingerprint并证明无 facade-only/test-only遗漏。
5. 完成后再实施 staged lease RED：同一 fixed tick draw 后 failure/abort/retry必须返回相同 draw和draw index。

禁止临时方案：不保留 raw-key acquire兼容入口；不在 Drop中猜测commit；不以 retry-loop掩盖 mixed generation；不把 random checkpoint称为 world/replay checkpoint；不让 AI/Particle复制一份 PCG；不通过增大 Vec上限解决 wire budget；不以本机 microbenchmark宣称优于 Unreal。

## 13. 本轮记录

本轮只新增 current-source review、索引和 coverage 记录；没有修改 Runtime、contracts、AI、Particle、App、Host、Interface、tests、Cargo manifest、lockfile或workflow。未运行动态验证。mixed-era checkpoint failure保留 fixed；RandomService registry slice保留 `source-complete-validation-pending` 的原状态，本报告新增的是其外部工程闭环差距，而不是否定已修复的锁边界。
