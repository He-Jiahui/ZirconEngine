# Navigation M3 Agent / Crowd 产出记录

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

| 状态 | 日期 | 切片 | 产出与证据 |
|------|------|------|------------|
| 已完成 | 2026-07-12 | M3-T1 Crowd C bridge 与 safe wrapper | bridge ABI v2 新增 Crowd create/free、agent add/remove、target、position sync、update 与 batch state read。创建成功后才把 Detour query/navmesh owner 转移给 Crowd，失败仍由 Rust RAII 释放；所有可能抛出异常的命令边界均转换成固定布局错误。Rust `RecastCrowd` 保持 `Send`/非 `Sync` 和唯一可变访问。精确测试 `crowd_update_round_trips_agent_states`、`crowd_syncs_controller_owned_position_into_the_corridor` 已通过。 |
| 已完成 | 2026-07-12 | M3-T2 `navigation.agent_tick` 与 repath 预算 | runtime 插件在 descriptor、静态 manifest 和运行时注册中声明 Update 阶段 `navigation.agent_tick` system anchor，并显式 after `ai.behavior_tick`。`NavRepathBudget` 的一个单位对应一次真实 `requestMoveTarget`，删除了重复的临时 preflight query；round-robin cursor 保证待处理 agent 不饥饿。精确测试 `agent_tick_registered_after_ai_behavior_tick`、`repath_budget_caps_queries_per_frame`、`repath_budget_rotates_across_pending_agents_without_starvation` 已通过。 |
| 已完成 | 2026-07-12 | M3-T3 avoidance、过滤与双写回模式 | Detour `dtQueryFilter` 增加有 NOTICE 记录的 64-bit area mask 扩展；Crowd 最多复用 16 个 `(area_mask, asset area costs/walkability)` filter slot，agent 投影和 corridor 使用同一 filter。Transform 写回 Crowd position/rotation；DesiredVelocity 写入 `navigation.Component.NavDesiredVelocity`，下一帧把 controller Transform 经 `dtPathCorridor::movePosition` 同步回 Crowd。精确测试覆盖面积掩码拒绝、DesiredVelocity feedback、20-agent corridor。 |
| 已完成 | 2026-07-12 | M3-T4 per-navmesh Crowd owner | `NavMeshAgentDescriptor.nav_mesh` 可显式路由；未指定时使用最早加载 handle。runtime 按 `NavMeshHandle` 分组并持有独立 persistent Crowd，卸载/设置变化时收束 owner，避免不同 surface agent 误共享最低 handle。精确测试 `agents_route_to_their_explicit_nav_mesh_crowd` 已通过。 |
| 已通过 | 2026-07-12 | Windows native 验证 | 协调器 job `2dc36f6c89f44ffe837b19fc69284c60`：`cargo test -p zircon_plugin_navigation_recast --locked` 全包通过，22 unit + 4 integration / 0 failed；包含 Crowd area-mask、controller-position sync、filter slot 回收回归；doc tests 通过。 |
| 已通过 | 2026-07-12 | Windows runtime 验证 | 协调器 job `de1d93af6e734c9d9af4eda1fb58d737`：`cargo test -p zircon_plugin_navigation_runtime --locked`，47 passed / 0 failed；doc tests 通过。期间由 no-path fixture 暴露 partial corridor 写回问题，现统一将 partial/failed target 报为 blocked；新增跨 navmesh 预算公平、owner 切换清理与坏 agent 隔离回归。 |
| 已完成 | 2026-07-12 | 结构约束处理 | native Crowd、runtime agent/repath/writeback、组件 DTO/descriptor 分层独立；未新增兼容 shim、旧 API 转发或跨插件反向依赖。结构/格式最终门禁在本记录提交前重新审计。 |

| 里程碑 | 测试阶段 | 状态 |
|---|---|---|
| M3 | M3-Testing：Crowd native、runtime system/repath/writeback 精确包级验收 | 通过 |

---
related_code:
  - zircon_plugins/navigation/plugin.toml
  - zircon_plugins/navigation/native/NOTICE.md
  - zircon_plugins/navigation/native/vendor/recastnavigation/Detour/Include/DetourNavMeshQuery.h
  - zircon_plugins/navigation/native/vendor/recastnavigation/Detour/Source/DetourNavMeshQuery.cpp
  - zircon_plugins/navigation/native/vendor/recastnavigation/DetourCrowd/Include/DetourCrowd.h
  - zircon_plugins/navigation/native/vendor/recastnavigation/DetourCrowd/Source/DetourCrowd.cpp
  - zircon_plugins/navigation/native/native/recast_bridge.h
  - zircon_plugins/navigation/native/native/detour_query.cpp
  - zircon_plugins/navigation/native/native/detour_crowd.cpp
  - zircon_plugins/navigation/native/src/ffi.rs
  - zircon_plugins/navigation/native/src/detour.rs
  - zircon_plugins/navigation/native/src/crowd.rs
implementation_files:
  - zircon_plugins/navigation/plugin.toml
  - zircon_plugins/navigation/native/NOTICE.md
  - zircon_plugins/navigation/native/build.rs
  - zircon_plugins/navigation/native/native/recast_bridge.cpp
  - zircon_plugins/navigation/native/native/recast_bridge.h
  - zircon_plugins/navigation/native/native/detour_query.cpp
  - zircon_plugins/navigation/native/native/detour_crowd.cpp
  - zircon_plugins/navigation/native/vendor/recastnavigation/Detour/Include/DetourNavMeshQuery.h
  - zircon_plugins/navigation/native/vendor/recastnavigation/Detour/Source/DetourNavMeshQuery.cpp
  - zircon_plugins/navigation/native/src/ffi.rs
  - zircon_plugins/navigation/native/src/detour.rs
  - zircon_plugins/navigation/native/src/crowd.rs
  - zircon_plugins/navigation/native/src/lib.rs
  - zircon_runtime/src/core/framework/navigation/agent.rs
  - zircon_runtime/src/core/framework/navigation/constants.rs
  - zircon_runtime/src/core/framework/navigation/mod.rs
  - zircon_plugins/navigation/runtime/src/agent.rs
  - zircon_plugins/navigation/runtime/src/agent/repath.rs
  - zircon_plugins/navigation/runtime/src/agent/writeback.rs
  - zircon_plugins/navigation/runtime/src/components.rs
  - zircon_plugins/navigation/runtime/src/components/agent.rs
  - zircon_plugins/navigation/runtime/src/manager.rs
  - zircon_plugins/navigation/runtime/src/manager/state.rs
  - zircon_plugins/navigation/runtime/src/manager/tick.rs
  - zircon_plugins/navigation/runtime/src/plugin.rs
  - zircon_plugins/navigation/runtime/src/lib.rs
plan_sources:
  - docs/plans/zircon_plugins/05-navigation.md
  - user: 2026-07-12 strict zircon_plugins architecture implementation goal
tests:
  - zircon_plugins/navigation/native/src/tests/crowd.rs
  - zircon_plugins/navigation/native/src/tests/linkage.rs
  - zircon_plugins/navigation/runtime/src/tests/crowd.rs
  - zircon_plugins/navigation/runtime/src/tests/registration.rs
doc_type: module-detail
---
