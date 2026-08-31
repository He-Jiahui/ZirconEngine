---
title: Editor profiling artifact capture isolation protected routing
date: 2026-08-23
status: routing_requested
owner_record: 2026-08-23-editor-profiling-artifact-capture-isolation-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Keep one concise `pending.md` module entry:

`zircon_editor retained-host host_contract/profiling_artifacts.rs + profiling_artifacts/**`
- 36/36 Rust files source-reviewed. Bounded Job admission, worker JSON/PNG export, typed geometry and
  non-C output validation are retained. M0 post-request environment polling/dead softbuffer export
  gate cleanup is applied and statically GREEN. Pending M1-M4: isolate capture preparation from
  measured present, use submitted GPU readback, replace duplicate frame/sample ownership and repeated
  route scans with a generation index, add scale-aware admission and run current-source WPR/power/
  RenderDoc acceptance.

Do not add these files to `review.md` before M0-M4 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach M0-M4 to MVP editor evidence integrity. Record product/capture CPU separately, environment
reads, materializations, readbacks/reference paints, control visits, route scans, hit queries,
allocated/string/pending bytes, JSON/PNG work, p95 latency, context switches, package energy and
same-build RenderDoc identity.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

Own deletion of distributed per-call profile environment reads, present-thread capture preparation,
software-rendered GPU evidence, duplicate clickable/sample tables and the dead forced-softbuffer
export check after typed replacements are authoritative.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own the generation-bound editor capture transaction and stable control/route index consumed by
geometry, hit evidence and automation. Normal present must retain borrowed generation behavior.

## `docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md`

Own bounded diagnostic preparation/export stages, terminal completion delivery, cancellation and
scale-aware admission. Background work must not be staged by first allocating its full payload on the
present thread.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own the typed source/draw/frame generation receipt and stable UI control identity needed by capture
without cloning complete presentation DTOs.

## `docs/plans/zircon_runtime/render/17-performance-and-profiling.md`

Own presenter GPU readback, capture phase markers, exact frame/backend/size identity, capture overhead
counters and RenderDoc parity. WPR CPU/power and editor Job pressure remain owned by Performance01.

## Acceptance handoff

The handoff requires 36/36 post-change fingerprints, focused and managed Rust tests, capture/backend/
scale/route/output/warmup matrices, same-executable WPR and power artifacts on D/E/F, current-source
RenderDoc GPU parity, milestone commit and quantified WeCom notification. Protected ledgers remain
unchanged until then.
