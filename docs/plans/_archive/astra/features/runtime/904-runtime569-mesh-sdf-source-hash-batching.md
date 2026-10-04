---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-31-runtime569-mesh-sdf-source-hash-batching.md
related_records:
  - docs/plans/astra/features/runtime/903-runtime568-animation-channel-projection-single-dispatch.md
  - docs/plans/astra/features/runtime/905-runtime570-material-override-bulk-sort.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/asset/assets/mesh/mesh_sdf/source_hash.rs
tests:
  - zircon_runtime/src/asset/assets/mesh/mesh_sdf/source_hash.rs
---

# Runtime904 Runtime569 Mesh-SDF Source Hash Batching

| 范围 | 修复 | 证据与门禁 |
| --- | --- | --- |
| Mesh-SDF source identity | Materialize canonical little-endian vertex/index bytes into fixed buffers and update BLAKE3 once per block, preserving schema, byte order, special float bits, and zero heap allocation. | Behavior contract covers empty input, 255/256/257 boundaries, partial tails, sign bits, and NaN payloads. |
| 性能门禁 | 16,384 vertices and 49,152 indices reduce hasher update calls from 98,313 to 265; managed gate requires at least 25% P95 improvement. | ignored marker `RUNTIME569_MESH_SDF_HASH_BATCH_BENCH_V1`; real managed BLAKE3 timing remains pending. |

- `source_hash.rs` contains the Runtime569 behavior/source contracts and marker.
- No tooling changes; standalone segmentation-invariant evidence is not treated as managed acceptance.

### Grouped validation submission (2026-09-25)

Runtime569 is included in the shared `20260831f` batch covering Runtime565–570:
Runtime development PTY `34078`, Editor development PTY `73116`, Runtime02
Release PTY `31967`, and Editor Release PTY `1779`. All wrappers remain
intentionally unpolled and managed compiler/P95 receipts are pending.
