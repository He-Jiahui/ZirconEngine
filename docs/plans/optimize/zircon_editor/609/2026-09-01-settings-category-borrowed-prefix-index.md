---
title: Editor609 Settings Category Borrowed Prefix Index
category: zircon_editor
report_id: Editor609-settings-category-borrowed-prefix-index-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor609 Settings Category Borrowed Prefix Index

Settings-page category projection now indexes borrowed category-key and label prefixes while it
deduplicates localized pages. The previous `BTreeMap` cloned every prefix and bundle ID before each
entry lookup, including duplicate prefixes shared by thousands of pages. The optimized path clones
the keys, labels, and bundle ID only for each unique final category. Canonical key ordering,
bundle separation, localized labels, page ordering, and the immutable public snapshot are unchanged.

Page localization also traverses the descriptor's category keys once into two capacity-planned
vectors instead of constructing keys and labels through two independent iterator passes. A source
regression requires the borrowed prefix map, rejects prefix `to_vec`, and requires the single key
loop. Existing locale, fallback, ordering, revoke, and category behavior coverage remains active.

The ignored Windows Release benchmark emits
`EDITOR609_SETTINGS_CATEGORY_BORROWED_PREFIX_BENCH_V1` over 17 alternating sample pairs and 4,096
pages sharing three-level category prefixes across four leaves. The gate requires borrowed-prefix
projection P95 to be at most 45% of the legacy owned-prefix implementation.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Editor609 is prepared with Runtime609 under request
`runtime609-editor609-borrowed-group-prefix-performance-20260901hz-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.
