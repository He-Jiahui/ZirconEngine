---
related_code:
  - zircon_runtime_interface/src/profiling.rs
  - zircon_runtime_interface/src/runtime_api.rs
  - zircon_runtime_interface/src/lib.rs
  - zircon_runtime_interface/src/tests/contracts.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/frame.rs
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
implementation_files:
  - zircon_runtime_interface/src/profiling.rs
  - zircon_runtime_interface/src/runtime_api.rs
  - zircon_runtime_interface/src/lib.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/frame.rs
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
plan_sources:
  - .codex/plans/Zircon 性能时间轴与 Tracy 集成设计.md
  - user: 2026-05-13 continue profiling timeline and Tracy integration milestone
tests:
  - zircon_runtime_interface/src/tests/contracts.rs
  - zircon_runtime/src/dynamic_api/tests.rs
  - zircon_app/src/entry/runtime_library/tests.rs
  - target: cargo check -p zircon_runtime_interface --locked
  - target: cargo test -p zircon_runtime_interface --locked
  - target: cargo check -p zircon_app --profile profiling --features "target-editor-host profiling profiling-tracy profiling-chrome" --locked
  - target: cargo test -p zircon_app runtime_api_profile_control_is_optional_after_present_prefix --profile profiling --features "target-editor-host profiling profiling-tracy profiling-chrome" --locked
  - target: cargo build -p zircon_app --release --locked
  - target: cargo build -p zircon_app --release --features profiling --locked (expected failure)
doc_type: module-detail
---

# Profiling ABI Contract

## Purpose

`zircon_runtime_interface::profiling` defines the transport-safe contract for profiling configuration, timeline snapshots, hotspot reports, and runtime profile-control commands. It is the neutral boundary shared by `zircon_runtime`, the dynamic runtime cdylib API, `zircon_app`, and `zircon_editor`.

The interface crate does not own recorder state, file I/O, Tracy subscribers, editor panels, or runtime render behavior. It only owns serialized DTOs and the optional ABI function pointer type.

## DTOs

`ProfileCaptureConfig` carries capture limits, output location, frame budget, and whether Perfetto output should be included. `normalized()` fills empty ids/paths and zero limits from defaults:

- `PROFILE_DEFAULT_OUTPUT_ROOT = "target/zircon-profiles"`.
- `PROFILE_DEFAULT_SESSION_ID = "local"`.
- `PROFILE_DEFAULT_FRAME_BUDGET_MS = 16.67`.
- `PROFILE_DEFAULT_MAX_FRAMES = 512`.
- `PROFILE_DEFAULT_MAX_SPANS = 16384`.
- `PROFILE_DEFAULT_MAX_COUNTERS = 4096`.

`ProfileSnapshot` is the ABI-safe timeline view. It contains session metadata, active/feature flags, the frame budget, and vectors of frame, span, and counter snapshots. Spans carry ids, optional parent ids, optional frame indices, stream/category/name/path strings, timestamps, durations, and nesting depth.

`HotspotReport` groups recorded spans by `stream/category/name/path` and carries totals, averages, p95, max, count, frame count, over-budget count, and conservative optimization hints.

## Optional Runtime ABI Hook

`ZrRuntimeProfileControlFnV1` has this ABI shape:

```text
unsafe extern "C" fn(session, request_json, out_json) -> ZrStatus
```

The request and response are JSON-encoded `ProfileControlRequest` and `ProfileControlResponse` values carried through `ZrByteSlice` and `ZrOwnedByteBuffer`. The command enum supports:

- `start_capture` with optional config.
- `stop_capture`.
- `snapshot`.
- `export_report`.
- `reset`.

`ZrRuntimeApiV1` appends `profile_control` after `present_viewport`. It remains optional so hosts can safely load older runtimes by checking the advertised function-table size before reading the field. The required v1 prefix still ends at `capture_frame`; viewport surface present and profile control are both optional extensions.

## Buffer Ownership

Dynamic runtime responses use `ZrOwnedByteBuffer` with a runtime-owned free callback. `zircon_runtime::dynamic_api::frame` owns the profile-response buffer token and free routine, mirroring the existing frame and accessibility JSON buffers. Hosts must call the returned free function when present after decoding the response.

## Test Coverage

`zircon_runtime_interface/src/tests/contracts.rs` verifies the runtime API table size, optional `profile_control` field ordering, and JSON roundtrip for `ProfileControlRequest`. Runtime dynamic API tests verify invalid JSON is rejected before session lookup and that a valid snapshot request returns a serialized response. App runtime-library tests verify `profile_control` remains an optional extension after the viewport-present prefix, while the release build gates verify ordinary release builds stay profiling-free and `--release --features profiling` is rejected with the `--profile profiling` guidance.
