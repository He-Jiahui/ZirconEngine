# Plugins 03 · Physics M2-T1 形状族产出记录

> 日期：2026-07-12
> 状态：`plugins_03_m2_t1_shape_family_windows_jolt_21_of_21_runtime_10_of_10_mesh_1_of_1_passed`
> 父计划：[`../03-physics.md`](../03-physics.md)

## 完成范围

- `PhysicsColliderShape` 与 scene/asset 对应形状补齐 `Cylinder`、`ConvexHull`、`TriangleMesh`、`HeightField`、递归 `Compound`。
- 共享 Runtime 消费者完成精确穷举：scene asset cache、scene project IO、属性投影与写入、render post-process 边界；Navigation bake 同步识别新增形状。
- 属性投影从 `entries/physics.rs` 拆出独立 `entries/collider_shape.rs`，避免继续向混合职责文件堆叠递归形状逻辑。
- builtin 后端为 TriangleMesh、HeightField、Compound 提供结构化 `Unsupported`，Cylinder 与 ConvexHull 保持可查询降级路径。
- Jolt 后端实现 Cylinder、ConvexHull、Compound 原生 shape；新增后端中立 `PhysicsMeshAsset`，注册后解析 TriangleMesh 与 HeightField，并对缺失、类型不匹配、非法索引/采样数返回 typed descriptor error。
- TriangleMesh/HeightField（包括 Compound 递归子形状）仅允许 static body，动态/运动学创建在进入原生后端前明确拒绝。
- 已返还跨计划 P0：[`../../zircon_editor/editor/01/fixed-2026-07-12-collider-shape-consumer-exhaustiveness.md`](../../zircon_editor/editor/01/fixed-2026-07-12-collider-shape-consumer-exhaustiveness.md)。

## 验证证据

- Windows managed Cargo check：`zircon_plugin_physics_runtime --features backend-jolt`，通过。
- Windows Physics 插件库测试：21 passed / 0 failed；覆盖计划命名的 convex-hull round-trip、builtin trimesh Unsupported，以及 Jolt mesh/heightfield 注册、静态体限制和既有 box-stack 确定性回归。
- Windows Runtime collider 消费者测试：10 passed / 0 failed；覆盖框架 JSON、项目 IO、属性读写/投影、render post-process 与 Navigation 兼容。
- 已构建 Runtime 测试宿主单项：`physics_mesh_asset_payloads_round_trip_json`，1 passed / 0 failed。
- `git diff --check` 对本切片代码范围通过。
- 新增/拆分生产文件行数：`mesh_shape.rs` 208、`conversion.rs` 268、`runtime.rs` 476、`entries/collider_shape.rs` 271、`entries/physics.rs` 241、`mesh_asset.rs` 17，均低于结构规范约 1000 行拆分阈值。
- 全仓 `audit_runtime_structure.py --json` 在当前并发超大工作树中两次超时（30 秒、120 秒）；该结果不记为通过，也不影响已经单独证明的本切片 owner 预算与 diff hygiene。

## 未认领范围

- M2-T2 MassProperties、CCD、SleepPolicy 与完整 BodyType 运行期切换。
- M2-T3 QueryMode 与 sweep 多命中排序。
- mesh 资产从通用资产管线自动注入 Jolt 注册表；M2-T1 交付的是稳定 DTO、显式注册/校验和原生映射边界。
- Linux/WSL 交叉平台验证；本切片没有出现必须转入 Linux 工具链的独占故障，按 Windows-first 验证策略未虚构 WSL 证据。
