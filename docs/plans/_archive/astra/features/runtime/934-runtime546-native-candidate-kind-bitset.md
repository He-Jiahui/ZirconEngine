---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-30-runtime546-native-candidate-kind-bitset.md
implementation_files:
  - zircon_runtime/src/plugin/native_plugin_loader/load_discovered.rs
tests:
  - zircon_runtime/src/plugin/native_plugin_loader/load_discovered.rs
---

# Runtime934 Runtime546 native candidate-kind bitset

Native plugin discovery now materializes one four-bit requested-kind mask per
load and shares it across direct and embedded candidate scans. Per-library entry
gating intentionally retains its measured slice path.

Marker `RUNTIME546_NATIVE_CANDIDATE_KIND_BITSET_BENCH_V1` reports the two-to-one
membership model. Managed Release validation remains pending.
