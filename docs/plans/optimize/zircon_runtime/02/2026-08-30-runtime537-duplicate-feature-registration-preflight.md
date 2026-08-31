---
title: Runtime Duplicate Feature Registration Preflight 537
category: zircon_runtime
report_id: Runtime537-duplicate-feature-registration-preflight-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime Duplicate Feature Registration Preflight 537

Runtime optional-feature registration previously cloned the complete manifest and provider before
checking whether the same `feature_id@provider` key had already been registered. Duplicate reports
therefore paid for deep materialization that was immediately discarded. Registration now projects
the existing canonical key first, checks the registration set by reference, and materializes a
`FeatureDefinition` only for the first registration. A catalog-private constructor accepts the
precomputed key and debug-checks it against the canonical key algorithm.

The ignored Release evidence `RUNTIME537_DUPLICATE_FEATURE_REGISTRATION_PREFLIGHT_BENCH_V1`
models 65,536 reports for the same key. The legacy path performs 65,535 discarded duplicate
manifest clones; the preflight path performs zero, a 100% reduction. This is an exact
ownership-operation model, not elapsed-time evidence. A first-time unique registration now performs
a membership lookup followed by insertion, so this optimization intentionally targets duplicated
runtime registration streams.

## Static evidence

- TDD RED: the regression failed while `FeatureDefinition` was materialized before duplicate
  rejection.
- TDD GREEN: canonical key projection and borrowed membership testing precede
  `registration.manifest.clone()`.
- The `feature_id@provider` key format and both duplicate diagnostics remain unchanged.
- `rustfmt 1.94.1 --edition 2021` passes on both owned Runtime sources.
- Scoped `git diff --check` passes with only repository LF/CRLF notices.
- `feature_definitions/definition.rs` SHA-256:
  `254fc16efa7b566d865375d1b168af71ac78e8b7640d091fa8074fb78fe56eba`.
- `runtime_feature_definitions/registration.rs` SHA-256:
  `919d7489a62674b3192fd799ee86f926e9ce48747aac8213dc48914c971ae0dd`.
- `runtime_feature_definitions/merge.rs` contains unrelated pre-existing worktree changes; this
  task did not edit or attribute it and excludes it from validation and commit manifests.

## Acceptance gates

1. Managed Windows native Release compilation and focused Runtime tests pass.
2. Duplicate registration diagnostics and declared-feature conflict behavior remain unchanged.
3. The ignored evidence emits the Runtime537 marker and reports zero duplicate manifest clones.
4. Commit and push remain coordinator-owned; WeCom publication follows accepted validation only.

No direct Cargo validation, compile, commit, push, or WeCom success is claimed by this record.
