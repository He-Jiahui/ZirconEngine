---
title: Runtime Plugin Target Dedup Bitsets 538
category: zircon_runtime
report_id: Runtime538-plugin-target-dedup-bitsets-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime Plugin Target Dedup Bitsets 538

Three runtime-plugin validation paths previously allocated a `Vec<RuntimeTargetMode>` and scanned
it for every target row: module target modes, capability-status targets, and package-supported
targets. `RuntimeTargetMode` has exactly three definition-bound variants, so each transient state is
now a one-byte bitset. Duplicate rows retain their original traversal point and diagnostic text;
coverage and Editor-host checks still run in the same order.

The ignored Release evidence `RUNTIME538_PLUGIN_TARGET_DEDUP_BITSET_BENCH_V1` models 65,536
three-target validations for one state site. The legacy state records at least 65,536 heap growth
events while the bitset records zero, a 100% reduction. The other two owned state sites use the
same one-byte representation. This is deterministic state/allocation evidence, not elapsed-time or
whole-plugin-load evidence.

## Static evidence

- TDD RED: the structural regression failed while all three production states still contained
  `Vec<RuntimeTargetMode>` / `Vec::new()`.
- TDD GREEN: the three state aliases are `u8`; their constructors return zero and uniqueness uses
  exhaustive enum-to-bit matches with no `seen.contains` or `seen.push`.
- Focused tests cover empty state, first insertion, repeated insertion, full three-bit coverage, and
  unchanged duplicate diagnostics.
- The bit values remain local definition-bound enum-shape details; they are not persisted and are
  not promoted into a shared protocol constant.
- `rustfmt 1.94.1 --edition 2021` and scoped `git diff --check` pass.
- Module state/row/uniqueness SHA-256:
  `19aab80070d5cb445848be988e180887c10fbfb81d9f33793b4f7c61b0da84a4`,
  `93cff342a7fe4bf2bc2db60624f10115a1d569191296c9fcf027c82edd5ff85a`,
  `fa4a5ec1ce8925bdaaed04b7f48d2061a41dbedaa8d98674f304461a1f770d1e`.
- Capability-target state/row/wrapper/uniqueness SHA-256:
  `25163d2d125499eaf8dd66e8020fea20fcc6bc0d5951eba0761360abfd5362b0`,
  `903d66ed2f736f64b6e39f7efd3ce1e80c05b8f1abfd47dd6b39e83a3c5f3366`,
  `f698d337047b692a6314ef0d4c334fed68f83705f2984c7c4b28e99a3c24868f`,
  `535949f0264ff74c2a092a15cdf1db601f34167ff631db24c63d6eadead64368`.
- Supported-target state/uniqueness SHA-256:
  `940cb2326450e0bd2d963571adfc079037539f2bb3020261bd86bf62414695f3`,
  `6eb3e2fc87d89b58f05eaa0da6e2349ba40529100b5444561f5bedd25c361ee7`.

## Acceptance gates

1. Managed Windows native Release compilation and focused Runtime tests pass.
2. Duplicate target diagnostics, coverage checks, and validation order remain unchanged.
3. The ignored evidence emits the Runtime538 marker with zero optimized heap growth events.
4. Commit and push remain coordinator-owned; WeCom publication follows accepted validation only.

No direct Cargo validation, compile, commit, push, or WeCom success is claimed by this record.
