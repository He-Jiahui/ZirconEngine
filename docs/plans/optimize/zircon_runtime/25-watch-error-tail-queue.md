# Runtime25 Watch Error Tail-Queue Optimization

- Source review: `docs/plans/optimize/zircon_runtime/25-filesystem-path-uri-vfs-mount-watch-sandbox-atomic-io-review.md`
- Owner: `optimize-runtime25-watch-error-tail-queue-r1-01a00797-20260820`
- Status: implementation complete; combined managed validation pending

## Scope

This slice preserves the existing bounded watcher error and reconciliation
contract while removing prefix movement from the activation lock. It does not
change watch event mapping, project generation, retry, or publication policy.

## Contract

- Retain at most 64 pending `AssetWatchError` values.
- On overflow, discard the oldest error and require project reconciliation.
- Broadcast retained errors in FIFO order.
- Use constant-time tail-queue admission instead of `Vec::remove(0)`.

## Acceptance

- A focused regression overflows the queue by two entries and proves capacity,
  oldest eviction, FIFO order, and reconciliation.
- A release benchmark runs 21 alternating legacy/optimized sample pairs for
  200,000 admissions at capacity 64.
- The validator independently recomputes nearest-rank P50/P95 from all raw
  samples and requires optimized P95 to be at most 75% of legacy P95.
- Cargo regression and release evidence run only in a serialized multi-task
  Windows coordinator batch.

## Remaining Runtime25 Scope

The parent Runtime25 plan still owns the filesystem provider/mount model,
portable URI codec, secure-open capability, watch mapping outcomes, I/O
scheduling, and cross-platform qualification. This slice closes only the
bounded watcher-error admission data movement.
