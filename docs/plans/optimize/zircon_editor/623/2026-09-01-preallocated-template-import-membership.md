---
title: Editor623 Preallocated Template Import Membership
category: zircon_editor
report_id: Editor623-preallocated-template-import-membership-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor623 Preallocated Template Import Membership

Built-in template import collection now reserves its recursive seen-import set from the root
document's widget and style import counts, using saturating addition. Recursion may discover more
documents, but common root-only and shallow graphs no longer start from zero capacity.

Widget-before-style traversal, contains-before-owning-insert admission, first-reference retention,
alias projection, and ordered output maps remain unchanged. Existing first-admission coverage stays
in force, while focused source coverage locks the initial capacity without introducing duplicate
string allocations.

The ignored Windows Release benchmark emits `EDITOR623_PREALLOCATED_TEMPLATE_IMPORT_BENCH_V1`
over 17 alternating sample pairs with 32,768 unique import references. The gate requires
preallocated membership P95 to be at most 85% of unreserved membership P95.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Editor623 is prepared with Runtime623 under request
`runtime623-editor623-module-template-import-performance-20260901im-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.
