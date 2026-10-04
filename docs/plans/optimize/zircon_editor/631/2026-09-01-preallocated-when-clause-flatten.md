---
title: Editor631 Preallocated When Clause Flatten
category: zircon_editor
report_id: Editor631-preallocated-when-clause-flatten-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor631 Preallocated When Clause Flatten

`WhenClause::all` now reserves its flattened clause vector from the input iterator lower bound.
Exact-size descriptor inputs avoid vector growth, while iterators with nested `All` clauses remain
free to grow beyond the lower bound as their children are flattened.

`Always` removal, nested `All` flattening, canonical sort, deduplication, and single-clause collapse
remain unchanged. The regression locks those normalization semantics alongside the source shape.

The ignored Windows Release benchmark emits `EDITOR631_PREALLOCATED_WHEN_CLAUSE_FLATTEN_BENCH_V1`
over 17 alternating sample pairs and 32,768 flat clauses. The gate requires preallocated flatten
P95 to be at most 85% of the unreserved path P95.

No direct Cargo validation was run. The coordinator owns the batched Runtime/Editor Windows
Release regression and performance validation. Receipt, measured P95, commit, push, and WeCom
outcome are recorded only after coordinator completion.

## Managed Validation Attempt (2026-09-01)

Exact `rustfmt --check`, `git diff --check`, source-shape guards, and SHA-256 capture passed for
the four Runtime631/Editor631 source and regression paths. Request
`runtime631b-editor631b-reference-when-capacity-performance-20260901iv-v1` returned
`offline/descriptor_absent` before admission. No validation ticket exists from this attempt; no
test, performance, commit, push, or WeCom success is claimed.
