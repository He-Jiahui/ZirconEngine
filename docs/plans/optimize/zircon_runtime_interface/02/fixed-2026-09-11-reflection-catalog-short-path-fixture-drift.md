---
handoff_kind: fixed
failure_scope: local
status: fixed
created_at: 2026-09-08
summary_slug: reflection-catalog-short-path-fixture-drift
origin_plan: docs/plans/optimize/zircon_runtime_interface/02-serialization-reflection-resource-project-world-sync-public-dto-contract-review.md
fixing_plan: docs/plans/optimize/zircon_runtime_interface/02-serialization-reflection-resource-project-world-sync-public-dto-contract-review.md
origin_child_dir: docs/plans/optimize/zircon_runtime_interface/02
fixing_child_dir: docs/plans/optimize/zircon_runtime_interface/02
plan_link_mode: child_record_only
related_code:
  - zircon_runtime_interface/src/reflect/schema_catalog/tests.rs
tests:
  - cargo test -p zircon_runtime_interface --lib --no-default-features --locked reflect::schema_catalog::tests::
  - cargo test -p zircon_runtime_interface --lib --no-default-features --locked tests::reflect
resolved_at: 2026-09-11
---

# Interface02: schema catalog fixture violates canonical short paths

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_runtime_interface/02-serialization-reflection-resource-project-world-sync-public-dto-contract-review.md`
- 来源执行切片：Full Interface library regression after project fixture closure.
- 修复责任计划：`docs/plans/optimize/zircon_runtime_interface/02-serialization-reflection-resource-project-world-sync-public-dto-contract-review.md`
- 交接原因：Local test fixture drift; no external zr_vm source is involved.

## 失败现象与复现证据

Managed job `dd18d653fe3c494c91fbe57f5959580a` on frozen input
`interface-project-identity-3170-20260908` reported 754 passed, 11 failed and
101 ignored. One failure was
`reflect::schema_catalog::tests::catalog_fingerprint_is_independent_of_input_and_metadata_set_order`.
Its fixture constructor panicked at `tests.rs:15:58` with
`InvalidTypePath { type_path: "game.Alpha", reason: "short type path must match terminal segment Alpha" }`.
Log: `E:/cargo-targets/zircon-engine/cache/build-benchmarks/interface-project-identity-3170-20260908/results/interface-library-3170.log`.
The current test file is unchanged from HEAD, unowned before repair, SHA-256
`f6f3a7ba0ea7aec2f506d2361e1f16c60e948bb50fc2057f75edc4ae2b049560`.

## 最低共享层根因

Both metadata-order variants construct full path `game.Alpha` with short path
`Shared`. `ReflectTypePath::new` calls the production
`reflect/type_path/validation.rs::validate_short_type_path`, which requires the
short name to match the terminal segment. The failure occurs before either
catalog fingerprint or snapshot equality is evaluated. The separate ambiguous
short-path test correctly uses `game.first.Shared` and `game.second.Shared`.

## 架构修复验收

- Use `Alpha` for both full-path `game.Alpha` fixtures, preserving different
  metadata order and both fingerprint/snapshot equality assertions.
- Execute all nine schema-catalog tests, including ambiguity, duplicate keys,
  closure, stable slots, tampering, incremental publication and atomic replace.
- Execute the separately mounted `tests::reflect` contract batch on the exact
  frozen source, including constructor and wire-grammar rejection.
- Bind formal fixing-Session validation, independent C0/I0/M0 review and
  canonical return/closeout before claiming completion.

## 禁止临时方案

- Do not weaken production type-path validation or introduce aliases.
- Do not skip the test, remove assertions, or count filtered-out tests as passed.
- Do not alter external zr_vm or accept its dirty worktree as a pinned input.

## 修复结果与回传

- 根因：Schema-catalog fingerprint fixtures used short path Shared for full path game.Alpha, so the production canonical type-path validator rejected the fixture before fingerprint assertions.
- 架构修复：Changed only the fixture short path to Alpha for both metadata-order variants, preserving the production single validated constructor and the independent ambiguous Shared-path coverage.
- 验证：Managed Windows locked no-default Interface02 jobs 085a233ed2c945b28885f2be3f0edf8e (9/9 schema-catalog tests) and deff74bafac140c8e7edde2f617f1 (18/18 reflect contracts) on snapshot 3213; independent review C0/I0/M0 report .codex/tmp/interface02-reflection-catalog-3213-review-20260908-result.txt; source hash 5da956b6581962148f72856952bb641a2162b50303ab621ef724321e3dd6e41a.
- 回传：Return reflection-catalog-short-path-fixture-drift as fixed: canonical Alpha fixture paths now satisfy the existing validator, all 27 targeted managed tests executed and passed, and review found no Critical/Important/Moderate findings; formal closeout remains coordinator-gated.
