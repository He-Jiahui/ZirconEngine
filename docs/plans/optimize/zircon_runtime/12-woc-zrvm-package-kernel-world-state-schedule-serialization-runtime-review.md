---
related_code:
  - examples/woc/scripts/woc_game/plugin.toml
  - examples/woc/scripts/woc_game/woc_game.zrp
  - examples/woc/scripts/woc_game/src/main.zr
  - examples/woc/scripts/woc_game/src/kernel
  - examples/woc/scripts/woc_game/src/kernel/world.zr
  - examples/woc/scripts/woc_game/src/kernel/clock.zr
  - examples/woc/scripts/woc_game/src/kernel/entity.zr
  - examples/woc/scripts/woc_game/src/kernel/rng.zr
  - examples/woc/scripts/woc_game/src/protocol/binary.zr
  - examples/woc/scripts/woc_game/src/generated/contracts.zr
  - examples/woc/scripts/woc_game/src/generated/m8_eastbrook_encounter.zr
  - examples/woc/scripts/woc_game/src/world/state.zr
  - examples/woc/scripts/woc_game/src/world/player_motion.zr
  - examples/woc/scripts/woc_game/src/combat
  - examples/woc/scripts/woc_game/src/progression
  - examples/woc/scripts/woc_game/src/social
  - examples/woc/scripts/woc_game/src/instances
tests:
  - examples/woc/scripts/woc_game/woc_game_tests.zrp
  - examples/woc/scripts/woc_game/woc_world_state_tests.zrp
  - examples/woc/scripts/woc_game/src/kernel/test_main.zr
  - examples/woc/scripts/woc_game/src/kernel/tests.zr
  - examples/woc/scripts/woc_game/src/kernel/entity_test_main.zr
  - examples/woc/scripts/woc_game/src/world/state_test_main.zr
plan_sources:
  - docs/plans/optimize/zircon_app/03-woc-product-role-host-zrvm-transaction-state-client-server-integration-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/07-script-plugin-runtime-review.md
  - docs/plans/optimize/zircon_tooling/05-woc-content-codegen-build-scripts-generated-artifact-incremental-review.md
  - docs/plans/woc/00-woc-engine-capability-foundation.md
  - docs/plans/woc/01-woc-zrvm-one-to-one-replication.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/TickTaskManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Public/MassEntityManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Public/MassEntityQuery.h
  - dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Private/MassProcessingPhaseManager.cpp
  - dev/bevy/crates/bevy_app/src/main_schedule.rs
  - dev/bevy/crates/bevy_ecs/src/schedule/schedule.rs
  - dev/bevy/crates/bevy_ecs/src/storage/table/mod.rs
  - dev/bevy/crates/bevy_ecs/src/query/state.rs
  - dev/godot/scene/main/scene_tree.cpp
  - dev/Fyrox/fyrox-impl/src/engine/mod.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraph.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 12 · WOC ZrVM Package Kernel、World State、Fixed Schedule 与 Serialization Runtime 工程化差距

## 1. 结论

WOC 的 Zr 源码已经不是玩具规模：`woc_game/src` 物理目录有 817 个 `.zr`、246,765 行、9,978,430 bytes；其中 `world` 为 208 个文件/102,546 行，`generated` 为 102/81,093，`combat` 为 197/21,357，`progression` 为 101/12,849，`instances` 为 103/10,993，`social` 为 60/8,425。大量规则、合同、确定性随机数和旧版本读取代码值得保留，不能以“示例”为由推倒重写。

但当前产品运行根不是工程级 package kernel，而是一份 68,730 行、3.30 MiB 的 `world/state.zr`。该文件拥有 534 个公开字段，其中 325 个是 `entity*` 字段、158 个是 `offline*` 字段；顶部直接导入 109 个模块，整文件含 538 个 `%import` 表达式、1,691 个顶层函数、1,010 个 `while`、728 个 `.indexOf(`。它同时承担实体存储、命令路由、战斗/进度/社交/副本规则、迁移、校验、二进制 codec、fixed schedule 和 204 个内嵌测试。模块数量很多，但状态和执行 authority 仍集中在一个可任意修改平行列的类中。

`fixedTick` 不是 scheduler。它先解码完整 input/state，把命令列再次复制到 `CommandBatch`，再手写顺序调用 51 个阶段函数，最后重编码完整 state。51 个阶段中有 35 个直接扫描 `entityIds`，这些函数体合计出现 41 个全实体扫描条件、56 个 `while` 和 20 次线性 entity lookup；尚未计入其调用的下层函数。没有 system descriptor、read/write set、依赖 DAG、ambiguity check、并行 executor、dirty frontier、world partition、deadline 或 per-system telemetry。增加一个 aura/职业/副本功能，当前默认做法就是给所有实体增加列、给每个 tick 增加一次扫描或分支。

内部 codec 又把这种成本放大。state writer/reader 分别是 2,200/2,485 行，含 548/546 个手写 primitive codec 调用点和 92/217 个 `schemaVersion` 分支命中。单实体编码循环静态可见 244 个写入点，固定部分下界约 1,449 bytes/row，另有 11 个可变子循环；decoder 虽允许 100,000 entities，但 64 MiB state envelope 在不计全局/可变数据时最多只能容纳约 46,313 行。普通 tick 内部至少经历 input 全量复制、committed state 抽取复制、完整 decode、完整 encode、writer finish 再复制、digest 全扫描、snapshot writer 再复制、final finish 再复制。`readF64LeAt` 每次还会复制整个 payload 并从头跳到 offset。

长期世界生命周期也不成立。真实 `WorldState` 从不物理删除 entity row；替换 pet 只把行标记为 dead/inert，代码明确把删除推迟到 Plugins08。退役行因此永远占据 325 个实体列、snapshot 和后续全量扫描。另一个 `kernel/entity.zr` 虽有 generation reuse/despawn，却是私有、AoS、线性查找的测试孤岛，产品 `world/state.zr` 不使用它。`kernel/world.zr` 只有 37 行常量和 contract test；`kernel/clock.zr` 使用累加 `float + 0.05`；`kernel/rng.zr` 又保存 module-global shared stream。所谓 kernel 与实际产品 authority 是两套实现。

snapshot restore 也不是纯函数。`decodeState` 在读完 bytes 后会 seed item discovery、重算 deeds、补生成 quest NPC、normalize title；同一 snapshot 因当前 catalog/build 不同可能产生不同 world。当前 writer 写 118 而 reader 只接受到 117 的硬错误已由 App03 作为跨宿主 schema P0 拥有，本篇不重复计数，但所有 codec/migration里程碑必须以修复该 blocker 为入口。

本轮登记 **5 项 P0、60 项 P1 和 14 项 P2**。正确方向不是继续给 `WorldState` 追加列和 `fixedTick` 追加调用，而是建立唯一 package kernel：world-owned entity/component storage、可声明依赖和访问集的 fixed schedule、明确的 structural command barrier、按 shard/page 的增量状态、纯 decoder + 显式 migration graph、稳定 command result/event journal，以及与生产包分离的测试构建。只有这些基线通过长期 soak 和硬件性能门，WOC 才能成为验证 ZirconEngine 上限的产品，而不是用代码体积掩盖缺失的 runtime architecture。

## 2. 审查边界与证据

### 2.1 物理范围

| 子域 | 文件 | 行 | bytes | 本篇关注点 |
|---|---:|---:|---:|---|
| `world` | 208 | 102,546 | 4,245,497 | authoritative state、motion、collision、AI slice、fixed root |
| `generated` | 102 | 81,093 | 3,362,552 | runtime catalog访问形状、contract test混入生产路径 |
| `combat` | 197 | 21,357 | 815,747 | state接线与schedule成本；具体玩法完整性留后续专题 |
| `progression` | 101 | 12,849 | 486,872 | inventory/talent/deed/craft state接线 |
| `instances` | 103 | 10,993 | 410,346 | pet/delve state、RNG owner、world边界 |
| `social` | 60 | 8,425 | 305,975 | party/chat/card service与结果通道 |
| `kernel` | 13 | 3,004 | 93,814 | 实际kernel与fixture/test authority辨识 |
| `protocol` | 5 | 2,650 | 110,168 | fixed input/snapshot binary codec |
| `parity` | 14 | 2,812 | 110,327 | RNG/trace fixture；不重复App03 runner结论 |
| `content` + `world_api` + root | 14 | 1,036 | 37,132 | package entry、薄合同与content facade |

`world/state.zr` 的 production 部分到 `fixedTick` 结束共 46,132 行；紧随其后的 `selfTest` 尾段有 22,598 行，占文件 32.88%，创建 `WorldState` 314 次并定义 204 个公开测试函数。整个 source tree 有 369 个 `*test_main.zr`，合计 5,634 行/222,739 bytes；另有 325 个带公开 test symbol 的源文件，合计 178,940 行、574 个公开 test symbol。两种集合有交叠但不是同一分类，说明测试入口、测试实现和生产模块长期混放。

### 2.2 World state 与 hot path 量化

| 信号 | 当前值 | 工程含义 |
|---|---:|---|
| `WorldState` 公开字段 | 534 | 325 entity / 158 offline / 51 其他；任意函数可破坏跨列不变量 |
| 顶部直接 import | 109 | package root静态耦合到combat/progression/world/social/instances/generated |
| 整文件 import表达式 | 538 | dependency与加载点散落到函数和测试中 |
| `fixedTick` 直接阶段调用 | 51 | 顺序由源码位置手工维护 |
| 直接扫描entity的阶段 | 35 | 至少41个全实体扫描命中/帧 |
| `partyEntityIndexById` 调用点 | 167 | 全文件168个文本命中含1处定义；helper本身从index 0线性扫描 |
| `applyCommands` command ID绑定 | 86 | 442行单函数分发、68个显式else-if command分支 |
| state encode/decode | 2,200 / 2,485行 | schema、validation、migration和业务repair揉在一起 |
| encode/decode primitive调用 | 548 / 546 | 无schema IR生成的手写镜像合同 |
| 单实体静态写入点 | 244 | 约1,449 fixed bytes/row，未计可变CSR/collection |
| decoder entity cap | 100,000 | 与64 MiB envelope及当前row size不可同时成立 |

`fixedTick` 的当前顺序是：decode input和state digest；构建/恢复完整 `WorldState`；逐列复制 command batch；apply commands/movement；advance time；依次运行 gathering/emote/charge/cooldown/loot、多个ground/projectile/aura/proc/player/pet/mob/dot/party/card/resurrection阶段；完整 encode；完整 digest；封装 snapshot。该函数没有根据 world kind、活跃 archetype、dirty component、pending timer、interest cell 或 capability裁掉任何阶段。

### 2.3 Codec 与生命周期证据

- `protocol/binary.zr:66-74` 的 `ByteReader` constructor复制完整 source；`readBytes:165-193` 再复制字段；`ByteWriter.finish:293-303` 再复制完整结果。
- `readF64LeAt:206-220` 为读取8 bytes而构造上述 reader、复制整个 source并线性跳过offset；`world/state.zr` 当前有10个调用点。
- `StateWriter.finish:2766-2775` 再复制完整 state；`stateDigest:2779-2787` 全扫描；snapshot writer随后逐byte复制state并再次finish复制。
- `encodeStateVersion:26918-29117` 手写支持80..118；`decodeState:34539-37023` 手写接受2..117。此 identity blocker由App03 `WOC-APP-P0-005` canonical拥有。
- `decodeState:37003-37016` 在 reader完成后运行migration及五类当前业务repair，其中 `ensureOfflineQuestNpcs` 会扫描并spawn缺失NPC。
- entity decode在`34814-34817`允许100,000行；encode只写u32 count，没有对100,000或64 MiB做preflight，大小拒绝发生在完整candidate已经分配之后。
- `retireOfflineOwnedPet:6228-6249` 明确不删除row；真实entity columns没有通用remove/compaction路径，generation也从不递增复用。

### 2.4 动态验证边界

本篇复用App03记录的 Windows验证结果：WOC native workspace在 `woc_protocol` 编译阶段有6个现存错误，测试没有开始；未变化的失败lane不重复运行。当前仓库也没有可用的真实 `WocProjectVm`/产品host来执行解释器fixed tick，因此本篇没有把静态self-test数量当成动态通过证据。

本轮新执行的都是只读结构检查：文件/行/bytes inventory、fixed stage解析、entity scan与lookup计数、codec调用/版本分支计数、test-tail边界和reference source联读。没有修改 WOC production、test、manifest、generated artifact 或 lockfile。

### 2.5 参考引擎约束

- Unreal `TickTaskManager.cpp` 为tick function维护group、prerequisite、completion event、any-thread和cycle处理；Mass在phase manager中求解processor dependency、裁掉无匹配archetype的processor，并通过query按archetype chunk并行遍历、用deferred command buffer处理结构变更。WOC无需照抄UObject/Mass，但必须达到“访问声明 -> 依赖图 -> 可执行计划 -> structural barrier -> telemetry”的闭环。
- Bevy `main_schedule.rs:221-235` 和 `349-365` 将Main/FixedMain阶段作为可插入顺序资源，schedule graph维护hierarchy/dependency/ambiguity，query缓存匹配table/archetype，table支持swap-remove。当前WOC的51行手写调用序列和平行全世界列没有同类运行时合同。
- Godot `SceneTree` 将physics/process frame分开，并在process group上排序、分组和可选worker执行；它不是ECS，但说明成熟scene runtime也不会把所有节点逻辑塞进一个world函数。
- Fyrox `Engine::pre_update/update/post_update` 明确plugin、script、resource、scene的阶段和lag语义，并记录plugins time。它的插件循环仍较保守，不能作为WOC大规模ECS性能上限，但至少拥有真实lifecycle context、error queue和阶段统计。
- Unity Graphics本地镜像只用于“声明式执行图”的有限类比：RenderGraph记录pass/resource side effect，compile、cull/merge并execute。它不是gameplay/world scheduler，本文不把graphics API误当成Unity完整runtime参考；可复用的是先声明资源访问再生成计划的工程方法。

## 3. 可保留的正确基础

### 3.1 Deterministic tick 与状态内RNG有正确意图

`advanceState` 要求下一tick和同代/下一reload generation，并用整数microseconds推进；`WorldState`保存RNG state/draw count/digest，多个combat adapter尝试把完整cursor取出再写回。这些不变量应迁入新kernel，而不是丢弃后改用wall clock或无记录随机数。

### 3.2 输入、collection与finite value已有大量上限检查

command/movement count、payload bytes、projectile/dot/charge等pending collection已有显式上限；state decoder检查严格递增entity ID、column长度、enum/boolean和trailing bytes。问题是这些检查散落在巨型函数且经常在分配后生效，不是“没有任何防御”。重构应从同一schema IR生成bounded reader、preflight和fuzz target。

### 3.3 Generation identity与严格命令序列值得保留

command actor同时带id/generation，per-actor sequence要求严格递增；movement也有ack/accepted tick。新的entity table和command journal必须保留这些语义，并补齐reject/result receipt，而不是退回裸整数entity ID和静默失败。

### 3.4 Candidate state思路优于直接改committed bytes

`fixedTick`从committed snapshot解码candidate，throw时外层意图是不发布candidate。App03已指出VM内部rollback仍未被接口证明，但“先candidate、后commit”的方向正确。新实现应把candidate从完整深拷贝升级为COW page/write-set transaction。

### 3.5 Generated catalog具有source-pinned provenance

生成模块包含hash、固定顺序、contract test和边界检查；M8 encounter也保留source entity/spawn order。应该把branch-ladder accessor转成只读typed table/artifact，并把contract test移出产品调用，而不是抛弃provenance后手写常量。

## 4. P0：Runtime Kernel 落地前必须硬阻断

### WOC-ZRRT-P0-001 · `WorldState` 是跨全部业务域的单一可变 god authority

68,730行文件同时拥有534个公开字段、109个顶部依赖、实体spawn、command reducer、combat/progression/social/instance规则、codec、migration、fixed schedule和测试。325个entity列只靠手写append与一次巨型length校验保持对齐；任何新增字段都必须同步constructor、多个spawn、历史decode、encode、validation和测试。当前结构无法给不同system声明最小读写权、无法做并发借用/冲突检查，也无法独立演进schema shard。

必须先定义 `WocWorld` owner graph：EntityRegistry/LocationTable、按component/archetype或稳定chunk组织的storage、domain resources、structural command buffer、query cache、event journal和snapshot view。`world/state` 最终只保留兼容入口/调度facade并在hard cut后删除；禁止用partial class或继续拆“helper文件”但共享全部534字段来伪装模块化。

### WOC-ZRRT-P0-002 · Fixed tick 是51阶段串行全扫描，性能复杂度随功能线性恶化

51个手写调用里35个直接遍历entity rows，且全文件下层又频繁调用167处线性ID查找。一个entity可能在同一20 Hz tick被movement、cooldown、十余aura/proc、auto attack、pet、mob、dot、party等多次触碰，即使它不具备对应component或没有pending work。没有依赖图、query/archetype过滤、timer wheel、dirty frontier、partition或并行计划，无法证明大world的上界，更不可能以此超过Unreal。

建立 `FixedScheduleDescriptor` 和compile step：system声明phase、before/after、read/write component/resource、structural effects、determinism class、budget和capability；编译器检查cycle/ambiguity/conflict，生成可复现batch，按匹配chunk执行并在barrier提交structural commands。空query、无timer、未启用capability和未加载partition必须零成本或接近零成本跳过。

### WOC-ZRRT-P0-003 · 普通tick内部强制全量state materialization与多重复制

一次tick会完整复制input、抽取并复制committed state、重建全部arrays、完整重编码、复制writer结果、全扫描digest、再次写入snapshot并复制最终output。64 MiB只是单字段上限，不是实际内存/带宽上限；在解释器和GC环境里，多份峰值resident、逐byte `Array.add` 与51阶段扫描叠加后没有工程化性能空间。App03拥有host跨边界full-state P0，本项只拥有Zr内部codec/copy算法。

普通tick必须消费qualified base generation和bounded command/input delta，在COW page/chunk上产生write-set、event journal和incremental projection；digest只覆盖changed pages并通过Merkle/page tree形成world root。完整snapshot只允许checkpoint/resync/save触发，异步、budgeted并有generation lease；任何ordinary tick出现O(world bytes) encode/copy都应被性能门拒绝。

### WOC-ZRRT-P0-004 · Entity没有物理despawn，长期运行必然积累永久扫描的墓碑行

替换或移除pet只设置dead/inert字段，代码明确承认无法删除完整平行row。主`entityIds`没有remove、generation reuse或location修复，所有退役实体仍保留325列、变长offset、完整codec和每帧system筛选成本。100,000 cap只能把最终失败推迟，不能提供稳定长期world。

实现generation-safe physical despawn：deferred structural barrier原子删除/move row，更新location、关系、query/archetype、sparse set、replication/persistence引用并发出despawn receipt；ID reuse必须递增generation。建立反复summon/despawn、zone churn、24h server soak，live entity归零后RSS/snapshot/query scan回落到稳定水位。

### WOC-ZRRT-P0-005 · Decoder带当前业务副作用，snapshot restore不可复现

`decodeState` 在bytes完全消费后继续seed discovery、重算deed、补NPC和normalize title。旧版本迁移、损坏修复、当前catalog reconciliation与普通decode没有边界或receipt；相同bytes在不同BuildSet/catalog上可能生成不同entity和奖励状态，load本身也会消费nextEntityId并扩大snapshot。这破坏rollback、replay、hot reload、crash recovery和跨版本parity。

将decoder收敛为纯 `bytes + schema descriptor -> decoded old state`；migration由显式DAG按source/target BuildSet运行，输出changed field/entity、reason、loss policy、digest和可回滚receipt。content reconciliation是独立authoritative command/system，只能在明确版本和幂等key下执行。资格门要求同bytes+同BuildSet重复decode的canonical digest逐bit相等，纯decode不spawn、不发奖励、不改业务字段。

## 5. P1：Package Kernel 与 Authority

| ID | 当前差距 | 需要的工程化重构 |
|---|---|---|
| WOC-ZRRT-P1-001 | `kernel/world.zr` 只有37行频率/schema常量与test，不是产品world owner | 建立真实kernel facade，拥有world create/load/tick/checkpoint/destroy和全部child owner lifetime |
| WOC-ZRRT-P1-002 | `kernel/entity.zr` 的私有`EntityStore`不被产品state使用 | 选择唯一EntityRegistry实现，经过规模/ABI验证后hard cut；删除测试孤岛authority |
| WOC-ZRRT-P1-003 | 孤立EntityStore用object AoS、resolve/getById线性扫描、排序插入搬移、snapshot深拷贝 | 使用generation location table、chunk/table storage、O(1) resolve、batch spawn/despawn和COW snapshot |
| WOC-ZRRT-P1-004 | `kernel/clock`保存float秒并累加0.05，产品state另存integer micros | 唯一fixed-time resource保存integer tick/rational step/overstep；float只作为derived presentation值 |
| WOC-ZRRT-P1-005 | `kernel/rng`同时提供对象stream和module-global shared mutable stream | 禁止产品module-global RNG；所有stream由world/session owner持有并进入snapshot、transaction和trace |
| WOC-ZRRT-P1-006 | 整个world主要共享单一RNG cursor，system插入或多一次draw会扰动后续全部域 | 使用counter-based或qualified substream：world/domain/entity/action key；记录draw purpose和first divergence |
| WOC-ZRRT-P1-007 | 538个import表达式分散在顶层、函数和测试，runtime依赖/初始化点不透明 | 编译期materialize module DAG、cycle、init order和capability；hot path只使用resolved handles，不临时发现依赖 |
| WOC-ZRRT-P1-008 | package只声明`foundation.log`，实际执行world/combat/social/instance等大量runtime能力 | package descriptor列出required/optional capability、provider ABI、fallback和admission结果；缺失即不激活相关system |
| WOC-ZRRT-P1-009 | kernel下`lifecycle/locomotion/roster/targeting`大量代码是scenarioMetric/ContractTest fixture | 将reference parity adapter移到test package；kernel目录只保留生产owner与可复用primitive |
| WOC-ZRRT-P1-010 | `WorldState`所有跨域字段公开写，module函数无read/write facade | component/resource type拥有validated mutation API；跨域只能经command/event/query view，禁止裸列引用泄漏 |

## 6. P1：Entity Storage、Query 与 World Partition

| ID | 当前差距 | 需要的工程化重构 |
|---|---|---|
| WOC-ZRRT-P1-011 | 325个entity字段靠手工相同length维持行对齐 | schema注册component column，spawn bundle原子materialize；compile-time/generated layout validator拒绝漏列 |
| WOC-ZRRT-P1-012 | player/mob/pet/NPC都承担几乎全部列，大量永远无意义的默认值 | 按archetype/component composition存储，只为具备能力的entity分配combat/pet/social/progression状态 |
| WOC-ZRRT-P1-013 | `partyEntityIndexById`为O(N)且有167个调用点 | generation location table O(1) resolve；query批量取所需关系，禁止热循环重复ID lookup |
| WOC-ZRRT-P1-014 | 每条command从entity 0扫描actor，target等又重复扫描 | command admission一次resolve并缓存validated handles；batch按owner/archetype分组后dispatch |
| WOC-ZRRT-P1-015 | entity ID严格排序依赖append-only，删除/迁移没有location修复合同 | entity order与storage location解耦；deterministic iteration使用stable sort key/view而非禁止删除 |
| WOC-ZRRT-P1-016 | CSR-like offsets插入时逐项后移，threat/cooldown/known ability等会放大O(N)维护 | 使用per-entity small storage、paged arena或chunk sidecar；定义inline/overflow、compaction和allocation budget |
| WOC-ZRRT-P1-017 | threat、party、loot和target逻辑包含嵌套全表/关系扫描 | 建立关系索引和change-driven cache，声明失效条件；复杂度门覆盖dense raid/loot/threat场景 |
| WOC-ZRRT-P1-018 | decoder cap 100,000是孤立magic number，与64 MiB state和row下界冲突 | capacity由workload profile、partition和byte budget联合推导；preflight同时验证rows/bytes/collections |
| WOC-ZRRT-P1-019 | encode没有在分配前检查entity cap或最终state budget | schema提供checked encoded-size upper bound；超限在分配/copy前失败并返回typed budget diagnostic |
| WOC-ZRRT-P1-020 | 一个WorldState同时容纳overworld、party、arena、card duel、instance/offline状态 | 定义World/Zone/Instance/Session/Account shard owner与引用规则；不同lifetime不共享一块snapshot |
| WOC-ZRRT-P1-021 | 没有component/entity change tick、dirty page或query generation | mutation更新change version，query支持added/changed/removed；snapshot/projection只消费dirty frontier |
| WOC-ZRRT-P1-022 | 没有world streaming cell、inactive partition或跨区迁移事务 | partition owner管理load/activate/quiesce/transfer/unload；跨shard引用使用lease/qualified handle和失败回滚 |

## 7. P1：Fixed Schedule、Command 与 Execution

| ID | 当前差距 | 需要的工程化重构 |
|---|---|---|
| WOC-ZRRT-P1-023 | 51个阶段顺序硬编码在`fixedTick`函数体 | 用versioned schedule descriptor和compiled execution plan；顺序变化产生plan digest与reviewable diff |
| WOC-ZRRT-P1-024 | 35个阶段直接扫描全部entity，空能力/空队列仍进入函数 | query/archetype/timer/event驱动；空匹配集不创建iterator、不触发domain import或allocation |
| WOC-ZRRT-P1-025 | 没有First/Pre/Update/Post/Last或physics/network/presentation barrier | 定义稳定phase与允许插入点，明确input、simulation、structural commit、event、snapshot、projection顺序 |
| WOC-ZRRT-P1-026 | before/after只存在于手写行号，没有dependency graph/cycle检测 | system registration声明before/after/set，compile时拓扑排序、cycle path和missing owner fail-closed |
| WOC-ZRRT-P1-027 | system不声明component/resource access，无法查冲突或并行 | 生成read/write access set，冲突形成edge；无冲突chunk batch允许确定性parallel executor |
| WOC-ZRRT-P1-028 | system可在遍历中直接spawn/改任意列，没有structural barrier | structural command buffer按stable key合并，在显式barrier原子应用并更新query/location/event |
| WOC-ZRRT-P1-029 | 没有per-system deadline、work quota、yield或cancellation | system profile声明max chunks/items/time/fuel；overrun返回receipt并执行degrade/abort/reschedule policy |
| WOC-ZRRT-P1-030 | cooldown/aura/timer每帧扫描，而不是只处理到期集合 | world-owned timer wheel/min-heap按tick触发；取消、重排、snapshot和deterministic same-tick order有合同 |
| WOC-ZRRT-P1-031 | `advanceState`直接写50000，kernel/world/clock另有20Hz/0.05 authority | fixed-step由唯一schedule clock注入；schema只记录clock ID/rational step，禁止业务magic number |
| WOC-ZRRT-P1-032 | `applyCommands`绑定86个ID并用442行if/else路由 | schema生成command registry：ID、payload decoder、capability、auth、target phase、handler、result type |
| WOC-ZRRT-P1-033 | 业务拒绝大量`return`，但actor sequence仍推进且无结果 | 每条command生成Accepted/Rejected/Deferred receipt，含reason、server tick、sequence、effects/events |
| WOC-ZRRT-P1-034 | chat合法消息可消耗token却不产生任何效果，部分pet命令是显式no-op | capability未就绪时在admission拒绝且不消费业务资源；禁止“成功解码即假执行” |
| WOC-ZRRT-P1-035 | 只有整个tick的成功/throw，没有system/entity/command级诊断与统计 | 记录plan/system/query rows、duration、alloc、commands、events、skip reason和first failure，受cardinality budget约束 |

## 8. P1：Serialization、Schema 与 Migration

App03的 `WOC-APP-P0-005` 负责修复当前writer 118/reader 117/schema identity矛盾；下列条目不重复该P0，只定义其后的内部codec基线。

| ID | 当前差距 | 需要的工程化重构 |
|---|---|---|
| WOC-ZRRT-P1-036 | encode/decode合计4,685行并手写1,094个primitive调用点 | 从唯一schema IR生成reader/writer/size calculator/validator/migration fixture，手写业务codec归零 |
| WOC-ZRRT-P1-037 | encode/decode分别含92/217个version分支，旧版本逻辑永久留在当前hot函数 | current codec只读写current layout；历史版本进入独立adapter链，可按release窗口退休 |
| WOC-ZRRT-P1-038 | 一个WOS version同时控制combat/progression/social/entity所有字段 | 每个shard/component有qualified schema ID/version；world manifest只组合shard目录和依赖 |
| WOC-ZRRT-P1-039 | wire没有section table、field ID、offset/length或unknown-field policy | 使用bounded section directory、stable field IDs、可跳过块、required/optional标志和unknown preservation策略 |
| WOC-ZRRT-P1-040 | 64 MiB state是不可随机访问的单blob | page/chunk/shard framing，支持lazy read、partial checkpoint、interest load和独立corruption隔离 |
| WOC-ZRRT-P1-041 | 没有per-section checksum、root tree或authenticated storage identity | page checksum + Merkle/root digest；持久化/网络按威胁模型再加authenticated integrity和签名 |
| WOC-ZRRT-P1-042 | `StateWriter.finish`和protocol writer都复制完整buffer | writer拥有可冻结buffer/segment chain，finish零拷贝转移ownership；host用lease/iovec传输 |
| WOC-ZRRT-P1-043 | protocol ByteReader constructor和`readBytes`重复复制 | bounded slice/view reader，字段返回borrowed range或arena handle；需要ownership时显式一次materialize |
| WOC-ZRRT-P1-044 | `readF64LeAt`每次复制完整payload并从头跳offset | 提供checked random-access primitive或cursor slice，读取复杂度O(width)且零额外payload copy |
| WOC-ZRRT-P1-045 | f64解码以最多1,074次乘/除计算2的幂 | VM/core提供经过位级/标准库验证的finite LE f64 primitive，并覆盖NaN/Inf/subnormal/-0 policy |
| WOC-ZRRT-P1-046 | `(1,1,2,3,5,8)` marker参数被当作backend签名缺陷规避 | 修复ZrVM overload/signature ABI；codec API用type/endianness descriptor，删除运行时magic marker分支 |
| WOC-ZRRT-P1-047 | `fixed6` primitive自身不验证finite、业务range和可表示上界 | schema字段声明numeric domain/scale/rounding/overflow；writer preflight fail-closed，reader产出typed error path |
| WOC-ZRRT-P1-048 | migration/repair没有source/target、changed set、loss、耗时或幂等receipt | migration runner输出qualified receipt、before/after digest、changed shards、warnings、loss和rollback artifact |

## 9. P1：Determinism、Testing 与 Diagnostics

| ID | 当前差距 | 需要的工程化重构 |
|---|---|---|
| WOC-ZRRT-P1-049 | `world/state.zr`尾部22,598行/204个测试函数与production同module | 移到独立test package/crate；production artifact和interp module不解析/发布测试symbol |
| WOC-ZRRT-P1-050 | 325个生产源文件携公开test symbol，test owner与runtime API混合 | test通过friend/test adapter访问窄接口；release symbol/module inventory拒绝test entry |
| WOC-ZRRT-P1-051 | 369份手写`.zrp` test manifest分散维护入口 | 由typed test catalog生成matrix，声明unit/contract/integration/perf/fuzz/product及required platform |
| WOC-ZRRT-P1-052 | `lifecycleSelfTest`导入23个模块、调用19个contract test并执行巨大world selfTest | 启动只做轻量ABI/schema/admission；深测在CI/diagnostic product显式运行，不能污染activate latency |
| WOC-ZRRT-P1-053 | production encounter bootstrap调用generated `contractTest()` | build/cook时验证artifact并记录receipt；runtime只验证artifact identity/shape，不执行测试逻辑 |
| WOC-ZRRT-P1-054 | 没有system级CPU、allocation、rows visited、cache/memory bandwidth指标 | schedule自动埋点并生成budgeted profile；指标绑定plan/world/build/workload identity |
| WOC-ZRRT-P1-055 | 没有state copy/encoded bytes/dirty ratio/page reuse指标 | 每tick记录logical changes、physical writes、copy bytes、snapshot pages、compression与peak resident |
| WOC-ZRRT-P1-056 | shared RNG只能给出全局draw count/digest，无法定位哪个system多draw | trace记录qualified stream/system/entity/action cursor；first divergence直接指向draw purpose |
| WOC-ZRRT-P1-057 | float clock与多模块临时float状态缺少跨backend数值合同 | 明确fixed/floating math profile、rounding/denormal/NaN策略；interp/AOT/平台golden覆盖 |
| WOC-ZRRT-P1-058 | state差异只能比较整blob digest或大量手写断言 | 提供schema-aware state diff：shard/entity/component/field、before/after、first writer和command/event provenance |
| WOC-ZRRT-P1-059 | 没有固定world-size sweep或复杂度回归门 | 以1/100/1k/10k active+inactive entity、dense effects/commands做sweep，检查斜率、p50/p95/p99和allocation |
| WOC-ZRRT-P1-060 | 没有反复load/migrate/tick/despawn/checkpoint的长期soak | 24h+ deterministic soak注入reload、corruption、budget、partition churn，验证RSS、row、snapshot和latency稳定 |

## 10. P2：完成度与维护性

| ID | 当前差距 | 收敛方向 |
|---|---|---|
| WOC-ZRRT-P2-001 | `WorldState`、codec、schedule和tests在同一命名空间难以导航 | 按owner目录与公开facade组织，索引生成module/owner/dependency图 |
| WOC-ZRRT-P2-002 | WOS编号散落在注释、分支和函数名 | generated schema changelog显示introduced/readable/writable/retired，代码只使用typed version |
| WOC-ZRRT-P2-003 | 多个函数用`marker/required/observe`布尔或整数制造不同签名 | ABI修复后使用窄type和明确方法名，非法状态不可构造 |
| WOC-ZRRT-P2-004 | 大量错误是无结构throw字符串 | typed error含code/stage/schema/field/entity/offset/retryability，presentation再本地化 |
| WOC-ZRRT-P2-005 | 生成catalog使用长if链和string field selector | 生成packed readonly table、enum field ID和checked indexed accessor |
| WOC-ZRRT-P2-006 | source comment频繁写“until Plugins08”代替可追踪dependency | 用plan dependency ID/capability gate；过期comment由source recheck清理 |
| WOC-ZRRT-P2-007 | `offline*`前缀长期侵入通用WorldState | Offline profile是world configuration/plugin，不是所有runtime component的命名和storage owner |
| WOC-ZRRT-P2-008 | hard-coded 20/60 Hz在main、kernel、state、schema重复 | 唯一qualified clock profile生成runtime/schema/presentation projection |
| WOC-ZRRT-P2-009 | command helper大量裸payload offset/length参数 | generated bounded payload view和typed command DTO，handler不做重复手写byte arithmetic |
| WOC-ZRRT-P2-010 | 多处one-line return隐藏reject原因 | handler返回typed outcome，显式区分NoChange/Rejected/Consumed/Deferred |
| WOC-ZRRT-P2-011 | state validation巨型布尔表达式难以定位坏列 | schema validator返回精确path/index/expected/actual和受限sample |
| WOC-ZRRT-P2-012 | 测试函数用负整数编码失败位置 | test harness使用断言/diagnostic，保留case ID和first diff，不维护手工负数表 |
| WOC-ZRRT-P2-013 | `kernel`目录同时放production primitive、parity fixture和test main | 物理分离`runtime/kernel`、`test_support/parity`、`tests/entries` |
| WOC-ZRRT-P2-014 | App03、Tooling05与本篇容易重复WOC问题 | App03拥有产品host/跨VM事务，Tooling05拥有生成链，本篇唯一拥有Zr内部world/schedule/storage/codec |

## 11. Owner 与依赖收敛

| Owner | 唯一职责 | 禁止承担 | 前置依赖 |
|---|---|---|---|
| `WocPackageKernel` | package admission后创建/销毁world，绑定clock/schedule/schema/capability | client/server host、codegen、业务规则细节 | Runtime01/07、App03真实VM adapter |
| `WocEntityRegistry` | generation、location、spawn/despawn、structural barrier | combat/progression字段、snapshot wire | Runtime05 ECS identity基线 |
| `WocComponentStorage` | archetype/chunk/table、query/change version、allocation/compaction | command routing、跨world全局状态 | EntityRegistry |
| `WocFixedSchedule` | phase、access DAG、executor、budget、telemetry | 直接拥有业务state或wire codec | Core task/diagnostics、component schema |
| `WocCommandRegistry` | typed decode、auth/capability、dispatch phase、outcome/event receipt | 86项手写if/else、UI/network transport | Protocol authority、schedule |
| `WocSnapshotStore` | page/shard snapshot、COW transaction、root digest、lease | 业务repair、current content lookup | App03 transaction、schema registry |
| `WocSchemaRegistry` | current codec生成、version range、migration DAG、unknown/loss policy | 人工WOS字符串authority | Tooling05 generator transaction |
| `WocMigrationRunner` | pure old decode、显式migration、receipt/rollback | 普通load中隐式发奖励/spawn | SchemaRegistry、BuildSet identity |
| `WocTestRuntime` | test catalog、fixture、fuzz/parity/perf/soak entry | production activation/self-test hot path | Tooling test inventory、App03 runner |

跨报告依赖顺序固定为：Tooling05先恢复clean/generated source与schema原子性；App03恢复native compile、唯一world schema identity和真实VM transaction；本篇再替换Zr内部state/schedule/codec。不能在writer 118/reader 117仍冲突或真实VM adapter不存在时，把新storage性能数字当作产品证据。

## 12. 重构里程碑

### M0 · Truth Freeze、Schema Roundtrip 与 Baseline Capture

1. 修复App03/Tooling05前置P0，clean source graph、native compile、writer-reader roundtrip全部通过。
2. 冻结当前WOS fixture和可达command行为，生成field/system/import/owner inventory。
3. 对1/100/1k实体记录tick阶段、scan、lookup、encode/copy bytes、peak memory和snapshot size基线。
4. 给partial/no-op command与AI slice生成machine-readable capability状态，禁止继续扩大无结果surface。

### M1 · 唯一 Package Kernel 与 State Ownership

1. 建立 `WocPackageKernel/WocWorld` lifecycle、integer fixed clock、world-scoped RNG registry。
2. 按entity core、motion/combat/progression/social/instance/resource拆component/schema owner。
3. 删除产品对孤立kernel entity/clock/shared RNG authority的依赖；parity fixture移入test package。
4. 建立typed cross-domain command/event，不允许新代码引用整个mutable world列集合。

### M2 · Entity/Component Storage 与 Structural Barrier

1. 落地generation location table、archetype/chunk或经基准证明等价的SoA storage。
2. 提供cached query、change tick、relation index、timer collection和stable deterministic iteration。
3. spawn/despawn/move component经deferred structural command原子提交。
4. 反复pet/zone/instance churn证明row、RSS、snapshot可回收且stale handle失效。

### M3 · Compiled Fixed Schedule 与 Command Outcome

1. 将51阶段迁为descriptor，声明phase、access、dependency、capability和budget。
2. compile DAG、冲突batch、empty-query pruning和deterministic parallel execution。
3. 86 command路由迁为generated registry，返回typed outcome/event journal。
4. per-system profile和first divergence trace进入诊断，不依赖手写计数器。

### M4 · Sharded Codec、COW Transaction 与 Pure Migration

1. 从schema IR生成current shard codec、size preflight和bounded slice reader。
2. ordinary tick改为COW page/write-set；完整checkpoint异步生成并持有generation lease。
3. historical decoder和migration DAG与current hot codec物理分离。
4. 删除decode业务副作用，content reconciliation改为幂等authoritative operation并记录receipt。

### M5 · Test Artifact 分离与长期资格

1. 将22,598行state tests和所有test main移出production package build。
2. generated test catalog运行roundtrip/migration/fuzz/determinism/fault/perf/soak矩阵。
3. Windows/Linux、interp/AOT允许矩阵比较state/event root与first divergence。
4. 同硬件、同world、同正确性oracle对Unreal/Fyrox/Bevy/Godot相关基线做可解释性能比较；Unity Graphics只比较声明式graph方法，不伪造world对照。

## 13. Runtime 资格门

| Gate | 通过条件 |
|---|---|
| R12-G01 · Authority | product world只有一个EntityRegistry、Clock、RNG registry、SchemaRegistry和Schedule owner；旧`WorldState` authority为0 |
| R12-G02 · Module boundary | 每个component/resource/schema/system有唯一owner；任何system不得获得全world裸mutable column集合 |
| R12-G03 · Schedule | 全部fixed systems声明phase/access/dependency/capability/budget；cycle、ambiguity、missing owner和非法冲突为0 |
| R12-G04 · Query complexity | ordinary system只遍历匹配chunk/到期timer/dirty frontier；热路径线性ID lookup和无条件全world scan为0 |
| R12-G05 · Entity lifecycle | generation-safe physical despawn通过关系/引用/replication/persistence测试；24h churn后dead row、RSS和snapshot无单调增长 |
| R12-G06 · Ordinary tick copy | ordinary tick不encode/copy完整world；copy bytes与changed bytes同阶，完整checkpoint只在显式异步路径 |
| R12-G07 · Codec | 当前writer产物100%由当前reader接受；schema生成的size upper bound在分配前执行；unknown/required/loss policy明确 |
| R12-G08 · Restore purity | 相同bytes+BuildSet重复pure decode的canonical root逐bit相等；decode不spawn、奖励、repair或读取current wall time |
| R12-G09 · Migration | 每个支持版本有golden、forward migration、receipt、rollback与loss test；不支持版本返回typed拒绝且不产生partial world |
| R12-G10 · Command outcome | 每条admitted command产生Accepted/Rejected/Deferred receipt；无owner capability时不消费token/cooldown/sequence业务效果 |
| R12-G11 · Determinism | 同seed/input在允许平台/backend/worker count矩阵得到同state/event root；差异定位到system/entity/action |
| R12-G12 · Performance | 1/100/1k/10k workload sweep无意外超线性斜率；tick p99、allocation、RSS、copy、snapshot均满足版本化budget |
| R12-G13 · Test separation | release package不包含test main/public test symbol/deep self-test；启动admission时间不随test inventory增长 |
| R12-G14 · Evidence | 每个gate记录source/build/schema/schedule/workload/hardware identity和raw artifact；未执行、跳过、空world不得记pass |

## 14. 状态与边界

本篇已完成WOC Zr package kernel、`world/state`、fixed schedule、entity storage、binary/state codec、migration副作用、RNG/clock authority和测试混入生产路径的首轮E3审查；状态为 `review_complete / implementation pending / source_recheck_required`。

本篇不拥有四个native product role、network/persistence/client presentation或VM host transaction，它们由App03拥有；不拥有contract/codegen mixed-generation与test runner生成链，它们由Tooling05拥有；也不在此逐个判断197个combat、101个progression、103个instances和60个social模块的玩法完整性。后续专题应分别审查这些域的状态机、算法、来源一致性与性能，但所有域实现都必须先服从本篇kernel/storage/schedule/codec边界，不能再向`world/state.zr`追加临时列和手写tick调用。
