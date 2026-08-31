---
title: Editor06 Unstable Viewport Overlay Capability Sort
category: zircon_editor
report_id: Editor06-unstable-viewport-overlay-capability-sort-2026-08-26
date: 2026-08-26
session_id: root-editor06-plugin-admission-borrows-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor06 Unstable Viewport Overlay Capability Sort

## Scope

`ViewportOverlayProviderRegistration::with_required_capabilities` normalizes plugin capability
names at admission. The previous path appended into an empty vector and used stable sorting even
though duplicate values are removed before publication.

## Implementation

The builder now reserves the iterator lower bound and uses `sort_unstable` before deduplication.
The normalized capability set and provider factory ownership remain unchanged.

## Performance Evidence

| Evidence | Before | After / target |
| --- | ---: | ---: |
| Capability vector reservation | 0 | iterator lower bound |
| Stable sort calls | 1 | 0 |
| Release p95 | dynamic evidence pending | <= 95% of legacy p95 |

The coordinator must publish `EDITOR06_VIEWPORT_OVERLAY_CAPABILITY_NORMALIZATION_BENCH_V1` with
legacy/optimized p95, sample/iteration/capability counts, unique count, and reservation/sort
reduction.

## Validation

Scoped rustfmt, diff checks, source contracts, and normalized-set equivalence tests are prepared.
The ignored benchmark runs in one Editor crate release command; commit integration, terminal p95
values, and WeCom delivery remain coordinator-owned.
