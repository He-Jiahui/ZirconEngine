---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/170/2026-09-01-compiled-sequence-channel-sampling-hard-cut.md
related_records:
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/animation/sequence/channel_sample.rs
  - zircon_runtime/src/animation/sequence/compiled.rs
  - zircon_runtime/src/core/framework/animation/compiler/sequence/model.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/sequences.rs
tests:
  - zircon_runtime/src/animation/sequence/tests.rs
  - tools/tests/test_runtime170_compiled_sequence_sampling_performance_contract.py
---

# Runtime170 · compiled sequence channel sampling evidence refresh

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime170 animation channel sampling | Preserve the compiled-only runtime sampling hard cut, transfer the validated IR by ownership into the world compiler, reserve its known successful-track projection bound, and strengthen the ignored Release probe from 17 samples to 101 samples. Compute nearest-rank P50/P95 explicitly and emit both sorted raw/compiled series for independent recomputation. | TDD RED→GREEN source contract `5/5`; Rustfmt and scoped diff checks pass; the existing logarithmic interval, semantic, and owned-source regressions remain unchanged. Managed Cargo/Release and product animation percentile evidence remain pending. | implemented_pending_validation |

## Implementation evidence

The production sampling path is unchanged by this evidence refresh: compiled tracks still borrow
validated keys, use `partition_point`, and avoid the raw finite-key scan. The Release marker now
uses `sample_count=101`, nearest-rank indices, and complete sorted sample series so a coordinator
can recompute the threshold instead of treating the maximum observation as P95.

The compiler boundary now exposes `AnimationSequenceCompilation::into_artifact`; the plugin moves
that owned IR into `compile_sequence_for_world`, which moves it into the retained `Arc` instead of
cloning the full sequence at the boundary. Existing world-binding and apply semantics are
unchanged.

The source IR now stores its saturating validated track count once; the lower sequence regression
asserts the resulting one-track metadata. Before compiling writers, the
world projection reads that metadata and reserves its successful-track output without a second
binding traversal. The source product pays one `usize` of metadata per sequence. Missing entity or
property diagnostics still allocate only when failures occur. This removes geometric growth on the
common fully resolvable projection without changing binding order or failure semantics.

## Source fingerprints

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/animation/sequence/channel_sample.rs` | `511F211725503A552DD94188A47A42FD63909C411155E2B6B1DC15416DF17456` |
| `zircon_runtime/src/animation/sequence/compiled.rs` | `A6FACD330D789E8910A2EF866B2A40C1F316FA7003451C57AD25FEA75900D0E5` |
| `zircon_runtime/src/core/framework/animation/compiler/sequence/model.rs` | `03D14AE969E528C9E787502DD4530AFC3F0D7238CFED2F01FABC0E0BF7FD1DF1` |
| `zircon_runtime/src/animation/sequence/tests.rs` | `2A668FE14E5288CCB0D735760FAB8FFFBC92519092663C3962B452B40657969B` |
| `zircon_plugins/animation/runtime/src/evaluation/pipeline/sequences.rs` | `BFDE820CDBC95BE3088A214D7BC369B7EEE992EED87B3FC46991B22F8E3D1746` |
| `tools/tests/test_runtime170_compiled_sequence_sampling_performance_contract.py` | `E3BFC9B3454B5BB0A15539F2D45C39D1750988AA884B749BCB5A01869066F617` |

The focused source contract passes `5/5`. The current post-capacity batched local validation ran
the Runtime170 and Editor641 contracts together with four adjacent Runtime/Editor contract suites:
`45` tests passed in `94.665s` with zero failures or errors. The bounded track-output assertion
also passes in the focused Runtime170/Editor641 batch (`7/7`). The one-process all-surface
`test_*performance_contract.py` batch previously passed `2443/2443` in `405.162s`
before the ownership and capacity assertions were added; the focused and 45-test receipts above
are the current post-change contract evidence. This is local static evidence only. No managed timing
or Cargo result is inferred, and tooling production remains out of scope.

The later non-tooling Runtime/Editor loader covers 57 modules and passes `203/203` in `0.171s`,
including this Runtime170 contract after the Runtime08c source-contract matcher repair. This
remains local source evidence; managed Cargo/Release and product percentile gates are unchanged.

## 性能与受管验证边界

The 101-sample marker improves statistical stability and auditability, but it does not establish
an engine-level p50/p95/p99 or power result. Keep this record `implemented_pending_validation`
until the asynchronous Windows behavior/release batch validates the frozen source and records the
compiled-vs-raw threshold on the same machine.
