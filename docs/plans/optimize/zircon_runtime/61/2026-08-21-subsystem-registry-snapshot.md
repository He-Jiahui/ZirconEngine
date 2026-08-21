# Runtime61 immutable subsystem registry snapshot

- Owner: `optimize-runtime61-subsystem-snapshot-r1-01a00797-20260821`
- Source plan: `61-runtime-scene-world-level-registry-lifecycle-project-io-snapshot-clone-serialization-schema-transaction-product-integration-review.md`
- Finding: `RWL-P2-004`
- Status: implementation and deterministic clone evidence complete; combined managed Cargo validation pending

## Problem

`LevelSystem::registered_subsystems()` cloned every registered subsystem name while holding the
registry mutex. Diagnostics that repeatedly inspect an unchanged level therefore paid allocation and
string-copy cost proportional to the full registry on every read, and every read produced an
unrelated allocation even when the registry was unchanged.

## Change

The Level registry now publishes `Arc<[String]>`. Registration remains the low-frequency write path:
it copies the current slice, appends the new name, and atomically replaces the published snapshot
under the existing mutex. Reads hold the mutex only long enough to clone the Arc. Pointer identity
therefore identifies an unchanged immutable view without a finite generation counter or overflow
policy.

Existing semantics are preserved: subsystem names remain in insertion order, duplicates remain
allowed, cloned `LevelSystem` handles still observe the same registry, and poison recovery behavior is
unchanged. The return type is a deliberate hard cut to the immutable snapshot; a workspace-wide call
site scan found no production consumer requiring a compatibility Vec clone.

## Deterministic evidence

The release workload registers 1,024 subsystem names and reads the unchanged registry 4,096 times.
The legacy model clones every string per read, while the production API clones one Arc per read.

| Metric | Legacy Vec read | Immutable snapshot read | Reduction |
| --- | ---: | ---: | ---: |
| Clone work units | 4,194,304 strings | 4,096 Arc references | 99.902% |
| Unchanged snapshot identity | Unrelated allocation per read | Shared Arc pointer | Explicit |

The benchmark runs 21 alternating legacy/snapshot sample pairs and emits raw arrays plus
independently recomputable nearest-rank P50/P95 values. The release gate requires snapshot P95 and
structural clone work to each use at most 25% of legacy. Exact Windows timing remains pending until
the post-Main combined batch; it is not inferred from deterministic clone counts.

## Acceptance

- `subsystem_registry_reads_share_one_immutable_snapshot` proves repeated reads share the same Arc,
  registration publishes a new allocation, name order is stable, and an older snapshot remains
  immutable.
- `level_subsystem_registry_snapshot_release_benchmark` emits 21 alternating timing pairs, validates
  observed entry counts, and enforces the 75% P95 threshold.
- The managed eleven-task Runtime follow-up batch runs LevelSystem regressions and the ignored release
  gate together; this session launches no per-task Cargo process.

Pinned validation artifacts:

- Runtime61 child: `zircon-validation-runtime61-subsystem-registry-snapshot.ps1`, SHA-256
  `FED32F4205E788323533CD3DD3938BD4DAA0A101581B9364D4F4EACBFFB39AF3`.
- Eleven-task Runtime batch: `zircon-validation-runtime-rust-followup-eleven.ps1`, SHA-256
  `8B0AA8283EBAF7B8512A62EC8083B0CCAD8ED0BCF725B4B9B546A0F9C5855878`.
- Both scripts parse with zero PowerShell AST errors. Windows release timing, compilation, and test
  results remain pending until the post-Main materialized batch executes.

## Remaining scope

This closes only `RWL-P2-004`. Runtime61's five P0 blockers, 60 P1 findings, and remaining 13 P2
findings stay open, including lossless authoring snapshots, exact play-world restore, lifecycle
transactions, schema-driven serialization, durable save/load operations, and product qualification.
