---
related_code:
  - zircon_runtime/src/tests/runtime_absorption/input_stack/inventory/mirror_docs.rs
  - docs/zircon_runtime/input/input_state.md
  - docs/plans/zircon_runtime/runtime/12/2026-07-17-m5-input-event-bounds-current-source-closeout.md
implementation_files:
  - zircon_runtime/src/tests/runtime_absorption/input_stack/inventory/mirror_docs.rs
  - docs/zircon_runtime/input/input_state.md
  - docs/plans/zircon_runtime/runtime/12/2026-07-17-m5-input-event-bounds-current-source-closeout.md
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
  - docs/plans/zircon_runtime/runtime/12/2026-07-17-m5-input-event-bounds-current-source-closeout.md
tests:
  - python -m unittest tools.tests.test_runtime_input_stack_audit
  - cargo check -p zircon_runtime --lib --locked --jobs 1
  - managed Windows job 8fbd021bed7641d3909cb981c55d083d / run 85db1f8c7a524bbe950e4ea37b65dd31
doc_type: milestone-detail
---

# Runtime12 M4 Mirror-Doc Guard Post-Commit Fix

Plan: docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md

Milestone: M4

Status: validating

Files: ["zircon_runtime/src/tests/runtime_absorption/input_stack/inventory/mirror_docs.rs", "docs/zircon_runtime/input/input_state.md", "docs/plans/zircon_runtime/runtime/12/2026-07-17-m5-input-event-bounds-current-source-closeout.md", "docs/plans/zircon_runtime/runtime/12/2026-07-17-m4-mirror-doc-guard-post-commit-fix.md"]

Date: 2026-07-17

## Main accepted commit

Runtime12 bounded input-event retention and indexed action evaluation is already committed as `94da2b39e79722a030b5aeb27fbcdbf3f2611c27`. Its managed Windows input gate passed 39/39, its canonical plan-status guard passed exactly 1/1, the direct structure audit reported runtime/framework/test `18/25/7` and behavior anchors `21` with empty unexpected/missing/wiring/risk lists, and independent review reported Critical 0 / Important 0.

## Post-commit finding and correction

The post-commit read-only review found that `runtime_12_input_stack_mirror_docs_match_structure_audit_counts` required the concise M4 closeout to duplicate every detailed audit anchor. That made the guard enforce a false mirror relationship: the module document is the detailed audit authority, while the M4 addendum is intentionally a concise accepted summary.

The correction keeps the detailed exact-anchor and uniqueness checks on `docs/zircon_runtime/input/input_state.md`, limits the M4 addendum check to its title, milestone, `18/25/7`, behavior-anchor `21`, and empty-list acceptance summary, and states that the protected parent plan/runtime index are outside this four-path business manifest. The historical filename segment `m5` is documented as an execution-batch label, not a new protected-plan milestone. No compatibility shim, skipped guard, cfg gate, or threshold weakening was introduced.

## Current validation

Source-manifest-bound Windows job `591948392cbb428487ff2f3908754c36` / run `6d9bc0aa6bb243d992c23988d6e35f97` ran `cargo check -p zircon_runtime --lib --locked --jobs 1` to natural completion. It exited `0`, released with `live_process_pids = []`, and finished in 11m07s. The 511 emitted warnings are existing unused-item diagnostics and contain no Runtime12 compile error.

`python -m unittest tools.tests.test_runtime_input_stack_audit` passes 1/1, and scoped `git diff --check` reports no whitespace error (only repository LF/CRLF notices). Independent review of the three-path correction reports Critical 0 / Important 0.

The source-manifest-bound focused Rust guard used reservation `0e6fda4fdfc84ae2a60e8e9565c5cce1`, source fingerprint `28beda781f8d537d2ff80f302b322106663e043c35ee2bf27c13c4576eac4dac`, job `8fbd021bed7641d3909cb981c55d083d`, and run `85db1f8c7a524bbe950e4ea37b65dd31`. Raw stdout proves exactly one test executed: `tests::runtime_absorption::input_stack::inventory::mirror_docs::runtime_12_input_stack_mirror_docs_match_structure_audit_counts ... ok`; the result is `1 passed; 0 failed; 0 ignored; 8240 filtered` in 0.01s. The isolated lib-test build finished in 42m01s, exited `0`, and released with `live_process_pids = []`. This remains valid evidence for the guard split, but the coordinator workflow then required the historical closeout field to become `Accepted Milestone: M4` so that this follow-up record is the unique current M4 manifest owner. Because that closeout is an `include_str!` input, this record returns to `validating` until the focused guard is rerun against the new source hash.

## Boundary

This follow-up corrects only Runtime12 M4 mirror authority and evidence wording. It does not alter the accepted bounded-retention/action-index implementation, promote the unrun `zircon_app` broad gate, change Runtime10 pointer-frequency ownership, or absorb Render01/Render05/Shader06/Performance01 paths.
