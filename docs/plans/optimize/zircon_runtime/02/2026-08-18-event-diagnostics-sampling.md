Plan: docs/plans/optimize/zircon_runtime/02-core-runtime-events-tasks-review.md
Milestone: P2-2 routine timing sampling and managed comparison
Status: passed

# Runtime02 event diagnostics timing sampling

## Delivered

- `EventBusDiagnosticsMode::Enabled` remains the exhaustive timing baseline and `Disabled` keeps
  its zero-counter behavior. The default bus now uses an explicit sampled mode with a 64-event
  interval.
- Published, delivered, dropped, disconnected, queued, peak-depth and waiter counters remain
  exact. Only routine publish-duration and queue-age timestamp capture is sampled; every actual
  delivery-lock contention still receives a timestamp.
- Publish and delivery sampling use their existing monotonic counters as independent sequences, so
  no new selector atomic is added to either hot path. The snapshot exposes the active interval.
- A full bounded/latest queue now records replacement directly. Because pop-plus-push leaves the
  physical and global queue depth unchanged, the replacement path no longer performs a depth
  decrement followed by an increment; dropped, delivered and sampled queue-age accounting remain.
- Publish clones the shared payload only for subscribers before the last one and transfers the
  original `Arc` to the last subscriber. Delivery order and pointer identity remain unchanged.

## Performance contract

- Stable full-interval timing capture: `64/64 -> 1/64`, a deterministic `98.4375%` reduction in
  routine `Instant::now` calls while exact traffic/depth counters remain enabled.
- Queue-depth RMW operations per bounded replacement: `2 -> 0`, a deterministic `100%` reduction.
- Payload `Arc` clones per publish with `N` subscribers: `N -> N-1`; the common one-subscriber case
  is `1 -> 0` (`100%`), while 2/5/100 subscribers reduce clones by `50%/20%/1%`.
- The managed comparison rotates full, sampled and disabled modes over five repetitions in one
  binary. It emits P50/P95/P99 and measured publish throughput for 1 and 100 subscribers with a
  4 KiB payload, plus exact full and sampled timestamp counts.
- The existing 1/2/5/100-subscriber and 64 B/4 KiB/256 KiB full-diagnostics matrix emits P99 and
  passed all 12 cases in the managed release run.

## Managed performance result

| Case | Full baseline | Sampled / optimized | Result |
|---|---:|---:|---:|
| routine timing captures, 1 subscriber | 2,720 | 50 | 98.1618% fewer |
| publish P95, 1 subscriber / 4 KiB | 400 ns | 300 ns | 25.0% lower |
| throughput, 1 subscriber / 4 KiB | 3,711,220.64/s | 6,454,866.36/s | 73.9% higher |
| routine timing captures, 100 subscribers | 72,720 | 1,140 | 98.4323% fewer |
| publish P95, 100 subscribers / 4 KiB | 6,600 ns | 3,300 ns | 50.0% lower |
| throughput, 100 subscribers / 4 KiB | 168,492.00/s | 334,815.59/s | 98.7% higher |
| bounded replacement depth RMW | 17,280 | 0 | 100% fewer |
| one-subscriber payload Arc clones | 1 | 0 | 100% fewer |

The full publish matrix P95 ranged from 300 ns (one subscriber, 64 B or 256 KiB) to 6,700 ns
(100 subscribers, 4 KiB). All 12 cases retained zero delivery-lock wait samples. The bounded
pressure case retained 4,194,304 bytes at capacity 64 while processing 8,704 publishes and 8,640
replacements.

## Validation

- Exact Rust formatting passed for all eight changed Rust paths.
- Scoped `git diff --check` passed; output contained only the repository's LF-to-CRLF checkout
  notices.
- The exact publish/subscriber/diagnostics source-order guard passed all five checks, including
  absence of the retired decrement-plus-increment overflow path and a cloned last-subscriber
  payload.
- The corrected two-command validator has SHA-256
  `76906b4f03146c4c38149d3ef5de6f5fbcdf103510c8256916bc2104fed801b5` and zero PowerShell parser
  errors. It rejects incomplete matrices, less than 98% routine timestamp reduction, incorrect
  clone/RMW counts, invalid percentile order, or sampled P95/throughput regression over 5% versus
  the exhaustive mode.
- Managed Cargo job `1ccefd55b24740558cba5b07fa6f823d`, run
  `4479497a25944e20bd16c620a4e3d451`, input manifest
  `1ae8cb6e54ac44256635d95e48a823c82f25f898a2b3c588c26bb24309b721a2` reported 25 behavior tests
  passed with 3 ignored release tests, then 3/3 ignored release evidence tests passed.
- The original parser rejected the legal empty percentile prefix only after both Cargo stages had
  exited zero. Its terminal evidence was frozen at SHA-256
  `1217c09055ba90383bfe2fdf78c1e5f4758939d17e2fe121b6dc8a43c3e3b3e0`; replay through the corrected
  validator passed 12 publish rows, 2 sampling rows, and 1 pressure row.
- Coordinator Main replay run `e3880656eefa4064aaa5920b37a1cb4d` accepted the frozen Runtime02
  rows together with App02, Runtime06, and Runtime04. The terminal contract pins 19 source Cargo
  groups, 30 Python tests, one real GPU run, and the exact Runtime02 12 publish / 2 sampling / 1
  pressure evidence matrix.
