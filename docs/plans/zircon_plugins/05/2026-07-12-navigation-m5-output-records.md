---
related_code:
  - zircon_plugins/navigation/native/native/detour_off_mesh_connections.cpp
  - zircon_plugins/navigation/native/native/detour_tile_cache.cpp
  - zircon_plugins/navigation/runtime/src/manager/traversal/mod.rs
  - zircon_runtime/src/core/framework/navigation/off_mesh_link.rs
  - zircon_runtime/src/core/framework/navigation/asset/v1.rs
implementation_files:
  - zircon_plugins/navigation/native/native/detour_off_mesh_connections.cpp
  - zircon_plugins/navigation/native/native/detour_off_mesh_connections.h
  - zircon_plugins/navigation/runtime/src/manager/traversal/advance.rs
  - zircon_plugins/navigation/runtime/src/manager/traversal/capacity.rs
  - zircon_plugins/navigation/runtime/src/manager/traversal/selection.rs
  - zircon_plugins/navigation/runtime/src/manager/traversal/state.rs
  - zircon_runtime/src/core/framework/navigation/asset/v1.rs
plan_sources:
  - docs/plans/zircon_plugins/05-navigation.md
  - user: 2026-07-12 严格按 zircon_plugins 架构计划完成插件功能
tests:
  - zircon_plugins/navigation/native/src/tests/tile_cache.rs
  - zircon_plugins/navigation/runtime/src/tests/off_mesh.rs
  - zircon_plugins/navigation/runtime/src/tests/asset_migration.rs
  - zircon_plugins/navigation/runtime/src/manager/traversal/tests.rs
doc_type: milestone-detail
---

# 2026-07-12 Navigation M5 Off-mesh 产出记录

## 状态与产出记录

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M5 | T1 link/bridge bake 注入 | `完成` | 2026-07-12 | Off-mesh 稳定 ID 进入 Detour user id；新增独立 `detour_off_mesh_connections` 责任模块，并让普通查询与 TileCache tile rebuild 共用；`offmesh_link_present_in_baked_tiles` 通过。 |
| M5 | T2 traverse 状态机、事件与 bridge 容量 | `完成` | 2026-07-12 | `manager/traversal/` 按 Approach/Traverse/Exit、路径选择、容量队列拆分；typed `OffMeshTraverseEvent`、`jump_link_end_to_end_traverse`、`bridge_capacity_queues_agents` 通过。 |
| M5 | Testing | `完成` | 2026-07-12 | Windows coordinator-managed：native build/test `OK`（33 unit + 4 integration），runtime build/test `OK`（59 tests）；共享 runtime 聚焦回归 12/12；`rustfmt --check`、`git diff --check` 与文件规模门禁通过。 |

## 架构证据

- 当前仓库：framework 保持中立 DTO；runtime 持有 agent traversal 与容量状态；native 仅持有 Detour 数据编码和 tile 注入。
- Recast/Detour：沿用 `dtNavMeshCreateParams::offMeshCon*`、稳定 `offMeshConUserID` 以及 `DT_STRAIGHTPATH_OFFMESH_CONNECTION` 语义。
- Godot：链接启用、双向、端点和 travel cost 保持独立运行期迭代数据。
- Unreal：自定义链接以 moving-agent 集合和显式完成通知管理占用，映射为 Zircon 的共享容量队列与 started/completed typed event。

## 里程碑测试确认

- TileCache 重建后仍保留链接并返回稳定 link id：已确认。
- 抛物线跳跃能完成 Approach → Traverse → Exit 且发送开始/完成事件：已确认。
- capacity=1 时两个 agent 按 FIFO 排队且都能最终通过：已确认。

## 验证补充

- `zircon_runtime` 完整包验证及其完整测试二进制各运行 904 秒后超时，期间未报告失败；该超范围结果不计为通过。
- 同一受管构件中的 M5 直接相关集合已单独通过：framework navigation 9/9、asset navigation 1/1、builtin navigation runtime 2/2。
- Review 修复后的 `zircon_runtime -SkipBuild` 全包尝试在其他会话的 `material_shader_redirect_dependency_contract.rs` 编译处失败（缺少 `MaterialAsset::from_toml_str`、两处 `AssetReference` 借用类型不匹配）；该外部测试不属于 Navigation 05，导航 runtime 59 项已重新从清理后的受管 target 全量通过。
- 新增遍历阈值按代码审查规则收敛为 `manager/traversal/advance.rs` 私有常量；归一化进度边界保留为定义绑定值。

## 独立代码审查闭合

- FIFO：取消 destination 或关闭 position update 时，queued/active capacity 都通过同一幂等释放路径清理；两个后继可达回归通过。
- 路径：仅当前或下一 actionable path point 的 link id 可启动 traversal，普通拐角不会被越过。
- 事件：Started/Completed 在 Transform 写回成功后才进入 world event 与 tick report；静态 Transform 失败回归通过。
- 资产：v1 link wire DTO 独立迁移到 v2，未知版本返回 typed error；runtime 跨边界迁移测试通过。
- ABI：非零 link count 与 null link pointer 在 bounds 扫描前被拒绝；直接 ABI 回归通过。
