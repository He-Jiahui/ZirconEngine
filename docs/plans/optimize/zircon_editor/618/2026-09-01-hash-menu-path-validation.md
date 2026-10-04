---
title: Editor618 Hash Menu Path Validation
category: zircon_editor
report_id: Editor618-hash-menu-path-validation-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor618 Hash Menu Path Validation

Document toolkit admission now validates menu-path uniqueness through a preallocated borrowed
`HashSet<&str>`. The previous `BTreeSet<&str>` paid ordered-tree comparison costs even though the
set was never iterated and its order was not part of the registry contract.

Descriptor iteration, path-shape validation, and first invalid-or-duplicate diagnostics remain in
their original order. Existing lifecycle coverage continues to verify that duplicate typed menu
paths fail before snapshot publication; focused source coverage locks the borrowed preallocated
hash implementation.

The ignored Windows Release benchmark emits `EDITOR618_HASH_MENU_PATH_VALIDATION_BENCH_V1` over 17
alternating sample pairs with 32,768 unique long menu paths. The gate requires hash membership P95
to be at most 40% of ordered membership P95.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Editor618 is prepared with Runtime618 under request
`runtime618-editor618-native-resource-menu-performance-20260901ih-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.
