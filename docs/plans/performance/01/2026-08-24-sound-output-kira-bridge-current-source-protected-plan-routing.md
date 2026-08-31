---
title: Sound Output and Kira Bridge Current-Source Protected Plan Routing
date: 2026-08-24
status: routing_record_only
source_review: docs/plans/performance/01/2026-08-24-sound-output-kira-bridge-current-source-algorithm-performance-review.md
---

# Sound Output and Kira Bridge Current-Source Protected Plan Routing

## Review ledger status

Sound `runtime/src/{output,kira_bridge}` **26/26 Rust files** completed E3 current-worktree static review at `7fe97290fd3b0350c2c0f404fd00ad2d18f1335d`; fingerprint `06818cd315f2b19e241e953b22356f85146b0e3a187e59f744f85f8b646c5776`. All files pass standalone rustfmt and scoped diff check; no source changed. Protected `review.md` and `pending.md` remain unchanged because Cargo, current-source product, real CPAL, WPR/ETW, xrun, memory, power and quality evidence are unavailable.

## Canonical owner notifications

| Finding | Canonical plan owner | Required update |
|---|---|---|
| Global state mutex owns device, graph, clip, voice and query work | Plugins11 P1-01/P1-15 + Runtime08b + Runtime59 | Add device/backend actor, per-session/world slots, immutable graph generation and bounded typed commands; publish only in short critical sections. |
| API-triggered completion scan/sort and source linear lookup | Plugins11 P1-10 + Runtime08b/Runtime02 | Produce bounded generation-qualified completion events once; use stable dense voice/source slots and cursor consumers. |
| Graph compile/diff/build has repeated maps, closure copies and O(T^2+) scans | Plugins11 P1-04/P1-15 + Runtime08b | Replace with one indexed compiler generation, sparse sends, cached topology/adjacency and dirty-slot diff; publish at audio boundary. |
| 64 tracks/128 voices request up to 24,960 per-track sound slots | Plugins11 P1-09 + Runtime08b | Establish global voice allocator and separate shadow-graph budget; measure Kira allocation/RSS by tracks/voices/sends. |
| Full SoundAsset PCM plus full Kira Frame copy, no streaming/residency | Plugins11 P1-07/P1-08 + Runtime64 | Add prepared/streamed cook artifact, single-flight, bounded decode pages, leases and eviction metrics. |
| Display-name device identity, requested formats presented as capability, no hotplug/recovery | Plugins11 P1-13 + Runtime08b | Add stable device identity, supported-config negotiation, catalog generation and last-good supervisor transition. |
| Callback/xrun/latency counters have no producer and report zero/estimate | Plugins11 P1-14 + Runtime03 + Editor25 | Add realtime-safe telemetry pages with generation/window/staleness and explicit Unsupported; separate estimated/measured latency. |
| Sound Editor graph/device operations lack safe live publish and truthful telemetry | Editor17 + Editor25 | Bind preview/Play to a real audio session slot; show transition/telemetry availability and validate live graph/device workloads. |

## Acceptance routing

Implementation order is product baseline -> ownership split -> indexed graph compiler -> global voice allocator -> clip residency -> device supervisor -> observability -> product qualification. Local removal of a sort or map is not an accepted substitute for these owner-boundary changes.

Dynamic acceptance requires a launchable current-source artifact and a fixed MVP audio workload. Publish source/build/config/device identity plus graph/voice/clip scale, audio/control/main-thread CPU, lock/command P50/P95/P99, RSS/allocation, I/O, wakeups, callback/xrun/latency, power and audible-output parity. WPR/ETW owns CPU and scheduling evidence; RenderDoc applies only to rendered Sound Editor/audio-reactive frames.

No Git milestone commit or quantified WeCom message is warranted by this static routing record.
