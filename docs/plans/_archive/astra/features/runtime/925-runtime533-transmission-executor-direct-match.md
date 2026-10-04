---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-30-runtime533-transmission-executor-direct-match.md
implementation_files:
  - zircon_runtime/src/graphics/pipeline/declarations/advanced_pbr_pass_contract.rs
tests:
  - zircon_runtime/src/graphics/pipeline/declarations/advanced_pbr_pass_contract.rs
---

# Runtime925 Runtime533 transmission executor direct match

Transmission executor IDs now use direct exhaustive matching instead of the
previous repeated candidate scan. All stable IDs and the unsupported fallback
remain covered by the focused contract.

Marker `RUNTIME533_TRANSMISSION_EXECUTOR_DIRECT_MATCH_BENCH_V1` reports the
direct-match operation model. Managed Release validation remains pending.
