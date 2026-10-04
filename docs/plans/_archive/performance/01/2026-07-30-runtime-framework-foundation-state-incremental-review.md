# Runtime framework Foundation and State current-source incremental review

## Status

- Result: `static_complete / dynamic_pending`.
- Review date: 2026-07-30.
- Accounting: keep both modules in `pending.md`; do not add them to `review.md` before current-source managed Cargo, scale counters and product traces are GREEN.
- Code disposition: no Rust source was changed. Dirty and untracked sources were preserved as external work.

## Exact scope

| module | files | physical lines | tests | ordered path-and-file-SHA fingerprint |
|---|---:|---:|---:|---|
| `zircon_runtime/src/core/framework/foundation` | 5/5 | 59 | 0 | `61165e1f686b05af4313bf0a91dd8170345814bdcea0af0598a3498c47058fa8` |
| `zircon_runtime/src/core/framework/state` | 12/12 | 519 | 0 | `a14972f1fd8552a4563b49d48d10c0693c9a8b4ae91d96f8b356e612f67e9f85` |

All 17 Rust files were read at their current hashes. Support tracing also read the runtime config manager persistence worker/fence/state/writer, runtime event manager, and Editor project/layout/startup persistence consumers. The older bridge review's Foundation grouping and the MVP-contract review's State `11/11` count describe pre-split trees; the current counts are `5/5` and `12/12`.

## Foundation boundary

1. The five files are thin module identity and manager facades. `ConfigManager::contains_key` inherits the default trait implementation that clones a full `Value`, but no production caller was found; typed load/full snapshot and background persistence already belong to `PERF-MVP-318` and `PERF-MVP-223`.
2. `ConfigPersistenceReport` builds and sorts at most the configured 64 failure samples and clones `last_error` only when an explicit report is requested. No production polling consumer was found, so it is not promoted as a frame hotspot.
3. `EventManager` only delegates owned topic/payload operations. Topic-key, payload ownership and dynamic-bus costs remain under `PERF-MVP-015` and `PERF-MVP-323`; a duplicate task would split the same authority.

## State boundary

1. Current `StateHookIndex` owns `HashMap<T, Vec<StateHook<T>>>` enter/exit buckets and a nested state-pair map for transition hooks. This removes the older three full hook-table scans and must be retained.
2. `StateMachine` still keeps every `StateTransitionEvent` in an unbounded `Vec`. `transition_events()` returns a clone of the complete history, so memory and query cost grow with lifetime transitions rather than active consumers.
3. Every transition clones matching enter, exit and state-pair hook vectors into three new `Vec<StateHook<T>>` owners before callbacks run. Running callbacks outside the registry/state lock is correct for re-entry, but stable registrations still pay three potential allocations plus one `Arc` increment/decrement per matching hook.
4. `PERF-MVP-320` is corrected accordingly: add a bounded/cursor history contract and publish immutable matching hook slices/slots or reuse dispatch scratch without moving callbacks back under the lock.

## Acceptance

- State types/transitions/matching hooks/history `1/100/100k`, rates `1/60/120 Hz`, consumers `0/1/16`: record history bytes, query clone bytes, hash probes, hook Vec allocations, Arc RMW, registry/state lock hold and p50/p95.
- Require explicit default history budget or cursor/drain retention; full-history clone must not grow on routine incremental reads. Stable hook registration must produce zero per-transition snapshot allocation while preserving exit -> transition -> enter order, re-entry and panic/error behavior.
- Foundation config values `1 KiB/1 MiB/100 MiB`, keys `1/1k/100k`, event topics/payloads `1/1k/1M`: reuse the existing `223/318/015/323` counters and do not create facade-only caches or worker pools.

## Static gates executed

- Read 17/17 exact module Rust files and the listed support/consumer paths.
- `rustfmt --check --edition 2021` passed all 17 files at the recorded current source.
- No managed Cargo, ETW/WPR product trace or scale benchmark ran. RenderDoc is not applicable to these non-rendering framework slices. Both modules remain pending.
