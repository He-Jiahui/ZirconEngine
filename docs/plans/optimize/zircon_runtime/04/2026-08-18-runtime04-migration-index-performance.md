Plan: docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
Fixing plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
Milestone: indexed resolver, managed scale matrix and missing-subasset identity
Status: passed

# Runtime04 migration index and scale performance

## Delivered

- One generation-owned resolver index serves reference lookup without per-reference filesystem
  fallback. Physical aliases, distinct logical roots, compound sidecars and rejected link/reparse
  paths retain typed production semantics.
- Migration reports expose production entry visits, directory reads/sorts, document reads/parses,
  actual resolver-index lookups and pending output counts. The lookup counter lives in both index
  query entrances, so successful, missing and rejected queries all contribute. Zero filesystem
  probing and zero whole-document clones are source-bound invariants rather than default-zero
  runtime fields.
- A missing subasset label is a typed dangling-subasset result with stable same-source candidates;
  GUID and path-hint resolution no longer erase semantic identity by falling back to the parent
  asset.

## Performance protocol

- Five sequential repetitions, one Cargo worker and one test thread.
- Seventeen groups per repetition and 85 raw samples total: resolver build for 1/4 roots, resolver
  lookup for 1/1K/100K references at both root counts, and migration scale for 1/1K/100K
  files/references/directories.
- Percentiles use nearest-rank P50/P95/P99. Acceptance also requires exactly five samples per group,
  bounded resolver-index queries per project-reference visit, equal directory read/sort counts and
  equal document read/parse counts. Dedicated behavior tests cover successful, missing and rejected
  index queries; source guards cover the no-filesystem and no-whole-document-clone boundaries.
- The uncommitted default-zero `resolver_filesystem_probes` and `full_value_clones` getters were
  removed before acceptance; `resolver_index_lookups` is the replacement measured contract.
- Every group has an explicit Windows P95 ceiling. The evidence replay fails when any measured P95
  exceeds that ceiling; percentile ordering alone is not sufficient.

## Managed performance result

| Group | P50 (ms) | P95 (ms) | P95 ceiling (ms) |
|---|---:|---:|---:|
| migration directories / 1 | 5.706 | 6.233 | 20 |
| migration directories / 1K | 417.352 | 498.622 | 1,000 |
| migration directories / 100K | 45,351.436 | 46,081.886 | 60,000 |
| migration files / 1 | 16.360 | 25.639 | 50 |
| migration files / 1K | 1,159.466 | 1,233.782 | 2,000 |
| migration files / 100K | 132,366.731 | 137,170.815 | 160,000 |
| migration references / 1 | 13.975 | 18.486 | 50 |
| migration references / 1K | 143.028 | 159.802 | 500 |
| migration references / 100K | 12,646.745 | 13,909.106 | 20,000 |
| resolver build / 1 root / 100K entries | 1,118.167 | 1,147.101 | 2,000 |
| resolver build / 4 roots / 100K entries | 1,606.859 | 1,828.100 | 3,000 |
| resolver lookup / 1 root / 1 ref | 0.165 | 0.223 | 2 |
| resolver lookup / 1 root / 1K refs | 2.161 | 2.507 | 10 |
| resolver lookup / 1 root / 100K refs | 214.028 | 275.822 | 500 |
| resolver lookup / 4 roots / 1 ref | 0.076 | 0.083 | 2 |
| resolver lookup / 4 roots / 1K refs | 3.092 | 3.613 | 10 |
| resolver lookup / 4 roots / 100K refs | 271.192 | 294.080 | 500 |

## Validation

- Source-bound snapshot: `1846`; copy `4872e9b11aa74bcc80da787a06404441` has input-manifest
  `1c6e6c40d7d576d5b575a3778eba32d4301ee9008dc23a6aa77dc67e9ac2371d` and passed the exact
  post-materialization audit `217/217`.
- The preceding run `a1d800fd78dd4c43a2a8896c0dccad0a` stopped in the first compile stage
  before Runtime04 sampling; it is compiler-red evidence and contains zero performance samples.
- Successor request `f71486fef10b4dbf899843f0b885bcac` completed as run
  `5a1ee52cdb27449b86878a76ee712792` with exit 101 after the first App01 cadence behavior test.
  Runtime04 tests and all five performance repetitions did not start; this run contains zero
  Runtime04 samples. The App cadence state repair is frozen for a successor unified batch.
- Covered failure lifecycle keys: `asset-migration-indexed-resolver-generation`,
  `asset-migration-scale-acceptance-matrix`, and `missing-subasset-parent-fallback`.
- Current-source job `197e37fe25f94d00915fcd890b03724d`, run
  `1562528434194a17879de2abbc2dbebf`, completed all five resolver and migration repetitions with
  85/85 samples and zero Rust test failures. Its wrapper then failed only while formatting the
  first performance row because PowerShell parsed the `-f` comma list as method arguments.
- The corrected validator freezes the 121 relevant terminal rows at SHA-256
  `95f78031ad1f44ab27b3abed38c798b20ce53c207c36619a4498d72a2cbb01ba`, restores all ten
  successful stage boundaries, independently rebuilds the 5 x 17 matrix, and enforces the P95
  ceilings above without rerunning Cargo.
- Coordinator replay run `e3880656eefa4064aaa5920b37a1cb4d` exited zero with 17 performance
  rows and terminal `repeats=5 groups=17 samples=85 percentile=nearest_rank`.
