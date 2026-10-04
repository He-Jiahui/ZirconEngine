---
title: Runtime200 Product Input to Frame Profile
category: zircon_runtime
report_id: Runtime987-product-input-to-frame-profile-2026-09-26
date: 2026-09-26
implementation_status: fixture_complete
validation_status: managed_validation_pending
performance_status: product_samples_pending
---

# Runtime200 product input to frame profile

## Scope

The local valid-owner-route benchmark times one helper on a retained tree. It does
not measure Dynamic UI event ingress, typed host action publication, frame
extraction, or WGPU frame capture. The existing product UI behavior test reaches
the host queue and render extract, but has no latency samples. This slice adds a
small product-session fixture without changing runtime production code.

`product_pointer_action_reaches_host_output_and_render_extract` loads an authored
project UI view, sends a touch start/end pair through `RuntimeDynamicSession`,
checks the typed `UiAction` target, commits host output, checks the retained render
command, and verifies the action cannot replay after commit.

The ignored Windows WGPU profile uses that same Dynamic session path. For one and
sixteen clicks per sample, it records eight warmups and 31 measured cycles. Each
cycle times event dispatch, host-output prepare/commit, runtime tick plus WGPU
frame capture, and the total input-to-capture cycle. It reports p50/p95/p99 for
each stage under marker `RUNTIME200_UI_INPUT_TO_CAPTURE_CYCLE_PROFILE_V1`.
Each timed sample then checks its action batch, non-empty WGPU pixels, executed
render passes, and submitted UI commands outside the timed region.

## Acceptance boundary

This fixture supplies product-path evidence, not an optimization speedup claim.
The button sends a host action but does not change the authored UI; the total
sample measures input through the next completed frame capture, not latency to
a visible action response.
No numeric product latency or allocation budget is asserted here. It covers one
authored 640×360 viewport, one touch source, one button, and bursts of at most 16
clicks. Large trees, deep routes, keyboard/IME, accessibility, multi-window/seat,
queue age, allocations, RSS, matched Unreal workloads, and actual measured
p50/p95/p99 remain open under `G-UI-25` and `RUII-P1-048`.

Run the regular behavior test and ignored profile in the next grouped managed
Windows Runtime batch. Static formatting and diff checks alone are not acceptance.
