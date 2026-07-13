---
related_code:
  - zircon_runtime/src/core/framework/animation/asset/mod.rs
  - zircon_runtime/src/core/framework/animation/asset/binary.rs
  - zircon_runtime/src/core/framework/animation/asset/channel.rs
  - zircon_runtime/src/core/framework/animation/asset/clip.rs
  - zircon_runtime/src/core/framework/animation/asset/error.rs
  - zircon_runtime/src/core/framework/animation/asset/graph.rs
  - zircon_runtime/src/core/framework/animation/asset/reference.rs
  - zircon_runtime/src/core/framework/animation/asset/sequence.rs
  - zircon_runtime/src/core/framework/animation/asset/skeleton.rs
  - zircon_runtime/src/core/framework/animation/asset/state_machine.rs
  - zircon_runtime/src/core/framework/animation/asset/state_kind.rs
  - zircon_runtime/src/asset/importer/error.rs
  - zircon_runtime/src/asset/importer/ingest/import_animation_asset.rs
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/asset/tests/assets/animation.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_loaders.rs
  - zircon_plugins/animation/runtime/src/sequence.rs
  - zircon_plugins/animation/runtime/tests/animation_state_kind_asset_contract.rs
  - docs/engine-architecture/hard-cutover-migration-smells-m1.md
  - zircon_plugins/animation/runtime/tests/animation_state_kind_asset_contract.rs
implementation_files:
  - zircon_runtime/src/core/framework/animation/asset/mod.rs
  - zircon_runtime/src/core/framework/animation/asset/binary.rs
  - zircon_runtime/src/core/framework/animation/asset/channel.rs
  - zircon_runtime/src/core/framework/animation/asset/clip.rs
  - zircon_runtime/src/core/framework/animation/asset/error.rs
  - zircon_runtime/src/core/framework/animation/asset/graph.rs
  - zircon_runtime/src/core/framework/animation/asset/reference.rs
  - zircon_runtime/src/core/framework/animation/asset/sequence.rs
  - zircon_runtime/src/core/framework/animation/asset/skeleton.rs
  - zircon_runtime/src/core/framework/animation/asset/state_machine.rs
  - zircon_runtime/src/core/framework/animation/asset/state_kind.rs
  - zircon_runtime/src/asset/importer/error.rs
  - zircon_runtime/src/asset/importer/ingest/import_animation_asset.rs
  - docs/zircon_runtime/core/framework/animation-assets.md
  - docs/engine-architecture/hard-cutover-migration-smells-m1.md
plan_sources:
  - user: 2026-06-04 optimize Zircon Engine runtime architecture with breaking changes allowed
  - .codex/plans/Zircon Runtime 架构渐进式 Review 与优化计划.md
  - docs/plans/zircon_plugins/04-animation.md
tests:
  - rustfmt --edition 2021 --check zircon_runtime/src/core/framework/animation/asset/*.rs zircon_runtime/src/asset/importer/error.rs zircon_runtime/src/asset/importer/ingest/import_animation_asset.rs
  - static scan: animation asset binary owners contain no `Result<Self, String>`, `Result<Vec<u8>, String>`, `type Error = String`, `Err(format!)`, or `error.to_string()` rollback anchors
  - status/docs anchor scan for `runtime_15_animation_asset_binary_typed_errors_static_passed_cargo_deferred`
  - python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py --json
  - hard_cutover_migration_smells legacy-runtime-asset-debt count check
  - cargo +nightly test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_animation_runtime --test animation_state_kind_asset_contract --offline --jobs 1 (2026-07-11 Windows: 1/1 passed)
doc_type: module-detail
---

# Animation Asset

## Purpose

`zircon_runtime::core::framework::animation::asset` owns the neutral, versioned animation resource schemas for skeletons, clips, sequences, animation graphs, and state machines. The module serializes these records with the `ZRANIM01` binary envelope and exposes direct-reference helpers used by the concrete asset manager and animation runtime.

## Module Layout

The public module is now folder-backed. `mod.rs` is only the structural entry point and public re-export list; it must not accumulate payload behavior.

Payload owners are split by responsibility:

- `binary.rs` owns `ZRANIM01` document headers, binary kind routing, version validation, and the v1 fallback decoder entry point.
- `channel.rs` owns channel interpolation, channel values, channel keys, and binary channel value conversion.
- `error.rs` owns `AnimationAssetError` and `AnimationAssetResult`, including bincode source preservation, document/stream fallback reporting, v1 payload fallback reporting, reference parse errors, channel tag errors, and graph node tag errors.
- `skeleton.rs` owns skeleton and bone payloads.
- `clip.rs` owns clip tracks, event tracks, clip binary/v1 DTO conversion, and clip direct references.
- `sequence.rs` owns sequence track/binding DTO conversion and target path extraction.
- `graph.rs` owns graph node variants, graph binary/v1 DTO conversion, graph parameters, and graph direct references.
- `state_machine.rs` owns states, transitions, transition conditions, layer
  definitions, the transition exit-time/interruption fields,
  current/v3/v2/v1 state-machine binary conversion, and direct references.
- `state_kind.rs` owns graph-reference, clip, blend-space, and nested-machine
  state variants plus their direct-reference projection.
- `reference.rs` owns reusable direct-reference binary DTOs and de-duplication.

This split removed the former animation asset file from the 1000-line large-file hotspot list. The current Runtime 07 owner-budget mirror records `large_file_hotspot_count = 41`, `runtime-other = 16`, and `large_file_unclassified_hotspot_count = 0`; Cargo-level asset/runtime validation remains pending while active build lanes are occupied.

## Versioned Payloads

The current binary document version is `ANIMATION_BINARY_VERSION = 1`, but several payload shapes have already gained optional fields such as clip target ids, event tracks, sequence target ids, graph additive nodes, and mask nodes.

The module keeps explicit `*V1` payload structs for older binary payload shapes. These are versioned migration DTOs, not compatibility shims. `decode_binary_asset_with_v1_payload_fallback(...)` first decodes the current payload shape and then tries the matching `*V1` payload only to convert stored v1 bytes into the current runtime asset shape.

The v3/v2/v1 conversions fill added fields with current defaults:

- `AnimationClipBoneTrackAsset.target_id = None`
- `AnimationClipAsset.event_tracks = []`
- `AnimationSequenceBindingAsset.target_id = None`
- additive and mask graph nodes remain absent from v1 graph payloads
- `AnimationStateTransitionAsset.exit_time = None`
- `AnimationStateTransitionAsset.interruption = None`
- former graph-only states become `AnimationStateKindAsset::GraphRef`
- historical state machines receive `layers = []`

The state-machine v3 DTO preserves the prior StateKind payload without layers;
the v2 DTO converts the former direct `graph` state into the current `GraphRef`
state kind. Because bincode sequence payloads are positional,
optional state-transition
fields are always serialized even when their value is `None`; omitting a field
would shift the following interruption/condition bytes. The v1 DTO is the only
path that decodes the older transition layout without those fields.

Keeping the fallback named as v1 payload handling matters for the hard-cutover migration gate: old generic `legacy` naming would hide a versioned asset migration as general compatibility behavior.

## Public Surface

`AnimationSkeletonAsset`, `AnimationClipAsset`, `AnimationSequenceAsset`, `AnimationGraphAsset`, and `AnimationStateMachineAsset` expose `from_bytes(...)` and `to_bytes(...)` helpers. These helpers return `AnimationAssetResult<_>` rather than a lossy `Result<_, String>` so invalid magic/version/kind, bincode failures, v1 payload fallback failures, invalid references, channel tags, and graph node tags remain classifiable.

`AssetImportError::AnimationAsset` preserves the animation asset decode source at the importer boundary. Import code may still render a display message for user-facing diagnostics, but the runtime no longer discards `AnimationAssetError` when ingesting `.zranim` files.

Clip, graph, and state-machine assets also expose `direct_references()` so generic asset dependency readers can discover skeleton, clip, graph, nested state-machine, and all BlendSpace graph references without running animation playback. References are de-duplicated in stable state/sample order.

The module is a leaf framework schema owner. It does not load project files, schedule animation systems, mutate ECS state, own plugin lifecycle, or participate in editor projection logic. Concrete importing/caching remains in `asset`; runtime behavior remains in animation runtime systems and plugins. The retired `asset/assets/animation` owner and all asset-facade re-exports were deleted in the Frameworks05 hard cut.

## Test Coverage

`zircon_runtime/src/asset/tests/assets/animation.rs` covers binary roundtrips,
kind mismatch rejection, direct reference reporting, additive/mask graph
roundtrips, clip target ids and event tracks, and older stream-shaped payload
decoding. The v1 state-machine case also verifies default exit-time and
interruption conversion. The older-payload test fixtures still create
historical bytes, but production code names the migration path as v1 payload
conversion rather than a generic legacy compatibility path.

`zircon_plugins/animation/runtime/tests/animation_state_kind_asset_contract.rs`
adds a public-boundary roundtrip over Clip, BlendSpace1D, BlendSpace2D,
SubMachine, and GraphRef states and verifies the exact direct-reference order.

Runtime 15 F5 status: `Runtime 15 F5 animation asset binary typed errors` / `runtime_15_animation_asset_binary_typed_errors_static_passed_cargo_deferred` is recorded in the Runtime 15 plan, runtime index, engine review findings, structure convention, status-output expectations, and `review_f5_animation_asset_binary_uses_typed_errors`. Cargo validation is still deferred while other cargo/rustc build lanes are active; scoped static and formatting checks cover this slice.
