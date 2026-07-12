# Physics M2 QueryMode 与 Sweep 多命中产出记录

## 归属

- 父计划：`docs/plans/zircon_plugins/03-physics.md`
- 里程碑：M2-T3 `QueryMode + sweep 多命中排序`
- 状态：`plugins_03_m2_t3_query_mode_sweep_windows_feature_62_of_62_passed`
- 完成日期：2026-07-12

## 完成项目

- 在 `zircon_runtime::core::framework::physics` 新增中立契约 `PhysicsQueryMode::{First, Closest, All}`，并以 serde default 保持旧查询文档解码为 `Closest`。
- RayCast、ShapeCast、ShapeOverlap 三类查询统一携带 `mode` 与既有 `PhysicsQueryFilter`；ray/shape-cast manager 与 `PhysicsQueryInterface` 硬切为 hit vector 返回值，不保留 Option 兼容分支。
- `First` 保留同步快照插入顺序，`Closest` 取最近命中，`All` 以距离并用 entity id 打破平局；Overlap 因 hit DTO 无距离字段，以查询中心到 collider 中心的距离排序。
- builtin 新增保守连续 shape sweep：以查询形状 OBB 的 AABB 半径扩张目标 collider AABB，再沿查询中心路径求交；初始重叠保持距离 0。该路径对非 box 凸形是保守近似，不伪装为精确 manifold 接触点。
- 过滤矩阵继续统一覆盖 collision mask、sensor、excluded entity 与 required collision group；新增测试锁定 sensor 默认排除。
- Animation 的 `WeakBridge<dyn PhysicsQueryInterface>` 消费点完成 Vec 返回值迁移，不保留旧 Option 使用方式。

## 测试证据

| 验证 | 结果 |
|---|---|
| RED：`query_all_returns_distance_sorted_hits` | managed job `bc299dde7f724a978b7b772b50553b25` 按预期只因缺失 QueryMode/mode/Vec 契约失败 |
| GREEN：计划命名精确测试 | managed job `f7e5af4af2f74a8986c65dae6c53ec0b`，1/1 通过 |
| Windows feature-on Physics 全量 | managed job `1097ab1322664963a9d752a3eea20958`；library 27/27、integration 35/35、doc tests 0，合计 62/62 |
| 共享 Runtime/Animation 消费者及补充重建 | managed 尝试超过 124 秒、304 秒与 244 秒预算；当时同一工作树存在多个长期 rustc/cargo lane，记为基础设施超时，不认领通过，也不归因产品失败；接受证据仍以此前完成的 62/62 为准 |
| 格式与 diff | scoped rustfmt 通过；`git diff --check` 通过 |
| 插件结构审计 | `tools/audit_plugin_structure.py --json` 通过，manifest/capability/registration/distribution 等违规计数均为 0 |
| 计划产出审计 | 新增的 03 产出记录未触发违规；全仓仍报告 23 项其他计划族的既有/并发问题，本切片未越权修改 |

## 能力边界

- 当前 `joltc-sys 0.3.1` 的 JoltC C ABI 提供 narrow-phase ray cast，但未提供 shape-cast/collide-shape 入口。因此 manager 查询继续基于 backend-neutral 同步 collider 快照执行；本切片不声称原生 Jolt shape sweep。
- builtin sweep 返回的 `position` 是查询中心沿 sweep 路径的位置；非 box 凸形只保证保守命中排序语义，不保证精确接触流形。
- `PHYSICS_QUERY_INTERFACE_ID` 保持计划既有的 `physics.query.v1`。本切片按同一计划契约扩充查询模式，没有新增并行兼容接口或 facade。

## 后续

- M2 已完成 T1 形状族、T2 刚体策略、T3 QueryMode/sweep 三个计划切片。
- 下一切片进入 M3-T1：trigger enter/stay/exit 经 backend event buffer 与 `drain_events` 闭环；M3-T2 再接 Runtime event store 注册路径。
