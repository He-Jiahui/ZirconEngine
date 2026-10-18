# ZirconEngine 导航寻路插件补齐计划

## Summary
- 采用 **Unity 组件体验 + Unreal NavigationSystem/Recast 架构 + Recast/Detour 原生后端 + Zircon 插件隔离** 的方案。
- `zircon_runtime` 只新增中性导航契约、资源类型、动态组件能力；真实导航行为落在 `zircon_plugins/navigation/runtime`，编辑器窗口、组件抽屉、烘焙命令和 Gizmos 落在 `zircon_plugins/navigation/editor`。
- v1 聚焦 3D NavMesh：`NavMeshSurface`、`NavMeshModifier`、`NavMeshAgent`、`NavMeshObstacle`、`NavMeshOffMeshLink`、Agent/Area Settings、烘焙资产、路径查询、基础 agent 跟随、障碍避让/静态 carving、编辑器烘焙与可视化。

## Reference Evidence
- Unity 组件形状参考：NavMesh Surface/Modifier/Agent/Obstacle/Link/OffMeshLink 官方文档：[Surface](https://docs.unity3d.com/Packages/com.unity.ai.navigation%402.0/manual/NavMeshSurface.html)、[Modifier](https://docs.unity3d.com/Packages/com.unity.ai.navigation%402.0/manual/NavMeshModifier.html)、[Agent](https://docs.unity3d.com/Packages/com.unity.ai.navigation%402.0/manual/NavMeshAgent.html)、[Obstacle](https://docs.unity3d.com/Packages/com.unity.ai.navigation%402.0/manual/NavMeshObstacle.html)、[NavMesh Link](https://docs.unity3d.com/Packages/com.unity.ai.navigation%402.0/manual/NavMeshLink.html)、[OffMesh Link](https://docs.unity3d.com/Packages/com.unity.ai.navigation%402.0/manual/OffMeshLink.html)。
- Unreal 主参考：`dev/UnrealEngine/Engine/Source/Runtime/NavigationSystem` 的 `NavigationSystem`、`RecastNavMesh`、`RecastNavMeshGenerator`、`NavModifierComponent`、`NavLinkComponent`、`PathFollowingComponent`、`CrowdManager`；底层 Recast/Detour 来自 [Navmesh 模块](E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Navmesh/Navmesh.Build.cs:1)。
- Recast/Detour 能力边界参考：[Recast README](https://github.com/recastnavigation/recastnavigation/blob/main/README.md)，模块按 Recast 生成、Detour 查询、DetourTileCache 动态 tile、DetourCrowd agent/avoidance 划分。
- Rust/编辑器对照：Fyrox [navmesh runtime](E:/Git/ZirconEngine/dev/Fyrox/fyrox-impl/src/scene/navmesh.rs:1) 与 [navmesh editor](E:/Git/ZirconEngine/dev/Fyrox/editor/src/interaction/navmesh/mod.rs:1)，Godot [NavigationServer3D](E:/Git/ZirconEngine/dev/godot/servers/navigation_3d/navigation_server_3d.h:1) 与 [server tests](E:/Git/ZirconEngine/dev/godot/tests/servers/test_navigation_server_3d.cpp:1)。

## Key Changes
- 新增 `zircon_runtime::core::framework::navigation`，定义中性 DTO：agent type、area/cost、bake settings、surface/modifier/agent/obstacle/link descriptors、path query/result、bake report、runtime stats、structured errors。
- 扩展资源系统：新增 `ResourceKind::{NavMesh, NavigationSettings}`、`NavMeshMarker`、`NavigationSettingsMarker`、`NavMeshAsset`、`NavigationSettingsAsset`；烘焙资产扩展名固定为 `.znavmesh`，settings 使用 TOML。
- 扩展动态组件支持，让 `ScenePropertyValue::{Vec2, Vec3, Vec4, Entity, Resource}` 能从 JSON 读写；导航五大组件都作为 `navigation.Component.*` 动态组件注册，不并入 `World` 固定字段。
- 在 `zircon_plugins/navigation/native` 新增 `zircon_plugin_navigation_recast`：vendored upstream Recast/Detour C++ 源码，`cc` 编译，手写 C ABI，安全 Rust wrapper；不使用 Unreal 修改版源码落地。
- 替换现有 `DefaultNavigationManager::straight_path` 占位，新增真实 `NavigationManager`：加载 navmesh asset、维护 map/surface/agent runtime state、提供 path query、sample、raycast、agent tick、obstacle update、off-mesh link traversal。
- 编辑器扩展新增导航专用窗口和 viewport overlay：Navigation 面板含 Surfaces、Agents & Areas、Bake、Debug/Gizmos；注册五个组件 drawer、bake/clear/open settings 操作、NavMesh asset editor、Scene Gizmos。

## Public Interfaces
- 组件 type id 固定：
  - `navigation.Component.NavMeshSurface`
  - `navigation.Component.NavMeshModifier`
  - `navigation.Component.NavMeshAgent`
  - `navigation.Component.NavMeshObstacle`
  - `navigation.Component.NavMeshOffMeshLink`
- 默认 area：`0=not_walkable`、`1=walkable`、`2=jump`、`3..63=custom`；agent area mask 用 `u64`。
- 默认 humanoid agent：radius `0.5`、height `2.0`、max_climb `0.4`、max_slope `45deg`、speed `3.5`、acceleration `8.0`、angular_speed `360deg/s`、stopping_distance `0.1`。
- `NavMeshSurface` 字段：enabled、agent_type、collect_mode、volume center/size、use_geometry、include_layers、default_area、generate_links、voxel/tile overrides、min_region_area、build_height_mesh、output_asset。
- `NavMeshModifier` 字段：mode add/modify/remove、affected_agents、apply_to_children、override_area/area、override_generate_links/generate_links。
- `NavMeshAgent` 字段：agent_type、radius、height、base_offset、speed、angular_speed、acceleration、stopping_distance、auto_braking、avoidance_quality、priority、area_mask、auto_repath、auto_traverse_links、update_position、update_rotation。
- `NavMeshObstacle` 字段：shape box/capsule、center、size/radius/height、avoidance_enabled、carve、move_threshold、time_to_stationary、carve_only_stationary。
- `NavMeshOffMeshLink` 字段：start/end entity 或 local point、width、bidirectional、activated、auto_update_positions、cost_override、area_type、agent_type、traversal_mode。
- `NavigationManager` 最小 API：
```rust
fn bake_surface(&self, world: &World, request: NavMeshBakeRequest) -> Result<NavMeshBakeReport, NavigationError>;
fn load_nav_mesh(&self, asset: NavMeshAsset) -> Result<NavMeshHandle, NavigationError>;
fn find_path(&self, query: NavPathQuery) -> Result<NavPathResult, NavigationError>;
fn sample_position(&self, query: NavSampleQuery) -> Result<Option<NavSampleHit>, NavigationError>;
fn raycast(&self, query: NavRaycastQuery) -> Result<NavRaycastResult, NavigationError>;
fn tick_world_agents(&self, world: &mut World, dt_seconds: f32) -> Result<NavAgentTickReport, NavigationError>;
```

## Milestones
1. **Framework and asset foundation**
   - Add navigation framework DTOs, resource markers/kinds, asset serialization, artifact-store routing, dynamic property JSON support, and module docs.
   - Testing stage: `cargo test -p zircon_runtime navigation framework resource dynamic_components`.

2. **Recast/Detour backend**
   - Add `zircon_plugin_navigation_recast`, safe wrapper, license notice, deterministic coordinate conversion, simple mesh bake, tiled mesh bake, Detour path query, tile-cache obstacle carve, DetourCrowd agent update.
   - Testing stage: plugin workspace tests for plane path, blocked region, offmesh link, area cost, invalid config, deterministic binary round-trip.

3. **Runtime plugin completion**
   - Replace placeholder manager, register component descriptors/settings/options/events, implement bake inputs from `World`, load baked assets, query APIs, agent tick, obstacle/link state, plugin unload guard.
   - Testing stage: `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_navigation_runtime`.

4. **Editor authoring and baking**
   - Add Navigation window, component drawers, settings editor, bake/clear/open asset operations, bake diagnostics, dirty-state tracking, asset browser integration.
   - Add viewport overlay descriptors and renderer support for navmesh triangles/tile boundaries/areas, surface volumes, agent cylinders/path corridors, obstacles, off-mesh links, and dirty bake regions.
   - Testing stage: `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_navigation_editor` plus editor registry/gizmo snapshot tests.

5. **Workspace acceptance and docs**
   - Update `docs/zircon_runtime/core/framework/navigation.md`, `docs/zircon_plugins/navigation/runtime.md`, `docs/zircon_plugins/navigation/editor.md`, and plan-source headers.
   - Final validation: `cargo check --workspace`, `cargo test --workspace`, and `cargo test --manifest-path zircon_plugins/Cargo.toml --workspace`.

## Test Scenarios
- Bake collection modes: all objects, hierarchy, volume, modifier-only; render mesh vs collider geometry; agents/obstacles excluded from bake geometry.
- Modifiers: remove object, override area, apply-to-children, child override precedence, affected agent filtering.
- Assets: bake creates `.znavmesh`, reload preserves tile data, stale/missing asset reports diagnostics, settings hash invalidates old bake.
- Queries: empty map returns structured no-path, valid map returns string-pulled path, area mask blocks forbidden areas, off-mesh link appears in path flags.
- Runtime: agent reaches target, partial path auto-repath, obstacle avoidance changes velocity, stationary carved obstacle invalidates affected tiles, disabled plugin removes capabilities.
- Editor: all views/drawers/menu operations register, capability gating hides navigation UI, Bake/Clear updates asset reference, Gizmos produce stable overlay counts and pick shapes.

## Assumptions
- v1 is 3D only; 2D navigation, world-partition streaming, automatic jump-link generation beyond basic Recast links, and advanced crowd formation are follow-up work.
- `NavMeshOffMeshLink` keeps the requested Unity-style name, but its semantics follow modern `NavMeshLink`.
- Runtime/editor communicate through descriptors, resources, handles, and `NavigationManager`; editor never owns authoritative runtime world data.
- No compatibility shim for the old straight-line placeholder API; this is a hard replacement inside the navigation plugin.
