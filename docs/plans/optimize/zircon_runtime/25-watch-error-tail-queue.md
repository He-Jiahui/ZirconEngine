# Runtime25 Watch Error Tail-Queue Optimization

- Source review: `docs/plans/optimize/zircon_runtime/25-filesystem-path-uri-vfs-mount-watch-sandbox-atomic-io-review.md`
- Owner: `optimize-runtime25-watch-error-tail-queue-r1-01a00797-20260820`
- Status: implementation and focused static validation complete; managed release batch queued

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

## Current Execution Evidence

- Integration Session: `root-runtime-interface03-activate-link-failure-20260831`;
  ownership apply `4d4f16a2b15749efbe8f598b0f952bc0`, preview
  `4e898c3ed0724e668caf944bf25d3d1b`, fingerprint
  `aec6b18d14d1538bdfac602af684c2971afa2b8468cce6ed5d755567a5fc2ecd`.
- Current source SHA-256:
  `F537D196BC32DBCC0864966D270D80A77BC6E393ED352BAFE50F7558E1C62D7D`.
- Deterministic model manifest SHA-256:
  `EEB0CEB289E7CE32681420FF71872E23FCA174F1C7BD6D8E0A8860D3D2FE631D`.
  At 200,000 admissions and capacity 64 it records 199,936 overflows and
  prefix record moves `12,595,968 -> 0`, preserving 64 retained errors.
- Focused source/model/validator contract passed locally `7/7`; Python and
  PowerShell syntax, exact Rustfmt, and scoped diff checks are green. Managed
  static ticket: `3bf72aa1430e4e289b028355f9f852ec`.
- FIFO/reconciliation regression and the exact ignored release benchmark are
  queued together in ticket `ea3432d3bfb94435840108dc30e253a0`, using
  validator SHA-256
  `90086EEE602F0382AA6550D2DAF4BBA95644ED582A0545BC3BD5C04A4282E54B`.
  Measured P50/P95 remain pending.

## Remaining Runtime25 Scope

The parent Runtime25 plan still owns the filesystem provider/mount model,
portable URI codec, secure-open capability, watch mapping outcomes, I/O
scheduling, and cross-platform qualification. This slice closes only the
bounded watcher-error admission data movement.
