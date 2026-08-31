---
title: Runtime WOC Progression / Inventory / Item / Economy / Crafting / Quest / Talent 当前源码复审
category: zircon_runtime
report_id: Runtime165
review_date: 2026-08-30
baseline_head: e76240e1299259b8c4abb4def5e3f0537bda5074
baseline_epoch: current-working-tree
verification_head: e76240e1299259b8c4abb4def5e3f0537bda5074
verification_epoch: current-working-tree
supersedes_currentness_of:
  - docs/plans/optimize/zircon_runtime/14-woc-progression-inventory-item-economy-crafting-quest-talent-runtime-review.md
related_code:
  - examples/woc/scripts/woc_game/src/main.zr
  - examples/woc/scripts/woc_game/src/world/state.zr
  - examples/woc/scripts/woc_game/src/progression
  - examples/woc/scripts/woc_game/src/generated/m5_content_catalog.zr
  - examples/woc/scripts/woc_game/src/generated/talent_modifier_catalog.zr
  - examples/woc/scripts/woc_game/plugin.toml
  - examples/woc/scripts/woc_game/woc_game.zrp
  - examples/woc/zircon-project.toml
  - examples/woc/native/apps/woc_server/src/main.rs
  - examples/woc/native/apps/woc_headless/src/main.rs
  - examples/woc/native/plugins/woc_runtime/src/lib.rs
  - examples/woc/native/apps/woc_server/src/fixed_tick_driver.rs
  - examples/woc/native/plugins/woc_runtime/src/transaction.rs
  - examples/woc/native/crates/woc_protocol/src/generated_command_payloads.rs
plan_sources:
  - docs/plans/optimize/zircon_runtime/14-woc-progression-inventory-item-economy-crafting-quest-talent-runtime-review.md
  - docs/plans/optimize/zircon_runtime/12-woc-zrvm-package-kernel-world-state-schedule-serialization-runtime-review.md
  - docs/plans/optimize/zircon_runtime/13-woc-combat-casting-effect-aura-damage-threat-death-runtime-review.md
  - docs/plans/optimize/zircon_app/03-woc-product-role-host-zrvm-transaction-state-client-server-integration-review.md
  - docs/plans/optimize/zircon_tooling/05-woc-content-codegen-build-scripts-generated-artifact-incremental-review.md
  - docs/plans/woc/00-woc-engine-capability-foundation.md
  - docs/plans/woc/01-woc-zrvm-one-to-one-replication.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/AssetManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/GameFramework/SaveGame.h
  - dev/UnrealEngine/Engine/Source/Runtime/Net/Core/Classes/Net/Serialization/FastArraySerializer.h
  - dev/UnrealEngine/Engine/Source/Runtime/GameplayTags/Classes/GameplayTagContainer.h
  - dev/bevy/crates/bevy_ecs/src/schedule/schedule.rs
  - dev/bevy/crates/bevy_ecs/src/change_detection/params.rs
  - dev/bevy/crates/bevy_asset/src/handle.rs
  - dev/godot/core/io/resource.h
  - dev/godot/modules/multiplayer/scene_multiplayer.cpp
  - dev/Fyrox/fyrox-core/src/visitor/mod.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/IRenderGraphBuilder.cs
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Runtime165 · WOC Progression 当前源码复审

## 1. 结论

当前 WOC progression 不是空目录，但也不是可交付的工程级 progression runtime。`examples/woc/scripts/woc_game/src/progression` 当前有 101 个 `.zr` 文件、55 个非 test-main 文件、11,844 行和 487,097 bytes；从 `world/state.zr` 的普通 fixed-tick 生产根递归只能到达 15 个非测试模块，40 个模块不可达。不可达集合正好包含 `bank_state`、`trade_state`、`market_state`、`quest_state`、`crafting_transaction`、`inventory_instance_ledger`、`equipment_routing_state`、`talent_*_commit`、`xp_state` 等应当成为产品 authority 的核心 owner。

可保留的内容是 source-pinned 规则和少量边界底座：`ItemInstanceData` 已区分 signer、rolled/masterwork、enchant、binding 与 charges；`CraftingTransactionState` 已表达 recipe acquisition、材料/费用检查、节流与一次 RNG draw；`TalentAllocationCommitState` 已表达 combat/arena lock、allocation revision 和完整 effect flags；WorldState 已有有界 inventory scalar、craft、talent、loot 与 quest 投影；native server driver 已有固定 20 Hz host accumulator、命令/移动队列上限和 VM 外层候选快照。

这些内容不能被当作产品能力。`main.zr` 只把 fixed tick 委托给 66,105 行的单体 `WorldState`，其 inventory 是全局平行数组，craft 只有两个 recipe 投影且 `ritual_vestments` 永久报告 unsupported instance output，talent modifier 热路径仍会从 189 条生成目录解释字段。`quest_state.zr` 明确声明不被 main/world 导入，并以 A/B 两个玩家的布尔/标量数组作为 host facts。native server/headless 入口只调用 `identity_report_json`；当前生产 native 文件没有任何 `WocProjectVm` 实现，只有测试 VM 实现。因此即使脚本规则通过局部 contractTest，也没有 server/client/headless 的真实 VM、progression command、save/reload 或 parity 产品闭环。

本次将旧 Runtime14 按当前工作树重新冻结为 **6 项 P0 Open、64 项 P1 Open、15 项 P2 Open、16 项资格门全部 Fail**。没有关闭旧差距，也不新增与 Runtime12（ZrVM/world/snapshot）、Runtime13（combat outcome）、App03（外层 VM transaction）或 Tooling05（生成式 artifact）重复的 owner。本文只拥有 WOC progression 的产品 authority、identity、transaction、persistence、reachability 与 qualification。

## 2. 当前源码证据

### 2.1 物理范围与可达性

| 范围 | 当前证据 | 工程含义 |
|---|---:|---|
| progression `.zr` 全量 | 101 files / 11,844 lines / 487,097 bytes | 混有生产候选和独立 test-main |
| 非 test-main | 55 files / 11,158 lines / 457,914 bytes | 公开类、字段和函数很多，但不等于产品 owner |
| test-main | 46 files / 686 lines / 29,183 bytes | 独立入口，不能代替聚合 product runner |
| `world/state.zr` | 66,105 lines / 3,298,500 bytes | 仍是 progression 的实际单体 authority |
| 从 `world/state.zr` 普通 fixed-tick 可达 | 15 modules | 只有 M5 scalar/craft/talent helper 等有界投影 |
| 不可达非 test-main | 40 modules | bank/trade/market/quest/item instance/progression commit 等核心 owner 断线 |
| `m5_content_catalog.zr` | 5,287 lines / 4,264 `if` | 生成表以分支 ladder 进入源码 |
| `talent_modifier_catalog.zr` | 13,010 lines / 12,936 `if` / 189 entries | 规则规模直接乘到解释和重算成本 |

当前 progression 目录 fingerprint（按路径与文件 SHA-256 归一化后计算）为 `8b4719502c74095c101745ea6fc027834360f8e638131564e5d4f54539df022a`。这是工作树静态冻结，不是提交或可发布 artifact；并行修改后实施前必须重新取指纹。

### 2.2 入口和 authority

- `main.zr:16-21` 的 `fixedTick` 只检查 package active，然后直接调用 `world.fixedTick`；没有 `ProgressionRuntime`、BuildSet admission、character principal 或 per-world transaction boundary。
- `main.zr:24-46` 的 `saveState`/`restoreState` 只保存和替换一段字符串；`stateSchema` 返回固定 schema/world-state 文本，但没有 progression component schema、content revision 或 migration receipt。
- `world/state.zr:209` 定义单一 `WorldState`，inventory 从 `entityInventoryCopper`、`entityInventoryStackOffsets`、`entityInventoryStackItemCodes`、`entityInventoryStackCounts` 和 manual slots 等平行数组组成；删除一个 stack 会搬移其后实体的 offset，结构成本随所有后续实体增长。
- `world/state.zr:3638-3785` 的 craft path 先验证、扣材料/铜币、调用 `grantM5InventoryItem`、增加技能并写结果字段；输出只允许 M5 recipe projection，signed rolled instance 以显式 unsupported reason 终止。
- `world/state.zr:37344` 及多个战斗 caller 每次构造 `TalentModifierState` 并调用 `recompute`；当前目录虽新增 `entryIndex`，但没有按 allocation revision 安装 immutable modifier snapshot，索引修复不等于 prepared runtime。
- `examples/woc/zircon-project.toml` 的插件 selection 没有 progression/WOC runtime；`woc_game` 只声明 `zr_vm_language` 与 `woc_runtime`，并且 `plugin.toml` 固定 `zr_vm.execution_mode = "interp"`。
- `examples/woc/native/apps/woc_server/src/main.rs:3-6` 和 headless 对同一 identity inspector 做报告，没有构造 `WocTransactionalRuntime`、加载 `woc_game.zrp` 或提供生产 VM。仓库中 `impl WocProjectVm` 命中仅在 tests 下出现。

### 2.3 候选 owner 的真实状态

| 候选模块 | 当前形状 | 当前结论 |
|---|---|---|
| `inventory_instance_ledger.zr:7-74` | `ItemInstanceData`、fungible/instance slot、charge arrays | 规则 oracle；无 stable `ItemInstanceId`、generation、codec、container owner、产品 caller |
| `inventory_instance_ledger.zr:197-262` | `countFit`、`addStacked`、`addItemInstance` | 线性扫描、clone、容量由 caller 约定，overflow 无 container/receipt |
| `quest_state.zr:5` | 明确写着不导入 main/world；A/B arrays 和 `q_boars`/`q_wolves` 特判 | 不可达 fixture；不能称为 QuestRuntime |
| `crafting_transaction.zr:1-8` | 明确将 hub/item/gold/XP/host throttle 留给 host | 局部 transaction 规则，不拥有真实 inventory/currency/instance/persistence |
| `crafting_transaction.zr:391-480` | ordered gates 和一次 RNG draw | 缺 reservation、write-set、idempotency、output ItemFactory、outbox |
| `talent_allocation_commit_state.zr:58-87` | allocation scalar、lock 和 revision | effect flags 是计划，不是跨 combat/equipment/pet/charge/aura 的原子提交 |
| `bank_state`/`trade_state`/`market_state` | 平行数组与本地状态机 | 不可达、无 durable store、escrow、CAS、reconnect 或 cross-process owner |
| `xp_state`/`deed_completion_state` | 被 lifecycle self-test 拉入 whole-root | 不是普通 fixed-tick owner，缺事件 journal 与 level transition product caller |

### 2.4 协议和 host 不能反证脚本已交付

`generated_command_payloads.rs` 已列出 market、trade、bank、quest、craft、talent 等 command descriptors，客户端 `intent.rs` 也能编码对应 payload。这只证明 wire catalog，不证明 command reducer。`FixedServerTickDriver` 只负责 host accumulator、队列 admission、排序和把 bytes 送入 `WocTransactionalRuntime`；它不解释 progression，也没有当前 production VM 将 bytes 执行到 `world/state.zr`。`WocTransactionalRuntime` 的 rollback/usage/fault 类型是可复用外层底座，但不能把测试 VM 或 identity inspector 计入 progression 产品资格。

## 3. 与参考引擎的架构差距

### 3.1 内容身份与资源生命周期

Unreal `UAssetManager` 把 PrimaryAsset、异步 acquire、下载资源和 chunk 规则集中在一个可替换 manager；Godot `Resource` 持有 path/cache、修改时间和本地 scene 关联；Bevy `Handle` 是资源生命周期和加载状态的 typed identity。Zircon 当前以 string item ID、M5 uint code、generated `if` ladder 和本地数组并存，缺 `DefinitionId + Revision + BuildSet + artifact` 的唯一内容身份。重构必须先建立 typed content registry，再允许 progression state 引用 definition revision，不能在 WorldState 中追加更多 code 分支。

### 3.2 持久化、迁移与网络增量

Godot Resource 和 Fyrox Visitor 都把复杂对象树作为 schema-aware serialization，而不是将业务事实拼回平行数组。Unreal SaveGame 和 FastArraySerializer 分离持久对象与按 dirty item 的增量复制，FastArray 明确要求 item dirty 标记、删除通知和不保证数组顺序。当前 `saveState` 只替换 opaque string，WorldState codec 固定写入大量 primitive columns；instance、bank、trade、market 和 quest 不能在 reload/reconnect 后保持同一 identity。目标是 versioned component/container journal + generation-qualified delta，decode 保持纯，migration 单独产生 reconciliation receipt。

### 3.3 执行阶段与变更检测

Bevy Schedule 将系统执行顺序和 change detection 作为明确 world 机制；Unity Graphics RenderGraph 的 builder 先声明 resource read/write，再由 graph 验证和提交。两者共同说明“声明依赖、准备候选、验证、提交、发布”必须有阶段边界。当前 craft、talent、inventory 在一个脚本 tick 中直接顺序改写字段，失败点依赖调用者约定；没有 read/write set、CAS revision、bounded scratch、commit receipt 或 rollback journal。

### 3.4 标签和玩法规则身份

Unreal GameplayTags 使用稳定的 typed/tag-container 语义，避免将任意文本 branch 当作规则身份。当前 quest、recipe、item、talent 大量使用 string equality、uint code 与 catalog index 混合，generated tables 在热路径重复解释。应把 identity、权限、版本和显示 key 分离：runtime 只消费 prepared numeric handles，Editor/UI 才把它们投影成本地化文本。

## 4. P0：必须先硬阻断的断链

### WOC-PROG-P0-001 · Progression owner graph 不可达

55 个非 test-main 模块只有 15 个进入普通 fixed tick，WorldState 同时承载 item、inventory、craft、quest、talent scalar 真相。必须生成 owner/reachability manifest，建立唯一 `WocProgressionRuntime`；不可达模块只能作为迁移 oracle 或独立测试，不能宣称 supported。

### WOC-PROG-P0-002 · ItemInstance 无法持久化和迁移

当前产品只保存 item code/count/manual slot；`ItemInstanceData` 的 signer、rolled/masterwork、enchant、binding、charges 没有进入 WorldState codec、equipment、loot、trade、market、bank、save 或 replication。必须建立 qualified `ItemDefinitionId`、`ItemInstanceId`、revision/generation 与 versioned payload，未知 schema 要 fail-closed 或显式 migration。

### WOC-PROG-P0-003 · Inventory/Economy 没有原子、持久、幂等 transaction

grant/remove/move/equip/vendor/bank/trade/market/craft/reward 仍由平行数组和调用顺序维护；没有跨 container/currency revision CAS、escrow write-set、outbox 或 replay receipt。任何 throw、disconnect、duplicate command、budget exceeded 都必须证明 zero partial mutation 和 zero duplicate。

### WOC-PROG-P0-004 · 内容规模没有统一 BuildSet/artifact qualification

M5 content 只是一组有界 scalar 投影，current-head reference 有 580 条 item-level records，而 generated catalogs 以数千至上万 `if` 分支散落。必须将 item/recipe/quest/talent definition 编译为 versioned artifact，列出 implemented/unsupported、owner、schema、source digest、target/backend compatibility；加载前完整校验，不能以存在的目录文件发布 Ready。

### WOC-PROG-P0-005 · Quest/Talent lifecycle 依赖 fixture 和单玩家假设

Quest fixture 只有 A/B 两个玩家和两个 quest code；WorldState 仍可能通过第一个 player entity 推断 primary player。Talent effect flags 只规划 effect，未和 combat/equipment/pet/aura/charge 事务接线。必须使用 authenticated character/party handle、definition graph、journal event idempotency 和完整 effect plan。

### WOC-PROG-P0-006 · 没有可执行 progression parity/release qualification

46 个 test-main 不等于产品 runner，native production 没有 `WocProjectVm` 实现，16 个 parity owner 没有从真实产品 root 执行的 receipt。必须恢复 generated test catalog 和真实 runner，覆盖 command/tick/save/reload/reconnect/duplicate/fault/scale；missing、skipped、not-run 或 zero assertion 都是 Fail。

## 5. P1：必须重构的内容

### 5.1 Identity、Container、Item 与 Equipment

| ID | 当前差距 | 重构要求 |
|---|---|---|
| WOC-PROG-P1-001 | string、uint code、catalog index 混用 | `ItemDefinitionId + ItemRevision + BuildSet` typed registry，wire 只传 validated compact handle |
| WOC-PROG-P1-002 | 82 项 M5 投影与 current-head records 无共同发布物 | 生成 artifact manifest、compatibility matrix、last-good/lifecycle receipt |
| WOC-PROG-P1-003 | generated catalog 是巨型 `if` ladder | cook 成 immutable SoA/packed table/perfect hash，load 时验证并生成 typed view |
| WOC-PROG-P1-004 | instance 没 stable ID/generation | 创建时分配不可复用 qualified identity，所有 container/receipt 引用 generation-safe handle |
| WOC-PROG-P1-005 | instance payload 仅在不可达 ledger | payload 贯穿 codec、equipment、loot、bank、trade、market、save、replication |
| WOC-PROG-P1-006 | durability、repair、provenance、creation reason 无统一 schema | componentized optional fields、validation、migration 与 unknown-field policy |
| WOC-PROG-P1-007 | fungible/instance stack 由 caller 约定 | definition/revision/traits/binding/enchant 编译 stack key，非同质物禁止合并 |
| WOC-PROG-P1-008 | inventory 仅角色 CSR，bank/mail/escrow/corpse 无统一 owner | `ContainerId` + principal + capacity + revision + persistence scope |
| WOC-PROG-P1-009 | manual slot/layout 可能突破 capacity | layout 变成 view，overflow 进入 mailbox/claim container 并有 expiry/receipt |
| WOC-PROG-P1-010 | equipment 是有限 slot 投影且与 inventory 分开写 | typed compatibility、requirements、two-hand/offhand、set state 同一 transaction |
| WOC-PROG-P1-011 | equip 后 stats/abilities/procs 分散刷新 | 编译 `EquipmentEffectPlan`，一次发布 attribute/proc/known-ability revisions |
| WOC-PROG-P1-012 | hot reload 不明确 instance content revision | migrate、lease-old 或 reject，禁止静默替换规则 |

### 5.2 Inventory、Currency、Vendor、Bank、Trade、Market

| ID | 当前差距 | 重构要求 |
|---|---|---|
| WOC-PROG-P1-013 | CSR 删除会搬移所有后续实体 offset | per-container paged/chunked store + stable slot handle，barrier 批量提交 |
| WOC-PROG-P1-014 | count/capacity/layout 多次线性扫描 | item index、used-slot counter、dirty layout cache，给大背包/多角色斜率门 |
| WOC-PROG-P1-015 | grant 默认绕过 capacity | destination plan 显式选择 inventory/overflow/mailbox/reject 并返回 receipt |
| WOC-PROG-P1-016 | remove 没有具体 instance/slot reservation | 预留具体 identity 和 count，commit 返回被消费 identity |
| WOC-PROG-P1-017 | currency 只有 non-negative copper | typed currency ledger、source/sink、上限、balance revision、double-entry journal |
| WOC-PROG-P1-018 | fee/price/reward policy 分散 | 编译 rounding/saturation/tax/discount policy，记录 quoted/committed price |
| WOC-PROG-P1-019 | vendor stock/buyback 是本地数组 | vendor catalog revision、restock clock、durable buyback container |
| WOC-PROG-P1-020 | buyback 只存 code/count | 转移原 instance 或 stack fragment，不重建替代物品 |
| WOC-PROG-P1-021 | bank_state 不可达 | account/character/tab scoped durable service，deposit/withdraw 具 CAS |
| WOC-PROG-P1-022 | trade 无 session/reconnect/confirm revision | `TradeSessionId`、participant lease、offer lock、timeout/cancel 状态机 |
| WOC-PROG-P1-023 | trade preflight 后顺序 apply | 双方 container/currency revision 一次 CAS，失败零变更 |
| WOC-PROG-P1-024 | market 用局部 int ID 和平行 arrays | qualified ListingId、escrow、price/quantity/revision/expiry/state machine |
| WOC-PROG-P1-025 | market search 文本特判和全表扫描 | normalized index、pagination、stable sort、permission 与 result budget |
| WOC-PROG-P1-026 | expiry 全表扫描且交付不恢复 | expiry index + outbox，sale/cancel/expire 幂等可重放 |

### 5.3 Loot、Crafting、Gathering、Profession

| ID | 当前差距 | 重构要求 |
|---|---|---|
| WOC-PROG-P1-027 | loot entitlement/roll/award 分散在 WorldState | `LootSession` 拥有 source、eligible principal、policy、deadline、terminal receipt |
| WOC-PROG-P1-028 | pending/candidate 固定上限且超限策略不透明 | workload cap + typed Reject/Defer/Overflow，不静默丢候选 |
| WOC-PROG-P1-029 | party/member 反复线性扫描 | generation-safe participant snapshot + indexed choice |
| WOC-PROG-P1-030 | loot grant 可能越过 capacity | inventory/mailbox/claim 原子选择，断线仍唯一交付 |
| WOC-PROG-P1-031 | master loot/FFA/distribution 不在产品图 | registered policy strategy 进入同一 LootSession |
| WOC-PROG-P1-032 | recipe 只有两个 projection，至少一项永久 unsupported | generated recipe registry 为每项声明 handler/station/profession/output/status |
| WOC-PROG-P1-033 | craft 材料/费用没有 reservation | 锁定具体 reagent instance、currency 和 revision 后统一 revalidate |
| WOC-PROG-P1-034 | unique output 不能创建 signed rolled instance | `ItemFactory` 用 qualified RNG 生成 payload/provenance/receipt 后一次插入 |
| WOC-PROG-P1-035 | RNG purpose 隐含且只用共享 cursor | world/character/transaction/recipe/purpose 分流，记录 draw count/result |
| WOC-PROG-P1-036 | skill/quest/deed hook 分散写 | craft journal 派生 XP/objective/deed events，受同一 transaction/outbox 约束 |
| WOC-PROG-P1-037 | gathering/enchant/salvage/tool effects 不可达 | 统一 profession operation core，共享 reservation/RNG/cooldown/receipt |
| WOC-PROG-P1-038 | harvest node/station 没有 world partition owner | generation-safe node/station、claim lease、respawn 和 partition persistence |

### 5.4 Quest、XP、Deed、Talent

| ID | 当前差距 | 重构要求 |
|---|---|---|
| WOC-PROG-P1-039 | quest 只按两个 code 保存标量 | versioned `QuestDefinition` + per-character `QuestInstance` objective graph |
| WOC-PROG-P1-040 | primary player 依赖 entity 扫描顺序 | command 必须携 authenticated character handle |
| WOC-PROG-P1-041 | generic quest module 不可达 | 唯一 QuestRuntime 进入 schedule 并输出 typed outcome |
| WOC-PROG-P1-042 | kill/collect/craft/gather credit 无统一 journal | qualified event + objective index + event idempotency |
| WOC-PROG-P1-043 | quest item counter 与 inventory migration 可能合成 | requirement 从 inventory index 派生，migration 产生 reconciliation report |
| WOC-PROG-P1-044 | party share/link 没有统一 owner | participant snapshot、range/phase/tag/ownership policy 和 member result |
| WOC-PROG-P1-045 | reward 直接写 copper/XP/items | 编译为 ProgressionTransaction，满包/level-up/retry 共用 receipt |
| WOC-PROG-P1-046 | XP/deed 只由 lifecycle self-test 导入 | 注册为 progression system，明确事件和 schedule phase |
| WOC-PROG-P1-047 | level-up/ability refresh 手写耦合 | `LevelTransitionPlan` 声明 curve、overflow、stats、abilities、rewards |
| WOC-PROG-P1-048 | talent 有 fixture/commit/modifier/WorldState 多份 authority | 单一 TalentRuntime 管 definition/allocation/loadout/compiled effects/migration |
| WOC-PROG-P1-049 | lock 缺 authoritative combat/session tags 和 expiry | admission 读取 typed tags，返回稳定 lock reason |
| WOC-PROG-P1-050 | talent effect flags 未原子执行 | candidate 编译 modifier/stat/ability/proc/charge/aura/equipment/pet/echo plan |
| WOC-PROG-P1-051 | modifier 每次战斗重扫 189 entries | allocation revision 生成 immutable snapshot，hot path 只取 prepared block |
| WOC-PROG-P1-052 | nested effect/proc 在 WorldState 再手写 | compiled talent output 注册到 CombatBuildSet，combat 不再解释 catalog |

### 5.5 Persistence、Replication、Performance、Security、Evidence

| ID | 当前差距 | 重构要求 |
|---|---|---|
| WOC-PROG-P1-053 | schema 散落 WorldState version branches | 独立 versioned components + migration DAG，snapshot 绑定 BuildSet |
| WOC-PROG-P1-054 | decode 可能补业务事实 | decode 只恢复 bytes，reconciliation/migration 独立事务 |
| WOC-PROG-P1-055 | inventory/economy 无增量 replication | component/container revision + delta journal + ack/resync/interest |
| WOC-PROG-P1-056 | command 缺 principal/replay boundary | authenticated principal/character、nonce/sequence/capability/expected revision |
| WOC-PROG-P1-057 | grant/trade/market/craft 无 anti-dupe invariant | instance conservation、idempotency、escrow/outbox audit |
| WOC-PROG-P1-058 | generated branch table 在 tick 重复解释 | load/cook 建 indexed prepared registry，operation 零文本扫描 |
| WOC-PROG-P1-059 | clone/Array 构造没有 operation memory budget | transaction scratch arena，bytes/items/ops 超限在 mutation 前拒绝 |
| WOC-PROG-P1-060 | CSR/expiry/loot 没有复杂度门 | 报告 p50/p99、alloc、bytes moved 和规模斜率 |
| WOC-PROG-P1-061 | contractTest 混入候选生产模块 | 移到独立 test package，release symbol inventory 拒绝测试代码 |
| WOC-PROG-P1-062 | test-main 与 manifest 无生成 inventory | catalog 记录 owner/entry/artifact/backend/oracle/required 状态 |
| WOC-PROG-P1-063 | parity owner 缺失 | 产品 root runner 覆盖 save/reload/reconnect/duplicate/full/conflict |
| WOC-PROG-P1-064 | 无 progression transaction/economy diagnostics | 记录 reads/writes、revision conflict、RNG、latency、allocation、receipt、first invariant failure |

## 6. P2：在核心闭环后处理的长期能力

P2 保留旧 Runtime14 的 15 项：大规模 secondary index、跨分片 market/economy、通胀 simulation、loot/craft Monte Carlo、按 read/write set 并行、online instance migration、Editor inventory/economy inspector、quest/talent 状态图、transaction replay、mailbox 治理、offline bounded catch-up、localized display key、schema-guided fuzz、自动 reachability delta、与 Unreal 同 oracle 的 progression benchmark。P2 不能在 P0/P1 未闭合时通过更多规则文件抢跑。

## 7. Owner 与依赖收敛

| Owner | 唯一职责 | 禁止承担 | 依赖 |
|---|---|---|---|
| `ProgressionBuildSetRegistry` | item/recipe/quest/talent/economy schema、revision、artifact组合 | mutable player state、运行时文本分支 | Tooling05 |
| `ItemRegistry` | prepared definition、instance schema、qualified handle | container mutation、reward policy | BuildSet、Runtime04/24 |
| `ContainerStore` | inventory/equipment/bank/mail/escrow/corpse、revision | vendor/quest/craft业务规则 | Runtime12/24 |
| `ProgressionTransactionEngine` | reservation、read/write set、CAS、commit、receipt、outbox | VM snapshot、网络认证 | App03、Runtime12 |
| `EconomyRuntime` | currency/vendor/bank/trade/market state machine | item definition、UI | Transaction、ContainerStore |
| `LootProfessionRuntime` | loot/craft/gather/enchant/salvage plan | combat damage | ItemFactory、Runtime13 |
| `QuestProgressionRuntime` | quest/XP/level/deed instance 和 event consumption | 第一个 player 推断 | character/party journal |
| `TalentRuntime` | allocation/loadout revision、prepared effects、migration | combat opcode执行 | Runtime13、BuildSet |
| `ProgressionEvidenceRunner` | unit/contract/parity/save-reload/perf/fault qualification | 第二套玩法公式 | App03、source golden |

依赖顺序必须是 Runtime12/App03 先提供 typed collection、generation、snapshot、outer transaction；Tooling05 产出同代 BuildSet；然后 Item/Container/Transaction，再接 Economy/Loot/Profession，之后 Quest/XP/Talent，最后才开放 UI、online 和 competitive performance。不得先把 market/craft/talent 按钮接到 WorldState 再承诺未来补 transaction。

## 8. 重构里程碑

1. **M0 owner/reachability freeze**：生成 55 个非测试模块的 production/test/fixture 分类、产品 root reachability、16 个 parity oracle 和 item/recipe/quest/talent capability matrix；禁止新增 WorldState progression 字段。
2. **M1 Item/Container/Transaction**：建立 qualified definition/instance/container identity、revision/CAS、reservation、receipt、outbox 和 conservation audit；迁移 scalar inventory/equipment/overflow 并删除 CSR authority。
3. **M2 Economy/Loot/Profession**：实现 vendor/bank/trade/market escrow、currency journal、loot session、craft ItemFactory、RNG receipt 和 profession operation；所有失败零变更。
4. **M3 Quest/XP/Talent**：把 objective graph、event journal、level transition、allocation/loadout 和 compiled effects 接入 schedule；删除 A/B player 与 fixture authority。
5. **M4 Persistence/Replication/Host**：实现 component snapshot/migration、delta journal、principal admission、真实 production VM、server/client/headless startup 和 reconnect/replay。
6. **M5 Qualification**：从同一产品 root 执行 16 个 parity 场景及 1/100/1k scale、fault、soak、memory、latency workload；生成 source/BuildSet/schema/backend/seed/hardware/first-diff receipt。

## 9. 资格门

| Gate | 当前状态 | 必须证明 |
|---|---|---|
| PROG-G01..G04 | Fail | owner reachability、qualified identities、instance preservation、all operation transaction receipts |
| PROG-G05..G07 | Fail | fault/duplicate zero partial mutation、ledger conservation、explicit overflow/mailbox |
| PROG-G08..G10 | Fail | BuildSet capability status、Quest/Talent lifecycle、complete effect plan |
| PROG-G11..G12 | Fail | no generated-text hot path、container/catalog scale slope |
| PROG-G13..G16 | Fail | executable test catalog、16 parity owners、schema/revision retention、review/index consistency |

## 10. Review 边界与当前状态

- 本轮只做 current-source 静态 review 和 refactor plan；没有修改 production/test/manifest/generated artifact，也没有运行 Cargo、ZrVM/native DLL、真实 server/client/editor、fault、fuzz、scale、soak 或动态 benchmark。
- `examples/woc/native` 当前工作树有大量并行修改；本文只采纳可重复的文件和行证据，不把 dirty diff 当作已集成能力。
- Tooling 未来迁移 Rust 的范围按用户要求不在本轮实施；Tooling05 只作为 generated artifact owner 依赖引用。
- native identity inspector 成功只表示 manifest/source commit/target plugin selection 一致，不能表示 VM、WorldState 或 progression provider 已运行。
- 由于没有生产 `WocProjectVm`、没有真实 product root runner 和所有 P0 都 Open，本报告 `implementation_status` 保持 `pending`，`source_recheck_required` 保持 `true`。
