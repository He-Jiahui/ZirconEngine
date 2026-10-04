---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/82/2026-08-27-secure-text-event-projection.md
  - docs/plans/optimize/zircon_runtime/81/2026-08-27-paragraph-analysis-construction-profile.md
  - docs/plans/optimize/zircon_runtime/82/2026-09-14-secure-text-presentation-capacity.md
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/ui/text/presentation.rs
  - zircon_runtime/src/ui/text/presentation_capacity_tests.rs
tests:
  - zircon_runtime/src/ui/text/presentation_capacity_tests.rs
  - tools/tests/test_runtime_secure_text_presentation_capacity_performance_contract.py
---

# Runtime754 · Secure-text presentation bounded capacity

`UiSecureTextPresentation::new` now reuses the materialized hard-line list and reserves a
mask-safe display-string bound, the known outer cluster/line bounds, and each hard line's
grapheme upper bound before projecting mask clusters and bidi ranges. Mask glyphs, separators,
source/display offsets, bidi signatures, and fail-closed behavior remain unchanged. This is a
narrow allocation improvement under the Runtime82 secure-text projection plan; it does not close
the secure trusted-session or product latency gates.

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime82 / secure-text presentation | Reserve a mask-safe display bound plus hard-line, cluster, and per-line logical-range bounds while consuming one grapheme iterator per line. | TDD RED/GREEN source contract `4/4`; lower mask/separator/source-length/overflow regression; ignored `RUNTIME754_SECURE_TEXT_PRESENTATION_CAPACITY_BENCH_V1` marker; adjacent Runtime/Editor capacity batch `29/29` in `0.513s`; merged Runtime/Editor source-contract batch `1852/1852` across `517` modules in `159.897s`. | implemented_pending_validation |

## 性能边界

The deterministic model removes geometric growth from the display string and the three retained
vector families for representable source sizes. Managed Cargo, Release allocation, and
secure-text p50/p95/p99 evidence remain required.

## 源码指纹

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/text/presentation.rs` | `DFA49848890E13DA79CAABB0F8B913D4BB8A6A760305F23130CF416075735C91` |
| `zircon_runtime/src/ui/text/presentation_capacity_tests.rs` | `4AC0664D0093A761D19D82A7C8CF2529A7A60FD52E554B548B7C3816F5785358` |
| `tools/tests/test_runtime_secure_text_presentation_capacity_performance_contract.py` | `15EE6BA9F4ED9287EBD912EB83A11F6CFA36B677D2D24BE6FE795A5DD4E22B2E` |

## 受管验证

This feature joins the existing owner-attributed Runtime/Editor Windows Release lane. No
standalone Cargo process or coordinator status query was made. Keep the record
`implemented_pending_validation` until current-source compile, secure semantic parity,
allocation behavior, and percentile evidence arrive; tooling production changes remain deferred.
