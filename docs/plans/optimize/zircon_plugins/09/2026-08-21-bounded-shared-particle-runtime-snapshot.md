# Plugins09 Bounded Shared Particle Runtime Snapshot Optimization Record

- Date: 2026-08-21
- Owner: `optimize-plugins09-particle-snapshot-r1-01a00797-20260821`
- Source plan: `docs/plans/optimize/zircon_plugins/09-first-party-particle-vfx-source-runtime-editor-dist-catalog-simulation-render-product-integration-review.md`, PVFX-P1-033
- Status: implementation complete; combined managed validation pending

## Problem

`ParticlesManager` retained runtime diagnostics in an unbounded `Vec`, and every
snapshot deep-cloned both the full sprite payload and the full diagnostic
payload. Repeated diagnostics could therefore grow retained memory without a
limit, while unchanged snapshot readers paid payload-proportional allocation
and copy costs on every call.

## Change

- Diagnostics now use a 256-entry sequenced ring. Overwrite count and stale
  cursors are explicit; page reads are capped at 64 entries.
- Diagnostic sequence allocation emits `u64::MAX` at most once. After the
  sequence space is exhausted, new diagnostics increment the explicit dropped
  count instead of reusing an existing paging identity.
- The manager exposes sequence-based paging and manager-global acknowledgement.
  Acknowledged entries are removed and the diagnostic snapshot is invalidated.
- Sprite and diagnostic payloads are stored as `Arc<[T]>`. An unchanged manager
  snapshot reuses both allocations; simulation changes rebuild sprites while
  retaining the unchanged diagnostic allocation.
- Snapshot construction appends all instance sprites into one buffer rather
  than allocating an intermediate vector for every instance.
- Render extraction retains one owned sprite copy because camera sorting and
  renderer ownership require mutable, frame-local storage.

## Deterministic Performance Evidence

The managed release gate measures complete snapshot clones with 4,096 sprites
and 256 diagnostics. The legacy branch deep-clones both large payloads; the
optimized branch clones their shared handles. Emitter metadata and GPU feedback
are still cloned identically by both branches.

| Measure | Legacy | Optimized | Gate |
|---|---:|---:|---:|
| Snapshot clones per sample | 128 | 128 | exact |
| Large payload elements cloned per sample | 557,056 | 0 | eliminated |
| Timing distribution | 21 samples | 21 samples | alternating first-run order |
| Nearest-rank P95 | pending | pending | optimized <= 25% of legacy |

Exact Windows P50/P95 values remain pending the combined coordinator batch and
must be written here before integration acceptance.

The pinned Plugins09 child validator is
`zircon-validation-plugins09-particle-snapshot.ps1` at SHA-256
`DEA35FC7689F8D05DD8542191FCCB29ADADC9B7A16C359F837B28D3B2D0F940A`.
It is aggregated with fourteen existing plugin tasks by
`zircon-validation-plugin-super-batch-six.ps1` at SHA-256
`8706E8D5487255392CFF8388CCF4B0C9AEBC63A9C948DF943B6BAA7664A0B698`.

## Acceptance

- A 300-diagnostic corpus retains sequences 45 through 300, records 44 dropped
  entries, reports a stale zero cursor, and caps the first page at 64 entries.
- Acknowledging the first page removes exactly 64 entries and rebuilds the
  shared diagnostic payload; the next page begins at sequence 109.
- Sequence exhaustion preserves the final `u64::MAX` entry, rejects the next
  diagnostic as dropped, and never emits a duplicate paging sequence.
- Unchanged snapshots share sprite and diagnostic allocations. A simulation
  tick replaces the sprite allocation without replacing unchanged diagnostics.
- `particle_snapshot_shared_clone_release_benchmark` emits 21 alternating raw
  sample pairs, recomputable nearest-rank P50/P95 values, and exact payload clone
  counts.
- Exact-file Rustfmt, scoped diff checks, Cargo regressions, and release timing
  are required in one managed multi-task Windows validation copy. No per-task
  Cargo invocation is used.

## Remaining Scope

Acknowledgement is manager-global rather than subscriber-specific. Independent
diagnostic consumers need subscriber identity and per-subscriber cursors before
one consumer can acknowledge without affecting another. This slice also does
not remove the renderer's required owned/sortable frame copy or split the
manager-wide mutex described by PVFX-P1-030.
