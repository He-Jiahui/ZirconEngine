---
related_code:
  - examples/woc/scripts/woc_game/src/main.zr
  - examples/woc/scripts/woc_game/src/world/state.zr
  - examples/woc/scripts/woc_game/src/social
  - examples/woc/scripts/woc_game/src/social/chat_wire_state.zr
  - examples/woc/scripts/woc_game/src/social/chat_state.zr
  - examples/woc/scripts/woc_game/src/social/party_raid_state.zr
  - examples/woc/scripts/woc_game/src/social/party_frame_projection_state.zr
  - examples/woc/scripts/woc_game/src/social/targeting_markers_state.zr
  - examples/woc/scripts/woc_game/src/social/arena_state.zr
  - examples/woc/scripts/woc_game/src/social/duel_state.zr
  - examples/woc/scripts/woc_game/src/social/dungeon_finder_role_state.zr
  - examples/woc/scripts/woc_game/src/social/fiesta_state.zr
  - examples/woc/scripts/woc_game/src/social/vale_cup_queue_state.zr
  - examples/woc/scripts/woc_game/src/social/yumi_match_state.zr
  - examples/woc/scripts/woc_game/src/social/m6_scenario_matrix.zr
  - examples/woc/scripts/woc_game/src/social/card_duel_queue.zr
  - examples/woc/scripts/woc_game/src/social/card_duel_queue_coordinator.zr
  - examples/woc/scripts/woc_game/src/social/card_duel_service.zr
  - examples/woc/scripts/woc_game/src/social/card_duel_command_router.zr
  - examples/woc/scripts/woc_game/src/social/card_duel_snapshot.zr
  - examples/woc/scripts/woc_game/src/protocol/commands.zr
  - examples/woc/scripts/woc_game/src/protocol/command_payloads.zr
  - examples/woc/native/apps/woc_client/src/input/intent.rs
  - examples/woc/reference/current-head/command_catalog.json
  - examples/woc/reference/current-head/parity_scenarios.json
tests:
  - examples/woc/scripts/woc_game/woc_m6_chat_social_tests.zrp
  - examples/woc/scripts/woc_game/woc_m6_party_raid_tests.zrp
  - examples/woc/scripts/woc_game/woc_m6_party_frame_projection_state_tests.zrp
  - examples/woc/scripts/woc_game/woc_m6_arena_tests.zrp
  - examples/woc/scripts/woc_game/woc_card_duel_command_tests.zrp
  - examples/woc/scripts/woc_game/woc_card_duel_command_router_tests.zrp
  - examples/woc/scripts/woc_game/woc_card_duel_match_tests.zrp
  - examples/woc/scripts/woc_game/woc_card_duel_primitives_tests.zrp
  - examples/woc/scripts/woc_game/woc_card_duel_queue_coordinator_tests.zrp
  - examples/woc/scripts/woc_game/woc_card_duel_service_tests.zrp
  - examples/woc/scripts/woc_game/woc_card_duel_snapshot_tests.zrp
  - examples/woc/scripts/woc_game/woc_card_duel_world_state_tests.zrp
  - examples/woc/native/Cargo.toml
  - examples/woc/tools/wos158_chat_ready_runtime_static_guard.mjs
  - examples/woc/tools/wos20_ready_check_state_check.mjs
  - examples/woc/tools/wos21_arena_queue_state_check.mjs
plan_sources:
  - docs/plans/optimize/zircon_app/03-woc-product-role-host-zrvm-transaction-state-client-server-integration-review.md
  - docs/plans/optimize/zircon_runtime/08e-network-runtime-review.md
  - docs/plans/optimize/zircon_runtime/12-woc-zrvm-package-kernel-world-state-schedule-serialization-runtime-review.md
  - docs/plans/optimize/zircon_runtime/13-woc-combat-casting-effect-aura-damage-threat-death-runtime-review.md
  - docs/plans/optimize/zircon_runtime/14-woc-progression-inventory-item-economy-crafting-quest-talent-runtime-review.md
  - docs/plans/optimize/zircon_tooling/05-woc-content-codegen-build-scripts-generated-artifact-incremental-review.md
  - docs/plans/woc/00-woc-engine-capability-foundation.md
  - docs/plans/woc/01-woc-zrvm-one-to-one-replication.md
reference_engines:
  - dev/UnrealEngine/Engine/Plugins/Online/OnlineFramework/Source/Party/Public/Party/SocialParty.h
  - dev/UnrealEngine/Engine/Plugins/Online/OnlineFramework/Source/Party/Public/Party/PartyDataReplicator.h
  - dev/UnrealEngine/Engine/Plugins/Online/OnlineFramework/Source/Party/Public/Chat/SocialChatManager.h
  - dev/UnrealEngine/Engine/Plugins/Online/OnlineServices/Source/OnlineServicesInterface/Public/Online/Auth.h
  - dev/UnrealEngine/Engine/Plugins/Online/OnlineServices/Source/OnlineServicesInterface/Public/Online/Social.h
  - dev/UnrealEngine/Engine/Plugins/Online/OnlineServices/Source/OnlineServicesInterface/Public/Online/Presence.h
  - dev/UnrealEngine/Engine/Plugins/Online/OnlineServices/Source/OnlineServicesInterface/Public/Online/Lobbies.h
  - dev/godot/modules/multiplayer/scene_multiplayer.h
  - dev/godot/modules/multiplayer/multiplayer_synchronizer.h
  - dev/godot/modules/multiplayer/multiplayer_spawner.h
  - dev/bevy/crates/bevy_ecs/src/entity/mod.rs
  - dev/bevy/crates/bevy_ecs/src/message/messages.rs
  - dev/bevy/crates/bevy_ecs/src/event/mod.rs
  - dev/Fyrox/fyrox-core/src/pool/handle.rs
  - dev/Fyrox/fyrox-core/src/pool/mod.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/IRenderGraphBuilder.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 15 · WOC Social Identity、Party、Raid、Chat、Duel、Arena、Matchmaking 与 Minigame Runtime 工程化差距

## 1. 结论

WOC social 目录不是一组空接口。它共有 60 个 `.zr`、8,425 行、305,975 bytes；去掉 29 个 test main 后，仍有 31 个候选生产/共享模块、7,699 行、277,741 bytes。Chat、Party/Raid、Ready Check、Target Marker、Arena、Duel、Fiesta、Yumi、Vale Cup、Dungeon Finder 和 Card Duel 都保存了可用于迁移的 current-head 规则投影。Card Duel 尤其已经进入 `world/state.zr` 的命令和 fixed-tick 路径，具备 FIFO queue、match lifecycle、caller-only hand query、snapshot 和共享 RNG 恢复，不能笼统标成“未实现”。

产品执行事实却与物理代码量严重分裂。从 `main.zr` 以及 `world/state.zr` 正常 fixed-tick 生产段沿静态 import 递归，只有 10/31 个 social 模块可达、1,737 行；21 个模块、5,962 行不在产品图中，约占非 test-main social 源码的 77.4%。不可达集合包含完整 `chat_state`、`party_raid_state`、`party_frame_projection_state`、`arena_state`、`duel_state`、Dungeon Finder、Fiesta、Vale Cup 与 Yumi。产品真正的 party/chat/arena authority 仍是 68,000 行级 `WorldState` 中按 entity row 复制的 36 组 social 字段和手写 reducer，领域模块存在并不等于产品交付。

公开能力面比实际 reducer 更危险。`command_catalog.json` 和 native client 已公开 18 个 SocialGraph、9 个 Dungeon Finder、6 个 Vale Cup、6 个 Duel/Arena 命令；WorldState 只实现 arena queue/leave，duel request/accept/decline、arena augment 以及前三组命令最终都落入 `authoritative command reducer is not implemented`。Chat 的非空合法消息会先消耗 token，除 `/ready`/`/readycheck` 外随后全部 state-neutral；命令 sequence 仍前进，也没有 receipt、delivery event 或 rejection 原因。这个行为不是“UI 尚未补齐”，而是服务端确认消费了命令却没有提供承诺的社会交互语义。

身份与连接生命周期是根部缺口。WorldState 只有 raw entity ID，没有稳定 account/principal、session、presence、platform identity、connection generation、offline membership 或 reconnect handoff；实体生产数组又不会物理删除。Party invitation、leader、member、arena queue 和 Card Duel liveness 都绑定临时 entity row，因此无法定义跨 world party、断线保留、重复登录、server transfer、离线 guild/friend 或可靠私聊。Unreal 的 Party/Auth/Social/Presence/Lobbies 把 account、party/lobby ID、成员关系、连接状态和事件作为独立 owner；Godot multiplayer 也显式维护 pending/authenticated/connected peer 及超时、spawn/despawn、per-peer visibility。当前 WOC 没有等价边界。

Card Duel 的接线还暴露了更直接的正确性阻断：可达的 `CardDuelQueueCoordinator.join(pid, cardMasterInRange, alreadyInDuel)` 需要三个参数，而 `card_duel_service.zr` 只传两个。当前 dynamic lane 在更早的既有编译错误处停止，29 个 social manifests 又没有 checked-in 对应 artifact，所以这条可达路径未被可执行证据证明。即便修复参数，WorldState 仍丢弃 typed `CardDuelCommandResult`，并在每个命令 batch 和每个 fixed tick 无条件 decode、全 entity/player 双向同步、update、encode；queue 配对连续两次 `removeAt(0)`。这与工程级 online/minigame runtime 所需的增量 owner、事件 outbox、明确 match identity 和可量化性能斜率仍有本质差距。

## 2. 审查范围与执行拓扑

### 2.1 物理清单

| 项目 | 结果 |
|---|---:|
| `src/social` 全部 `.zr` | 60 文件 / 8,425 行 / 305,975 bytes |
| 非 `*_test_main.zr` | 31 文件 / 7,699 行 / 277,741 bytes |
| test main | 29 文件 / 726 行 / 28,234 bytes |
| 正常产品 import closure | 10 文件 / 1,737 行 / 63,448 bytes |
| 不可达非 test-main | 21 文件 / 5,962 行 / 214,293 bytes |
| public class / var / function | 26 / 338 / 312 |
| `throw` / `new container.Array` / `while` | 48 / 201 / 168 |
| inline `contractTest()` | 22 |

最大的不可达或半独立模块包括 706 行 `fiesta_state`、566 行 `yumi_maze_layout`、552 行 `arena_state`、491 行 `party_raid_state`、413 行 `vale_cup_queue_state`、380 行 `party_frame_projection_state`、378 行 `chat_state` 和 366 行 `yumi_match_state`。这些文件保存了有价值的规则，但当前只能计作迁移 oracle 或 fixture，不能计入 shipped capability。

### 2.2 产品可达集合

正常产品图只包含 Card Duel 的 command/router/queue/coordinator/hand/match/service/snapshot 八个模块，以及 `chat_wire_state`、`targeting_markers_state`，合计十个 social 模块。完整 chat、party、arena、duel 和其他 minigame 模块都没有被 main/world import。`m6_scenario_matrix` 列出的十个场景也不在产品执行图中。

### 2.3 WorldState 实际 authority

`WorldState` 当前精确有 519 个 `pub var`，其中 36 个具有 party/raid/ready/chat/arena/card-duel 信号：party/leader/raid/group/member order/invite、marker、master loot、ready-check、chat token bucket、arena rating/queue，以及一块 opaque Card Duel snapshot bytes。它没有 guild、friend、block、social relationship、account principal 或 presence 字段。生产段对 `partyEntityIndexById(` 有 168 个源码调用点；`updatePartyReadyChecks` 每 tick 执行，内部 prior/member scan 可形成平方斜率；marker 命令会恢复完整 `MarkerState`，再分配并提交三组数组。

Arena 只保存五种格式的 admission row：1v1、2v2、Fiesta、Yumi3、Yumi5。fixed tick 没有 matcher drain；queue row 只有显式 leave 才退出。Card Duel 则在 command batch 入口恢复 service、batch 末提交，fixed tick 又恢复、同步、更新、提交。这里的“存在状态”与“完成 online match service”必须严格区分。

### 2.4 公开命令与 reducer 差异

| Facet | Catalog 命令数 | WorldState 结果 |
|---|---:|---|
| `IWorldParty` | 14 | hand-written party/loot/marker/ready 路径可达，但无事件回执 |
| `IWorldDuelArena` | 6 | 仅 arena queue/leave；duel 三命令与 arena augment 抛未实现 |
| `IWorldSocialGraph` | 18 | 全部未实现 |
| `IWorldDungeonFinder` | 9 | 全部未实现 |
| `IWorldValeCup` | 6 | 全部未实现 |
| `IWorldCardMinigame` | 4 | 路由至 Card Duel，但 typed result 被丢弃 |

Native client 的 `input/intent.rs` 同样暴露这些 intent，但 duel、Card Duel、Vale Cup、Dungeon Finder 没有对应状态 projection/UX；arena 也只有输入映射，没有 match lifecycle 视图。协议能够构造命令不是产品能力资格。

## 3. P0 阻断

| ID | 差距 | 证据与影响 | 必须重构 |
|---|---|---|---|
| SOC-P0-001 | 产品 authority 与领域模块分裂 | 21/31 个非 test-main social 模块不在产品图，party/chat/arena 又在 WorldState 维护平行状态；修一侧不会修另一侧 | 建立 production/fixture/test manifest，每个 capability 只保留一个 runtime owner，hard cut WorldState 平行 authority |
| SOC-P0-002 | Catalog/client 宣称的 social capability 无 reducer | 18 SocialGraph、9 Dungeon Finder、6 Vale Cup，以及 duel 三命令和 arena augment 都能通过协议入口，最终抛未实现 | 先以 capability negotiation 对外标 Unsupported；完成 owner、auth、transaction、receipt 后才重新发布命令 |
| SOC-P0-003 | Chat 接受并消费合法命令但静默丢消息 | 除 ready route 外，合法消息消耗 token、推进 sequence，却没有 channel/whisper/moderation/delivery event；客户端无法区分成功与丢弃 | 引入 `ChatRuntime`、typed receipt、delivery outbox、channel membership、moderation、privacy 和 retry/idempotency |
| SOC-P0-004 | 无稳定 online principal、presence 与连接生命周期 | party/arena/duel 都绑定 raw entity row；无 account/session/connection generation、disconnect/reconnect、cross-world transfer、offline membership | 先交付 `OnlinePrincipalRegistry` 与 `PresenceRuntime`，所有 social ID 使用 qualified generation identity，entity 只作 avatar binding |
| SOC-P0-005 | 可达 Card Duel 合同不一致且更新模型不可接受 | coordinator `join` 需要 3 参数，service 只给 2；结果被丢弃；每 batch/tick 全 snapshot decode-sync-encode，queue 前删搬移 | 修复并编译证明 typed API；改为增量 `CardDuelRuntime`、MatchId、command receipt/event journal、dirty snapshot 和 O(1) queue |
| SOC-P0-006 | 社交 parity 与动态证据不能证明产品语义 | 29 manifests 无 checked-in artifact；十个 M6 social parity owner 路径不存在；native Cargo 与 npm lane 均在既有错误处停止 | 建 generated test inventory、真实产品 root parity、exact WTR1、artifact provenance；任何 missing/not-run 不得计 pass |

## 4. P1 工程化差距

### 4.1 Authority、能力与身份

| ID | 差距 | 重构要求 |
|---|---|---|
| SOC-P1-001 | `main.zr` 只提供 fixed tick/lifecycle/string save/schema，没有 social query、event 或 delivery API | 为 host 定义 typed command receipt、event stream、projection snapshot 和 acknowledgement 边界 |
| SOC-P1-002 | `WorldState` 同时承担 avatar、party、chat token、match queue、minigame persistence | 按 principal/presence/party/chat/match/minigame 拆 owner，World 只持 qualified handles |
| SOC-P1-003 | capability 由 catalog 存在隐式推断，缺少 Implemented/Unsupported/Degraded 状态 | 生成与 reducer、client projection、test artifact 同代的 capability manifest |
| SOC-P1-004 | raw `uint` 同时表示 entity、player、party member、duel participant | 定义 `AccountId`、`CharacterId`、`EntityHandle`、`PartyId`、`MatchId` 等不透明强类型 |
| SOC-P1-005 | entity identity 没有 generation，旧 ID 可在复用或 restore 后误指 | 采用 generation/epoch 校验；参考 Bevy/Fyrox stale handle fail-closed |
| SOC-P1-006 | 没有 login/session ticket 与 platform user 映射 | Auth owner 建立 account、platform、session、connection 的显式映射和撤销 |
| SOC-P1-007 | 没有 online/offline/away/in-game/joinable presence | Presence 独立缓存、更新、查询和事件，不能从 entity 存在推断在线 |
| SOC-P1-008 | 没有 connection generation、duplicate login、kick/revoke 语义 | 每条入站命令绑定 authenticated connection generation 和 anti-replay window |
| SOC-P1-009 | 没有 disconnect timeout、reconnect lease 或 server transfer handoff | 定义 grace period、ownership lease、resume token、跨 world transfer transaction |
| SOC-P1-010 | social 状态没有 tenant/shard/world/build/schema identity | 所有 durable handle 与 snapshot 携 authority scope、BuildSet、schema、generation |
| SOC-P1-011 | invalid actor/target/permission 多数静默 `return` | 返回 stable status code、reason、retryability 和 authoritative revision |
| SOC-P1-012 | 没有 social audit、abuse、security 事件 owner | 建低基数审计 schema，覆盖 auth、invite、moderation、guild、match 和 admin action |

### 4.2 Party、Raid、Ready Check 与 Marker

| ID | 差距 | 重构要求 |
|---|---|---|
| SOC-P1-013 | party shared facts 复制到每个 entity row | 建单一 `PartyRecord` 与 member map，entity 只保存 membership handle |
| SOC-P1-014 | `nextPartyId` 是 world-local 自增 `uint`，无 generation/namespace | 使用 globally qualified PartyId 或 shard-issued ID，restore/transfer 不冲突 |
| SOC-P1-015 | invite 只保存 inviter entity ID 和 30 秒 deadline | 保存 invite ID、principal、party revision、issuer、target、expiry、status 与幂等键 |
| SOC-P1-016 | invite 不感知 target presence、block/privacy、cross-world 状态 | admission 统一查询 SocialGraph/Presence/Party policy，返回 typed outcome |
| SOC-P1-017 | party 5 人、raid 10 人和两组固定布局硬编码在 reducer | 将规模/role/subgroup policy 放入 versioned game rules artifact |
| SOC-P1-018 | leader 更替取第一条成员顺序，没有 lease、offline policy 或 transfer event | 定义 leader election/transfer state machine 和 revisioned event |
| SOC-P1-019 | party member order、leader、difficulty、loot facts 可能跨 row 不一致 | mutation 在单 record 上 CAS，projection 由 record revision 派生 |
| SOC-P1-020 | `partyReadyCheckStateIsValid` 与 tick sweep 重复全 entity 嵌套扫描 | ReadyCheck 按 PartyId 索引，pending count 增量维护，tick 使用 deadline heap |
| SOC-P1-021 | ready-check prompt、response 和 final counts 没有事件通道 | 发布 initiator/member/finalized typed events，并支持迟到/断线语义 |
| SOC-P1-022 | leave/kick/party dissolve 对其他系统只有同 state 内直接清字段 | 通过 outbox 通知 matchmaking、loot、instance、chat、UI，支持可重放补偿 |
| SOC-P1-023 | raid subgroup normalization 每个位置重新全 entity 选最小 order | 在 PartyRecord 内维护有序 member slots，移动/删除为有界增量操作 |
| SOC-P1-024 | marker 命令每次恢复全 `MarkerState` 并新建三数组提交 | marker 成为 PartyRecord 子资源，按 marker/entity 索引增量更新 |
| SOC-P1-025 | marker target 通过 f64 payload 传递并回转 entity ID | 协议使用明确整数 handle/generation，拒绝超出精度和 stale target |
| SOC-P1-026 | 无跨 world party、offline member、instance reservation 或 party revision | PartyRuntime 以 principal 为成员、avatar 为临时 binding，并与 instance allocator 建 lease |

### 4.3 Chat、Social Graph、Guild 与 Presence

| ID | 差距 | 重构要求 |
|---|---|---|
| SOC-P1-027 | `chat_wire_state` 只做 route/长度/token admission | 分离 parser、policy、routing、delivery、retention、moderation owner |
| SOC-P1-028 | UTF-8 校验从 progression `craft_item_state` 借用 | 将 text/UTF admission 收敛到 protocol/text utility，消除领域反向依赖 |
| SOC-P1-029 | token bucket 为 entity-row 状态，重连/换 world 可重置 | rate limit 绑定 account/device/IP policy 与 durable window，支持分级配额 |
| SOC-P1-030 | say/yell/general/world/LFG/party/whisper/reply/emote 没有产品 routing | 每种 route 定义 audience、range、membership、privacy、failure 和 delivery semantics |
| SOC-P1-031 | 没有 channel membership、join/leave、room owner 和 topic revision | 实现 `ChatChannelId`、membership state machine、room policy 与事件 |
| SOC-P1-032 | 没有 whisper target lookup、reply cursor、offline policy | 基于 principal lookup 和 privacy/block graph，定义 online/offline delivery |
| SOC-P1-033 | 没有 profanity/spam/abuse/report/mute moderation pipeline | admission 前后都可挂 policy，保留审计而不泄漏消息内容 |
| SOC-P1-034 | 没有 per-recipient visibility、locale、ignore/block filter | delivery fan-out 在 recipient policy 下生成 projection，不广播全量 state |
| SOC-P1-035 | 没有 message ID、sequence、ack、dedupe、retention 或 backpressure | 建分区有序 message/event log 与 bounded subscriber cursor |
| SOC-P1-036 | 18 个 friend/block/guild 命令全无 state owner | 先实现 SocialGraph relationship 与 guild aggregate，再发布 catalog capability |
| SOC-P1-037 | guild 缺 membership rank、invite、ownership、event、disband transaction | GuildRecord 使用 revision/CAS、permission matrix、audit 与 transactional outbox |
| SOC-P1-038 | client 无 social state projection，输入 intent 不能展示 authoritative result | 提供 friends/guild/chat/party/presence 增量 projection、loading/error/reconnect 状态 |

### 4.4 Duel、Arena、Matchmaking、Dungeon Finder 与其他 Minigame

| ID | 差距 | 重构要求 |
|---|---|---|
| SOC-P1-039 | Duel request/accept/decline 有协议和 client intent，但 World 无 reducer | capability 暂时关闭；实现 invite、range、expiry、busy、decline、start transaction |
| SOC-P1-040 | `duel_state` 不可达，无法证明产品生命周期 | 迁入 `CompetitiveMatchRuntime` 或明确删除，禁止长期双 authority |
| SOC-P1-041 | arena queue 只记录 admission row，fixed tick 没有 matcher drain | MatchmakingRuntime 按 format/rating/wait/party revision 消费队列并产 reservation |
| SOC-P1-042 | 五种 arena format 用 `uint 1..5` 隐式编码 | 使用 generated FormatId/RulesetId，能力与队伍规模由 BuildSet 定义 |
| SOC-P1-043 | queue unit order 溢出只依赖非零 sentinel | 使用宽 generation identity，overflow/restore/migration fail-closed |
| SOC-P1-044 | queue validity 在 encode/decode 多次平方扫描 | 维护 unit/member 索引和局部 invariant，full audit 只在 qualification 路径 |
| SOC-P1-045 | party queue 不锁 party revision，排队后成员可变 | queue ticket 保存 party revision/roster digest，变更原子撤销或重新 admission |
| SOC-P1-046 | matcher 没有 reservation、server allocation、instance admission | 输出 durable MatchReservation，和 instance allocator 做 prepare/commit/timeout |
| SOC-P1-047 | 没有 reconnect、backfill、no-show、cancel、ready-up state | 定义 ticket 和 match 两级 lifecycle、deadline、lease、补位策略 |
| SOC-P1-048 | 没有 result attestation、rating/honor transaction | authoritative result 以 MatchId 幂等提交 progression/economy ledger |
| SOC-P1-049 | arena augment 在 protocol/client 中可见但 reducer 缺失 | 将 augment 变为 ruleset-defined pre-match transaction 或明确 Unsupported |
| SOC-P1-050 | Dungeon Finder 九命令和 role fixture 完全断线 | 独立 listing/queue/proposal owner，角色、instance、party 通过 typed handles 协作 |
| SOC-P1-051 | Vale Cup 六命令及 queue/ball/bot/layout modules 不可达 | 定义 product adapter、authoritative simulation owner 和 server-only bot policy |
| SOC-P1-052 | Fiesta/Yumi 模块大量规则只在 fixture 内运行 | 用 ruleset adapter 接入 MatchRuntime；不能复制一套独立 queue/session identity |
| SOC-P1-053 | 多种 match/minigame 没有统一 seed、clock、result、replay contract | MatchContext 固定 BuildSet/ruleset/seed/fixed clock/participant revisions |
| SOC-P1-054 | client 只有 intent，没有 queue position、proposal、match、result projection | 为所有 supported format 交付状态机 UI 数据，不从本地输入猜测成功 |

### 4.5 Card Duel

| ID | 差距 | 重构要求 |
|---|---|---|
| SOC-P1-055 | service/coordinator `join` 参数数量不一致 | 编译期统一 typed request；加入接口 arity/ABI contract test |
| SOC-P1-056 | World 丢弃 `CardDuelCommandResult` | receipt/outbox 必须携 accepted/status/match/void/winner/forfeiter 和 revision |
| SOC-P1-057 | 只有全局 `lastWinner/lastForfeiter/lastMatchVoided` | 每场建立 MatchId、generation、participant、transition journal 与 result record |
| SOC-P1-058 | 每个 command batch 即使无 Card Duel 命令也 restore/commit | 先按 command facet 分组，仅 dirty owner 解码与提交 |
| SOC-P1-059 | 每个 fixed tick 都 snapshot decode、player sync、update、encode | service 常驻 owner，按 deadline/dirty set 调度；snapshot 只在 publish/checkpoint 编码 |
| SOC-P1-060 | `syncCardDuelPlayers` 形成 O(P*E + E*P) 扫描 | 用 EntityHandle↔ParticipantId 双向索引和 lifecycle event 增量同步 |
| SOC-P1-061 | queue `pairFirst` 连续两次 `removeAt(0)`，purge 也搬移数组 | 使用 deque/intrusive queue；按 candidate generation O(1) 删除 |
| SOC-P1-062 | liveness 来自 entity kind/dead 而不是 authenticated presence | 区分 alive、connected、eligible、in-range，并定义 disconnect grace/forfeit/void policy |

### 4.6 Storage、性能、诊断与资格

| ID | 差距 | 重构要求 |
|---|---|---|
| SOC-P1-063 | Card Duel snapshot 允许 100,000 records，内部多次线性 membership scan | 对 participant/match/queue 分别设 budget，decode 先验长度、索引构建和成本上限 |
| SOC-P1-064 | social deadline 混用 float seconds 与 integer micros/fixed tick | 统一 monotonic integer tick/micros，协议与 snapshot 明确 time domain |
| SOC-P1-065 | 静态 guard 只证明字段/分支形状，不能证明 delivery、privacy、match outcome | qualification 必须从产品 root 执行，比较 event/receipt/state/replay 全结果 |
| SOC-P1-066 | 无 queue/chat/party/card-duel 的 p50/p99、allocation、slope、drop 指标 | 建 bounded telemetry 与 1/100/1k/10k principal 规模曲线、fault/soak artifact |

## 5. P2 完整性与维护性差距

| ID | 差距 | 收敛方向 |
|---|---|---|
| SOC-P2-001 | social module header 仍写“later bridge/not connected”，与 Card Duel 当前可达事实冲突 | generated reachability 检查并更新 owner/status 注释 |
| SOC-P2-002 | route/format/status 使用裸整数和字符串 | generated enum/newtype 与稳定 wire code |
| SOC-P2-003 | 大量 `while`/parallel arrays 降低局部不变量可读性 | typed aggregate、iterator/query 和明确索引 owner |
| SOC-P2-004 | 失败文案是散落字符串 | stable error catalog、localization key 和 diagnostics context |
| SOC-P2-005 | party/chat/match 的常量散落在 reducer | rules artifact 与 documented source provenance |
| SOC-P2-006 | 29 个 test main/manifests 缺统一 suite taxonomy | generated test catalog 标明 unit/contract/parity/fault/perf |
| SOC-P2-007 | `contractTest()` 混在候选生产模块 | 移到 test package，production artifact 不携测试执行入口 |
| SOC-P2-008 | M6 scenario matrix 与 parity catalog 没有 machine-checked owner linkage | 生成 scenario→owner→artifact→result 映射 |
| SOC-P2-009 | social snapshot 没有独立 schema doc 与 compatibility matrix | 发布 schema/version/migration/support-window 文档 |
| SOC-P2-010 | client intent 名称与 capability 状态没有生成同源 | 从 capability manifest 生成 client action availability |
| SOC-P2-011 | ready/arena 等注释把 host 缺口当成未来责任但无依赖 ID | 注释引用唯一 owner、gate 和 hard-cut milestone |
| SOC-P2-012 | 无 privacy threat model | 记录 hand/whisper/presence/guild/party 可见性与日志脱敏规则 |
| SOC-P2-013 | 无 social save/reload/reconnect golden corpus | 建 versioned corpus 和 migration diff artifact |
| SOC-P2-014 | 无 deterministic match/minigame replay inspector | 记录 seed、input、ruleset、tick、result digest 并提供差异工具 |
| SOC-P2-015 | Unreal/Godot/Bevy/Fyrox 参考没有落到 owner decision record | 每个采用/拒绝点记录语义与 Zircon 约束，禁止照搬 API 名称 |
| SOC-P2-016 | Unity Graphics 只能提供依赖声明类比，容易被误当 social reference | 仅借鉴 declared read/write/owner/culling 思路，不宣称其定义 online semantics |

## 6. 参考引擎差异

### 6.1 Unreal Engine

`SocialParty.h` 以 PartyId、member map、leader、joinability、leave/disconnect、member/leader/config/state change、degraded mode、invite、platform session 和 join-in-progress 事件组织 party。`PartyDataReplicator.h` 又把 typed replicated data、scratch copy、ordered pending update、async load、compare/post-replication 和 deferred dirty change 独立出来。当前 WOC 把 shared party facts 复制到 entity row，没有 aggregate revision、ordered replication 或 connection-aware lifecycle。

`SocialChatManager.h` 明确区分 room/channel/private/group/read-only channel，包含 join/exit、sent/received、membership 和 failure handlers。Online Services 的 `Auth.h`、`Social.h`、`Presence.h`、`Lobbies.h` 进一步把 account/session、friend/invite/block relationship、presence/joinability、lobby owner/member/join policy/attributes 与异步事件分开。WOC 目前没有这些根 owner，因此不应直接在 WorldState 继续补 guild/friend/chat 分支。

### 6.2 Godot

`SceneMultiplayer` 显式管理 pending peers、authentication callback/timeout、connected peers、disconnect、packet routing、replication/RPC 和 spawn/despawn/sync；Synchronizer/Spawner 还包含 authority、per-peer visibility/filter、replication interval、tracked object 和 spawn limit。Godot 不是账号社交后端参考，但它证明 transport peer lifecycle、replication visibility 和 gameplay entity lifecycle不能合并成一个 raw entity-exists 判断。

### 6.3 Bevy 与 Fyrox

Bevy entity 使用 generation 和 fallible stale ID；message/event 设施提供 retained buffer、per-reader cursor、writer/reader、显式 update/drain 及 typed target。Fyrox pool 同样用 generational handle、free stack/ticket 防止旧句柄命中新对象。WOC 需要吸收的是 identity 与事件寿命语义，而不是照搬 ECS API。

### 6.4 Unity Graphics

Unity Graphics 的 RenderGraph builder 通过 `UseTexture/UseBuffer(AccessFlags)`、transient resource、pass culling 和 render function 声明 owner/read/write。它不是 social runtime 语义来源，只能作为架构纪律类比：Party、Chat、Matchmaking、Card Duel 应显式声明输入、写集、event output 和 dirty condition，避免当前无条件 snapshot rebuild。

## 7. 目标 Owner 与边界

| Owner | 唯一职责 | 禁止承担 | 前置依赖 |
|---|---|---|---|
| `OnlinePrincipalRegistry` | account/platform/session/connection generation 与 avatar binding | party/chat/gameplay rule | Runtime08E transport/auth、App03 host role |
| `PresenceRuntime` | online status、joinability、game/session properties、query/cache/event | entity health、party ownership | PrincipalRegistry |
| `SocialGraphRuntime` | friend/invite/block/ignore/guild relationship 与 permission/audit | chat payload、match state | Principal/Presence、durable transaction |
| `PartyRuntime` | PartyId、member/leader/invite/raid/ready/loot/marker aggregate 与 revision | avatar combat、instance simulation | Principal/Presence、Runtime14 loot boundary |
| `ChatRuntime` | channel/membership/routing/moderation/rate/delivery/receipt | UI rendering、entity progression | Principal/SocialGraph/Party/EventOutbox |
| `MatchmakingRuntime` | ticket、queue、rating window、proposal、reservation、allocation handoff | match simulation、rating commit | Party/Presence、instance allocator |
| `CompetitiveMatchRuntime` | duel/arena/minigame participant lifecycle、clock、result、reconnect | queue admission、economy mutation | Matchmaking、Runtime13 combat、Runtime14 transaction |
| `CardDuelRuntime` | Card Duel queue/match/hand/RNG/result 与 privacy projection | WorldState full snapshot rebuild | Principal/Presence、Match/EventOutbox |
| `SocialEventOutbox` | typed receipt/event journal、subscriber cursor、ack/retry/backpressure | 业务规则判断 | App03 outer transaction、Runtime12 journal |
| `SocialProjectionRuntime` | recipient-filtered party/chat/presence/match/client snapshots | authority mutation | all domain owners、Runtime08E replication |
| `SocialEvidenceRunner` | source parity、product replay、fault/load/privacy/security qualification | 第二套游戏规则 | Tooling05/10、source golden |

依赖顺序必须从 Principal/Auth/Presence 开始，再建立 EventOutbox 与 durable identity，之后才是 SocialGraph/Party/Chat，继而 Matchmaking/Match/Card Duel，最后才启用 client projection 和 catalog capability。不能先把 33 个当前未实现的 SocialGraph/Dungeon Finder/Vale Cup 命令塞进 WorldState，再承诺以后迁移身份。

## 8. 重构里程碑

### M0 · Capability 与 topology 冻结

- 生成 31 个非 test-main 模块的 production/fixture/test 分类及 main/fixed-tick reachability artifact；
- 以 reducer、projection、artifact、parity 同时存在为 Supported 条件，关闭 false capability；
- 固定十个 social parity 场景和 Card Duel current behavior oracle。

### M1 · Principal、Presence 与 Event 基础

- 建 Account/Character/Entity/Party/Match/Connection 强类型 identity 和 generation；
- 实现 auth binding、disconnect/reconnect/transfer lease、presence cache/event；
- 实现 typed command receipt、transactional event outbox、subscriber cursor 和 backpressure。

### M2 · Party、Chat 与 SocialGraph 收敛

- PartyRecord 取代 per-entity shared facts，ready/marker/loot 成为 aggregate 子状态；
- ChatRuntime 完成 channel/whisper/party/range/moderation/privacy/delivery；
- Friend/block/guild 在 durable relationship owner 完成后才开放 catalog。

### M3 · Matchmaking 与 Competitive Match

- queue ticket 绑定 roster/rating/ruleset revision，matcher 输出 instance reservation；
- Duel/Arena/Dungeon Finder/Fiesta/Yumi/Vale Cup 共用 lifecycle、reconnect、result contract；
- rating/honor/reward 通过 Runtime14 transaction 幂等提交。

### M4 · Card Duel 增量化

- 修复 typed API/arity 并通过真实 ZrVM compile/test；
- 建 MatchId、result journal、dirty snapshot、O(1) queue 和 lifecycle index；
- 接入 filtered client projection，证明对手 hand 不可见与断线规则。

### M5 · Hard cut 与 qualification

- 迁移或删除 21 个不可达候选 owner，删除 WorldState 平行 social 列；
- 恢复十个真实 parity owner 和 29 个 manifest 的 required artifact；
- 通过 replay/save/reload/reconnect/fault/security/privacy/load/soak/perf gates。

## 9. Runtime 资格门

| Gate | 验收内容 |
|---|---|
| SOC-G01 | capability manifest、协议、reducer、client projection、test artifact 同代；缺任一项不得标 Supported |
| SOC-G02 | account/session/connection generation/presence/character binding 可登录、断线、重连、转服且 stale handle fail-closed |
| SOC-G03 | party invite/join/leave/kick/leader/raid/ready/marker/loot 在 revision/CAS 下可重放、可断线恢复、可跨 world handoff |
| SOC-G04 | 每条命令产生 typed accepted/rejected receipt；每项状态变化产生可确认、可背压的 event/outbox record |
| SOC-G05 | say/yell/channel/party/whisper/reply/emote 通过 membership/range/privacy/block/moderation/delivery/ack 测试 |
| SOC-G06 | friend/block/ignore/guild 全生命周期具有 permission、audit、idempotency、offline/reconnect 和 migration 证明 |
| SOC-G07 | queue 能实际 drain，输出 reservation/server allocation/instance admission；party revision 变化和 timeout 可恢复 |
| SOC-G08 | duel/arena lifecycle 覆盖 proposal/start/no-show/disconnect/reconnect/forfeit/void/result/rating/honor 幂等提交 |
| SOC-G09 | Dungeon Finder/Fiesta/Yumi/Vale Cup 每项 Supported 都有产品 adapter、authoritative ruleset、client projection 和 parity |
| SOC-G10 | Card Duel API 编译一致、结果不丢、hand privacy、MatchId、dirty persistence、disconnect/deed/result transaction 全通过 |
| SOC-G11 | snapshot 携 schema/BuildSet/generation；decode 有预算、无业务补事实，migration 保留 aggregate identity/revision |
| SOC-G12 | party/chat/queue/card-duel 普通路径无全 entity 平方扫描和无条件全 snapshot 重建，规模曲线证明斜率与预算 |
| SOC-G13 | native client 展示 authoritative queue/party/chat/match/result/error/reconnect 状态，不从 intent 本地猜测成功 |
| SOC-G14 | 十个 social parity 从产品 root 执行，绑定 source/BuildSet/schema/backend/seed/exact WTR1 与 actual diff |
| SOC-G15 | duplicate/reorder/drop/disconnect/crash/timeout/abuse/privacy/load/soak 结果可审计且零 silent loss/dupe/partial publish |
| SOC-G16 | 29 manifests 进入 generated inventory且 required artifact 实际执行；动态 lane、frontmatter、链接、统计与 diff check 全绿 |

## 10. 状态与边界

| 项目 | 状态 |
|---|---|
| social 物理清单、main/fixed-tick reachability | review_complete |
| protocol/catalog/client intent 与 World reducer 差异 | review_complete |
| party/chat/arena/Card Duel 热路径与身份边界 | review_complete |
| Unreal/Godot/Bevy/Fyrox/Unity Graphics 参考路由 | review_complete |
| production 代码、测试、manifest、generated artifact 修改 | pending，未在本轮执行 |
| native/npm 动态验证 | blocked_by_existing_failures，未重复无变化 lane |

本篇不把 21 个不可达模块登记为“应该全部删除”。其中规则投影和 `contractTest()` 可以作为重构 oracle，但必须先分类、再迁入唯一 owner 或移出 production package。它同样不否认 Card Duel 已经比其他 social 域更接近产品路径；结论是当前接线仍有可达 API 合同错误、结果丢失和无条件全量重建，因此尚未取得工程级资格。

Runtime12 继续唯一拥有 WOC world storage/schedule/codec 与纯 decode/migration；Runtime08E 拥有 transport/replication；App03 拥有 VM/host outer publish；Runtime13 拥有 combat outcome；Runtime14 拥有 item/economy/reward transaction；Tooling05/10 拥有 generated artifact 与 evidence orchestration。本篇只拥有 social identity、presence、party/chat/social graph、matchmaking/minigame owner、产品接线和 qualification，实施时不得借审查之名继续扩大 WorldState。
