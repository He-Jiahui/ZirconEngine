---
handoff_kind: failure
status: open
created_at: 2026-09-24
summary_slug: scene-mode-test-owner-budget
origin_plan: docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md
fixing_plan: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
origin_child_dir: docs/plans/zircon_editor/editor_ui/10
fixing_child_dir: docs/plans/zircon_editor/editor/05
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/scene/modes/tests.rs
  - zircon_editor/src/scene/modes/tests/registry_contract.rs
  - zircon_editor/src/scene/modes/tests/command_projection.rs
tests:
  - python -B -m unittest tools.tests.test_editorui10_test_file_budget_contract -v
  - python -B .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_editor_structure.py --json --repo-root E:\Git\ZirconEngine
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_editor -SkipBuild -LibTests -TestFilter scene::modes -TestThreads 1 -VerboseOutput
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_editor -SkipBuild -LibTests -TestFilter structure_convention -TestThreads 1 -VerboseOutput
---

# Editor05：SceneMode 测试 owner 超过 800 行预算

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md`
- 来源执行切片：M3.T1 测试文件零容忍结构门禁。
- 修复责任计划：`docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md`
- 交接原因：`scene/modes/tests.rs` 的模式栈、注册表与命令投影测试属于 Editor05 行为 owner；EditorUI10 只拥有结构审计和最终汇总验收。
- 上游 EditorUI10 failure：`docs/plans/zircon_editor/editor_ui/10/failure-2026-08-13-editor-test-file-budget-gate-missing.md`；本记录只处理它报告的一个文件，不替其他功能 owner 声明全仓门禁通过。

## 失败现象与复现证据

2026-09-24 拆分前，当前源码的 `zircon_editor/src/scene/modes/tests.rs` 为 949 行，超过 EditorUI10 的 800 行测试文件预算；结构审计报告 12 个超限测试文件（另有 3 个超限生产文件）。原文件直接定义 18 个测试，其中 3 个属于注册表合同，2 个属于命令投影合同。

`failure-roll-01a084c8-editor05-scene-mode-test-budget-r2` 已通过协调器从旧 Editor05 source owner 接管原文件精确租约，并申领两个新子文件及本 handoff；修改前确认原文件与 Git HEAD 一致，新文件原先不存在。未接管其他 Editor05 既有 failure 或外部会话的源码。

## 最低共享层根因

不同场景的测试持续堆放在单个模式栈测试 owner 内，而不是按已存在的 `scene/modes/tests/` 文件夹拆分。注册表与命令投影测试自身的断言不需要变化；在本测试模块声明两个语义子模块，原 helper 仍由父模块共享即可消除这个 owner 的结构超限。

## 架构修复验收

- 将 3 个注册表测试移入 `tests/registry_contract.rs`，2 个命令投影测试移入 `tests/command_projection.rs`；不复制旧测试、不改断言，不留兼容入口。总测试数仍为 18，父文件及两个子文件各不超过 800 行。
- 原始 EditorUI10 Python 合同实际执行，结构审计中本路径退出 `oversized_test_files`；保留余下未修复路径以及 `editor_ui_10_no_oversized_test_files` 零容忍断言，不把仍红的全局 gate 误报为绿。
- 外部 `E:\Git\zr_vm` 源码恢复到可验证快照后，按协调器 Windows 合规 target 执行 `--locked` 的 Editor05 场景测试和 EditorUI10 `structure_convention` 上游验收；精确过滤必须实际运行目标测试，并保留源码快照、票据、独立 C/I/M=0/0/0 审查和回传。

## 禁止临时方案

- 不提高预算、不添加豁免、不修改审计逻辑或生产行为；不删除测试或留下同名测试的双重树。
- 静态结构通过、rustfmt 通过和协调器排队回执都不能代替受管 Rust 动态测试；本项未完整验收前保持 open。

## 修复结果与回传

2026-09-24 scoped source split 已写入：原文件移出 5 个原样测试，只添加两个 `mod` 声明，静态审计超限测试文件数从 12 降到 11 且本路径不再超限；Python 上游合同实际运行 5/5 通过，三个精确源文件 rustfmt `--check` 通过。独立审查发现原 `tests` 字段写的是不可执行的受管验收说明，现已修正为实际 `validate-matrix.ps1` 命令；针对源码快照 `3761` 与修正后 handoff 哈希 `5371901083447e095af75d0f3c6c8265a9fa85f639ea69fff9531a53dd18a319` 的只读复审，Critical/Important/Moderate 为 `0/0/0`，未运行 Cargo。原 EditorUI10 全局零容忍门禁仍因其他 11 个文件失败，且 `E:\Git\zr_vm` 工作树含非本会话改动，不能启动有正确依赖快照的受管 Cargo 验收。本 failure 保持 `open / Rust validation pending / final review pending`，不声明 fixed、return、closeout 或企微通知完成。
