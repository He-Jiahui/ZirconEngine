---
related_code:
  - examples/woc/scripts/woc_game/src/main.zr
  - examples/woc/scripts/woc_game/src/world/state.zr
  - examples/woc/scripts/woc_game/src/progression
  - examples/woc/scripts/woc_game/src/progression/m5_inventory_rules.zr
  - examples/woc/scripts/woc_game/src/progression/inventory_instance_ledger.zr
  - examples/woc/scripts/woc_game/src/progression/inventory_vendor_state.zr
  - examples/woc/scripts/woc_game/src/progression/equipment_routing_state.zr
  - examples/woc/scripts/woc_game/src/progression/bank_state.zr
  - examples/woc/scripts/woc_game/src/progression/trade_state.zr
  - examples/woc/scripts/woc_game/src/progression/market_state.zr
  - examples/woc/scripts/woc_game/src/progression/loot_distribution_state.zr
  - examples/woc/scripts/woc_game/src/progression/loot_roll_runtime.zr
  - examples/woc/scripts/woc_game/src/progression/craft_item_state.zr
  - examples/woc/scripts/woc_game/src/progression/crafting_transaction.zr
  - examples/woc/scripts/woc_game/src/progression/gathering_state.zr
  - examples/woc/scripts/woc_game/src/progression/enchanting_state.zr
  - examples/woc/scripts/woc_game/src/progression/salvage_state.zr
  - examples/woc/scripts/woc_game/src/progression/quest_state.zr
  - examples/woc/scripts/woc_game/src/progression/xp_state.zr
  - examples/woc/scripts/woc_game/src/progression/talent_state.zr
  - examples/woc/scripts/woc_game/src/progression/talent_modifier_state.zr
  - examples/woc/scripts/woc_game/src/progression/talent_allocation_commit_state.zr
  - examples/woc/scripts/woc_game/src/progression/talent_world_commit_state.zr
  - examples/woc/scripts/woc_game/src/progression/talent_loadout_migration.zr
  - examples/woc/scripts/woc_game/src/progression/deed_completion_state.zr
  - examples/woc/scripts/woc_game/src/generated/m5_content_catalog.zr
  - examples/woc/scripts/woc_game/src/generated/current_talent_selection_catalog.zr
  - examples/woc/scripts/woc_game/src/generated/talent_modifier_catalog.zr
  - examples/woc/scripts/woc_game/src/generated/talent_proc_catalog.zr
  - examples/woc/scripts/woc_game/src/generated/item_level_catalog.zr
  - examples/woc/reference/current-head/item_level_catalog_contract.json
  - examples/woc/reference/current-head/parity_scenarios.json
  - examples/woc/reference/current-head/source_manifest.json
tests:
  - examples/woc/scripts/woc_game/woc_game_tests.zrp
  - examples/woc/scripts/woc_game/woc_world_state_tests.zrp
  - examples/woc/scripts/woc_game/woc_m5_inventory_instance_ledger_tests.zrp
  - examples/woc/scripts/woc_game/woc_m5_inventory_vendor_tests.zrp
  - examples/woc/scripts/woc_game/woc_m5_bank_round_trip_tests.zrp
  - examples/woc/scripts/woc_game/woc_m5_player_trade_tests.zrp
  - examples/woc/scripts/woc_game/woc_m5_market_round_trip_tests.zrp
  - examples/woc/scripts/woc_game/woc_m5_crafting_transaction_tests.zrp
  - examples/woc/scripts/woc_game/woc_m5_quest_state_tests.zrp
  - examples/woc/scripts/woc_game/woc_m5_talent_allocation_commit_state_tests.zrp
  - examples/woc/scripts/woc_game/woc_m5_xp_progression_tests.zrp
  - examples/woc/native/Cargo.toml
plan_sources:
  - docs/plans/optimize/zircon_app/03-woc-product-role-host-zrvm-transaction-state-client-server-integration-review.md
  - docs/plans/optimize/zircon_runtime/08g-gameplay-ability-effect-attribute-tag-cue-prediction-runtime-review.md
  - docs/plans/optimize/zircon_runtime/12-woc-zrvm-package-kernel-world-state-schedule-serialization-runtime-review.md
  - docs/plans/optimize/zircon_runtime/13-woc-combat-casting-effect-aura-damage-threat-death-runtime-review.md
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
  - dev/godot/core/io/resource.cpp
  - dev/godot/modules/multiplayer/scene_multiplayer.cpp
  - dev/Fyrox/fyrox-core/src/visitor/mod.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/IRenderGraphBuilder.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 14 · WOC Progression、Inventory、Item、Economy、Crafting、Quest 与 Talent Runtime 工程化差距

## 1. 结论

WOC progression 目录不是空壳。它保存了物品实例、背包、装备、vendor、bank、trade、market、loot、craft、gather、enchant、salvage、quest、XP、talent、deed/title 等大量 current-head 规则投影，物理量达到 101 个 `.zr`、12,849 行、486,872 bytes。去掉 46 个 `*_test_main.zr` 后仍有 55 个候选生产/共享模块、12,082 行。这里面的 source-pinned contract 和边界样例应作为迁移 oracle 保留，而不是在重构时全部推倒。

真正的问题仍是“文件存在”与“产品执行”严重分裂。从整个 `src/main.zr` 沿静态 import 递归，只有 17 个 progression 模块可达，其中 `xp_state` 和 `deed_completion_state` 仅由 package lifecycle self-test 拉入。以 `world/state.zr` 正常 fixed-tick 生产段为根重新扫描，只剩 15 个模块、2,505 行、98,583 bytes；40/55 个非 test-main 模块、9,577 行、359,106 bytes 不在普通产品执行图中，约占 79.3%。断线集合包括 `inventory_instance_ledger`、bank、trade、market、通用 quest、crafting transaction、gathering、enchanting、salvage、loot distribution、talent allocation/world commit、talent migration、XP 和 deed completion。产品真实 authority 仍是 68,000 行级 `world/state.zr` 的平行数组与手写 reducer。

这个差距已经直接限制正确性。产品库存只保存 `itemCode/count/manualSlot`，源码注释明确写着实例 payload 要等 ZrVM 能持久化 structured values；候选 `inventory_instance_ledger` 虽表达 signer、charges、rolled stats、masterwork、enchant、binding，却不在产品图中。`craft_item_state` 只识别两份 recipe，`ritual_vestments` 因需要 signed rolled instance 被明确拒绝，实际成功路径只剩 minor healing potion。`grantM5InventoryItem` 可在容量不足时保留 overflow，布局函数再为未放置 stack 扩展 cell；loot award 没有 mailbox/escrow/claim owner。移动一个全局 CSR stack 还会更新其后所有 entity offset，无法支撑大世界、多角色和高频交易。

银行、玩家交易、市场与通用 crafting 的代码也不能被当作已交付服务。`trade_state` 自己声明不被 main/world 导入，`quest_state` 声明只是 host facts 缺失时的 source-contract fixture；bank/market/trade 都是进程内平行数组候选状态，没有 durable identity、CAS、幂等 command、escrow journal、断线恢复、跨 world owner 或审计账本。WorldState 的 craft 路径先扣铜币、再逐个消耗材料、抽 RNG、发物品和加技能，只有外层 VM transaction 才可能阻止中途异常发布，progression 域本身没有可重放的 write-set/receipt。

Quest 与 talent 同样只有有界离线子集。产品任务只保存 `q_boars` 与 `q_wolves` 两组标量，并用“第一个 kind=player 的 entity”决定 primary player；通用 accept/link/abandon/kill/collect/turn-in 模块不可达。产品天赋提交直接修改 spec 和六行选择，只刷新 known abilities、charge cap 与 derived stats；不可达的 candidate commit 反而列出了 proc cleanup、form aura cleanup、offhand revalidation、pet dismissal、Temporal Echo cleanup 等完整影响。更糟的是 modifier snapshot 没有按 allocation revision 缓存，产品多个战斗路径会重复构造状态并线性扫描 189 项 generated modifier catalog。

内容规模也被 M5 投影硬截断。`m5_content_catalog.zr` 有 5,332 行和 4,264 个 `if`，但产品表只覆盖 82 items、2 quests、1 mob、6 NPC、3 talent options、3 abilities、2 specs；current-head item-level 合同则记录 580 条 catalog records、395 个 source items、331 个有 item level。580 不是“580 个都已可直接装备和交易”的证明，但足以说明 82 项 M5 表只是有界子集，不能成为长期 runtime authority。生成目录中的多份表又以数千到上万条分支实现 lookup，热路径重复扫描会把 codegen 体积直接转换成 tick 成本。

本轮登记 **6 项 P0、64 项 P1 和 15 项 P2**。Runtime12继续拥有 WOC world storage、schedule、snapshot 与 migration 基础，Runtime13拥有 combat transaction，App03拥有 VM/host 外层事务，Tooling05拥有 generated artifact 编译形态。本篇唯一拥有 WOC progression 语义收敛：qualified item/instance/container identity、原子 inventory/economy transaction、durable bank/trade/market/loot/craft、typed quest/XP/talent lifecycle，以及这些能力进入产品根与可执行 parity/performance 资格。

## 2. 审查边界与证据

### 2.1 物理范围与产品可达性

| 集合 | 文件 | 行 | bytes | 结论 |
|---|---:|---:|---:|---|
| `progression/*.zr` 全量 | 101 | 12,849 | 486,872 | 含生产候选和独立 test main |
| 非 `*_test_main.zr` | 55 | 12,082 | 457,689 | 67 classes、573 个 `pub var`、466 个 public functions |
| `*_test_main.zr` | 46 | 767 | - | 独立入口，不代表聚合或产品执行 |
| 从整个 `src/main.zr` 可达 | 17 | 2,905 | 112,089 | 包含 lifecycle self-test 拉入的 XP/deed |
| 普通 fixed-tick 生产图可达 | 15 | 2,505 | 98,583 | 以 `world/state.zr` 生产段 import 为根递归 |
| 普通 fixed-tick 图不可达 | 40 | 9,577 | 359,106 | 占候选代码约 79.3%，含核心 progression owners |

普通产品图中的 15 个模块主要是 `craft_item_state`、`harvest_node_state`、`loot_roll_runtime`、`m5_equipment_state`、`m5_inventory_rules`、`player_stat_input_state`、`profession_action_xp`、`stat_core_rules`、`talent_modifier_state`、`town_focus_state/wire_state`、`vendor_stack_state`、`weapon_skin_state` 和少量 deed/item-level rule。它们多数作为 scalar helper 或 copy-in/copy-out adapter 被 WorldState 调用，并不形成独立 progression runtime。

普通执行图外的 40 个模块是：

- Item/inventory/economy：`inventory_instance_ledger`、`inventory_order_state`、`inventory_vendor_state`、`equipment_routing_state`、`bank_state`、`trade_state`、`market_state`、`money_format_state`、`persisted_resource_state`、`supply_state`。
- Loot/craft/profession：`loot_distribution_state`、`loot_ffa_state`、`master_loot_state`、`crafting_hub_state`、`crafting_transaction`、`professions_craft_fixture`、`gathering_state`、`enchanting_state`、`salvage_state`、`mobile_crafting_station_state`、`tool_effect_state`、`masterwork_item_budget`、`masterwork_rules`。
- Quest/progression：`quest_state`、`quest_fallback_state`、`xp_state`、`battlefield_xp_state`、`deed_completion_state`、`deed_join_repair_state`、`archetype_state`、`combo_eligibility`、`cooldown_persist_state`。
- Talent/content：`talent_state`、`talent_allocation_commit_state`、`talent_world_commit_state`、`talent_loadout_migration`、`talent_row_selection`、`m5_scenario_matrix`、`item_level_catalog_state`、`item_score_state`。

前两项 whole-root 额外可达模块只来自 lifecycle self-test，因此不能用 `main.zr` 的 17 项数字宣称 XP/deed 已接普通 tick。反过来，静态不可达也不能证明模块内每条规则错误；它证明的是这些规则目前不是产品 authority。

### 2.2 代码形状与测试代码混入

| 信号 | 当前值 | 工程含义 |
|---|---:|---|
| classes / `pub var` / public functions | 67 / 573 / 466 | 状态多由公开可变字段和调用约定维持不变量 |
| `throw` | 17 | 多数仍是字符串，缺稳定 transaction/result taxonomy |
| `new container.Array` | 157 | candidate copy与查询可能在命令或战斗热路径分配 |
| `while` | 216 | 大量平行数组扫描、搬移和 catalog lookup |
| `.indexOf` / `.removeAt` | 18 / 43 | 显式线性查找和全局密集数组搬移 |
| 字符串相等分支 | 206 | item/recipe/quest/talent/economy identity仍常以文本参与执行 |
| 含 `contractTest` 的非test-main模块 | 48/55 | 测试符号与生产候选同文件 |
| `contractTest` 尾段 | 2,001 行 | 占非test-main代码 16.56% |

15 个非 test-main 模块没有 matching test main，7 个模块没有 `contractTest`。这不是要求每个文件都机械一测一，而是说明当前既没有以产品 owner 为中心的测试 inventory，也不能用目录里的 contractTest 数量替代 integration/parity 证据。

### 2.3 WorldState 才是当前 progression authority

名称审计显示 WorldState 519 个公开字段中至少 100 个带 quest/gather/deed/item/focus/craft/talent/level/inventory/vendor/loot 等 progression/economy 信号；生产段还有约 280 个函数名匹配这些主题。该数字只是保守的命名信号，不是正式 component taxonomy，但足以显示业务状态仍被压进单一 world owner。

当前真实状态形状包括：

- Quest：两组离线 quest state/objective 标量、quest copper/XP，只有 boars 与 wolves；primary player 是首次扫描到的 player entity。
- Inventory：每 entity copper/bag codes，所有角色共享 flattened stack offsets/item codes/counts/manual slots；没有 item instance payload columns。
- Vendor：共享 flattened buyback offsets/item codes/counts；插入和删除更新所有后续 entity offsets。
- Equipment：M5 投影主要暴露 helmet、feet、mainhand 等有界 slot/code，候选 equipment routing owner 不可达。
- Craft：一份全局结果、throttle、alchemy skill 与两个 recipe code；其中一份 recipe 永远返回 unsupported instance output。
- Talent：每 entity spec、6 row options、loadouts以及大量战斗派生列；提交与派生刷新手写在 WorldState。
- Loot：pending roll/candidate/choice/result 使用多组 flattened arrays，存在 1,024 pending 与 10,240 candidate 固定上限。

`m5InventoryRemoveStack` 会从三个全局数组 `removeAt`，再递减当前 entity 后的全部 offset。`grantM5InventoryItem` 刻意不执行容量 gate，注释把 overflow 视为强制奖励保留；layout 为 unplaced stacks 追加 cell。该策略避免静默丢物品是可保留的正确意图，但必须由明确的 overflow container/mailbox/claim policy 表达，不能让 UI cell 数突破 inventory capacity 成为持久语义。

### 2.4 Item instance 与事务断层

不可达的 `inventory_instance_ledger` 已经表达 `signer`、rolled quality/stats/masterwork、enchant、`boundTo`、charge IDs/counts，以及 fungible/instance 分离。这是有价值的 source-contract oracle。但它使用局部 object arrays、文本 item ID、线性搜索和可变 clone，既未进入 WorldState codec，也没有 qualified instance identity、durability/provenance/revision/container owner。

产品 `removeM5InventoryItem` 的注释明确说明 instance payload 仍在 compact scalar partition 之外；`m5_inventory_rules` 又明确说明 WorldState继续持有平行数组，直到 ZrVM cross-module Array ABI 可靠。这个问题不能继续由 WOC 业务代码绕行：Runtime12/App03必须先提供可持久、可事务传递的 typed collection/component边界，本篇再把 item instance 作为唯一 progression owner 接入。

Craft路径的当前顺序是：preflight材料和throttle，饱和扣费，逐项消耗reagent，消费一次共享RNG，grant scalar result，加alchemy skill，写result。饱和扣费是 source-pinned policy，不应被误报为当前实现bug；但整段缺少明确 reservation/write-set/journal，且 masterwork signed instance 路径被拒绝。Progression transaction必须能独立证明相同 command idempotency、RNG receipt、库存/货币/技能/任务hook全有或全无；App03只负责其外层VM发布。

### 2.5 Catalog、talent 与热路径

| Artifact | 行 / bytes | 分支形状 | 结论 |
|---|---:|---:|---|
| `m5_content_catalog.zr` | 5,332 / 234,177 | 4,264 `if` | 仅82 items、2 quests等有界M5投影 |
| `current_talent_selection_catalog.zr` | 792 / 50,717 | 756 `if` | 27 specs、162 options，仍是分支式lookup |
| `talent_modifier_catalog.zr` | 13,022 / 628,513 | 12,931 `if` | 189 entries、18 stat fields、39 global fields等 |
| `talent_proc_catalog.zr` | 2,236 / 103,385 | 2,093 `if` | proc规则分支生成 |
| `item_level_catalog.zr` | 759 / 39,688 | 740 `if` | 740次直接文本比较式lookup |

Tooling05拥有“生成器为何产出巨型if ladder”的修正；本篇拥有runtime不得在每次cast/damage/proc或inventory命令中反复解释这些表。当前 `TalentModifierState.findEntry` 对最多7个selection线性扫描189项，并逐项遍历18 stat、39 global和ability/nested effect字段；WorldState有多处构造/重算调用。应在loadout commit时编译并缓存 immutable modifier/proc/ability snapshot，以 allocation revision 失效，而不是把 catalog 规模乘到每次战斗操作。

### 2.6 可执行证据状态

- progression 有46个test main；45份 `.zrp` 指向45个唯一entry和45个唯一binary目录，当前这些binary目录均不存在。`stat_core_rules_test_main` 没有matching manifest。
- 这只能表述为“当前树没有checked-in executable artifact evidence”，不能反推历史上从未执行。
- `parity_scenarios.json` 中与本域直接相关的16项是 `solo_warrior`、`pet_commands`、`duel_to_winner`、`party_loot`、`l1_loot_distribution`、3个quest、talents、delve、market、inventory/vendor、bank、XP/prestige、trade、professions craft；16个 `woc_owner` 当前全部不存在。
- reference golden可以作为上游oracle，但owner缺失、runner未执行或native compile未通过时不能记为pass。
- native workspace当前仍在 `woc_protocol` 的6个既有编译错误处停止，测试没有开始；本轮不重复无变化失败lane。

### 2.7 参考引擎路由边界

Unreal `UAssetManager` 的 PrimaryAsset identity/rules、`USaveGame` 的持久对象边界、FastArray delta serializer 与 GameplayTag identity展示了内容身份、保存和增量复制不应由玩法字符串分支临时拼接。Bevy的typed asset handles、change detection与compiled Schedule说明读取、写入、变更和执行顺序应显式化。Godot Resource/ResourceCache与SceneMultiplayer、Fyrox Visitor提供资源生命周期、序列化与连接状态参考。

Unity Graphics 的 RenderGraph builder只用于一个有限类比：操作先声明resource read/write access，验证后提交，生命周期结束即封闭；它不是inventory/economy语义来源。WOC current-head contracts仍是item、quest、craft和talent语义第一依据，参考引擎只提供owner、identity、transaction、persistence、replication与qualification机制。

## 3. 可保留的正确基础

### 3.1 Source-pinned contract保留了大量规则知识

模块注释、generated hash和contractTest对照指定source commit，能作为hard cut迁移的逐规则oracle。应把oracle移入独立test artifact并绑定BuildSet，而不是删除或继续混入release module。

### 3.2 Item instance候选模型已经识别核心非同质字段

signer、rolled stats/masterwork、enchant、binding和charges的区分方向正确。重构重点是稳定identity、container ownership、schema/persistence/replication与事务接线，不是回退成纯item code/count。

### 3.3 强制奖励不静默销毁的意图正确

容量不足时保留award优于直接丢弃，但需要明确进入overflow/mailbox/claim container，生成receipt并保证可恢复，而不是让布局数组突破容量。

### 3.4 Craft与talent已显式列出部分阶段

Craft具备preflight、fee、reagent、RNG、result和skill概念；talent candidate commit已列出完整side-effect flags。它们可直接转为prepared transaction plan和ordered commit阶段。

### 3.5 Generated catalog应成为编译输入而非运行时分支authority

现有catalog保留source映射和稳定code，可以迁移为sorted table/perfect hash/packed blobs与prepared handles。禁止为了性能手写第二份简化规则表。

## 4. P0：Progression产品化前必须硬阻断

### WOC-PROG-P0-001 · 产品authority与40个候选owner断线

55个非test-main模块只有15个进入普通fixed-tick图；bank/trade/market/quest/crafting transaction/item ledger/talent commit等核心owner不可达，产品行为另写在WorldState。相同语义同时存在候选module、WorldState有界实现和contract fixture，多份truth无法同步演进。

必须建立唯一 `WocProgressionRuntime` owner graph和production reachability manifest。每个item/container/economy/loot/craft/quest/XP/talent概念只能有一个生产owner；迁移oracle可以保留，但旧投影authority在hard cut后删除或降为纯test fixture。声明production的module若只被test import，构建直接失败。

### WOC-PROG-P0-002 · 产品无法持久化唯一ItemInstance

WorldState只编码item code/count/manual slot，明确排除structured instance payload；因此signed/masterwork/rolled/enchant/bound/charges无法在inventory、equipment、loot、trade、market、bank、craft、save和replication间保持同一件物品身份。`ritual_vestments`已因此被产品显式拒绝。

建立qualified `ItemDefinitionId`、`ItemRevision`、`ItemInstanceId`与generation-safe handle；实例schema至少覆盖rolled data、signer/provenance、binding、charges、durability、enchant、creation transaction和content revision。所有container、save、delta、market escrow和combat stat引用同一instance，未知schema fail-closed或显式迁移，禁止降级成同ID fungible stack。

### WOC-PROG-P0-003 · Inventory与economy没有原子、持久、幂等事务

当前WorldState直接搬移全局数组；候选bank/trade/market/craft虽有preflight/candidate字样，但不在产品图且只持有进程内局部状态。没有command identity、revision/CAS、reservation、escrow、durable journal、rollback/recovery、double-spend与anti-dupe proof。

所有grant/remove/move/equip/vendor/bank/trade/market/loot/craft/reward必须编译为 `ProgressionTransaction`：声明读写集与expected revisions，先reserve/validate，再按stable order原子commit，写immutable receipt与outbox。重复command返回同receipt；断线、崩溃、超时、跨world handoff和恢复不能多发或吞掉物品/货币。

### WOC-PROG-P0-004 · M5有界catalog与current-head内容规模未形成完整BuildSet

产品表只有82 items、2 quests、3 abilities等小子集，current-head item-level合同却有580条records；craft只有一份可成功recipe，通用quest/gather/enchant/market等仍是断线fixture。当前没有machine-readable capability matrix区分Implemented、Unsupported、ServerOnly、Fixture和NotWired。

建立 `ProgressionBuildSet`，绑定item/recipe/quest/talent/profession/loot/economy schema与source hashes；每条current-head记录必须有明确支持状态、owner、artifact、migration和test。未接线能力不能因目录里有同名module就在UI、命令或发布说明中显示available。

### WOC-PROG-P0-005 · Quest与talent生命周期不完整且依赖单玩家假设

产品quest只有两项并取第一个player；generic quest module不可达。Talent直接写spec/六行，只执行部分刷新，缺arena与战斗后5秒锁、proc/aura/offhand/pet等完整清理，且没有失败回滚。多人、角色切换、重连或同tick战斗事件会产生错误owner或半提交状态。

Quest必须以qualified character/party/quest instance/objective owner运行；talent必须以candidate allocation编译完整effect plan并在单一transaction提交。所有命令显式携带principal/character/revision，锁定条件和side effects由同一policy执行；第一个player、全局offline字段与手工清理路径必须hard cut。

### WOC-PROG-P0-006 · 当前没有可执行的progression parity与release资格

16个相关parity owner全部缺失，45个test manifest没有当前checked-in binary artifact，另有1个test main没有manifest；native workspace编译失败且测试未开始。目录中的48个contractTest和golden文件不能证明产品root、VM、codec、host与持久恢复后的行为。

恢复generated test catalog和真实runner，所有场景从产品root加载同一BuildSet，执行command/tick/save/reload/reconnect并比较typed transaction receipts、state projection与golden。missing/not-run/skipped/zero-assertion都不是pass；资格产物记录source、BuildSet、schema、backend、seed、hardware、actual和first diff。

## 5. P1：Item、Instance、Container 与 Equipment

| ID | 当前差距 | 需要的工程化重构 |
|---|---|---|
| WOC-PROG-P1-001 | item identity在string、catalog index、uint code间转换 | 生成namespace-qualified `ItemDefinitionId`与revision，wire/runtime使用validated compact handle |
| WOC-PROG-P1-002 | 82项M5表与580条current-head records没有同一BuildSet | BuildSet列出definition来源、支持状态、artifact hash与compatibility range |
| WOC-PROG-P1-003 | item definition通过巨型if逐字段查询 | cook为immutable SoA/packed table或perfect hash，load时全量验证并生成typed views |
| WOC-PROG-P1-004 | instance没有stable ID/generation | 每次创建分配不可复用qualified identity，所有receipt和container引用generation-safe handle |
| WOC-PROG-P1-005 | signer/rolled/enchant/bound/charges只存在不可达ledger | 提升为versioned `ItemInstancePayload`并贯穿codec、replication、trade与equipment |
| WOC-PROG-P1-006 | durability、repair、provenance和creation reason没有统一schema | componentized payload声明optional fields、defaults、validation、migration与unknown-field policy |
| WOC-PROG-P1-007 | fungible与instance stack规则靠调用者约定 | stack key由definition/revision/instance traits/binding/enchant等编译，非同质物禁止合并 |
| WOC-PROG-P1-008 | container只有角色CSR与局部fixture | `ContainerId`覆盖character/bank/mail/escrow/vendor/corpse/guild/world并绑定owner principal |
| WOC-PROG-P1-009 | manual slot与layout可突破capacity | slot/layout是view；overflow进入显式container并有claim/expiry/mailbox policy |
| WOC-PROG-P1-010 | equipment只覆盖有限slot并与inventory分离修改 | typed equipment schema、slot compatibility、requirements、two-hand/offhand与set状态同事务提交 |
| WOC-PROG-P1-011 | equip后stat/ability/proc刷新散落 | compile `EquipmentEffectPlan`，commit一次发布attribute/proc/known-ability dirty revisions |
| WOC-PROG-P1-012 | item与equipment引用content revision不明确 | 保存qualified definition revision；hot reload选择migrate/lease-old/reject，禁止静默换规则 |

## 6. P1：Inventory、Currency、Vendor、Bank、Trade 与 Market

| ID | 当前差距 | 需要的工程化重构 |
|---|---|---|
| WOC-PROG-P1-013 | 所有entity stack共享CSR，删除会搬移全局tail与后续offset | per-container paged/small-vector store加stable slot handles，结构变更在barrier批量提交 |
| WOC-PROG-P1-014 | inventory count/capacity/layout多次线性扫描 | 维护per-container item index、used slots与dirty layout cache，复杂度门覆盖大背包/多角色 |
| WOC-PROG-P1-015 | grant默认绕过capacity且无typed destination | grant plan显式选择inventory/overflow/mailbox/corpse/reject并记录原因与receipt |
| WOC-PROG-P1-016 | remove按倒序stack扫描但没有instance选择receipt | reservation固定具体slot/instance/count，commit与receipt返回被消费identity |
| WOC-PROG-P1-017 | currency只是非负copper字段 | typed currency ledger支持币种、上限、source/sink、balance revision和不可变double-entry journal |
| WOC-PROG-P1-018 | fee饱和、vendor price与reward policy散落 | economy policy编译rounding/saturation/tax/discount，transaction记录quoted与committed price |
| WOC-PROG-P1-019 | vendor stock与buyback是本地数组 | vendor instance拥有catalog revision、stock policy、restock clock和durable buyback container |
| WOC-PROG-P1-020 | buyback只保存item code/count，实例属性会丢失 | vendor transfer移动原ItemInstance或完整stack fragment，不重建替代物品 |
| WOC-PROG-P1-021 | bank_state是不可达局部state | durable bank service按account/character/tab/version授权并提供transactional deposit/withdraw |
| WOC-PROG-P1-022 | trade没有session identity与重连状态机 | `TradeSessionId`、participants、offers、locks、confirm revisions、timeout/cancel/reconnect状态图 |
| WOC-PROG-P1-023 | trade preflight后顺序apply，缺跨owner CAS | 双方inventory/currency revision与escrow write-set一次比较交换，失败零变更 |
| WOC-PROG-P1-024 | market listing用局部int ID和平行数组 | durable qualified ListingId、seller、instance escrow、price、quantity、revision、expiry与state machine |
| WOC-PROG-P1-025 | market search特判文本并扫描全部listing | normalized indexed query、pagination、stable sort、visibility/permission与bounded result budget |
| WOC-PROG-P1-026 | market expiry每秒全表扫描且无交付恢复 | expiry timer/index驱动；sale/cancel/expire通过outbox投递货币/物品并可幂等重放 |

## 7. P1：Loot、Crafting、Gathering 与 Profession

| ID | 当前差距 | 需要的工程化重构 |
|---|---|---|
| WOC-PROG-P1-027 | loot entitlement、roll、award分散在WorldState数组 | `LootSession`拥有source corpse/encounter、eligible principals、items、policy、deadline与terminal receipt |
| WOC-PROG-P1-028 | pending/candidate固定上限且超限策略不明确 | workload-derived caps与typed Reject/Defer/Overflow，不允许数组越界或静默丢候选 |
| WOC-PROG-P1-029 | need/greed候选和party成员通过反复线性扫描解析 | 快照generation-safe participant handles与indexed choices，成员变更有明确eligibility policy |
| WOC-PROG-P1-030 | 掉落grant可能绕过capacity | award原子写inventory或mailbox/claim container，离线/断线/满包仍有唯一交付结果 |
| WOC-PROG-P1-031 | master loot、FFA和distribution模块不在产品图 | loot policy作为registered strategy进入同一LootSession，不复制inventory/party state |
| WOC-PROG-P1-032 | recipe入口只支持两项且一项永久unsupported | generated recipe registry对全BuildSet逐项标记handler、station、profession、output与support status |
| WOC-PROG-P1-033 | craft材料与费用没有reservation | candidate锁定具体reagent instances/currency/revision，commit前统一revalidate |
| WOC-PROG-P1-034 | craft独特输出不能创建signed rolled instance | `ItemFactory`使用qualified RNG stream生成payload、provenance和creation receipt后一次插入 |
| WOC-PROG-P1-035 | craft RNG只消费共享cursor一次且purpose隐含 | stream按world/character/transaction/recipe/roll purpose分流，draw count和结果可重放 |
| WOC-PROG-P1-036 | skill gain、quest hook与result分散写入 | craft journal派生profession XP/objective/deed events，全部受同一transaction/outbox约束 |
| WOC-PROG-P1-037 | gathering/enchant/salvage/tool effects是不可达局部投影 | 统一为profession operations，共享reservation、instance mutation、RNG、cooldown与receipt core |
| WOC-PROG-P1-038 | harvest node/portable station状态没有world partition owner | generation-safe node/station entity、claim lease、respawn timer、partition persistence与anti-race gate |

## 8. P1：Quest、XP、Deed 与 Talent

| ID | 当前差距 | 需要的工程化重构 |
|---|---|---|
| WOC-PROG-P1-039 | quest只按两个hard-coded code保存标量 | generated `QuestDefinition`与per-character `QuestInstance`，支持versioned typed objective graph |
| WOC-PROG-P1-040 | primary player取第一个player entity | command携带authenticated character handle；party/world query不得推断authority |
| WOC-PROG-P1-041 | generic accept/link/abandon/turn-in模块不可达 | 单一QuestRuntime状态机进入product schedule并输出typed command outcome |
| WOC-PROG-P1-042 | kill/collect/craft/gather objective靠专用hook或no-op | 订阅qualified gameplay journal，按objective index增量更新并以event id幂等去重 |
| WOC-PROG-P1-043 | quest item计数与旧counter迁移会互相合成 | quest requirement从inventory index派生；migration产生显式reconciliation report，decode保持纯 |
| WOC-PROG-P1-044 | party credit/share/link规则没有统一owner | snapshot参与者、range/phase/tag/ownership与share policy，记录每个成员accept/credit结果 |
| WOC-PROG-P1-045 | quest reward直接写铜币/XP/物品 | reward编译为ProgressionTransaction，容量不足、level-up、已领取和重试有唯一receipt |
| WOC-PROG-P1-046 | XP/deed owners仅由lifecycle self-test导入 | XP/level/prestige/deed作为registered progression systems进入明确事件与schedule阶段 |
| WOC-PROG-P1-047 | XP cap、level-up和known ability刷新手写耦合 | `LevelTransitionPlan`声明curve revision、overflow/prestige、stats/abilities/rewards与ordered effects |
| WOC-PROG-P1-048 | talent存在fixture/commit/modifier/WorldState多份authority | 单一TalentRuntime拥有definition、allocation、loadout、compiled effects与migration |
| WOC-PROG-P1-049 | allocation lock缺arena与战斗后linger完整owner | admission读取authoritative combat/session tags和integer expiry，返回稳定lock reason |
| WOC-PROG-P1-050 | 产品提交只执行部分side effects | candidate编译modifier/stat/ability/proc/charge/aura/equipment/pet/echo完整effect plan并原子commit |
| WOC-PROG-P1-051 | modifier每次战斗路径重新扫描189项catalog | allocation revision生成immutable cached snapshot；hot path只取handle与prepared numeric blocks |
| WOC-PROG-P1-052 | nested effect/granted ability/proc在module与WorldState再次手写 | compiled talent output直接注册到CombatBuildSet，Runtime13执行但不重新解释talent catalog |

## 9. P1：Persistence、Replication、Performance、Security 与 Evidence

| ID | 当前差距 | 需要的工程化重构 |
|---|---|---|
| WOC-PROG-P1-053 | progression schema散落在WorldState版本分支 | 独立versioned components与migration DAG，Runtime12 snapshot记录BuildSet/schema/provenance |
| WOC-PROG-P1-054 | decode过程会补物品/任务事实 | decoder只恢复bytes；reconciliation/migration是单独事务并保存变更报告 |
| WOC-PROG-P1-055 | inventory/economy没有增量replication模型 | component/container revision加delta journal，ack/resync/interest与FastArray类语义明确 |
| WOC-PROG-P1-056 | 客户端命令缺统一principal/permission/replay boundary | host认证principal与character，runtime验证nonce/sequence/capability和expected revision |
| WOC-PROG-P1-057 | grant/trade/market/craft缺anti-dupe invariant | instance唯一归属、守恒账本、transaction idempotency、escrow conservation和离线审计扫描 |
| WOC-PROG-P1-058 | generated分支表在tick/command路径重复解释 | load/cook阶段构建indexed prepared registries，普通operation零文本catalog扫描 |
| WOC-PROG-P1-059 | Array构造与candidate clone缺内存预算 | transaction scratch arena与bytes/items/ops上限，失败在mutation前返回typed budget result |
| WOC-PROG-P1-060 | CSR搬移、market expiry、loot候选缺复杂度门 | 基准覆盖container/listing/member规模斜率，规定p50/p99、alloc、bytes moved和最大工作量 |
| WOC-PROG-P1-061 | contractTest混在48个候选生产模块 | 移入独立test package/friend adapter，release module/symbol inventory拒绝测试代码 |
| WOC-PROG-P1-062 | 45个manifest与46个test main没有生成式inventory | test catalog列出owner、entry、artifact、backend、oracle、required/optional和last receipt |
| WOC-PROG-P1-063 | 16个parity owner缺失 | 重建产品root runner，覆盖save/reload/reconnect/duplicate command/满包/并发冲突，不只单次内存结果 |
| WOC-PROG-P1-064 | 没有transaction、economy与progression诊断指标 | 记录operation、reads/writes、revision conflict、RNG、latency、allocation、receipt与first invariant failure |

## 10. P2：规模、工具与长期上限

| ID | 当前差距 | 收敛方向 |
|---|---|---|
| WOC-PROG-P2-001 | 大规模inventory查询仍可能逐container扫描 | secondary item/tag/owner index与batch query，按dirty revision增量维护 |
| WOC-PROG-P2-002 | market可能需要跨分片扩展 | 分区listing/escrow owner、全局query projection和幂等跨分片saga，正确性先于规模 |
| WOC-PROG-P2-003 | economy缺长期通胀/流动性模拟 | 使用同一price/reward/craft定义做离线simulation，输出source/sink和分布趋势 |
| WOC-PROG-P2-004 | loot与craft概率只靠样例 | deterministic Monte Carlo与统计置信门，结果回溯同一RNG和definition revision |
| WOC-PROG-P2-005 | progression默认单线程 | 依赖显式read/write set做per-character/container并行，stable merge保持determinism |
| WOC-PROG-P2-006 | hot reload中的实例迁移缺批处理工具 | revision lease、background migration、quarantine和loss report，在线对象不静默重解释 |
| WOC-PROG-P2-007 | 缺inventory/economy管理检查器 | Editor/admin查看instance provenance、container history、escrow、receipt与守恒校验 |
| WOC-PROG-P2-008 | 缺quest/talent可视化状态图调试 | 展示definition revision、objective/event trace、allocation plan和dirty dependents |
| WOC-PROG-P2-009 | 缺transaction replay与first-divergence工具 | 以journal重放inventory/economy/progression并逐revision定位首次差异 |
| WOC-PROG-P2-010 | mailbox/overflow长期堆积缺治理 | quota、expiry、notification、claim batching、support recovery与不可丢弃审计策略 |
| WOC-PROG-P2-011 | 离线玩家progression处理可能形成上线峰值 | bounded catch-up plan、分段commit和进度receipt，不一次扫描无限历史 |
| WOC-PROG-P2-012 | localization与presentation仍可能读取规则字符串 | runtime只输出qualified display keys/structured args，Editor/UI拥有本地化呈现 |
| WOC-PROG-P2-013 | property/fuzz主要依赖手写case | schema-guided生成stack split、满包、重复消息、并发revision、迁移和断线边界 |
| WOC-PROG-P2-014 | source sync后仍需人工盘点reachability | 自动生成production owner/reachability/capability/parity delta并阻断mixed generation |
| WOC-PROG-P2-015 | “优于Unreal”缺同正确性oracle的progression基准 | 固定内容/操作/规模/硬件，发布吞吐、p99、RSS、bytes moved、恢复时间和raw evidence |

## 11. Owner 与依赖收敛

| Owner | 唯一职责 | 禁止承担 | 前置依赖 |
|---|---|---|---|
| `ProgressionBuildSetRegistry` | item/recipe/quest/talent/profession/economy schema与revision组合 | 可变player state、运行时字符串分支 | Tooling05 artifact identity |
| `ItemRegistry` | prepared definitions、instance schema与qualified handles | container mutation、reward policy | BuildSet、Runtime asset identity |
| `ContainerStore` | inventory/equipment/bank/mail/escrow/corpse container与revision | vendor/quest/craft业务规则 | Runtime12 storage/generation |
| `ProgressionTransactionEngine` | reservation、read/write set、CAS、commit、receipt、outbox | VM snapshot publish、网络认证 | App03 outer transaction、Runtime12 journal |
| `EconomyRuntime` | currency/vendor/bank/trade/market state machine与ledger | item definition、UI presentation | Transaction、ContainerStore |
| `LootProfessionRuntime` | loot/craft/gather/enchant/salvage operation plans | combat伤害结算 | ItemFactory、Transaction、Runtime13 credit events |
| `QuestProgressionRuntime` | quest/XP/level/deed实例和event consumption | 第一个player推断、直接combat写入 | qualified character/party、journal |
| `TalentRuntime` | allocation/loadout revision、prepared modifier/effect plan与migration | combat opcode执行 | Runtime08G/13、BuildSet |
| `ProgressionEvidenceRunner` | unit/contract/parity/save-reload/perf/fault qualification | 自行维护第二套玩法公式 | Tooling10、source golden |

依赖顺序必须是 Runtime12/App03先提供typed collection、generation、snapshot与outer transaction，Tooling05产出同代BuildSet，随后Item/Container/Transaction，之后Economy与Loot/Profession，再接Quest/XP/Talent，最后才启用UI、online入口与competitive perf gate。不能先把market或craft按钮接到现有WorldState，再承诺以后补事务。

## 12. 重构里程碑

### M0 · Inventory与owner冻结

- 生成55模块production/test/fixture分类与普通fixed-tick reachability artifact；
- 冻结current behavior、16项parity oracle、item/recipe/quest/talent capability matrix；
- 为每项P0指定唯一owner和hard-cut目标，禁止新增WorldState progression字段。

### M1 · Item/Container/Transaction基础

- 实现qualified definition/instance/container identity与versioned payload；
- 建立ContainerStore、revision/CAS、reservation、receipt、outbox和守恒校验；
- 将scalar inventory、equipment、overflow与codec迁移到唯一owner，删除全局CSR authority。

### M2 · Economy与Loot收敛

- Vendor、bank、trade、market使用同一transaction和escrow；
- LootSession接入combat credit，满包/离线交付进入mailbox/claim；
- 完成duplicate/disconnect/crash/reload/fault矩阵后再开放产品入口。

### M3 · Craft/Profession与完整ItemInstance

- 全recipe registry进入BuildSet，支持signed rolled/masterwork instance输出；
- gathering/enchant/salvage/tool effect共享operation core；
- RNG、材料、货币、技能、任务hook与result在单一transaction中闭合。

### M4 · Quest/XP/Talent收敛

- generic QuestRuntime替换两项offline标量和first-player owner；
- XP/level/deed进入明确schedule并以journal事件驱动；
- TalentRuntime执行完整commit effect plan并缓存prepared modifier/proc snapshot。

### M5 · Evidence、性能与hard cut

- 46个test main进入generated inventory，恢复16项相关parity owner；
- 通过save/reload/reconnect/concurrency/fault/soak/scale gates；
- 删除不可达候选双authority、旧WorldState progression列与巨型热路径catalog解释。

## 13. Runtime 资格门

| Gate | 验收内容 |
|---|---|
| PROG-G01 | production owner manifest显示所有supported capability从产品root可达，fixture不能计supported |
| PROG-G02 | 每个item definition/instance/container引用qualified BuildSet/revision/generation，陈旧handle fail-closed |
| PROG-G03 | signed/rolled/enchant/bound/charges实例经inventory/equip/bank/trade/market/save/reload不丢字段或identity |
| PROG-G04 | grant/remove/move/equip/vendor/bank/trade/market/loot/craft/reward均产生幂等transaction receipt |
| PROG-G05 | crash、throw、budget、revision conflict、disconnect和duplicate command证明零partial mutation/zero dupe |
| PROG-G06 | currency与item守恒审计可由ledger重建，escrow与outbox重放不重复交付 |
| PROG-G07 | 满包奖励进入显式overflow/mailbox，UI capacity不被隐式扩展，物品不静默丢失 |
| PROG-G08 | 全BuildSet item/recipe/quest/talent有Implemented/Unsupported等显式状态和owner/test |
| PROG-G09 | Quest按qualified character/party执行，kill/collect/craft/gather/reward事件幂等且可save/reload |
| PROG-G10 | Talent提交完整执行modifier/stat/ability/proc/charge/aura/equipment/pet/echo计划并可rollback |
| PROG-G11 | 普通inventory/talent/craft/quest热路径没有generated文本if-ladder扫描和无界临时分配 |
| PROG-G12 | 1/100/1k角色、背包、listing、loot candidate规模报告p50/p99、alloc、bytes moved与斜率 |
| PROG-G13 | 46个test main均有catalog状态，required artifact实际执行；missing/skipped/not-run不能pass |
| PROG-G14 | 16项相关parity从产品root执行并绑定source/BuildSet/schema/backend/seed/actual diff |
| PROG-G15 | snapshot/migration/replication保持instance与transaction revision，decode本身不补业务事实 |
| PROG-G16 | `git diff --check`、frontmatter路径、Markdown链接、severity/owner/index/coverage统计全部通过 |

## 14. 状态与边界

| 项目 | 状态 |
|---|---|
| progression物理与普通fixed-tick可达性 | review_complete |
| item/inventory/equipment/economy/loot/craft纵向审查 | review_complete |
| quest/XP/deed/talent纵向审查 | review_complete |
| current-head catalog/parity与参考引擎路由 | review_complete |
| production代码、测试、manifest、generated artifact修改 | pending，未在本轮执行 |
| native/npm动态验证 | blocked_by_existing_failures，未重复无变化lane |

本篇不宣称580条item-level records都已经具备完整definition，也不把不可达module存在等同于功能交付。它登记的是更严格的工程事实：当前产品只有有界scalar progression投影，完整item instance和多数经济/任务/职业owner没有进入正常执行图，且没有通过可执行parity与事务恢复资格。

Runtime12继续唯一拥有WOC world storage/schedule/codec和纯decode/migration基础；Runtime13拥有combat outcome与credit journal；App03拥有VM/host发布和outer rollback；Tooling05拥有generated lookup与BuildSet产物。本篇只拥有WOC progression内部identity、transaction、owner、产品接线和qualification。实施阶段必须按上述依赖hard cut，禁止新增平行临时实现继续扩张WorldState。
