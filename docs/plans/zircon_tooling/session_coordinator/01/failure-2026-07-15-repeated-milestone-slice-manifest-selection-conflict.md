---
handoff_kind: failure
status: open
created_at: 2026-07-15
summary_slug: repeated-milestone-slice-manifest-selection-conflict
origin_plan: docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md
origin_workflow_node: M4
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_plugins/12
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
related_code:
  - tools/session_coordinator/workflows/milestones.py
  - tools/session_coordinator/git_finalize.py
tests:
  - .\tools\zircon-session.ps1 -Json milestone validate --session-id plugins12-failure-priority-20260715 --run-id 8f51a0df781d414ca86220fc90cd5d2f --milestone M4 --template coordinator-actions
---

# Session Coordinator 01：重复里程碑编号错误选择历史切片 manifest

## 产出记录与时间

| 状态 | 记录日期 | 完成项目与当前门禁 |
|---|---|---|
| `OPEN / 待修复` | 2026-07-15 | Plugins12 当前 runtime event consumer 收口切片已完成受管产品门与独立复核，但 `milestone validate M4` 重新选择了同一编号下已提交或不属于当前切片的历史 M4 manifest，返回 `milestone_manifest_not_attributed`；共享暂存区保持 0，未绕过协调器。 |
| `OPEN / SAME-TOPOLOGY STATUS EDIT INVALIDATES SLICE EVIDENCE` | 2026-07-15 | Editor02 Runtime15 Failure return 只把 `02-data-sync-and-messaging.md` 的一行 `open` 链接改为 `fixed`；workflow topology hash 在 version 1/2 中均为 `67a355e8804dd6b4b6678c81cfe2e0470dc13b57fc34b47a527fde7a869b65a7`，milestones、slices 与 dependencies 完全相同。`topology.refresh` action `fcdc7a3b9a2b45b3922bae55b399f92c` 仍创建 version 2 `e88bff87699644c689f58bfc2be53ae4`，使 M1.3 的 immutable manifest、validation 与 review 留在 version 1 并全部显示 rejected。CLI 的 `milestone prepare --milestone M1.3` action payload 又只发送 `sessionId`，未传 node key，因而不能在新 version 上绑定所请求 slice。该复现证明冲突不仅是重复编号聚合，还包含 same-topology content churn 与 prepare identity 丢失。 |

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md`
- 来源执行切片：M4 runtime event consumer 与 linked runtime module 收口
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：当前切片的业务实现与测试归 Plugins12，但协调器将同一里程碑编号的历史切片 manifest 重新并入当前验证，最低共享原因位于 milestone manifest 选择和生命周期判定。

## 失败现象与复现证据

准备阶段生成 run `8f51a0df781d414ca86220fc90cd5d2f`（topology version `945c...`）。执行：

```powershell
.\tools\zircon-session.ps1 -Json milestone validate --session-id plugins12-failure-priority-20260715 --run-id 8f51a0df781d414ca86220fc90cd5d2f --milestone M4 --template coordinator-actions
```

验证未只读取当前 runtime event consumer 切片，而是重新选择了 Plugins12 既有 M4 记录，包括 first-party catalog M4、event-generation closeout 以及 `schedule_runner`、`derived_state` hierarchy/world-driver 等历史路径，最终返回 `milestone_manifest_not_attributed`。这些路径不是当前 Session 的 exact manifest，也不能通过扩大 attribution 合法吸收。

当前 Plugins12 exact manifest 另有独立复核发现的业务依赖边界问题，继续由 Plugins12/Editor03/Editor09 的有序提交处理；本 failure 只负责修复“同编号新切片错误重选历史 manifest”的协调器问题。

## 最低共享层根因

协调器以里程碑编号作为主要选择键，但没有在同一编号被多个时间切片复用时，将已完成、已提交或属于旧 topology 的 manifest 从当前 run 排除，也没有要求调用方显式绑定唯一 slice/manifest identity。因此新的 M4 prepare/validate 会重复装载历史 M4 业务路径。

当前 importer 还把 plan 全文 content hash 变化当作新 topology version，即使机器拓扑 hash
完全相同。Failure return、状态表或证据链接的普通文本更新因此切断当前 version 的 manifest、
validation 与 review 绑定。与此同时，CLI 接受 `--milestone` 却没有把该 node key 放进
`topology.refresh` action parameters；所谓 prepare 实际只刷新整份 plan，不能持久化调用方选择。

## 架构修复验收

- milestone prepare 产生并持久化唯一 slice/manifest identity；validate/commit 必须绑定该 identity，而不是按 `M4` 重新聚合所有同名历史记录。
- topology hash 未变化时不得创建新 version 或切断当前 manifest/evidence；plan 状态、Failure 链接和普通说明文字可更新 content metadata，但 workflow node identity 保持原 version。
- `milestone prepare --milestone <node>` 必须把规范化 node key 传入 typed action/service，并在响应中返回实际绑定的 topology version、node id 与 manifest hash；不得静默忽略参数。
- 已提交、已终结或属于旧 topology 的同编号 manifest 不得进入新 run；当前 Session 也不得通过 attribution 吸收历史业务路径。
- 以 Plugins12 runtime event consumer 切片重放：validate 只检查当前 exact manifest，并且不包含 first-party catalog、event-generation closeout、`schedule_runner` 或 `derived_state` 历史路径。
- 保持 current-hash attribution、独立复核、受管 staging 和 foreign staged isolation；修复后原始复现不再返回 `milestone_manifest_not_attributed`。

## 禁止临时方案

- 不得修改或删除历史 M4 manifest 来迎合当前 run。
- 不得扩大 Plugins12 attribution、合并 Editor03/Editor09 所有权文件、接受 degraded baseline 或手工暂存。
- 不得用 milestone 编号改名、兼容分支或调用方特判掩盖 manifest identity 缺失。

## 修复结果与回传

Open state: `待修复`; no pass is claimed.
