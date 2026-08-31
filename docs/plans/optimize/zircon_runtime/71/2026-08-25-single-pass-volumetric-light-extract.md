---
title: Runtime71 Single-pass Volumetric Light Extract
category: zircon_runtime
report_id: Runtime71-single-pass-volumetric-light-extract-2026-08-25
date: 2026-08-25
session_id: root-runtime71-light-single-pass-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime71 Single-pass Volumetric Light Extract

## Scope

This slice addresses the repeated frame-extract light-table work recorded by RSL-P1-031,
RSL-P1-032, RSL-P1-035, and RSL-P1-048. It only consolidates the existing snapshot projection and
volumetric-ID sideband. It does not claim the parent plan's authoring schema, photometry, shadow,
cookie/IES, incremental registry, multi-view sharing, or product qualification is complete.

## Implementation

Frame extraction now enters one `collect_render_lights` orchestration path. Directional, point,
rect, and spot collectors append eligible volumetric IDs while they already visit active,
camera-layer-visible lights to create render snapshots. The separate four-family volumetric scan
is removed.

The viewport packet keeps its existing collectors and does not request the unused sideband.
Snapshot ordering remains family-local and entity-sorted; volumetric IDs retain the previous
global sort and dedup behavior. Active-in-hierarchy and camera-layer filters are evaluated before
an ID is admitted, matching the removed scan.

## Performance Evidence

| Evidence | Before | After / target | Reduction |
| --- | ---: | ---: | ---: |
| 10K visible volumetric point lights in frame extract | 20K point-table visits | 10K point-table visits | 50.00% table-visit reduction |
| Four volumetric-capable light families | 8 typed table passes | 4 typed table passes | 50.00% family-pass reduction |
| Volumetric sideband when unused by viewport extraction | separate storage was not requested | zero-capacity empty sideband | no heap allocation required |
| Focused Windows release wall-clock target | unbounded | <= 500 ms | pending terminal evidence |

The ignored release evidence prints `RUNTIME71_LIGHT_SINGLE_PASS_BENCH_V1` with light count,
legacy and optimized table visits, reduction percentage, elapsed microseconds, and target. Exact
elapsed time is accepted only from the coordinator's terminal result.

## Validation

- RED established that tests required a combined collector while production still exposed the
  separate volumetric table scan.
- Correctness tests cover sideband inclusion and the no-sideband viewport-style path.
- Static GREEN confirms the combined collector exists and the old volumetric collector is gone.
- The focused release tests are prepared for the shared Runtime+Editor coordinator batch.
- Final terminal marker values, integration commit, and WeCom delivery remain pending.

## Documentation Decision

The public render-extract contract and serialized light schema are unchanged. This numbered plan
record is sufficient for the internal extraction optimization.

## Remaining Parent-plan Work

Typed light definitions, photometric units, layer and shadow policy, cookies/IES, authoring and
serialization parity, incremental generation, multi-view sharing, scalability, telemetry, and
full product-scale qualification remain open under Runtime71.
