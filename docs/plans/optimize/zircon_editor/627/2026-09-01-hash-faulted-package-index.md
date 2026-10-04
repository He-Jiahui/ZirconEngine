---
title: Editor627 Hash Faulted Package Index
category: zircon_editor
report_id: Editor627-hash-faulted-package-index-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor627 Hash Faulted Package Index

The immutable editor plugin catalog snapshot now stores faulted package IDs in a preallocated
`HashSet<String>`. The previous `BTreeSet<String>` maintained ordering that was never exposed; its
only consumer is the package-fault membership query used during manager snapshot construction.

Fault detection, package ownership, catalog registration order, capability projections, and public
snapshot access remain unchanged. IDs remain snapshot-owned strings and are cloned exactly once
from failed registration reports as before.

The ignored Windows Release benchmark emits `EDITOR627_HASH_FAULTED_PACKAGE_INDEX_BENCH_V1` over
17 alternating sample pairs with 32,768 long package IDs. The gate requires hash construction plus
membership P95 to be at most 40% of ordered-tree P95.

No direct Cargo validation was run. The coordinator owns combined Runtime627/Editor627 Windows
Release compilation, prefixed regression tests, ignored benchmarks, measured record finalization,
manifest-only commit/push, and one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Editor627 is prepared with Runtime627 under request
`runtime627-editor627-shader-faulted-index-performance-20260901iq-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.
