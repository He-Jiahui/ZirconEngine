---
title: Runtime82 Native WGPU Present Million Character Edit Diagnostic
category: zircon_runtime
report_id: Runtime82-native-wgpu-present-million-edit-diagnostic-2026-09-26
date: 2026-09-26
implementation_status: fixture_complete
validation_status: managed_validation_pending
performance_status: product_budget_pending
---

# Runtime82 native WGPU present million character edit diagnostic

## Scope and evidence

The retained edit-to-render-extract fixture measures the CPU UI path but stops
before WGPU. This ignored Windows Release diagnostic extends the in-crate path
through a real winit Win32 window, bound WGPU viewport, UI graph execution,
native surface present, and same-generation diagnostic readback. It starts with
`W` followed by one million `i` characters, then dispatches Backspace through
`UiInputManager` with the caret after `W`. It asserts the typed text edit event,
one-million-character result beginning with `i`, and a visible text render
command. Both frames use the unfocused visual state, so a changed caret or focus
border cannot stand in for the changed text.

Each frame must publish a `RenderFrameSubmissionReceipt` containing
`present_submission()`, which is issued only when the native surface reports
`Presented`; a successful API call or a scene submission alone is insufficient.
The diagnostic also requires a text payload and executed UI graph pass. Its
`capture_frame()` result must match that receipt's generation, contain nonzero
RGBA pixels, and differ by more than four pixels within the text field after
the edit. The capture reads the renderer's retained output texture for that
generation. It is evidence of pixels sent through the present path, not a
direct read of the operating system's swapchain image.

The `RUNTIME82_NATIVE_PRESENT_MILLION_EDIT_DIAGNOSTIC_V1` line reports the one
observed edit dispatch, rebuild through render-extract publication, the
`present_frame_extract_with_ui()` call (`present_enqueue_*_ns`), and the public
`capture_frame()` call (`finish_and_readback_*_ns`) in nanoseconds, plus both
generations, changed pixel count, OS, architecture, and crate version.
`capture_frame()` completes any pending submission before reading back pixels;
the second timer includes that synchronization and cannot be interpreted as
readback alone. The first timer is API wall time, not monitor presentation
latency. This is a bounded diagnostic with one edit and two frames, not a
statistical latency benchmark.

The Windows test harness may run on a worker thread, so the event loop uses
the versioned winit Windows `with_any_thread(true)` builder extension. It exits
with a test failure if surface availability or redraw does not arrive within
ten seconds. The full event loop and GPU diagnostic run on a dedicated thread;
the libtest thread receives its result or fails after sixty seconds, even if a
synchronous GPU call blocks the event callback. This is a test-thread deadline;
the managed runner has no verified process-level timeout. The test remains
ignored in regular suites because it requires a live window, native WGPU
device, and expensive one-million-character shaping.

## Acceptance boundary

Run the ignored test in a separate managed Windows Release Runtime lib ticket
with `ui` and `platform-winit` enabled; retain its marker output and hardware
identity. Static Rustfmt and diff checks only establish source readiness. No
native result or performance pass is claimed before that lane completes. This
in-crate fixture excludes App and Dynamic Session ingress, actual monitor
scanout latency, allocator/RSS measurements, repeated p50/p95/p99 samples,
and a matched Unreal workload. No frozen product latency threshold is available;
`RTE-GATE-016` and `RTE-GATE-047` remain open.
