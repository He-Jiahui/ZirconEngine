# M1 Resource Management Current-Source Locator Index

Date: 2026-08-27

## Status

- architecture review: `complete`
- current consumer audit: `complete`
- reference-engine review: `complete`
- read-side release measurement: `complete`
- write-side pre/post measurement: `complete`
- production optimization: `source_implemented`
- exact-source semantic validation: `green`
- independent review: `complete`, `C0/I0/M0`
- managed Cargo/product trace/power qualification: `pending`
- M1 acceptance: `not_claimed`

This is a source-implementation record, not M1 acceptance. The read bottleneck was measured before
production code changed, the immutable-generation-owned candidate was measured on both read and write
paths, and exact production owners were acquired through the coordinator. No compatibility path,
forwarding API, Editor-owned cache, or second Resource authority was added. Managed Cargo, an Editor
product trace, and system power qualification remain mandatory before acceptance.

## Frozen Inputs

| Input | Pre-change SHA-256 | Current SHA-256 |
|---|---|---|
| `management_generation.rs` | `db644452c7fee0ba9778b45ba2ddf3f4947a5f8df3bf8c90e749564ffcc0874d` | `4943856a4f71de4202fcd6c03092d1b6e56917e2b2eb570cc10d6160259034db` |
| `manager/management_projection.rs` | `64c454d0bc91ccb908f2709574d69fe673accf59739ba8debce52cd207672458` | `b3c1c90d1472f34f78390ae1fe333d4ac8770ea7817d1a9f2ad616e4cbe8887c` |
| `management_generation/tests/projection.rs` | current-source caller | `36eed2ecdea16c617f51cf00aad4327b2ebecb91d305fb4751002813e4924f88` |
| `core/resource/mod.rs` | current-source facade | `abaffed6fe5c125cab6011d8d7b324070303e6d379ed4f41b8b2347e10798757` |
| Editor `asset_workspace_state.rs` | `d098e758b66a7a5804a6557f027a099e7a4456162a80ed91a8588d7137713759` | not modified |
| Unreal `AssetRegistryState.h` | `c639dfec1f433c3385034a975a86b6d83e8c245b03fff40819fc38b5a61f8052` | reference only |
| Unreal `AssetRegistryState.cpp` | `4057f5591878eba609f0bafe1a3c00fec3e145fe794ad9a852313f58e1d92e0a` | reference only |

The current generation owns 64 immutable ID-hashed shards. Each shard stores locator-sorted rows and
an ID hash index. A generation update copies only changed shards; ID lookup is one shard hash lookup;
locator lookup binary-searches every shard; scan and page perform a 64-way heap merge.

## Whole-Module Consumer Audit

- `ProjectAssetManager` uses stateful `ResourceManagementScan` for kind queries and one full management
  record-set traversal. It does not restart offset pagination.
- dynamic-scene reload reconciliation retains a stateful scan across scheduler budgets. It does not
  revisit an already consumed shard prefix.
- Editor asset workspace calls `row_by_locator` while materializing each asset item and again for the
  selected item. Its incremental update path limits work to changed locators, but a full catalog
  projection can issue one locator lookup per asset.
- `page(offset, limit)` has only focused test consumers in the current source. Deep-page restart cost is
  therefore an architectural risk, not yet a demonstrated product hotspot.

This changes the priority from the earlier generic "deep pagination" hypothesis: the first measured
candidate is locator lookup during Editor projection. Page/cursor work remains rejected unless a real
production consumer or product trace establishes it.

## Reference Direction

Unreal `FAssetRegistryState` keeps ObjectPath, PackageName, PackagePath and other accelerators under the
same state owner. `AddAssetData`, `UpdateAssetData` and `RemoveAssetDatasImpl` update those accelerators
with the authoritative asset mutation. The applicable rule is not to copy Unreal containers literally;
it is to keep every accepted secondary index inside the immutable Resource generation/transaction owner
and update it atomically with ID shards. An Editor-owned cache cannot become a second authority.

## Release Measurement Matrix

The D-drive harness includes the exact current `management_generation.rs` implementation and uses
ABI-shape stand-ins for the interface DTOs so the query/index algorithms can be compiled without the
dirty workspace Cargo graph. This is an isolated algorithm preflight, not product validation.

Record counts: `1,000`, `10,000`, `100,000`.

Scenarios:

1. random `row_by_id` lookup;
2. random current `row_by_locator` lookup;
3. comparison-only locator `HashMap` accelerator model;
4. complete stateful scan;
5. first page (`offset=0`, `limit=64`);
6. deep page (`offset=N-64`, `limit=64`);
7. repeated offset pages versus one stateful scan for 1k/10k.

Each result uses release optimization, `target-cpu=native`, warm-up, at least 11 samples and median/p95
reporting. Outputs and temporary files stay under
`D:\zircon-validation\frameworks01-resource-management-read-profile-20260827`.

## Decision Gates

- A locator accelerator is admitted for design only if current locator lookup is materially slower than
  the model across 10k and 100k and the projected full-Editor traversal cost is large enough to matter.
- Any accepted locator index must be immutable-generation-owned, structurally shared/sharded, and updated
  in the same Resource mutation commit. A global map copied on every generation and an Editor cache are
  both rejected.
- Offset pagination is not changed from isolated data alone. A production consumer must first exist, or
  a product trace must demonstrate repeated prefix work. The existing stateful scan remains the preferred
  continuation mechanism.
- Write-side shard strategy is not changed from read-side evidence. Publish/update cost requires its own
  mutation profile with uniform and skewed workloads.
- No isolated result is a frame-time, power, full-engine, or Unreal-parity claim. Those require a managed
  product build, trusted trace, representative project and system-level power capture.

## Read-Side Pre/Post Result

Rust 1.94.1 compiled the frozen harness with `-C opt-level=3 -C target-cpu=native -C debuginfo=1`.
Each matrix uses warm-up plus 11 samples. The paired rerun fixes the same input generator, sample count,
query count and machine for both production snapshots. An earlier 50,000-query run exceeded the command
limit after partial output and is excluded.

| Random locator lookup | Pre median | Post median | Improvement |
|---|---:|---:|---:|
| 1,000 records | 2,947.740 ns | 170.273 ns | 17.31x |
| 10,000 records | 31,342.000 ns | 540.427 ns | 57.99x |
| 100,000 records | 118,931.573 ns | 911.293 ns | 130.51x |

The isolated full-projection extrapolation, one locator query per row, changes from 2.948/313.420/
11,893.157 ms to 0.170/5.404/91.129 ms at 1k/10k/100k. These are algorithm-model values, not measured
Editor frame times. Stateful scan remains `O(N)` and is unchanged. Repeated deep offset pagination is
still expensive, but no production caller was found, so no cursor/page API change was admitted.

Frozen evidence:

- pre executable `resource_management_read_profile_release_r3.exe`:
  `59864ebffa0fb54ebf30e06c2439a2fa91fe363e3d83780ad7a9fc5da520779c`;
- pre rerun CSV: `02f63c73944d12ddf589508663329a297cfcad01f6e59b4a81a7cc1a60d75395`;
- post executable `resource_management_read_profile_release_r4.exe`:
  `01467b2511c02d9f023ba22b8957edcd099ee410fd1e196a684b6314c1355a0a`;
- post rerun CSV: `db2d1f9e0b7e2b6af58b461ca487931aef03e2aed9025444ca246127be6de20c`.

## Write-Side Pre/Post Result

The same isolated model measured 100k-record mutation publication. Medians are reported because p95 was
more sensitive to shared-machine scheduling. Ordinary record updates reuse the locator index; locator
membership changes copy only affected locator shards.

| Mutation, 100k records | Pre median | Post median | Delta |
|---|---:|---:|---:|
| revision update | 535,312.500 ns | 507,025.000 ns | -5.28% |
| uniform revision update | 1,246,692.188 ns | 1,170,108.594 ns | -6.14% |
| ID-skewed revision update | 10,133.594 ns | 8,801.562 ns | -13.14% |
| add/remove | 481,787.500 ns | 586,350.000 ns | +21.70% |
| rename | 488,425.000 ns | 658,250.000 ns | +34.77% |
| locator-skewed rename | 1,179,634.375 ns | 1,305,965.625 ns | +10.71% |

The write regression is intentionally paid only by locator-membership changes; it avoids an `O(N)`
generation-wide map clone and leaves the dominant non-locator update cases shared. At 100k entries the
64 locator maps retain capacity 114,688 (1.1469 capacity/entry). A conservative container/Arc/id model is
about 6.87 MiB incremental; locator text is shared through `Arc<str>`, not duplicated.

Frozen evidence:

- pre executable `6c7111362a50bcf579c966723f1799a23dcf8ba7326ff98dbde6e645e40f8b52`;
- pre rerun CSV `c2ba5fad6ff5eca00d25645b25cea9ef5a243e1a3843a308393cf3b0eda1bc91`;
- post executable `468e566dec301f95012fe7c8eab599f4c677391131e42cf7a41c195141c5a0c5`;
- post rerun CSV `e21ae2da6497d8097eb6c3d32dedb2a59af2f5cffaf17ecc53f63c72455b5533`;
- capacity CSV `e527e49e970ae57fe72d2343e4f94c8b778454e6ca4ce275718e6e0d37062d5e`.

## Production Implementation

- `ResourceManagementGeneration` now owns 64 immutable locator-hash shards containing
  `Arc<str> -> ResourceId`; `row_by_locator` hashes once, resolves the ID, then uses the existing ID shard.
- `apply_delta` clones only locator shards touched by add/remove/rename and shares the entire locator index
  for revision/state updates. All removals are applied before insertions, so a valid replacement cannot
  erase its successor. Batch conflict legality remains the transaction preflight owner's responsibility.
- the old per-ID-shard locator binary search was removed. `core::resource` re-exports locator shard
  internals only as `pub(crate)` for the manager; no public API or compatibility path was added.
- focused tests cover non-derived ID lookup, same-locator replacement, swap ordering, outer-Arc sharing,
  unaffected-shard identity, and a source guard against rebuilding a linear locator scan.

## Validation And Review

- exact-source smoke includes the final production generation and projection implementation, keeps debug
  assertions enabled, and reports `records=10000 locator_entries=10000 status=green`; executable SHA-256
  is `08016b558cf0a65031332b5b1d9a89975fb6e3f8d2ca4c4ed3f1f8e43480980a`;
- TDD first caught a 10,000-to-9,999 entry loss during a two-locator swap; the root cause was interleaved
  delete/insert ordering, fixed with the two-phase mutation above;
- Resource consumer-manifest tests are 16/16 GREEN. The combined boundary run is 17 passed/4 failed;
  all four failures are the pre-existing M1 physical `zr_resource` hard-cut RED (missing crate root,
  workspace membership, runtime facade re-export, and hidden assembly surface), not locator-index failures;
- rustfmt and scoped `git diff --check` are GREEN. Independent read-only review found C0/I0/M0 on the
  generation, projection, tests, and crate-private facade integration;
- the cross-session Tooling Cargo job `57769fdbe2394a6f9919dc66b5f81946` is durably `released`
  (exit 1, no live PID), so the short Cargo window has ended. The resumed managed Windows `zircon_runtime`
  `core-min` check was rejected before Cargo by `unmanaged_artifacts_detected` for foreign path
  `D:\ZirconBuilds\tooling15-wave163-runtime-20260827-135832`; Frameworks01 did not delete or claim it.
  The lowest shared cause is already tracked by Tooling15's canonical
  [`integrated-bootstrap-artifact-lease`](../../../optimize/zircon_tooling/15/failure-2026-08-27-integrated-bootstrap-artifact-lease.md)
  failure, so no duplicate handoff was created.
  No compile ticket, product frame-time, wattage, or cross-engine parity claim is recorded.

All generated harnesses, binaries, PDBs and CSVs are under
`D:\zircon-validation\frameworks01-resource-management-read-profile-20260827`; none were placed on C.

## Atomic Write Compile-Fingerprint Recheck

UI12 reported three E0432 imports from an older isolated runtime fingerprint. Current source retains
exactly one curated public single-file entry, `core::resource::io::atomic_write`, from private
`atomic_file`; `io/mod.rs` SHA-256 is
`5b5d5dc7c9c5c9b8b7fa89b5cc11e33b4a1625044c6f686e0fec8ed0df9015c5`.
`ibl_bake_artifact_asset_derived.rs` and `ibl_bake_artifact_cache.rs` correctly consume that facade.
`ibl_source_cubemap_staging.rs` current SHA-256
`d7b131180aa24e2333b614f0219898e5967dd5574f6a702a77309463a17649af` already consumes
`io::transaction::{commit_prepared_files, ...}` and has no `atomic_write` import. Therefore no consumer
migration, compatibility export, or foreign Shader06 blob rewrite is required; UI12 must rebuild from a
current source snapshot after its shared Cargo window is released.

## Ownership Gate

The coordinator transferred the archived `management_generation.rs`, projection test and
`core/resource/mod.rs` blobs to the active Frameworks01 session by exact fingerprints. The active session
already owned `management_projection.rs` and this plan record. Final current hashes must be attributed
again before any integration candidate; accepted closeout remains blocked on managed compilation rather
than ownership.
