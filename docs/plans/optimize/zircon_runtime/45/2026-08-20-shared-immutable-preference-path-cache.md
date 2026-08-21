# Runtime45 Shared Immutable Preference Path Cache

- Date: 2026-08-20
- Owner: `optimize-runtime45-arc-path-cache-r1-01a00797-20260820`
- Integration session: `optimize-runtime49-gizmo-shared-asset-r2-01a00797-20260820`
- Source plan: `docs/plans/optimize/zircon_runtime/45-preference-settings-scope-storage-overlay-bounded-io-generation-fence-durability-migration-multi-process-product-integration-review.md`, PREF-P2-09 / M7
- Status: implementation complete; combined managed validation pending

## Problem

`AtomicFilePreferenceStorageBackend` cached a `PathBuf`, but every cache hit
deep-cloned that path before releasing the cache lock. Each cold miss also
deep-cloned the two-String `PreferenceKey` into both the map and FIFO order.
Hot operations therefore copied immutable path bytes, while a full 4,096-entry
cache retained two independent key/string allocations per logical entry.

## Change

- The bounded path cache stores `Arc<Path>` and returns an Arc clone on hits.
- The map and FIFO order share one `Arc<PreferenceKey>` per logical entry, so a
  miss deep-clones the key once instead of twice.
- Cold misses still build and retain exactly one owned path.
- File operations borrow `&Path` only after the cache lock is released, so no
  filesystem I/O occurs while holding the cache mutex.
- Hashing, storage layout, FIFO eviction, cache capacity, and diagnostics are
  unchanged.

## Deterministic Performance Evidence

| Workload | Before | After | Reduction |
|---|---:|---:|---:|
| short path, 4,096 clones/sample, 21 pairs | 86,016 deep path copies | 0 deep path copies | 100% |
| medium path, 4,096 clones/sample, 21 pairs | 86,016 deep path copies | 0 deep path copies | 100% |
| long path, 4,096 clones/sample, 21 pairs | 86,016 deep path copies | 0 deep path copies | 100% |
| all three path cases | 258,048 deep path copies | 0 deep path copies | 100% |
| 4,096 maximum-size keys/sample, 21 pairs | 172,032 key / 344,064 String clones | 86,016 key / 172,032 String clones | 50% |
| copied key bytes across 21 pairs | 110,100,480 bytes | 55,050,240 bytes | 50% |
| One cache hit | O(path bytes) allocation/copy | O(1) Arc clone | one complexity class |
| Cold cache miss key retention | two deep key clones | one deep key clone + Arc clone | one deep clone removed |

The ignored release benchmark covers short, medium, and long Windows paths.
Each case alternates legacy and shared-first order across 21 pairs, emits every
raw nanosecond sample, and computes nearest-rank P50/P95. Acceptance requires
shared-clone P95 to be no more than 75%/50%/50% of legacy `PathBuf::clone` P95
respectively. A fourth 4,096-key distribution compares the former duplicated
map/FIFO key ownership with the shared Arc owner; it requires shared P95 to be
no more than 75% of legacy P95 and emits all 21 raw pairs. Exact Windows timings
remain pending the serialized coordinator batch.

## Acceptance

- `platform_preference_storage_path_cache_is_stable_and_bounded` now requires
  repeated hits to share the same immutable path allocation and the map/FIFO to
  share one key allocation, while retaining the existing hit/miss/eviction
  assertions.
- `platform_preference_storage_path_cache_shared_clone_release_benchmark_evidence`
  supplies four 21-pair raw distributions: three per-size path clone gates and
  one maximum-size key-retention gate.
- The current combined validator covers Runtime45, Runtime48, Runtime49, and
  five prepared Runtime08C animation slices in eight logical tasks and twelve
  Cargo groups. It has seven independent performance gates, four of which
  remain attributed to this Runtime45 record. Validator SHA-256:
  `A2C1864BDCA19026FD02493EC066031AF95CE6A050E59A608859C64FBC9E0943`.
- Rust 1.94.1 rustfmt and scoped `git diff --check`: passed.
- Cargo regressions and release timing: pending behind the active Main batch.

## Managed Validation Failure And Repair (2026-08-21)

- Immutable copy `c886ad123a1243e6b6244d0f0ca77917`, run
  `d67b02e15b6c4f558143531d2f692a0a`, first passed the Runtime07 baseline gate:
  3 passed, 0 failed across the 1 / 1,000 / 100,000-node workloads.
- The next Runtime45 regression group compiled successfully, then reported 24 passed,
  4 failed, and 1 ignored. All four atomic preference cases failed their first write with
  Windows `os error 3`; no Runtime45 release benchmark or timing row ran. The coordinator
  removed the failed copy normally, so this run is failure evidence rather than performance
  evidence.
- Managed runs place `TEMP` below the already-long immutable target root. Rust filesystem
  calls created that hierarchy, but the atomic commit layer passed ordinary paths directly to
  `MoveFileExW` / `ReplaceFileW`; once a staging or target path crossed the legacy 260-character
  boundary, the raw Win32 call could not resolve it.
- The Windows atomic-file owner now shares a NUL-checked path encoder across create, replace,
  and transaction-replace calls. Existing short, verbatim, and device paths keep their prior
  call shape; only paths that resolve beyond the legacy boundary receive `\\?\` or
  `\\?\UNC\` encoding. This avoids a new `GetFullPathNameW` call on the normal absolute-path
  hot path.
- `platform_preference_storage_atomic_file_supports_long_managed_temp_roots` constructs a
  deterministic over-limit root and covers write plus read through the public preference
  service. Exact rustfmt and scoped diff checks pass; replacement batched Cargo and release
  performance evidence remain pending.

## Remaining Scope

This slice removes hit-path path copies and duplicate map/FIFO key ownership.
It does not change multi-process coordination, durability fences, schema
migration, path identity, or broader preference scheduling work in Runtime45.
