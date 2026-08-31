---
title: Runtime Renderer Feature Dedup Owned Insert 536
category: zircon_runtime
report_id: Runtime536-renderer-feature-dedup-owned-insert-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime Renderer Feature Dedup Owned Insert 536

Renderer asset validation already receives an owned feature name from `feature_name()`, but the
deduplication path cloned that string before inserting it into the feature set. The set now uses
`BTreeSet::replace` to take ownership of the generated name directly. On a duplicate, the replaced
equal value supplies the unchanged diagnostic text; stage ordering, feature ordering, and the
first-duplicate rejection contract are unchanged.

The ignored Release evidence `RUNTIME536_RENDERER_FEATURE_DEDUP_OWNED_INSERT_BENCH_V1` models
65,536 feature validations. The legacy path performs 65,536 redundant generated-name clones; the
owned insertion path performs zero, a 100% reduction. This is an exact ownership-operation model,
not elapsed-time evidence. Tree lookup complexity and deterministic ordering remain unchanged.

## Static evidence

- TDD RED: the structural regression failed while production still used
  `seen_features.insert(feature_name.clone())`.
- TDD GREEN: production uses `seen_features.replace(feature_name)` and contains no insertion-time
  feature-name clone.
- `rustfmt 1.94.1 --edition 2021` passes on the owned Runtime source.
- Scoped `git diff --check` passes with only the repository LF/CRLF notice.
- Source SHA-256:
  `4e0e9404f5377bca714d95fcd1f59c01389927cfb9fe825edacd0bda2120a1b0`.

## Acceptance gates

1. Managed Windows native Release compilation and focused Runtime tests pass.
2. Unique features remain accepted and duplicate features retain the same diagnostic contract.
3. The ignored evidence emits the Runtime536 marker and reports zero optimized clones.
4. Commit and push remain coordinator-owned; WeCom publication follows accepted validation only.

No direct Cargo validation, compile, commit, push, or WeCom success is claimed by this record.
