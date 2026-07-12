# Physics M5 Ragdoll 产出记录

## 归属

- 父计划：`docs/plans/zircon_plugins/03-physics.md`
- 里程碑：M5-T1 `RagdollProfile`；M5-T2 runtime 生成与 mode 切换；M5-T3 pose handoff
- 当前状态：`plugins_03_m5_ragdoll_complete`
- 记录日期：2026-07-12

## 完成项目

- 新增 plugin-owned `skeletal/` 领域，`RagdollProfile` 严格解析 `.ragdoll.toml`，并在任何世界变更前验证 profile id、骨骼路径唯一性、父子拓扑、质量、shape、body offset 与混合权重。
- `RagdollRuntime::spawn_configured` 采用准备/提交两阶段流程，按 profile 生成刚体、碰撞体、Generic6Dof joint 与 skeleton binding；任何准备失败均不留下半生成实体。
- Animated、Simulated、Blended 三种 mode 有明确固定阶段语义。Animated 在 FixedUpdate 使用上一帧动画目标驱动物理体；首次切入物理模式时只注入一次最近有限释放速度，避免姿态跳变和持续覆盖 solver 速度。
- FixedPostUpdate 从同步后的物理体移除 body offset，重建 parent-local bone transform，再按 mode、profile mask 与 interpolation weight 合成 `SimulatedPoseFeed`，供下一帧 Animation 消费。
- `SkeletalPoseTargets`、`SimulatedPoseFeed` 与 `RagdollRuntime` 由 Physics runtime module 注册，Runtime 核心只保留中立姿态交换协议。

## 计划命名测试与验证

- `ragdoll_profile_spawns_expected_body_count`：覆盖 profile 生成数量、拓扑与事务回滚。
- `ragdoll_drop_golden_snapshot`：覆盖完整模拟姿态黄金快照。
- `animated_to_simulated_switch_has_no_pose_pop`：覆盖释放速度继承与 mode 首帧连续性。
- blended 权重与父子环检测补充测试覆盖局部姿态混合和无效 profile 拒绝。
- Windows 精确源码 `backend-jolt` managed Cargo 最新验证 job `808fd60e6cca4b20b425b22cb205f872` 通过 library 36/36、integration 37/37；M5 全部行为测试包含其中。此前 Windows nightly focused behavior 亦通过 5/5。
- scoped rustfmt、`git diff --check` 与生产文件行数预算通过。

| 里程碑 | 测试阶段 | 状态 |
|---|---|---|
| M5 | M5-Testing：profile、生成、mode 切换、姿态交接 | 通过 |

## 验收结论

- M5-T1、M5-T2、M5-T3 与 M5-Testing 均完成。
- ragdoll 编排完全位于 Physics 插件，未回流 `zircon_runtime::physics`，符合插件边界与大文件拆分规范。
