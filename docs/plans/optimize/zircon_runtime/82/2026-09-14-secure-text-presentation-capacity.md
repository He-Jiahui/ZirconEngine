---
title: Runtime82 secure-text presentation bounded projection capacity
category: zircon_runtime
report_id: Runtime754-secure-text-presentation-capacity-2026-09-14
date: 2026-09-14
session_id: root-astra-optimize-20260914
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime82 secure-text presentation bounded projection capacity

## Scope

`UiSecureTextPresentation::new` already had to walk every canonical hard line and grapheme to
preserve masking, source offsets, separators, and original UAX#9 ordering. Its three retained
projection vectors nevertheless started empty, and each hard line's logical-range vector also
paid geometric growth. This is a bounded allocation follow-up to the Runtime82 secure-text event
projection and Runtime81 text-construction plans; it does not change the secure owner, mask
policy, bidi authority, or public presentation shape.

## Implementation

- Materialize `hard_lines(source_text)` once and reuse its exact length for the retained line
  vector instead of rebuilding the iterator after allocation decisions.
- Reserve a mask-safe display-string upper bound (`source bytes × MASK_GLYPH UTF-8 width`) and
  the source-byte upper bound for clusters before the hard-line loop. Every source
  grapheme and retained separator contributes at most one cluster, so the bound is conservative
  and preserves all existing separator/source ranges.
- Keep one `grapheme_indices(true)` iterator per hard line, use its upper `size_hint()` for the
  logical-range vector, and then consume that iterator for the existing mask and range projection.
- Leave display masking, bidi signature construction, line/cluster ranges, and fail-closed error
  behavior unchanged. The checked multiplication falls back to the source length only for an
  unrepresentable capacity, avoiding an overflow-driven allocation request.
- Add a lower semantic regression and ignored Release marker
  `RUNTIME754_SECURE_TEXT_PRESENTATION_CAPACITY_BENCH_V1` for the next owner-attributed batch.

## Deterministic pressure model

For a 64-line source, the legacy projection shape has one geometric-growth path for the display
string, one for the outer cluster vector, one for the line vector, and one per hard line for
logical ranges. The bounded model removes those known growth paths for representable inputs;
grapheme traversal and bidi work are unchanged. This is allocation-shape evidence only, not CPU,
RSS, power, or product p50/p95/p99 evidence.

## TDD and local evidence

- The source contract was observed RED before the reservations were wired, then GREEN at `4/4`.
- The lower Rust regression covers mask cardinality, hard-line separators, source-length, and
  overflow-safe display-capacity behavior; the ignored Release marker is ready for the shared
  managed lane.
- The seven-contract adjacent Runtime/Editor capacity batch passes `29/29` in `0.513s` after the
  implementation. Python compilation covers all seven contracts and scoped Rustfmt covers the
  touched Runtime/Editor Rust set. These are local source/model receipts.
- The final one-process merged Runtime/Editor performance-contract loader includes Editor762's
  finite-lane tightening and this Runtime754 contract, covers `517` modules, and passes
  `1852/1852` in `159.897s`. The adjacent seven-contract capacity batch passes `29/29` in
  `0.513s`; these are local source/model receipts.

## Source fingerprints

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/text/presentation.rs` | `DFA49848890E13DA79CAABB0F8B913D4BB8A6A760305F23130CF416075735C91` |
| `zircon_runtime/src/ui/text/presentation_capacity_tests.rs` | `4AC0664D0093A761D19D82A7C8CF2529A7A60FD52E554B548B7C3816F5785358` |
| `tools/tests/test_runtime_secure_text_presentation_capacity_performance_contract.py` | `15EE6BA9F4ED9287EBD912EB83A11F6CFA36B677D2D24BE6FE795A5DD4E22B2E` |

## Managed acceptance gate

Keep this slice at `managed_validation_pending` until the owner-attributed batched Windows
Release lane proves current-source compilation, mask/source/bidi parity, allocation behavior, and
secure-text construction p50/p95/p99. No standalone Cargo command or coordinator status query is
used here; tooling production work remains deferred by request.
