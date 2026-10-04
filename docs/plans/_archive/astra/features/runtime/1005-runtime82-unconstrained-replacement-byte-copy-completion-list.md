---
related_code:
  - zircon_runtime/src/ui/surface/input/text_constraints.rs
plan_sources:
  - docs/plans/optimize/zircon_runtime/82/2026-09-27-unconstrained-replacement-byte-copy.md
tests:
  - zircon_runtime/src/ui/surface/input/text_constraints/optimization_tests.rs
doc_type: milestone-detail
status: implemented_pending_validation
---

# Runtime82 unconstrained replacement byte copy completion list

| Work | Source evidence | Remaining acceptance |
|---|---|---|
| Copy unconstrained replacement bytes directly. | The early return requires no grapheme limit, `Any` filter, multiline, and no boundary map. The owned string and empty receipt contract remain. | Grouped managed compile and five behavior regressions pending. |
| Preserve constrained and IME paths. | Tests cover grapheme authority, filter receipts, CRLF removal, Unicode cursor/clause mapping, and empty preedit. | Managed behavior execution pending. |
| Compare the actual production path with the full prior scan. | ASCII and Unicode million-grapheme inputs; five warmup pairs and 31 alternating measured pairs; output checks outside timing; raw samples and p50/p95/p99. | Local new p95 <= 80% baseline gate pending for both workloads. Runtime82/RTE native latency and product memory gates remain open. |
