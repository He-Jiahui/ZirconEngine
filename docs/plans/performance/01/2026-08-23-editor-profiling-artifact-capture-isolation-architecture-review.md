---
title: Editor profiling artifact capture isolation and scale architecture review
date: 2026-08-23
module: zircon_editor retained-host profiling_artifacts
priority: MVP-P0 performance evidence integrity and editor present latency
status: source_reviewed_m0_gate_cleanup_applied_static_pass_dynamic_pending
reference_engine: Unreal Engine Slate screenshot preparation and TraceScreenshot task handoff
---

# Goal

Make editor profile capture an explicit, generation-bound diagnostic transaction whose preparation,
readback, validation, encoding and durable export are visible and excluded from product present
latency. Capturing evidence must not rebuild the full presentation, software-rasterize a GPU frame,
repeat route scans, or read process environment state on every later present.

## Reviewed source

- owner Rust files: 36/36
- lines: 1,755
- bytes: 60,517
- source-only SHA256 over lexicographically sorted owner files:
  `a7d7106bb2d22e30511a075d9b0d7392fd6d54d245214926e871425d7eff847f`
- post-M0 owner lines/bytes/SHA256: 1,747 / 60,219 /
  `95d77896b31e40f9b9035004b2c76c9e645f5b8d267c6f79dbbd594901ef2974`
- owning commit at review: `0d70d1ac6499abcf56c3f6c3ef43cb3a7502a249`

| Owner group | Files | Lines | Bytes |
| --- | ---: | ---: | ---: |
| `host_contract/profiling_artifacts.rs` | 1/1 | 18 | 567 |
| `host_contract/profiling_artifacts/*.rs` | 5/5 | 710 | 24,001 |
| `host_contract/profiling_artifacts/environment/**` | 1/1 | 43 | 1,227 |
| `host_contract/profiling_artifacts/geometry/**` | 24/24 | 847 | 28,823 |
| `host_contract/profiling_artifacts/schema/**` | 5/5 | 137 | 5,899 |

All owner production and colocated test files were read in full. Direct consumers were read through
window successful-present dispatch, warmup/measurement restart, presenter backend selection, editor
Job admission, profile hit routes, runtime hit-grid queries and `tools/ui-profile-capture.ps1`. The
cited Unreal screenshot/readback and trace-compression sources were also read directly. Supporting
files are not counted as owner coverage.

The 2026-07-17 report covered an older 35-file/1,252-line shape and is superseded for this owner by
this record. Its still-valid route-scan concerns are retained below rather than treated as current
acceptance.

## Correct foundations to retain

1. Artifact export is explicitly gated by profile capture and output-root configuration. Normal
   capture-disabled execution does not build geometry, paint a reference image or submit export work.
2. Output roots reject relative paths, Windows device paths and the C: system drive. Profile outputs
   remain on an absolute D/E/F or UNC destination.
3. Job admission is reserved before presentation materialization. A saturated export queue or invalid
   root rejects the request before the large screenshot payload is allocated.
4. JSON serialization, directory creation, file writes and PNG encoding run in the injected bounded
   editor Job system. Cancellation is checked between output phases.
5. The successful-present caller owns a one-shot `profile_artifact_capture_requested` bit, so the
   current path no longer overwrites the same artifact on every frame.
6. Geometry is a typed export schema covering layout, controls, clips and route-consistency samples;
   capture tooling consumes source-bound data instead of maintaining a second hard-coded coordinate
   model.
7. The softbuffer verification process is deliberately separate: the script disables profile export,
   forces the fallback backend and records a window screenshot. It is not a second profiler writer.

## Structural findings

### P0: capture preparation runs synchronously inside the successful-present callback

After Job admission, `submit_present_artifact_after_admission` immediately invokes its materializer.
The caller materializes the complete immutable presentation generation; geometry then projects all
docks, floating windows, tabs, toolbar/template controls and hit samples. Screenshot mode additionally
software-paints the complete GPU presentation into RGBA. Only the already-built DTO and pixel buffer
are handed to the Job.

This work occurs before the warmup present is declared complete and therefore can perturb startup,
input-to-present and recorder restart timing. At 1920x1080 the screenshot reservation alone is
8,294,400 bytes before command discovery/raster cost; at 4K it is 33,177,600 bytes. A profile run can
therefore measure evidence construction rather than the product path it is intended to diagnose.

M1 introduces an explicit capture transaction. A successful product present publishes a stable
generation receipt and, for GPU, schedules presenter readback. Geometry validation and screenshot
post-processing execute in bounded diagnostic work after the measured phase marker. The product
present counter and capture-preparation counter remain separate.

### P0: GPU evidence is produced by a second software renderer instead of the submitted frame

When the active presenter is GPU, the reference screenshot calls
`paint_host_presentation_snapshot`. That reconstructs and CPU-rasterizes the presentation instead of
reading the actual submitted surface. It can prove software-renderer expectations, but it cannot prove
GPU draw order, clipping, image residency, shader output or the pixels associated with the measured
submission.

M1/M2 use the renderer's explicit readback path for the primary screenshot and retain software paint
only as a separately labelled parity oracle. The artifact records source generation, prepared draw
generation, submitted frame id, backend, target size and readback completion. RenderDoc remains the
GPU draw/upload/scissor/time authority; it is not replaced by a CPU reference bitmap.

### P0: exported route validation repeats collection-scale searches

Geometry first owns each typed collection, then clones/converts all controls into a second
`clickable_frames` vector. Every frame creates three separately owned samples and clones
`id`/`kind`/`surface`. Each sample then re-enters product-style routing. Tab and rail routes linearly
scan row models and format candidate ids; template/toolbar routes retry fixed docks and floating
windows before using the hit grid. For C similar controls, the scan portions can grow toward O(C^2),
while strings and sample objects grow as at least 3*C.

Surface-frame control collection also walks every arranged node and invokes a hit-grid query for each
pointer node. The grid bounds a point query to its cell rather than scanning the whole tree, so this is
O(N*cell_occupancy), not an assumed unconditional O(N^2). It is still redundant capture preparation
when the arranged frame already owns stable node/control identity.

M2 builds one generation-owned control index while projection is produced. Export collections refer
to stable control ids/indices, hit samples borrow or intern identity, and each requested point performs
at most one indexed route/hit query. Center/outside/clip/z-order parity remains required.

### P1: one-shot state still reads environment variables on later successful presents

The caller evaluates `profile_capture_enabled()` before passing the already-requested bit to
`should_queue_profile_artifacts`. After the first request, every successful present still reads
`ZIRCON_PROFILE_CAPTURE` even though the result cannot change the one-shot decision. Presenter factory
creation and warmup initialization also read environment independently rather than consuming one
immutable startup profile configuration.

M0 now short-circuits the already-requested gate before the environment read. M1 replaces the
remaining distributed startup/first-request environment reads with an immutable startup config owned
by the host/profile transaction. Acceptance requires zero profile environment reads after startup and
zero capture gate reads after the one-shot request.

### P1: forced-softbuffer export suppression is an unreachable duplicate branch

`submit_present_artifacts` first returns unless capture is enabled. It then calls
`is_forced_softbuffer_screenshot_run`, whose definition requires force-softbuffer and capture disabled.
The second condition can never be true in this call and rereads the capture environment. The capture
script confirms that the forced-softbuffer process sets capture to `0`; it already exits at the first
gate and obtains its screenshot externally.

M0 deletes this dead export check/helper while retaining backend selection in
`presenter/backend.rs` and the script's separate softbuffer screenshot protocol. The focused static
contract proves both the short-circuit ordering and removal of the unreachable second gate.

### P1: export payload accounting is an estimate, not measured preparation pressure

Admission charges a fixed 4 KiB for geometry plus exact RGBA screenshot bytes. The geometry DTO,
duplicated clickable frames, 3*C hit samples and strings can substantially exceed 4 KiB at scale.
Actual pending bytes are recomputed only after materialization, after admission was already granted.

M3 establishes a bounded capture budget before projection, reserves from scale inputs or a measured
upper bound, and records estimated/actual/peak bytes plus rejection reason. No unbounded geometry
payload may bypass editor Job pressure limits.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Rendering/SlateRenderer.h`
- `dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/ProfilingDebugging/TraceScreenshot.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/ProfilingDebugging/TraceScreenshot.h`

`FSlateApplication::TakeScreenshot` resolves an explicit widget/window rectangle, calls the renderer's
`PrepareToTakeScreenshot`, then draws the owning window (`SlateApplication.cpp:4477-4532`). The
renderer contract explicitly captures a portion of rendered output (`SlateRenderer.h:484-490`). It
does not rebuild the UI with an unrelated software renderer and label those pixels as the GPU frame.

`FTraceScreenshot::RequestScreenshot` is an explicit one-shot request and forces redraw only when an
editor viewport otherwise would not render (`TraceScreenshot.cpp:74-102`). After pixels exist,
`TraceScreenshotInternal` copies the image once and launches an `UE::Tasks` job that fixes alpha,
optionally resizes, compresses and emits the trace screenshot (`TraceScreenshot.cpp:123-177`). The
transferable rule is explicit request plus renderer-owned capture followed by task-owned processing.
Zircon's target additionally requires a bounded Job reservation and an explicit measurement phase so
even the readback/copy cost is visible rather than silently attributed to product latency.

## Target architecture

1. Parse one immutable `EditorProfileCaptureConfig` during startup and inject it into presenter,
   warmup and artifact owners. Normal presents do not read process environment state.
2. Model capture as `Requested -> FrameSubmitted -> ReadbackReady -> Prepared -> Exported/Failed`, with
   one source/draw/frame generation receipt and bounded terminal state.
3. Keep product present and diagnostic capture timing separate. Measurement begins only after capture
   preparation completes or an explicit exclusion marker closes.
4. Use presenter readback for GPU evidence. Keep CPU software output under a distinct parity-oracle
   label and compare it with submitted-frame pixels.
5. Build a stable control/route index once per captured generation; export typed views and samples
   without a second owned clickable-frame table or repeated row scans.
6. Reserve scale-aware geometry, readback and encoded-output budgets before work begins. Record actual
   bytes and release capacity on every terminal path.
7. Publish artifacts atomically with schema/source/executable/workload fingerprints; never let a
   partial JSON/PNG pair masquerade as accepted evidence.

## Instrumentation and acceptance

| Evidence | Acceptance |
| --- | --- |
| capture gate/config environment reads | startup only; zero after one-shot request |
| product present vs capture preparation CPU | separately timed; preparation excluded from product p95 |
| presentation materializations | zero in ordinary present; at most one per explicit capture generation |
| GPU readbacks and software reference paints | one labelled readback; software oracle never labelled submitted GPU output |
| control visits, row scans, hit queries and strings | near O(C); no second full clickable clone; at most one query per sample |
| estimated/actual/peak pending bytes | bounded before materialization and reported at completion |
| JSON/PNG encode/write/fsync CPU and bytes | worker-only, one per request, cancellation visible |
| source/draw/frame/backend/size fingerprints | exact match across trace, JSON, screenshot and RenderDoc |
| CPU/RSS/p95 latency/context switches/package energy | same executable/workload before and after |
| hit, clip, z-order and screenshot parity | exact or thresholded with named mismatch evidence |

Matrix: capture `off/one-shot/rejected/cancelled`; backend `runtime GPU/standalone GPU/softbuffer`;
controls `1/100/1K/10K`; surface `720p/1080p/4K`; screenshot `off/readback/software oracle/both`;
route `tab/rail/template/toolbar/floating`; output `valid D/E/F/UNC/invalid C/full queue`; warmup
`0/1/120`; present count `1/1K`.

WPR owns CPU, allocation, context-switch, disk and package-energy evidence. RenderDoc owns submitted
GPU draws, uploads, scissors, GPU time and pixel parity only. All targets and artifacts stay on D/E/F.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Short-circuit the one-shot gate and delete the unreachable duplicate softbuffer export check. | applied; static contract GREEN, Rust/dynamic pending |
| M1 | Add immutable config, capture state machine, phase markers and presenter GPU readback receipt. | product p95 excludes capture preparation; exact frame identity |
| M2 | Replace duplicate frame/sample ownership and repeated route scans with one generation index. | near-linear 1/100/1K/10K scaling and route parity |
| M3 | Add scale-aware admission, actual byte accounting and atomic artifact publication. | bounded memory and no partial accepted pair |
| M4 | Run WPR/power/RenderDoc/backend/scale matrix and close source-bound acceptance. | quantified current-source evidence and parity |

## Validation state

- Owner source review: passed, 36/36 Rust files.
- Present, warmup, presenter selection, Job admission, hit-grid, route and capture-script consumers:
  read and mapped.
- Unreal primary screenshot/readback/task handoff: read and mapped.
- M0 static contract moved RED 0/2 to GREEN 2/2. Three changed Rust files pass independent
  `rustfmt --check`; scoped `git diff --check` passes with line-ending warnings only.
- Performance-contract discovery passes 118/123. The five failures are unchanged: two missing test-
  support files, missing `available_slots`, preview resize `.roots.clone()` and UI-asset root helper
  `.roots.clone()`.
- One additional adjacent plugin-V2 static test passes 1/2 but still names the deleted
  `window/template_hover/panes.rs` path. That test-manifest drift is outside this owner and is not
  treated as a product regression or silently repaired here.
- Managed Rust tests, WPR and RenderDoc remain pending because the managed Cargo Session is terminal
  `archived` with `cargo_session_not_executable`. No raw Cargo bypass and no dynamic performance claim
  is permitted.
- M1-M4 remain architecture/dynamic work. This module stays in `pending.md` until the full matrix
  passes on one source/executable/workload fingerprint.
