---
title: Runtime09B Disabled Directional Shadow View Admission
category: zircon_runtime
report_id: Runtime09B-disabled-directional-shadow-view-admission-2026-09-09
date: 2026-09-09
session_id: astra-optimize-20260909-root
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime09B Disabled Directional Shadow View Admission

## Finding

`VisibilityContext` still emitted one directional shadow view when a light had
no shadow settings or `casts_shadow == false`. The shadow planner already
rejected that light, so the visibility side paid for a frustum candidate pass
and a view allocation that could never be consumed. The capacity estimate also
counted every directional light, inflating the temporary `extra_views` reserve.
This is the concrete VIS213-G08/P0 finding from the current visibility review.

## Change

- `directional_shadow_ranges` now returns an empty range list for disabled
  directional lights and keeps the default cascade set for enabled lights.
- The visibility producer now selects the first shadow-enabled directional
  light, matching the current shadow planner's single-directional owner. The
  capacity estimate reserves the effective default cascade count for that one
  owner instead of counting every directional entry or under-reserving four
  cascade views.
- Existing custom-target, point-face, spot, and enabled directional view keys
  remain unchanged.

## Regression coverage

The view-context capacity owner checks disabled/explicitly-disabled/enabled
directional settings and the single-planner-owner source contract. The
construct owner adds integration regressions that assert zero views for a
disabled directional light and four views for two enabled lights with only the
first owner's `ShadowCascade` keys. Existing enabled-light layer-isolation and
11-slot atlas coverage remain in place.

## Local evidence

- `rustfmt --edition 2021 --emit stdout` parsed all three touched Rust files.
- The scoped `git diff --check` and a source-contract check for the capacity and
  empty-range predicates passed.
- No Cargo command was run. Managed Windows Cargo tests, product shadow
  captures, and release p50/p95/p99 evidence remain pending until the immutable
  validation input is admitted.

## Acceptance boundary

This record is implementation-complete only. It does not claim the broader
VIS213 gate, GPU shadow correctness, or performance acceptance. The remaining
acceptance must verify enabled cascade parity, planner/visibility key parity,
and a paired release workload with disabled-light-heavy scenes.
