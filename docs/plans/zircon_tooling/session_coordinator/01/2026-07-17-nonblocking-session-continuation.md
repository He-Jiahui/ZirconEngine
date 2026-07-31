---
record_kind: implementation_slice
status: accepted
created_at: 2026-07-17
plan_source: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
related_code:
  - tools/session_coordinator/work_continuations.py
  - tools/session_coordinator/control_plane/snapshot.py
  - tools/session_coordinator/web/src/pages/OverviewPage.tsx
  - docs/tools/session_coordinator/validation-queue.md
  - docs/tools/session_coordinator/control-plane.md
tests:
  - python -m unittest tools.session_coordinator.tests.test_control_snapshot
  - npm --prefix tools/session_coordinator/web run check
  - controlled service.rollover in a natural no-managed-Cargo window
---

# Coordinator01：验证等待时的同计划续作

## 验收结论

验证与租约等待已变成局部、只读的同计划续作建议：每次只给出一个
可信编号计划中尚未完成的实现或文档切片。它不占用 Cargo、不修改队列、
不抢外部租约，也不创建跨计划 WIP。完成该切片后，Session 优先回到
主里程碑；只有没有可做代码或验证已终结时，才处理验证队列。

## 运行证据

- 快照回归 13/13 通过；等待 Session 只获得一个受限候选，测试项和
  非编号/不可信计划不会进入浏览器工作建议。
- 前端完整检查通过：类型检查、56/56 测试、生产构建和 27 项哈希资源校验。
- 在自然无受管 Cargo 窗口，动作 `8b9df8…` 触发一次 rollover；连接中断是
  预期服务切换行为，后继 `1e3d…` 随后以 `read_write` 和健康监督状态恢复，
  `maintenanceHold=false`。
- 实时控制面包含 `experience.continuations`，当前没有等待 Session 时该数组为空；
  已发布 Overview bundle 包含“验证等待时的续作”。

## 边界

- 续作投影只读且有数量、计划大小与文本长度上限；执行者仍须先领取自己的具体作用域。
- 任何运行中的受管 Cargo 均禁止 rollover；本次没有排空、停止或释放外部作业。
- 该机制只降低局部等待的空转，不放宽验证、租约、FIFO 或来源计划的验收责任。
