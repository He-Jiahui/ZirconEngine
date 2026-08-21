---
related_code:
  - examples/woc/scripts/woc_game/src/main.zr
  - examples/woc/scripts/woc_game/src/world/state.zr
  - examples/woc/scripts/woc_game/src/world/projectile_travel_state.zr
  - examples/woc/scripts/woc_game/src/combat
  - examples/woc/scripts/woc_game/src/combat/ability_admission.zr
  - examples/woc/scripts/woc_game/src/combat/auto_attack_state.zr
  - examples/woc/scripts/woc_game/src/combat/casting.zr
  - examples/woc/scripts/woc_game/src/combat/damage_state.zr
  - examples/woc/scripts/woc_game/src/combat/death_state.zr
  - examples/woc/scripts/woc_game/src/combat/heal_state.zr
  - examples/woc/scripts/woc_game/src/combat/threat_state.zr
  - examples/woc/scripts/woc_game/src/combat/effect_sequence_state.zr
  - examples/woc/scripts/woc_game/src/combat/effect_numeric_dispatch_state.zr
  - examples/woc/scripts/woc_game/src/combat/effect_aura_dispatch_state.zr
  - examples/woc/scripts/woc_game/src/combat/effect_world_dispatch_state.zr
  - examples/woc/scripts/woc_game/src/combat/aura_state.zr
  - examples/woc/scripts/woc_game/src/combat/cc_state.zr
  - examples/woc/scripts/woc_game/src/combat/stun_dr_state.zr
  - examples/woc/scripts/woc_game/src/combat/lockout_dr_state.zr
  - examples/woc/scripts/woc_game/src/combat/mob_melee_pursuit_state.zr
  - examples/woc/scripts/woc_game/src/combat/mob_swing_affix_state.zr
  - examples/woc/scripts/woc_game/src/combat/nythraxis_state.zr
  - examples/woc/scripts/woc_game/src/combat/drowned_litany_state.zr
  - examples/woc/reference/current-head/m4_ability_projection_coverage.json
  - examples/woc/reference/current-head/known_ability_catalog.json
  - examples/woc/reference/current-head/talent_proc_catalog.json
  - examples/woc/reference/current-head/combat_command_contract.json
  - examples/woc/reference/current-head/casting_lifecycle_contract.json
  - examples/woc/reference/current-head/cc_contract.json
  - examples/woc/reference/current-head/parity_scenarios.json
  - examples/woc/reference/current-head/source_manifest.json
  - examples/woc/reference/current-head/parity/golden/c4a_casting_lifecycle.json
  - examples/woc/reference/current-head/parity/golden/c4b_effect_dispatch.json
  - examples/woc/reference/current-head/parity/golden/c5_auto_attack.json
  - examples/woc/reference/current-head/parity/golden/mob_swing_affixes.json
  - examples/woc/reference/current-head/parity/golden/nythraxis_full_pull.json
tests:
  - examples/woc/scripts/woc_game/woc_game_tests.zrp
  - examples/woc/scripts/woc_game/woc_world_state_tests.zrp
  - examples/woc/scripts/woc_game/woc_m4_damage_state_tests.zrp
  - examples/woc/scripts/woc_game/woc_m4_nythraxis_state_tests.zrp
  - examples/woc/native/Cargo.toml
plan_sources:
  - docs/plans/optimize/zircon_app/03-woc-product-role-host-zrvm-transaction-state-client-server-integration-review.md
  - docs/plans/optimize/zircon_runtime/08g-gameplay-ability-effect-attribute-tag-cue-prediction-runtime-review.md
  - docs/plans/optimize/zircon_runtime/12-woc-zrvm-package-kernel-world-state-schedule-serialization-runtime-review.md
  - docs/plans/optimize/zircon_tooling/05-woc-content-codegen-build-scripts-generated-artifact-incremental-review.md
  - docs/plans/woc/00-woc-engine-capability-foundation.md
  - docs/plans/woc/01-woc-zrvm-one-to-one-replication.md
reference_engines:
  - dev/UnrealEngine/Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilities/Public/AbilitySystemComponent.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilities/Public/Abilities/GameplayAbility.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilities/Public/GameplayEffect.h
  - dev/bevy/crates/bevy_ecs/src/schedule/schedule.rs
  - dev/bevy/crates/bevy_ecs/src/archetype.rs
  - dev/bevy/crates/bevy_time/src/fixed.rs
  - dev/godot/scene/main/scene_tree.cpp
  - dev/godot/modules/multiplayer/scene_multiplayer.cpp
  - dev/Fyrox/fyrox-impl/src/engine/mod.rs
  - dev/Fyrox/fyrox-impl/src/script/mod.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/IRenderGraphBuilder.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 13 · WOC Combat、Casting、Effect、Aura、Damage、Threat 与 Death Runtime 工程化差距

## 1. 结论

WOC 战斗域已经积累了相当多的规则知识，不能把它简单归类为“缺功能”。`src/combat` 物理目录有 197 个 `.zr`、21,357 行、815,747 bytes；去掉 88 个 `*_test_main.zr` 后仍有 109 个模块、20,616 行。目录中能看到职业公式、命中表、吸收、治疗、DoT/HoT、光环叠层、控制递减、仇恨、自动攻击、投射物、Boss、词缀和大量 source-pinned contract。这些规则应作为迁移 oracle 保留。

问题在于“写了模块”并不等于“产品在使用模块”。从 `src/main.zr` 沿静态 import 做可达性扫描，109 个非 test-main 战斗模块中只有 54 个可达，合计 10,198 行；另有 55 个、10,418 行不可达。不可达集合不是边角工具：它包含 `ability_admission`、`damage_state`、`death_state`、`threat_state`、`effect_sequence_state`、`effect_aura_dispatch_state`、`effect_world_dispatch_state`、`aura_state`、`cc_state`、Nythraxis、Drowned Litany 和 mob swing affix 等本应成为语义 owner 的模块。多数只被自己的 test main 引用；Drowned Litany 另被一个同样不在产品根可达图中的 M7 scenario matrix 引用。

产品真实 authority 仍然是 `world/state.zr` 的巨型平行列与手写函数。名称审计显示 `WorldState` 的 519 个字段中至少 202 个字段名直接落在 cast/projectile/dot/hot/absorb/aura/threat/power/crit/haste 等战斗语义；这只是保守信号，不是正式组件分类。生产段存在 77 个直接 `entityHp[...] =` 写入点。`applySupportedCastSlotCommand` 用 616 行处理 117 个 ability-code 条件，`applySupportedCastCommand` 用 780 行重复 116 个 payload ability 匹配，`stepRetainedCasting` 再用 219 行处理 45 个完成分支，`stepOfflineEastbrookProjectiles` 用 257 行处理 33 个投射物能力分支。四份手写路由必须同时保持 target、cost、cooldown、cast、effect、projectile 和失败语义一致，但当前没有一份可编译的 `AbilitySpec -> CombatProgram` authority。

能力覆盖也没有闭环。current-head M4 projection 文件记录 308 个 known abilities，其中 117 个进入 M4 generic projection、191 个没有进入；这不等于 191 个能力在 WOC 全树中必然完全不存在，但足以证明 generic execution surface 不是完整 catalog。117 个 projected ability 里只有 21 个带 `m4_scenarios`，96 个没有逐能力场景；全 308 项中有 287 项没有该场景字段。未知或不支持的 cast-at/slot/payload 路径有的 throw，有的大量早退且没有 typed rejection receipt，无法区分“规则拒绝”“功能未接线”“无效果成功”。

战斗结果也不是一个原子事务。可达的 `heal_state`、numeric dispatch 和 auto-attack 多通过临时对象 copy-in/copy-out 接入 WorldState；通用 damage/death/threat/aura/effect-sequence owners却不在产品图中。离线 mob melee 的玩家死亡函数明确只实现 single-player 子集，并把完整 aura teardown、全部 pet、revenge、multiplayer retargeting 留给“future shared death transaction”。追击模块也明确把 damage、threat retargeting、flee、leash、cast 和 boss mechanics 推迟。当前一次命中可在多个函数里直接写 HP、rage、stealth、cast、threat、proc、reflect、death 和 reward，没有唯一 commit/rollback 边界或 ordered event journal。

测试数量不能补上这个断层。88 个 test main 很多，但 109 个非 test-main 模块中有 99 个把 `pub contractTest` 混在生产文件内；这些 test tail 合计约 4,808 行，占 20,616 行的 23.32%。83 份可解析为 JSON 的 combat `.zrp` 虽各自声明 entry，但当前没有对应的 checked-in combat binary 目录。更关键的是 `parity_scenarios.json` 的 54 个 `woc_owner` 全部指向当前树中不存在的 `scripts/woc_game/tests/parity/*.zr`；golden 只能作为上游输入 oracle，不能作为当前实现通过证明。native workspace又在 `woc_protocol` 的 6 个现存编译错误处停止，测试尚未开始。

本轮登记 **6 项 P0、62 项 P1 和 15 项 P2**。Runtime08G继续拥有引擎通用 Ability/Effect/Attribute/Tag/Cue/Prediction 基础设施，Runtime12继续拥有 WOC world storage/schedule/codec，App03继续拥有跨 VM/host 事务。本篇唯一拥有 WOC 战斗语义收敛：单一能力定义和 prepared program、activation/effect instance、原子 combat transaction、统一 aura/damage/heal/threat/death owner、Boss/affix 产品接线，以及可执行的逐能力 parity/performance 资格。

## 2. 审查边界与证据

### 2.1 物理与可达性盘点

| 集合 | 文件 | 行 | bytes | 结论 |
|---|---:|---:|---:|---|
| `combat/**/*.zr` | 197 | 21,357 | 815,747 | 目录物理总量 |
| 非 `*_test_main.zr` | 109 | 20,616 | - | 候选生产/共享模块 |
| `*_test_main.zr` | 88 | 741 | - | 独立入口，不代表被聚合执行 |
| 从 `src/main.zr` 可达的 combat 模块 | 54 | 10,198 | 383,731 | 产品静态依赖图可见部分 |
| 不可达的非 test-main combat 模块 | 55 | 10,418 | 400,924 | 约占候选实现一半，含核心语义 owner |

不可达集合中的代表性模块如下：

- 基础执行：`ability_admission`、`casting`、`spell_combat_state`、`effect_sequence_state`、`effect_aura_dispatch_state`、`effect_world_dispatch_state`。
- 结果与生命周期：`damage_state`、`death_state`、`threat_state`、`aura_state`、`resurrection_offer`、`resurrection_sickness_state`。
- 控制与 proc：`cc_state`、`stun_dr_state`、`lockout_dr_state`、`exclusive_aura_state`、`equip_procs_state`、`set_procs_state`、`sure_crit_state`。
- 职业/能力：`frost_proc_state`、`frozen_orb_state`、`heroic_leap_state`、`hunter_trap_state`、`warrior_stances_state`、`warrior_hit_table_state`。
- Encounter：`nythraxis_state`、`nythraxis_channel_state`、`drowned_litany_state`、`mob_swing_affix_state`。

`WorldState` 顶部导入 51 个唯一 combat 模块名，但存在重复 import，且“import 过”仍不意味着某个通用语义 owner被产品调用。反向依赖检查显示 damage/death/threat/effect sequence/aura/world dispatch/ability admission等大多只被其 test main引用；因此不能用目录文件数或单模块 contract test替代产品根可达性。

### 2.2 代码形状与热路径信号

| 信号 | 当前值 | 工程含义 |
|---|---:|---|
| 非 test-main combat classes | 135 | 多数是状态投影对象而非被runtime统一拥有的component/instance |
| `pub var` 字段 | 1,467 | 公开可变平行状态多，跨字段不变量依靠调用约定 |
| `throw` | 120 | 多为字符串错误，缺少稳定reject/result taxonomy |
| `new container.Array` | 282 | 解释器热路径临时分配风险高 |
| `while` | 277 | 大量手写扫描/移位，没有query/timer/index plan |
| `.indexOf(` | 18 | 显式线性查找；WorldState还有更多手写扫描 |
| 字符串相等分支 | 332 | ability/effect/aura/form等身份仍频繁靠字符串 |
| `randomUnits` 相关引用 | 139 | 随机draw顺序成为跨函数隐式ABI |
| exact `0.05` | 90 | fixed step在业务模块重复硬编码 |
| epsilon literal | 44 | 数值容差没有统一profile |

最大的对象包括 106-field `auto_attack_state`、99-field `death_state`、94-field `effect_world_dispatch_state`、76-field `damage_state`、73-field `form_derived_state` 和 64-field `drowned_litany_state`。这不是“类多所以模块化”的证据：WorldState会创建临时 AutoActor/AutoTarget/AutoEvents、HealEntity/HealEvents、TimedSpellSpec/Result、SpellResistState等，复制字段进去、调用、再复制字段出来。全树可见 192 个 combat alias对象构造和753个alias成员访问，说明模块边界主要是手动 projection adapter，而不是稳定拥有状态的runtime boundary。

### 2.3 Dispatch、coverage 与结果语义

- `applySupportedCastSlotCommand` 为616行，含117个 `abilityCode ==` 条件、119个start调用和120个 `return`；尾部对未知能力throw。
- `applySupportedCastCommand` 为780行，含116个 exact payload ability匹配、121个start调用、121个 `return`，并重复读取target payload；尾部同样throw。
- `applySupportedCastAtCommand` 只显式接 Rain of Fire、Hurricane、Flamestrike、Temporal Hourglass，其余throw。
- `stepRetainedCasting` 为219行，含45个 completed-ability分支、33个completion调用和5个channel tick调用。
- `stepOfflineEastbrookProjectiles` 为257行，含33个projectile ability分支和36个landing调用。
- M4 projection：308 known、117 projected、191 not projected；117 projected中21有scenario、96无scenario。文件自身没有声称not-projected能力在全WOC中不存在，本文也不做该误推断。

### 2.4 Damage、death 与 threat 证据

- `damage_state` 定义Combatant/DamageHit/DamageEvents，并试图统一 mitigation、rage、stealth、casting、threat、frenzy、reflect和death，但不在产品根可达图。
- `death_state` 用99个公开字段表达DeathEntity/DeathState与handleDeath，并把XP/quest/loot/deed/reward视为下游；它同样不可达。
- WorldState生产段有77个直接HP赋值点，只有少数直接dead标记；死亡副作用散落在pet、mob、player等分支。
- `applyOfflineMobMeleePlayerDeath` 明确只完成single-player subset，完整aura/pet/revenge/multiplayer retargeting被推迟。
- 生产 `setThreat` 使用flattened CSR数组；插入会移动后续全部threat rows并更新所有后续entity offset。`clearStateThreat`逐项从中部删除并反复更新offset，最坏复杂度接近owner-threat-count乘global-rows/entity-count。
- 不可达的 `threat_state` 虽有较窄API，内部仍是平行数组和线性扫描；因此简单把WorldState调用切到该文件也达不到工程基线。

### 2.5 RNG、时间与一致性

AutoAttack拥有单独RNG adapter：优先消费传入random array，否则修改模块state/draw/digest，WorldState再把cursor复制进出。其他模块则由调用者手动准备random units。新增一个draw、改变路由顺序或把模块接入产品图，都会改变后续跨系统结果；当前没有按world/domain/activation/effect命名的stream，也没有draw receipt。时间同时使用float remaining、exact `0.05` 与uint microsecond expiry，periodic effect、cast completion、projectile landing和aura expiry没有统一same-tick顺序合同。

### 2.6 动态验证边界

本篇没有重复运行已知不变失败lane。`cargo test --manifest-path examples/woc/native/Cargo.toml --workspace` 已在 `woc_protocol` 编译阶段以6个错误停止，没有测试执行；npm聚合check也在typed contract数量不一致处提前停止。83份JSON格式combat manifest的entry和binary目录名各不相同，但当前没有这些combat binary目录；其余TOML-like `.zrp` 也不构成已执行证据。根 `woc_game_tests.zrp` 只指向M3 foundation test，`woc_world_state_tests.zrp` 只指向state test，不是combat aggregate suite。

`parity_scenarios.json`记录54个golden场景和上游commit `5ef9f7cb21cd8875b6d2c49701015dfcd78de35a`，包括 casting/effect/auto attack、mob affix、multi-class heal/frenzy、Nythraxis和Drowned Litany；但54个 `woc_owner` 文件当前全部缺失。故本篇把golden作为待接入的differential oracle，不把它们记为当前pass。

### 2.7 参考引擎约束

- Unreal `AbilitySystemComponent` 持有ability specs、active gameplay effects、prediction key、activation/end/cancel RPC与周期effect执行；`GameplayAbility`明确CanActivate/Commit/Cancel/End阶段；`GameplayEffect`明确duration、period、execution、stack limit、stack refresh/reset、inhibition和cue。WOC无需照抄UObject/Blueprint，但必须拥有同等级的lifecycle、identity、policy和诊断闭环。
- Bevy schedule graph提供依赖、冲突与deferred barrier，archetype/storage避免为无关entity执行system；fixed time使用明确accumulator/overstep。WOC战斗应成为Runtime12 compiled schedule上的typed systems，而不是继续扩展字符串分支和全量列扫描。
- Godot SceneTree区分physics/process阶段、process group排序和线程执行，SceneMultiplayer维护对象配置生命周期。它不是ability system参考，但说明成熟runtime会把执行阶段、对象注册和网络配置作为owner合同。
- Fyrox提供plugin/script on_start、update、deinit与阶段统计。其玩法能力不作为性能上限，但可用于审查WOC encounter/combat extension是否有完整activate/quiesce/deinit，而不是只靠import即可生效。
- Unity Graphics的RenderGraph只作有限方法类比：pass先声明资源访问和side effect，再compile/cull/execute。它不是gameplay参考；本篇只借鉴“内容定义先编译为prepared execution plan”的工程方法。

## 3. 可保留的正确基础

### 3.1 Source-pinned catalogs与golden有迁移价值

known ability、talent proc、command、effect contract和54个golden都记录上游commit/hash/order。应把它们转成schema/codegen输入和differential oracle，而不是在重构时重新手抄行为。

### 3.2 分域规则模块包含大量可复用语义

Damage/Death/Threat/Aura/CC/DR/Nythraxis等模块虽然未接产品，但已经表达了不少边界条件。迁移应先把它们变成oracle fixture，再把规则移入唯一owner；不能直接删除后只保留WorldState现有的较小子集。

### 3.3 Actor generation、sequence和state内RNG意图正确

WOC命令携actor id/generation/sequence，world state也试图持久化RNG cursor。新的CombatContext和transaction必须保留这些确定性/防重放基础，只需把单一隐式stream升级为qualified streams与receipt。

### 3.4 Cast、projectile、periodic effect已有明确阶段雏形

现有代码区分admission/start/completion/channel/projectile landing/periodic update，只是这些阶段被复制成多个if ladder且生命周期不闭合。prepared CombatProgram可以吸收当前顺序，不必退回“一次函数直接扣血”的简化方案。

### 3.5 Collection和finite检查应迁入schema

多个模块已有长度、finite、范围和严格顺序检查。重构应从统一definition/schema生成这些验证并在分配/修改前执行，而不是以性能为由删除防御。

## 4. P0：战斗产品化前必须硬阻断

### WOC-COMB-P0-001 · 产品战斗authority与候选语义owners分裂

109个非test-main模块中55个不在`src/main.zr`可达图，且包含damage/death/threat/effect sequence/aura/CC/Boss等核心owner；产品实际依赖WorldState字段和手写adapter。相同概念因此存在“较完整但不可达模块”“WorldState局部实现”“contract test行为”三份truth，修一份不能保证产品变化。

必须建立唯一 `WocCombatRuntime` owner graph，逐域选择并迁移规则；每个Ability/Activation/ActiveEffect/Aura/Threat/Death/Encounter只能有一个生产owner。构建产物生成root-reachability与owner manifest，声明为production的模块不可只被test引用；hard cut后删除旧投影authority，禁止长期双写。

### WOC-COMB-P0-002 · 308项能力没有可证明完整、单源的执行计划

M4 generic projection只覆盖117/308，且仅21项带M4场景；cast slot、payload、completion和projectile又各自维护百级分支。未知路径可能throw或静默return，无法证明每个能力的admission、cost、target、cast、effect、cooldown和terminal result来自同一内容定义。

从source-pinned ability/effect/talent catalog生成versioned `CombatProgram`: typed target schema、admission predicates、cost/cooldown commit、cast/channel phases、ordered effects、projectile/area plan、proc/cue和result contract。所有入口只按stable AbilityId查prepared handle；build gate要求308项每项明确为Implemented/Unsupported/ServerOnly/Deprecated之一，禁止隐式漏分支。

### WOC-COMB-P0-003 · 命中到死亡没有单一原子CombatTransaction

77个生产HP直接写入点与分散的rage/stealth/cast/threat/proc/reflect/death/reward修改，使中途throw、预算耗尽或下游失败时无法证明all-or-nothing。通用damage/death owners不可达，copy-in/copy-out adapter又可能漏回写字段；同一命中没有稳定ordered journal。

建立 `CombatTransaction`：以qualified world tick/command/activation为base，收集attribute deltas、active-effect mutations、threat、control、death、spawn/despawn、progression handoff和cue；先validate/budget，再以stable order原子commit，失败不发布partial state。App03拥有VM/host外层transaction，本项拥有其内部combat write-set与event receipt。

### WOC-COMB-P0-004 · Death只实现离线单玩家子集，生命周期会留下不一致状态

当前player death函数明确推迟完整aura teardown、全部pet、revenge和multiplayer retargeting；mob pursuit又推迟damage/threat/flee/leash/cast/boss。Pet、mob、player、boss分别手写dead与清理分支，kill credit、loot、quest、deed、resurrection和encounter terminalization没有统一排序/幂等合同。

Death必须成为transaction内唯一terminal phase：判定致死、锁定killer/contributors、cancel casting、清理/保留effect、pet/target/threat detach、encounter通知、corpse/respawn/resurrection、reward handoff和cue都由typed policy驱动。重复命中、同tick多杀手、reflect、DoT owner离线、pet owner退出和跨world transfer必须产生唯一幂等结果。

### WOC-COMB-P0-005 · Boss与词缀参考实现未接产品执行图

Nythraxis、Drowned Litany、mob swing affix等完整度较高的状态模块不在产品根可达图；Eastbrook产品路径的注释则明确只覆盖有限狼/野猪行为并推迟affix/完整追击语义。golden里有对应场景不改变这一事实，因为测试owner也缺失。

Encounter/affix必须作为显式CombatExtension注册到package/schedule：声明content IDs、state schema、events、read/write set、RNG stream、activation/quiesce/deinit和migration。构建时验证golden场景引用的extension进入产品plan；未接线的Boss/affix不得在能力表、UI或发行说明中宣称supported。

### WOC-COMB-P0-006 · 当前没有可执行的战斗parity与release资格证据

54个parity owner全部缺失、combat test manifests没有对应checked-in artifact、aggregate test不覆盖combat，native workspace编译失败且测试未启动。生产文件内的99个contractTest、reference golden和文件数量都不能证明当前解释器/host上的产品行为。

先恢复编译和generated test catalog，再生成真实parity test package：每个场景从current product root加载同一BuildSet，执行命令/tick，比较typed events与canonical state projection。资格系统必须记录source commit、catalog/program/schema/schedule digest、backend、hardware和raw diff；missing owner、not run、skipped或zero-event一律不是pass。

## 5. P1：Authority、Definition 与 Activation

| ID | 当前差距 | 需要的工程化重构 |
|---|---|---|
| WOC-COMB-P1-001 | Ability identity在字符串、abilityCode和payload bytes间重复转换 | 生成stable `AbilityId`/revision/namespace，字符串只在authoring与诊断边界出现 |
| WOC-COMB-P1-002 | Ability、effect、talent、proc和command catalog没有统一BuildSet identity | `CombatBuildSet`绑定全部source hash、schema、program compiler和compatibility range |
| WOC-COMB-P1-003 | 运行时没有只读prepared ability/effect registry | cook/activate时解析并验证为immutable indexed registry，hot path只取qualified handle |
| WOC-COMB-P1-004 | 规则散落在generated catalog、WorldState ladder与独立module | 每个规则字段有单一source owner；compiler输出provenance到source path/key/revision |
| WOC-COMB-P1-005 | activation没有稳定instance identity | `ActivationHandle`包含world/entity generation/ability/revision/serial，进入cast/effect/RNG/journal |
| WOC-COMB-P1-006 | activation state由多个cast/channel/projectile数组和字段拼接 | `AbilityActivation`拥有phase、target snapshot、cost reservation、timers、children和terminal result |
| WOC-COMB-P1-007 | 没有明确CanActivate/Activate/Commit/Cancel/End合同 | 定义纯admission、resource reserve、atomic commit、running、terminal phase与合法转换图 |
| WOC-COMB-P1-008 | interrupted/cancelled/failed/completed常靠清字段区分 | typed terminal reason只写一次，释放reservation/timer/effect/cue并生成receipt |
| WOC-COMB-P1-009 | 角色form/stance/stealth等条件在各能力分支重复 | 统一tag/state query IR，编译为prepared predicate并记录first failed requirement |
| WOC-COMB-P1-010 | 模块import本身被当作能力可用信号 | package admission输出capability/extension availability，ability registry按依赖fail-closed |

## 6. P1：Casting、Targeting、Dispatch 与 Projectile

| ID | 当前差距 | 需要的工程化重构 |
|---|---|---|
| WOC-COMB-P1-011 | slot cast和payload cast维护两套百级branch ladder | 二者decode为同一typed `CastRequest`，只调用一次program executor |
| WOC-COMB-P1-012 | completion另有45项分支，可能与start表漂移 | program显式编码phase graph，compiler验证每个started phase都有terminal continuation |
| WOC-COMB-P1-013 | projectile landing再维护33项能力分支 | projectile携prepared impact program handle与revision，不按字符串重新dispatch |
| WOC-COMB-P1-014 | cast-at只支持四项且其余统一throw | target schema在admission返回UnsupportedTargetMode或具体validation error，不让业务拒绝炸tick |
| WOC-COMB-P1-015 | target payload在多个分支重复读取/resolve | decoder生成bounded DTO，admission一次resolve generation-safe target handles |
| WOC-COMB-P1-016 | range/LOS/facing/alive/friendly条件散落且执行时点不一致 | 定义target policy和snapshot/revalidate规则，start/commit/impact分别声明需要重验的条件 |
| WOC-COMB-P1-017 | ground point没有统一nav/physics/world bounds与height policy | `GroundTarget`由spatial service canonicalize，记录source ray、surface、cell和validation receipt |
| WOC-COMB-P1-018 | cost/cooldown/GCD可能在不同早退位置消费 | admission只reserve，commit一次性消费；失败、cancel、interrupt按policy refund并可审计 |
| WOC-COMB-P1-019 | queued cast/empowered release命令未形成统一状态机 | activation queue定义replace/drop/priority/expiry，empower press/release/timeout共享同一handle |
| WOC-COMB-P1-020 | channel tick与cast completion使用手写float timers | schedule timer使用integer tick/rational substep，定义same-tick interrupt/tick/complete顺序 |
| WOC-COMB-P1-021 | projectile state与world entity/visual/impact owner边界不清 | simulation projectile拥有generation、trajectory、collision policy和impact program；presentation只消费cue |
| WOC-COMB-P1-022 | silent `return`无法说明请求是否生效 | 每个入口返回Accepted/Rejected/Deferred/NoChange，含reason、activation、cost和next legal action |

## 7. P1：Effect、Aura、Damage、Healing 与 Proc

| ID | 当前差距 | 需要的工程化重构 |
|---|---|---|
| WOC-COMB-P1-023 | effect numeric/aura/world/sequence四类dispatch owner并不同时可达 | 合并为typed effect opcode/graph executor，所有opcode有唯一handler与coverage状态 |
| WOC-COMB-P1-024 | Active effect没有统一stable handle与container | per-entity `ActiveEffectContainer`管理handle、spec revision、source、stacks、timers、inhibition和children |
| WOC-COMB-P1-025 | instant/duration/infinite/periodic语义分散 | EffectSpec明确duration policy、period、execute-on-apply、expiry和catch-up policy |
| WOC-COMB-P1-026 | aura stack/exclusive/refresh规则由多个state模块各自维护 | 统一stack key、limit、overflow、duration refresh、period reset、source aggregation与remove policy |
| WOC-COMB-P1-027 | effect inhibition与解除后period行为没有通用合同 | active effect记录inhibition reason；恢复时按Keep/Reset/ExecuteAndReset等typed policy执行 |
| WOC-COMB-P1-028 | periodic effect依赖逐tick扫描和float remaining | world timer service只调度到期handle，支持cancel/reschedule/snapshot和bounded catch-up |
| WOC-COMB-P1-029 | damage阶段没有统一capture/mitigation/crit/absorb顺序 | 编译DamagePipeline：capture -> hit/avoid -> modifiers -> crit -> mitigation -> absorb -> clamp -> triggers |
| WOC-COMB-P1-030 | healing通过临时HealEntity投影，字段回写完整性靠人工 | HealingPipeline直接访问声明的component view，输出overheal、absorb、threat和proc receipt |
| WOC-COMB-P1-031 | absorb shield排序、部分消耗、破盾trigger缺少统一container | ActiveEffectContainer定义priority/source/apply-order、remaining、overflow和break event |
| WOC-COMB-P1-032 | DoT/HoT/ground effect各有并行数组和专用step | 共享scheduled effect core，domain handler只拥有公式/targeting，不复制timer/lifecycle机制 |
| WOC-COMB-P1-033 | proc从talent/set/equip/affix/reflect多处递归触发 | `ProcGraph`声明event filters、chance、ICD、charges、depth和cycle guard，执行生成parent-child trace |
| WOC-COMB-P1-034 | break-on-damage、stealth/form取消与damage写入耦合 | effect/tag removal由post-damage trigger phase统一处理，规则按priority和event snapshot执行 |
| WOC-COMB-P1-035 | cue/VFX/audio没有与authoritative effect event形成稳定映射 | CombatJournal输出qualified CueEvent、prediction/reconciliation key和lifetime，不让simulation直接操作renderer |

## 8. P1：Death、Threat、Control 与 Encounter

| ID | 当前差距 | 需要的工程化重构 |
|---|---|---|
| WOC-COMB-P1-036 | player/mob/pet/boss分别维护死亡清理 | `DeathPolicy`按entity archetype组合terminal steps，共享幂等transaction和事件顺序 |
| WOC-COMB-P1-037 | killer、owner、contributor、tap和reflect attribution散落 | `DamageProvenance`链保存original/source/controller/ability/effect/activation并用于credit |
| WOC-COMB-P1-038 | reward/loot/quest/deed在死亡后由松散下游触发 | death journal只发布一次qualified credit event，下游以idempotency key提交自己的transaction |
| WOC-COMB-P1-039 | resurrection offer/sickness不在统一death state machine | Corpse/Release/Offer/Accept/Respawn是versioned状态图，跨区/超时/重复响应有typed结果 |
| WOC-COMB-P1-040 | threat CSR插入/删除搬移全局数组 | 使用per-owner small map/paged sparse store、O(1) owner lookup和budgeted deterministic top target |
| WOC-COMB-P1-041 | threat value查询和clear反复线性扫描 | 缓存owner table与target index，batch clear/decay/transfer并维护generation-safe refs |
| WOC-COMB-P1-042 | taunt/forced target、retarget和flee没有统一priority | `AggroPolicy`显式定义forced target lease、immunity、expiry、fallback和visibility/reachability |
| WOC-COMB-P1-043 | stun/silence/lockout/DR有多个不可达state owner | 统一ControlContainer与DR category/timer，apply/remove/break/immunity返回structured outcome |
| WOC-COMB-P1-044 | encounter state可用raw array offset表达Boss数据 | generated typed encounter schema与validated accessors，禁止`combatData[magicOffset]`业务authority |
| WOC-COMB-P1-045 | encounter/affix activation没有生命周期与schedule registration | extension明确load/activate/quiesce/migrate/deactivate、systems、timers、effects和cleanup验收 |

## 9. P1：Determinism、Storage、Performance 与 Persistence

| ID | 当前差距 | 需要的工程化重构 |
|---|---|---|
| WOC-COMB-P1-046 | 单一/手工RNG draw顺序随模块接线变化 | counter-based或qualified stream按world/domain/activation/effect/proc分流，draw记录purpose |
| WOC-COMB-P1-047 | float remaining、0.05和microsecond expiry并存 | 唯一simulation time type和rounding profile；authoring秒值在compile时量化 |
| WOC-COMB-P1-048 | 44处epsilon没有字段/公式级数值合同 | numeric profile声明unit、precision、comparison/rounding/saturation并跨interp/AOT验证 |
| WOC-COMB-P1-049 | 临时projection对象和282个Array构造可能进入热路径 | component/query view零拷贝访问，scratch arena有budget，ordinary hit/cast不分配长期对象 |
| WOC-COMB-P1-050 | ability/effect/threat/target常用字符串与线性查找 | prepared integer handles、generation table、small-map/index和batch query；热路径字符串比较为0 |
| WOC-COMB-P1-051 | periodic/aura/projectile每帧扫描整个集合 | timer wheel + spatial/owner buckets + dirty queues，只处理到期/移动/变更项 |
| WOC-COMB-P1-052 | combat state散落在WorldState 202+命名字段 | 按activation/effect/threat/control/encounter component分片，服从Runtime12 storage/change tracking |
| WOC-COMB-P1-053 | save/load只保存平行字段，active program revision关系不明确 | snapshot持有qualified spec/program revision、timer和migration adapter，旧BuildSet fail-closed或显式迁移 |
| WOC-COMB-P1-054 | 没有能力数/active effect/raid threat/projectile规模预算 | workload profile定义caps、memory、ops、queue、degrade/reject策略并做1/100/1k entity sweep |

## 10. P1：Content、Testing、Parity 与 Diagnostics

| ID | 当前差距 | 需要的工程化重构 |
|---|---|---|
| WOC-COMB-P1-055 | 308项能力没有machine-readable实现状态总表 | 生成coverage matrix：definition/program/handler/test/parity/perf/unsupported reason与owner |
| WOC-COMB-P1-056 | 96个projected ability没有M4 scenario | 至少生成definition/activation/effect/terminal contract case；高风险能力再加golden与property test |
| WOC-COMB-P1-057 | 88个test main和大量manifest没有聚合inventory | typed test catalog生成unit/contract/integration/parity/perf/fuzz矩阵及required backend |
| WOC-COMB-P1-058 | 99个contractTest混在生产模块，占约23%代码尾段 | 移到test package/friend adapter，release module/symbol inventory拒绝test code |
| WOC-COMB-P1-059 | 22个规则模块没有matching test main，10个没有contractTest | coverage gate按production owner和risk要求测试，不以命名约定猜测覆盖 |
| WOC-COMB-P1-060 | parity manifest owner缺失且golden没有runner receipt | 重建54个owner或生成等价runner，验证golden hash/source commit并保存actual/first diff |
| WOC-COMB-P1-061 | combat没有per-stage rows/alloc/RNG/effect/threat/transaction指标 | 自动记录program opcode、targets、timers、writes、events、draws、alloc、latency和reject reason |
| WOC-COMB-P1-062 | 没有raid/affix/proc storm/long combat soak与复杂度门 | 固定workload跑短基准、24h soak、fault/reload/migration，检查p99/RSS/queue/determinism斜率 |

## 11. P2：完成度、工具与长期上限

| ID | 当前差距 | 收敛方向 |
|---|---|---|
| WOC-COMB-P2-001 | Ability/effect DSL仍可能允许无界或递归组合 | compiler做termination、cycle、max-target/max-event/max-proc静态分析 |
| WOC-COMB-P2-002 | 大规模periodic effect仍可能产生同tick峰值 | 分层timer wheel与deterministic budget partition，超限有显式carry/degrade policy |
| WOC-COMB-P2-003 | 公式逐对象解释执行难以达到高端性能 | 对prepared numeric blocks做specialization/vectorization，并保留逐bit oracle |
| WOC-COMB-P2-004 | 战斗系统默认单线程 | 基于Runtime12 access graph做deterministic chunk parallelism和stable event merge |
| WOC-COMB-P2-005 | 缺少authoring-time战斗图可视化 | Editor展示program phase、target/effect/proc graph、cost和worst-case budget |
| WOC-COMB-P2-006 | 缺少combat replay inspector | 按activation/effect/entity时间线显示input、RNG、writes、events和first divergence |
| WOC-COMB-P2-007 | 缺少平衡/蒙特卡洛模拟器 | 在同一prepared program与numeric profile上离线批量模拟，不能维护第二套公式 |
| WOC-COMB-P2-008 | Cue与渲染资源需求在运行时才暴露 | cook时生成cue/resource dependency和residency hints；缺资源有明确fallback/qualification |
| WOC-COMB-P2-009 | hot reload无法迁移运行中的activation/effect | spec revision lease + migration/finish-old/cancel policy，禁止handle悄然换定义 |
| WOC-COMB-P2-010 | test scenario主要手写 | schema-guided property/fuzz生成target、stack、interrupt、same-tick和budget边界 |
| WOC-COMB-P2-011 | 多人raid的interest/replication可能复制全部combat事件 | relevance graph按observer/party/encounter/cue class投影，authority journal仍完整 |
| WOC-COMB-P2-012 | 缺少anti-cheat proof surface | admission/target/RNG/cost/commit receipt可抽样签名验证，客户端不得决定authoritative result |
| WOC-COMB-P2-013 | 缺少跨版本combat trace migration工具 | trace reader按BuildSet加载旧schema/program metadata并输出loss-aware diff |
| WOC-COMB-P2-014 | source reference更新只能人工重新盘点 | source sync自动生成coverage/reachability/owner/golden diff并阻断mixed generation |
| WOC-COMB-P2-015 | “优于Unreal”没有同正确性oracle的公开基准 | 固定硬件/内容/workload/正确性，比较吞吐、p99、RSS、扩展斜率并发布raw artifact |

## 12. Owner 与依赖收敛

| Owner | 唯一职责 | 禁止承担 | 前置依赖 |
|---|---|---|---|
| `CombatBuildSetRegistry` | ability/effect/proc/encounter schema与source revision组合 | 运行时可变WorldState、host transaction | Tooling05生成原子性 |
| `CombatProgramCompiler` | definition校验、target/effect/proc编译、预算与provenance | entity状态、临时字符串dispatch | BuildSet registry |
| `CombatRuntime` | activation/effect/threat/control/encounter owners生命周期 | world codec/storage、UI/network transport | Runtime08G基础接口、Runtime12 kernel |
| `ActivationStore` | handle、phase、reservation、timer、children、terminal result | ability catalog authoring | Entity generation、timer service |
| `ActiveEffectStore` | effect handle、stack/period/inhibition/source与expiry | damage/reward任意跨域写入 | CombatProgram、timer service |
| `CombatTransactionExecutor` | validate、budget、stable commit、journal、rollback | VM host commit、persistence transport | App03外层transaction、Runtime12 COW write-set |
| `DamageHealingPipeline` | capture/mitigation/absorb/crit/heal/overheal及trigger事件 | death reward、presentation cue执行 | Numeric profile、ActiveEffectStore |
| `DeathCreditService` | terminal policy、attribution、cleanup与幂等credit event | loot/quest/account数据库直接提交 | Combat transaction、progression owner |
| `ThreatControlService` | threat/taunt/forced target/CC/DR容器与query | AI完整决策或movement实现 | AI08F、entity relation index |
| `CombatExtensionRegistry` | Boss/affix lifecycle、schema、systems、program hooks | import即激活、raw offset共享数组 | package admission、compiled schedule |
| `CombatTestRuntime` | generated matrix、parity/golden/fuzz/perf/soak | product activation时deep self-test | App03真实runner、Tooling test catalog |

边界固定如下：Runtime08G负责引擎通用Ability System接口、tag/attribute/effect/prediction/replication能力；Runtime12负责WOC entity storage、fixed schedule、timer、COW snapshot和schema shard；App03负责VM/host外层candidate/commit/rollback；本篇负责这些基座之上的WOC战斗语义和产品接线。任何里程碑不得用继续给`world/state.zr`追加字段/branch或把不可达module再包一层adapter来代替hard cut。

## 13. 重构里程碑

### M0 · Truth Freeze、Reachability 与可执行基线

1. 修复App03/Tooling05现存compile/schema/test runner blocker，生成真实combat aggregate artifact。
2. 冻结308项能力、effect/proc/command/encounter catalog与54个golden，记录BuildSet digest。
3. 生成production root reachability/owner map；标注55个不可达模块为Migrate/Oracle/TestOnly/Delete，禁止Unknown。
4. 记录当前cast/damage/heal/death/threat/affix场景的state/event/RNG/latency基线，不把missing owner记pass。

### M1 · Definition、Program 与 Activation Kernel

1. 建立typed IDs、CombatBuildSet与immutable prepared registries。
2. 从catalog生成CombatProgram，替换slot/payload/completion/projectile四套dispatch ladder。
3. 落地ActivationHandle、phase graph、target policy、cost reservation、commit/cancel/end receipt。
4. 建立machine-readable 308项support/coverage matrix，unsupported能力fail-closed且对产品可见。

### M2 · Active Effect、Damage/Healing 与 Atomic Transaction

1. 建立ActiveEffectStore和duration/period/stack/inhibition policy。
2. 迁移damage/heal/absorb/DoT/HoT/ground/proc到统一pipeline和timer service。
3. 落地CombatTransaction write-set、stable event order、budget和rollback。
4. 删除77类WorldState直接HP/副作用写入路径，所有combat mutation经过窄component owner。

### M3 · Death、Threat、Control 与 Encounter Extension

1. 统一player/mob/pet/boss death、credit、cleanup、resurrection和progression handoff。
2. 替换flattened threat CSR与线性retarget，落地taunt/forced target/CC/DR合同。
3. 将Nythraxis、Drowned、mob affix等迁为typed extensions并注册产品schedule。
4. 删除不可达旧owner与WorldState重复实现；reachability/owner gate确保每个概念仅一份生产truth。

### M4 · Parity、Performance 与发行资格

1. 重建54个parity owners，执行golden differential并输出typed first diff。
2. 为308项能力补definition/program/terminal coverage；高风险能力补property/fuzz/golden。
3. 跑1/100/1k active entities、raid、proc storm、periodic burst、affix/Boss和24h soak。
4. interp/AOT、Windows/Linux、worker-count矩阵比较state/event root；发布同正确性oracle性能artifact。

## 14. Runtime 资格门

| Gate | 通过条件 |
|---|---|
| R13-G01 · Single authority | ability/activation/effect/aura/damage/heal/threat/death/encounter各只有一个production owner；旧WorldState/孤立module双authority为0 |
| R13-G02 · Reachability | 所有declared production combat modules从产品root可达且进入compiled plan；只被test引用的production owner为0 |
| R13-G03 · Ability coverage | 308项每项有stable ID、revision、support status、prepared program或明确unsupported reason；隐式漏项为0 |
| R13-G04 · Dispatch | slot/payload/completion/projectile统一走program executor；百级字符串/ability if ladder和未知silent return为0 |
| R13-G05 · Activation lifecycle | admission/reserve/commit/run/cancel/end转换完整，每个request和activation都有typed terminal receipt |
| R13-G06 · Atomicity | 一次combat operation的attribute/effect/threat/death/credit write-set全成功或全不发布；fault injection无partial state |
| R13-G07 · Effect policy | duration/period/stack/inhibition/expiry/catch-up可配置且有same-tick顺序测试；逐帧全量timer scan为0 |
| R13-G08 · Death | player/mob/pet/boss、DoT/reflect/simultaneous hit、disconnect/transfer、resurrection均幂等且credit唯一 |
| R13-G09 · Threat/control | hot lookup/insert/clear/retarget满足版本化复杂度budget；taunt/forced target/CC/DR有统一policy和generation safety |
| R13-G10 · Determinism | qualified RNG/time/numeric profile在backend/platform/worker矩阵得到同state/event root，差异定位到activation/effect/draw |
| R13-G11 · Encounter wiring | 所有宣称supported的Boss/affix extension进入产品schedule并通过activate/quiesce/migrate/deactivate与golden |
| R13-G12 · Test artifact | combat aggregate suite真实编译执行；release artifact不含contractTest/test main；missing/skipped/not-run不记pass |
| R13-G13 · Parity | 54个owner均存在并在current product root运行，golden hash/source identity匹配且actual/first-diff artifact可追溯 |
| R13-G14 · Performance | cast/hit/effect tick普通路径无字符串dispatch和无界临时allocation；scale sweep p99/RSS/ops斜率满足硬预算 |
| R13-G15 · Soak | raid/affix/proc/periodic/projectile/death-respawn 24h churn无state row、timer、effect、threat、RSS或queue单调泄漏 |
| R13-G16 · Evidence | 每个gate绑定BuildSet/program/schema/schedule/source/backend/hardware identity和raw artifact；空场景或旧golden不是证据 |

## 15. 状态与边界

本篇完成WOC combat物理目录、main可达图、casting/effect/aura/damage/heal/threat/death/control/Boss/affix状态、M4能力coverage、parity owner与当前动态验证边界的首轮E3审查；状态为 `review_complete / implementation pending / source_recheck_required`。

本篇不重复Runtime08G已经登记的引擎通用Ability System缺失，不重复Runtime12的world storage/schedule/codec，也不重复App03的host/VM transaction与writer-reader identity P0。后续实现必须先恢复可执行baseline，再按M1-M4逐层hard cut；在此之前，新增职业、Boss或effect不得继续复制WorldState字段、字符串dispatch和孤立contractTest模式。
