---
handoff_kind: failure
status: open
created_at: 2026-07-15
summary_slug: live-ephemeral-target-misclassified-unmanaged
origin_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_plugins/13
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
related_code:
  - tools/session_coordinator/artifact_governance.py
  - tools/session_coordinator/cargo_jobs.py
  - tools/session_coordinator/server.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_artifact_governance -v
  - powershell -File tools/zircon-session.ps1 -Json cargo acquire test --session-id plugins13-vg-runtime-support-workload-fix-20260715 --pid $PID
---

# Coordinator01：活跃受管 ephemeral target 被误判为 unmanaged

## 产出记录与时间

| 状态 | 记录日期 | 完成项目与当前门禁 |
|---|---|---|
| `OPEN / 待修复` | 2026-07-15 | Plugins13 准备 fresh Virtual Geometry integration gate 时，一次受管 `cargo acquire` 在其他已注册 ephemeral job 仍处于运行/回收生命周期期间返回 `unmanaged_artifacts_detected`，路径错误收敛到 `D:\cargo-targets\zircon-engine\ephemeral` 父目录。未执行手工删除；外部作业终结后，同一 Session 的 dry-run acquire `80ede85841604733b12a46522036a837` 成功并已受管 release，证明这是并发时序门禁而不是 Plugins13 业务失败。 |

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/13-standalone-plugin-build.md`
- 来源执行者：`plugins13-vg-runtime-support-workload-fix-20260715`
- 来源执行切片：Virtual Geometry Runtime support compute-workload failure fresh retest
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：D/E/F artifact 扫描、Cargo job 注册和 ephemeral cleanup 的并发一致性由协调器拥有；Plugins13 不得删除受管 target、修改协调器数据库或绕过 acquire。

## 失败现象与复现证据

Plugins13 首次通过协调器申请兼容池时，`cargo.acquire` 在业务命令启动前返回：

```text
unmanaged_artifacts_detected
D:\cargo-targets\zircon-engine\ephemeral
```

当时 `plugins05-control-prop-binding-ref-20260715` 与
`plugins05-navigation-selection-20260715` 的 ephemeral jobs 正处于受管执行或终结回收窗口；这些
job 的 target 均位于该父目录下，且最终由协调器 finish/release 并删除。来源 Session 没有执行
`artifact.cleanup`、`Remove-Item` 或任何手工 target 清理。外部 job 状态收敛后，dry-run acquire
`80ede85841604733b12a46522036a837` 成功并受管 release；因此只确认一次真实并发误判，不把后续
成功重试夸大为第二次复现。

## 最低共享层根因

`cargo.acquire` 先调用 `ArtifactGovernanceService.require_clean()`，后台 maintenance 同时也可调用
`cleanup()`；scan 则从 Cargo job 数据库快照推导 managed leaf，再递归分类父目录。当前实现缺少
一项覆盖“同一 ephemeral 父目录下存在活跃/正在终结的受管 leaf，同时另一个 Session acquire”
的原子并发合同。实际运行已证明某个时序窗口会把父目录本身报告为 unmanaged。精确竞态点仍需
由 Coordinator01 通过可控并发回归定位，不能由来源计划猜测后删除目录。

## 架构修复验收

- 为 artifact scan/cleanup 与 `cargo.acquire` 建立一致的 service-level reservation 或原子快照边界；只要父目录下存在 leased/running、尚未完成受管 cleanup，或正由 acquire 注册的 target，就不得把该父目录或其祖先分类为 unmanaged。
- 新增确定性并发回归：Session A 在 `.../ephemeral/test/<job-a>` 运行或 release-cleanup 窗口时，Session B acquire 必须成功，且 scan/cleanup 不得返回或删除 `.../ephemeral`、`.../ephemeral/test`、`<job-a>`。
- 仍须识别同级真实手工目录；不得因一个受管 descendant 永久豁免整个 ephemeral 树。
- 失败响应需返回发生分类时的 job id/status/target snapshot，便于审计竞态而不依赖人工猜测。
- 修复后由 Plugins13 重跑同类 acquire 和 fresh Virtual Geometry focused gates；不得要求来源 Session 手工等待、删除目录或停掉其他合法 Cargo job。

## 禁止临时方案

- 不得手工删除 `D:\cargo-targets\zircon-engine\ephemeral` 或任何其他 Session target。
- 不得关闭 unmanaged artifact governance、把整个 ephemeral 根永久加入白名单，或忽略所有祖先目录。
- 不得通过 raw Cargo、固定睡眠、无限重试或直接改 SQLite 绕过协调器。
- 不得将本并发门禁计入 Plugins13 业务实现通过，也不得修改 Shader04 文件。

## 修复结果与回传

Open state: `待 Coordinator01 修复并回传`；一次后续 dry-run 成功只说明竞态窗口已消失，不构成根因修复。
