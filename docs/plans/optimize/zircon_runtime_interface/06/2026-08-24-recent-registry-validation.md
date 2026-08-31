---
title: Runtime Interface 06 Recent Registry Validation Performance
category: zircon_runtime_interface
report_id: RuntimeInterface06-recent-registry-validation-2026-08-24
date: 2026-08-24
session_id: optimize-runtime-interface06-recent-validation-r1-20260824
implementation_status: implementation_complete
validation_status: managed_validation_passed
---

# Runtime Interface 06 Recent Registry Validation Performance

## Scope

This batch advances the bounded recent-project projection described by Runtime Interface 06. It
preserves the v1 wire shape, entry validation, duplicate-path errors, canonical timestamp/path
ordering, and the eight-entry retention limit. It does not claim the parent plan's ProjectId,
revision/writer/tombstone, crash recovery, or cross-process transaction work is complete.

## Change

`HubRecentProjectsV1::validate` previously validated and normalized each path, then cloned every
project into `merge_hub_recent_projects`, normalized every path again, rebuilt the merge map,
sorted, truncated, and compared the reconstructed vector. It now checks canonical timestamp/path
order through borrowed adjacent entries during the same pass as entry and duplicate validation.
The order result is deferred until the pass completes, preserving the prior field/duplicate error
precedence. Accepted normalized path keys are moved directly into the map; only the duplicate error
branch copies an already occupied key for its public diagnostic.

At the eight-entry protocol limit, entry clones are `8 -> 0`, path normalizations are `16 -> 8`,
and entry visits are `16 -> 8`. Accepted path-key clones are also `8 -> 0`. The ignored release gate
uses 16 KiB names over 21 alternating sample pairs and requires full optimized validation P95 to be
at most 25% of the legacy rebuild path. Each sample now averages 128 complete validations before
the nearest-rank P95 is calculated, preventing a single scheduler preemption from dominating a
microsecond-scale optimized sample while preserving the same implementation and threshold.

A streaming top-eight merge was also evaluated and rejected before implementation. Under the v1
contract, an equal-timestamp replacement for the same normalized key may carry a different raw
display path; that replacement can change final path ordering and make an earlier evicted candidate
eligible again. Bounding the merge to nine retained values would therefore change observable
membership. The existing merge remains until the parent plan establishes canonical ProjectId/path
ordering semantics that make bounded selection equivalent.

## Validation

- TDD red state: the two new source-contract tests failed before implementation.
- Follow-up accepted-key move red state: 2/2 failed before the occupied/vacant entry split.
- Follow-up single-pass red state: 2/2 failed before canonical-order checking was folded into the
  validation pass.
- A focused Rust regression keeps duplicate-path errors ahead of a previously observed
  non-canonical-order condition; execution is pending the grouped coordinator Cargo ticket.
- Source contract 2/2, Python bytecode compilation, `rustfmt`, and scoped whitespace validation:
  passed.
- Managed ticket `c5d40654d5b14b91a425ccc239c4caa0` passed duplicate-precedence behavior and
  confirmed all structural reductions, but its single-call samples measured an optimized
  `p50=2.0 us` and `p95=342.7 us`; the scheduler-dominated P95 missed the 75% reduction gate even
  though legacy P95 was `979.7 us`. The follow-up batches 128 validations per sample and keeps the
  original gate unchanged.
- Managed follow-up ticket `bc8ac17918e74bbba4afb7be4b70545c` passed the duplicate-precedence
  behavior regression and ignored release gate. Across 21 alternating samples with 128 validations
  per sample, P95 measured `117,023 ns -> 1,913 ns`, a `98.365%` reduction. Entry clones remain
  `8 -> 0`; path normalizations and entry visits are both `16 -> 8`; accepted path-key clones remain
  `8 -> 0`.
- No local Cargo lane or Cargo dry-run was launched, polled, or terminated.

## Remaining Parent-plan Work

Runtime Interface 06 still requires the versioned compatibility/admission DTO chain, bounded
manifest and mailbox codecs, session/focus generation and acknowledgement, a single recent-store
owner with ProjectId/revision/tombstone semantics, and crash/restart qualification.
