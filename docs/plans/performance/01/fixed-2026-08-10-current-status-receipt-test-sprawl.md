---
handoff_kind: fixed
status: fixed
created_at: 2026-08-02
summary_slug: current-status-receipt-test-sprawl
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_plugins/13
plan_link_mode: child_record_only
related_code:
  - tools/plugin_structure_audits
tests:
  - git ls-files tools/tests/test_plugin_docs_current_status*.py
  - python tools/tests/test_current_status_tests_do_not_assert_local_validation_receipts.py
  - python .codex/skills/zircon-project-skills/write-plan-output-records/scripts/audit_plan_output_records.py --repo-root E:/Git/ZirconEngine
resolved_at: 2026-08-10
---


# Plugins13：current-status receipt tests 膨胀并反向约束计划文案

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：2026-08-02 全仓计划、实现与过时测试审阅
- 修复责任计划：`docs/plans/zircon_plugins/13-standalone-plugin-build.md`
- 交接原因：这些测试围绕 Plugins09/13 的 standalone build、manifest schema、validator 与发布状态建立，最低共享 owner 是 Plugins13；Performance01 不应跨越当前大规模并发修改直接删改该测试族。

## 失败现象与复现证据

`git ls-files tools/tests/test_plugin_docs_current_status*.py` 在当前 tracked 基线返回 276 个文件。当前 worktree 中仍存在 272 个、合计 23,122 行，其中 246 个直接读取 `docs/plans`，260 个条目处于修改或删除状态。

抽样测试不是执行 plugin validator 或解析 fixture，而是要求多份计划/架构文档同时包含状态 ID、内部 helper 名、历史测试名乃至原始 Cargo 命令。例如 `test_plugin_docs_current_status_interfaces_gate.py` 在六个文档 section 中重复检查同一组短语；`test_plugin_docs_current_status_animation_runtime_helper_arc_import.py` 把一次 Arc import 修复及其 Cargo 命令固化为六份文档的测试契约。修改计划措辞会使 tooling 测试失败，但这些失败不能证明 manifest、build、export 或运行时行为回归。

现有 `test_current_status_tests_do_not_assert_local_validation_receipts.py` 只禁止少量已知命令/时间戳片段，因此它当前通过并不能证明该测试族已经摆脱 receipt coupling。

## 最低共享层根因

Plugins13 的逐切片验收记录被复制到多份长期文档，再用一个 Python 测试文件对应一个状态 ID 或 owner-split 事件进行反向锁定。`plugin_status_document.py` 甚至需要过滤 resolved output archive，说明测试事实源已经从生产 schema/validator 漂移到计划历史文本。

真正的 plugin manifest、build/export 和结构约束应由 validator、schema fixture 和生产 owner 测试证明；计划状态、failure/fixed 生命周期和 output-record 布局应由通用 plan tooling 验证，不能成为数百个 Plugins 专用 receipt tests。

## 架构修复验收

- 在保留当前 260 个并发修改语义后，分类并删除只检查状态 ID、命令回执、计划短语同步或历史 owner-split 叙述的 `test_plugin_docs_current_status*` 测试。
- 将仍有产品价值的 schema、manifest、validator、build/export 与 owner-boundary 断言迁入相应生产 validator 单元测试或结构审计 fixture；测试必须直接执行/解析权威实现或 fixture，而不是读取计划状态段。
- 当最后一个消费者退役后删除 `tools/tests/plugin_status_document.py`，不保留 archive-stripping 兼容层或改名后的 receipt facade。
- 运行 Plugins13 的实际 validator/schema/tooling 测试、plan-output audit 和 handoff audit，证明清理没有删除产品行为覆盖且计划措辞不再是测试事实源。

## 禁止临时方案

- 不通过扩充 allowlist、过滤更多 archive、复制状态短语或把文件改名来维持 receipt tests。
- 不删除真正执行 validator、解析 schema fixture、检查生产 owner 或验证发布行为的测试。
- 不覆盖当前 260 个并发修改条目；由 Plugins13 owner 在可审查的原子 hard cut 中完成归类和退役。

## 修复结果与回传

- 根因：Plugins13 duplicated implementation receipts and plan wording into 272 current-status test modules plus archive-expansion support, so historical prose became a second product contract.
- 架构修复：Hard-deleted all 272 receipt tests, five dedicated support modules, the archive-expansion facade and its unit test; retained production validator/schema/owner suites and replaced the old allowlist check with a hard-cut guard that rejects the entire retired family and facade.
- 验证：git ls-files current-status family: 0; hard-cut guard 1/1; plugin validate schema/semantics 185/185; owner/structure 9/9; real plugin validate --all target_count=41 failed_count=0 diagnostics=[]; plan-output audit passed; handoff validator 579 artifacts / 0 errors.
- 回传：Plugins13 current-status receipt-test sprawl is removed without deleting product behavior coverage; plan wording no longer controls plugin validator success.
