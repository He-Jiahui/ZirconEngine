---
title: Runtime07 Animation Compiled-Channel Sampling Current-Source Review
date: 2026-09-01
status: sequence_hard_cut_implemented_clip_authority_review_profiled_managed_acceptance_pending
origin_plan: docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
---

# Runtime07 animation compiled-channel sampling current-source review

## Decision

The animation sampling bottleneck is structural rather than an interpolation detail. Raw
`AnimationChannelAsset::sample` correctly fails closed, but it validates every key time on every
sample before using `partition_point`. Pose evaluation repeats that `O(keys)` scan for translation,
rotation, and scale on every track and frame. World-bound sequence evaluation has a second split:
`CompiledAnimationSequence` compiles writers, then indexes back into the raw asset and samples the
raw channel.

The accepted direction is a two-stage hard cut:

1. Source compilation owns canonical, strictly increasing, finite channel keys and interpolation
   metadata. It is the only input admitted to the frame sampling path.
2. World compilation owns an `Arc` to that immutable source product plus stable compiled-track
   indices and `CompiledScenePropertyWriter`s. `apply_compiled_sequence_to_world` no longer accepts
   the raw `AnimationSequenceAsset`.
3. Compiled-channel sampling checks only the finite sample time and performs the boundary checks
   plus `partition_point`. Raw asset sampling retains its current full defensive validation for
   untrusted/editor callers.
4. Clip pose evaluation receives an equivalent compiled clip product. It resolves bone targets
   and validates translation/rotation/scale channels outside the frame loop; the manager samples
   compiled tracks directly.
5. The old raw-asset frame lane, a parallel validity cache, and a `validated: bool` flag on mutable
   authoring assets are deleted rather than retained as fallbacks.

This preserves the current exact-key Step behavior: an exact interior key samples the preceding
interval. It also preserves fail-closed behavior for malformed raw channels.

## Current-source evidence

| Path | Current SHA-256 | Finding |
| --- | --- | --- |
| `animation/sequence/channel_sample.rs` | `822419da0f5f16e4e1959ad601faeba92d03dd622ce743bdf9c274a27b685004` | Raw sampling retains the full defensive scan; compiled sampling admits only the immutable compiled channel and performs binary interval lookup. |
| `animation/sequence/compiled.rs` | `0962c8749c1537d24e31e61b7cacee8f8548d295c57ee14ba2b0e4901d7e64ee` | World binding now retains the immutable compiled sequence in an `Arc`; apply no longer accepts the raw asset. |
| `animation/manager/pose.rs` | `b72b78e81c1e56253b8c2ed62d37e94575a2f8315abc20031044f3f2709a8788` | Each clip track samples three raw channels per frame. |
| `core/framework/animation/compiler/sequence/compile.rs` | `483d205fb2ec67ba3e24c2a334ba1dffed7d820fa59c554547457b6b36b81df2` | Compilation already rejects non-finite, out-of-range, and non-increasing key times. |
| `core/framework/animation/compiler/sequence/model.rs` | `e2f566a02e282c3fd56056a0bc35e6666400bdbfb9eaa35e55483af3c2b4d089` | The immutable compiled track already owns canonical keys, but runtime sampling does not consume it. |

For `T` sampled tracks and `K` keys per track, the current validated-asset frame path is
`Theta(T * K)` before interpolation. The accepted compiled path is `Theta(T * log K)`. Sequential
playback can later add a cursor fast path, but only after the canonical compiled path exists and a
profile shows binary lookup remains material; a cursor must not become a second authority.

## Reference-engine alignment

The local Unreal source separates authored/raw animation from runtime evaluation data:

- `dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimSequence.cpp:1756` scopes and
  instruments `UAnimSequence::GetBonePose`.
- `AnimSequence.cpp:1766` acquires the compressed-data read scope before pose evaluation.
- `AnimSequence.cpp:1878-1881` evaluates the runtime compressed bone data through one
  `FAnimSequenceDecompressionContext` and `DecompressPose` call.
- `AnimSequence.cpp:1521-1526` resolves a compiled track index and delegates to the bone
  compression codec for a single-bone read.
- `dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimCompressionTypes.cpp:1641-1643`
  allocates, deserializes, and binds the runtime compressed data structure before sampling.

Zircon does not need to copy Unreal's compression format for this milestone. It must copy the
ownership rule: validate/lower/bind once, then evaluate the runtime product without rescanning the
authoring representation.

## Pre-implementation profile

An isolated optimized Rust model on the E drive reproduced the exact current algorithm
(`any(!finite) + partition_point`) against the accepted compiled-channel lookup. It used 17
alternating samples, `opt-level=3`, and cardinality-scaled iterations. Marker:
`RUNTIME07_ANIMATION_CHANNEL_SCAN_MODEL_V1`.

| keys/channel | iterations | current P50/sample | compiled P50/sample | speedup | redundant validation visits/sample |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 64 | 62,500 | 70 ns | 15 ns | 4.73x | 64 |
| 1,024 | 3,906 | 771 ns | 24 ns | 30.94x | 1,024 |
| 16,384 | 256 | 11,298 ns | 34 ns | 324.99x | 16,384 |

This model proves the complexity defect and establishes the optimization direction. It is not a
managed current-source Cargo result, end-to-end pose benchmark, or power measurement.

## Clip evaluator authority and pose ownership review

The clip lane has a broader ownership defect than the sequence key scan. The current source has
two raw manager evaluators plus an existing cached clip evaluator:

- `zircon_runtime/src/animation/manager/pose.rs` validates every bind transform, constructs track
  target maps, samples three raw channels per track, and clones every bone name into a new
  `AnimationPoseOutput` on each frame.
- `zircon_plugins/animation/runtime/src/manager/pose.rs` duplicates that evaluator with an adaptive
  target lookup. Its private raw channel sampler performs a linear interval scan.
- `zircon_plugins/animation/runtime/src/evaluation/clip_evaluator/sample.rs` already caches skeleton
  targets, bind pose, a pose pool, and `CompiledAnimationClip` by revision. It validates channels
  once and uses a binary sampler, so it is the correct authority to converge around.
- That evaluator is not yet a complete compiled-product boundary. `CompiledClipTrack` clones and
  retains raw `AnimationChannelAsset` values, its sampler and Hermite implementation are separate
  authorities, and converting the pooled pose to `AnimationPoseOutput` still allocates the output
  vector and clones one name `String` per bone before returning the pose buffer to the pool.

| Path | Current SHA-256 | Structural finding |
| --- | --- | --- |
| `zircon_runtime/src/animation/manager/pose.rs` | `b72b78e81c1e56253b8c2ed62d37e94575a2f8315abc20031044f3f2709a8788` | Runtime raw manager rebuilds target state and owned output metadata each frame. |
| `zircon_plugins/animation/runtime/src/manager/pose.rs` | `3c3212d05416945bbb830ba9c5b1f5a0e01cec24b278243fe08f000f1ae62cb8` | Plugin duplicates the raw manager authority. |
| `zircon_plugins/animation/runtime/src/channel_sampling/channel_sample.rs` | `62ad1611c11458b20ee4ff6d016a5e27ead2e00c1ea5cd5e89996759fc27a476` | Plugin raw sampler uses a linear interval scan. |
| `zircon_plugins/animation/runtime/src/evaluation/clip_evaluator/channel_sample.rs` | `232de31097de394ffff7fe87c56111809cc21584eff8643b59ad0f1d795399ca` | Cached evaluator has a second binary sampler over the raw channel type. |
| `zircon_plugins/animation/runtime/src/evaluation/clip_evaluator/sample.rs` | `1934fe49c5dd50f979a540ce6afd1870ca910f653aeeb12ee1d1eefb0dbf3136` | Revision caches are useful, but output conversion discards pose reuse and clones names. |
| `zircon_plugins/animation/runtime/src/evaluation/compiled_animation_clip/compile.rs` | `b31e1f17ae395147234b173d9478b6cbc600ae5e868f5255bba4c8687ae1769c` | Compile resolves targets but clones raw transform channels. |
| `zircon_plugins/animation/runtime/src/evaluation/compiled_clip_track.rs` | `0ae50566040160b0bd0c4a41c403e259bfe39b3073aa94e6e233693a4247f432` | The compiled track still owns three raw channel DTOs rather than canonical compiled channels. |
| `zircon_runtime/src/core/framework/animation/pose_output.rs` | `fca6dde9056d80e8544cc46aab5461aa8be7b5af178946ad93d7aaf6cb550a18` | Pose output couples immutable bone names with per-frame transforms in one owned vector. |

The hard-cut target is one compiler-owned generic `AnimationCompiledChannel` IR shared by sequence
and clip products. `CompiledClipTrack` owns three compiled channels and a dense resolved bone slot;
one revision-keyed evaluator service owns clip evaluation. The two raw manager frame paths, the
plugin raw sampler, the evaluator-local sampler, and duplicate Hermite implementations are deleted
after all callers move. Skeleton hierarchy and bone names are immutable shared metadata; a player
owns reusable or persistent transform storage, and snapshots shallow-share metadata instead of
cloning names. Raw asset sampling remains only an untrusted/editor fallback.

An isolated optimized Rust ownership model on the E drive measured the construction work separately
from the channel lookup already profiled above. It used 21 alternating samples and marker
`RUNTIME170_CLIP_POSE_OWNERSHIP_MODEL_V1`.

| bones | lane | allocations/frame | modeled bytes/frame | P50 | P95 |
| ---: | --- | ---: | ---: | ---: | ---: |
| 1 | raw manager | 3 | 188 B | 364 ns | 519 ns |
| 1 | cached evaluator output | 2 | 72 B | 140 ns | 167 ns |
| 1 | persistent pose frame | 0 | 0 B | 10 ns | 12 ns |
| 64 | raw manager | 66 | 7,824 B | 11,853 ns | 14,269 ns |
| 64 | cached evaluator output | 65 | 4,608 B | 5,695 ns | 13,464 ns |
| 64 | persistent pose frame | 0 | 0 B | 72 ns | 82 ns |
| 256 | raw manager | 258 | 31,248 B | 47,563 ns | 64,561 ns |
| 256 | cached evaluator output | 257 | 18,432 B | 23,476 ns | 30,506 ns |
| 256 | persistent pose frame | 0 | 0 B | 324 ns | 360 ns |

The byte counts describe the model's `PoseBone` representation, not exact current-source Rust
allocation totals. The model excludes channel sampling, blending/state-machine work, GPU work,
managed Cargo validation, and power. It does prove that the cached evaluator still has linear
per-bone allocation ownership and that pooling only its temporary transforms cannot close the hot
path while the public output owns copied metadata.

## Required implementation scope

- Generalize the sequence compiled-channel product into one compiler-owned
  `AnimationCompiledChannel` IR used by both sequence and clip compilation, preserving the existing
  interpolation and exact-key Step semantics.
- Keep world-bound `CompiledAnimationSequence` on the immutable compiled source product with no raw
  asset input at apply time.
- Make `CompiledClipTrack` own three generic compiled channels plus a dense resolved bone slot, and
  route all clip sampling through one revision-keyed evaluator service.
- Delete both duplicate raw manager frame paths and all plugin/evaluator-local sampling and Hermite
  authorities after their callers hard-cut to the evaluator.
- Split immutable skeleton hierarchy/name metadata from per-frame transforms. Reuse persistent
  player pose storage and make read snapshots shallow-share metadata without per-bone name clones.
- Keep raw `AnimationChannelAsset::sample` defensive and cover a non-finite raw key regression.
- Add deterministic work counters proving zero key-validation visits on compiled sampling and
  exactly logarithmic interval comparisons within a documented bound.
- Add ignored Windows release profiles for 1/64/1,024/16,384 keys and a representative skeleton
  matrix. Report allocations, copied bytes, P50, P95, tracks, and channels sampled.

Runtime170 supersedes the older 08C currentness review and is the fixing plan for this slice.
Session `root-runtime170-compiled-sequence-sampling-20260901` now owns exact leases for the sequence
sampler, world projection, interpolation adapter, focused tests, plugin caller, and its plan output.
The source hard cut is implemented under that scope: world compilation accepts only
`AnimationCompiledSequence`, retains its immutable IR in an `Arc`, and apply no longer accepts or
reads `AnimationSequenceAsset`. The plugin compiles source IR only when its asset revision or world
projection becomes stale. Raw sampling remains defensive for untrusted/editor callers.

The compiler product currently crosses into the world projection with one deep clone at the
compile boundary because `AnimationSequenceCompilation` exposes only `artifact(&self)`. This clone
is not frame work, but a future ownership-returning compiler API should remove it. Clip-pose
evaluation remains a separate Runtime170 milestone because its manager/source files belong to a
different active lifecycle; this sequence milestone deliberately does not introduce a partial clip
cache or compatibility lane. The sequence-specific compiled channel is therefore an accepted
intermediate hard cut, not the final generic sequence/clip channel authority.

Managed current-source behavior request
`runtime170-compiled-sequence-sampling-behavior-20260901-r1` was accepted with recovery request
`32520719c3354c44a17e433198888727`. The release benchmark request
`runtime170-compiled-sequence-sampling-release-20260901-r1` was submitted once and remains in
coordinator reconciliation. Neither request is polled or treated as a green result. Static
`rustfmt --check` and `git diff --check` are green for the frozen source manifest.

## Acceptance checklist

- [x] Reviewed raw channel sampling, both hot callers, compiler validation, and compiled model.
- [x] Compared the ownership boundary with Unreal runtime compressed animation evaluation.
- [x] Recorded a cardinality profile proving the linear validation scan is the dominant operation.
- [x] Defined the single compiled-channel execution model and forbidden compatibility lanes.
- [x] Exact animation source/test ownership established without overwriting the active compiler lifecycle.
- [x] Deterministic current-source work counter proves logarithmic compiled interval lookup.
- [x] Sequence frame sampling hard-cut to the compiled source product.
- [x] Malformed raw regression and exact-key Step behavior encoded in focused tests.
- [x] Reviewed both raw clip managers, the cached evaluator, all three sampler authorities, and pose output ownership.
- [x] Recorded an isolated 1/64/256-bone allocation and elapsed-time model for raw, cached, and persistent pose ownership.
- [ ] Managed Windows behavior and release profile accepted.
- [ ] Generic compiled channel shared by sequence and clip products under a legal owner.
- [ ] Clip frame sampling hard-cut to the single revision-keyed evaluator.
- [ ] Immutable skeleton metadata and persistent per-player transform storage replace owned per-frame name copies.
- [ ] Duplicate raw manager, sampler, and Hermite authorities deleted after caller migration.
- [ ] Final current-source performance and power evidence recorded and returned to Runtime02.
