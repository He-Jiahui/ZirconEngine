---
title: Editor UI Asset Conflict Buffer Reuse 573
category: zircon_editor
report_id: Editor573-ui-asset-conflict-buffer-reuse-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor UI Asset Conflict Buffer Reuse 573

External UI widget/style promotion previously allocated asset, document, and display strings for
every occupied suffix candidate. Resolution now retains one capacity-planned asset ID buffer and
rewrites only its suffix while probing. Document ID and display name are materialized once, after
the first available path is known. `.zui` suffix placement and unsuffixed candidate behavior remain
unchanged.

## Static and calibration evidence

- TDD coverage verifies unsuffixed, `.zui`, extensionless, document, and display-name candidates.
- Ignored benchmark marker: `EDITOR573_DEFERRED_TARGET_ALLOCATIONS_BENCH_V1`.
- Focused test prefix: `optimization_batch_20260831gr_`.
- Performance gate: optimized P95 must be at least 30% below eager three-string allocation for 32
  occupied suffixes across 21 interleaved sample pairs and 8,000 scans per sample.
- The first allocation-deferral probe measured ratio `0.7648` and missed the `0.70` gate. After adding asset
  buffer reuse, a standalone Rust 1.94.1 `-O` calibration measured median ratio `0.1464`. These are
  preliminary data and do not replace managed Release evidence.
- Rust 1.94.1 `rustfmt --edition 2024 --check` and scoped `git diff --check` pass.
- Production/test source SHA-256: `ab369d084e27a2646976426bf17fed2c8301501f0731010830478486a852c2f6`.

## Acceptance gates

1. Managed Windows native Release compilation and focused Editor573 tests pass.
2. Candidate IDs preserve existing suffix placement and terminal values.
3. Managed ignored benchmark retains at least a 30% P95 reduction.
4. Commit/push and WeCom performance publication remain coordinator-owned after accepted
   validation.

No managed Cargo pass, commit, push, or WeCom success is claimed by this record.
