---
handoff_kind: failure
status: open
failure_scope: local
created_at: 2026-09-01
summary_slug: runtime-ui-architecture-rust-guard-contract
origin_plan: docs/plans/optimize/zircon_tooling/13-repository-codex-skill-hook-structural-audit-governance-security-currentness-review.md
fixing_plan: docs/plans/optimize/zircon_tooling/13-repository-codex-skill-hook-structural-audit-governance-security-currentness-review.md
origin_child_dir: docs/plans/optimize/zircon_tooling/13
fixing_child_dir: docs/plans/optimize/zircon_tooling/13
plan_link_mode: child_record_only
related_code:
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ui_architecture_boundary.py
  - tools/tests/test_runtime_ui_architecture_boundary.py
  - zircon_runtime/src/tests/runtime_absorption/ui_architecture/architecture_boundaries.rs
tests:
  - python -B -m unittest tools.tests.test_runtime_ui_architecture_boundary -v
  - managed zircon_runtime lib test filter runtime_09_ui_architecture_doc_records_current_boundaries
  - managed zircon_runtime lib test filter runtime_09_ui_architecture_baselines_match_current_source_scan
---

# Tooling13: Runtime UI architecture Rust guard contract

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_tooling/13-repository-codex-skill-hook-structural-audit-governance-security-currentness-review.md`
- 来源执行切片：Runtime UI architecture audit independent review
- 修复责任计划：`docs/plans/optimize/zircon_tooling/13-repository-codex-skill-hook-structural-audit-governance-security-currentness-review.md`
- 交接原因：第一份本地支持 failure 的不可变范围包含 Python audit、mirror guard 和 current-source 文档，但遗漏了同时执行相同 source scan 的 `architecture_boundaries.rs`；本记录为该 Rust guard 扩围保留独立、可审计的 Tooling13 责任。

## 失败现象与复现证据

独立 review 证明 `runtime_09_ui_architecture_doc_records_current_boundaries` 仍断言
`ui=20`、`surface=26`，而当前精确 owner map 是 22/44；
`runtime_09_ui_architecture_baselines_match_current_source_scan` 仍以 raw `legacy`
substring 和旧 70/175/10 基线扫描，与 Python audit 的收紧语义不一致。

同一 review 还定位了七条 property 性能 comparator 假阳性：六条
`legacy_properties` 基准数据引用和一条
`legacy_property_resolutions_per_rename` 输出标签。它们不是 retired API debt，不能计入
general migration metric；真实 property cutover 已由 `legacy_renames.rs` 的精确 owner guard
保护。

## 最低共享层根因

Rust current-boundary guard 与 Python audit 复制了两套 inventory 和 source-scan 语义，
但 failure 范围只同步了 mirror guard。raw substring 又无法区分 retired identifier 与
性能对照标签，导致局部 GREEN 无法代表完整 Runtime09 static contract。

## 架构修复验收

- Rust current-boundary guard 必须同步精确 22/44 owner 数量与新增 owner 名称。
- Python 与 Rust general legacy metric 只统计相同的无歧义 retired migration vocabulary；同一行命中多个词只计一次。
- `legacy_properties` 和 `legacy_property_resolutions_per_rename` 等真实性能 comparator 不得命中；其生产 owner cutover 继续由精确 Rust owner test 保护。
- Runtime09 source-scan 基线统一为 legacy 15/0、Taffy 217/16，文档和两个 Rust tests 同步通过。

## 禁止临时方案

- 不得删除 Rust guard、只过滤 mirror test、恢复旧 owner 数量或把 raw `legacy` 高值直接提升为基线。
- 不得删除 property owner-specific cutover tests 来消除 comparator 冲突。
- 不得在同一行逐 term 累加，也不得用文件名硬编码跳过某个现有 benchmark fixture。
- 不得使用 raw Cargo。

## 修复结果与回传

Open state: `review RED captured / implementation updated / validation pending`.

## 2026-09-05 executable guard repair

The sibling audit-contract failure records the shared repair and immutable focused ticket
`dabfc471dcba41eab6262e3350f0b3c5`. The Rust boundary now uses named 22/44 inventory constants;
the mirror reads that owner directly instead of satisfying assertions from its own source. The
six-command Cargo-gate test is executable again, and the Python inventory requires masked Rust
test declarations rather than documentation mentions. The actual 49-source inventory is now
checked as well. Local Python 4/4, exact rustfmt and
independent C0/I0/M0 review passed.

The working Taffy scan is 242/16, beyond the historical 217/16 recorded above. That change belongs
to the retained-parent production union and must be integrated as its own dependency; this repair
does not certify or absorb it. The complete static batch and managed Rust boundary/scan tests are
still required, so both local failures remain open and no fixed return is claimed.
