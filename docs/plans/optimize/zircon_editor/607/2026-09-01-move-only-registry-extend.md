---
title: Editor607 Move-Only Runtime Event Consumer Registry Extend
category: zircon_editor
report_id: Editor607-move-only-registry-extend-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor607 Move-Only Runtime Event Consumer Registry Extend

`EditorRuntimeEventConsumerRegistry::extend` now preflights incoming consumer IDs against the live
registry before mutation, then moves the disjoint incoming `BTreeMap` into the live registry with
`append`. The previous transaction cloned every active registration, including manifests and Arc
handles, then inserted the incoming batch into the clone. A duplicate still rejects the whole batch
before any live mutation, including when the duplicate sorts after valid incoming registrations.

Existing coverage already verifies the late-duplicate zero-mutation contract. A new behavior test
covers successful disjoint movement, and a source guard requires preflight plus append without a
registry clone. The ignored Windows Release benchmark emits
`EDITOR607_MOVE_ONLY_REGISTRY_EXTEND_BENCH_V1` over 17 alternating sample pairs with 8,192 active
and 2,048 incoming registrations using 512-byte retained payloads. The modeled active-registration
clones fall from 8,192 to zero, and the gate requires move-only P95 to be at most 20% of legacy P95.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Editor607 is prepared with Runtime607 under request
`runtime607-editor607-source-registry-performance-20260901hx-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.
