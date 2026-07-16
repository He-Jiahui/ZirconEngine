# 05 · Navigation 插件完善计划（Surface / Bakery / Agent / Obstacle / Modifier / OffMeshLink）

```zircon-workflow
{
  "schema": 1,
  "workflow_id": "plugins-05-navigation",
  "goal": "完成 Navigation 插件的 bake、crowd、obstacle、off-mesh 与 Editor 产品闭环，并保留逐层受管验证证据。",
  "milestones": [
    {"id": "M1", "title": "SimpleBake 闭环", "depends_on": []},
    {"id": "M2", "title": "TiledBake 与异步", "depends_on": ["M1"]},
    {"id": "M3", "title": "Agent 与 Crowd", "depends_on": ["M1"]},
    {"id": "M4", "title": "Obstacle 与 Modifier", "depends_on": ["M2"]},
    {"id": "M5", "title": "Off-mesh", "depends_on": ["M1", "M3"]},
    {"id": "M6", "title": "Editor", "depends_on": ["M2", "M3"]},
    {"id": "M7", "title": "UI binding shared support", "depends_on": []}
  ]
}
```

<!-- Workflow topology is maintained independently from milestone output records. -->
<!-- M6 support slices use exact child-plan manifests and do not close unrelated open failures. -->

> 状态：工程化细化版 v2 · M1 代码与隔离 Windows 验证完成，待共享检出 closeout · 优先级：P1 · 前置：[01 插件架构核心](01-plugin-architecture-core.md) M1–M2
> 关联计划：`.codex/plans/ZirconEngine 导航寻路插件补齐计划.md` · 现状文档：`docs/zircon_plugins/navigation/{runtime,editor,native}.md`
> 参考实现：Unity NavMesh 组件体验（Surface/Modifier/Agent/Obstacle/Link 五组件）、Unreal NavigationSystem（tile-based 异步重建）、Godot NavigationServer3D（map/region/agent/obstacle/link RID API）、upstream Recast/Detour/DetourCrowd

## 1. 目标

把 `zircon_plugins/navigation` 推进到完整导航系统：真实 Recast 烘焙（含 tiled 异步重建）、Detour 路径查询、DetourCrowd 群体避障 agent、TileCache carving obstacle、modifier 区域语义、off-mesh link/bridge 闭环、编辑器烘焙与可视化工具。

## 2. 现状基线（实查）

成熟度 Beta / Partial。比早期"烘焙 ~99% 缺"的判断更进一步——**native 层已 vendored upstream recastnavigation 并编译四模块**：

- **契约层** `zircon_runtime/src/core/framework/navigation/`：agent/bake/gizmo/handle/manager/modifier/obstacle/off_mesh_link/query/settings/stats/surface 全套 DTO 与 `NavigationManager` trait。
- **native 绑定** `zircon_plugins/navigation/native/`：`build.rs` 经 cc 编译 vendor/recastnavigation 的 Recast/Detour/DetourCrowd/DetourTileCache 全源码 + 自有 C bridge（`native/recast_bridge.cpp`、`recast_bake.cpp`、`detour_query.cpp`、`detour_tile_cache.cpp`）；`src/ffi.rs` 已绑定 `zr_nav_recast_bake_triangle_mesh` 等；`src/{bake,detour,detour_result,tile_cache,asset_ffi}.rs` safe wrapper；`src/fallback_query/`（geometry/graph/path/raycast/sampling/validation）为无 native 时的纯 Rust 退化查询。
- **runtime** `zircon_plugins/navigation/runtime/src/`：六组件（`components/{surface,agent,modifier,obstacle,off_mesh_link,off_mesh_bridge}.rs`）、`manager/bake/`（asset/diagnostics/filter/geometry/modifier/surface）、`manager/{agent_motion,query,state,stats,tick,traversal}.rs`、`runtime_obstacles.rs`、`off_mesh_connections.rs`、`settings_hash.rs`/`settings_validation.rs`；注册经 `register_module(module_descriptor())`（`lib.rs:85`）。
- **editor**：`editor/src/lib.rs` 骨架（绑定层在）。

缺口（按严重度，校准后）：

| # | 缺口 | 证据 |
|---|------|------|
| N1 | TiledBake 异步：bridge 仅单块 `bake_triangle_mesh`，无 tile 网格并行、无后台任务池、无运行期局部重建 | `native/src/ffi.rs:228` 单入口 |
| N2 | DetourCrowd 未接：源码已编译但 C bridge 无 crowd 接口；agent_motion 为简化 steering | `native/build.rs`（仅四个 bridge cpp，无 crowd bridge） |
| N3 | TileCache carving：绑定在（`tile_cache.rs`），obstacle 组件变更 → 受影响 tile 重建的运行闭环缺 | `runtime_obstacles.rs` |
| N4 | Off-mesh：组件与 `off_mesh_connections.rs` 在，bake 期注入与运行期 traverse 状态机不完整 | `manager/traversal.rs` |
| N5 | Modifier area type/cost 语义未贯通 rasterize → query filter | `manager/bake/modifier.rs`、framework `query.rs` |
| N6 | ECS 锚点：Module 形态轮询，无 `navigation.agent_tick` 系统 | `lib.rs`、`manager/tick.rs` |
| N7 | Editor 无烘焙面板/navmesh 可视化/agent 调试 | `editor/src/lib.rs` |

## 3. 架构设计

分层维持既定决策：framework DTO 在 `zircon_runtime::core::framework::navigation`；`navigation/native` 为绑定 crate（vendored upstream + 手写 C ABI bridge，**选型已落地，不再讨论纯 Rust 替代**）；`navigation/runtime` 为插件本体；六组件保持组件注册。

### 3.1 烘焙管线（解决 N1/N5，`runtime/src/bake/` [manager/bake 收编扩展]）

```
CollectInputs(场景几何 + Modifier volumes + OffMesh 声明)
  → BakeContext(per-surface settings: cell size, agent radius/height/climb/slope)
  → SimpleBake | TiledBake(tile 网格并行, 后台任务池)
  → NavMeshAsset(.znavmesh: tile blob[] + settings hash + area table + offmesh 注入记录)
```

```rust
pub struct BakeInput {
    pub triangles: Vec<[f32; 9]>,          // 收集期合并的世界空间三角形
    pub area_volumes: Vec<AreaVolume>,     // Modifier → area id 标记（rasterize 期生效）
    pub off_mesh: Vec<OffMeshConnectionDesc>,
}
pub struct TiledBakeJob { pub surface: NavSurfaceHandle, pub tile: [i32; 2], pub input: Arc<BakeInput> }
```

- 输入收集走 ECS 查询：physics collider 优先（`finish` 阶段经 `CapabilityView::has("runtime.capability.physics.raycast")` 探测，01 §3.4）；无 physics 时退化为 render mesh 提取（`manager/bake/geometry.rs` [改造]，退化路径必须有测试守护）。
- TiledBake：C bridge 新增 `zr_nav_recast_bake_tile`（单 tile 入口，`native/recast_bake.cpp` [改造]）；runtime 侧用 tasks 基础设施后台任务池逐 tile 并行（Unreal 异步重建形态），主线程仅提交/收割 `TiledBakeJob`；运行期局部重烘焙（obstacle/几何变更）复用同一路径。
- area 语义：`AreaVolume` 在 rasterize 期写 area id（`rcMarkConvexPolyArea` bridge [新增]）；area table（id → name/cost 默认值）进 `.znavmesh` 资产头。
- `.znavmesh` 资产经正常资产管线加载（`native/src/asset_ffi.rs` 现有序列化通道扩展 tile 化布局：header{version, settings_hash, area_table, tile_grid} + tile blob 数组）；settings hash 不匹配出诊断并拒绝加载（`settings_hash.rs` 现有校验保持）。

### 3.2 运行期世界（`runtime/src/world/` [新增，manager/state.rs 收编]）

```rust
pub struct NavWorld { /* per-Surface: dtNavMesh + dtNavMeshQuery 池 + dtCrowd + dtTileCache */ }
#[derive(Clone, Copy)] pub struct NavSurfaceHandle(u64);   // 复用契约 handle.rs 形态

impl NavWorld {
    pub fn find_path(&self, surface: NavSurfaceHandle, from: Vec3, to: Vec3,
                     filter: &NavQueryFilter, out: &mut NavPath) -> Result<(), NavigationError>;
    pub fn raycast(&self, …) -> …;
    pub fn closest_point(&self, …) -> …;
    pub fn random_point_in_radius(&self, …) -> …;
}
pub struct NavQueryFilter { pub area_costs: [f32; 64], pub include_flags: u16, pub exclude_flags: u16 } // 契约 query.rs [改造]
```

- Obstacle：`dtTileCache` carving（圆柱/盒，`tile_cache.rs` 现绑定补 add/remove obstacle bridge）；obstacle 组件变更（change detection）→ 受影响 tile 异步重建（§3.1 任务池）。

### 3.3 Agent 与 Crowd（解决 N2，`runtime/src/agent/` [agent_motion.rs 收编扩展]）

- C bridge 新增 crowd 接口（`native/detour_crowd.cpp` [新增]）：`zr_nav_crowd_{create,add_agent,remove_agent,set_target,update,read_states}`，批量读写（每 tick 一次 update + 一次状态数组回读，最小化 FFI 次数）。
- `NavMeshAgent` 组件字段补全：radius/height/max_speed/max_acceleration/avoidance_priority/auto_traverse_offmesh。
- `navigation.agent_tick` 系统 ∈ Update，约束 after `ai.behavior_tick`（01 锚点表）：
  1. 目标变更（change detection）→ repath（增量，分帧预算 `NavRepathBudget` 资源：每帧最多 N 次 find_path）；
  2. `dtCrowd::update` 步进产生期望速度与位置；
  3. 写回方式由组件字段二选一：直接写 Transform，或写 `DesiredVelocity` 组件交角色控制器消费。
- Off-mesh traverse（解决 N4）：agent 进入 link 端点 → `OffMeshTraverseState { link, phase: Approach|Traverse|Exit, t }` 状态机（`manager/traversal.rs` [改造]）；traverse 期间 crowd agent 置 offmesh 状态；位移内置线性/抛物线或交事件（`OffMeshTraverseEvent`）由游戏侧驱动动画。Bridge 为双向多 agent 容量版 link（容量计数在 `off_mesh_connections.rs`）。

### 3.4 ECS 集成（解决 N6，对接 01 定稿 API）

- `register_native_system::<…>(owner, "navigation.agent_tick", SystemStage::Update, …).after("ai.behavior_tick")`；Module 形态保留为 NavWorld 生命周期宿主，每帧轮询逻辑移入系统。
- `register_event::<NavMeshBakeCompleted>` / `::<OffMeshTraverseEvent>`；`register_resource::<NavWorldSettings>`（契约 settings.rs）/`::<NavRepathBudget>`。
- 与 AI 交接（与 [06 AI](06-ai.md) §3.5 对偶）：行为树 `MoveTo` 节点只写 `NavMeshAgent.target` 字段，不直接调用查询。

## 4. 模块文件树

```
zircon_plugins/navigation/native/
  native/recast_bake.cpp        [改造] +zr_nav_recast_bake_tile、area 标记
  native/detour_crowd.cpp       [新增] crowd C bridge
  native/detour_tile_cache.cpp  [改造] +obstacle add/remove
  src/ffi.rs                    [改造] 新 extern 声明
  src/crowd.rs                  [新增] safe wrapper（批量 update/回读）
  src/asset_ffi.rs              [改造] tile 化 .znavmesh 布局
zircon_plugins/navigation/runtime/src/
  bake/mod.rs                   [改造自 manager/bake] BakeInput/TiledBakeJob/任务池
  world/mod.rs                  [新增] NavWorld/查询 API/TileCache 闭环
  agent/mod.rs                  [改造自 manager/agent_motion] crowd 接入 + agent_tick 系统体
  manager/traversal.rs          [改造] OffMeshTraverseState 状态机
  runtime_obstacles.rs          [改造] change detection → tile 重建提交
  lib.rs                        [改造] register_system/resource/event 注册
zircon_plugins/navigation/editor/src/   [扩展] 烘焙面板/overlay（M6）
```

## 5. 里程碑与任务分解

### M1 SimpleBake 闭环

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M1-T1 | 输入收集（physics 优先 + render mesh 退化） | bake/、manager/bake/geometry.rs | 01-M3（CapabilityView） | `bake_input_falls_back_to_render_mesh_without_physics` |
| M1-T2 | 单块烘焙 → .znavmesh → 加载 → find_path 端到端 | bake/、world/、asset_ffi.rs | M1-T1 | `golden_level_bake_then_path_length_within_tolerance` |
| M1-T3 | area volume 标记 + area table | recast_bake.cpp、bake/modifier 路径 | M1-T2 | `modifier_volume_marks_area_id_in_polymesh` |

### M2 TiledBake 与异步

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M2-T1 | zr_nav_recast_bake_tile bridge + tile 网格切分 | recast_bake.cpp、ffi.rs、bake/ | M1 | `tile_bake_matches_simple_bake_geometry` |
| M2-T2 | 后台任务池并行 + 收割；bake 诊断进 store | bake/ | M2-T1 | `tiled_bake_does_not_block_main_thread`、`tile_boundary_paths_are_continuous` |
| M2-T3 | 运行期局部重烘焙路径 | bake/、world/ | M2-T2 | `dirty_tile_rebuild_only_affects_neighbors` |

### M3 Agent / Crowd

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M3-T1 | crowd C bridge + safe wrapper（批量接口） | detour_crowd.cpp、crowd.rs | M1 | `crowd_update_round_trips_agent_states` |
| M3-T2 | navigation.agent_tick 系统 + repath 预算 | agent/、lib.rs | 01-M1、M3-T1 | `agent_tick_registered_after_ai_behavior_tick`、`repath_budget_caps_queries_per_frame` |
| M3-T3 | avoidance/写回模式（Transform 或 DesiredVelocity） | agent/ | M3-T2 | `twenty_agent_corridor_crossing_has_no_deadlock` |

### M4 Obstacle / Modifier

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M4-T1 | TileCache obstacle add/remove bridge + carving 闭环 | detour_tile_cache.cpp、runtime_obstacles.rs | M2 | `obstacle_carving_changes_path`、`obstacle_removal_restores_path` |
| M4-T2 | NavQueryFilter area cost/flags 贯通 | framework query.rs、world/ | M1-T3 | `area_cost_biases_path_choice` |

### M5 Off-mesh

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M5-T1 | link/bridge bake 注入 | bake/、off_mesh_connections.rs | M1 | `offmesh_link_present_in_baked_tiles` |
| M5-T2 | traverse 状态机 + 事件 + bridge 容量 | manager/traversal.rs、agent/ | M3、M5-T1 | `jump_link_end_to_end_traverse`、`bridge_capacity_queues_agents` |

### M6 Editor

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M6-T1 | 烘焙面板（bake/clear/进度，布局 `ai-navmesh-ai-layout.png`） | navigation/editor | M2、[10 规范](10-editor-integration.md) | editor 契约测试 |
| M6-T2 | navmesh viewport overlay（按 area 着色，gizmos 通道，`View/Debug Overlays/Navigation`；契约 `gizmo.rs` 现有 DTO） | navigation/editor | M6-T1 | overlay 注册快照测试 |
| M6-T3 | agent 路径/avoidance 调试视图（play-in-editor 只读镜像） | navigation/editor | M3 | 镜像通道契约测试 |

### M7 UI binding shared support

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M7-T1 | cross-control property parser、tree-scoped descriptor validation 与 Navigation payload kind | runtime interface / Runtime UI binding | 无 | parser 3 项、Runtime behavior 6 项、Runtime UI upward compile |

## 6. 验收命令

```bash
cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_navigation_runtime --locked
cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_navigation_recast --locked
```

## 7. 风险

- Recast C++ 跨平台编译与 [03 Physics](03-physics.md) 的 joltc-sys 同属 native 构建矩阵（cc/cmake + MSVC/clang），CI 配置一起做，避免两套方案。
- `fallback_query/`（纯 Rust 退化查询）与 native 路径的行为一致性需要对照测试守护；fallback 仅保证功能正确，不保证与 Detour 路径逐点一致（容差断言）。
- crowd FFI 每 tick 批量交换的数据布局要一次定准（`#[repr(C)]` 数组），中途改动会破坏 native bridge 版本（`zr_nav_recast_bridge_version` 现有版本号机制递增守护）。
- v1 仅 3D（维持既定范围）；2D 导航与 world-partition streaming 留在后续池。

## 8. 附录 · dev 参考源码对位

实现各任务前**必须先读对应参考实现**，烘焙参数语义与 crowd 调参对照真实代码核对，禁止凭空实现：

| 设计点 | 参考源码（已核验存在） | 看什么 |
|--------|----------------------|--------|
| Recast/Detour/Crowd/TileCache 权威用法 | 仓内 vendored：`zircon_plugins/navigation/native/vendor/recastnavigation`（含官方 RecastDemo 用例） | rcConfig 参数推导、tile 烘焙调用序列、dtCrowd 参数与 obstacle avoidance 档位、dtTileCache obstacle 流程——M1–M4 的第一参考 |
| tiled 异步重建编排 | `dev/UnrealEngine/Engine/Source/Runtime/NavigationSystem/`、`Navmesh/` | dirty area 聚合、tile 重建任务切分与收割时序、NavModifier 区域语义 |
| Server 形态（map/region/agent/obstacle/link） | `dev/godot/servers/navigation_3d/`、`dev/godot/modules/navigation_3d/` | RID API 面、agent 回调模型、off-mesh link 双向语义 |

## 9. 状态与产出记录

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

- M1 SimpleBake：`完成`；产出记录：[2026-07-12 Navigation M1 SimpleBake](05/2026-07-12-navigation-m1-output-records.md)。
- M2 TiledBake：`完成`；产出记录：[2026-07-12 Navigation M2 TiledBake](05/2026-07-12-navigation-m2-output-records.md)。
- M3 Crowd Agents：`完成`；产出记录：[2026-07-12 Navigation M3 Crowd Agents](05/2026-07-12-navigation-m3-output-records.md)。
- M4 Obstacle / Modifier：`完成`；产出记录：[2026-07-12 Navigation M4 Obstacle / Modifier](05/2026-07-12-navigation-m4-output-records.md)。
- M4 跨计划编译回传：`已修复`；[NavQueryFilter 固定数组 serde 编译失败回传](../zircon_editor/editor/15/fixed-2026-07-12-navigation-query-filter-serde-array.md)。
- M5 Off-mesh Link / Bridge：`完成`；产出记录：[2026-07-12 Navigation M5](05/2026-07-12-navigation-m5-output-records.md)。
- M6 Editor：`实现整改中，跨计划 gate 未关闭；注册层硬切代码与非 Cargo 门禁完成，当前源包复验排队`；产出记录：[2026-07-13 Navigation M6](05/2026-07-13-navigation-m6-output-records.md)、[2026-07-13 Navigation registration hard cut](05/2026-07-13-navigation-registration-hard-cut-output-records.md)。
- fixed 已修复：[plugin-operation-factory-runtime-wiring](05/fixed-2026-07-15-plugin-operation-factory-runtime-wiring.md)
- M6-T1 surface 选择态与 operation 参数投影：`待修复（open）`；[failure 交接](05/failure-2026-07-15-navigation-bake-selection-operation-arguments.md)。
- fixed 已修复：[control-prop-ref-validation-runtime-gate](../zircon_runtime/render/18/fixed-2026-07-15-control-prop-ref-validation-runtime-gate.md)
- M6-T2 viewport provider host：`待修复（open）`；[Editor 05 failure](../zircon_editor/editor/05/failure-2026-07-13-plugin-viewport-overlay-provider-runtime-wiring.md)。
- fixed 已修复：[navigation-runtime-driver-manager-layering](../zircon_runtime/render/18/fixed-2026-07-13-navigation-runtime-driver-manager-layering.md)
- fixed 已修复：[plugin-editor-runtime-mirror-consumer-wiring](05/fixed-2026-07-15-plugin-editor-runtime-mirror-consumer-wiring.md)

## 10. 治理失败交接

- 产出记录归档上限：`已修复（fixed）`；[回传记录](../zircon_editor/editor/09/fixed-2026-07-15-navigation-plan-output-record-archive-limit.md)。
<!-- M7 owns the independent Runtime UI binding support gate. -->
