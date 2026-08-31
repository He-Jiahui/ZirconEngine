---
title: Editor Identity Image Row Strides 555
category: zircon_editor
report_id: Editor555-identity-image-row-strides-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor Identity Image Row Strides 555

The retained-host opaque identity-image fast path previously rebuilt source and destination row
offsets from two-dimensional coordinates in both its alpha scan and copy pass. It now computes row
byte length, source stride, destination stride, and initial offsets once, then advances offsets by
addition. The full alpha scan still finishes before any frame bytes are written, preserving the
all-or-nothing fast-path contract.

For 65,536 rows, stride-dependent row-address multiplications fall from 196,608 to three setup
calculations, a 99.998% reduction. A standalone Rust 1.94.1 `opt-level=3` benchmark exercised eight
million source/destination row addresses with 11-sample medians. The modeled path changed from
9.5123 ms to 2.7027 ms, a 71.59% improvement on this machine. Alpha scanning and row-copy bandwidth
are excluded.

## Static evidence

- TDD RED: both passes contained `row` inside their two-dimensional offset expressions.
- TDD GREEN: source and destination offsets advance through precomputed strides.
- Alpha rejection still precedes mutable frame access and any copy.
- Focused tests use prefix `optimization_batch_20260830et_editor555_`.
- Ignored evidence marker: `EDITOR555_IDENTITY_ROW_STRIDE_BENCH_V1`.
- `rustfmt 1.94.1 --edition 2021` and scoped `git diff --check` pass.
- Production/test source SHA-256: `7b646592e2744f65dc9b09c03dbc61565b35aadf32db7894970d2edc21044bd5`.

## Acceptance gates

1. Managed Windows native Release compilation and focused Editor tests pass.
2. Transparent source pixels still reject without modifying the frame.
3. Opaque clipped rows retain exact source and destination byte ranges.
4. Commit and push remain coordinator-owned; WeCom publication follows accepted validation only.

No direct Cargo validation, compile, commit, push, or WeCom success is claimed by this record.
