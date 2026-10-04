---
title: Runtime609 Bounded Schema Slot Scan
category: zircon_runtime
report_id: Runtime609-bounded-schema-slot-scan-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime609 Bounded Schema Slot Scan

`MaterialAsset::all_texture_slots` now freezes the number of schema-v1 PBR slots before it appends
custom entries and limits alias checks to that initial slice. The previous loop searched the entire
growing output for every custom slot even though custom slots originate from a `BTreeMap` and are
already unique. This produced quadratic string comparisons as custom material schemas grew.

Schema-v1 entries remain first, an active schema entry still shadows a same-named custom entry, a
custom entry remains visible when the corresponding schema reference is absent, and custom entries
retain `BTreeMap` order. The public return type and owned slot-name contract are unchanged. Focused
behavior coverage locks both shadowing branches and custom ordering; a source guard rejects the
former growing-slice scan.

The ignored Windows Release benchmark emits `RUNTIME609_BOUNDED_SCHEMA_SLOT_SCAN_BENCH_V1` over 17
alternating sample pairs with five schema slots and 4,096 custom slots. Modeled worst-case duplicate
comparisons fall from 8,407,040 to 20,480, and the gate requires bounded-scan P95 to be at most 10%
of legacy P95.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime609 is prepared with Editor609 under request
`runtime609-editor609-slot-layer-performance-20260901hz-v1`. Receipt, validation ticket, measured
P95, pushed SHA, and notification result are recorded only after coordinator completion.
