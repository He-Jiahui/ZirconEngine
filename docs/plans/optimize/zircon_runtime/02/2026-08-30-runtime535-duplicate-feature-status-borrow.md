---
title: Runtime Duplicate Feature Status Borrow 535
category: zircon_runtime
report_id: Runtime535-duplicate-feature-status-borrow-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime Duplicate Feature Status Borrow 535

Runtime plugin feature evaluation previously cloned every missing plugin or capability at the
caller and cloned it again before `HashSet` admission. Repeated dependencies therefore performed
two deep string clones even though the ordered diagnostic vectors retained only the first value.
The status accumulator now accepts borrowed identifiers, checks membership first, and owns the
identifier only on the first insertion. Ordering, deduplication, block projection, and capability
resolution behavior are unchanged.

The ignored Release evidence `RUNTIME535_DUPLICATE_FEATURE_STATUS_BORROW_BENCH_V1` models 65,536
updates for one already-missing dependency. The legacy path performs 131,072 string clones; the
borrowed path performs the two clones required by the first accepted value, a 99.9985% reduction.
This is an exact ownership-operation model, not elapsed-time evidence. A first-time unique value
now performs a membership lookup followed by insertion, so the optimization is intentionally
targeted at repeated dependency/status projection rather than unique-only input.

## Static evidence

- TDD RED: the regression called the accumulator with `&str` while production required `String`.
- TDD GREEN: both accumulators accept `&str`; all four production call sites pass borrowed fields.
- The behavior regression verifies duplicate plugin and capability inputs preserve one ordered
  output row each.
- `rustfmt 1.94.1 --edition 2021` passes on all three owned Runtime sources.
- Scoped `git diff --check` passes with only the repository LF/CRLF notice.
- Source SHA-256:
  `feature_status.rs` =
  `3791678ccbebda36f670df10899e37acd1541a956fc9768a8ae809cd3c848430`.
- Source SHA-256:
  `feature_status/dependencies.rs` =
  `09302ae6363d2a763bb192dc5748ed82a2784aace7ed917c4ae878bcd1b742de`.
- Source SHA-256:
  `feature_status_record/mutation.rs` =
  `fd0d1045cedc8537c1adc1d1c215ca69b8ee28e1f2dab7a220f92ad303eb90cd`.

## Acceptance gates

1. Managed Windows native Release compilation and focused Runtime tests pass.
2. Duplicate missing dependency behavior preserves first-observation ordering and resolution.
3. The ignored evidence emits the Runtime535 marker and reports at most two optimized clones.
4. Commit and push remain coordinator-owned; WeCom publication follows accepted validation only.

No direct Cargo validation, compile, commit, push, or WeCom success is claimed by this record.
