---
title: Runtime82 Native Present Million Character Edit Scale Profile
category: zircon_runtime
report_id: Runtime82-native-present-million-edit-scale-profile-2026-09-26
date: 2026-09-26
implementation_status: fixture_complete
validation_status: managed_validation_pending
performance_status: product_budget_pending
---

# Runtime82 native present million character edit scale profile

## Workload and measurement

The ignored Windows Release integration profile
`runtime82_million_edit_native_present_profile` extends the single-edit native
diagnostic with five warmups and 31 measured, identical Backspace edits. Each
sample creates a new `UiSurface` with an InputField containing `W` followed by
one million `i` characters and a caret after `W`. Setup, baseline native
present, baseline capture, focus, and focused rebuild are outside the measured
interval. The interval starts before real `UiInputManager` keyboard dispatch
and ends after the edited frame's native present call, public `capture_frame()`
submission finish and retained-output readback, and submission receipt checks.
The renderer, viewport, native Win32 surface, and GPU caches remain live across
samples. This is a repeated cold Surface edit with a warm renderer, not a
continuous edit stream on one retained document.

Every sample asserts `TextEditChange`, the one-million-character result
beginning with `i`, a text render command, a native `present_submission()`
ticket, a newer frame generation, a same-generation capture, executed UI graph
and text payload, nonzero capture pixels, and changed pixels in the text field.
Both compared frames are visually unfocused, so focus or caret changes cannot
explain the pixel difference. Captures read the renderer's retained output
texture for the presented generation, not the operating system swapchain.

The `RUNTIME82_NATIVE_PRESENT_MILLION_EDIT_SCALE_V1` line retains all 31 raw
samples for edit dispatch, rebuild to render extract, present API wall time,
finish plus readback wall time, total edit to finished present plus readback,
process working-set RSS before and after, signed RSS delta, changed pixel
counts, and before/after submission generations. It also prints nearest-rank
p50/p95/p99 for total latency and RSS delta, plus OS, architecture, crate
version, Windows processor identifier when available, and renderer adapter
name/device type. The total is an upper-bound proxy for native
present completion because it includes readback and checks. It is not monitor
scanout latency. Process RSS includes renderer caches, both captures, and other
process activity; its delta is descriptive, not per-edit allocation.

The winit event loop runs on a dedicated thread using Windows
`with_any_thread(true)`. A ten-second deadline covers missing surface/redraw
events; the libtest thread fails if the complete profile does not report within
900 seconds. The managed runner has no verified process-level timeout.

## Acceptance boundary

The new integration test is a separate managed Windows Release diagnostic
ticket with `ui` and `platform-winit` enabled, `--ignored --nocapture`, and a
single test thread. Retain its raw marker and machine/device identity. Static
Rustfmt and diff checks establish source readiness only. No native result or
product performance pass is claimed before a managed run. The fixture omits
App/Dynamic Session ingress, allocator counts, power, actual display scanout,
continuous document editing, and matched Unreal measurements. The Runtime82
review defines `RTE-GATE-016` and `RTE-GATE-047` without frozen numeric
thresholds; both remain open pending matched product evidence and budgets.
