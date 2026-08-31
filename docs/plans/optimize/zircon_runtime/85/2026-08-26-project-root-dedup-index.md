---
title: Runtime85 Project Root Dedup Index
category: zircon_runtime
report_id: Runtime85-project-root-dedup-index-2026-08-26
date: 2026-08-26
session_id: root-runtime85-project-root-dedup-index-20260826
implementation_status: implementation_complete
validation_status: managed_validation_queued
---

# Runtime85 Project Root Dedup Index

## Scope

This slice optimizes duplicate detection while registering canonical project asset roots. It does
not change root canonicalization, containment checks, authored root order, primary-root selection,
typed error precedence, package roots, source discovery, or public asset contracts.

## Change

- `PackageAssetRegistry::register_project_roots` now preallocates a session-local fingerprint set
  instead of linearly scanning every previously accepted canonical path for every new root.
- The set stores only `u64` fingerprints. It therefore adds one bounded table allocation without
  cloning every `PathBuf`; the original ordered `Vec<PathBuf>` remains the publication owner.
- A fingerprint collision falls back to canonical path equality before reporting a duplicate, so
  collisions cannot reject distinct roots and repeated colliding roots remain detectable.
- The existing cross-platform Rust alias-root regression continues to lock canonical identity and
  `DuplicateProjectAssetRoot` behavior. A Python source contract locks capacity sizing, collision
  fallback, no per-root path clone, and authored-order publication.

## Deterministic Performance Evidence

The independent release model uses 1,024 unique canonical roots and 21 alternating legacy/indexed
sample pairs per run. It also appends the last root once and proves both paths report the same
first-seen index. All three runs use the same input paths and include the ordered result-vector
path clones in both variants.

| Evidence | Linear scan | Fingerprint index | Result |
|---|---:|---:|---:|
| Path comparisons / hash probes | 523,776 | 1,024 | 99.804497% fewer operations |
| Collision path comparisons | n/a | 0 | No collision fallback on measured corpus |
| Measured allocations | 1,025 | 1,026 | One bounded table allocation; no per-root clone |
| Run 1 P50 | 172.603 ms | 0.498 ms | 99.712% faster |
| Run 1 P95 | 219.348 ms | 0.681 ms | 99.689% faster |
| Run 2 P50 | 228.988 ms | 0.449 ms | 99.804% faster |
| Run 2 P95 | 548.366 ms | 0.707 ms | 99.871% faster |
| Run 3 P50 | 157.177 ms | 0.416 ms | 99.736% faster |
| Run 3 P95 | 200.818 ms | 0.479 ms | 99.762% faster |

The managed gate requires the exact 523,776-to-1,024 operation counts, zero measured collision
comparisons, at most two allocations above the legacy path, at least 99.8% operation reduction,
at least 70% P50 improvement, and at least 50% P95 improvement.

## Acceptance

- TDD RED observed two missing-index failures while the existing Rust-coverage guard passed.
- `tools.tests.test_runtime85_project_root_dedup_performance_contract` passes 3/3 locally.
- Exact-file `rustfmt --check`, model compilation, three independent model runs, and scoped
  `git diff --check` pass locally.
- The focused Rust canonical-alias regression, source contracts, formatting, performance model,
  and scoped diff checks are submitted together in one coordinator validation batch.
- Commit integration and automatic WeCom performance notification remain gated on managed
  validation and the repository's independent-review policy.

### Recovery Batch 2026-08-31

- Ownership transfer apply: `6779c8224fc74a1aa211d8739d11592e`.
- `tools/runtime85_project_root_dedup_model.rs` restores the allocation, deterministic work, parity,
  and paired P50/P95 gates described above under the current source tree.
- Managed batch script: `tools/zircon-validation-runtime85-project-dedup-recovery-batch.ps1`.
- Coordinator ticket: `pending_submission`; exact current-run performance values will be copied from
  terminal managed evidence before closeout.

## Remaining Parent-plan Work

Runtime85 still needs complete source-dependency identity, deterministic importer recipes,
subasset lineage and remap, build graph/DDC authority, bounded worker scheduling, streaming
artifact sections, signed package/install recovery, and product-scale fault qualification.
