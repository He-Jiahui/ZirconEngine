---
title: Editor13 Owned Layout Normalization
category: zircon_editor
report_id: Editor13-owned-layout-normalization-2026-08-25
date: 2026-08-25
session_id: root-editor13-drawer-tab-dedup-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor13 Owned Layout Normalization

## Scope

This slice removes an avoidable deep clone while migrating legacy drawer-only layouts and removes
unconditional active-ID clones from the valid normalization path. It preserves canonical drawer
mapping, active-page repair, missing-selection counts, collapsed drawer semantics, and the legacy
drawer mirror. It does not close Editor13's schema, bounded restore, transaction, placeholder, or
last-known-good migration gaps.

## Implementation

When `activity_windows` was empty, normalization cloned the complete legacy drawer map into the new
workbench activity window and later cloned the normalized drawers back into the compatibility
mirror. The migration now transfers `layout.drawers` with `mem::take`, so only the required final
compatibility mirror clone remains.

The retired drawer selection path also cloned both active IDs before checking membership. The new
path borrows each optional ID through `is_some_and`; it clones a fallback ID only when an invalid
selection is actually repaired. The test matrix covers valid selections, two independent invalid
selections, old/new full-layout equivalence, and source guards for both clone eliminations.

## Performance Contract

| Evidence | Retired path | Optimized gate |
| --- | ---: | ---: |
| Legacy drawer-tree deep-clone passes | 2 | 1 |
| Valid active-ID clones for five drawers | 10 | 0 |
| Pressure matrix | 5 drawers x 1,024 long tab IDs, 11 samples x 32 normalizations | optimized P95 <= 75% of retired P95 |

Input fixture construction is outside the timed region. The ignored release benchmark emits
`EDITOR13_OWNED_LAYOUT_NORMALIZATION_BENCH_V1` with both P95 timings, reduction basis points,
drawer/tab/sample/iteration counts, tree clone passes, and valid active-ID clone counts.

## Validation

Rust 1.94.1 `rustfmt --check`, scoped `git diff --check`, and all production source guards passed
before submission (apart from the repository's existing CRLF notice). One managed Editor layout
normalization batch covers old/new output equivalence, valid and invalid active selection behavior,
the ownership/borrow source contract, and the ignored release benchmark. Dynamic P95 evidence,
integration SHA, and automatic WeCom performance delivery remain coordinator-owned and pending.
