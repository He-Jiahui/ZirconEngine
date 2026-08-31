# Frameworks01 M1 `zr_math` current-source hard-cut preflight (2026-08-24)

## Status

- `preflight_complete`
- `superseded_by_zr_math_physical_hard_cut`
- `ownership_transfer_completed_after_preflight`
- `implementation_record: 2026-08-24-m1-zr-math-physical-hard-cut.md`
- `performance_claims_not_admitted`

This record preserves the current-source architecture preflight that preceded the M1 math cut. Its
implementation-blocked statements describe the captured preflight state and are superseded by
`2026-08-24-m1-zr-math-physical-hard-cut.md`. They must not be read as the current source status.
M1 milestone acceptance, compile-time improvement, runtime improvement, and energy improvement are
still not claimed.

## Post-preflight disposition

After this snapshot, the coordinator transferred the exact math/manifests/lock/docs scope to the
Frameworks01 r9 owner. The foreign Frameworks05 lock additions were preserved, the schema partition
was implemented as reviewed, and the physical `zr_math` hard cut completed without a dual owner.
Managed focused validation then established the evidence and external blockers recorded in the
implementation record. The original admission gates below remain useful as an audit trail; they are
no longer open implementation blockers.

## Scope and current snapshot

The inspected math owner is the modified `zircon_runtime_interface/src/math.rs` plus its 12
nonignored untracked child files under `zircon_runtime_interface/src/math/`:

- files: 13;
- lines: 1,556;
- bytes: 46,405;
- tree SHA-256: `6640a92d7b4d982232ef2e5ecedae16de8f3acf636b96fde85599cea8408a8c6`;
- coordinator baseline epoch at the final ownership recheck: 416;
- all 13 blobs are currently unowned; the 12 child files report `attribution_missing`, and
  `math.rs` reports the same condition at current hash
  `4e96649d9ff286a125d38f26b09df044f2c291056c8968acf4e05ac22a839976`.

A lexical current-worktree inventory for
`crate::core::math`, `zircon_runtime::core::math`, `zircon_runtime_interface::math`, and
`core::math::` found 1,356 matching lines in 1,102 files: 1,343 lines / 1,090 files are tracked,
and 13 lines / 12 files are nonignored untracked input. This is a preflight impact estimate, not the
atomic migration manifest. Before implementation, the owner must regenerate the union of literal
matches and a structured Rust use-tree inventory so nested imports and generated/test consumers are
not lost.

At snapshot time, the physical crate set under `zircon_runtime/crates/` contained only `zr_rhi` and
`zr_rhi_wgpu`; no `zr_math` directory or manifest existed. The root `Cargo.lock` also had a foreign
10-line Frameworks05 change, so it was not absorbed into the preflight.

## Dependency review and fatal cutover issue

The current math tree is not one dependency-neutral unit:

1. `fallible.rs` depends on `DepthDirection` plus the primitive matrix/scalar and numeric policy.
2. `numeric_policy.rs` and `transform.rs` depend on `Axis3`.
3. `space.rs` depends on `SpaceKind`, `NumericPolicy`, and `UnitDirection3`.
4. `render_conversion.rs` depends only on primitive runtime/render math aliases.
5. `schema.rs` mixes two different responsibilities:
   - pure math vocabulary such as axes, coordinate handedness, spaces, units, depth direction, and
     scalar precision;
   - versioned ABI documents (`CoordinateSchema`, `UnitSchema`, and `PrecisionProfile`) whose
     identity is `crate::serialization::SchemaId`.

Moving `schema.rs` wholesale into `zr_math` would create the forbidden reverse edge
`zr_math -> zircon_runtime_interface`. Moving `SchemaId` into math would instead make the
serialization-wide ABI identity a math concern and create a second owner. Both options violate the
M1 layer order and the single-owner/hard-cut rules.

The three `SchemaId`-bearing schema DTOs currently have no production consumer outside the math
module itself. That makes this the correct point to partition ownership before they become an
installed cross-crate dependency.

## Reference-engine findings

- Unreal keeps its fundamental vector/matrix/transform vocabulary in Runtime Core's public Math
  owner. `MathFwd.h` exposes stable template declarations and aliases, while implementation remains
  in the same foundational owner. The relevant lesson is one foundational implementation owner
  with stable projections, not a serialization layer below math.
- Bevy's `bevy_math` is an independent `#![no_std]` crate over `glam` and low-level utilities. Its
  serialization and reflection edges are feature-gated, and its prelude is a curated projection.
  The relevant lesson is a low-dependency math crate that does not depend on the engine facade.

Zircon therefore follows the shared structural principle while preserving its three-package product
surface: `zr_math` is the private canonical implementation owner; Runtime and Runtime Interface
remain approved product projections, not compatibility implementations.

## Locked target partition

### `zr_math` canonical owner

Move the following implementation and definitions into `zr_math` as one reviewed hard-cut batch:

- primitive aliases and render aliases backed by `glam`;
- `fallible`, `numeric_policy`, `render_conversion`, `space`, and `transform`;
- the pure convention vocabulary currently mixed into `schema.rs`: `Axis3`, `AxisDirection`,
  `CoordinateHandedness`, `MatrixConvention`, `ClipDepthRange`, `DepthDirection`,
  `FrontFaceWinding`, `SpaceKind`, `LengthUnit`, `AngleUnit`, `TimeUnit`, and `ScalarPrecision`;
- the owned tests for those algorithms and value types.

`zr_math` may depend on `glam`, `thiserror`, and feature-gated `serde`; it must not depend on
`zircon_runtime`, `zircon_runtime_interface`, `zr_contracts`, or any higher runtime crate. Initial M1
does not require claiming `no_std` acceptance, but the crate boundary must not add an avoidable
engine/facade dependency that prevents later `no_std` use.

### Runtime Interface ABI projection

Keep `CoordinateSchema`, `UnitSchema`, `PrecisionProfile`, their three canonical constants, and
their `SchemaId` ownership in `zircon_runtime_interface`. These DTOs import and serialize the pure
enums from `zr_math`; they do not duplicate those enums or any math algorithm.

`zircon_runtime_interface::math` re-exports the approved `zr_math` surface and its local versioned
schema DTOs. `zircon_runtime::core::math` remains the approved product facade projection. These two
projections are required architecture, not migration aliases. App, Editor, and plugins continue to
use the stable product paths and must not depend directly on `zr_math`.

### Internal consumers

Runtime-internal member crates use `zr_math` directly. The atomic consumer manifest must distinguish
those implementation edges from product consumers so the cut does not replace one facade reverse
dependency with another. The same batch deletes the old Interface implementation files after all
imports, tests, API guards, docs, examples, manifests, and the lock file are updated; no copied
implementation, forwarding module, old owner, or dual-path period is permitted.

## Atomic implementation admission gate

Physical work may start only after all of the following are true:

1. A live Frameworks01 M1 owner has an immutable scope covering the 13 current math blobs, the new
   `zr_math` tree, root/Runtime/Interface manifests, `Cargo.lock`, Runtime and Interface roots,
   every structured consumer, guards, docs, and examples.
2. The 13 current math blobs are attributed or transferred at the exact current hashes. An owner
   must not silently adopt the present unowned edits.
3. The Frameworks05 root-lock change is accepted or transferred; the math owner must not regenerate
   or overwrite a foreign `Cargo.lock` blob.
4. A fresh tree fingerprint and literal-plus-structured consumer manifest match the implementation
   input immediately before editing.
5. The schema split is reviewed as an ABI/DAG decision before source movement. `SchemaId` remains in
   Runtime Interface and no duplicate math vocabulary is admitted.
6. Managed Windows validation is available on an approved D/E/F target. Required M1 checks remain
   the parent plan's Runtime/App/Editor checks, focused math tests, facade/public-API guards, and the
   plugin workspace check.

M0 cold/incremental Cargo timing evidence is still absent. Consequently no compile-time target,
runtime speedup, bottleneck removal, energy reduction, or parity with Unreal/Bevy is claimed here.
The implementation owner must capture the pre-cut timings before using build performance as an
acceptance result.

## Resource-I/O cross-session conclusion

UI12 reported three E0432 failures for `crate::core::resource::io::atomic_write`. This is not a
Shader06 consumer migration:

- current HEAD `1538a67d526d4c8dff93aa96e189751c06f80ad6` and UI12's recorded base
  `4cc196150` both contain `pub use atomic_file::atomic_write` in
  `core/resource/io/mod.rs`;
- Frameworks01 M1 explicitly locks `io::atomic_write` as the curated publication entry for IBL and
  other upper-layer consumers;
- current source also uses the same entry from artifact storage, project preview, and graphics
  pipeline-cache code;
- UI12's reported lines are not its recorded base/current blobs: asset-derived currently imports
  `atomic_write` with `sync_parent_directory` at line 14, while source-cubemap staging imports the
  durable transaction surface at line 16.

The E0432 fingerprint therefore came from a mixed/stale validation materialization rather than an
invalid product API. Frameworks01 must keep the public facade, UI12 must rematerialize a coherent
current snapshot, and Shader06 must not rewrite the foreign IBL blobs to hide the snapshot defect.

## Validation performed

- read-only current-source module dependency review;
- current tree line/byte count and SHA-256 inventory;
- current coordinator ownership matrices for math, Interface/Runtime roots, and resource I/O;
- tracked plus nonignored-untracked lexical consumer inventory;
- Unreal Core Math and Bevy math crate source/manifests review;
- UI12 base/current `git show`, current imports, history, and blame comparison.

No Cargo command was started for this record. Source migration, managed build/test, independent
review, service commit, milestone acceptance, and WeCom notification all remain pending.
