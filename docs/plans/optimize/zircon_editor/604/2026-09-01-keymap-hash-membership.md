---
title: Editor604 Keymap Hash Membership
category: zircon_editor
report_id: Editor604-keymap-hash-membership-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor604 Keymap Hash Membership

`EditorCommandRegistry::missing_default_keymap_bindings` now builds a preallocated `HashSet` of
effective keymap command IDs instead of a `BTreeSet`. This temporary collection is used only for
membership checks; missing command IDs are still emitted in the command registry's canonical
`BTreeMap` order, so externally observed ordering and borrowed return values are unchanged.

A focused equivalence test compares the default workbench registry/keymap against the retired tree
implementation. A source guard requires exact keymap-sized preallocation and hash membership. The
ignored Windows Release benchmark emits `EDITOR604_KEYMAP_HASH_MEMBERSHIP_BENCH_V1` over 17
alternating sample pairs with 32,768 long command IDs and 16,384 keymap bindings. The gate requires
hash-membership P95 to be at most 50% of the legacy BTree membership P95.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Editor604 is prepared with Runtime604 under request
`runtime604-editor604-pointer-keymap-performance-20260901hv-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.
