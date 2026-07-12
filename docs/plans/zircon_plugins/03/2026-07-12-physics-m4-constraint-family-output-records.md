# Physics M4 约束族产出记录

## 归属

- 父计划：`docs/plans/zircon_plugins/03-physics.md`
- 里程碑：M4-T1 `ConstraintDesc/JointParams + Joint 组件解析`；M4-T2 `六约束 Jolt 实现 + motor/limit`；M4-T3 `builtin Fixed/Distance 降级实现`
- 当前状态：`plugins_03_m4_constraint_family_complete`
- 记录日期：2026-07-12

## 完成项目

- 新增 plugin-owned `constraint/` 领域，`ConstraintDesc` 只携带 opaque body handle、双端 anchor、关节类型、`JointParams` 与 connected-collision 策略；Runtime 中立层不依赖具体 backend 对象。
- `JointParams` 定稿为 Fixed、Distance、Hinge、Slider、ConeTwist、Generic6Dof 六种变体；scene 的 legacy `limits` 与 per-axis `PhysicsJointConstraintMetadata` 被显式映射到对应 limit、motor、spring 与六自由度轴契约。
- `ConstraintDesc::from_joint_sync` 在 fixed-step 同步边界解析 joint owner entity 与 connected entity；缺失 body 返回 typed `InvalidDescriptor`，不 panic、不生成悬空 handle。
- `JoltManagedWorld` 在 body 创建后对账 constraint，先删除引用 stale/recreated body 的旧 constraint，再重建变更 joint；body 删除前保证 constraint 已销毁。
- Jolt backend 接收六种 constraint，并在每次原生 Jolt step 后读取 authoritative native body state、执行 plugin-owned constraint projection，再仅写回受约束 body 的变换/速度。
- builtin backend 只接受 Fixed 与 Distance，其他四种保持 typed `Unsupported`；constraint 引用中的 body 返回 `ObjectInUse`，避免 generation handle 被悬空复用。
- builtin Fixed 将 anchor 投影到另一 body/world；Distance 对 min/max 进行位置投影，并可使用 stiffness/damping spring 参数。非法、非有限、逆序 limit 与 self-link 均返回 typed descriptor error。
- `HandlePool::get_pair_mut` 使用分片借用一次解析两代 handle，既拒绝同槽 self-link，也避免 constraint solver 绕过 generation 检查。

## 计划命名测试

- `joint_resolves_entity_pair_to_handles`：同时覆盖 descriptor 解析和 Jolt managed-world 实体映射。
- `hinge_pendulum_period_matches_analytic`：固定步进下对比小角度摆周期容差。
- `slider_limit_clamps_travel`：验证 slider 线性范围夹取。
- `six_dof_swing_twist_respects_limits`：验证 Generic6Dof 三轴线性/角速度限制。
- `builtin_constraint_gap_is_a_typed_unsupported_error`：验证 builtin 非 Fixed/Distance 返回结构化 Unsupported。
- `builtin_fixed_and_distance_constraints_project_body_motion`：补充 builtin 两种降级实现的行为回归。

## 当前验证证据

| 验证 | 当前结果 |
|---|---|
| scoped rustfmt | 通过 |
| scoped `git diff --check` | 通过 |
| 新增 production owner 行数 | `constraint/projection.rs` 241；`constraint/params.rs` 183；`backend/builtin/constraint.rs` 106，均低于结构规范预算 |
| TDD 首个编译 RED | managed job `4a86ae613be342a5b21571c284ea4926` 到达 Physics crate，仅因测试先引用尚未补齐的 `collide_connected` accessor 失败；修正为 descriptor 字段断言 |
| Windows default Cargo GREEN | 最新 coordinator job `acb783ff253a48c48c6b776e809f355b`，精确 Physics 源码影子清单：library 23/23、integration 36/36，总计 59/59 |
| Windows `backend-jolt` Cargo GREEN | 最新 coordinator job `808fd60e6cca4b20b425b22cb205f872`，精确 Physics 源码影子清单：library 36/36、integration 37/37，总计 73/73 |

| 里程碑 | 测试阶段 | 状态 |
|---|---|---|
| M4 | M4-Testing：六约束族、builtin 降级、default/Jolt 回归 | 通过 |

补充：managed job `09040422f5bd42dcb623e882ac1e738d` 继续运行到 coordinator 因 wrapper 生命周期将其标记 orphaned，底层仍停留在 `zircon_runtime` link，未进入 Physics crate；该 attempt 不计通过或产品失败。共享工作区原计划命令 job `165db271eb56441bbb1bfbcfd1c61087` 在解析测试依赖前因其他会话同时修改的 `zircon_plugins/Cargo.toml` 与 `Cargo.lock` 不一致退出 101。为避免覆盖外部会话状态，本会话未更新锁文件，而是在 `E:\cargo-targets\plugins03-shadow-20260712` 建立仓库外清单，复制当前 Physics runtime/editor 精确源码并继续指向仓库内真实依赖；上述 59/59 与 73/73 均来自 coordinator 管理、D/E 盘 target 的该等价源码验证。

## JoltC ABI 边界

- 当前锁定的 `joltc-sys 0.3.1+Jolt-5.0.0` 不导出 Constraint 创建/销毁 C API，vendored JoltC header 中对应入口仍为注释状态。
- 因此本切片不虚构 native Jolt constraint object，也不修改 Cargo registry 或引入未受计划约束的 C++ ABI bridge。Jolt 仍负责 body 碰撞、重力与 solver step；constraint family 由 Jolt backend 内的 plugin-owned projection 层执行。
- 该实现保持同一 backend ownership、opaque handle、fixed-step 读写边界与 typed error 契约。若未来升级到导出 Constraint C API 的绑定，`PhysicsBackend::create_constraint` 和 `JointParams` 是唯一需要替换的 concrete seam。

## 验收结论

- 六个计划命名测试全部包含在 `backend-jolt` 73/73 通过结果中；default 59/59 同时证明 feature-off 编译与 builtin 降级契约。
- M4-T1、M4-T2、M4-T3 与 M4-Testing 均完成。JoltC ABI 限制保留为显式后端边界，不再作为本里程碑阻塞项。
