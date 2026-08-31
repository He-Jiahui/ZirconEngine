---
title: Sound State Playback and Source Service Current-Source Protected Plan Routing
date: 2026-08-24
status: routing_record_only
source_review: docs/plans/performance/01/2026-08-24-sound-state-playback-source-service-current-source-algorithm-performance-review.md
---

# Sound State Playback and Source Service Current-Source Protected Plan Routing

## Review ledger status

Sound state/playback/source service **28/28 Rust files** completed E3 current-worktree static review at `7fe97290fd3b0350c2c0f404fd00ad2d18f1335d`; fingerprint `cda1c9b2eeb3f902b13efde0f81e5028a756185072b405b06c9f61a2eb3d5694`. All files pass standalone rustfmt and scoped diff check. Shared `storage.rs` formatting and `clip_assets.rs` early-cache/test changes are preserved. Protected `review.md` and `pending.md` remain unchanged because Cargo, product playback, real AssetManager/Kira/CPAL, WPR/ETW and power evidence are unavailable.

## Canonical owner notifications

| Finding | Canonical plan owner | Required update |
|---|---|---|
| Registered service has no production runtime/scene/Editor playback path | Plugins11 P1-01/P1-17 + Runtime08b + Editor17 | Build one real supervisor/world slot and Editor preview/Play activation before profiling or claiming baseline parity. |
| Any source descriptor update stops/restarts from authored start | Plugins11 P1-05/P1-10 + Runtime60 | Compile scene changes to dirty stable-slot commands; preserve cursor, crossfade incompatible replacement and retain last-good on failure. |
| API-triggered cleanup plus unbounded finished vectors/history scans | Plugins11 P1-10/P1-15 + Runtime02/Runtime59 | Add fixed completion pump, O(1) owner mapping, bounded cursor retention and loss/age telemetry. |
| Global manager mixes every world/session and IDs have no generation | Plugins11 P1-02 + Runtime60 | Split global device/residency from `AudioWorldSlot`; add owner epoch/generation and complete world-close cleanup. |
| Early exact-string cache hit still allows concurrent full decode/conversion | Plugins11 P1-07/P1-08 + Runtime04/Runtime64 | Preserve local hit; add canonical artifact-generation single-flight, lease/pin/evict and prepared/streamed budgets. |
| Listener/volume/external updates store data without render behavior | Plugins11 P1-05/P1-16 + Runtime08b | Report Unsupported until executable adapters exist; then use scene change extraction and bounded external realtime ring. |
| Deep synchronous snapshot/status under global state lock | Plugins11 P1-15 + Runtime03 | Publish immutable generation/freshness pages and cursor deltas; explicit budgeted full capture only. |
| Default manager factory ignores resolved plugin/project options | Plugins11 P1-03 + Runtime08b | Build one typed resolved config generation with prepare/publish/last-good across device, graph and world owners. |
| Bounded gameplay emission journal lacks world retirement | Runtime02/Runtime60 + Plugins11 | Move journal into world slot, retire on close and preserve bounded cursor/loss semantics. |

## Acceptance routing

Implementation order is product path -> state ownership -> source dirty diff -> completion plane -> clip residency -> spatial/external truth -> observation/config -> dynamic qualification. Do not wire scene transforms into the current full-restart `update_source` path.

Dynamic acceptance uses a fixed current-source MVP scene and records source/build/config/device identity, per-frame source changes, voice restarts, real/virtual/rejected voices, completion backlog/loss, clip cache/decode/residency, audio/control/main CPU, P50/P95/P99 command latency, RSS/allocation, I/O, wakeups, xrun, power and audible parity.

No Git milestone commit or quantified WeCom message is warranted by this static routing record.
