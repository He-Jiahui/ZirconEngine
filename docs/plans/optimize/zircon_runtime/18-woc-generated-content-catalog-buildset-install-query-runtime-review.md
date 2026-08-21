---
related_code:
  - examples/woc/scripts/woc_game/src/main.zr
  - examples/woc/scripts/woc_game/src/content
  - examples/woc/scripts/woc_game/src/content/camp_mob_spec.zr
  - examples/woc/scripts/woc_game/src/content/m3_mobs.zr
  - examples/woc/scripts/woc_game/src/content/rules.zr
  - examples/woc/scripts/woc_game/src/content/m4_ability_catalog_test_main.zr
  - examples/woc/scripts/woc_game/src/content/m5_content_catalog_test_main.zr
  - examples/woc/scripts/woc_game/src/generated
  - examples/woc/scripts/woc_game/src/generated/contracts.zr
  - examples/woc/scripts/woc_game/src/generated/current_known_ability_catalog.zr
  - examples/woc/scripts/woc_game/src/generated/current_talent_selection_catalog.zr
  - examples/woc/scripts/woc_game/src/generated/m3_camp_mob_core.zr
  - examples/woc/scripts/woc_game/src/generated/m3_npc_initialization.zr
  - examples/woc/scripts/woc_game/src/generated/m4_ability_catalog.zr
  - examples/woc/scripts/woc_game/src/generated/m4_ability_effects.zr
  - examples/woc/scripts/woc_game/src/generated/m5_camp_mob_loot.zr
  - examples/woc/scripts/woc_game/src/generated/m5_class_baseline_stats.zr
  - examples/woc/scripts/woc_game/src/generated/m5_content_catalog.zr
  - examples/woc/scripts/woc_game/src/generated/m8_eastbrook_encounter.zr
  - examples/woc/scripts/woc_game/src/generated/m8_fresh_player_stats.zr
  - examples/woc/scripts/woc_game/src/generated/m8_offline_bootstrap_content.zr
  - examples/woc/scripts/woc_game/src/generated/talent_modifier_catalog.zr
  - examples/woc/scripts/woc_game/src/generated/talent_proc_catalog.zr
  - examples/woc/scripts/woc_game/src/world/state.zr
  - examples/woc/scripts/woc_game/src/progression/talent_modifier_state.zr
  - examples/woc/scripts/woc_game/src/combat/talent_proc_state.zr
  - examples/woc/tools/package.json
tests:
  - examples/woc/scripts/woc_game/woc_m3_camp_mob_core_tests.zrp
  - examples/woc/scripts/woc_game/woc_m3_npc_initialization_tests.zrp
  - examples/woc/scripts/woc_game/woc_m4_ability_catalog_tests.zrp
  - examples/woc/scripts/woc_game/woc_m5_camp_mob_loot_tests.zrp
  - examples/woc/scripts/woc_game/woc_m5_content_catalog_tests.zrp
  - examples/woc/native/Cargo.toml
plan_sources:
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/12-woc-zrvm-package-kernel-world-state-schedule-serialization-runtime-review.md
  - docs/plans/optimize/zircon_runtime/13-woc-combat-casting-effect-aura-damage-threat-death-runtime-review.md
  - docs/plans/optimize/zircon_runtime/14-woc-progression-inventory-item-economy-crafting-quest-talent-runtime-review.md
  - docs/plans/optimize/zircon_runtime/15-woc-social-identity-party-raid-chat-duel-arena-matchmaking-minigame-runtime-review.md
  - docs/plans/optimize/zircon_runtime/16-woc-instance-dungeon-delve-pet-companion-lockout-reset-collision-runtime-review.md
  - docs/plans/optimize/zircon_runtime/17-woc-world-terrain-collision-locomotion-spawn-spatial-targeting-runtime-review.md
  - docs/plans/optimize/zircon_editor/24-data-table-structured-data-schema-import-validation-save-game-slot-migration-platform-cloud-storage-authoring-review.md
  - docs/plans/optimize/zircon_editor/40-procedural-content-generation-rule-graph-biome-world-generation-authoring-review.md
  - docs/plans/optimize/zircon_tooling/05-woc-content-codegen-build-scripts-generated-artifact-incremental-review.md
  - docs/plans/optimize/zircon_tooling/10-test-architecture-partition-selection-isolation-fixture-flake-results-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/DataTable.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/DataRegistry/Source/DataRegistry/Public/DataRegistry.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/AssetManager.h
  - dev/godot/core/io/resource.h
  - dev/godot/core/io/resource_uid.h
  - dev/godot/core/io/resource_loader.h
  - dev/bevy/crates/bevy_asset/src/assets.rs
  - dev/bevy/crates/bevy_asset/src/handle.rs
  - dev/bevy/crates/bevy_asset/src/server/mod.rs
  - dev/Fyrox/fyrox-resource/src/manager.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraphResourceRegistry.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 18 · WOC Generated Content、Catalog、BuildSet、Install 与 Query Runtime 工程化差距

## 1. 结论

WOC 的 `src/generated` 已经不是少量 glue，而是当前脚本产品最大的内容载体之一：102 个 `.zr`、81,093 行、3,362,552 bytes。排除一个 29 行 test main 后，101 个生产候选文件仍有 81,064 行；再加 `src/content` 三个非 test-main 模块，本篇物理主体达到 104 文件、81,440 行、3,374,512 bytes。生成目录包含 105 个 public var、1,098 个 public function、3,382 个 `throw` 和约 58,230 个 `if`。这套实现已把内容数据、identity lookup、字段选择、rank/effect 选择和错误策略一起编译成 ZrVM 控制流，不能再以“生成代码无需架构”解释其加载、查询和性能合同。

产品只从 `main.zr` 静态 import closure 到达 46 个 generated 文件、78,270 行；`content` 三个非测试 facade 全部断线。合并后有 58 个非 test-main 文件、3,170 行、147,798 bytes 不在产品根可达图，其中包括 `m3_npc_initialization`、corpse harvest、enchanting、ground object、mailbox、reserved NPC 与多份 contract/candidate。断线模块不应自动算已交付能力，但简单补 import 也不会建立唯一内容 authority：`WorldState` 当前直接 import M3/M4/M5/M8 与 current-* 具体模块，绕过了 `content` facade 和任何统一 registry。

内容代际没有进入产品 schema。`main.stateSchema()`只绑定 `generated/contracts.schemaFingerprint()`、command catalog、command payload schema、`WOS113` 与 20/60 Hz；21 个生成文件虽暴露 64 字符 catalog digest，但产品根不聚合也不校验它们。当前进程同时消费 308 项 known-ability catalog、117 项 M4 ability/effect projection、162 个 current talent options、189 个 talent modifiers、55 个 talent procs，以及 M3/M5/M8 分期投影。名称中的 milestone 不是兼容性合同；没有一份 machine-readable `ContentBuildSet` 能证明这些 shard 来自同一 source snapshot、schema、tool revision、target 和语义闭包。

当前数据形状也不具备工程级运行时成本模型。七个主要 catalog 合计 48,466 行、1,916,502 bytes、35,044 个 `if`、2,754 个 `throw`；`talent_modifier_catalog` 单文件 13,022 行并以字段名/entry index 分支，`m4_ability_effects` 13,077 行并含 2,113 个 `throw`。WorldState 和 talent state 在 command/tick 路径反复调用这些 accessor，部分路径还线性扫描 189 entries、构造多列 Array 和临时 definition/response 对象。当前没有 load-time decode、typed immutable table、key/index、prepared program、generation handle、query budget 或规模基线，因而不能证明增加内容量后 VM compile、startup、instruction count、allocation和cache行为仍可控。

证据链已经出现静态假绿。`m4_ability_catalog_test_main` 固定期待旧 SHA `1790cc...` 和 21 项，当前 generated 文件返回 `f4349a...` 和 117 项；`m5_content_catalog_test_main`期待旧 SHA `9ff52d...` 和 14 个 item，当前返回 `2d3f83...` 和 82 个 item。三份 content test main 没有 manifest，五份相关 manifest声明的 binary目录全部不存在。默认 npm check 又在更早步骤中断，native workspace也在既有 `woc_protocol` 编译错误处停止，因此这些确定失败尚未由产品证据 runner 执行。

本轮登记 **6 项 P0、68 项 P1 和 16 项 P2**。Tooling05继续唯一拥有 generator discovery、ContentBuildGraph、changed-set scheduling与原子产物发布；Runtime04继续拥有通用 asset/artifact/residency。本文只拥有生成内容被 runtime 安装、准入、索引、查询、兼容、热切换、回滚和语义验证的唯一产品边界。

## 2. 审查范围与执行拓扑

### 2.1 物理清单

| 范围 | 文件 | 行数 | bytes | 产品根可达 |
|---|---:|---:|---:|---:|
| `src/generated` 全量 | 102 | 81,093 | 3,362,552 | 46 文件 / 78,270 行 |
| generated 非 test-main | 101 | 81,064 | 3,361,294 | 46 文件 |
| generated 不可达非 test-main | 55 | 2,794 | 134,580 | 0 |
| `src/content` 全量 | 10 | 616 | 22,698 | 0 |
| content 非 test-main | 3 | 376 | 13,218 | 0 |
| 本篇非测试主体 | 104 | 81,440 | 3,374,512 | 46 文件 |
| 本篇不可达非测试主体 | 58 | 3,170 | 147,798 | 0 |

`src/content` 的 `camp_mob_spec` 是 current-source wrapper，`m3_mobs` 是旧七 mob 标量投影，`rules` 是十项测试规则；三者均未被 `main.zr` closure 消费。物理存在只能作为 source/candidate/oracle，不能替代产品 capability declaration。

### 2.2 Generated 文件分类

| 分类 | 文件 | 行数 | bytes | 可达文件 | 主要问题 |
|---|---:|---:|---:|---:|---|
| 核心 catalog/data | 7 | 36,148 | 1,549,982 | 7 | 数据与查找/错误策略被编译成巨型控制流 |
| contract projection | 78 | 8,502 | 377,201 | 29 | 协议/规则常量分片，没有统一安装与 capability manifest |
| M3/M4/M5/M8 projection | 14 | 36,296 | 1,427,208 | 9 | milestone 命名承担了错误的版本/分区语义 |
| test main | 1 | 29 | 1,258 | 0 | 测试与生产根 inventory 分裂 |
| wire/meta | 2 | 118 | 6,903 | 1 | 只有局部 schema/trace identity |

最大的七份表是 `m4_ability_catalog`、`m4_ability_effects`、`current_known_ability_catalog`、`current_talent_selection_catalog`、`talent_modifier_catalog`、`talent_proc_catalog` 与 `m5_content_catalog`。它们合计约占 generated bytes 的 57%，并承载跨 combat/progression/world 的热查询；任何性能结论都必须以这组真实数据而不是小 fixture 为基线。

### 2.3 产品消费与多套 authority

`WorldState` 顶部及函数体直接 import 多份具体生成模块。生产路径已同时形成以下视图：

| 内容域 | 当前投影 | 规模/形状 | 风险 |
|---|---|---|---|
| Ability identity | `current_known_ability_catalog` | 308 abilities / 9 classes | 与 M4 prepared/effect 目录不是同一完整性集合 |
| Ability definition/effect | `m4_ability_catalog` + `m4_ability_effects` | 117 abilities | known 但无 definition/effect 的支持状态未全局验证 |
| Talent selection | `current_talent_selection_catalog` | 27 specs / 162 options | 与 modifier/proc projection独立生成 |
| Talent modifier | `talent_modifier_catalog` | 189 entries | 运行时按entry/字段分支和线性查找 |
| Talent proc | `talent_proc_catalog` | 55 procs | definition/response逐次物化 |
| Progression content | `m5_content_catalog` | 82 items、2 quests等 | 旧测试仍期待14 items |
| World bootstrap | `m3_camp_mob_core` / `m8_eastbrook_encounter` | 47 mobs / 4 camps / 24 spawns | constructor snapshot与world generation绑定不清 |

这不是说不同目录规模必须相等，而是每个差集必须有明确 `Implemented`、`Unsupported`、`ServerOnly`、`Deprecated` 或 projection policy。当前没有 cross-catalog referential closure、capability matrix、unknown-key policy和生成/安装 receipt证明差集是有意设计。

### 2.4 Provenance 与 BuildSet 缺口

102 个 generated 文件中，92 个注释携同一 40 字符 source commit `5ef9f7cb21cd8875b6d2c49701015dfcd78de35a`；另外 10 个没有独立 source commit。21 个文件暴露一个独立的 64 字符 catalog/schema digest值，但没有产品级 aggregate。digest也没有统一绑定 generator binary/version、IR schema、dependency digests、target、feature set、compiler/VM ABI和canonical payload hash。

`stateSchema()`因此只能标识协议与世界存档外形，不能标识构成一次运行的内容集合。相同 `WOS113` 可在不同 ability/item/talent/encounter 表下启动、保存、重放或连接；当前没有 admission 规则说明哪些差异兼容、哪些需要迁移、哪些必须拒绝。

### 2.5 测试与 artifact 矛盾

| 证据项 | 现状 |
|---|---|
| content test mains | 7；其中 `camp_mob_spec_test_main`、`rules_test_main`、`test_main` 无 manifest |
| 相关 manifests | M3 mob、M3 NPC、M4 ability、M5 loot、M5 content 共5份 |
| checked-in binary dirs | 五份 manifest 对应目录全部缺失 |
| M4静态预期 | SHA/21 abilities 与当前 SHA/117 abilities冲突 |
| M5静态预期 | SHA/14 items 与当前 SHA/82 items冲突 |
| native lane | 既有 `woc_protocol` 6个编译错误，tests未开始 |
| npm lane | 默认check在前置typed contract计数冲突处中断，后续检查未执行 |

本轮没有重跑未变化的失败 lane，也没有把 static `contractTest()`、缺失 artifact 或 generator成功日志记为 runtime pass。

## 3. P0 阻断

| ID | 差距 | 当前证据 | 必须重构为 |
|---|---|---|---|
| CONTENT-P0-001 | 没有唯一 `ContentBuildSet` 与产品准入 | state schema遗漏21份catalog digest；M3/M4/M5/M8/current投影可任意混装 | signed/hashed aggregate manifest绑定source、schema、producer、dependency、target、VM ABI与全部artifact；启动、连接、restore、replay均先协商 |
| CONTENT-P0-002 | 内容被编译成无成本上限的脚本控制流 | 81K行、约58K `if`、3,382 `throw`；七大表35K `if` | cook为typed immutable table/index/prepared program；runtime按handle/query读取，热路径禁止解释巨型字段分支 |
| CONTENT-P0-003 | catalog authority分裂且无语义闭包 | 308 known abilities对117 M4 definitions；talent/content/encounter各自分代 | 唯一ContentRegistry记录每个definition、projection、support状态、dependency和consumer contract；cross-catalog closure fail-closed |
| CONTENT-P0-004 | product root绕过 facade且58个候选断线 | `WorldState`直接import具体generated模块；三个content owner全断线 | 所有domain通过generation-qualified registry/query接口消费；候选接入、迁为oracle或删除，禁止补import形成第二authority |
| CONTENT-P0-005 | 无install/hot reload/rollback与旧handle生命周期 | generated源码静态链接到package；无candidate generation、quiesce、lease、retire、last-good | artifact load→validate→prepare→atomic activate；旧generation由lease保活，失败回滚且存档/网络/replay绑定generation |
| CONTENT-P0-006 | content资格链静态过期且不可执行 | M4/M5预期已被当前产物推翻，3 tests无manifest、5 binary dirs缺失、默认lane提前停止 | generated inventory驱动required product-root test；stale expected、missing owner/artifact、not-run/skip均fail并输出完整receipt |

## 4. P1 工程化差距

### 4.1 Topology、Schema 与 BuildSet

| ID | 差距 | 重构要求 |
|---|---|---|
| CONTENT-P1-001 | 104个非测试文件没有统一production/candidate/oracle分类 | 生成source→producer→artifact→registry→consumer→evidence拓扑，未分类为0 |
| CONTENT-P1-002 | 46可达只说明import，不说明运行时capability | 每项capability声明definition set、handler owner、projection和required tests，构建时验证产品根闭包 |
| CONTENT-P1-003 | M3/M4/M5/M8名称承担版本职责 | artifact ID改为domain+schema+partition+revision；milestone只保留migration alias和历史记录 |
| CONTENT-P1-004 | 10份generated文件没有独立source commit | 每个artifact manifest记录完整input set及digest；aggregate不依赖自由文本注释 |
| CONTENT-P1-005 | source commit相同不能证明生成结果同代 | 另存generator/toolchain/IR schema/config/dependency/output digest并进入BuildSet root |
| CONTENT-P1-006 | 21份局部digest无人聚合 | `ContentBuildSetRegistry`计算canonical Merkle root并列出缺失、重复、未声明和跨代节点 |
| CONTENT-P1-007 | schema fingerprint只覆盖protocol/world外形 | 分离ProtocolSchema、WorldSchema、ContentSchema、ScheduleSchema并由ProductBuildIdentity显式组合 |
| CONTENT-P1-008 | 没有artifact dependency graph | ability→effect、talent→modifier/proc、mob→loot/spawn、item→recipe/quest等引用成为typed edges并检测cycle |
| CONTENT-P1-009 | 缺optional/required shard规则 | 每个profile/zone/mode定义required set与合法optional set；required缺失启动失败，optional缺失返回typed unsupported |
| CONTENT-P1-010 | content namespace与key格式未注册 | 定义stable domain/type/key、normalization、case policy、reserved range和collision检查 |
| CONTENT-P1-011 | 旧key/rename/delete没有生命周期合同 | alias/tombstone/redirect带introduced/deprecated/removed BuildSet与migration policy，禁止静默复用 |
| CONTENT-P1-012 | BuildSet未进入save/network/replay/crash identity | checkpoint、join handshake、trace、crash report和telemetry都携aggregate root与active generation |

### 4.2 Typed Data、Identity 与 Query

| ID | 差距 | 重构要求 |
|---|---|---|
| CONTENT-P1-013 | catalog以free function集合模拟table | 定义typed schema、column/record layout和只读view；生成payload，不生成大段业务控制流 |
| CONTENT-P1-014 | string key每次走线性if ladder | cook阶段生成minimal/perfect hash或排序index；lookup输出typed handle与稳定not-found结果 |
| CONTENT-P1-015 | integer index脱离generation后可误指新内容 | `ContentHandle<T>`包含registry/type/key/generation；index只在generation lease内有效 |
| CONTENT-P1-016 | accessor遇未知kind/field大量throw | admission先验证schema/query descriptor；运行时返回typed MissingKey/Field/Unsupported而非脚本异常控制流 |
| CONTENT-P1-017 | 字段名string反复扫描 | schema生成typed field IDs与column offsets；debug保留名称，热路径只用prepared accessor |
| CONTENT-P1-018 | rank/effect选择由generated ladder拥有 | hand-written domain compiler将definition编译为prepared ability/effect program，generated只承载数据 |
| CONTENT-P1-019 | modifier查询扫描189 entries | 按spec/allocation/ability/stat建立immutable secondary index并按allocation revision缓存compiled modifier set |
| CONTENT-P1-020 | proc查询逐次物化definition/response | load时编译trigger→proc bucket与prepared response，tick只遍历匹配bucket并受budget约束 |
| CONTENT-P1-021 | item/quest/mob/NPC混在kind字符串API | 每种definition有独立typed registry/view，共享identity与lifecycle，不共享无类型metric bag |
| CONTENT-P1-022 | metric API用float承载整数/枚举/flag | schema保留精确bool/int/fixed-point/enum/ID类型和range/unit，非法转换在cook时失败 |
| CONTENT-P1-023 | relation通过裸string/index跨表 | cook解析成generation-qualified typed references；missing/wrong-type/ambiguous target阻断发布 |
| CONTENT-P1-024 | 没有batch/snapshot query | 提供immutable `ContentSnapshot`、batch resolve和stable iteration；一次system tick固定一个generation |
| CONTENT-P1-025 | consumer可直接import具体artifact | package lint禁止production domain绕过ContentRegistry；仅loader/compiler adapter可见artifact modules |
| CONTENT-P1-026 | query没有复杂度和allocation contract | 每类lookup声明O(1)/O(log n)/bounded range、零或有界allocation，并用真实catalog规模验证 |

### 4.3 Install、Hot Reload、Compatibility 与 Lifetime

| ID | 差距 | 重构要求 |
|---|---|---|
| CONTENT-P1-027 | generated源码只能随package编译 | 引入versioned cooked content artifact；代码与内容可以独立构建但必须按兼容矩阵准入 |
| CONTENT-P1-028 | 没有artifact loader状态机 | 明确Requested/Reading/Decoded/Validated/Prepared/Active/Retiring/Failed状态、deadline、cancel和错误链 |
| CONTENT-P1-029 | 没有atomic multi-artifact activation | candidate BuildSet全部prepare成功后一次交换active pointer；任何节点失败保持last-good |
| CONTENT-P1-030 | 没有generation lease/fence | system/query/snapshot持lease；retire等待readers、jobs、network snapshots和save transaction完成 |
| CONTENT-P1-031 | hot reload可能跨tick观察混合内容 | world在明确barrier切换generation；一个tick/transaction只见一个ContentSnapshot |
| CONTENT-P1-032 | active gameplay跨reload无迁移策略 | activation/effect/quest/item/encounter实例按type声明PinOld/Migrate/Restart/Reject政策和receipt |
| CONTENT-P1-033 | 删除内容时旧save引用行为不明 | tombstone definition保留decode/display/migration最低合同；不可恢复时给出typed blocking report |
| CONTENT-P1-034 | server/client内容差异无协商 | handshake交换required BuildSet root和projection capability；不兼容拒绝，合法差异有显式mask |
| CONTENT-P1-035 | replay可能在当前内容下重放历史命令 | replay绑定原BuildSet或verified compatible successor；无法获取时不得宣称deterministic pass |
| CONTENT-P1-036 | 没有内容安装配额与回收 | 按BuildSet/shard记录disk/resident/prepared bytes、lease、last access和safe eviction policy |
| CONTENT-P1-037 | 没有损坏/部分安装恢复 | manifest、payload、index逐层校验；staging+journal+active pointer支持启动恢复和回滚 |
| CONTENT-P1-038 | runtime与Runtime04资产体系边界未接 | 通用artifact负责bytes/lease/IO；ContentRuntime负责schema、semantic prepare、BuildSet和domain query，禁止双缓存authority |

### 4.4 Domain Completeness 与 Semantic Validation

| ID | 差距 | 重构要求 |
|---|---|---|
| CONTENT-P1-039 | known ability与M4 definition差集无状态 | 对308项逐一输出definition/effect/handler/projection/support/test矩阵，unknown或部分支持不能默认为可用 |
| CONTENT-P1-040 | ability rank/effect引用未全局闭合 | 验证每rank的cost/cooldown/target/effects、每effect opcode handler和所有linked aura/projectile/cue |
| CONTENT-P1-041 | talent option/modifier/proc独立生成 | spec/options/allocation/modifier/proc形成一个typed graph并验证重复、orphan、cycle、mutual exclusion和point budget |
| CONTENT-P1-042 | modifier field集合靠字符串约定 | domain schema注册合法stat/global/ability/effect fields及unit/stacking/combination policy |
| CONTENT-P1-043 | proc定义缺执行预算闭包 | 每proc验证trigger、chance/PPM/ICD、target、effect、recursion guard、RNG stream和max fan-out |
| CONTENT-P1-044 | M5 item只是scalar field projection | ItemDefinition验证type、stack、bind、equip、vendor、loot、craft、serialization和presentation refs完整性 |
| CONTENT-P1-045 | quest只有局部objective scalar | QuestDefinition使用typed objective/reward/prerequisite graph并验证可达性、循环和owner handler |
| CONTENT-P1-046 | mob core与loot/spawn/ability分片未闭合 | MobDefinition必须解析stats、faction、behavior、ability set、loot、spawn/encounter和presentation refs |
| CONTENT-P1-047 | M8 encounter是固定constructor snapshot | 编译为versioned encounter/world-spawn definition，运行时实例拥有独立identity、RNG和lifecycle |
| CONTENT-P1-048 | baseline/fresh-player/offline bootstrap多投影可能互相漂移 | 统一CharacterBootstrapDefinition，按profile/class/race解析并做守恒、range和cross-field invariants |
| CONTENT-P1-049 | 78份contract把常量存在误当实现 | contract registry区分WireKnown/RuntimeHandled/ProductReachable/Qualified；只有全链满足才声明Supported |
| CONTENT-P1-050 | unreachable candidate无处置规则 | 每个候选指定Integrate/Oracle/Fixture/Retire及owner/date；不可长期留在生产source root形成暗示能力 |
| CONTENT-P1-051 | `content/m3_mobs`与generated M3 core重复 | 选择唯一source schema和compiler，旧七mob表仅作migration oracle后hard cut |
| CONTENT-P1-052 | `camp_mob_spec`较新wrapper仍断线 | 将其语义并入MobDefinition compiler/validator或删除；禁止另建runtime facade掩盖registry缺失 |

### 4.5 Performance、Safety 与 Diagnostics

| ID | 差距 | 重构要求 |
|---|---|---|
| CONTENT-P1-053 | 没有VM parse/compile/startup基线 | 记录102文件与七大表的parse、bytecode、startup、peak bytes和instruction count，建立规模曲线 |
| CONTENT-P1-054 | 热查询未按真实cardinality benchmark | 覆盖308 ability、189 modifier、55 proc、82 item和并发world consumers的p50/p95/p99/worst |
| CONTENT-P1-055 | 大文件可能放大instruction/cache成本 | 比较typed binary/columnar/indexed布局；以CPU cycles、branch miss、resident bytes和startup预算选型 |
| CONTENT-P1-056 | throw字符串进入生产artifact | cook时把schema错误前移；runtime错误使用compact code+context，详细文本驻留在diagnostic/source map |
| CONTENT-P1-057 | string重复量与临时对象未量化 | 建string interner/ID table、arena或borrowed view，记录query allocation bytes并要求热路径为0 |
| CONTENT-P1-058 | 无恶意/损坏内容输入上限 | 对文件、record、string、array、reference depth、graph edges、prepared instructions和decode time设hard limit |
| CONTENT-P1-059 | 内容可驱动行为但无trust policy | BuildSet manifest签名、trusted key/revocation、source channel与server authority明确；未认证artifact不得激活 |
| CONTENT-P1-060 | 无per-domain/content query telemetry | 记录generation、artifact、query kind、hit/miss、visited rows、duration、alloc、error及sample policy |
| CONTENT-P1-061 | 无registry/content inspector | 展示active/candidate/retiring BuildSet、digests、leases、dependencies、support状态、last validation和diff |
| CONTENT-P1-062 | 无content diff与影响分析 | semantic diff列出added/removed/changed definitions、dependent systems/save/network影响和reload policy |

### 4.6 Evidence、Migration 与跨报告边界

| ID | 差距 | 重构要求 |
|---|---|---|
| CONTENT-P1-063 | 测试expected手工固定并已过期 | expected由approved source oracle生成且独立于implementation；变更需semantic diff review而非盲目改SHA |
| CONTENT-P1-064 | 三个content test main没有manifest | generated test inventory要求每个required entry恰有一份manifest、owner、backend、artifact和oracle |
| CONTENT-P1-065 | 五个manifest binary目录缺失 | clean-build runner生成临时artifact并验证receipt；checked-in目录不是通过条件，缺required output直接fail |
| CONTENT-P1-066 | 默认check前序失败使后续静默未运行 | runner汇总独立step结果或显式blocked；末尾给executed/passed/failed/blocked/not-run计数 |
| CONTENT-P1-067 | domain报告与ContentRuntime可能重复实现 | Runtime13–17拥有业务语义；本篇只提供definition identity/install/query/validation，domain compiler由双方接口接线 |
| CONTENT-P1-068 | producer/runtime边界没有共同acceptance | Tooling05发布canonical artifact+manifest；ContentRuntime按schema/semantic/perf/trust准入并返回machine-readable receipt |

## 5. P2 完整性与维护性差距

| ID | 差距 | 改进方向 |
|---|---|---|
| CONTENT-P2-001 | generated目录按历史批次导航困难 | 生成domain/owner/artifact/dependency索引，文件布局服从runtime分区 |
| CONTENT-P2-002 | header marker格式不统一 | 统一producer ID、source root、schema、tool、BuildSet node、digest与do-not-edit marker |
| CONTENT-P2-003 | 无source map到原始定义 | 每record/field保留source path/key/span，diagnostic可定位真实source而非巨型generated行 |
| CONTENT-P2-004 | 巨型generated diff不可审 | review输出semantic summary和canonical payload diff，源码投影视为可再生产物 |
| CONTENT-P2-005 | public surface过大 | generated adapter仅公开manifest/load入口；领域查询集中到typed registry facade |
| CONTENT-P2-006 | contract/candidate命名不能表达资格 | metadata显示Production/Experimental/Oracle/Test/Retired，名称不承担状态 |
| CONTENT-P2-007 | numeric unit与范围不易发现 | schema文档生成unit、range、default、sentinel和invariant，不靠调用方猜测 |
| CONTENT-P2-008 | 无内容依赖可视化 | inspector显示definition→effect/loot/spawn/quest/talent引用图和cycle/orphan |
| CONTENT-P2-009 | 无BuildSet变更日志 | 自动生成compatible/breaking/migration-required/removed摘要与owner审批 |
| CONTENT-P2-010 | 错误缺稳定诊断码 | 统一ContentError code、artifact/key/field/generation context和operator action |
| CONTENT-P2-011 | 无内容加载进度/预算面板 | 显示read/decode/validate/prepare/activate阶段、bytes、time和blocked dependency |
| CONTENT-P2-012 | test矩阵不可浏览 | 索引显示每artifact的unit/semantic/product-root/save/replay/perf/fuzz状态与receipt |
| CONTENT-P2-013 | 参考引擎采纳理由未记录 | ADR说明吸收的identity/lifetime/cache模式及Zircon拒绝照搬的对象模型 |
| CONTENT-P2-014 | generated与hand-written责任易回流 | lint和code review gate禁止generated拥有业务分支选择、transaction或world mutation |
| CONTENT-P2-015 | catalog大小变化无趋势 | CI记录records/bytes/branches/prepared bytes/startup/query回归并按domain设预算 |
| CONTENT-P2-016 | Unity Graphics类比易被误当内容语义 | 仅借鉴versioned handle/resource lifetime与显式dependency，不引用其定义gameplay content |

## 6. 参考引擎差异

### 6.1 Unreal Engine

`UDataTable`以明确 `RowStruct` 和 `RowMap`拥有typed rows，查询要求row type匹配；DataRegistry进一步用`FDataRegistryId`、source/cache policy、acquire/release与cached item状态组织运行时数据。`UAssetManager`提供PrimaryAssetId、bundle、streamable handle与async load生命周期。Zircon应吸收的是稳定typed identity、registry source、cache/acquire、bundle与async generation owner，而不是照搬UObject反射、全局singleton或Unreal包格式。

### 6.2 Godot

Godot `Resource`、`ResourceUID` 与 `ResourceLoader`把资源对象、稳定UID/path、loader/cache/reload状态分开。它说明内容引用不能只靠临时数组index，reload也必须通过统一cache/loader和identity生命周期。Zircon仍应保留强类型schema、不可变snapshot和服务端BuildSet准入，不采用动态Variant作为核心内容合同。

### 6.3 Bevy 与 Fyrox

Bevy `Assets<T>`按AssetId保存typed asset，`Handle<T>`区分strong/weak生命周期，AssetServer可查询load state；Fyrox `ResourceManager`按path/type共享异步request，resource显式经历Pending/LoadError/Ok。两者共同证明“生成了文件”与“已加载可用”之间需要typed handle、状态、共享和错误语义。Zircon还需在其上增加跨artifact ContentBuildSet、server/client协商和domain semantic compiler。

### 6.4 Unity Graphics

Unity Graphics `RenderGraphResourceRegistry`维护handle version、write count、import/create/release与有效性。它只提供结构类比：ContentHandle也必须绑定generation，record/prepare/activate/retire阶段要显式，旧handle不能命中新generation。RenderGraph不定义ability、item、quest或MMO内容语义，不能作为这些domain的正确性参考。

## 7. 目标 Owner 与边界

| Owner | 唯一职责 | 禁止承担 | 前置依赖 |
|---|---|---|---|
| `ContentSchemaRegistry` | typed definition/field/reference/schema version与compatibility | generator scheduling、live world mutation | Tooling04/05、domain owners |
| `ContentBuildSetRegistry` | aggregate manifest/root、profile/shard集合、source/tool/target provenance | bytes IO、业务规则 | Tooling05、App03 product identity |
| `ContentArtifactLoader` | async read/decode/limit/checksum/signature状态机 | semantic gameplay execution | Runtime04 asset/artifact |
| `ContentValidationRuntime` | schema、reference closure、domain invariant、support/capability validation | 第二套domain transaction | SchemaRegistry、Runtime13–17 compilers |
| `ContentRegistry` | candidate/active/retiring generation与typed handles/snapshots | source authoring、generator discovery | BuildSet/Loader/Validation |
| `ContentQueryRuntime` | indexed immutable lookup、batch view、budget与telemetry | rule mutation、world scanning | ContentRegistry |
| `ContentCompatibilityRegistry` | save/network/replay/hot-reload compatibility与migration policy | state decode、transport | Runtime08E/12、ContentRegistry |
| `ContentActivationCoordinator` | prepare/barrier/atomic switch/lease/retire/rollback | artifact generation、domain semantics | App03 transaction、all content owners |
| `ContentReferenceResolver` | typed cross-artifact reference resolve、alias/tombstone/source map | soft fail-open | Schema/Registry |
| `ContentProjectionRuntime` | recipient/operator-safe identity、diff、diagnostic projection | authority mutation | ContentRegistry、Runtime08E |
| `ContentEvidenceRunner` | product-root semantic/save/replay/fault/load/perf与receipt | 第二套expected规则 | Tooling05/10、domain oracle |

Tooling05必须先交付完整、唯一、原子发布的artifact graph；Runtime04只提供通用bytes/residency；App03提供package/host外层transaction；Runtime12提供world/save/replay identity；Runtime13–17提供业务semantic compilers和transactions。本篇在这些owner之间建立唯一内容代际，不允许各domain再直接管理一套generated module cache或BuildSet。

## 8. 重构里程碑

### M0 · Topology 与 current truth freeze

- 固定104个非测试文件的producer、分类、consumer、digest和support inventory；
- 将58个不可达候选标为NotQualified并指定迁移/fixture/retire owner；
- 保存当前M4/M5 stale test矛盾与真实产品查询trace，防止迁移时误改oracle。

### M1 · Schema、BuildSet 与 Artifact Contract

- 定义ContentSchema/Artifact/BuildSet/Profile/Shard/Generation/Handle；
- Tooling05输出canonical manifest、payload、source map和aggregate root；
- state/network/replay/crash identity携BuildSet并执行compatibility admission。

### M2 · Loader、Registry 与 Immutable Query

- 接Runtime04异步artifact loader，实施decode limit、signature与last-good；
- 建candidate/active/retiring registry、snapshot lease和typed cross-reference；
- 将string/if ladder编译为immutable table/index/prepared views。

### M3 · Domain Semantic Compilation

- Ability/Effect、Talent/Proc、Item/Quest、Mob/Loot/Spawn、Bootstrap/Encounter分别编译；
- 对全部definition输出Implemented/Unsupported/owner/handler/test状态；
- cross-catalog missing、orphan、cycle、type/range/unit错误在activation前失败。

### M4 · Product Migration 与 Hard Cut

- WorldState及domain模块只消费ContentSnapshot/typed handle；
- 删除直接generated import、旧七mob表和平行runtime authority；
- static package仅保留bootstrap loader/thin schema adapter，不再承载81K行内容控制流。

### M5 · Reload、Compatibility 与 Recovery

- 在world barrier执行candidate prepare/atomic activate/retire；
- 实施active instance Pin/Migrate/Restart/Reject policy；
- 验证save/replay/network差异、损坏安装、崩溃恢复和rollback。

### M6 · Qualification 与规模门

- 修复test inventory、stale expected、missing artifact和blocked result汇总；
- 用真实catalog规模跑product-root semantic、fault、fuzz、save/replay和cross-generation矩阵；
- 锁定startup/resident/query/instruction/allocation预算，回归超限阻断发布。

## 9. Runtime 资格门

| Gate | 必须满足 |
|---|---|
| CONTENT-G01 · Topology | 104个非测试文件均有唯一producer/classification/owner/consumer/evidence；unclassified=0 |
| CONTENT-G02 · BuildSet | active product有唯一aggregate root；全部required artifact/schema/tool/source/target digest可追溯 |
| CONTENT-G03 · Admission | unknown、missing、duplicate、cross-generation或未签名required artifact均fail-closed且保持last-good |
| CONTENT-G04 · Identity | 所有ContentHandle带type/key/generation；retired/reused index不会命中新内容 |
| CONTENT-G05 · Closure | ability/talent/item/quest/mob/loot/spawn/encounter跨表引用missing/wrong-type/orphan/cycle为0 |
| CONTENT-G06 · Capability | 每个公开definition有Implemented/Unsupported等显式状态、runtime owner与required evidence |
| CONTENT-G07 · Query | 热路径不直接解释巨型string/field/rank `if` ladder；真实规模lookup满足复杂度与零分配预算 |
| CONTENT-G08 · Snapshot | 一个tick/transaction固定单一ContentSnapshot；混合generation观察为0 |
| CONTENT-G09 · Activation | multi-artifact candidate全量prepare后原子激活；任意失败、取消或崩溃可确定回滚 |
| CONTENT-G10 · Lifetime | reader/job/snapshot/save/network lease未释放前旧generation不回收；stale access返回typed error |
| CONTENT-G11 · Compatibility | save/network/replay/hot reload均协商BuildSet；breaking差异无明确migration时拒绝 |
| CONTENT-G12 · Semantics | domain全catalog invariants与support matrix通过；308/117等差集每项有解释，不存在隐式支持 |
| CONTENT-G13 · Evidence | M4/M5 stale expected已消除；所有required entry实际executed，missing/blocked/not-run/skip不计pass |
| CONTENT-G14 · Performance | 记录startup、bytecode/prepared/resident bytes、p95/p99/worst query、alloc和scale slope并满足预算 |
| CONTENT-G15 · Diagnostics | receipt含source/tool/BuildSet/backend/hardware、first error/diff；inspector可见active/candidate/leases/deps |
| CONTENT-G16 · Documentation | `git diff --check`、frontmatter路径、Markdown链接、severity/ID/owner/index/coverage统计全部通过 |

## 10. 状态与边界

| 项目 | 状态 |
|---|---|
| `src/content` 与 `src/generated` 物理inventory | complete；112文件逐文件分类并统计 |
| product static closure与断线集合 | complete；generated 46可达、55个非测试不可达，content三个非测试均不可达 |
| catalog/WorldState/query/provenance纵向复核 | complete；当前source可复核 |
| Unreal/Godot/Bevy/Fyrox/Unity Graphics参考联读 | complete；按identity/registry/lifetime边界路由 |
| production代码、tests、manifest、generated artifact修改 | pending；本轮未执行 |
| native/npm动态资格 | blocked by existing failures；本轮未重复无变化lane |

本篇不把81K行generated源码判定为全部无价值。catalog数据、source commit、局部digest和大量规则映射都是迁移输入；需要删除的是“数据必须成为巨型ZrVM控制流、consumer必须直接import具体文件、局部SHA即可代表产品代际”的临时架构。迁移必须先冻结semantic oracle，再建立BuildSet/registry/prepared query，最后hard cut旧authority。

Runtime04、12–17、App03、Tooling05/10与Editor24/40继续保留各自canonical owner。本篇只负责WOC runtime内容定义的共同identity、安装代际、查询、兼容、激活和资格；不接管通用asset IO、生成器调度、业务transaction或编辑器authoring。任何后续实现都应以上述16项runtime资格门为关闭条件，而不是以“generator成功”“文件被import”或“单个contractTest返回0”代替工程级交付。
