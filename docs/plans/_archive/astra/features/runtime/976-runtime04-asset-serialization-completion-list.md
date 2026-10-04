---
doc_type: completion-list
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/04/2026-08-26-migration-scan-unstable-sort.md
  - docs/plans/optimize/zircon_runtime/04/2026-08-26-pack-dedup-entry-lookup.md
  - docs/plans/optimize/zircon_runtime/04/2026-08-26-pack-delta-capacity.md
  - docs/plans/optimize/zircon_runtime/04/2026-08-26-targeted-capacity.md
  - docs/plans/optimize/zircon_runtime/04/2026-09-09-manifest-bound-and-root-admission.md
  - docs/plans/optimize/zircon_runtime/04/2026-09-21-sidecar-uri-direct.md
related_records:
  - docs/plans/astra/features/runtime/660-manifest-bounded-root-admission.md
  - docs/plans/astra/features/runtime/879-sidecar-uri-direct.md
  - docs/plans/astra/features/runtime/975-runtime03-ui-optimization-completion-list.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/asset/migration/scan.rs
  - zircon_runtime/src/asset/pack/dedup.rs
  - zircon_runtime/src/asset/pack/delta.rs
  - zircon_runtime/src/asset/registry/targeted.rs
  - zircon_runtime/src/asset/project/manifest/load.rs
  - zircon_runtime/src/asset/project/manifest/project_manifest.rs
  - zircon_runtime/src/asset/project/manifest/validation.rs
  - zircon_runtime/src/asset/project/manifest/error.rs
  - zircon_runtime/src/asset/project/manifest/save.rs
  - zircon_runtime/src/asset/migration/sidecar.rs
tests:
  - zircon_runtime/src/asset/migration/scan/optimization_tests.rs
  - zircon_runtime/src/asset/pack/dedup/optimization_tests.rs
  - zircon_runtime/src/asset/pack/delta/optimization_tests.rs
  - zircon_runtime/src/asset/registry/targeted/optimization_tests.rs
  - zircon_runtime/src/asset/project/manifest/save/borrowed_serialization_tests.rs
  - zircon_runtime/src/asset/project/manifest/validation/astra_root_tests.rs
  - zircon_runtime/src/asset/project/manifest/validation/optimization_batch_ir_runtime629_tests.rs
  - zircon_runtime/src/asset/migration/sidecar/direct_uri_tests.rs
  - tools/tests/test_runtime879_sidecar_uri_direct_performance_contract.py
---

# Runtime04 · asset and serialization completion list

This batch records six implementation-complete Runtime04 asset/serialization slices that were
present in `docs/plans/optimize` but did not yet have a matching Astra completion entry. The
changes preserve ordering, duplicate handling, manifest wire compatibility, validation error
precedence, and sidecar URI normalization while removing avoidable lookups, growth, and temporary
strings. Tooling migration remains deferred.

| Plan slice | Optimization boundary | Acceptance boundary | Status |
| --- | --- | --- | --- |
| Migration scan unstable sort | Directory entries, root-relative identities, and the shared path sort/dedup helper use unstable ordering where complete comparison keys make stability unobservable; dedup keys are unchanged. | `RUNTIME04_MIGRATION_SCAN_UNSTABLE_SORT_BENCH_V1`; Release P95 must be at most 95% of the stable-sort baseline with `stable_sorts=3->0`. | implemented_pending_validation |
| Pack dedup entry lookup | `ZrPackDedupTable::insert_or_get` uses one `BTreeMap::entry` admission instead of a `get` followed by a second insert lookup, preserving first-index semantics. | `RUNTIME04_PACK_DEDUP_ENTRY_LOOKUP_BENCH_V1`; Release P95 must be at most 95% of legacy with `btree_lookups=2->1`. | implemented_pending_validation |
| Pack delta capacity | Removed/changed/reused result vectors reserve bounded base/target counts and the chunk output reserves the unique changed-hash count; ordering and manifest semantics remain unchanged. | `RUNTIME04_PACK_DELTA_CAPACITY_BENCH_V1`; Release P95 must be at most 95% of legacy and report reservation reductions. | implemented_pending_validation |
| Targeted registry capacity | Source-entry projection reserves the known UUID-set length and dependency tuples reserve the root-plus-entry lower bound after one root check. | `RUNTIME04_TARGETED_CAPACITY_BENCH_V1`; source/dependency Release P95 pairs and reservation reductions are required. | implemented_pending_validation |
| Manifest byte bound and root admission | Loads reject oversized input before materialization, file reads are bounded to limit-plus-one, roots use count/path-boundary admission, and UI duplicate checks borrow canonical paths without formatting a URI. | Runtime85/Astra source contracts plus managed Cargo, allocator, Release, and project-boundary gates; no dynamic timing claim is made here. | implemented_pending_validation |
| Sidecar URI direct | Missing-sidecar URI construction appends normalized components directly into the final URI, removing the component vector, joined child, and second formatted wrapper while retaining validation and lossy-Unicode behavior. | `RUNTIME879_SIDECAR_URI_DIRECT_BENCH_V1`; alternating Release P95 must stay within the plan's 110% guard, with allocation/product gates still required. | implemented_pending_validation |

## Local evidence and validation boundary

The listed source contracts, equivalence regressions, and ignored benchmark markers are wired for
one grouped Runtime package validation request. This record does not claim a Cargo result, a
current-source Release percentile, allocator counts, or product acceptance. The grouped managed
Runtime/Editor gates remain coordinator-owned and are intentionally not polled from this session.

## Source snapshots

| Owner | SHA-256 |
| --- | --- |
| `zircon_runtime/src/asset/migration/scan.rs` | `3C19411E9C67D42B08044BEC9ACCBB3BF2A0B86BFE7DBE27D62055AABDC00DBB` |
| `zircon_runtime/src/asset/migration/scan/optimization_tests.rs` | `3530F1679C9D8475941AC231217DDE4EF033C62A3AEFD9BF7BCBFD1C02267A5A` |
| `zircon_runtime/src/asset/pack/dedup.rs` | `9BF67C23EA221ABAD335A77DBF409332A3FE56CD353088AE6755DD070820C668` |
| `zircon_runtime/src/asset/pack/dedup/optimization_tests.rs` | `AAEEC99A2E7CC755C5E56F4CA707C3D2A34C363EAEE5FA390886D2A58A4FAF41` |
| `zircon_runtime/src/asset/pack/delta.rs` | `FA490B262084F13B006B324D87D48E0C0E8FCC9DCD6299A6C06042C68FF7B07` |
| `zircon_runtime/src/asset/pack/delta/optimization_tests.rs` | `B99978F85CC9C5ABAB5155B95858FD95125031FF1D4D93ECA30A2DFEBEC05995` |
| `zircon_runtime/src/asset/registry/targeted.rs` | `A69D0213B20B7F4FA1573A3E3D7763AD7B5AA883B2AB3E03FDAC9EDE524AEAE7` |
| `zircon_runtime/src/asset/registry/targeted/optimization_tests.rs` | `D929BA938F71C62A7013386F29F680AD66DBE0F0D0387B562B58103B52C65636` |
| `zircon_runtime/src/asset/project/manifest/load.rs` | `3B406D030D29D2DB0E273822CCB1E620A842FEEF147DE061F3D1EE9451EFA841` |
| `zircon_runtime/src/asset/project/manifest/project_manifest.rs` | `862ECCBA6CA7F434D371065C86290F69C6AEFFA613F2AA2B824AEFFDAFEC7464` |
| `zircon_runtime/src/asset/project/manifest/validation.rs` | `1CC8C8552D43D4F2237DF92C70B871DCDDBD6209C1F25417432A1DC5558AE6B3` |
| `zircon_runtime/src/asset/project/manifest/error.rs` | `D394E4AB51BAA584AAF75B89B5055E494CD48B211B9F7A762ABA205FE9C9C727` |
| `zircon_runtime/src/asset/project/manifest/save.rs` | `8A251474650C8ADD87FD927E13F567FBB29E2DE947B59CC6F1C86617DAA85967` |
| `zircon_runtime/src/asset/project/manifest/save/borrowed_serialization_tests.rs` | `A392BD1868FB99E04D77BE594FA18DA7309D2D4A87656DE411786B458A7CB9EF` |
| `zircon_runtime/src/asset/project/manifest/validation/astra_root_tests.rs` | `1D9CEF99AD2AB4AC3DFF1D867331003D10645DBBD59E2D914CCF885B6D474EA1` |
| `zircon_runtime/src/asset/project/manifest/validation/optimization_batch_ir_runtime629_tests.rs` | `C59FA854F39E27F6A7872B4D69DF0C826102D1C78920567E680CD52955914010` |
| `zircon_runtime/src/asset/migration/sidecar.rs` | `D196B0D396197B4045A2BDE6F7274DB84CA7276A25D708F460C479AE0EB9327D` |
| `zircon_runtime/src/asset/migration/sidecar/direct_uri_tests.rs` | `3BDA7648DD58876BCCB243D365C02C762FC033A5B77E99E8BCFAC343456DBA99` |
| `tools/tests/test_runtime879_sidecar_uri_direct_performance_contract.py` | `8BF4822168112230A2C8F2E87EDB1B7EDD4A65DBD58FBC74F6E6D1D84607B506` |
