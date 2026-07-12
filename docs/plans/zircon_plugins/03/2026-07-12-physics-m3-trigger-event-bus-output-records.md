# Physics M3 Trigger 与事件总线产出记录

## 归属

- 父计划：`docs/plans/zircon_plugins/03-physics.md`
- 里程碑：M3-T1 `trigger enter/stay/exit 经 drain_events 闭环`；M3-T2 `ContactEvent/TriggerEvent 接 register_event`
- 状态：`plugins_03_m3_trigger_event_bus_windows_feature_66_of_66_passed`
- 完成日期：2026-07-12

## 完成项目

- Builtin backend 保持每 world 的有序 trigger pair map，在每次真实 step 后生成 `Enter/Stay/Exit`，并通过 `PhysicsEventBuffer::drain_events` 一次性移交 contact 与 trigger 批次。
- Jolt backend 接入相同的中立事件输出边界：原生 step/readback 后从其权威 body/collider 快照更新 contact 与 trigger pair 生命周期，再由 managed world drain；未回落调用 builtin step，也未把事件状态放入 Runtime 中立契约。
- Physics runtime module 注册 `PhysicsContactEvent` 与 `PhysicsTriggerEvent`，事件 id 分别为 `physics.events.contact` 与 `physics.events.trigger`，schema 分别为 `physics.contact_event.v1` 与 `physics.trigger_event.v1`。
- `physics.step` 将 manager 本帧 drain 的两类事件逐项写入 scene `EventStore`，同时保留原有 `LevelSystem` 当帧诊断快照；插件消费者可通过普通 event subscription 读取。
- sensor pair 继续复用双向 collision mask、collision matrix 与形状 overlap 规则；双 sensor 对会按各自 trigger owner 生成两条有序生命周期事件。

## 测试证据

| 验证 | 结果 |
|---|---|
| 初始 managed RED 尝试 | job `53013e85d4214690b3ece89a297b251b` 在共享编译压力下超过 304 秒预算，未得到测试断言结果，按基础设施超时记录 |
| Windows default feature | managed job 对应 `physics-m3-event-green`；library 20/20、integration 36/36，合计 56/56 |
| 首次 Windows feature-on | managed Jolt 编译到本切片代码并暴露 `synchronize_jolt_world` 返回类型不一致，作为有效编译 RED；修复单一 tuple 返回边界后重跑 |
| Windows feature-on GREEN | managed `physics-m3-final`；library 29/29、integration 37/37、doc tests 0，合计 66/66 |
| 计划命名生命周期测试 | `trigger_lifecycle_enter_stay_exit_contract` 通过 |
| EventStore 测试 | `physics_contact_and_trigger_events_reach_event_store` 通过 |
| Backend drain 测试 | `builtin_backend_drain_events_reports_trigger_enter_stay_exit` 与 `jolt_drain_events_reports_trigger_lifecycle` 均通过 |
| 结构/静态检查 | plugin structure audit 违规 0；scoped rustfmt 与 `git diff --check` 通过 |

## 能力边界

- 当前 JoltC 绑定未暴露原生 ContactListener 回调。本切片在 Jolt 原生 solver step/readback 后，使用 Jolt backend 自己持有的权威同步 collider 描述计算事件生命周期；这是 backend 内部实现，不是 builtin step 降级，也不声称原生 manifold 回调精度。
- `LevelSystem::physics_contacts/physics_triggers` 仍是当帧诊断快照；跨系统消费的正式路径是注册后的 scene EventStore。

## 后续

- M3-T1/T2 已完成，下一计划切片进入 M4-T1：`ConstraintDesc/JointParams + Joint 组件解析`，随后实现六约束 Jolt motor/limit 与 builtin Fixed/Distance 降级。
