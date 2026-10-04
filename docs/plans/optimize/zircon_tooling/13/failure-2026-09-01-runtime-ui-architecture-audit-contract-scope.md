---
handoff_kind: failure
status: open
failure_scope: local
created_at: 2026-09-01
summary_slug: runtime-ui-architecture-audit-contract-scope
origin_plan: docs/plans/optimize/zircon_tooling/13-repository-codex-skill-hook-structural-audit-governance-security-currentness-review.md
fixing_plan: docs/plans/optimize/zircon_tooling/13-repository-codex-skill-hook-structural-audit-governance-security-currentness-review.md
origin_child_dir: docs/plans/optimize/zircon_tooling/13
fixing_child_dir: docs/plans/optimize/zircon_tooling/13
plan_link_mode: child_record_only
related_code:
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ui_architecture_boundary.py
  - tools/tests/test_runtime_ui_architecture_boundary.py
  - zircon_runtime/src/tests/runtime_absorption/ui_architecture/mirror_docs.rs
  - docs/crates/zircon_runtime/ui/architecture.md
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
  - docs/plans/zircon_runtime/runtime/index.md
  - docs/architecture/runtime-architecture-review-m0.md
  - docs/architecture/runtime-interface-convergence.md
  - docs/plans/zircon_runtime/runtime/09/2026-07-09-ui-subsystem-architecture-output-records.md
tests:
  - python -B -m unittest tools.tests.test_runtime_ui_architecture_boundary -v
  - python -B -m unittest discover -s tools/tests -p test_runtime_*.py
  - managed zircon_runtime lib test filter runtime_09_ui_architecture_mirror_docs_match_structure_audit_counts
---

# Tooling13: Runtime UI architecture audit contract scope

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_tooling/13-repository-codex-skill-hook-structural-audit-governance-security-currentness-review.md`
- 来源执行切片：Runtime UI architecture audit contract support scope
- 修复责任计划：`docs/plans/optimize/zircon_tooling/13-repository-codex-skill-hook-structural-audit-governance-security-currentness-review.md`
- 交接原因：原始跨计划 failure 的不可变 `related_code` 只登记审计脚本，但其验收同时要求测试、Runtime09 Rust 镜像守卫和 current-source 文档锚点原子一致。本记录为该完整合同声明 Tooling13 的精确范围，不改变上游 failure 的生命周期归属。

## 失败现象与复现证据

`python -B -m unittest tools.tests.test_runtime_ui_architecture_boundary -v` 的 RED
测试要求审计使用明确的 retired migration vocabulary，但当前实现没有
`LEGACY_MIGRATION_TERMS`、`_matching_term_line_count` 或
`_files_with_matching_term`，模块导入即失败。

原始审计还保留三项已删除的 template flat source、遗漏两个当前 `ui/` 顶层
owner，并把任意包含 `legacy` 的性能比较变量和说明文本都当成迁移债务。这样会把
`legacy_samples`、`legacy_p95_ns` 和 `legacy comparator` 等基准标签计入生产债务，
同时 Runtime09 的文档和 Rust 镜像守卫仍固定在旧 source/UI/surface/Taffy 数值。

## 最低共享层根因

最低共享层是 Tooling13 的结构审计指标合同：它以不受约束的 substring 代替已退役
UI API vocabulary，并在脚本、Python 测试、Rust 镜像和五份 current-source 文档间
复制数字而没有共同的语义边界。合法的 UI owner 与 Taffy 实现演进因此表现为基线
漂移，而新的性能比较代码又造成 legacy 假阳性。

## 架构修复验收

- source、`ui/` 和 `surface/` inventory 必须精确覆盖当前 owner，继续拒绝任意新增或缺失。
- legacy 指标只统计已声明的 retired UI migration vocabulary；同一行命中多个词只计一次，性能 comparator 标签不得命中。
- 生产 legacy 必须保持零；真实命中不得通过简单提升数字被隐藏。
- Taffy 指标继续统计生产所有权，其 current-source 行数和文件数与实际 owner 一致。
- Python 审计、Rust 镜像守卫和 Runtime09 current-source 文档必须同步同一组语义与数字。
- 精确 Python 测试、原始 static batch 和受管 Rust 镜像测试全部通过。

## 禁止临时方案

- 不得恢复已删除的 template flat 文件、放宽 exact inventory、忽略 unexpected entry 或删除失败断言。
- 不得把所有 `legacy` substring 的当前高值直接提升为绿色基线。
- 不得只改脚本而留下 Rust/doc 镜像漂移，也不得改写历史 dated snapshot 冒充 current truth。
- 不得使用 raw Cargo 或绕过 Coordinator 的构建与测试通道。

## 修复结果与回传

Open state: `guard repair implemented / complete union validation pending`.

## 2026-09-05 guard declaration repair

The audit previously accepted test names from documentation and from assertion strings in the
mirror test itself. Removing each of three actual Rust test declarations reproduced three false
GREEN reports. The audit now masks Rust comments/string literals with the existing lexical helper
and requires real `#[test] fn` declarations. The template guard follows the current
`runtime_74_template_boundary_has_one_compiler_authority` owner. The missing Cargo-gate declaration
is restored as a test of the six commands in the current Runtime09 plan. `mirror_docs.rs` now
checks the actual boundary owner and its 22/44 constants without including itself.

Current local Python validation passed 3/3, including removal of actual declarations while keeping
documentation, block comments and raw-string fake declarations. Exact Rust 1.94.1 rustfmt and
scoped diff checks passed. Independent review reported C0/I0/M0. Coordinator attribution request
`89c517202e1542feafc190e85395f81e` recorded the four source hashes; immutable focused regression
ticket `fba1848ba6c74d71b0858a2ff65b2805` was admitted by request
`051426c5b5484b7baecf9313cf4cecd3`.

This is an implemented guard repair, not full acceptance. The current full static batch has
31 failures and 18 errors across 1055 tests. The 242/16 Taffy baseline also depends on the separately
owned retained-parent `taffy_bridge/product_cache.rs` union, which is not integrated. Full Rust
mirror validation, atomic production/doc dependency closure and failure return remain open.

The subsequent source-inventory check found 48 declared paths behind an expected count of 49.
The actual mirror-doc guard is now the 49th required source, and the audit rejects a changed
materialized inventory count. Its removal regression was RED before the fix; the complete local
module now passes 4/4. Successor ticket `dabfc471dcba41eab6262e3350f0b3c5` seals these final source
hashes and the three focused regressions (request `e8b57784b67e4cdfb6429d13073dc902`). The earlier
ticket retains its earlier source revision and cannot certify this successor.
