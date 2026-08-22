---
handoff_kind: fixed
status: fixed
created_at: 2026-08-22
summary_slug: non-network-server-context-classification-drift
origin_plan: docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
fixing_plan: docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/15
plan_link_mode: child_record_only
related_code:
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/non_network_server_naming.py
  - tools/tests/test_non_network_server_naming.py
tests:
  - python -B -m unittest tools.tests.test_non_network_server_naming -v
  - python -B .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py --json
resolved_at: 2026-08-23
---


# Runtime15: non-network server context classification drift

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md`
- 来源执行切片：Frameworks01 current-source structure and algorithm re-audit
- 修复责任计划：`docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md`
- 交接原因：最低共享原因位于 Runtime15 的 module-convention 审计算法，而不是任一被扫描
  consumer。Frameworks01 不应通过批量改名掩盖审计器把合法 server runtime 语义识别为债务的问题。

## 失败现象与复现证据

`audit_runtime_structure.py --json` 在 current HEAD
`a922089697e41e07fa29e3e42a5e4c9afc1ae31b` 报告 module-convention migration debt。
扩展到 16,883 个 tracked `zircon_*/src/**/*.rs` 的诊断扫描后，non-network server gate 产生
869 个未分类位置：650 个来自
`tests`、`tests.rs`、`*_tests.rs` 或 `test_sources`，其余 219 个中又有 201 个位于 render plugin
`test_sources`。排除测试 owner 后的 18 个 production 位置全部是 UNC placeholder、canonical
`server_runtime` capability、server export profile 或 server/headless runtime policy。

## 最低共享层根因

`non_network_server_references` 扫描了 test-owned Rust source，却没有像 hard-cutover audit 一样
建立 production/test 边界；其 allowed-context 判定同时漏掉带下划线的 `server_runtime`、通用 UNC
placeholder、export validation 的 canonical server profile 和 `server or headless` policy 文案。
因此 gate 把测试 fixture 与合法 production policy 混入真实非网络 `*_server` 命名债务。

## 架构修复验收

- test-owned source 必须从 production migration debt 中排除，并以独立计数保持审计可见性。
- UNC、canonical `server_runtime`、server export profile 与 server/headless policy 必须明确分类为合法语义。
- 任意 production `render_server` 仍必须进入 unclassified debt；即使同一行还含 UNC、
  `server_runtime`、export profile 或 server/headless policy，也只能放行匹配合法语义 span 的
  token，禁止整行豁免隐藏真实 owner 问题。
- 聚焦 Python 回归、完整 runtime structure audit 与 handoff audit 必须通过；完整审计不得再由
  test-owned 或上述合法 production server context 产生 non-network migration debt。

## 禁止临时方案

- 不得批量改写 869 个 consumer 来迎合错误分类。
- 不得增加兼容 alias、静默 fallback、全局 `server` 豁免、测试专用 bypass 或调用点例外。
- 不得削弱 production 任意 `*_server` 的负例或计划验收标准。

## 修复结果与回传

- 根因：The Runtime15 non-network server audit mixed test-owned source with production and used line-wide allowances and owner classification, so legal runtime server contexts created false debt while mixed render_server tokens could be hidden.
- 架构修复：Separate test-owned source accounting from production debt; classify allowed contexts and scene comments by token span; keep render_server globally unclassified; use asset-owner paths only as remaining-token fallback; define count as unique path-line locations while preserving decision-group counts; project multi-decision samples with plural owner and action arrays.
- 验证：Independent final review Critical 0 Important 0 Moderate 0 Minor 0 Ready to merge Yes; focused unittest 7 of 7, AST 2 of 2 and scoped diff-check green; tracked-current 16878 Rust sources in 24.153 seconds and full runtime structure audit in 235.959 seconds both report zero non-network source locations, decisions, classifications, unclassified references and migration debt.
- 回传：Return the corrected Runtime15 audit gate to Frameworks01; no consumer renames or compatibility facade were introduced, and the remaining 15 Runtime15 structure debt groups stay with their existing owners.
