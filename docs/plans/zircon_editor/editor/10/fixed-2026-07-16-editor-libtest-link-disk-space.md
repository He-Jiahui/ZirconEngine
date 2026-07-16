---
handoff_kind: fixed
status: fixed
created_at: 2026-07-11
summary_slug: editor-libtest-link-disk-space
origin_plan: docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
fixing_plan: docs/plans/zircon_runtime/runtime/01-tech-stack-and-dependency-governance.md
origin_child_dir: docs/plans/zircon_editor/editor/10
fixing_child_dir: docs/plans/zircon_runtime/runtime/01
related_code:
  - zircon_editor/src/core/project
  - zircon_editor/src/ui/host/startup
  - zircon_editor/src/ui/retained_host/app/welcome_session
plan_sources:
  - docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
  - docs/plans/zircon_runtime/runtime/01-tech-stack-and-dependency-governance.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - cargo build -p zircon_editor --locked
  - cargo test -p zircon_editor --locked
resolved_at: 2026-07-16
---


# Runtime 01：Editor lib-test 链接磁盘空间失败

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md`
- 来源执行切片：Plan10 M1.2 ProjectAuthority、共享模板与 `.zircon` 硬切测试阶段
- 修复责任计划：`docs/plans/zircon_runtime/runtime/01-tech-stack-and-dependency-governance.md`
- 交接原因：生产 build 已通过，lib-test 在 Windows 链接阶段因验证盘空间不足失败，测试二进制未生成，Plan10 行为断言未执行。

## 失败现象与复现证据

2026-07-11 受管 `validate-matrix.ps1 -Package zircon_editor` 使用 coordinator lane `E:\targets\zircon-engine\lanes\test-5faee3338e0b44738c1836487d9e8e4c`。执行前 E 盘仅余 27.19 GB，低于 50 GB 清理阈值；`cargo clean` 成功，`cargo build -p zircon_editor --locked` 成功，`cargo test -p zircon_editor --locked` 在生成 `zircon_editor-e886edbd9b5dcf71.exe` 时由 MSVC linker 报 `LNK1180: 没有足够的磁盘空间完成链接`。因此没有任何 lib-test 运行结果，不能记为代码行为失败或通过。

2026-07-11 后续协调服务识别出 6 个过期、无活动租约的受管 lane，但 `cleanup apply` 被 `maintenance_unauthorized` 拒绝；本会话未绕过维护权限手工删除目录。该证据进一步确认空间恢复归验证环境治理，而非 Editor 10 业务实现。

## 最低共享层根因

最低失败层是 Runtime 01 管辖的受管 Cargo 验证容量与清理权限，不是 ProjectAuthority、模板包、Hub Summary 或 `.zircon` 路径逻辑。Editor lib-test 链接需要同时容纳大型 runtime/editor 对象、PDB 与最终测试二进制；当前可用空间不足以完成该阶段。

## 架构修复验收

- 在不删除用户工作树内容的前提下释放或提供至少满足仓库 50 GB 门槛的受管验证空间。
- 继续使用 coordinator 管理的 cargo lane，不回退到仓库 `target/` 或未登记的临时目录。
- 复跑 `cargo build -p zircon_editor --locked` 与 `cargo test -p zircon_editor --locked`；只有测试进程实际执行并通过，才能关闭 Plan10 M1.2 Editor gate。
- 若复跑出现代码编译/行为失败，按最低功能 owner 新建或更新对应失败交接，不得把新的代码失败混入本磁盘单。

## 禁止临时方案

- 禁止删除、跳过或弱化 ProjectAuthority/事务/路径安全测试以缩小二进制。
- 禁止把 production build 通过冒充 lib-test 通过。
- 禁止清理用户未授权的工作树、示例源码、文档证据或其他会话产物。
- 禁止绕过 coordinator 直接使用不受管 Cargo target。

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| Runtime 01 / Editor 10 | M1.2 Editor build/lib-test 验证环境 | `生产构建通过-libtest链接被磁盘空间阻断` | 2026-07-11 | 受管 lane build exit 0；test link `LNK1180`，E 盘执行前 27.19 GB，测试未执行；安全 cleanup plan 有 6 个候选，但 apply 被维护权限拒绝。 |

## 修复结果与回传

- 根因：The original Editor lib-test linker lane had only 27.19 GB free, below the repository 50 GB managed-target threshold, so MSVC terminated with LNK1180 before producing the test executable.
- 架构修复：Coordinator-governed Windows Cargo target allocation and retained target-pool cleanup restored sufficient managed validation capacity without using repository target directories or deleting user worktree data.
- 验证：Managed Editor validation later linked and launched the zircon_editor lib-test process (job 5ac5), proving the LNK1180 capacity failure is gone; execution then stopped on the separately owned Plugins12 WorkbenchShellState self-deadlock, so Editor10 business tests remain unaccepted.
- 回传：Return the environment failure as fixed: managed linking capacity is restored. Do not mark Editor10 M1.2 passed; its current upper-layer blocker is the separately tracked Plugins12 shell self-deadlock.
