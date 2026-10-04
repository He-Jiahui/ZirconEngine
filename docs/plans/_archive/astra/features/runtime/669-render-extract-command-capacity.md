---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11c-gpu-ui-renderer-atlas-sdf-batch-clip-submit-review.md
  - docs/plans/optimize/zircon_editor/01/2026-08-26-ui-architecture-audit.md
related_code:
  - zircon_runtime/src/ui/surface/render/extract.rs
tests:
  - zircon_runtime/src/ui/tests/render_atoms.rs
---

# Render Extract Command Capacity

The retained UI render extractor now reserves one command slot per arranged draw-order node
before collecting owner and component-specific commands. A node may still emit additional
commands and grow the vector as needed, but the common lower bound no longer starts from zero.
Command order, visibility admission, specialized painter output, and text prewarm behavior are
unchanged.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Runtime11C / Editor01 render extraction | Reserve the arranged draw-order lower bound for command collection | implemented_pending_validation | Render source regression asserts the draw-order capacity bound and existing visibility-order contract. Production Rustfmt and scoped diff checks pass; the batched Runtime/Editor static contracts pass `82/82`. Managed Runtime/Editor Cargo and release command-allocation/p50/p95/p99 evidence remain pending; no coordinator state was polled. |

## Source snapshot

| File | SHA-256 |
|---|---|
| `zircon_runtime/src/ui/surface/render/extract.rs` | `C237B4C961719B9EB89B4C10FC2510A040B0AC78BF2EA8586F7E49860EDEC61C` |
| `zircon_runtime/src/ui/tests/render_atoms.rs` | `89D3B92F4786F8703F0910F3386AA51D2611452A9BDD6C13F940986B06C58BB2` |
