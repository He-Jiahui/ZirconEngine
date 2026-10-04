---
title: Runtime656 Preallocated Builtin Descriptor Index
category: zircon_runtime
report_id: Runtime656-preallocated-builtin-descriptor-index-2026-09-02
date: 2026-09-02
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: retracted_after_review
validation_status: not_submitted
---

# Runtime656 Preallocated Builtin Descriptor Index

Source review found that the descriptor closure can exceed the profile root-module count, so the
reservation had no reliable performance contract. The proposal was retracted and no source or
test remains attached to this plan.

Builtin runtime profile selection now reserves the temporary descriptor map from the candidate
registry length. Dependency closure can visit each candidate at most once, so candidate count is a
strict upper bound. Module selection order, missing-dependency errors, descriptor ownership, and
the final name index remain unchanged.

The ignored Windows Release benchmark emits
`RUNTIME656_BUILTIN_DESCRIPTOR_INDEX_CAPACITY_BENCH_V1` over 17 alternating sample pairs and 16,384
candidate descriptors. The gate requires reserved P95 to be at most 80% of unreserved P95 and
requires zero reserved capacity growths.

No direct Cargo validation was run. The coordinator owns combined Runtime656/Editor656 Windows
Release compilation, prefixed regression tests, ignored benchmarks, measured record finalization,
manifest-only commit/push, and one-shot WeCom publication after both gates pass.
