---
handoff_kind: fixed
status: fixed
created_at: 2026-07-14
summary_slug: milestone-validation-copy-template-scope
origin_plan: docs/plans/zircon_plugins/08-zr-vm.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_plugins/08
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
related_code:
  - tools/session_coordinator/workspace_copy.py
  - tools/session_coordinator/workflows/milestones.py
  - tools/session_coordinator/control_plane/actions/executor.py
tests:
  - ".\\tools\\zircon-session.ps1 milestone validate --session-id plugins-08-zrvm-m1-20260714 --run-id 5a2cf030099a486bb61ce888630c2dd9 --milestone M1 --template coordinator-actions"
resolved_at: 2026-07-14
---


# Session Coordinator：里程碑验证副本缺少模板依赖

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/08-zr-vm.md`
- 来源执行切片：M1 服务化里程碑收口 / managed validation gate
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：失败发生在协调器对“不可变提交清单”和“验证模板运行源码”的边界建模，Plugins 08 无权通过扩大提交清单或修改模板依赖绕过。

## 失败现象与复现证据

M1 运行 `5a2cf030099a486bb61ce888630c2dd9` 的清单只包含两份当前归属且待提交的计划证据：

- `docs/plans/zircon_plugins/08-zr-vm.md`
- `docs/plans/zircon_plugins/08/2026-07-14-zr-vm-m1-output-records.md`

协调器成功创建验证副本 `3cd54adeea5b4031bfb7c9508f7f8e63`，随后以验证运行 `cecb8394d4ba485699f38b427c1c7235` 执行 `coordinator-actions`。副本只物化上述清单文件，却固定运行五个 `tools.session_coordinator.tests.*` 模块；五项均以 `ModuleNotFoundError: No module named 'tools'` 失败，exit code 为 1。`workflow_validation_bindings.terminal_code` 已记录为 `managed_validation_failed`。

## 最低共享层根因

验证副本把 milestone commit manifest 同时当作验证运行所需的完整源码闭包。普通引擎里程碑的 manifest 按门禁只能列出当前 Session 的 dirty/current-hash-attributed 文件，不能把未修改的协调器实现与测试伪装成待提交内容；但当前唯一通用收口流程又固定选择依赖这些文件的 `coordinator-actions` 模板。因此清单正确时模板必然缺依赖，扩大清单则违反精确提交与归属不变量。

## 架构修复验收

- 验证副本必须把不可变待提交 manifest 与只读验证基线/模板依赖分开建模；模板依赖不得自动进入 milestone commit manifest。
- 使用上述两文件清单重跑原命令，`tools.session_coordinator.tests.*` 可被导入并得到真实测试结果，不再出现 `ModuleNotFoundError`。
- 原 M1 managed validation gate 产生 terminal accepted evidence，且后续 commit 仍只允许精确 manifest 文件。

## 禁止临时方案

- 不得把未修改的 `tools/session_coordinator/**` 文件加入 Plugins 08 提交清单，或通过触碰文件制造 dirty attribution。
- 不得添加 `PYTHONPATH` 指向共享工作树、静默回退到非隔离执行、跳过模板、伪造 exit code、弱化 gate 或复制第二套提交范围真相。

## 修复结果与回传

- 根因：验证副本将精确的里程碑提交清单误当作模板运行所需的源码闭包，导致协调器测试包缺失。
- 架构修复：协调器将模板依赖根与提交清单分离：依赖以固定HEAD的只读归档物化，只有会话当前归属文件作为覆盖层和提交清单。
- 验证：Plugins 08 M1 managed validation b180fe15f1ae49c98688a89edef6aaef：coordinator-actions 24项通过、exit code 0、terminal managed_validation_succeeded。
- 回传：验证模板依赖与精确提交清单已分离，Plugins 08 M1 的受管验证已通过。
