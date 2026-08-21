---
related_code:
  - examples/woc/scripts/woc_game/src/main.zr
  - examples/woc/scripts/woc_game/src/world/state.zr
  - examples/woc/scripts/woc_game/src/instances
  - examples/woc/scripts/woc_game/src/instances/delve_state.zr
  - examples/woc/scripts/woc_game/src/instances/dungeon_state.zr
  - examples/woc/scripts/woc_game/src/instances/dungeon_reset_state.zr
  - examples/woc/scripts/woc_game/src/instances/pet_state.zr
  - examples/woc/scripts/woc_game/src/instances/pet_follow_rules.zr
  - examples/woc/scripts/woc_game/src/instances/pet_target_rules.zr
  - examples/woc/scripts/woc_game/src/instances/pet_water_jet_state.zr
  - examples/woc/scripts/woc_game/src/instances/emberkin_ranged_attack_rules.zr
  - examples/woc/scripts/woc_game/src/instances/delve_companion_content.zr
  - examples/woc/scripts/woc_game/src/instances/delve_companion_decisions.zr
  - examples/woc/scripts/woc_game/src/instances/delve_companion_target_rules.zr
  - examples/woc/scripts/woc_game/src/instances/delve_companion_upgrade_rules.zr
  - examples/woc/scripts/woc_game/src/instances/delve_admission_rules.zr
  - examples/woc/scripts/woc_game/src/instances/delve_affix_selection.zr
  - examples/woc/scripts/woc_game/src/instances/delve_module_content.zr
  - examples/woc/scripts/woc_game/src/instances/delve_lockpick_rules.zr
  - examples/woc/scripts/woc_game/src/instances/delve_lockpick_session.zr
  - examples/woc/scripts/woc_game/src/instances/delve_lockpick_projection.zr
  - examples/woc/scripts/woc_game/src/instances/drowned_litany_boss_rules.zr
  - examples/woc/scripts/woc_game/src/instances/drowned_litany_rite_rules.zr
  - examples/woc/scripts/woc_game/src/instances/drowned_litany_loot_rules.zr
  - examples/woc/scripts/woc_game/src/instances/drowned_litany_room_rules.zr
  - examples/woc/scripts/woc_game/src/instances/m7_scenario_matrix.zr
  - examples/woc/scripts/woc_game/src/world/world_collision_router.zr
  - examples/woc/scripts/woc_game/src/world/instance_collision_content.zr
  - examples/woc/scripts/woc_game/src/world/instance_collision_routing.zr
  - examples/woc/scripts/woc_game/src/world/instance_collision_static.zr
  - examples/woc/scripts/woc_game/src/world/instance_line_of_sight.zr
  - examples/woc/scripts/woc_game/src/world/delve_collision_content.zr
  - examples/woc/scripts/woc_game/src/world/delve_collision_routing.zr
  - examples/woc/scripts/woc_game/src/world/delve_collision_static.zr
  - examples/woc/scripts/woc_game/src/world/delve_collision_sweep.zr
  - examples/woc/scripts/woc_game/src/world/delve_run_layout.zr
  - examples/woc/scripts/woc_game/src/world/dungeon_difficulty_state.zr
  - examples/woc/scripts/woc_game/src/protocol/commands.zr
  - examples/woc/scripts/woc_game/src/protocol/command_payloads.zr
  - examples/woc/native/apps/woc_client/src/input/intent.rs
  - examples/woc/native/apps/woc_client/src/input/hud_routes.rs
  - examples/woc/native/crates/woc_protocol/src/command_payload.rs
  - examples/woc/reference/command_catalog.json
  - examples/woc/reference/parity_scenarios.json
  - examples/woc/reference/current-head/command_catalog.json
  - examples/woc/reference/current-head/parity_scenarios.json
tests:
  - examples/woc/scripts/woc_game/woc_m7_delve_tests.zrp
  - examples/woc/scripts/woc_game/woc_m7_dungeon_tests.zrp
  - examples/woc/scripts/woc_game/woc_m7_dungeon_reset_state_tests.zrp
  - examples/woc/scripts/woc_game/woc_m7_pet_tests.zrp
  - examples/woc/scripts/woc_game/woc_m7_pet_water_jet_state_tests.zrp
  - examples/woc/scripts/woc_game/woc_m7_delve_lockpick_rules_tests.zrp
  - examples/woc/scripts/woc_game/woc_m7_delve_lockpick_session_tests.zrp
  - examples/woc/scripts/woc_game/woc_m7_delve_lockpick_projection_tests.zrp
  - examples/woc/scripts/woc_game/woc_m7_delve_module_content_tests.zrp
  - examples/woc/scripts/woc_game/woc_m7_drowned_litany_boss_rules_tests.zrp
  - examples/woc/scripts/woc_game/woc_m7_scenario_matrix_tests.zrp
  - examples/woc/native/Cargo.toml
  - examples/woc/tools/m7_dungeon_state_source_check.mjs
plan_sources:
  - docs/plans/optimize/zircon_app/03-woc-product-role-host-zrvm-transaction-state-client-server-integration-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/08d-navigation-runtime-review.md
  - docs/plans/optimize/zircon_runtime/08e-network-runtime-review.md
  - docs/plans/optimize/zircon_runtime/12-woc-zrvm-package-kernel-world-state-schedule-serialization-runtime-review.md
  - docs/plans/optimize/zircon_runtime/13-woc-combat-casting-effect-aura-damage-threat-death-runtime-review.md
  - docs/plans/optimize/zircon_runtime/14-woc-progression-inventory-item-economy-crafting-quest-talent-runtime-review.md
  - docs/plans/optimize/zircon_runtime/15-woc-social-identity-party-raid-chat-duel-arena-matchmaking-minigame-runtime-review.md
  - docs/plans/optimize/zircon_tooling/05-woc-content-codegen-build-scripts-generated-artifact-incremental-review.md
  - docs/plans/optimize/zircon_tooling/10-test-architecture-partition-selection-isolation-fixture-flake-results-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/World.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/NetDriver.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/GameFramework/GameModeBase.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/GameFramework/GameStateBase.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/GameFramework/PlayerState.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/GameFramework/Actor.h
  - dev/godot/modules/multiplayer/scene_multiplayer.h
  - dev/godot/modules/multiplayer/multiplayer_spawner.h
  - dev/godot/modules/multiplayer/multiplayer_synchronizer.h
  - dev/bevy/crates/bevy_ecs/src/world/mod.rs
  - dev/bevy/crates/bevy_ecs/src/entity/mod.rs
  - dev/Fyrox/fyrox-impl/src/scene/mod.rs
  - dev/Fyrox/fyrox-impl/src/engine/mod.rs
  - dev/Fyrox/fyrox-core/src/pool/mod.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraph.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/IRenderGraphBuilder.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 16 · WOC Instance、Dungeon、Delve、Pet、Companion、Lockout、Reset 与 Collision Runtime 工程化差距

## 1. 结论

WOC 的实例域不是空白。`src/instances` 共有 103 个 `.zr`、10,993 行、410,346 bytes；去掉 46 个 test main 后，仍有 57 个候选生产/共享模块、9,939 行、359,339 bytes。目录里已经保存 Delve admission、layout、affix、module、boss、lockpick、reward、reset、pet/companion AI 等大量 current/historical source 投影，其中 `delve_state.zr` 甚至提供一条 1,409 行的端到端标量场景模型。这些代码有迁移与回归价值，不能用“尚未写功能”概括。

但物理代码量与产品执行事实几乎完全分离。从 `main.zr` 静态 import closure 只能到达 5/57 个非 test-main 模块、484 行；其中 `heroic_dungeon_tuning` 只被 package `contractTest()` 消费。普通 `WorldState` 产品段实际只依赖 `delve_companion_content`、`emberkin_ranged_attack_rules`、`pet_follow_rules`、`pet_target_rules` 四个叶模块，共 350 行、12,740 bytes；其余 53 个模块、9,589 行、346,599 bytes 不参与正常 authority。完整 `DelveState`、`DungeonState`、`DungeonResetState`、`PetState`、lockpick session、Drowned Litany 和 M7 scenario matrix 都是断线候选 owner。

产品中的“实例”也不是隔离 world。当前 runtime 把标准地下城、arena、Yumi 作为同一个 `WorldState` 内不同 X/Z 坐标带，`instance_collision_routing.zr` 依据 X 区间和最近 Z slot 选静态 layout；标准实例带里无法映射的 X 还会退回 index-0 crypt。active Delve 与 far-east 区域因布局未移植直接抛错。没有 `InstanceId`、allocation、claim、admission lease、participant membership、独立 clock/RNG/schedule、server placement、transfer ticket、disconnect/reconnect 或 shutdown owner，因此坐标碰撞路由不能等价于工程级 instance runtime。

公开能力面与 reducer 的差距是直接阻断。current-head catalog 公开 11 个 `IWorldPet`、4 个 `IWorldDungeons`、10 个 `IWorldDelves` 命令；WorldState 缺失 pet abandon/rename/feed/heal、dungeon enter/leave，以及 Delve enter/leave/interact/lockpick 三命令/chest collect/rite choose，共 14 个命令，都会落入 `authoritative command reducer is not implemented`。七个已路由 pet 命令又只服务离线 Hunter/Warlock 的 Emberkin/Imp：taunt 与 Water Jet 是 no-op，两个 autocast 命令强制写回 false。Delve 产品路径只剩商店购买与伙伴升级，地下城只剩 difficulty 与 heroic vendor，并不存在可进入、可运行、可退出、可恢复的地下城或 Delve。

证据链也不能支撑已交付结论。46 个 test main 中只有 39 个有 `.zrp` 接管，未接管的七个正好包含 lockpick state integration、三项交易测试和宠物规则消费者；38 个 M7 package 声明的 `bin-m7-*` 目录全部不存在。current-head parity catalog 的 11 个 pet/delve/dungeon `woc_owner` 文件也全部不存在。实例目录只有 9 个文件声明 40 字符 source commit，且同时混用 `7c10...` 与 `5ef9...`；其余 95 个文件没有同等 provenance。当前实现只能作为规则素材与 fixture 基线，不能被 catalog、文档或 UI 标记为工程级 supported runtime。

## 2. 审查范围与执行拓扑

### 2.1 物理清单

| 项目 | 结果 |
|---|---:|
| `src/instances` 全部 `.zr` | 103 文件 / 10,993 行 / 410,346 bytes |
| 非 `*_test_main.zr` | 57 文件 / 9,939 行 / 359,339 bytes |
| test main | 46 文件 / 1,054 行 / 51,007 bytes |
| `main.zr` 静态 import closure | 5 文件 / 484 行 / 18,063 bytes |
| 普通 WorldState 产品段可达 | 4 文件 / 350 行 / 12,740 bytes |
| 普通产品段不可达非 test-main | 53 文件 / 9,589 行 / 346,599 bytes |
| public/all class、public var/function | 6 / 14 / 264 / 532 |
| `throw` / `new container.Array` / `while` | 443 / 52 / 67 |

最大的候选模块是 1,493 行 `delve_module_content`、1,409 行 `delve_state`、739 行 `pet_state`、494 行 `delve_lockpick_rules`、443 行 `dungeon_reset_state`、390 行 `dungeon_state` 和 358 行 `drowned_litany_boss_rules`。20/57 个非 test-main 文件明确写有 future/later/scalar/fixture/unavailable 等延期边界；这类注释诚实揭示了缺口，但不能替代 owner、依赖 ID 和 hard-cut 计划。

### 2.2 产品可达集合

WorldState 只 import 四个实例叶模块：Emberkin ranged 规则有 13 个调用点、pet target 有 5 个、pet follow 有 3 个、Delve companion content 有 26 个。它们确实参与产品逻辑，但只是小型规则表或 helper。`heroic_dungeon_tuning` 进入 package closure，却没有 WorldState 调用点，只为 package contract test 提供断言。

相关 `src/world` 非 test-main 文件还有 17 个、16,450 行、428,372 bytes。产品图只接通 `dungeon_difficulty_state`、`instance_collision_content/routing/static` 四个文件、4,317 行；10,244 行 Delve collision content、Delve layout/collision/LOS、dungeon door/entrance 和 generic instance LOS 等 13 个模块未进入正常 world route。这里同样存在“内容投影已写”和“实例系统已运行”的断层。

### 2.3 WorldState 实际 authority

当前 `WorldState` 有 534 个 `pub var`。实例域只有 pet mode/taunt/autocast/path cooldown 九列、Delve marks/shop clears/companion rank 五列、personal/party dungeon difficulty 两列和 corpse instance ID 一列；没有 instance/claim/run/member/lockout/transfer/session collection。`advanceState` 每 tick 调用 Emberkin pet step，却没有 dungeon、Delve、lockpick 或 instance lifecycle step。

Emberkin step 先遍历所有 entity 找 pet，再对每只 pet 全 entity 扫 assist target，并通过线性 entity/threat lookup；移动只执行 direct open-ground leg。源码注释明确把 cached A*、obstacle routing、pull scan、forced recovery 和 mutable grid/path state延后。它是特定离线演示宠物闭包，不是通用 PetRuntime。

### 2.4 Catalog、client 与 reducer 差异

| Facet | Catalog | WorldState 可达结果 | 缺失/虚假部分 |
|---|---:|---|---|
| `IWorldPet` | 11 | revive/attack/mode 与四个 Emberkin compatibility route | abandon/rename/feed/heal 未实现；taunt/Water Jet no-op；autocast 强制 false |
| `IWorldDungeons` | 4 | set difficulty、heroic vendor | enter/leave 未实现；无 claim/instance lifecycle |
| `IWorldDelves` | 10 | shop buy、companion upgrade | enter/leave/interact/lockpick engage/action/abort/chest collect/rite choose 未实现 |

Native client `intent.rs` 和 protocol payload 已能构造上述命令；`hud_routes.rs` 还把 lockpick engage/action/abort 转成 `HudHostEffect::RequestLockpick*`。但 client source 中没有 authoritative lockpick session/projection consumer，WorldState 也没有 reducer。因此按钮和 typed payload 只能证明输入表面存在，不能证明 state machine、结果、隐私或重连语义存在。

### 2.5 候选状态的边界

`DelveState` 有 77 个 public vars、18 个 imports、46 个 public functions，并把 run、layout、collision、boss、companion、lockpick、inventory、lore、shop、daily、RNG observer 和 scenario tests 聚在一个私有 class。默认 owner/chest/run seed 是固定标量，事件是八组 `Array<int>`，module/mob/object 主要以 bool/count 表示。它适合作为源行为 oracle，不适合作为多 run、多 party、跨进程 authority。

`DungeonState` 以固定五元素数组、默认两人 party、整数 `claimedPartyKey`、mob/object count 和单个 raid lockout 模拟 Hollow Crypt/Nythraxis；`DungeonResetState` 已尝试 durable owner key、claim、lock 和“先验证后替换”，但仍是不可达内存平行数组。其 reset 在进程内直接改 claim ID/difficulty 并依次改 lock 列，不具备数据库事务、CAS、journal 或 crash atomicity。

`PetState` 则是另一套 739 行 fixture authority，覆盖 tame/mode/feed/revive/taunt/abandon/stow/restore/demon heal/target/tick，却没有接入 entity lifecycle、persistent pet identity、spellbook、stable、owner transfer 或 persistence。产品 Emberkin path 与候选 PetState 同时存在，形成典型双重 authority。

### 2.6 测试与 provenance

38 个 M7 manifests 全部指向实例 test main，但对应 `bin-m7-*` artifact 目录均不存在；另有七个 test main 没有 manifest。旧 parity catalog 的 source commit 是 `7c10...`，current-head 是 `5ef9...`，两者的 11 个 M7 owner 路径都不存在；`m7_scenario_matrix` 自身仍固定旧版本，而 `dungeon_state` 明确混合旧/current-head 语义。没有 generated BuildSet 证明规则、catalog、golden、VM/backend、snapshot schema 与执行结果同代。

## 3. P0 阻断

| ID | 差距 | 证据与影响 | 必须重构 |
|---|---|---|---|
| INST-P0-001 | 物理规则库与产品 authority 分裂 | 普通产品只达 4/57 模块，53 个候选模块含完整 dungeon/delve/pet/lockpick owner；两侧继续修改会永久漂移 | 建 production/fixture/test manifest，每项 capability 只保留一个产品 owner；候选规则迁入后 hard cut 平行状态 |
| INST-P0-002 | 没有真正的 InstanceWorld、claim 与 transfer 生命周期 | 当前依赖 X/Z 坐标分带和静态 layout，没有 InstanceId、allocator、membership、lease、独立 clock/RNG、placement、reconnect 或 shutdown | 先交付 `InstanceAllocator`、`InstanceWorldRuntime`、`InstanceClaimStore`、`InstanceTransferRuntime`，entity/location 全部 instance-qualified |
| INST-P0-003 | Catalog/client 公开 14 个无 reducer 命令，已路由 pet 又包含 no-op | 4 pet、2 dungeon、8 delve 命令最终抛未实现；taunt/Water Jet 无效果，autocast 强制 false | capability negotiation 先 fail-closed 标 Unsupported；owner、receipt、projection、测试齐全后才重新发布 |
| INST-P0-004 | Dungeon/Delve/lockout/reset 没有 durable identity 与事务边界 | candidate state 使用 int/string/parallel arrays，产品只有 difficulty/marks/rank；无法定义跨 party reform、断线、转服、重启与并发 reset | 用 qualified generation ID、revision/CAS、durable journal、idempotency 和 crash-recovery transaction 重建 claim/run/lockout |
| INST-P0-005 | Delve 核心 lifecycle 与 lockpick/reward 不在产品图 | 产品只有 shop/upgrade 直接改 economy；enter、interact、boss、lockpick、chest、rite、death/recovery 全不可达，client 却暴露 lockpick route | 建唯一 `DelveRuntime`/`EncounterRuntime`/`InstanceRewardTransaction`，从 admission 到 result/reward/projection 全链执行 |
| INST-P0-006 | Parity、artifact 与动态执行不能证明产品语义 | 11 个 parity owner 缺失，7 test main 无 manifest，38 M7 binary dirs 缺失；native/npm lane 均被既有错误阻断 | 生成 owner/artifact inventory，从产品 root 执行 exact replay/fault/save-load/perf；missing/not-run 一律不得计 pass |

## 4. P1 工程化差距

### 4.1 Authority、BuildSet 与 capability

| ID | 差距 | 重构要求 |
|---|---|---|
| INST-P1-001 | 没有 production/fixture/test 模块分类 | 生成 package topology，并把 capability 与唯一 production root 绑定 |
| INST-P1-002 | main closure 把 package `contractTest` 依赖误算成产品依赖 | 分离 runtime entry、self-test entry 和 evidence entry，分别生成 closure |
| INST-P1-003 | 53 个非 test-main 模块不可达却位于生产 source root | 迁入明确 fixture/oracle package，或接入唯一 owner 后删除旧路径 |
| INST-P1-004 | 20 个模块用 future/later 注释代替依赖治理 | 每项延期绑定 owner、gate、计划 ID、blocked contract 和 hard-cut 条件 |
| INST-P1-005 | 103 个文件只有 9 个 commit 声明且混用两个版本 | 每个 generated/ported artifact携 source commit、generator、schema、content digest |
| INST-P1-006 | current-head catalog 与 historical M7 matrix 没有兼容声明 | BuildSet 固定 source revision；跨版本合并必须有 decision record 和 golden diff |
| INST-P1-007 | protocol 枚举存在即被客户端视为可用 | capability manifest 以 reducer+projection+artifact+evidence 同时存在为 Supported |
| INST-P1-008 | 命令成功/拒绝/unsupported 多为 throw、silent return 或 no-op | 所有命令返回 stable typed receipt、reason、revision、retryability 与 effect IDs |
| INST-P1-009 | snapshot、catalog、rules、golden、client 不携统一 BuildSet | 在 handshake、save、replay、artifact metadata 中传递 BuildSet/schema/backend |
| INST-P1-010 | 534-column WorldState 继续吸收实例特例 | 以 owner handle/adapter 取代新增平行列，禁止在 monolith 内补完整 dungeon/delve |
| INST-P1-011 | 候选模块暴露大量自由函数和 mutable object，没有 runtime service contract | 定义 admission/tick/command/checkpoint/shutdown/projection 的 typed owner API |
| INST-P1-012 | combat/social/progression/network 与实例域通过直接字段突变耦合 | 用 typed handles、transaction intent、outbox event 和 projection schema 协作 |

### 4.2 Instance world、allocation、transfer 与 collision

| ID | 差距 | 重构要求 |
|---|---|---|
| INST-P1-013 | 无 `InstanceId`/generation/definition revision | 引入 globally qualified instance handle，区分 template、run、claim 和 generation |
| INST-P1-014 | EntityId、corpse、target 和位置不携 world/instance scope | 所有跨 owner引用使用 `(WorldId, InstanceId, EntityGeneration)` 并 stale fail-closed |
| INST-P1-015 | 无 allocator、capacity、placement 或 server ownership | allocator 根据 ruleset、region、load、party reservation 产生可续租 placement |
| INST-P1-016 | 无 admission token、roster revision 或 access policy | admission lease绑定 principal、party revision、difficulty、lockout、expiry 与 nonce |
| INST-P1-017 | enter/leave 不是原子 world transfer | transfer transaction冻结源状态、验证目标、spawn/restore、commit/rollback 并发 receipt |
| INST-P1-018 | 无 disconnect grace、reconnect 或 duplicate login handling | connection generation与participant lease分离，定义 reclaim/replace/expire 状态机 |
| INST-P1-019 | 无跨进程迁移、drain、crash recovery 或 orphan cleanup | placement lease、checkpoint/journal、fencing token 与 supervisor takeover共同验证 |
| INST-P1-020 | 所有内容共用 WorldState clock、RNG 和 schedule | 每个 instance固定 tick epoch、seed stream、ruleset revision、pause/catch-up budget |
| INST-P1-021 | collision 通过 X coordinate band推断实例 | collision query必须显式接收 instance/world context；坐标只在局部空间有意义 |
| INST-P1-022 | 未映射 standard-band X 静默回退 crypt layout | unknown route fail-closed并产生低基数诊断，禁止错误碰撞另一个 dungeon |
| INST-P1-023 | active Delve/far-east 碰撞在产品移动时抛错 | capability/admission在进入前拒绝未加载 layout，运行中不得走异常控制流 |
| INST-P1-024 | swept collision 按 0.2 单位步进，成本随距离和 collider 数增长 | 使用 broadphase、continuous sweep/TOI、bounded iteration和退化场景基准 |
| INST-P1-025 | 3,868/10,244 行碰撞内容以巨大条件树查询 | 生成 typed packed collider asset、索引/BVH、版本校验和 hot-reload/cook artifact |

### 4.3 Dungeon、claim、reset、lockout 与 reward

| ID | 差距 | 重构要求 |
|---|---|---|
| INST-P1-026 | `enter_dungeon`/`leave_dungeon` 无 reducer | DungeonRuntime 负责 definition lookup、admission、transfer、participant transition |
| INST-P1-027 | difficulty 仅是 personal/party entity bool | difficulty 是 revisioned party/run rule，claim 创建后冻结且变更需重新分配 |
| INST-P1-028 | `heroic_buy` 只是坐标附近 vendor 交易 | vendor 与 dungeon lifecycle 解耦，购买走 Runtime14 transaction 和 typed receipt |
| INST-P1-029 | `DungeonState` 固定两人、五元素数组和单 claim | 用动态 roster、participant record、encounter graph 和多 instance registry |
| INST-P1-030 | mob/object 只是 count，不是有 generation 的实体 | encounter spawn/despawn/loot/corpse必须落 entity lifecycle并可重放 |
| INST-P1-031 | participation 是整数 `enteredBy` 集合 | 记录 principal/character/instance/encounter participation revision和资格证据 |
| INST-P1-032 | `claimedPartyKey` 与 lock owner是裸 int fallback | ClaimId/PartyId/PrincipalId 使用稳定 namespace、generation、issuer和revision |
| INST-P1-033 | `DungeonResetState` 不可达且没有 catalog command | reset capability接入唯一 owner，否则移至 oracle package并对外明确 Unsupported |
| INST-P1-034 | “先验证后改”仍非 crash-atomic | claim replacement、lock inheritance、world cleanup、receipt/outbox做单 durable transaction |
| INST-P1-035 | lock 使用四条平行数组、线性扫描和 `removeAt` | keyed lock store + expiry index + CAS，避免错位、搬移和 O(owner*claim*lock) |
| INST-P1-036 | empty reset只按 bool/count与 300 秒标量 | 由 participant/corpse/loot/encounter leases判空，deadline heap驱动可审计 teardown |
| INST-P1-037 | reward函数只返回 bag/mail提示码 | reward eligibility、roll、inventory/mail fallback、commit与outbox交给幂等 transaction |

### 4.4 Delve、module、boss、lockpick 与 progression

| ID | 差距 | 重构要求 |
|---|---|---|
| INST-P1-038 | 十个 Delve 命令只有 buy/upgrade 两个 reducer | 完整实现或关闭八个 false capability，禁止协议成功掩盖无 authority |
| INST-P1-039 | buy/upgrade 不要求 active run/claim，只检查硬编码门坐标 | interaction解析 InstanceId、object handle、visibility/range和run revision |
| INST-P1-040 | 1,409 行 DelveState是单 run标量 fixture | DelveRuntime 管理多 run registry，run聚合只拥有自己的 participant/module/session state |
| INST-P1-041 | 77 个字段混合布局、战斗、伙伴、锁匠、商店、daily和test | 拆分 Definition/Run/Encounter/Interaction/Reward/Projection owner与schema |
| INST-P1-042 | 默认 owner=1、chest=1、seed=20061 等 fixture identity | allocator签发run/owner/object IDs和seed；测试通过fixture builder显式注入 |
| INST-P1-043 | 八组 `Array<int>` 用 magic code表达事件 | typed event schema含run/tick/entity/object/revision/payload，并通过outbox发布 |
| INST-P1-044 | module/mob/object主要用 bool/count | module graph引用真实spawn group、object generation、completion predicate和checkpoint |
| INST-P1-045 | `delve_module_content` 1,493 行、476 个条件分支 | 生成结构化 module asset、索引和schema validator，不以手写 accessor ladder运行 |
| INST-P1-046 | `delve_collision_content` 10,244 行、3,384 个条件分支且不可达 | cook为instance-local collider asset/BVH并按layout revision加载、缓存、卸载 |
| INST-P1-047 | source run layout存在但产品 router明确排除 active Delve | run创建冻结layout/module order，collision/LOS/spawn共享同一layout handle |
| INST-P1-048 | affix/admission/daily/bountiful规则分散在不可达叶 | definition compiler生成versioned ruleset，admission与run snapshot记录选中结果 |
| INST-P1-049 | Drowned Litany boss/rite/room/loot仍是标量函数集合 | EncounterRuntime拥有phase、cast、spawn、threat、wipe/reset、loot和replay事件 |
| INST-P1-050 | LockpickSession只在候选 DelveState 内，产品/client不消费projection | lockpick成为server-authoritative interaction子状态，recipient-filtered projection驱动UI |
| INST-P1-051 | session ID仅由 chest ID+tick拼接，缺instance/generation/nonce | SessionId包含instance/object generation、server nonce与revision；重复/迟到请求幂等 |
| INST-P1-052 | HUD产生 RequestLockpick host effect但无状态、错误、deadline或结果视图 | client只按authoritative projection展示board/tries/deadline/result/reconnect状态 |

### 4.5 Pet 与 Companion

| ID | 差距 | 重构要求 |
|---|---|---|
| INST-P1-053 | 11 个 Pet catalog命令只有七个进入 pet branch | abandon/rename/feed/heal在未实现前关闭capability并返回typed Unsupported |
| INST-P1-054 | pet branch只允许offline Hunter/Warlock | PetRuntime按owner principal/character/pet definition授权，不以offline demo角色硬编码 |
| INST-P1-055 | 只搜索template code 48 Emberkin/Imp | 支持稳定PetId、species/template、summon slot、stable和多宠物ownership policy |
| INST-P1-056 | taunt/Water Jet命令被接受后无效果 | capability与spellbook/autocast从pet definition生成；不可用动作明确拒绝 |
| INST-P1-057 | auto-taunt/auto-Water-Jet读取payload后强制false | 更新命令返回accepted/applied/reason/current revision，不制造成功假象 |
| INST-P1-058 | 739 行 PetState保留另一套完整fixture authority | 迁移可复用规则到PetRuntime test oracle，完成后删除双重状态机 |
| INST-P1-059 | 无pet persistent identity、name、health/spellbook/save schema | 独立PetRecord支持rename/feed/heal/stow/revive、migration和owner transfer |
| INST-P1-060 | abandon/retire通过dead/inert row保留实体 | 接入Runtime05 generation despawn和引用清理，旧handle必须失效 |
| INST-P1-061 | 每tick pet遍历entity，assist又全entity/threat扫描 | owner→pet、instance spatial query、threat/target index和dirty/deadline schedule增量化 |
| INST-P1-062 | follow/chase仅直线地面移动，碰撞/path/warp恢复缺失 | 复用Runtime08D navigation request/path cache/avoidance/teleport recovery contract |
| INST-P1-063 | Delve companion产品状态只有rank和静态content lookup | CompanionRuntime拥有spawn、owner/party binding、target/decision、death/revive、rank效果与projection |

### 4.6 Persistence、性能、诊断与资格

| ID | 差距 | 重构要求 |
|---|---|---|
| INST-P1-064 | shop/upgrade/heroic奖励直接突变marks/copper/item列 | 所有cost/reward通过Runtime14原子transaction、capacity/mail fallback、receipt/outbox |
| INST-P1-065 | 无instance/claim/run/pet/lockout snapshot schema与migration | versioned aggregate snapshot+journal，decode只校验结构，migration不补业务事实 |
| INST-P1-066 | 无allocator、instance tick、collision、pet scan的预算与规模曲线 | 记录p50/p99、queue depth、active runs、entities、colliders、allocations和slope artifact |
| INST-P1-067 | 无duplicate/reorder/drop/disconnect/crash/abuse/visibility资格 | 建fault/security/privacy/reconnect/transfer/reset/reward exactly-once test matrix |
| INST-P1-068 | 静态contract与缺失artifact无法证明产品root | generated inventory执行compile/run/result digest；动态阻断必须保留为not-run而非pass |

## 5. P2 完整性与维护性差距

| ID | 差距 | 收敛方向 |
|---|---|---|
| INST-P2-001 | instance/run/claim/owner/session大量使用裸int/string | generated newtype、stable wire code、generation和display/debug分离 |
| INST-P2-002 | state/result/event使用裸整数魔数 | typed enum并固定unknown-value策略与schema migration |
| INST-P2-003 | 每个规则函数携`required: bool`作为contract开关 | 测试前置条件移入assert/helper，生产API不携伪业务参数 |
| INST-P2-004 | 56 个模块混有inline `contractTest()` | 测试移出production artifact，保留public behavior而非test entry |
| INST-P2-005 | 46个test main没有统一suite taxonomy | catalog标注unit/contract/integration/parity/fault/perf和owner |
| INST-P2-006 | 七个test main无manifest | 生成manifest或删除孤儿入口，CI对未接管test source fail |
| INST-P2-007 | 38个manifest的binary目录全部缺失 | artifact registry记录build/run/digest/backend/source，missing fail gate |
| INST-P2-008 | 巨型generated条件树不可读且难profile | 生成结构化数据+validator+debug dump，代码只保留bounded query |
| INST-P2-009 | future/later注释没有plan/gate引用 | 注释链接owner contract和唯一milestone，自动检查过期状态 |
| INST-P2-010 | 95/103文件没有source commit header | provenance由生成器统一写metadata，不依赖手工注释 |
| INST-P2-011 | client action可见性未从capability生成 | UI route、enablement、unsupported reason与server manifest同源 |
| INST-P2-012 | lockpick board/action字符串和索引跨Zr/Rust重复 | generated protocol enum、projection schema和round-trip corpus |
| INST-P2-013 | 没有实例/reward/pet save-reload golden corpus | 建多schema、多BuildSet、disconnect/crash边界corpus |
| INST-P2-014 | 无instance replay inspector | 展示seed、tick、input、spawn、encounter、reward、digest首差异 |
| INST-P2-015 | 参考引擎采用点没有owner decision record | 每个借鉴/拒绝点记录Zircon约束，禁止复制API名称代替设计 |
| INST-P2-016 | Unity Graphics不是online instance语义来源 | 只借鉴显式resource owner/read-write/lifetime/culling，不宣称其定义server lifecycle |

## 6. 参考引擎差异

### 6.1 Unreal Engine

`UWorld` 明确拥有 persistent level、streaming levels、level collections、GameState 和 NetDriver；level collection又可绑定自己的persistent level、GameState、NetDriver与level set。`GameModeBase` 为login/post-login/logout、starting player和seamless travel定义authority hooks，`PlayerState`提供seamless-travel copy/override，`Actor`把owner、net owner、relevancy、role和destroy通知纳入网络生命周期。Unreal源码不直接提供MMO dungeon allocator，但它证明world/level、authority、connection、player state与travel是显式对象和阶段，不靠坐标带猜测。

### 6.2 Godot

`SceneMultiplayer` 持有peer connection status、root path、cache与replicator；Spawner跟踪spawn node、tracked ObjectID、spawn limit与node exit；Synchronizer提供authority、root、replication config、interval和per-peer visibility/filter。Godot同样不是持久化lockout服务参考，但其peer、authority、spawn/despawn和recipient visibility边界足以反证“有输入payload+实体数组”就等于多人实例生命周期。

### 6.3 Bevy 与 Fyrox

Bevy每个`World`有唯一`WorldId`和自己的`Entities`；entity经历spawn/despawn，generation变化使旧引用失效。Fyrox `Scene`独立拥有Graph、physics/animation更新和enabled lifecycle，Engine持有`SceneContainer`并逐scene更新；其Pool handle也通过generation防止复用位置命中旧对象。Zircon需要吸收的是world scope、独立容器、generation和显式生命周期，而不是照搬其编辑器或ECS接口。

### 6.4 Unity Graphics

Unity Graphics `RenderGraph`把import/create resource、record pass、execute和cleanup拆成明确阶段，resource registry集中拥有资源；builder通过声明读写关系形成依赖与生命周期。它不是dungeon/pet/online语义参考，只能作为结构纪律类比：instance definition、run、claim、participant、reward和projection必须声明owner、输入、写集、lifetime与cleanup，不能继续由WorldState任意列互相改写。

## 7. 目标 Owner 与边界

| Owner | 唯一职责 | 禁止承担 | 前置依赖 |
|---|---|---|---|
| `InstanceDefinitionRegistry` | versioned dungeon/delve/layout/encounter/ruleset metadata与provenance | live participant、economy mutation | Tooling05 cook/codegen、asset schema |
| `InstanceAllocator` | reservation、capacity、region/server placement、lease与fencing | encounter simulation、reward | Runtime15 party/match、App03 host role |
| `InstanceWorldRuntime` | isolated world/entity/schedule/clock/RNG、spawn/despawn、checkpoint/shutdown | queue、account、inventory commit | Runtime05 world/ECS、Runtime12 schedule/codec |
| `InstanceClaimStore` | claim、difficulty、roster revision、lockout、reset、expiry与CAS | collision、combat tick | Principal/Party、durable storage |
| `InstanceTransferRuntime` | admission token、source freeze、target restore、commit/rollback、reconnect | ruleset selection、reward | Runtime08E network、Allocator/Claim/World |
| `DungeonRuntime` | dungeon admission、encounter graph、participation、empty reset、completion | generic combat/economy internals | Definition/Claim/World、Runtime13 |
| `DelveRuntime` | run/module/affix/interaction/companion/lockpick/checkpoint/completion | account store、direct item mutation | Definition/Claim/World/Encounter |
| `EncounterRuntime` | phase、spawn group、boss/rite、wipe/reset、combat result events | allocator、mail/inventory | Runtime13 combat、InstanceWorld |
| `PetRuntime` | persistent pet identity、summon/stable/mode/spellbook/autocast/AI/lifecycle | navigation implementation、owner account | Runtime05/08D/13、Principal binding |
| `CompanionRuntime` | run-scoped companion spawn、owner/party、decision/target/rank effect、projection | permanent pet stable、shop transaction | Delve/Encounter/Pet rule adapters |
| `InstanceRewardTransaction` | participation/eligibility/cost/reward/inventory-mail/receipt/outbox exactly-once | encounter rules | Runtime14 transaction、Dungeon/Delve result |
| `InstanceProjectionRuntime` | recipient-filtered world/run/lockpick/pet/encounter/client snapshot | authority mutation | Runtime08E replication、all owners |
| `InstanceEvidenceRunner` | product-root parity、replay、save/load、fault/security/load/perf artifact | 第二套业务规则 | Tooling05/10、source golden |

依赖顺序必须先完成 identity/party、definition和instance-qualified handle，再建立allocator/claim/transfer与isolated world，之后接Dungeon/Delve/Encounter，继而Pet/Companion和RewardTransaction，最后开放client projection与catalog capability。不能先把缺失的14个命令继续塞进WorldState，再承诺以后迁移。

## 8. 重构里程碑

### M0 · Capability、topology 与 BuildSet 冻结

- 生成57个非test-main模块的production/fixture/oracle分类，以及main/world/self-test closure；
- 关闭14个无reducer命令和五个pet no-op/forced-false假能力；
- 固定11个M7 source/current行为oracle、source revision和expected diff。

### M1 · Identity、definition 与 instance-qualified handle

- 交付WorldId/InstanceId/ClaimId/RunId/EntityGeneration/PetId/SessionId；
- 生成versioned definition/layout/encounter/ruleset artifact与provenance；
- snapshot/handshake/replay统一BuildSet和schema。

### M2 · Allocator、claim、transfer 与 isolated world

- 实现reservation/placement/lease/fencing/capacity和admission policy；
- 实现source freeze、target restore、rollback、disconnect/reconnect和crash takeover；
- InstanceWorld拥有独立entity container、clock/RNG/schedule/collision context。

### M3 · Dungeon 与 Delve lifecycle

- Dungeon完成enter/leave、difficulty freeze、encounter、participation、empty reset、lockout；
- Delve完成layout/module/affix/interact/boss/death/lockpick/chest/rite/completion；
- collision/LOS从坐标分带切到instance-local cooked asset与broadphase。

### M4 · Pet、Companion 与 navigation

- PetRecord完成tame/summon/stable/rename/feed/heal/revive/abandon/save/load；
- spellbook/autocast/capability与definition同源，Pet AI使用spatial/threat/nav索引；
- Companion成为run-scoped owner并接Delve/Encounter projection。

### M5 · Reward、projection 与 hard cut

- dungeon/delve/pet相关cost/reward全部进入Runtime14 transaction/outbox；
- client只消费authoritative instance/lockpick/pet/result projection；
- 迁移或移出53个不可达候选模块，删除WorldState平行authority和坐标实例猜测。

### M6 · Qualification

- 恢复11个真实parity owner、46个test main inventory和required executable artifacts；
- 通过save/load/replay/transfer/reconnect/crash/reset/duplicate/reorder/privacy/security/load/soak；
- collision、allocator、tick、pet AI提供1/10/100/1k instance/entity规模斜率和预算证据。

## 9. Runtime 资格门

| Gate | 验收内容 |
|---|---|
| INST-G01 | capability manifest、protocol、reducer、projection、artifact、evidence同代；缺一项不得标Supported |
| INST-G02 | World/Instance/Claim/Run/Entity/Pet/Session handle携namespace、generation、BuildSet/schema且stale fail-closed |
| INST-G03 | allocator/reservation/placement/lease/fencing/admission在capacity、duplicate、timeout和server loss下可恢复 |
| INST-G04 | enter/leave/transfer/reconnect是可回滚事务，源/目标不会同时拥有或同时丢失participant |
| INST-G05 | InstanceWorld拥有独立clock/RNG/schedule/entity lifecycle/checkpoint/shutdown并可并行运行 |
| INST-G06 | collision/LOS显式instance-local，无coordinate-band fallback；broadphase/continuous sweep满足预算 |
| INST-G07 | dungeon difficulty/claim/participation/empty reset/lockout/reward在party reform、断线、重启下语义稳定 |
| INST-G08 | Delve enter/module/affix/interact/boss/death/lockpick/chest/rite/exit全链从产品root执行 |
| INST-G09 | encounter phase/spawn/despawn/wipe/reset/loot与Runtime13 combat event可重放、可诊断 |
| INST-G10 | lockpick session防重放/越权/迟到，board隐私、deadline、result、reconnect projection正确 |
| INST-G11 | pet/companion identity、spellbook/autocast、AI/nav、death/revive/stable/save/load/transfer全生命周期通过 |
| INST-G12 | reward/cost exactly-once，inventory capacity/mail fallback/receipt/outbox在crash边界无dup/loss |
| INST-G13 | snapshots携schema/BuildSet/generation，migration有golden corpus且不在decode阶段补业务事实 |
| INST-G14 | allocator/tick/collision/pet scan无未界定全表/步长扫描，规模曲线和p50/p99满足预算 |
| INST-G15 | 11个parity场景从产品root绑定source/backend/seed/exact trace，fault/security/privacy/load结果可审计 |
| INST-G16 | 46个test main全被inventory接管，required artifact实际执行；dynamic、frontmatter、链接、统计、diff check全绿 |

## 10. 状态与边界

| 项目 | 状态 |
|---|---|
| instance/world物理清单与main/product reachability | review_complete |
| catalog/client/reducer与WorldState authority差异 | review_complete |
| dungeon/delve/lockpick/pet/reset/collision热路径 | review_complete |
| Unreal/Godot/Bevy/Fyrox/Unity Graphics参考路由 | review_complete |
| production代码、测试、manifest、generated artifact修改 | pending，未在本轮执行 |
| WOC native动态验证 | blocked_by_existing_compile_errors，6个`woc_protocol`错误后无测试执行 |
| WOC npm动态验证 | blocked_by_existing_contract_drift，typed contract 157与expected 148不一致 |

本篇不要求把53个不可达模块全部删除。结构化规则、golden常量和scenario函数可以迁入definition compiler、oracle或evidence package；但它们只有在唯一产品owner中可达、携同代BuildSet、通过真实artifact执行后，才能计入支持度。相反，`delve_state`/`pet_state`/`dungeon_state`不得作为第二套authority长期留在production source root。

Runtime05继续唯一拥有通用world/entity generation lifecycle；Runtime08D拥有navigation；Runtime08E拥有transport/replication；Runtime12拥有WOC schedule/codec与纯decode/migration；Runtime13拥有combat outcome；Runtime14拥有inventory/economy transaction；Runtime15拥有principal/party/matchmaking；App03拥有VM/host outer publish；Tooling05/10拥有generated artifact和evidence orchestration。本篇只拥有instance allocation/claim/transfer、dungeon/delve/pet/companion产品接线与资格门，实施时不得借审查之名继续扩大WorldState。
