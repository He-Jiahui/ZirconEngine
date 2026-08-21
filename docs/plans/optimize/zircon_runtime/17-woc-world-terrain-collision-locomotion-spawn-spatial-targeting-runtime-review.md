---
related_code:
  - examples/woc/scripts/woc_game/src/main.zr
  - examples/woc/scripts/woc_game/src/world
  - examples/woc/scripts/woc_game/src/world/state.zr
  - examples/woc/scripts/woc_game/src/world/collision_content.zr
  - examples/woc/scripts/woc_game/src/world/collision_geometry.zr
  - examples/woc/scripts/woc_game/src/world/collision_grid.zr
  - examples/woc/scripts/woc_game/src/world/collision_static.zr
  - examples/woc/scripts/woc_game/src/world/collision_sweep.zr
  - examples/woc/scripts/woc_game/src/world/world_collision_router.zr
  - examples/woc/scripts/woc_game/src/world/safe_position.zr
  - examples/woc/scripts/woc_game/src/world/decoration_candidate.zr
  - examples/woc/scripts/woc_game/src/world/terrain_content.zr
  - examples/woc/scripts/woc_game/src/world/terrain_ground.zr
  - examples/woc/scripts/woc_game/src/world/terrain_gradient.zr
  - examples/woc/scripts/woc_game/src/world/terrain_height.zr
  - examples/woc/scripts/woc_game/src/world/terrain_mountains.zr
  - examples/woc/scripts/woc_game/src/world/terrain_noise.zr
  - examples/woc/scripts/woc_game/src/world/terrain_shape.zr
  - examples/woc/scripts/woc_game/src/world/terrain_sowfield.zr
  - examples/woc/scripts/woc_game/src/world/player_motion.zr
  - examples/woc/scripts/woc_game/src/world/player_motion_world.zr
  - examples/woc/scripts/woc_game/src/world/player_motion_vertical_world.zr
  - examples/woc/scripts/woc_game/src/world/player_motion_transition.zr
  - examples/woc/scripts/woc_game/src/world/player_motion_effects.zr
  - examples/woc/scripts/woc_game/src/world/player_motion_wall_standoff.zr
  - examples/woc/scripts/woc_game/src/world/mob_motion_world.zr
  - examples/woc/scripts/woc_game/src/world/pathfind_state.zr
  - examples/woc/scripts/woc_game/src/world/spatial_grid_state.zr
  - examples/woc/scripts/woc_game/src/world/target_selection.zr
  - examples/woc/scripts/woc_game/src/world/dead_target_selection.zr
  - examples/woc/scripts/woc_game/src/world/interaction_selection.zr
  - examples/woc/scripts/woc_game/src/world/mob_idle_aggro_state.zr
  - examples/woc/scripts/woc_game/src/world/fleeing_social_aggro_state.zr
  - examples/woc/scripts/woc_game/src/world/mob_lifecycle_state.zr
  - examples/woc/scripts/woc_game/src/world/mob_scan_counters_state.zr
  - examples/woc/scripts/woc_game/src/world/bootstrap_roster.zr
  - examples/woc/scripts/woc_game/src/world/camp_spawn_layout.zr
  - examples/woc/scripts/woc_game/src/world/camp_spawn_placement.zr
  - examples/woc/scripts/woc_game/src/world/npc_placement.zr
  - examples/woc/scripts/woc_game/src/world/ground_object_placement.zr
  - examples/woc/scripts/woc_game/src/world/mailbox_placement.zr
  - examples/woc/scripts/woc_game/src/world/reserved_npc_placement.zr
  - examples/woc/scripts/woc_game/src/world/spirit_healer_placement.zr
  - examples/woc/scripts/woc_game/src/world/corpse_loot_rights_state.zr
  - examples/woc/scripts/woc_game/src/world/world_boss_participation_state.zr
  - examples/woc/scripts/woc_game/src/world/projectile_travel_state.zr
tests:
  - examples/woc/scripts/woc_game
  - examples/woc/reference/parity_scenarios.json
  - examples/woc/reference/current-head/parity_scenarios.json
  - examples/woc/native/Cargo.toml
  - examples/woc/tools/package.json
plan_sources:
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/08a-physics-runtime-review.md
  - docs/plans/optimize/zircon_runtime/08d-navigation-runtime-review.md
  - docs/plans/optimize/zircon_runtime/08f-ai-behavior-tree-blackboard-perception-runtime-review.md
  - docs/plans/optimize/zircon_runtime/12-woc-zrvm-package-kernel-world-state-schedule-serialization-runtime-review.md
  - docs/plans/optimize/zircon_runtime/13-woc-combat-casting-effect-aura-damage-threat-death-runtime-review.md
  - docs/plans/optimize/zircon_runtime/16-woc-instance-dungeon-delve-pet-companion-lockout-reset-collision-runtime-review.md
  - docs/plans/optimize/zircon_editor/28-spawn-rules-encounter-population-world-state-scenario-quest-flag-authority-simulation-authoring-review.md
  - docs/plans/optimize/zircon_editor/40-procedural-content-generation-rule-graph-biome-world-generation-authoring-review.md
  - docs/plans/optimize/zircon_tooling/05-woc-content-codegen-build-scripts-generated-artifact-incremental-review.md
  - docs/plans/optimize/zircon_tooling/10-test-architecture-partition-selection-isolation-fixture-flake-results-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/World.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/CollisionQueryParams.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/WorldCollision.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/WorldPartition/WorldPartition.h
  - dev/UnrealEngine/Engine/Source/Runtime/NavigationSystem/Public/NavigationSystem.h
  - dev/UnrealEngine/Engine/Source/Runtime/NavigationSystem/Public/NavMesh/RecastNavMesh.h
  - dev/godot/servers/physics_3d/physics_server_3d.h
  - dev/godot/scene/resources/3d/world_3d.h
  - dev/godot/scene/3d/navigation/navigation_agent_3d.h
  - dev/bevy/crates/bevy_ecs/src/world/mod.rs
  - dev/bevy/crates/bevy_ecs/src/entity/mod.rs
  - dev/bevy/crates/bevy_transform/src/systems.rs
  - dev/Fyrox/fyrox-impl/src/scene/graph/mod.rs
  - dev/Fyrox/fyrox-impl/src/scene/navmesh.rs
  - dev/Fyrox/fyrox-core/src/pool/mod.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraph.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/IRenderGraphBuilder.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 17 · WOC World、Terrain、Collision、Locomotion、Spawn、Spatial 与 Targeting Runtime 工程化差距

## 1. 结论

WOC `src/world` 不是小型示例目录，而是一套体量已经足以掩盖执行真相的标量世界投影：208 个 `.zr`、102,546 行、4,245,497 bytes，其中 67 个非 test-main 文件占 100,755 行。Runtime12 已拥有 68,730 行 `WorldState`、schedule 与 codec，Runtime16 已拥有 17 个 instance/dungeon/delve world helper；剔除这些已审边界后，本篇仍逐文件核对 49 个模块、15,575 行、455,226 bytes。该集合有 30 个 class、179 个 public var、397 个 public function、274 个 `throw`、61 次显式 Array 构造、103 个 `while` 与 2,916 个 `if`，不是可以继续用“demo helper”免责的规模。

产品执行面与物理文件面仍然分裂。49 个模块中，35 个、13,080 行进入 `main.zr` 静态 import closure；14 个、2,495 行只由自己的 test main、其他断线 candidate 或根本无人消费。不可达集合恰好包含 `pathfind_state`、`spatial_grid_state`、`interaction_selection`、`corpse_loot_rights_state`、`world_boss_participation_state`、`bootstrap_roster`、camp/ground-object/mailbox/reserved-NPC/spirit-healer placement 与 scan counter。产品因此并没有因为这些文件存在而获得 navigation、spatial query、atomic spawn、interaction、corpse rights 或 world-boss participation owner。

最严重的产品风险是碰撞复杂度。`world_collision_router` 将移动按 0.2 world unit 切段；每个段内部先求 X、再求 Z。开放世界每个坐标求解最多三遍扫描 170 个 collider，因此一次“返回某个 axis”的移动查询内部最坏约 1,020 次 collider 检查；调用方为了拿 X/Z 又分别调用整条 pair-valued resolver，最坏约 2,040 次/采样段，还未计入 4,210 行 `collision_content` 的 1,389 个 accessor 分支和六段 fence。名为 `collision_grid` 的 16-yard grid 仍在每次查询中扫描全部 170 个静态 collider，只做 cell predicate 过滤，而且只被 safe-position/placement 路径使用；player/mob 主移动继续走全表 `collision_static`。

AI 与移动同样没有形成工程闭环。`WorldState` 的 idle aggro、social aggro、target candidate 和 pet/mob 辅助查询按实体数组全表扫描；`target_selection` 每个命令新建六列 candidate，并用四列临时数组执行 insertion sort。已经存在的 `SpatialGridState` 没有产品消费者。`PathfindState` 也只有测试消费者：每次查询重新建立局部 1-yard 网格和多列数组，默认 span 上限 64；宽域、同 cell 或无路时返回直线目标，destination search 又以 1..24 ring 和每 ring 十个样本重复 terrain/collision。产品 mob/pet 实际只用 direct step 与七向 slide fan，无法声称 navigation-qualified locomotion。

地形与世界构建则仍是单一 hardcoded world 的即时函数。`terrain_content` 3,178 行、82 个 public accessor、808 个 `if`、111 个 `throw`；height/gradient/ground/shape/noise 每次查询递归组合标量函数，没有 WorldDefinition、tile/chunk、cook artifact、partition、streaming source、residency、LOD、编辑 revision 或 invalidation owner。spawn 候选文件的头注释已明确说明它们只是 scalar projection，未来 materializer/entity/RNG owner 在外部；但那个 atomic materializer 并不存在，`WorldState` 继续直接拥有另一套 constructor/array mutation。

证据链不能为该 world runtime 资格背书。world 目录 141 个 test main 中五个没有任何 `.zrp` 接管；以 world entry 为根的 74 个 package 中，64 个声明的 binary directory 不存在。四组 lifecycle/locomotion/roster/targeting trace dump 有历史 artifact，而对应 trace-test package 的 artifact 均不存在；current-head parity 的 entity roster、mob targeting、mob locomotion、mob lifecycle、targeting markers 等六个 `woc_owner` 路径也不存在。67 个非 test-main 文件只有 `terrain_noise.zr` 带 40 字符 source commit，且同一文件并列两个不同 revision，其余 66 个没有同等 provenance。这里保存的是可迁移规则和历史 oracle，不是已通过产品根、规模与性能门的世界系统。

## 2. 审查范围与执行拓扑

### 2.1 物理清单与去重边界

| 项目 | 结果 |
|---|---:|
| `src/world` 全部 `.zr` | 208 文件 / 102,546 行 / 4,245,497 bytes |
| 非 test-main | 67 文件 / 100,755 行 |
| test main | 141 文件 / 1,791 行 |
| Runtime12 已拥有的 `state.zr` | 1 文件 / 68,730 行 / 3,298,740 bytes |
| Runtime16 已拥有的 instance/dungeon/delve world helper | 17 文件 / 16,450 行 / 428,372 bytes |
| 本篇非重复主体 | 49 文件 / 15,575 行 / 455,226 bytes |
| 本篇 main 静态闭包可达 | 35 文件 / 13,080 行 |
| 本篇不可达 | 14 文件 / 2,495 行 |
| class / public var / public function | 30 / 179 / 397 |
| `throw` / Array 构造 / `while` / `if` | 274 / 61 / 103 / 2,916 |

本篇不重新拥有通用 Scene/ECS、Physics 或 Navigation 实现。Runtime05负责 world/entity generation 与 partition 基础；Runtime08A负责 shape/broadphase/query/contact 的通用物理后端；Runtime08D负责 nav artifact/query/crowd；Runtime08F负责 AI/perception。这里审查的是 WOC 如何消费这些能力，以及现有标量 helper 为什么不能替代它们。

### 2.2 产品可达集合

可达 35 个模块集中于三条路径：terrain/ground/gradient，player/mob motion，static collision/targeting/lifecycle。`main.zr` 的普通产品段对 `player_motion_world` 有 3 个调用点、`target_selection` 6 个、`npc_placement` 6 个、mob idle/social aggro 各 4 个、mob lifecycle 50 个、projectile travel 86 个；这些调用证明局部规则确实运行，也证明热路径不是纯测试代码。

不可达 14 个模块为 `bootstrap_roster`、`camp_spawn_layout`、`camp_spawn_placement`、`corpse_loot_rights_state`、`geometry2d`、`ground_object_placement`、`interaction_selection`、`mailbox_placement`、`mob_scan_counters_state`、`pathfind_state`、`reserved_npc_placement`、`spatial_grid_state`、`spirit_healer_placement` 与 `world_boss_participation_state`。它们不能计入 supported capability；迁移前只能标为 candidate/oracle/fixture。

### 2.3 Collision 与 terrain 热路径

`collision_content` 用巨型 `if` ladder 把 170 个 collider 与六段 fence 暴露为逐字段 accessor。`collision_static` 每次 query 最多三遍遍历所有 collider；`world_collision_router` 再按 0.2 步长做 sweep-like sampling，但返回值仍用 `axis` 模拟 pair，因此 callers 为 X/Z 重跑全部计算。它既不是 broadphase+narrowphase，也不返回 hit fraction、normal、penetration、material、shape/entity handle、query filter 或 trace identity。

`collision_grid` 的名称容易造成误判。它求出 cell 后仍执行 `while index < colliderCount`，只是用 `touchesGridCell` 排除；decoration candidate 则在查询时按 seed 重算。它不保存 cell→collider handle 列表，不具 generation/invalidation，也没有进入 player/mob 主路由。`safe_position` 消费它并以 spiral/projection 求出生点，这属于构建辅助，不等于 runtime broadphase。

terrain 也没有可安装资源。`terrain_content` 和 `terrain_shape` 是生成式常量表；ground/height/gradient 在每次 movement、spawn、path query 中重复求值。没有 source→cooked tile、heightfield/mesh collision、nav tile、render LOD 和 gameplay region 的共同 revision，因而无法证明 client render、server collision 与 navigation 使用同一世界代。

### 2.4 Locomotion、navigation、spatial 与 targeting

player motion 把输入、速度、坡度、水面、wall standoff 与 scalar collision 组合成 coordinate function；没有 character-controller state、shape cast result、step height、floor contact/manifold、moving platform、root motion、network prediction/reconciliation 或 physics generation。mob motion 每次前进尝试七个 slide angle，每个 candidate 又重复 terrain 与 pair collision；实体规模扩大时，计算量与 collider count、移动距离和 candidate fan 相乘。

`PathfindState` 的局部 A* 构造 provisional/actual search，多次分配 width×height 的六列数组，walkability 又调用昂贵 terrain/collision；segment smoothing 按 0.25 采样。超过 64-cell span、same-cell 与 unreachable 的结果会退回直线 target，而不是 typed failure/partial route。更关键的是产品没有 import 它，mob/pet locomotion无法消费 path generation、nav revision 或 corridor validity。

`SpatialGridState` 能按 cell 保存 `SpatialEntity` 并 radius query，但只有自己的 test main import。产品 aggro/target/pet/interaction仍自行扫描 `WorldState.entityIds`，不同系统各自重建候选。`target_selection` 只负责排序，relationship/faction/visibility/LOS 由 caller 临时投影；tab selection 用 insertion sort，friendly cycle另建两列临时数组。这里缺少唯一 spatial snapshot、query filter、stable order、visibility revision与budget。

### 2.5 Spawn、interaction、corpse 与 world encounter

`bootstrap_roster` 明写“future atomic WorldState materializer”；camp placement 明写“scalar projection”；ground object、mailbox、reserved NPC、spirit healer均只返回常量或最终坐标。它们之间可以重放源顺序，却不拥有 SpawnId、definition revision、entity allocation、transaction、despawn、respawn、streaming cell、persistence或rollback。产品继续由 `WorldState` 另一组数组与构造流程 materialize，形成双 authority。

`interaction_selection`、`corpse_loot_rights_state` 和 `world_boss_participation_state` 都是断线 candidate。前者没有产品 relationship/spatial/LOS consumer；corpse rights 用平行数组保存 party/tap/FFA timer；world boss state用本地 participant/threat/damager数组和固定 lockout事实，并在清空时反复 `removeAt(0)`。它们没有与 Runtime13 death/threat、Runtime14 loot transaction、Runtime15 principal/party、Runtime16 instance qualification形成 typed transaction。

### 2.6 测试、artifact 与 provenance

141 个 world test main 有 136 个名字出现在某个 package manifest，五个未接管项是 `interaction_selection_test_main` 与 M3 lifecycle/locomotion/roster/targeting scenario main。另一方面，74 个以 world module 为 entry 的 `.zrp` package 只有十个声明 binary directory 实际存在，64 个缺失；“有 manifest”不能替代 required executable artifact。

现存 lifecycle、locomotion、roster、targeting trace dump artifact各有 4-5 个文件，但对应 trace-test artifact目录全部不存在。current-head parity catalog引用的六个相关 `scripts/woc_game/tests/parity/*.zr` owner也全部缺失。静态 `contractTest()` 被 49 模块提及 50 次，但它们多数只断言标量样本，不能覆盖 product root、规模曲线、stale generation、save/load、fault、deterministic replay或跨后端语义。

## 3. P0 阻断

| ID | 差距 | 证据与影响 | 必须重构 |
|---|---|---|---|
| WORLD-P0-001 | 单一 hardcoded scalar world 没有 definition、partition、streaming 与 generation owner | 7,388 行 terrain/collision generated table即时求值；无 tile/chunk/artifact/residency/revision，server collision、nav与render无法证明同代 | 建 `WorldDefinitionRegistry`、`WorldPartitionRuntime` 与 generation-qualified installed world；source/cook/install/retire全链同一 BuildSet |
| WORLD-P0-002 | 主碰撞/移动热路径按距离×轴×三遍×170全表扫描 | 一次 axis-return movement query内部约1,020检查，调用方取X/Z时最坏约2,040/0.2段；grid没有接主路由 | 接 Runtime08A 的 cooked shape+broadphase+continuous sweep，query一次返回typed pair/hit；以斜率和p99硬门替代样本断言 |
| WORLD-P0-003 | 产品 aggro、target、pet/mob query 全表扫描，SpatialGrid断线 | `SpatialGridState`只有test consumer；候选构建、relationship、排序重复分配，规模趋向O(E²) | 建 world-scoped `SpatialQueryRuntime`，mutation增量维护index，所有AI/target/interaction共享snapshot/filter/budget |
| WORLD-P0-004 | Navigation candidate断线且fail-open直线路径 | `PathfindState`无产品consumer；宽域/无路返回直线，mob/pet只direct step/slide，可能穿越不可达域 | 由Runtime08D提供versioned nav artifact、typed failure/partial corridor、async request/cancel与movement corridor follower；禁止silent straight fallback |
| WORLD-P0-005 | Spawn、interaction、corpse rights与world boss存在断线双authority | 14个不可达模块含future materializer和三套state；产品WorldState另行构造/突变，identity、transaction、despawn/respawn均未定义 | 建唯一Spawn/Interaction/WorldEncounter owner，与entity generation、combat、loot、party、instance事务接线后hard cut候选平行状态 |
| WORLD-P0-006 | Package/parity/provenance不能证明world capability | 5 test main无manifest、74 world packages有64个binary dir缺失、六个current owner缺失、66/67生产文件无source commit | 生成production/fixture/evidence topology；从产品root执行exact replay/save-load/fault/load/perf，artifact/digest/source/backend缺一不得计pass |

## 4. P1 工程化差距

### 4.1 Topology、WorldDefinition 与 BuildSet

| ID | 差距 | 重构要求 |
|---|---|---|
| WORLD-P1-001 | `src/world`把production、candidate、oracle、test main混放 | 生成module-role manifest和独立entry closure；normal tick、self-test、trace、fixture不可互相冒充 |
| WORLD-P1-002 | 49个主体中14个断线仍位于生产source root | 接入唯一owner后删除旧路径，或迁到明确oracle/fixture package并从capability排除 |
| WORLD-P1-003 | World identity只有隐含的单例与seed | 定义WorldDefinitionId、WorldInstanceId、WorldGeneration、BuildSet、content revision与qualified entity handle |
| WORLD-P1-004 | terrain/collision/spawn/nav没有统一definition revision | `WorldDefinitionArtifact`列出所有子artifact digest、坐标系、scale、bounds、dependency与schema |
| WORLD-P1-005 | `required: bool`参数被用作标量helper的伪合同 | 用typed required dependency/install handle与明确Missing/Unsupported/Stale错误替换运行时布尔哨兵 |
| WORLD-P1-006 | 大量free function绕过world context | query API必须接收generation-qualified world/query context，禁止仅凭seed+coordinate访问全局内容 |
| WORLD-P1-007 | 规则常量、generated table与源revision没有同代关系 | codegen输出source digest、generator/compiler version、target、dependency digest和payload digest |
| WORLD-P1-008 | `terrain_noise`同时声明两个source commit | 每个artifact固定唯一source revision；合并语义必须有显式merge record和golden diff |
| WORLD-P1-009 | 其余66个非test-main文件没有等价provenance | provenance由artifact manifest生成，不再依赖零散源码注释 |
| WORLD-P1-010 | WorldState可继续无界吸收地形/移动/spawn特例 | 只保留owner handle与事件/transaction adapter；禁止增加新的平行列作为永久实现 |

### 4.2 Terrain、partition、streaming 与 world content

| ID | 差距 | 重构要求 |
|---|---|---|
| WORLD-P1-011 | `terrain_content` 3,178行accessor ladder是唯一内容接口 | 编译为typed terrain/region definition artifact，runtime按handle和chunk读取 |
| WORLD-P1-012 | terrain shape/height/ground/gradient重复即时求值 | 生成共享heightfield/mesh/derivative数据或有代际cache，并为CPU/GPU精度定义误差合同 |
| WORLD-P1-013 | 无world bounds、origin、coordinate-space元数据 | 定义坐标系、unit、origin rebasing/large-world策略、bounds和out-of-world typed result |
| WORLD-P1-014 | 无partition cell与streaming source | partition artifact拥有cell bounds/dependency；玩家、server、camera等source产生load/retire demand |
| WORLD-P1-015 | 无cell lifecycle与异步状态 | 定义Unloaded/Requested/Loading/Installed/Active/Retiring/Failed及cancel/retry/last-good |
| WORLD-P1-016 | terrain、collision、nav、render chunk可独立漂移 | 同一cell activation transaction验证所有required subartifact generation后一次发布 |
| WORLD-P1-017 | decoration每次collision query按seed重算 | cook deterministic instance set与spatial index；seed只用于source generation，不进入每帧重复构建 |
| WORLD-P1-018 | 无LOD、residency与memory budget | 分别定义server simulation、client render、collision/nav residency tier与硬预算 |
| WORLD-P1-019 | 无world content hot-reload invalidation | revision change生成affected-cell dependency frontier，staging重建后原子切代并retire旧查询 |
| WORLD-P1-020 | 水面、道路、山体、sowfield等语义嵌在函数分支 | 输出typed region/surface/material/tag data，供movement/nav/audio/VFX共享而非重复猜测 |
| WORLD-P1-021 | 无authoring→cook→runtime roundtrip证据 | 与Editor28/40和Tooling05共享source schema、preview compiler、cook artifact及semantic diff corpus |

### 4.3 Collision、query 与 physics integration

| ID | 差距 | 重构要求 |
|---|---|---|
| WORLD-P1-022 | `collision_content` 170项靠分支accessor暴露 | cook contiguous SoA/BVH或physics shape batch，ID稳定且可直接批量安装 |
| WORLD-P1-023 | `collision_static`每query最多三遍全扫描 | 用Runtime08A broadphase candidate set+narrowphase solver，复杂度按局部候选而非全世界shape数增长 |
| WORLD-P1-024 | 0.2 fixed sampling伪装continuous collision | 使用shape sweep/TOI，定义initial overlap、penetration、skin width、iteration和tunneling合同 |
| WORLD-P1-025 | pair-valued结果经`axis`重复整条计算 | 返回`ResolvedMotion`/`HitResult`一次携position、normal、fraction、blocking handle和reason |
| WORLD-P1-026 | 六段fence另走特殊扫描 | fence也cook为普通shape/filter/material；特殊policy只留在definition层 |
| WORLD-P1-027 | `collision_grid`仍遍历170项 | cell直接保存generation-qualified shape handles；query只访问相交cell并去重 |
| WORLD-P1-028 | grid未接player/mob主移动 | 所有movement、spawn、LOS、interaction通过同一`WorldQueryContext`，禁止另选静态resolver |
| WORLD-P1-029 | decoration候选无installed lifetime | generated instance拥有stable instance ID、cell owner、install generation与retirement fence |
| WORLD-P1-030 | query无layer/channel/filter | 定义static/dynamic/character/projectile/trigger/interaction layer与response matrix，支持ignore owner/set |
| WORLD-P1-031 | query只返回坐标，无命中语义 | 返回shape/entity/material/surface、normal、distance、overlap depth、face/subshape和trace tag |
| WORLD-P1-032 | static、instance、Delve按坐标范围路由 | query context必须显式携World/Instance handle；坐标不再决定authority或layout |
| WORLD-P1-033 | collision异常多为throw且无诊断上下文 | typed error携world generation、query id、shape revision、input、budget与fallback policy |
| WORLD-P1-034 | 无collision correctness/performance corpus | 覆盖高速/角落/薄墙/初始穿透/斜坡/大半径/跨cell/切代，并记录p50/p99与候选斜率 |

### 4.4 Locomotion、navigation 与 movement authority

| ID | 差距 | 重构要求 |
|---|---|---|
| WORLD-P1-035 | player motion是多层scalar coordinate projection | 建world-scoped CharacterMovement owner，输入intent一次产生完整movement result和state transition |
| WORLD-P1-036 | 无稳定floor/contact状态 | 保存ground handle/normal/distance、slope、step、water、moving-base generation并在切代时失效 |
| WORLD-P1-037 | 无step-up/step-down与可配置capsule policy | agent definition携shape、step height、slope、skin、snap、air control和surface response |
| WORLD-P1-038 | wall standoff与collision correction分散 | 并入统一sweep/slide solver，保留typed contact reason而非额外坐标修补 |
| WORLD-P1-039 | mob七向slide fan重复昂贵query | corridor follower基于nav steering和局部avoidance；collision只处理短程物理约束 |
| WORLD-P1-040 | `PathfindState`每query重建局部网格 | 消费Runtime08D cooked/tiled nav data和共享query pool，不在脚本层按请求重栅格化世界 |
| WORLD-P1-041 | provisional/actual search重复分配六列数组 | request scratch来自有界arena/pool，提供峰值、reuse、cancel与deadline telemetry |
| WORLD-P1-042 | 64-cell span上限是裸硬门 | agent/query profile定义max nodes/cost/time/partial policy，超限返回typed budget result |
| WORLD-P1-043 | unreachable/too-wide退回直线 | fail-closed为NoPath/Partial/BudgetExceeded/StaleNav；调用方决定停下、重试或重规划 |
| WORLD-P1-044 | route不携nav generation/corridor identity | path handle绑定world/nav/agent revision、start/goal、cost filter、expiry和invalidated tiles |
| WORLD-P1-045 | 无async path request与取消/重规划 | owner提供request id、priority、deadline、cancel、supersede、result queue和backpressure |
| WORLD-P1-046 | 无client prediction/server reconciliation合同 | movement command、authoritative frame、ack、correction、teleport、root motion和replay必须同一schema |

### 4.5 Spatial、targeting、aggro 与 interaction query

| ID | 差距 | 重构要求 |
|---|---|---|
| WORLD-P1-047 | `SpatialGridState`未进入产品owner | 建`SpatialQueryRuntime`并由spawn/move/despawn增量维护，不允许调用方自建副本 |
| WORLD-P1-048 | grid entity/where/cell使用对象与平行数组 | stable entity generation、dense handle table、cell buckets与swap-remove索引保持一致性 |
| WORLD-P1-049 | radius query每次清空并重填output数组 | 支持caller scratch/iterator/bounded result与early-out，记录visited cells/candidates/truncation |
| WORLD-P1-050 | aggro/social/pet/target各自全entity扫描 | 全部改用共享snapshot上的typed filter query，并为每系统设置候选/时间预算 |
| WORLD-P1-051 | relationship/faction在target caller临时投影 | Runtime15 principal/team/party与Runtime13 threat提供versioned relationship view |
| WORLD-P1-052 | targeting没有统一LOS/visibility | query chain明确spatial→relationship→alive/phase→LOS→priority，携revision和拒绝reason |
| WORLD-P1-053 | tab target对候选执行O(n²) insertion sort | 使用bounded top-k/selection与稳定tie-break，避免为单个结果排序完整集合 |
| WORLD-P1-054 | friendly cycle再分配两列并排序 | 维护stable ordered view或使用有界selection；明确current missing/stale时语义 |
| WORLD-P1-055 | interaction candidate与target candidate双轨且前者断线 | 共享query/filter基础，但保持combat target与interaction policy为独立typed owner |
| WORLD-P1-056 | 无spatial/AI规模与staleness测试 | 覆盖1/100/10k/100k entity、跨cell移动、despawn/reuse、并发snapshot和预算截断 |

### 4.6 Spawn、roster、interaction、loot、encounter 与 evidence

| ID | 差距 | 重构要求 |
|---|---|---|
| WORLD-P1-057 | `bootstrap_roster`只有顺序count/projection | `SpawnDefinitionArtifact`声明stable spawn key、definition、cell、policy、dependencies与source revision |
| WORLD-P1-058 | camp layout以O(drawIndex)重放RNG accessor | compiler单向消费确定性stream并产出artifact；runtime不得为单点查询重放前缀 |
| WORLD-P1-059 | placement分别求X/Z/ground并重复safe-position | cook一次返回typed transform+surface+validation receipt，避免axis式重算 |
| WORLD-P1-060 | spawn没有atomic materializer | preflight definition/cell/collision/entity capacity后一次commit entity+components+spatial+event |
| WORLD-P1-061 | 无SpawnId、respawn与despawn lifecycle | stable spawn key与live entity generation分离，定义activation/death/respawn/retire/save规则 |
| WORLD-P1-062 | NPC/object/mailbox/healer placement owner分散 | 统一definition/compiler/instance protocol，typed component bundle保留领域差异 |
| WORLD-P1-063 | `interaction_selection`没有产品consumer | `InteractionRuntime`消费spatial/LOS/capability并返回可解释、稳定排序、有界结果 |
| WORLD-P1-064 | corpse rights candidate与产品loot双authority | 并入Runtime13 death fact、Runtime15 party snapshot和Runtime14 loot transaction/receipt |
| WORLD-P1-065 | world boss participation candidate与combat/lockout断开 | `WorldEncounterRuntime`拥有encounter instance、participant credit、phase/result并发出typed eligibility |
| WORLD-P1-066 | mob lifecycle/scan counter没有统一telemetry owner | counters从真实spatial/nav/movement pipeline采集，带world/tick/system和sampling policy |
| WORLD-P1-067 | package存在不等于artifact执行 | Tooling10 inventory校验entry、binary dir、source digest、command、exit、test count和result completeness |
| WORLD-P1-068 | parity trace与产品BuildSet/backend脱离 | exact trace从产品root记录world/nav/collision generation、seed、tick、inputs和首差异；historical oracle显式隔离 |

## 5. P2 完整性与维护性差距

| ID | 差距 | 完整化要求 |
|---|---|---|
| WORLD-P2-001 | 49模块有397个public function，domain facade缺失 | public surface收敛到definition/query/movement/spawn/interaction owner，helper默认私有 |
| WORLD-P2-002 | 274个throw主要只有字符串 | stable error enum、context、source location、retryability与telemetry mapping |
| WORLD-P2-003 | 多处`axis`参数表达二维结果 | 使用typed Vec2/Vec3/Transform/Hit/Route，减少重复计算和轴错配 |
| WORLD-P2-004 | magic distance/step/radius散落 | 纳入agent/query/content profile并记录单位与设计来源 |
| WORLD-P2-005 | generated accessor不可读也难做diff | 保留human-readable source和canonical generated binary/summary/diff，不手审数千分支 |
| WORLD-P2-006 | test main与production module数量关系不可见 | 生成source→owner→entry→artifact→test矩阵并在索引展示 |
| WORLD-P2-007 | contract test只返回整数首错 | 结构化case/result/expected/actual/source revision，允许多失败聚合 |
| WORLD-P2-008 | 无world query debug draw/capture | 提供broadphase cells、shapes、sweep、hit normal、nav corridor、spatial candidates有界capture |
| WORLD-P2-009 | 无spawn/roster inspector | 显示definition key、cell、entity generation、activation、respawn、dependency与last receipt |
| WORLD-P2-010 | 无terrain/content provenance inspector | 显示source/cook/install generation、tile digest、LOD/residency与cross-artifact mismatch |
| WORLD-P2-011 | target/interaction拒绝原因不可见 | debug projection列出filter stages与有界候选，不暴露未授权实体 |
| WORLD-P2-012 | profiler无query cardinality维度 | 记录visited cells、broadphase candidates、narrowphase tests、path nodes、sort candidates与alloc bytes |
| WORLD-P2-013 | test artifact缺失没有统一severity | required缺失直接fail，optional标skip+reason+owner+expiry，禁止静默忽略目录 |
| WORLD-P2-014 | 缺save/reload与world generation golden | 覆盖spawn keys、entity generations、nav/collision revision、pending request和retirement |
| WORLD-P2-015 | 参考引擎采用点没有decision record | 每个借鉴/拒绝点记录Zircon约束、owner与验证，不复制API外形代替设计 |
| WORLD-P2-016 | Unity Graphics不是world/physics/nav语义来源 | 只借鉴显式resource/pass dependency、compile/execute/cleanup阶段，不宣称其定义game world |

## 6. 参考引擎差异

### 6.1 Unreal Engine

`UWorld`显式拥有persistent level、streaming levels与level collections；`UWorldPartition`有Initialize/Uninitialize状态、runtime hash、streaming source/policy与runtime cell。碰撞查询使用`FCollisionQueryParams`、object/channel response和ignored actor集合，World collision面提供line trace、sweep、overlap及async datum，而不是把resolved X/Z作为唯一结果。`UNavigationSystemV1`和Recast navmesh则拥有NavigationData、project point、find path及异步/生成边界。Zircon应吸收的是world generation、partition lifecycle、typed query与nav data owner，不是照搬Unreal对象体系。

### 6.2 Godot

Godot `World3D`分别暴露physics space与navigation map的RID；`PhysicsServer3D`创建space/body并把body显式绑定space；`NavigationAgent3D`持有target、path、next position、avoidance与callback状态。它证明world资源、physics space、navigation map与agent lifecycle可以分层但必须显式连接。当前WOC的seed+coordinate free function、断线pathfinder和全表entity scan缺少这些连接点。

### 6.3 Bevy 与 Fyrox

Bevy `World`拥有自己的entities，Entity带generation，spawn/despawn使旧引用失效；transform propagation按changed frontier处理层级，而非每个消费者重建全量关系。Fyrox Graph集中拥有scene node pool与add/remove生命周期，Pool handle用generation阻止slot复用命中旧对象；其Navmesh是scene-owned数据而不是每次请求重建的局部脚本网格。Zircon需要同等明确的world scope、generation、graph/pool和nav artifact owner。

### 6.4 Unity Graphics

Unity Graphics `RenderGraph`把resource registry、record pass、compile、execute与cleanup分阶段，builder要求声明resource use。它只提供结构纪律类比：terrain/collision/nav/spawn artifact必须声明owner、dependency、generation、read/write与retire；它不是world partition、physics、navigation或MMO spawn语义来源。

## 7. 目标 Owner 与边界

| Owner | 唯一职责 | 禁止承担 | 前置依赖 |
|---|---|---|---|
| `WorldDefinitionRegistry` | versioned world/region/surface/spawn/collision/nav artifact manifest与provenance | live entity、movement mutation | Tooling05、Runtime04 asset |
| `WorldPartitionRuntime` | cell demand、load/install/activate/retire、residency与generation | content authoring、combat | Runtime05 world、asset streaming |
| `TerrainRuntime` | installed terrain tile/height/surface/gradient query与cache | render pipeline、spawn transaction | Definition/Partition、Runtime09D |
| `StaticCollisionRuntime` | cooked shapes、broadphase、sweep/overlap/raycast query与hit diagnostics | movement policy、instance allocation | Runtime08A、Definition/Partition |
| `MovementRuntime` | agent state、sweep/slide/floor、nav corridor follow与authoritative result | nav generation、input mapping | Runtime06/08A/08D、Network |
| `SpatialQueryRuntime` | generation-qualified entity index、snapshot、filter、bounded query | faction/threat policy | Runtime05、entity lifecycle |
| `SpawnRuntime` | spawn definition instance、atomic materialize、despawn/respawn、cell lifecycle | encounter rules、loot | Definition/Partition/Spatial |
| `InteractionRuntime` | interactable eligibility、spatial/LOS/capability filter、selection receipt | target combat、loot commit | Spatial/Collision、Runtime15 |
| `WorldEncounterRuntime` | open-world encounter/participant/phase/result与eligibility fact | inventory mutation、instance allocator | Runtime13/15/16、Spawn |
| `WorldProjectionRuntime` | recipient-filtered movement/target/spawn/world debug projection | authority mutation | Runtime08E、所有world owner |
| `WorldEvidenceRunner` | product-root trace/save-load/fault/scale/perf与artifact completeness | 第二套world规则 | Tooling05/10、source oracle |

依赖顺序必须先冻结WorldDefinition/BuildSet和generation identity，再交付partition/install，随后接terrain/collision/nav/spatial query，之后迁movement/spawn/interaction/encounter，最后hard cut标量候选与旧WorldState字段。不能先把`PathfindState`或`SpatialGridState`简单import进monolith；没有generation、budget和唯一owner时，那只会把断线candidate变成第二套长期authority。

## 8. 重构里程碑

### M0 · Topology、capability 与 source truth freeze

- 生成208文件的production/candidate/oracle/test分类和normal/self-test/trace closure；
- 将14个不可达candidate及缺artifact能力标为NotQualified；
- 固定source revision、BuildSet和historical/current oracle差异。

### M1 · WorldDefinition、identity 与 partition artifact

- 交付WorldDefinition/Instance/Generation/Cell/Spawn/Shape/Nav handle；
- 编译terrain/collision/nav/spawn共同revision的cell artifact；
- 建load/install/activate/retire与last-good transaction。

### M2 · Terrain、collision 与 spatial query

- terrain query消费installed tile/cache，不再重复生成content ladder；
- collision接Runtime08A broadphase/continuous sweep并返回typed hit；
- spatial index由spawn/move/despawn增量维护并服务所有world query。

### M3 · Navigation 与 locomotion

- nav接Runtime08D tiled artifact、async request、typed failure和corridor invalidation；
- player/mob/pet movement统一agent definition、floor/contact、sweep/slide与budget；
- 删除0.2全表采样、axis重复查询和silent straight fallback。

### M4 · Spawn、interaction 与 encounter

- spawn definitions以stable key原子materialize entity/components/spatial/event；
- interaction共享spatial/LOS但保持独立policy；
- corpse/world-boss事实接combat/party/loot transaction，hard cut断线state。

### M5 · Product migration 与 projection

- WorldState只保留owner handle/adapter，迁出constructor、target、movement、spawn平行列；
- client/server snapshot携world/nav/collision generation与movement receipt；
- historical scalar helpers移入oracle或删除，不留双authority。

### M6 · Qualification

- 修复五个未接管test main、64个缺失world binary dir和六个current parity owner；
- 通过save/load/replay/stale generation/fault/security/streaming切代；
- 以shape/entity/path/cell规模曲线和p50/p99硬预算证明性能。

## 9. Runtime 资格门

| Gate | 验收内容 |
|---|---|
| WORLD-G01 | production/candidate/oracle/test topology可生成；normal产品root不依赖test/self-test entry |
| WORLD-G02 | WorldDefinition/Instance/Generation/Cell/Entity/Shape/Spawn/Nav handle完整且stale fail-closed |
| WORLD-G03 | terrain/collision/nav/spawn/render cell artifact同一BuildSet/revision，mismatch不得原子发布 |
| WORLD-G04 | partition load/install/activate/retire支持cancel、failure、last-good与旧query安全退休 |
| WORLD-G05 | terrain query有明确精度、坐标、surface、out-of-world与cache/invalidation合同 |
| WORLD-G06 | collision一次返回typed pair/hit，broadphase候选随局部密度增长且continuous sweep覆盖高速体 |
| WORLD-G07 | player/mob/pet movement共享agent/query owner，floor/step/slope/water/contact/correction可重放 |
| WORLD-G08 | path request绑定nav generation，NoPath/Partial/Budget/Stale可区分且无silent直线fallback |
| WORLD-G09 | spatial index由entity lifecycle增量维护，despawn/reuse/cross-cell/snapshot无stale命中 |
| WORLD-G10 | targeting/aggro/interaction共享有界spatial候选，relationship/LOS/filter stage可解释 |
| WORLD-G11 | spawn materialization在definition/cell/entity/spatial/event间原子，失败零可见副作用 |
| WORLD-G12 | corpse rights/world boss participation接combat/party/loot durable transaction且exactly-once |
| WORLD-G13 | snapshot/replay携world/nav/collision/spawn generation，save/load与hot cutover语义稳定 |
| WORLD-G14 | 170/1k/100k shape及1/100/10k/100k entity规模曲线、alloc与p50/p99满足预算 |
| WORLD-G15 | current parity从产品root绑定source/backend/seed/tick/exact trace；historical oracle不计产品pass |
| WORLD-G16 | 141 test main与74 world package全进inventory；required artifact实际执行，frontmatter/link/count/diff门全绿 |

## 10. 状态与边界

| 项目 | 状态 |
|---|---|
| world物理清单、去重范围与main/product reachability | review_complete |
| terrain/collision/movement/nav/spatial/target/spawn热路径 | review_complete |
| package/parity/artifact/provenance证据 | review_complete |
| Unreal/Godot/Bevy/Fyrox/Unity Graphics参考路由 | review_complete |
| production代码、测试、manifest、generated artifact修改 | pending，未在本轮执行 |
| WOC native动态验证 | blocked_by_existing_compile_errors，6个`woc_protocol`错误后无测试执行 |
| WOC npm动态验证 | blocked_by_existing_contract_drift，typed contract 157与expected 148不一致 |

本篇没有把14个不可达模块判定为必须丢弃。terrain/collision/spawn常量、路径算法样本和parity trace均可作为source migration oracle；但只有进入唯一产品owner、绑定同代artifact并通过真实产品根与规模门后才算能力。反过来，不能通过给candidate补一个import或给缺失binary目录重新建空文件来关闭差距。

Runtime05继续拥有通用world/entity lifecycle；Runtime08A拥有物理shape/broadphase/query；Runtime08D拥有navigation；Runtime08F拥有AI/perception；Runtime12拥有WorldState schedule/codec；Runtime13拥有combat/death/threat；Runtime14拥有loot transaction；Runtime15拥有principal/party；Runtime16拥有instance/dungeon/delve/pet；Editor28/40拥有spawn/PCG authoring；Tooling05/10拥有artifact/evidence。本篇只拥有WOC world definition/partition消费、terrain/collision/movement/spatial/spawn/interaction产品接线及其资格门。
