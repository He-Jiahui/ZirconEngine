---
date: 2026-08-16
related_plan: docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
fixing_plan: docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
shared_acceptance_plan: docs/plans/zircon_runtime/runtime/06-module-plugin-hot-reload.md
doc_type: structural-performance-research
status: architecture_review_complete_runtime08_handoff_pending
coordination_owner: docs/plans/zircon_runtime/frameworks/01
related_code:
  - zircon_runtime/src/core/framework/scene/mod.rs
  - zircon_runtime/src/scene/module/default_level_manager.rs
  - zircon_runtime/src/scene/module/level_manager_lifecycle.rs
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_runtime/src/scene/level_system.rs
  - zircon_runtime/src/scene/runtime_extension/mod.rs
  - zircon_runtime/src/plugin/extension_registry/apply_to_world.rs
  - zircon_runtime/src/plugin/extension_registry/register/runtime_scene_system_registration.rs
  - zircon_plugins/plugin_sdk/src/registration.rs
  - zircon_plugins/ai/runtime/src/manager/state.rs
  - zircon_plugins/animation/runtime/src/manager.rs
  - zircon_plugins/physics/runtime/src/manager.rs
  - zircon_plugins/sound/runtime/src/engine/state/storage.rs
references:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/World.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/World.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimNotifyQueue.cpp
  - dev/bevy/crates/bevy_ecs/src/message/messages.rs
  - dev/Fyrox/fyrox-animation/src/machine/event.rs
tests:
  - world create rollback and exact extension-generation retention
  - level replacement rollback and old-world state retirement
  - reverse-order world retirement and stale LevelSystem rejection
  - AI, Animation, Physics, and Sound world-state cleanup
  - plugin unload quiescence while a live world retains owner callbacks
  - managed Windows world-churn latency, RSS, lock, I/O, and available energy evidence
---

# World Retirement Lifecycle Architecture Review

## Status

本记录是 Frameworks01 对 Scene/Animation open failure 的结构性复核产物，不是 Runtime08 实现完成记录，
也不是性能优化完成记录。current source、跨模块 owner、Unreal 主参考以及 Bevy/Fyrox 辅助参考已经复核；
实现尚未开始，原因是 Runtime08 当前 primary session
`runtime08-world-query-current-source-recovery-r2-20260815` 仍处于 `resolving_failure`，immutable scope 只有
`zircon_runtime/src/scene/world/query.rs`。在该 WIP 终态或 coordinator 批准 scope rotation 前，不绕过并发约束
创建第二个 Runtime08 primary，也不把修复塞入 Frameworks01、已完成的 Runtime05 或 Animation 局部队列。

UI12 报告的三条 IBL `atomic_write` E0432 与本问题无关：`core::resource::io::atomic_write` 仍是 hard-cut 后的
curated public facade，三份 consumer 的 current source 已在正确入口或 durable bundle transaction 上。
不得以恢复 `atomic_file` 公开模块或回退 Shader06 consumer 来换取旧 fingerprint 编译通过。

## Review Result

当前问题不是某个 IK queue、Physics cache 或 AI map 的局部泄漏，而是引擎没有统一的 World owner 生命周期：

- `LevelManager` 只有 create/query/load/save，没有 destroy/retire；
- `DefaultLevelManager` 永久保留 `HashMap<WorldHandle, LevelSystem>`，handle 只增不减；
- `LevelLifecycleState::Unloaded` 只是可任意写入的标签，`LevelSystem` clone 仍能访问和替换 World；
- `WorldRuntimeExtensionPlan` 只有 apply，没有 partial-create rollback、replace 或 retire；
- `WorldDriver` 每次只抓取最新 plan snapshot 应用，Level 没有保存自己实际应用的 generation；
- `replace_world_and_reset_runtime_state` 直接换入新 World，没有重新安装 runtime extensions；
- AI、Animation、Physics 和 Sound 在 World 外持有按 `WorldHandle` 分区的状态，没有统一的退役触发器；
- native plugin revoke/unload 可以删除未来 registration，但 live World 已安装的 callback/system 没有 owner
  provenance 与 quiescence barrier。

静态 inventory 在 2026-08-16 current source 上记录到 25 处 World-keyed map declaration，分布在 11 个 Rust
文件，其中 9 个是 production owner。这个数字只是当前显式 `HashMap`/tuple-key 形态的下界；World 内资源、
runtime system closure、audio source 反向引用和 native plugin code lifetime 还需要通过 lifecycle contract 统一
裁决，不能用一个定容量 LRU 隐藏。

## Reference-Engine Decision

### Unreal primary

Unreal `UWorld::DestroyWorld` 和 `CleanupWorld` 把 World 清理作为显式、平衡的 owner lifecycle：先停止/清理
World-owned activity，执行 cleanup，通知 Engine 世界已销毁，最后解除 root；`UWorld` 同时保留 initialized、
cleaning 和 cleaned 状态，并发布 `OnWorldCleanup`、`OnPostWorldCleanup`、`OnPreWorldFinishDestroy` 等边界。
Zircon 采用其“Init 必须由 Cleanup 平衡、World owner 统一广播生命周期、销毁后拒绝继续使用”的原则，
不复制 UObject GC、delegate ABI 或 streaming 实现。

Unreal 的 animation notify queue 每帧由明确 owner reset；Bevy message 双缓冲要求 owner 定期 update，文档明确
不 update 会持续增长；Fyrox animation event queue 默认有界并提供 pop/take。三者共同证明有界队列只能限制
单 World 的峰值，不能代替 World 终态和 owner cleanup。

## Required Ownership Model

### Single lifecycle authority

`DefaultLevelManager` 是 WorldHandle admission 和 terminal transition 的唯一 authority。公共合同硬切为：

- `create_default_level_handle` / `load_level_asset` 创建并发布一个 active Level；
- `replace_level` 通过 manager-owned transition 替换 World；
- `destroy_level` 关闭 admission、退役 domain state 并移除 handle；
- runtime shutdown 必须 drain 所有 active Level，再释放 Scene manager/driver；
- `LevelSystem::replace` 和任意公开 `set_lifecycle` 删除，不保留绕过 manager 的兼容路径。

WorldHandle 在 extension install 前分配。创建失败允许留下不可复用的单调 handle gap，但不得把 Level 插入
manager map；这样 apply、rollback 和 diagnostic 都使用同一个稳定 handle，避免无身份的半初始化副作用。

### Exact applied generation

`WorldDriver` 发布 immutable `WorldRuntimeExtensionPlanGeneration { generation, plan }`。创建 Level 时只抓一次
短锁 snapshot，按确定顺序 apply，并把 exact snapshot 存入 Level-owned
`AppliedWorldRuntimeExtensionPlan`。replace 和 destroy 必须使用该 Level 实际应用的 snapshot，禁止读取 latest
plan 猜测 cleanup generation。

每条 `WorldRuntimeExtensionRegistration` 必须保存：

- stable key；
- plugin/module owner provenance；
- install callback；
- infallible rollback/retire callback；
- callback 所属 generation 的 live reference accounting。

install 的外部副作用必须受返回 lease/token 约束，或者严格限制在 staged World 内。rollback/retire 不得因为
单个 domain 的诊断而中止后续清理；错误记录到 bounded diagnostic，释放动作继续执行。动态库卸载只能在该
owner 的 live callback/system reference 归零后发生。

Sound 没有 runtime scene system，但同样持有 World-keyed journal 和 World-bound source，因此 retire API 不能
附着在 `runtime_scene_system` builder 上。Plugin SDK 需要独立的 `world_lifecycle` registration；runtime system、
resource 和 event registration 继续投影为同一个 general World extension plan。

## Lifecycle State Machine

| 状态 | 可见性 | 允许操作 | 终态保证 |
|---|---|---|---|
| `Creating` | 不在 manager map | 对 staged World 顺序 install | 失败时逆序 rollback，handle 永不发布 |
| `Active` | handle 可查询 | tick、query、save、manager-owned replace/destroy | 每次访问在取得 World lane 后确认仍 active |
| `Replacing` | handle 暂停新 tick/admission | 准备新 World、应用 exact plan、清理旧 epoch、原子 swap | 新 install 失败则旧 World 保持 active |
| `Retiring` | handle 已从新 lookup 关闭 | 等待当前 World lane，逆序 retire | 所有 domain callback 都被调用一次 |
| `Retired` | handle 不存在 | 只允许读取 terminal diagnostic | stale `LevelSystem` clone 返回 typed retired error |

实现不得只在锁前检查状态。Level access 必须先取得 per-Level World lane，再验证 lifecycle/epoch；否则等待锁的
旧操作会在 retire 完成后重新进入。manager map lock 只用于 transition/admission 和取出 Level clone，插件
callback、World drop、audio stop、Physics cleanup 等全部在全局 manager lock 外执行。

### Create

1. 分配 monotonic WorldHandle，抓取 exact extension plan snapshot；
2. 对 staged World 依次 install，并记录 applied registration count/lease；
3. 任一步失败，逆序 rollback 已应用项，drop staged World，返回 typed error；
4. 构造 Level slot，状态从 `Creating` 变为 `Active`；
5. 最后在 manager map 中发布 handle，发布前外部不可观察。

### Replace

1. 由 manager 把 Level 从 `Active` CAS 到 `Replacing`，拒绝并发 replace/destroy/tick admission；
2. 在旧 World lane 外准备 incoming World，并应用该 Level 的 exact extension generation；
3. incoming install 失败时逆序 rollback incoming，恢复 `Active`，旧 World/epoch 完全不变；
4. 取得旧 World lane，逆序 retire 旧 replacement epoch，递增 epoch，原子 swap；
5. 重接 subscription/frame state，发布新 epoch，再回到 `Active`；
6. plan upgrade 必须是独立显式 API，不得在普通 replace 时静默切到 latest generation。

### Retire

1. manager map 中把 Level 从 `Active` CAS 到 `Retiring` 并关闭新 lookup；
2. 在 map lock 外取得 World lane，等待已入场操作退出；
3. 按 apply 的逆序执行 exact generation retire callback；
4. 清空 World-local events/subscriptions/runtime systems，drop World；
5. 标记 `Retired`，记录 bounded diagnostic；重复 destroy 返回 typed already-retired/unknown result；
6. runtime shutdown 对所有 handle 执行相同路径，不新增一套 Drop-only cleanup。

## Domain Convergence

| Domain | 当前结构问题 | hard-cut 目标 |
|---|---|---|
| Scene | manager map 永不 remove，stale clone 可继续访问 | manager-owned destroy、terminal gate、exact applied plan |
| AI | 5 张 `(WorldHandle, EntityId)` flat map；按 World retain 会扫描全部 agent | `HashMap<WorldHandle, AiWorldRuntimeState>`，一次 remove 后 O(该 World agent) drop |
| Animation | Runtime 与 plugin 各有一张 IK world queue；queue 有界但 world entry 不退役 | 收敛唯一产品 owner；epoch-aware O(1) remove，pipeline closure 随 World drop |
| Physics | 多张 world map 和多把锁，已有散落 `clear_world` 但无 lifecycle trigger | 短锁 world registry + per-world state owner，一次 retire 清 command/backend/contact/trigger |
| Sound | gameplay journal 有界但 world entry 不移除；source 可反向引用 World | per-world journal/source index；retire 停止或解除本 World source 并 O(本 World source) drop |
| Navigation/Net/VM | 注册 runtime system，当前显式 WorldHandle map inventory 未命中 | 证明状态仅由 World/system closure 持有，或补同一 lifecycle registration，禁止默认忽略 |

AI、Physics 和 Sound 的 owner 调整是算法修复，不是命名重构。目标是 retire cost 与被退役 World 的状态量
相关，而不是与进程历史创建过的 World 或全部 active entity/source 数相关。独立 World 的 tick/cleanup 不得被
一把长期全局锁串行化；registry lock 只解析 `WorldHandle -> Arc<PerWorldState>`，domain 工作走 per-world lane。

## Complexity And Memory Target

设 extension registration 数为 `R`，active World 数为 `W`，被退役 World 的 AI agent、Physics body/contact、
Sound source/journal entry 和 Animation command 总量为 `S_w`。

| 操作 | 当前上界/风险 | 目标 |
|---|---:|---:|
| create | `O(R)`，失败无 rollback | `O(R)`，失败逆序 rollback |
| replace | 未重装 extension，外部 state 跨 epoch | `O(R + S_w)`，旧 World 保持到 incoming 成功 |
| retire | 无统一入口，resident state 随历史增长 | `O(R + S_w)` |
| handle lookup | 平均 `O(1)`，但永不缩减 | 保持平均 `O(1)`，retire 后 entry 为 0 |
| AI world cleanup | flat-map retain 可退化为 `O(sum(S_all_worlds))` | map remove 平均 `O(1)` + drop `O(S_w)` |
| independent World work | 多 domain 全局 mutex 可能串行 | 短 registry lock + per-world lane |

不得用按 World 数量设置 LRU/TTL、周期性全表 sweep 或复用 WorldHandle 作为修复。前两者掩盖 owner 缺失并把
成本移到不可预测的帧，后者会让 stale async result 命中新 World。

## Measurement Before Optimization

生命周期正确性实现完成后，先增加低基数 typed observation，不改变 domain 算法：active/creating/replacing/
retiring World 数、每 domain per-world state 数、rollback/retire callback count、callback elapsed、等待 World lane
时间、stale admission rejection、plugin unload veto 和 cleanup diagnostic。disabled 状态不得分配、遍历 map 或
获取 profiler lock。

Windows 受管验证全部使用 D 盘 target/artifact，保存 source fingerprint，并覆盖：

1. 1、8、64 个并行 active World；
2. 1、100、1,000、10,000 次 create/replace/destroy churn；
3. 空 World，以及带 1/100/10,000 agent/body/source 的 scale fixture；
4. install 第 1/中间/最后 registration 失败，replace 失败，retire diagnostic，plugin unload race；
5. WPR/ETW CPU sampling、context switch/lock wait、allocation/RSS、可用的 package energy 与平均功率。

每组丢弃 warm-up，保存至少 31 个 settled warm sample，报告 median、p95、p99、MAD、peak working set、
allocation count/bytes 和 lock wait。10,000 次完整 churn 后，所有 domain 的 live World entry 必须精确回到
baseline，RSS 只允许 allocator 已解释的稳定 envelope，不能随历史 World 数呈正斜率。没有同机、同 fixture、
同采样方法的 Unreal/Bevy/Fyrox 数据时，不声称功耗或耗时“接近其它引擎”，也不声称算法达到经验最优；
只报告可复现的复杂度、回归与 profile 证据。

## Acceptance Matrix

- create 第 N 个 extension 失败：前 N-1 个按逆序 rollback，manager map 无 handle，domain state 为 baseline；
- replace incoming install 失败：旧 World generation/epoch/content/extension generation 完全不变；
- replace 成功：新 World 具备全部 extension，AI/Sound/Physics 旧 epoch 状态为 0；
- destroy 与 tick/query 并发：已入场操作先完成，destroy 后的新操作返回 typed retired/unknown error；
- destroy callback 中重入 driver/registry：不持有 manager map lock，不死锁；
- plan 发布新 generation 后销毁旧 Level：只调用旧 Level 保存的 exact generation callback；
- plugin owner revoke/unload：存在 live callback/system reference 时明确 veto，归零后才允许 unload；
- runtime shutdown：所有 active Level 走同一 retire path，不能只依赖 `Drop<DefaultLevelManager>`；
- source guard：Scene 不直接依赖可选 AI/Animation/Physics/Sound manager，domain 通过 neutral lifecycle callback
  注册清理；
- performance：retire 时间随 `S_w` 线性，不随历史 World 数或其它 World 状态增长。

## Ordered Milestones

- [x] W0-A：复核 Frameworks01 三份 open failure、Scene/World/extension/plugin current source。
- [x] W0-B：以 Unreal `UWorld` cleanup 为主参考，Bevy/Fyrox bounded queue 为辅助，确定问题不是局部 LRU。
- [x] W0-C：完成 25 处 keyed-map/11 文件静态 inventory、owner graph、状态机、复杂度与性能验证方案。
- [ ] W1-A：Runtime08 WIP 释放后，在 Runtime08 child plan 建立 canonical failure handoff，并由 coordinator
  import；Frameworks01 只保留 origin evidence。
- [ ] W1-B：Runtime08 实现 LevelManager destroy/replace、terminal admission gate、exact applied generation、
  partial-create rollback 和 shutdown drain。
- [ ] W1-C：Runtime06/Plugin SDK 增加 owner-provenance lifecycle registration 与 live-generation quiescence
  veto；不恢复旧 plugin API。
- [ ] W2-A：AI/Animation/Physics/Sound 按 per-world owner 收敛 cleanup；Navigation/Net/VM 完成有状态/无状态证明。
- [ ] W2-B：执行 focused、upward、feature-combination、plugin unload 与 product churn tests。
- [ ] W3-A：取得 Windows managed WPR/ETW/RSS/可用功耗 baseline 与 hard-cut 复测；profile 证明瓶颈前不做
  batching、parallelism 或 lock-free 细节优化。
- [ ] Acceptance：Runtime08/Runtime06 owner 验收、独立 review、managed validation ticket、计划状态、
  coordinator milestone commit 和 service-managed WeCom 完成后，才把该结构性问题记为关闭。

## Coordination Boundary

本记录由 Frameworks01 写入，是 origin analysis，不授权 Frameworks01 修改 Runtime08、Runtime06 或插件源码。
Runtime05 已完成且明确把后续 World/data alignment 交给 Runtime08，不得重新打开。当前 Runtime08 WIP 未释放时，
继续完成不需要 Cargo 的 inventory、reference review 和 acceptance design；不把验收队列作为唯一工作项。
任何实现 successor 必须一次性注册完整 immutable scope、先 claim/attribute 再编辑，并保持两份冻结的 Editor
mixed blob 不变。
