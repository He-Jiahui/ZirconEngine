# Runtime170 Compiled Sequence Channel Sampling Hard Cut

- Date: 2026-09-01
- Session: `root-runtime170-compiled-sequence-sampling-20260901`
- Parent plan: `docs/plans/optimize/zircon_runtime/170-runtime-animation-current-working-tree-source-compiled-pose-skinning-ik-root-motion-event-editor-boundary-review.md`
- Finding: `RT-AN-03`
- Implementation status: `source_complete`
- Managed validation: `pending`
- Milestone acceptance: `pending`

## Structural decision

Runtime sequence evaluation now consumes one canonical compiler product. Authored
`AnimationSequenceAsset` data is validated and lowered outside the frame path, then a world-bound
projection retains that immutable IR and compiled property writers. The frame path no longer
receives raw assets and cannot silently fall back to raw-channel validation.

This follows the local Unreal reference ownership model: authored animation is lowered into a
runtime evaluation product, and pose/property evaluation consumes that product rather than
rescanning authoring data. This milestone does not claim that Zircon's representation is yet a
compressed platform codec.

## Completed source work

- `AnimationCompiledSequenceTrackSampleExt` samples validated compiled keys with boundary checks
  and `partition_point`; it does not rescan key finiteness.
- `AnimationSequenceCompilation::into_artifact` transfers the validated source IR to the
  world-bound compiler; `compile_sequence_for_world` takes ownership and moves it into its
  `Arc`, removing the compile-boundary IR clone.
- The source IR stores its saturating validated track count once at compile admission; the
  world-bound compiler reads that metadata and reserves the exact success-track upper bound
  without a second binding traversal.
- Raw `AnimationChannelAsset::sample` retains its full finite-key fail-closed check.
- The linear and Hermite kernels accept a common borrowed key view, preserving value and tangent
  semantics without duplicating algorithms.
- World `CompiledAnimationSequence` owns an `Arc<AnimationCompiledSequence>` and stable binding/
  track indices together with compiled scene property writers.
- `compile_sequence_for_world` accepts only compiled source IR.
- `apply_compiled_sequence_to_world` no longer accepts `AnimationSequenceAsset`.
- The animation plugin compiles source IR only on asset-revision or world-projection invalidation.
- Focused tests cover raw non-finite rejection, compiled source ownership, exact-key Step behavior,
  source-track-count metadata, absence of compiled validation scans, and logarithmic interval work
  at 16,384 keys.

## Algorithm and performance evidence

For `T` sampled tracks with `K` keys each, redundant runtime key validation changed from
`Theta(T * K)` to zero after compile admission; interval lookup remains `Theta(T * log K)`.

The pre-implementation isolated E-drive model emitted
`RUNTIME07_ANIMATION_CHANNEL_SCAN_MODEL_V1`:

| Keys | Raw validation + lookup | Compiled lookup | Speedup |
| ---: | ---: | ---: | ---: |
| 64 | 70 ns/sample | 15 ns/sample | 4.73x |
| 1,024 | 771 ns/sample | 24 ns/sample | 30.94x |
| 16,384 | 11,298 ns/sample | 34 ns/sample | 324.99x |

The current-source ignored release benchmark emits
`RUNTIME170_COMPILED_CHANNEL_SAMPLE_V1` for 16,384 scalar keys, alternating raw/compiled order
across 101 samples and reporting the raw series for independent nearest-rank P50/P95
recomputation. The compiled P95 must beat the raw P95 by 4x. The deterministic work test
separately caps compiled key-time visits at `ilog2(K) + 8`, so acceptance does not rely on
wall-clock timing alone.

### 2026-09-18 evidence refresh

The Release probe was strengthened from 17 samples to 101 samples. Runtime170 now computes P50
and P95 with an explicit nearest-rank index instead of treating the final sample as P95, and the
marker includes both sorted raw/compiled series. This changes evidence robustness only; it does
not claim a managed timing result or alter the compiled sampling algorithm. The current source
contract is GREEN at `5/5`; Rustfmt and scoped diff checks are also green.

Sampling returns a fixed-size channel value and performs no heap allocation for scalar values.
This milestone does not claim measured process power, copied-byte totals, or end-to-end skeleton
cost: those require the managed current-source run and the separate compiled clip-pose milestone.

The world projection also starts its successful-track output at the validated source-track bound.
The bound is computed once in the source IR, so the world compiler avoids both geometric output
growth and a second counting traversal for a fully resolvable sequence. The source product pays one
`usize` of metadata per sequence. Missing-track diagnostics remain lazy: a failed entity/property
lookup still preserves its report-only path and does not allocate the success output. This is
deterministic allocation-shape evidence, not a Release timing or product-memory result.

## Frozen source manifest

| Path | SHA-256 |
| --- | --- |
| `zircon_runtime/src/animation/sequence/channel_sample.rs` | `511F211725503A552DD94188A47A42FD63909C411155E2B6B1DC15416DF17456` |
| `zircon_runtime/src/animation/sequence/interpolation.rs` | `1EC166EDAB46B160C4AB1ACAB03B0D247070F37D552528A94A99C9F360D6A218` |
| `zircon_runtime/src/animation/sequence/compiled.rs` | `A6FACD330D789E8910A2EF866B2A40C1F316FA7003451C57AD25FEA75900D0E5` |
| `zircon_runtime/src/animation/sequence/tests.rs` | `2A668FE14E5288CCB0D735760FAB8FFFBC92519092663C3962B452B40657969B` |
| `zircon_runtime/src/core/framework/animation/compiler/sequence/model.rs` | `03D14AE969E528C9E787502DD4530AFC3F0D7238CFED2F01FABC0E0BF7FD1DF1` |
| `zircon_plugins/animation/runtime/src/evaluation/pipeline/sequences.rs` | `BFDE820CDBC95BE3088A214D7BC369B7EEE992EED87B3FC46991B22F8E3D1746` |

## Validation receipts

- `rustfmt --edition 2024 --check` on the frozen Runtime source files and
  `rustfmt --edition 2021 --check` on the animation plugin pipeline: green.
- `git diff --check` on the owned source and plan paths: green before this record update.
- The ownership-transfer and bounded world-track-output source contract is green at `5/5`;
  scoped Rustfmt and diff checks pass for the ownership API, world compiler, plugin pipeline,
  and sequence tests.
- The current post-capacity local batch ran Runtime170, Editor641, and four adjacent Runtime/Editor
  source suites in one process: `45` tests passed in `94.665s` with zero failures or errors. Python
  compilation, both exact-file Rustfmt editions, scoped diff checks, and wiki validation also pass.
- Managed behavior request:
  `runtime170-compiled-sequence-sampling-behavior-20260901-r1`; accepted, recovery request
  `32520719c3354c44a17e433198888727`, result pending.
- Managed Windows release request:
  `runtime170-compiled-sequence-sampling-release-20260901-r1`; submitted once, reconciliation
  pending.

No direct Cargo build or test was run, no green managed result is inferred, and no commit/WeCom
milestone closeout is authorized yet.

## Remaining work

- Accept current-source managed behavior and release evidence for this frozen manifest.
- Establish legal ownership for clip pose manager/compiler files and hard-cut translation,
  rotation, and scale sampling to one compiled clip product.
- Profile representative skeleton matrices with allocations, copied bytes, P50, P95, and frame
  budget contribution before evaluating a sequential cursor or compression codec.
- Measure process/package power only in a controlled end-to-end runtime profile; do not infer it
  from this microbenchmark.

## Acceptance checklist

- [x] Architecture and local Unreal reference review recorded before production mutation.
- [x] Raw and compiled sampling authorities separated without a parallel validity cache.
- [x] Sequence world binding and apply API hard-cut to compiled source IR.
- [x] Deterministic complexity regression and malformed-raw regression added.
- [x] Current-source release benchmark added with a stable marker and threshold.
- [ ] Managed Windows behavior suite accepted for the frozen manifest.
- [ ] Managed Windows release benchmark accepted and quantified.
- [ ] Coordinator milestone commit and one-shot WeCom receipt emitted.
